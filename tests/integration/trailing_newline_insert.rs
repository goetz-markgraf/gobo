// Spec 008: integration regression guard for the trailing-newline cursor fix.
//
// The `line_of_char` fix only changes the char->line *display* mapping; it must
// not touch the insert path or the persisted content. These tests drive the
// real `EditingSession` + `handle_command` path to prove that editing at the end
// of the last non-empty line (just before a trailing `\n`) still behaves
// correctly and keeps the trailing `\n` intact (FR-003, FR-005, FR-007).

use gobo::app::EditingSession;
use gobo::document::DocumentBuffer;
use gobo::editor::buffer::line_of_char;
use gobo::editor::cursor::visual_column;
use gobo::editor::input::EditorCommand;
use gobo::editor::render::TerminalSize;

fn make_session(text: &str) -> EditingSession {
    let mut doc = DocumentBuffer::open("/tmp/spec008-trailing-insert.md").unwrap();
    doc.replace_contents(text);
    EditingSession::new(doc, TerminalSize::new(80, 24))
}

/// Spec 008 / User Story 2 acceptance 1 (FR-003/FR-005/FR-007): typing a
/// character at the end of the `abc` line -- just before the trailing newline,
/// not behind it -- inserts it there, leaves the cursor behind the new
/// character, and preserves the trailing `\n`. This already-correct case MUST
/// not regress under the `line_of_char` end-of-document branch.
#[test]
fn insert_at_end_of_line_before_trailing_newline_keeps_newline() {
    let mut session = make_session("abc\n");
    // End of the `abc` line, just before the trailing `\n`.
    session.cursor.char_index = 3;
    session.cursor.preferred_column = 3;

    session.handle_command(EditorCommand::InsertChar('x')).unwrap();

    // Content: `x` inserted at end of the line, trailing `\n` preserved (FR-007).
    assert_eq!(session.document.text.to_string(), "abcx\n");
    // Cursor sits behind the inserted `x`, still before the trailing `\n`.
    assert_eq!(session.cursor.char_index, 4);
    // The visible cursor matches the logical insert position: end of `abcx`
    // on line 0 (FR-002/FR-003). The cursor is ON the `\n` (not past it), so
    // the end-of-doc branch must NOT fire here (FR-005).
    assert_eq!(
        line_of_char(&session.document.text, session.cursor.char_index),
        0
    );
    assert_eq!(
        visual_column(&session.document.text, session.cursor.char_index),
        4
    );
}

/// Spec 008: persisting the edit leaves the trailing `\n` byte-identical
/// (FR-007, regression guard). The fix never mutates the `Rope` content; this
/// confirms an edited trailing-newline document still ends in exactly one `\n`.
#[test]
fn edited_trailing_newline_document_ends_with_single_newline() {
    let mut session = make_session("abc\n");
    session.cursor.char_index = 3;
    session.handle_command(EditorCommand::InsertChar('x')).unwrap();

    let saved = session.document.text.to_string();
    assert!(saved.ends_with('\n'), "trailing newline must survive the edit: {saved:?}");
    assert_eq!(saved, "abcx\n");
}
