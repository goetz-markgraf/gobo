//! File-backed document model with UTF-8 validation, read-only detection,
//! and save-time conflict checks.

use ropey::Rope;
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Display, Formatter};
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccessMode {
    Editable,
    ReadOnly,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiskSnapshot {
    pub modified_at: Option<SystemTime>,
    pub size_bytes: Option<u64>,
    pub content_fingerprint: Option<u64>,
}

#[derive(Debug)]
pub enum DocumentError {
    Directory(PathBuf),
    InvalidUtf8(PathBuf),
    Io { path: PathBuf, source: io::Error },
}

impl Display for DocumentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Directory(path) => write!(f, "{} is a directory, not a file", path.display()),
            Self::InvalidUtf8(path) => write!(f, "{} is not valid UTF-8 text", path.display()),
            Self::Io { path, source } => write!(f, "{}: {}", path.display(), source),
        }
    }
}

impl std::error::Error for DocumentError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SaveResult {
    Saved,
    ConflictDetected,
    BlockedReadOnly,
}

#[derive(Debug)]
pub struct DocumentBuffer {
    pub path: PathBuf,
    pub text: Rope,
    pub access_mode: AccessMode,
    pub dirty: bool,
    pub exists_on_disk: bool,
    pub disk_snapshot: Option<DiskSnapshot>,
    pub last_saved_at: Option<SystemTime>,
}

impl DocumentBuffer {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, DocumentError> {
        let path = path.into();

        if path.is_dir() {
            return Err(DocumentError::Directory(path));
        }

        match fs::read(&path) {
            Ok(bytes) => {
                let text = String::from_utf8(bytes)
                    .map_err(|_| DocumentError::InvalidUtf8(path.clone()))?;
                let metadata = fs::metadata(&path).map_err(|source| DocumentError::Io {
                    path: path.clone(),
                    source,
                })?;
                let access_mode = if metadata.permissions().readonly() {
                    AccessMode::ReadOnly
                } else {
                    AccessMode::Editable
                };

                Ok(Self {
                    path: path.clone(),
                    text: Rope::from_str(&text),
                    access_mode,
                    dirty: false,
                    exists_on_disk: true,
                    disk_snapshot: snapshot_for_path(&path)?,
                    last_saved_at: None,
                })
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Self {
                path,
                text: Rope::new(),
                access_mode: AccessMode::Editable,
                dirty: false,
                exists_on_disk: false,
                disk_snapshot: None,
                last_saved_at: None,
            }),
            Err(source) => Err(DocumentError::Io { path, source }),
        }
    }

    pub fn len_chars(&self) -> usize {
        self.text.len_chars()
    }

    pub fn is_read_only(&self) -> bool {
        self.access_mode == AccessMode::ReadOnly
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn replace_contents(&mut self, text: &str) {
        self.text = Rope::from_str(text);
        self.mark_dirty();
    }

    pub fn reload_from_disk(&mut self) -> Result<(), DocumentError> {
        let reloaded = Self::open(self.path.clone())?;
        self.text = reloaded.text;
        self.access_mode = reloaded.access_mode;
        self.exists_on_disk = reloaded.exists_on_disk;
        self.disk_snapshot = reloaded.disk_snapshot;
        self.dirty = false;
        Ok(())
    }

    pub fn save(&mut self) -> Result<SaveResult, DocumentError> {
        if self.is_read_only() {
            return Ok(SaveResult::BlockedReadOnly);
        }

        if self.has_external_change()? {
            return Ok(SaveResult::ConflictDetected);
        }

        self.write_to_disk()?;
        Ok(SaveResult::Saved)
    }

    pub fn overwrite_save(&mut self) -> Result<SaveResult, DocumentError> {
        if self.is_read_only() {
            return Ok(SaveResult::BlockedReadOnly);
        }

        self.write_to_disk()?;
        Ok(SaveResult::Saved)
    }

    pub fn has_external_change(&self) -> Result<bool, DocumentError> {
        Ok(snapshot_for_path(&self.path)? != self.disk_snapshot)
    }

    /// Atomic file save: write to a temp file in the same directory, then
    /// rename into place.  This prevents corruption if the process crashes
    /// mid-write, because the old file remains intact until the rename.
    fn write_to_disk(&mut self) -> Result<(), DocumentError> {
        let mut output = String::new();
        for chunk in self.text.chunks() {
            output.push_str(chunk);
        }

        // Build a temp-path next to the target file in the same directory.
        let temp_path = Self::temp_path(&self.path);
        fs::write(&temp_path, &output).map_err(|source| DocumentError::Io {
            path: temp_path.clone(),
            source,
        })?;

        // Rename is atomic on the same filesystem (POSIX renameat2 / mv). If it
        // fails, remove the temp file so repeated save attempts don't leave
        // orphaned `.tmp` files littering the directory.
        if let Err(source) = std::fs::rename(&temp_path, &self.path) {
            let _ = std::fs::remove_file(&temp_path);
            return Err(DocumentError::Io {
                path: self.path.clone(),
                source,
            });
        }

        self.exists_on_disk = true;
        self.dirty = false;
        self.last_saved_at = Some(SystemTime::now());
        self.disk_snapshot = snapshot_for_path(&self.path)?;
        Ok(())
    }

    /// Returns a temp-file path in the same directory as `canonical`. */
    fn temp_path(canonical: &Path) -> PathBuf {
        let mut tmp = canonical.as_os_str().to_owned();
        tmp.push(".tmp");
        PathBuf::from(tmp)
    }
}

pub fn snapshot_for_path(path: &Path) -> Result<Option<DiskSnapshot>, DocumentError> {
    if !path.exists() {
        return Ok(None);
    }

    let metadata = fs::metadata(path).map_err(|source| DocumentError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let bytes = fs::read(path).map_err(|source| DocumentError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);

    Ok(Some(DiskSnapshot {
        modified_at: metadata.modified().ok(),
        size_bytes: Some(metadata.len()),
        content_fingerprint: Some(hasher.finish()),
    }))
}
