use gobo::app::{PromptState, SessionMode, UnsavedChoice};
use gobo::app::EditingSession;
use gobo::editor::input::EditorCommand;
use gobo::editor::render::TerminalSize;
use tempfile::tempdir;

#[test]
fn quit_with_unsaved_changes_requires_confirmation() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("guard.txt");

    let mut session = EditingSession::open(&path, TerminalSize::new(80, 24)).unwrap();
    session.handle_command(EditorCommand::InsertChar('x')).unwrap();
    session.handle_command(EditorCommand::Quit).unwrap();

    assert_eq!(session.mode, SessionMode::ConfirmQuit);
    assert!(matches!(
        session.pending_prompt,
        Some(PromptState::UnsavedChanges {
            focus: UnsavedChoice::Save,
            ..
        })
    ));

    session.handle_command(EditorCommand::Cancel).unwrap();
    assert_eq!(session.mode, SessionMode::Editing);
    assert!(!session.is_exiting());
}
