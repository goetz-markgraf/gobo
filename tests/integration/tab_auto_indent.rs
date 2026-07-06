use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use gobo::app::{ConflictChoice, EditingSession, PromptAction, PromptState, SessionMode, UnsavedChoice};
use gobo::document::{AccessMode, DocumentBuffer};
use gobo::editor::cursor::Selection;
use gobo::editor::input::{map_key_event, EditorCommand};
use gobo::editor::render::TerminalSize;

fn make_session(text: &str) -> EditingSession {
    let mut doc = DocumentBuffer::open("/tmp/tab-auto-indent.md").unwrap();
    doc.replace_contents(text);
    EditingSession::new(doc, TerminalSize::new(80, 24))
}

#[test]
fn tab_key_maps_to_editing_tab_command_and_shift_tab_stays_previous_choice() {
    let tab = map_key_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
    let back_tab = map_key_event(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT));

    assert_eq!(tab, Some(EditorCommand::Tab));
    assert_eq!(back_tab, Some(EditorCommand::PreviousChoice));
}

#[test]
fn editing_tab_inserts_two_spaces_at_even_column() {
    let mut session = make_session("hello");
    session.cursor.char_index = 0;

    session.handle_command(EditorCommand::Tab).unwrap();

    assert_eq!(session.document.text.to_string(), "  hello");
    assert_eq!(session.cursor.char_index, 2);
    assert!(matches!(session.history.undo.last(), Some(gobo::editor::history::EditStep::Insert { text, .. }) if text == "  "));
}

#[test]
fn editing_tab_inserts_one_space_at_odd_column() {
    let mut session = make_session("hello");
    session.cursor.char_index = 1;

    session.handle_command(EditorCommand::Tab).unwrap();

    assert_eq!(session.document.text.to_string(), "h ello");
    assert_eq!(session.cursor.char_index, 2);
}

#[test]
fn editing_tab_works_mid_line() {
    let mut session = make_session("abcd");
    session.cursor.char_index = 2;

    session.handle_command(EditorCommand::Tab).unwrap();

    assert_eq!(session.document.text.to_string(), "ab  cd");
    assert_eq!(session.cursor.char_index, 4);
}

#[test]
fn tab_replaces_selection_atomically() {
    let mut session = make_session("abcdef");
    session.selection = Some(Selection { anchor: 2, head: 4 });

    session.handle_command(EditorCommand::Tab).unwrap();

    assert_eq!(session.document.text.to_string(), "ab  f");
    assert_eq!(session.cursor.char_index, 4);
    assert_eq!(session.selection, None);
    assert_eq!(session.history.undo.len(), 1);
    assert!(matches!(session.history.undo[0], gobo::editor::history::EditStep::Replace { .. }));

    session.handle_command(EditorCommand::Undo).unwrap();
    assert_eq!(session.document.text.to_string(), "abcdef");
}

#[test]
fn tab_is_blocked_in_read_only_mode() {
    let mut doc = DocumentBuffer::open("/tmp/tab-readonly.md").unwrap();
    doc.replace_contents("hello");
    doc.access_mode = AccessMode::ReadOnly;
    let mut session = EditingSession::new(doc, TerminalSize::new(80, 24));

    session.handle_command(EditorCommand::Tab).unwrap();

    assert_eq!(session.document.text.to_string(), "hello");
    assert_eq!(session.cursor.char_index, 0);
    assert_eq!(session.history.undo.len(), 0);
}

#[test]
fn tab_moves_unsaved_prompt_focus_forward() {
    let mut session = make_session("hello");
    session.mode = SessionMode::ConfirmQuit;
    session.pending_prompt = Some(PromptState::UnsavedChanges {
        action: PromptAction::Quit,
        focus: UnsavedChoice::Save,
    });

    session.handle_command(EditorCommand::Tab).unwrap();

    assert_eq!(
        session.pending_prompt,
        Some(PromptState::UnsavedChanges {
            action: PromptAction::Quit,
            focus: UnsavedChoice::Discard,
        })
    );
}

#[test]
fn tab_moves_conflict_prompt_focus_forward() {
    let mut session = make_session("hello");
    session.mode = SessionMode::SaveConflictPrompt;
    session.pending_prompt = Some(PromptState::SaveConflict {
        focus: ConflictChoice::Reload,
        resume_action: Some(PromptAction::Quit),
    });

    session.handle_command(EditorCommand::Tab).unwrap();

    assert_eq!(
        session.pending_prompt,
        Some(PromptState::SaveConflict {
            focus: ConflictChoice::Overwrite,
            resume_action: Some(PromptAction::Quit),
        })
    );
}

#[test]
fn enter_replaces_selection_with_newline_and_indent_atomically() {
    let mut session = make_session("  hello");
    session.selection = Some(Selection { anchor: 2, head: 4 });

    session.handle_command(EditorCommand::Enter).unwrap();

    assert_eq!(session.document.text.to_string(), "  \n  lo");
    assert_eq!(session.cursor.char_index, 5);
    assert_eq!(session.selection, None);
    assert_eq!(session.history.undo.len(), 1);
    assert!(matches!(session.history.undo[0], gobo::editor::history::EditStep::Replace { .. }));

    session.handle_command(EditorCommand::Undo).unwrap();
    assert_eq!(session.document.text.to_string(), "  hello");
}

#[test]
fn backspace_outdents_even_space_prefix_by_two_spaces() {
    let mut session = make_session("    hello");
    session.cursor.char_index = 4;

    session.handle_command(EditorCommand::Backspace).unwrap();

    assert_eq!(session.document.text.to_string(), "  hello");
    assert_eq!(session.cursor.char_index, 2);
}

#[test]
fn backspace_outdents_odd_space_prefix_by_one_space() {
    let mut session = make_session("   hello");
    session.cursor.char_index = 3;

    session.handle_command(EditorCommand::Backspace).unwrap();

    assert_eq!(session.document.text.to_string(), "  hello");
    assert_eq!(session.cursor.char_index, 2);
}

#[test]
fn backspace_falls_back_to_normal_delete_after_mixed_prefix() {
    let mut session = make_session("a  bc");
    session.cursor.char_index = 3;

    session.handle_command(EditorCommand::Backspace).unwrap();

    assert_eq!(session.document.text.to_string(), "a bc");
    assert_eq!(session.cursor.char_index, 2);
}

#[test]
fn backspace_at_column_zero_does_nothing() {
    let mut session = make_session("hello");
    session.cursor.char_index = 0;

    session.handle_command(EditorCommand::Backspace).unwrap();

    assert_eq!(session.document.text.to_string(), "hello");
    assert_eq!(session.cursor.char_index, 0);
    assert_eq!(session.history.undo.len(), 0);
}

#[test]
fn backspace_replaces_selection_then_outdents_in_one_step() {
    let mut session = make_session("    hello");
    session.selection = Some(Selection { anchor: 3, head: 5 });

    session.handle_command(EditorCommand::Backspace).unwrap();

    assert_eq!(session.document.text.to_string(), "  llo");
    assert_eq!(session.cursor.char_index, 2);
    assert_eq!(session.selection, None);
    assert_eq!(session.history.undo.len(), 1);
    assert!(matches!(session.history.undo[0], gobo::editor::history::EditStep::Replace { .. }));

    session.handle_command(EditorCommand::Undo).unwrap();
    assert_eq!(session.document.text.to_string(), "    hello");
}

#[test]
fn backspace_over_selection_falls_back_to_regular_char_delete_in_one_step() {
    let mut session = make_session("abXYZc");
    session.selection = Some(Selection { anchor: 2, head: 4 });

    session.handle_command(EditorCommand::Backspace).unwrap();

    assert_eq!(session.document.text.to_string(), "ac");
    assert_eq!(session.cursor.char_index, 1);
    assert_eq!(session.selection, None);
    assert_eq!(session.history.undo.len(), 1);
    assert!(matches!(session.history.undo[0], gobo::editor::history::EditStep::Replace { .. }));

    session.handle_command(EditorCommand::Undo).unwrap();
    assert_eq!(session.document.text.to_string(), "abXYZc");
}

#[test]
fn backspace_over_selection_at_line_start_deletes_previous_line_break_in_one_step() {
    let mut session = make_session("ab\nXYZc");
    session.selection = Some(Selection { anchor: 3, head: 5 });

    session.handle_command(EditorCommand::Backspace).unwrap();

    assert_eq!(session.document.text.to_string(), "abc");
    assert_eq!(session.cursor.char_index, 2);
    assert_eq!(session.selection, None);
    assert_eq!(session.history.undo.len(), 1);
    assert!(matches!(session.history.undo[0], gobo::editor::history::EditStep::Replace { .. }));

    session.handle_command(EditorCommand::Undo).unwrap();
    assert_eq!(session.document.text.to_string(), "ab\nXYZc");
}
