use gobo::app::EditingSession;
use gobo::document::DocumentBuffer;
use gobo::editor::input::EditorCommand;
use gobo::editor::render::TerminalSize;
use std::fs;
use tempfile::tempdir;

#[test]
fn opens_existing_utf8_file_edits_and_saves() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("note.txt");
    fs::write(&path, "hello\n").unwrap();

    let mut session = EditingSession::open(&path, TerminalSize::new(80, 24)).unwrap();
    session.handle_command(EditorCommand::MoveRight).unwrap();
    session.handle_command(EditorCommand::MoveRight).unwrap();
    session.handle_command(EditorCommand::InsertChar('X')).unwrap();
    session.handle_command(EditorCommand::Save).unwrap();

    assert_eq!(fs::read_to_string(&path).unwrap(), "heXllo\n");
    assert!(!session.document.dirty);
}

#[test]
fn missing_file_is_created_on_first_save() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("new.txt");

    let mut session = EditingSession::open(&path, TerminalSize::new(80, 24)).unwrap();
    session.handle_command(EditorCommand::InsertChar('a')).unwrap();
    session.handle_command(EditorCommand::InsertChar('b')).unwrap();
    session.handle_command(EditorCommand::Save).unwrap();

    assert_eq!(fs::read_to_string(&path).unwrap(), "ab");
}

#[test]
fn startup_failures_cover_directory_and_invalid_utf8() {
    let dir = tempdir().unwrap();
    let bad_file = dir.path().join("bad.bin");
    fs::write(&bad_file, [0xff, 0xfe, 0xfd]).unwrap();

    assert!(EditingSession::open(dir.path(), TerminalSize::new(80, 24)).is_err());
    assert!(DocumentBuffer::open(&bad_file).is_err());
}
