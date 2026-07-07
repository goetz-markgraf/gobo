//! System clipboard I/O for cut/copy/paste (spec 009, FR-008/FR-009/FR-013).
//!
//! Boundary: this module is the *only* place that talks to the OS clipboard
//! (`arboard`). The editor never keeps its own clipboard copy (FR-008): every
//! read/write is a transient call that returns a local [`String`] and touches
//! no editor state. Text-only acceptance is delegated to `arboard`'s
//! `get_text()`, which returns an error when the clipboard holds no text
//! (binary/image or cleared); see [`read_text`]'s `Option` contract (FR-009).
//!
//! 1 MB hard cap (FR-013/SC-005): enforced in both directions. [`write_text`]
//! rejects oversized writes up front. [`read_text`] returns the text
//! unconditionally and the caller checks [`fits_size_limit`] so it can show the
//! distinct "too large" warning rather than a silent no-op (contract §Paste).
//!
//! This module has no dependency on editor state (constitution II):
//! `app.rs` is the dispatch boundary, `input.rs` the key-binding table.

use arboard::Clipboard;

/// 1 MB ceiling on transferred clipboard text, in bytes (FR-013/SC-005).
/// `1 << 20 == 1_048_576`. Exactly 1 MB is accepted; one byte more is rejected.
pub const MAX_CLIPBOARD_BYTES: usize = 1 << 20;

/// `true` when `byte_len` fits the 1 MB clipboard cap (inclusive at exactly 1 MB).
/// Kept as a pure seam so the boundary value can be unit-tested (constitution IV).
pub fn fits_size_limit(byte_len: usize) -> bool {
    byte_len <= MAX_CLIPBOARD_BYTES
}

/// `true` iff `content` is valid UTF-8 (and therefore acceptable as clipboard
/// text). Mirrors `arboard`'s text-only selection as a testable pure predicate
/// for the data-parsing edge cases (FR-008/FR-009, edge "non-text-inhalt").
/// Production code relies on `arboard`'s `get_text()` returning
/// `Err(ContentNotAvailable)` for non-text content; this predicate exists as a
/// pure seam so the UTF-8 boundary can be unit-tested (constitution IV).
pub fn is_text_only(content: &[u8]) -> bool {
    std::str::from_utf8(content).is_ok()
}

/// Read the current OS clipboard text (FR-008/FR-009).
///
/// Returns:
/// - `Some(text)` for any text content, including an empty string and content
///   larger than 1 MB (the caller applies the size cap via [`fits_size_limit`] so
///   it can distinguish "too large" from "no text").
/// - `None` when the clipboard holds no text (binary/image/cleared) **or** the
///   OS clipboard could not be opened. `None` is the "silent no-op" case
///   (FR-009): the editor treats missing text exactly like an empty clipboard.
pub fn read_text() -> Option<String> {
    let mut clipboard = Clipboard::new().ok()?;
    match clipboard.get_text() {
        Ok(text) => Some(text),
        // No text content available (binary/image/cleared) — silent no-op.
        Err(arboard::Error::ContentNotAvailable) => None,
        // Any other clipboard error is treated defensively as "no text
        // available" → silent no-op (constitution III: fail safely).
        Err(_) => None,
    }
}

/// Write `text` to the OS clipboard (FR-008/FR-013).
///
/// Returns `Err(message)` when `text` exceeds the 1 MB cap (FR-013) **or** when
/// the OS rejects the write; the caller surfaces a warning. Never mutates
/// editor state (FR-008). The byte length (`str::len`) is the cap basis, so
/// multi-byte graphemes are counted in UTF-8 bytes, not characters.
pub fn write_text(text: &str) -> Result<(), String> {
    if !fits_size_limit(text.len()) {
        return Err("Clipboard content too large (>1 MB)".to_string());
    }
    let mut clipboard = Clipboard::new().map_err(|e| format!("Failed to open clipboard: {e}"))?;
    clipboard
        .set_text(text.to_string())
        .map_err(|e| format!("Failed to write clipboard: {e}"))
}
