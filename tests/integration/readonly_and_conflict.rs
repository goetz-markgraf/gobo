use gobo::app::{ConflictChoice, EditingSession, PromptState, SessionMode};
use gobo::document::AccessMode;
use gobo::editor::input::EditorCommand;
use gobo::editor::render::TerminalSize;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use tempfile::tempdir;

#[test]
fn readonly_file_blocks_edit_and_save() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("readonly.txt");
    fs::write(&path, "locked").unwrap();
    #[cfg(unix)]
    fs::set_permissions(&path, fs::Permissions::from_mode(0o444)).unwrap();

    let mut session = EditingSession::open(&path, TerminalSize::new(80, 24)).unwrap();
    session.handle_command(EditorCommand::InsertChar('!')).unwrap();
    session.handle_command(EditorCommand::Save).unwrap();

    assert_eq!(session.document.access_mode, AccessMode::ReadOnly);
    assert_eq!(session.document.text.to_string(), "locked");
    assert!(session.status.as_ref().unwrap().text.contains("Read-only"));
}

#[cfg(unix)]
#[test]
fn failed_save_keeps_previous_disk_content_intact() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("cannot-save.txt");
    fs::write(&path, "stable-content\n").unwrap();

    let mut session = EditingSession::open(&path, TerminalSize::new(80, 24)).unwrap();
    session.handle_command(EditorCommand::InsertChar('x')).unwrap();

    fs::set_permissions(&path, fs::Permissions::from_mode(0o444)).unwrap();
    let save_result = session.handle_command(EditorCommand::Save);
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();

    assert!(save_result.is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), "stable-content\n");
    assert!(session.document.dirty);
}

#[test]
fn external_change_prompts_for_reload_overwrite_or_cancel() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("conflict.txt");
    fs::write(&path, "base\n").unwrap();

    let mut session = EditingSession::open(&path, TerminalSize::new(80, 24)).unwrap();
    session.handle_command(EditorCommand::InsertChar('x')).unwrap();
    fs::write(&path, "changed elsewhere\n").unwrap();

    session.handle_command(EditorCommand::Save).unwrap();
    assert_eq!(session.mode, SessionMode::SaveConflictPrompt);
    assert!(matches!(
        session.pending_prompt,
        Some(PromptState::SaveConflict {
            focus: ConflictChoice::Cancel,
            ..
        })
    ));

    session.handle_command(EditorCommand::MoveRight).unwrap();
    session.handle_command(EditorCommand::MoveRight).unwrap();
    session.handle_command(EditorCommand::Confirm).unwrap();

    assert_eq!(fs::read_to_string(&path).unwrap(), "xbase\n");
}
