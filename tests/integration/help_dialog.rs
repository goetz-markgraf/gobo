// Integration tests for Help Dialog overlay (spec 011).
// Tests: Ctrl-H opens help, scrolling with arrows, Enter/Escape close,
// mode restoration, no document mutation during help open.

use gobo::app::{EditingSession, PromptState, SessionMode};
use gobo::editor::input::EditorCommand;
use gobo::editor::render::TerminalSize;
use std::fs;
use tempfile::tempdir;

/// Helper: create a dirty session with the given filename.
fn dirty_session(name: &str) -> (tempfile::TempDir, EditingSession) {
    let dir = tempdir().unwrap();
    let path = dir.path().join(name);
    fs::write(&path, "seed line\n".as_bytes()).unwrap();
    let mut session = EditingSession::open(&path, TerminalSize::new(80, 24)).unwrap();
    // Make it dirty
    session
        .handle_command(EditorCommand::InsertChar('x'))
        .unwrap();
    (dir, session)
}

// -----------------------------------------------------------------------
// S1: Ctrl-H opens HelpDialog (primary flow)

#[test]
fn ctrl_h_opens_help_dialog() {
    let (_dir, mut session) = dirty_session("open.txt");
    session.handle_command(EditorCommand::Quit).unwrap(); // start ConfirmQuit mode

    // Even in ConfirmQuit mode, Ctrl-H should open help
    // (mode stays ConfirmQuit, but pending_prompt changes)
    session
        .handle_command(EditorCommand::ShowHelp)
        .unwrap();

    assert_eq!(session.mode, SessionMode::ConfirmQuit);
    assert!(matches!(
        session.pending_prompt,
        Some(PromptState::HelpDialog)
    ));
    // help_scroll_offset should be 0 initially
    assert_eq!(session.help_scroll_offset, 0);

    // popup should be rendered with HelpDialog content
    let view = session.render_view();
    let popup = view.popup.expect("popup should exist");
    assert_eq!(popup.title, "Keyboard Shortcuts");
    assert!(popup.actions.is_empty()); // no action buttons
    assert!(!popup.popup_rows.is_empty()); // has keybinding rows
}

// -----------------------------------------------------------------------
// S2: Help dialog closes with Enter
// S3: Help dialog closes with Escape

#[test]
fn help_dialog_closes_with_enter() {
    let (_dir, mut session) = dirty_session("close_enter.txt");
    session
        .handle_command(EditorCommand::ShowHelp)
        .unwrap();

    assert!(matches!(
        session.pending_prompt,
        Some(PromptState::HelpDialog)
    ));

    // Press Enter to close
    session.handle_command(EditorCommand::Enter).unwrap();

    assert!(session.pending_prompt.is_none());
}

#[test]
fn help_dialog_closes_with_escape() {
    let (_dir, mut session) = dirty_session("close_esc.txt");
    session
        .handle_command(EditorCommand::ShowHelp)
        .unwrap();

    // Press Escape to close
    session.handle_command(EditorCommand::Cancel).unwrap();

    assert!(session.pending_prompt.is_none());
}

// -----------------------------------------------------------------------
// S4: No document mutation while help is open

#[test]
fn no_document_mutation_during_help() {
    let (_dir, mut session) = dirty_session("no_mutate.txt");
    let initial_buffer = session.document.text.to_string();

    // Open help dialog
    session
        .handle_command(EditorCommand::ShowHelp)
        .unwrap();

    // Type random keys - they should be silently ignored
    session
        .handle_command(EditorCommand::InsertChar('\n'))
        .unwrap();
    session
        .handle_command(EditorCommand::InsertChar('a'))
        .unwrap();
    session
        .handle_command(EditorCommand::MoveRight)
        .unwrap();
    session
        .handle_command(EditorCommand::MoveLeft)
        .unwrap();

    // Close help
    session
        .handle_command(EditorCommand::Cancel)
        .unwrap();

    // Buffer should be unchanged
    assert_eq!(session.document.text.to_string(), initial_buffer);
}

// -----------------------------------------------------------------------
// S5: Arrow keys scroll the list

#[test]
fn arrow_keys_scroll_help_list() {
    let (_dir, mut session) = dirty_session("scroll.txt");
    session
        .handle_command(EditorCommand::ShowHelp)
        .unwrap();
    assert!(matches!(
        session.pending_prompt,
        Some(PromptState::HelpDialog)
    ));

    // Scroll down
    session
        .handle_command(EditorCommand::MoveDown)
        .unwrap();
    let offset = session.help_scroll_offset;
    assert_eq!(offset, 1);

    session
        .handle_command(EditorCommand::MoveDown)
        .unwrap();
    assert_eq!(session.help_scroll_offset, 2);

    // Scroll up
    session
        .handle_command(EditorCommand::MoveUp)
        .unwrap();
    assert_eq!(session.help_scroll_offset, 1);

    // Boundary: back to top
    session
        .handle_command(EditorCommand::MoveUp)
        .unwrap();
    assert_eq!(session.help_scroll_offset, 0);
}

#[test]
fn scroll_offset_starts_at_zero() {
    let (_dir, mut session) = dirty_session("scroll_zero.txt");
    session
        .handle_command(EditorCommand::ShowHelp)
        .unwrap();
    assert_eq!(session.help_scroll_offset, 0);
}

// -----------------------------------------------------------------------
// S8: Help dialog over existing prompt (stacked popup)

#[test]
fn help_opens_over_confirm_quit_and_restores() {
    let (_dir, mut session) = dirty_session("stack.txt");

    // Enter ConfirmQuit mode (dirty document, press Quit)
    session.handle_command(EditorCommand::Quit).unwrap();
    assert_eq!(session.mode, SessionMode::ConfirmQuit);

    // Open help dialog on top
    session
        .handle_command(EditorCommand::ShowHelp)
        .unwrap();
    assert!(matches!(
        session.pending_prompt,
        Some(PromptState::HelpDialog)
    ));

    // Close help with Escape → should restore ConfirmQuit mode
    session.handle_command(EditorCommand::Cancel).unwrap();

    assert_eq!(session.mode, SessionMode::ConfirmQuit);
}

// -----------------------------------------------------------------------
// S6: Compact layout works (small terminal)

#[test]
fn compact_terminal_shows_help_dialog() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("compact.txt");
    fs::write(&path, "seed line\n".as_bytes()).unwrap();
    let mut session = EditingSession::open(&path, TerminalSize::new(30, 6)).unwrap();

    session
        .handle_command(EditorCommand::ShowHelp)
        .unwrap();

    let view = session.render_view();
    let popup = view.popup.expect("popup should exist");
    assert_eq!(popup.title, "Keyboard Shortcuts");
    // In Compact variant, fewer rows visible due to height constraint
    let body_height: usize = 6usize.saturating_sub(3).max(2);
    assert!(popup.popup_rows.len() <= body_height + 1); // +footer_row
}
