// Integration tests for clipboard Cut/Copy/Paste (spec 009).
// All tests drive real OS clipboard via `EditingSession::handle_command()`.
//
// Serialisation: clipboard is a process-global OS resource.  Parallel test
// threads sharing one clipboard risk data races and arboard "Busy" errors.
// Every test body grabs `CLIP_GUARD` for its entire duration, making the
// bodies run one at a time within this binary.  Other test binaries never
// touch the clipboard, so cross-process interference is not a concern.

use gobo::app::EditingSession;
use gobo::editor::clipboard;
use gobo::editor::cursor::Selection;
use gobo::editor::history::EditStep;
use gobo::editor::input::EditorCommand;
use gobo::editor::render::TerminalSize;
use std::fs;
use std::sync::Mutex;
use tempfile::tempdir;

// --------------------------------------------------------------------------
// Serialisation guard (one clipboard test body at a time within this binary)
// --------------------------------------------------------------------------
static CLIP_GUARD: Mutex<()> = Mutex::new(());

fn lock() -> std::sync::MutexGuard<'static, ()> {
    CLIP_GUARD.lock().unwrap_or_else(|e| e.into_inner())
}

// --------------------------------------------------------------------------
// Helpers
// --------------------------------------------------------------------------

/// Session over a temp file seeded with `text` (cursor at 0).
fn session_with_seed(text: &str) -> EditingSession {
    let dir = tempdir().unwrap();
    let path = dir.path().join("doc.txt");
    if !text.is_empty() {
        fs::write(&path, text).unwrap();
    }
    std::mem::forget(dir);
    EditingSession::open(&path, TerminalSize::new(80, 24)).unwrap()
}

/// Extract the status message text of `s`.
fn status(s: &EditingSession) -> &str {
    s.status.as_ref().map(|m| m.text.as_str()).unwrap_or("")
}

/// Write text to the OS clipboard directly via arboard, bypassing gobo's 1 MB
/// cap. Used for the large-clipboard test that needs to put >1 MB into the OS.
fn set_clip_raw(text: &str) {
    arboard::Clipboard::new().unwrap().set_text(text).unwrap();
}

/// Write text through gobo's production seam (enforces 1 MB cap).
fn set_clip(text: &str) {
    clipboard::write_text(text).unwrap();
}

/// Read current clipboard text through gobo's production seam.
fn read_clip() -> Option<String> {
    clipboard::read_text()
}

fn undo(s: &mut EditingSession) {
    s.handle_command(EditorCommand::Undo).unwrap();
}

// ==========================================================================
// T024 — US1: copy with selection (FR-001, SC-001)
// ==========================================================================
#[test]
fn copy_with_selection() {
    let _g = lock();
    let mut s = session_with_seed("Hello World");
    // select "World" = chars 6..11
    s.selection = Some(Selection { anchor: 6, head: 11 });
    s.handle_command(EditorCommand::Copy).unwrap();
    // clipboard contains "World"
    assert_eq!(read_clip().as_deref(), Some("World"));
    // editor text unchanged
    assert_eq!(s.document.text.to_string(), "Hello World");
    // selection preserved (FR-001)
    assert_eq!(s.selection, Some(Selection { anchor: 6, head: 11 }));
    // no undo entry
    assert!(s.history.undo.is_empty());
    // status "Copied 5 chars"
    assert_eq!(status(&s), "Copied 5 chars");
}

// ==========================================================================
// T025 — US1: copy without selection — single grapheme (FR-001)
// ==========================================================================
#[test]
fn copy_without_selection_single_char() {
    let _g = lock();
    let mut s = session_with_seed("Hello");
    s.cursor.char_index = 0; // cursor before 'H'
    s.handle_command(EditorCommand::Copy).unwrap();
    assert_eq!(read_clip().as_deref(), Some("H"));
    assert_eq!(s.document.text.to_string(), "Hello");
    assert!(s.history.undo.is_empty());
    assert_eq!(status(&s), "Copied 1 chars");
}

// Multi-byte grapheme: "a" + combining acute (U+0301) = 2 chars, 1 grapheme.
#[test]
fn copy_without_selection_multibyte_grapheme() {
    let _g = lock();
    let grapheme = "a\u{0301}"; // á (2 chars)
    let text = format!("{grapheme}b");
    let mut s = session_with_seed(&text);
    s.cursor.char_index = 0;
    s.handle_command(EditorCommand::Copy).unwrap();
    assert_eq!(read_clip().as_deref(), Some(grapheme));
    assert_eq!(s.document.text.to_string(), text);
    assert_eq!(status(&s), "Copied 2 chars");
}

// ==========================================================================
// T026 — US2: cut without selection, then undo (FR-003, FR-005, FR-006)
// ==========================================================================
#[test]
fn cut_without_selection_and_undo() {
    let _g = lock();
    let mut s = session_with_seed("HalloWelt");
    s.cursor.char_index = 5; // between 'o' and 'W'
    s.handle_command(EditorCommand::Cut).unwrap();
    assert_eq!(s.document.text.to_string(), "Halloelt");
    assert_eq!(read_clip().as_deref(), Some("W"));
    assert_eq!(status(&s), "Cut 1 chars");
    assert_eq!(s.history.undo.len(), 1);
    assert!(matches!(s.history.undo[0], EditStep::Delete { .. }));
    // cursor lands at cut position
    assert_eq!(s.cursor.char_index, 5);

    // single undo restores everything
    undo(&mut s);
    assert_eq!(s.document.text.to_string(), "HalloWelt");
    // FR-006: clipboard unchanged after undo
    assert_eq!(read_clip().as_deref(), Some("W"));
}

// ==========================================================================
// T027 — US3: cut with multi-line selection (FR-002, FR-005, FR-010)
// ==========================================================================
#[test]
fn cut_with_multiline_selection_and_undo() {
    let _g = lock();
    let seed = "Zeile1\nZeile2\nZeile3\n";
    let mut s = session_with_seed(seed);
    // "Zeile2\n" = chars 7..14 (7 chars: Z e i l e 2 \n)
    s.selection = Some(Selection { anchor: 7, head: 14 });
    s.handle_command(EditorCommand::Cut).unwrap();
    assert_eq!(s.document.text.to_string(), "Zeile1\nZeile3\n");
    assert_eq!(read_clip().as_deref(), Some("Zeile2\n"));
    assert_eq!(status(&s), "Cut 7 chars");
    assert_eq!(s.selection, None);
    assert_eq!(s.history.undo.len(), 1);
    assert!(matches!(s.history.undo[0], EditStep::Replace { .. }));

    // undo restores full text + selection cleared
    undo(&mut s);
    assert_eq!(s.document.text.to_string(), seed);
    assert_eq!(s.selection, None);
    // clipboard persists (FR-006)
    assert_eq!(read_clip().as_deref(), Some("Zeile2\n"));
}

// ==========================================================================
// T028 — clipboard persists after undo (FR-006)
// ==========================================================================
#[test]
fn clipboard_persists_after_undo() {
    let _g = lock();
    let mut s = session_with_seed("HalloWelt");
    s.cursor.char_index = 5;
    s.handle_command(EditorCommand::Cut).unwrap();
    assert_eq!(read_clip().as_deref(), Some("W"));
    undo(&mut s);
    // clipboard still has 'W'
    assert_eq!(read_clip().as_deref(), Some("W"), "FR-006: clipboard unchanged by undo");
    // can paste the same content again
    s.handle_command(EditorCommand::Paste).unwrap();
    assert_eq!(s.document.text.to_string(), "HalloWWelt");
}

// ==========================================================================
// T029 — US4: paste without selection (FR-004, FR-005, FR-007)
// ==========================================================================
#[test]
fn paste_without_selection_and_undo() {
    let _g = lock();
    set_clip("Test");
    let mut s = session_with_seed("HalloWelt");
    s.cursor.char_index = 5; // between 'o' and 'W'
    s.handle_command(EditorCommand::Paste).unwrap();
    assert_eq!(s.document.text.to_string(), "HalloTestWelt");
    assert_eq!(s.cursor.char_index, 9); // after "Test"
    assert_eq!(s.selection, None);     // FR-007
    assert_eq!(status(&s), "Pasted 4 chars");
    assert_eq!(s.history.undo.len(), 1);
    assert!(matches!(s.history.undo[0], EditStep::Insert { .. }));

    undo(&mut s);
    assert_eq!(s.document.text.to_string(), "HalloWelt");
    assert_eq!(s.cursor.char_index, 5);
}

// ==========================================================================
// T030 — US5: paste over selection (FR-004, FR-005, FR-007)
// ==========================================================================
#[test]
fn paste_over_selection_and_undo() {
    let _g = lock();
    set_clip("Earth");
    let mut s = session_with_seed("Hello World");
    // select "World" = 6..11
    s.selection = Some(Selection { anchor: 6, head: 11 });
    s.handle_command(EditorCommand::Paste).unwrap();
    assert_eq!(s.document.text.to_string(), "Hello Earth");
    assert_eq!(s.cursor.char_index, 11); // after "Earth"
    assert_eq!(s.selection, None);       // FR-007
    assert_eq!(status(&s), "Pasted 5 chars");
    assert_eq!(s.history.undo.len(), 1);
    assert!(matches!(s.history.undo[0], EditStep::Replace { .. }));

    undo(&mut s);
    assert_eq!(s.document.text.to_string(), "Hello World");
    assert_eq!(s.cursor.char_index, 6);
}

// ==========================================================================
// T031 — empty clipboard paste is silent no-op (FR-009)
// ==========================================================================
#[test]
fn empty_clipboard_paste_is_noop() {
    let _g = lock();
    // Clear the clipboard so get_text returns None or empty
    let _ = arboard::Clipboard::new().map(|mut c| c.clear());
    let mut s = session_with_seed("Hello");
    let status_before = s.status.clone();
    s.cursor.char_index = 2;
    s.handle_command(EditorCommand::Paste).unwrap();
    assert_eq!(s.document.text.to_string(), "Hello");
    assert!(s.history.undo.is_empty());
    // status is unchanged (silent no-op — no new status message)
    assert_eq!(s.status, status_before);
}

// ==========================================================================
// T032 — large clipboard (>1 MB) paste shows warning (FR-013)
// ==========================================================================
#[test]
fn large_clipboard_paste_shows_warning() {
    let _g = lock();
    // Bypass gobo's write cap — put >1 MB directly via arboard
    let big = "A".repeat((1 << 20) + 1);
    set_clip_raw(&big);
    let mut s = session_with_seed("Hello");
    s.cursor.char_index = 2;
    s.handle_command(EditorCommand::Paste).unwrap();
    assert_eq!(s.document.text.to_string(), "Hello");
    assert!(s.history.undo.is_empty());
    assert_eq!(status(&s), "Clipboard content too large (>1 MB)");
}

// ==========================================================================
// T033 — binary / no-text clipboard paste is silent no-op (FR-009)
// ==========================================================================
#[test]
fn binary_clipboard_paste_is_noop() {
    let _g = lock();
    // clear() leaves no text on the clipboard (ContentNotAvailable on get_text)
    arboard::Clipboard::new().unwrap().clear().unwrap();
    let mut s = session_with_seed("Hello");
    let status_before = s.status.clone();
    s.cursor.char_index = 2;
    s.handle_command(EditorCommand::Paste).unwrap();
    assert_eq!(s.document.text.to_string(), "Hello");
    assert!(s.history.undo.is_empty());
    assert_eq!(s.status, status_before);
}

// ==========================================================================
// T034 — multi-line cut and undo restores all lines (FR-010)
// ==========================================================================
#[test]
fn multiline_cut_restore_preserves_newlines() {
    let _g = lock();
    let seed = "Hello\nWorld\nFoo\n";
    let mut s = session_with_seed(seed);
    // select "World\n" = chars 6..12 (6 chars: W o r l d \n)
    s.selection = Some(Selection { anchor: 6, head: 12 });
    s.handle_command(EditorCommand::Cut).unwrap();
    assert_eq!(s.document.text.to_string(), "Hello\nFoo\n");
    assert_eq!(read_clip().as_deref(), Some("World\n"));

    undo(&mut s);
    assert_eq!(s.document.text.to_string(), seed,
        "undo must restore all lines including the cut newline (FR-010)");
    // clipboard still intact (FR-006)
    assert_eq!(read_clip().as_deref(), Some("World\n"));
}
