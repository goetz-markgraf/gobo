use gobo::app::EditingSession;
use gobo::editor::input::EditorCommand;
use gobo::editor::render::TerminalSize;
use tempfile::tempdir;

#[test]
fn search_is_case_insensitive_and_reports_no_match() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("search.txt");
    std::fs::write(&path, "First line\nTHIRD line\n").unwrap();

    let mut session = EditingSession::open(&path, TerminalSize::new(80, 24)).unwrap();
    session.handle_command(EditorCommand::Search).unwrap();
    for ch in "third".chars() {
        session.handle_command(EditorCommand::InsertChar(ch)).unwrap();
    }
    session.handle_command(EditorCommand::Confirm).unwrap();
    assert!(session.status.as_ref().unwrap().text.contains("Match found"));

    session.handle_command(EditorCommand::Search).unwrap();
    for ch in "missing".chars() {
        session.handle_command(EditorCommand::InsertChar(ch)).unwrap();
    }
    session.handle_command(EditorCommand::Confirm).unwrap();
    assert!(session.status.as_ref().unwrap().text.contains("No match"));
}

#[test]
fn resize_updates_viewport_and_render_output() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("resize.txt");
    std::fs::write(&path, "alpha\nbeta\ngamma\n").unwrap();

    let mut session = EditingSession::open(&path, TerminalSize::new(80, 24)).unwrap();
    session.handle_command(EditorCommand::Resize(TerminalSize::new(20, 5))).unwrap();
    let view = session.render_view();

    assert_eq!(session.viewport.visible_width, 20);
    assert_eq!(session.viewport.visible_height, 4);
    assert_eq!(view.body_lines.len(), 4);
}

#[test]
fn empty_and_very_long_lines_render_without_crashing() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("long-lines.txt");
    let long_line = "x".repeat(5000);
    std::fs::write(&path, format!("\n{}\n", long_line)).unwrap();

    let session = EditingSession::open(&path, TerminalSize::new(20, 5)).unwrap();
    let view = session.render_view();

    assert_eq!(view.body_lines.len(), 4);
    assert_eq!(view.body_lines[0], "");
    assert_eq!(view.body_lines[1].len(), 20);
}
