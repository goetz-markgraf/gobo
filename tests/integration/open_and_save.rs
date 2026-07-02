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
    session
        .handle_command(EditorCommand::InsertChar('X'))
        .unwrap();
    session.handle_command(EditorCommand::Save).unwrap();

    assert_eq!(fs::read_to_string(&path).unwrap(), "heXllo\n");
    assert!(!session.document.dirty);
}

#[test]
fn missing_file_is_created_on_first_save() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("new.txt");

    let mut session = EditingSession::open(&path, TerminalSize::new(80, 24)).unwrap();
    session
        .handle_command(EditorCommand::InsertChar('a'))
        .unwrap();
    session
        .handle_command(EditorCommand::InsertChar('b'))
        .unwrap();
    session.handle_command(EditorCommand::Save).unwrap();

    assert_eq!(fs::read_to_string(&path).unwrap(), "ab");
}

/// Spec 005 US1 scenario 4 & 5: the footer shows the filename right-aligned
/// with ` (*)` while the file is dirty, and drops the marker once saved.
#[test]
fn footer_dirty_marker_appears_on_edit_and_disappears_after_save() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("footer.txt");
    fs::write(&path, "seed\n").unwrap();

    let mut session = EditingSession::open(&path, TerminalSize::new(80, 24)).unwrap();

    // Clean file: footer shows the bare filename, no dirty marker.
    let view = session.render_view();
    assert!(view.footer_line.contains("footer.txt"));
    assert!(!view.footer_line.contains("(*)"));

    // Edit: the dirty marker kicks in alongside the filename.
    session
        .handle_command(EditorCommand::InsertChar('x'))
        .unwrap();
    let dirty_view = session.render_view();
    assert!(dirty_view.footer_line.contains("footer.txt"));
    assert!(dirty_view.footer_line.contains("(*)"));

    // Save: dirtiness clears and the marker disappears again.
    session.handle_command(EditorCommand::Save).unwrap();
    let saved_view = session.render_view();
    assert!(saved_view.footer_line.contains("footer.txt"));
    assert!(!saved_view.footer_line.contains("(*)"));
}

/// Spec 005 US1 scenario 2: a relative-style argument is shown verbatim on the left.
#[test]
fn footer_shows_relative_path_verbatim() {
    use gobo::editor::render::format_footer_line;
    let footer = format_footer_line("somedir/example.txt", false, "Ready", 80);
    assert!(footer.starts_with("somedir/example.txt"));
    assert!(footer.contains("Ready"));
}

#[test]
fn startup_failures_cover_directory_and_invalid_utf8() {
    let dir = tempdir().unwrap();
    let bad_file = dir.path().join("bad.bin");
    fs::write(&bad_file, [0xff, 0xfe, 0xfd]).unwrap();

    assert!(EditingSession::open(dir.path(), TerminalSize::new(80, 24)).is_err());
    assert!(DocumentBuffer::open(&bad_file).is_err());
}
