use gobo::app::{EditingSession, SessionMode};
use gobo::editor::input::EditorCommand;
use gobo::editor::render::TerminalSize;
use tempfile::tempdir;

fn dirty_session_with_size(
    path_name: &str,
    size: TerminalSize,
) -> (tempfile::TempDir, std::path::PathBuf, EditingSession) {
    let dir = tempdir().unwrap();
    let path = dir.path().join(path_name);
    std::fs::write(&path, "alpha\nbeta\ngamma\n").unwrap();
    let mut session = EditingSession::open(&path, size).unwrap();
    session.handle_command(EditorCommand::InsertChar('x')).unwrap();
    (dir, path, session)
}

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
fn quit_popup_takes_precedence_over_active_search() {
    let (_dir, _path, mut session) = dirty_session_with_size("search-precedence.txt", TerminalSize::new(80, 24));
    session.handle_command(EditorCommand::Search).unwrap();
    session.handle_command(EditorCommand::InsertChar('a')).unwrap();
    assert_eq!(session.mode, SessionMode::SearchInput);
    assert!(session.render_view().bottom_line.as_deref().unwrap().contains("Search:"));

    session.handle_command(EditorCommand::Quit).unwrap();

    assert_eq!(session.mode, SessionMode::ConfirmQuit);
    let view = session.render_view();
    assert!(view.popup.is_some());
    assert_eq!(view.bottom_line, None);
}

#[test]
fn resize_while_prompted_keeps_popup_visible_and_uses_compact_variant() {
    let (_dir, _path, mut session) = dirty_session_with_size("resize-prompt.txt", TerminalSize::new(80, 24));
    session.handle_command(EditorCommand::Quit).unwrap();

    let before = session.render_view().popup.expect("popup before resize");
    assert_eq!(before.variant, gobo::editor::render::PromptVariant::Full);

    session
        .handle_command(EditorCommand::Resize(TerminalSize::new(24, 6)))
        .unwrap();

    assert_eq!(session.mode, SessionMode::ConfirmQuit);
    let after = session.render_view().popup.expect("popup after resize");
    assert_eq!(after.variant, gobo::editor::render::PromptVariant::Compact);
    assert_eq!(after.actions[0].label, "[Save]");
    assert!(after.rect.width <= 24);
    assert!(after.rect.height <= 6);
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
