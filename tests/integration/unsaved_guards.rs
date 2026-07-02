use gobo::app::EditingSession;
use gobo::app::{PromptState, SessionMode, UnsavedChoice};
use gobo::editor::input::EditorCommand;
use gobo::editor::render::{PromptVariant, TerminalSize};
use std::fs;
use tempfile::tempdir;

fn dirty_session(path_name: &str) -> (tempfile::TempDir, std::path::PathBuf, EditingSession) {
    let dir = tempdir().unwrap();
    let path = dir.path().join(path_name);
    fs::write(&path, "seed\n").unwrap();
    let mut session = EditingSession::open(&path, TerminalSize::new(80, 24)).unwrap();
    session
        .handle_command(EditorCommand::InsertChar('x'))
        .unwrap();
    (dir, path, session)
}

#[test]
fn quit_with_unsaved_changes_requires_confirmation() {
    let (_dir, _path, mut session) = dirty_session("guard.txt");
    session.handle_command(EditorCommand::Quit).unwrap();

    assert_eq!(session.mode, SessionMode::ConfirmQuit);
    assert!(matches!(
        session.pending_prompt,
        Some(PromptState::UnsavedChanges {
            focus: UnsavedChoice::Save,
            ..
        })
    ));

    let view = session.render_view();
    let popup = view.popup.expect("unsaved popup should render");
    assert_eq!(popup.variant, PromptVariant::Full);
    assert_eq!(popup.title, "Unsaved changes");
    assert!(
        popup
            .message
            .as_deref()
            .unwrap()
            .contains("Save before quitting")
    );
    assert_eq!(popup.actions.len(), 3);
    assert_eq!(popup.actions[0].label, "[Save]");
    assert!(popup.actions[0].focused);
    assert_eq!(view.bottom_line, None);

    session.handle_command(EditorCommand::Cancel).unwrap();
    assert_eq!(session.mode, SessionMode::Editing);
    assert!(!session.is_exiting());
    assert!(session.render_view().popup.is_none());
}

#[test]
fn clean_document_quits_immediately_without_popup() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("clean.txt");
    fs::write(&path, "seed\n").unwrap();

    let mut session = EditingSession::open(&path, TerminalSize::new(80, 24)).unwrap();
    session.handle_command(EditorCommand::Quit).unwrap();

    assert!(session.is_exiting());
    assert!(session.pending_prompt.is_none());
    assert!(session.render_view().popup.is_none());
}

#[test]
fn enter_key_selects_focused_save_action_in_unsaved_prompt() {
    // Regression test: pressing Enter in the quit confirmation popup
    // must select the currently focused action (Save, by default).
    // Previously, EditorCommand::Enter fell through to `_ => {}` in
    // handle_prompt_command and did nothing.
    let (_dir, _path, mut session) = dirty_session("enter-select.txt");

    // Enter quit confirmation popup
    session.handle_command(EditorCommand::Quit).unwrap();
    assert_eq!(session.mode, SessionMode::ConfirmQuit);
    assert!(matches!(
        session.pending_prompt,
        Some(PromptState::UnsavedChanges {
            focus: UnsavedChoice::Save,
            ..
        })
    ));

    // Press Enter expecting it to confirm the focused "Save" action.
    // Before the fix this does nothing (Enter is not handled in handle_prompt_command),
    // so mode stays ConfirmQuit instead of entering Exiting state.
    session.handle_command(EditorCommand::Enter).unwrap();

    // After committing Save, the quit flow should complete and exit.
    assert!(
        session.is_exiting(),
        "Enter key should select the focused Save action and complete the quit"
    );
}

#[test]
fn long_path_status_text_does_not_replace_quit_popup() {
    let dir = tempdir().unwrap();
    let long_dir = dir
        .path()
        .join("very")
        .join("long")
        .join("path")
        .join("for")
        .join("popup")
        .join("visibility")
        .join("checks");
    fs::create_dir_all(&long_dir).unwrap();
    let path = long_dir.join("example.txt");
    fs::write(&path, "seed\n").unwrap();

    let mut session = EditingSession::open(&path, TerminalSize::new(80, 24)).unwrap();
    session
        .handle_command(EditorCommand::InsertChar('x'))
        .unwrap();
    session.handle_command(EditorCommand::Quit).unwrap();

    let view = session.render_view();
    // The path is absolute and long, so the footer truncates from the left.
    // It still carries the dirty marker and does not override the popup.
    assert!(view.footer_line.contains("(*)"));
    assert!(view.footer_line.starts_with("..."));
    let popup = view.popup.expect("quit popup should take precedence");
    assert_eq!(popup.title, "Unsaved changes");
    assert_eq!(view.bottom_line, None);
}
