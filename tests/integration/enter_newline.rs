use gobo::app::EditingSession;
use gobo::document::{AccessMode, DocumentBuffer};
use gobo::editor::input::EditorCommand;
use gobo::editor::render::TerminalSize;

fn make_session(text: &str) -> EditingSession {
    let mut doc = DocumentBuffer::open("/tmp/enter-test.md").unwrap();
    doc.replace_contents(text);
    EditingSession::new(doc, TerminalSize::new(80, 24))
}

fn assert_enter_text(session: &mut EditingSession, expected_text: &str) {
    session.handle_command(EditorCommand::Enter).unwrap();
    assert_eq!(session.document.text.to_string(), expected_text);
}

#[test]
fn enter_at_end_of_text_creates_new_blank_line() {
    let mut session = make_session("Hello");
    session.cursor.char_index = 5;
    assert_enter_text(&mut session, "Hello\n");
}

#[test]
fn enter_at_end_with_trailing_newline() {
    let mut session = make_session("Hello\n");
    session.cursor.char_index = 5;
    assert_enter_text(&mut session, "Hello\n\n");
}

#[test]
fn enter_at_end_of_last_line_multi_line_doc() {
    let mut session = make_session("First\nSecond");
    session.cursor.char_index = 12;
    assert_enter_text(&mut session, "First\nSecond\n");
}

#[test]
fn enter_mid_line_splits_correctly() {
    let mut session = make_session("Hello World");
     session.cursor.char_index = 6;
    assert_enter_text(&mut session, "Hello \nWorld");
}

#[test]
fn enter_at_cursor_position_keeps_content_before() {
    // split at position 2 in "Hello" produces "He\nllo"
    let mut session = make_session("Hello");
    session.cursor.char_index = 2;
    assert_enter_text(&mut session, "He\nllo");
}

#[test]
fn enter_empty_doc_creates_newline() {
    let mut session = make_session("");
    assert_enter_text(&mut session, "\n");
}

#[test]
fn enter_existing_single_line_creates_two_lines() {
    let mut session = make_session("\n");
    assert_enter_text(&mut session, "\n\n");
}

#[test]
fn enter_read_only_doc_does_nothing() {
    let mut doc = DocumentBuffer::open("/tmp/ro.md").unwrap();
    doc.replace_contents("Hello");
    doc.access_mode = AccessMode::ReadOnly;
    let mut session = EditingSession::new(doc, TerminalSize::new(80, 24));
    session.handle_command(EditorCommand::Enter).unwrap();
    assert_eq!(session.document.text.to_string(), "Hello");
}

#[test]
fn enter_at_start_of_text() {
    let mut session = make_session("Hello");
    session.cursor.char_index = 0;
    assert_enter_text(&mut session, "\nHello");
}

#[test]
fn enter_beyond_length_is_clamped() {
    let mut session = make_session("Hi");
    session.cursor.char_index = 100;
    assert_enter_text(&mut session, "Hi\n");
}

#[test]
fn enter_read_only_succeeds_gracefully() {
    let mut doc = DocumentBuffer::open("/tmp/ro2.md").unwrap();
    doc.replace_contents("Hello\ndir");
    doc.access_mode = AccessMode::ReadOnly;
    let mut session = EditingSession::new(doc, TerminalSize::new(80, 24));
    assert!(session.handle_command(EditorCommand::Enter).is_ok());
}

#[test]
fn enter_single_char_doc() {
    let mut session = make_session("A");
    session.cursor.char_index = 1;
    assert_enter_text(&mut session, "A\n");
}
