use gobo::app::EditingSession;
use gobo::editor::history::History;
use gobo::editor::input::EditorCommand;
use gobo::editor::render::TerminalSize;
use std::fs;
use tempfile::tempdir;

/// Build a session over a fresh temp file seeded with `text` (empty string ok).
/// Cursor starts at index 0.
fn session_with_seed(text: &str) -> EditingSession {
    let dir = tempdir().unwrap();
    let path = dir.path().join("doc.txt");
    if !text.is_empty() {
        fs::write(&path, text).unwrap();
    }
    // NOTE: `dir` is dropped here, but the file path remains valid on most
    // systems for the test's lifetime; we re-read via the session. To be safe
    // we keep the dir alive by leaking it (tests are short-lived processes).
    std::mem::forget(dir);
    EditingSession::open(&path, TerminalSize::new(80, 24)).unwrap()
}

/// Insert a single char at the current cursor, advancing the cursor.
fn type_char(s: &mut EditingSession, c: char) {
    s.handle_command(EditorCommand::InsertChar(c)).unwrap();
}

fn undo(s: &mut EditingSession) {
    s.handle_command(EditorCommand::Undo).unwrap();
}

fn redo(s: &mut EditingSession) {
    s.handle_command(EditorCommand::Redo).unwrap();
}

/// Build a session whose history is capped at `cap` (test injection point).
fn session_with_capped_history(text: &str, cap: usize) -> EditingSession {
    let mut s = session_with_seed(text);
    s.history = History::with_capacity(cap);
    s
}

// ---------------------------------------------------------------------------
// User Story 1 — Undo
// ---------------------------------------------------------------------------

#[test]
fn us1_a1_empty_doc_undo_back_to_empty() {
    let mut s = session_with_seed("");
    type_char(&mut s, 'a');
    type_char(&mut s, 'b');
    type_char(&mut s, 'c');
    assert_eq!(s.document.text.to_string(), "abc");
    assert_eq!(s.cursor.char_index, 3);

    undo(&mut s);
    assert_eq!(s.document.text.to_string(), "ab");
    assert_eq!(s.cursor.char_index, 2);
    undo(&mut s);
    assert_eq!(s.document.text.to_string(), "a");
    assert_eq!(s.cursor.char_index, 1);
    undo(&mut s);
    assert_eq!(s.document.text.to_string(), "");
    assert_eq!(s.cursor.char_index, 0);
    assert!(!s.history.can_undo());
}

#[test]
fn us1_a2_delete_last_char_then_undo_restores() {
    let mut s = session_with_seed("Hallo");
    s.cursor.char_index = 5; // end of "Hallo"
    s.cursor.preferred_column = 5;

    s.handle_command(EditorCommand::Backspace).unwrap();
    assert_eq!(s.document.text.to_string(), "Hall");
    assert_eq!(s.cursor.char_index, 4);

    undo(&mut s);
    assert_eq!(s.document.text.to_string(), "Hallo");
    assert_eq!(s.cursor.char_index, 5);
}

#[test]
fn us1_a3_k_fold_undo_equals_state_after_n_minus_k_edit() {
    let mut s = session_with_seed("");
    // 3 edits: "a", "b", "c" -> states "a","ab","abc"
    type_char(&mut s, 'a');
    type_char(&mut s, 'b');
    type_char(&mut s, 'c');

    // k=1 undo -> state after 2nd edit ("ab")
    undo(&mut s);
    assert_eq!(s.document.text.to_string(), "ab");
    assert_eq!(s.cursor.char_index, 2);

    // k=2 -> state after 1st edit ("a")
    undo(&mut s);
    assert_eq!(s.document.text.to_string(), "a");
    assert_eq!(s.cursor.char_index, 1);
}

#[test]
fn us1_undo_at_empty_stack_is_complete_noop() {
    let mut s = session_with_seed("xyz");
    s.cursor.char_index = 2;
    let text_before = s.document.text.clone();
    let cursor_before = s.cursor.char_index;
    let dirty_before = s.document.dirty;
    let status_before = s.status.clone();

    undo(&mut s);
    assert_eq!(s.document.text.to_string(), text_before.to_string());
    assert_eq!(s.cursor.char_index, cursor_before);
    assert_eq!(s.document.dirty, dirty_before);
    assert_eq!(s.status, status_before);
    assert!(!s.history.can_undo());
}

// ---------------------------------------------------------------------------
// User Story 2 — Redo
// ---------------------------------------------------------------------------

#[test]
fn us2_a1_three_undos_then_three_redos_back_to_edited_state() {
    let mut s = session_with_seed("");
    type_char(&mut s, 'a');
    type_char(&mut s, 'b');
    type_char(&mut s, 'c');
    assert_eq!(s.document.text.to_string(), "abc");

    undo(&mut s);
    undo(&mut s);
    undo(&mut s);
    assert_eq!(s.document.text.to_string(), "");
    assert!(!s.history.can_undo());

    redo(&mut s);
    assert_eq!(s.document.text.to_string(), "a");
    redo(&mut s);
    assert_eq!(s.document.text.to_string(), "ab");
    redo(&mut s);
    assert_eq!(s.document.text.to_string(), "abc");
    assert!(!s.history.can_redo());
    assert_eq!(s.cursor.char_index, 3);
}

#[test]
fn us2_a2_undo_to_origin_then_redo_once_is_first_edit_state() {
    let mut s = session_with_seed("");
    type_char(&mut s, 'a');
    type_char(&mut s, 'b');
    undo(&mut s);
    undo(&mut s);
    assert_eq!(s.document.text.to_string(), "");

    redo(&mut s);
    assert_eq!(s.document.text.to_string(), "a");
    assert_eq!(s.cursor.char_index, 1);
}

#[test]
fn us2_a3_redo_at_empty_redo_stack_is_noop() {
    let mut s = session_with_seed("");
    type_char(&mut s, 'a');
    let text_before = s.document.text.clone();
    let cursor_before = s.cursor.char_index;
    let dirty_before = s.document.dirty;
    let status_before = s.status.clone();

    redo(&mut s); // redo stack empty -> no-op
    assert_eq!(s.document.text.to_string(), text_before.to_string());
    assert_eq!(s.cursor.char_index, cursor_before);
    assert_eq!(s.document.dirty, dirty_before);
    assert_eq!(s.status, status_before);
    assert!(!s.history.can_redo());
}

#[test]
fn us2_fr012_determinism_full_undo_then_full_redo_byte_identical() {
    let mut s = session_with_seed("");
    type_char(&mut s, 'a');
    type_char(&mut s, 'b');
    type_char(&mut s, 'c');
    let final_text = s.document.text.to_string();
    let final_cursor = s.cursor.char_index;

    while s.history.can_undo() {
        undo(&mut s);
    }
    assert_eq!(s.document.text.to_string(), "");

    while s.history.can_redo() {
        redo(&mut s);
    }
    assert_eq!(s.document.text.to_string(), final_text);
    assert_eq!(s.cursor.char_index, final_cursor);
}

// ---------------------------------------------------------------------------
// User Story 3 — Redo cleared on new edit
// ---------------------------------------------------------------------------

#[test]
fn us3_a1_redo_cleared_on_new_edit() {
    let mut s = session_with_seed("");
    type_char(&mut s, 'a');
    type_char(&mut s, 'b');
    undo(&mut s); // text "a", redo has 'b'
    assert!(s.history.can_redo());

    type_char(&mut s, 'x'); // text "ax"
    assert_eq!(s.document.text.to_string(), "ax");
    assert!(s.history.redo.is_empty());

    // redo is now a no-op
    let before = s.document.text.to_string();
    redo(&mut s);
    assert_eq!(s.document.text.to_string(), before);
    assert!(!s.history.can_redo());

    // only the new edit ('x') is undoable; undoing returns to "a"
    undo(&mut s);
    assert_eq!(s.document.text.to_string(), "a");
}

#[test]
fn us3_a2_new_edit_after_multiple_undos_only_new_edit_undoable() {
    let mut s = session_with_seed("");
    type_char(&mut s, 'a');
    type_char(&mut s, 'b');
    type_char(&mut s, 'c'); // "abc"
    undo(&mut s);
    undo(&mut s);
    assert_eq!(s.document.text.to_string(), "a");
    assert!(s.history.can_redo());

    type_char(&mut s, 'y'); // "ay", redo cleared
    assert!(s.history.redo.is_empty());

    // redo is a no-op
    let before = s.document.text.to_string();
    redo(&mut s);
    assert_eq!(s.document.text.to_string(), before);

    // only the 'y' edit is undoable -> back to "a"; the original 'a' step
    // (never undone) is still on the stack, so a further undo reaches "".
    undo(&mut s);
    assert_eq!(s.document.text.to_string(), "a");
    assert!(s.history.can_undo());
    undo(&mut s);
    assert_eq!(s.document.text.to_string(), "");
    assert!(!s.history.can_undo());
}

// ---------------------------------------------------------------------------
// User Story 4 — Session lifetime
// ---------------------------------------------------------------------------

#[test]
fn us4_a1_fresh_session_has_empty_history_and_undo_redo_are_noops() {
    // Build history in one session, then drop it and open a fresh one.
    let dir = tempdir().unwrap();
    let path = dir.path().join("life.txt");
    fs::write(&path, "").unwrap();

    {
        let mut first = EditingSession::open(&path, TerminalSize::new(80, 24)).unwrap();
        type_char(&mut first, 'a');
        type_char(&mut first, 'b');
        assert!(first.history.can_undo());
        // session dropped here; nothing persisted about history
    }

    let mut second = EditingSession::open(&path, TerminalSize::new(80, 24)).unwrap();
    assert!(second.history.undo.is_empty());
    assert!(second.history.redo.is_empty());
    assert!(!second.history.can_undo());
    assert!(!second.history.can_redo());

    let text_before = second.document.text.to_string();
    let cursor_before = second.cursor.char_index;
    undo(&mut second);
    redo(&mut second);
    assert_eq!(second.document.text.to_string(), text_before);
    assert_eq!(second.cursor.char_index, cursor_before);
}

#[test]
fn us4_a2_save_then_reopen_only_future_edits_undoable() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("save.txt");
    fs::write(&path, "").unwrap();

    {
        let mut s = EditingSession::open(&path, TerminalSize::new(80, 24)).unwrap();
        type_char(&mut s, 'a');
        type_char(&mut s, 'b');
        s.handle_command(EditorCommand::Save).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "ab");
    }

    let mut s = EditingSession::open(&path, TerminalSize::new(80, 24)).unwrap();
    assert!(s.history.undo.is_empty());
    assert!(s.history.redo.is_empty());
    type_char(&mut s, 'c'); // "abc"
    undo(&mut s);
    assert_eq!(s.document.text.to_string(), "ab"); // only 'c' undone
    assert!(!s.history.can_undo());
}

// ---------------------------------------------------------------------------
// Phase 7 — Cross-cutting edge cases
// ---------------------------------------------------------------------------

#[test]
fn edge_unicode_and_newline_steps_restore_byte_identical() {
    let mut s = session_with_seed("");
    // 'ä' is a multibyte char; type it, then Enter, then 'x'
    type_char(&mut s, 'ä');
    s.handle_command(EditorCommand::Enter).unwrap();
    type_char(&mut s, 'x');
    let built = s.document.text.to_string();
    assert_eq!(built, "ä\nx");
    let cursor_built = s.cursor.char_index;

    undo(&mut s);
    assert_eq!(s.document.text.to_string(), "ä\n");
    undo(&mut s);
    assert_eq!(s.document.text.to_string(), "ä");
    undo(&mut s);
    assert_eq!(s.document.text.to_string(), "");

    redo(&mut s);
    redo(&mut s);
    redo(&mut s);
    assert_eq!(s.document.text.to_string(), built);
    assert_eq!(s.cursor.char_index, cursor_built);
}

#[test]
fn edge_large_single_insert_step_restores_exactly() {
    // Enter inserts "\n" as a single step; undo/redo exactly restore it.
    let mut s = session_with_seed("");
    s.handle_command(EditorCommand::Enter).unwrap();
    assert_eq!(s.document.text.to_string(), "\n");
    assert_eq!(s.history.undo.len(), 1);

    undo(&mut s);
    assert_eq!(s.document.text.to_string(), "");
    assert!(s.history.undo.is_empty());
    assert_eq!(s.history.redo.len(), 1);

    redo(&mut s);
    assert_eq!(s.document.text.to_string(), "\n");
    assert_eq!(s.history.undo.len(), 1);
    assert!(s.history.redo.is_empty());
}

#[test]
fn tab_undo_and_redo_restore_in_one_step() {
    let mut s = session_with_seed("");
    s.handle_command(EditorCommand::Tab).unwrap();
    assert_eq!(s.document.text.to_string(), "  ");
    assert_eq!(s.history.undo.len(), 1);

    undo(&mut s);
    assert_eq!(s.document.text.to_string(), "");

    redo(&mut s);
    assert_eq!(s.document.text.to_string(), "  ");
}

#[test]
fn smart_backspace_undo_and_redo_restore_in_one_step() {
    let mut s = session_with_seed("    hello");
    s.cursor.char_index = 4;
    s.cursor.preferred_column = 4;

    s.handle_command(EditorCommand::Backspace).unwrap();
    assert_eq!(s.document.text.to_string(), "  hello");
    assert_eq!(s.history.undo.len(), 1);

    undo(&mut s);
    assert_eq!(s.document.text.to_string(), "    hello");

    redo(&mut s);
    assert_eq!(s.document.text.to_string(), "  hello");
}

#[test]
fn edge_fr009_mode_gating_undo_redo_ignored_in_search_and_prompts() {
    let mut s = session_with_seed("");
    type_char(&mut s, 'a');
    let text_before = s.document.text.to_string();
    let undo_before = s.history.undo.len();

    // Enter search input mode; Ctrl-Z / Ctrl-Y ignored.
    s.handle_command(EditorCommand::Search).unwrap();
    assert_eq!(s.mode, gobo::app::SessionMode::SearchInput);
    undo(&mut s);
    redo(&mut s);
    assert_eq!(s.document.text.to_string(), text_before);
    assert_eq!(s.history.undo.len(), undo_before);
    // cancel search
    s.handle_command(EditorCommand::Cancel).unwrap();
    assert_eq!(s.mode, gobo::app::SessionMode::Editing);

    // ConfirmQuit prompt: dirty doc, request quit -> prompt; undo/redo ignored.
    s.handle_command(EditorCommand::Quit).unwrap();
    assert!(s.pending_prompt.is_some());
    let prompt_undo = s.history.undo.len();
    undo(&mut s);
    redo(&mut s);
    assert_eq!(s.history.undo.len(), prompt_undo);
    assert_eq!(s.document.text.to_string(), text_before);
    // cancel back to editing
    s.handle_command(EditorCommand::Cancel).unwrap();
    assert_eq!(s.mode, gobo::app::SessionMode::Editing);
}

#[test]
fn edge_fr013_save_preserves_history_undo_still_works() {
    let mut s = session_with_seed("");
    type_char(&mut s, 'a');
    type_char(&mut s, 'b');
    s.handle_command(EditorCommand::Save).unwrap();
    // history survives save (save records no step, does not clear)
    assert_eq!(s.history.undo.len(), 2);

    undo(&mut s);
    assert_eq!(s.document.text.to_string(), "a");
    undo(&mut s);
    assert_eq!(s.document.text.to_string(), "");
}

#[test]
fn edge_fr006_sc007_memory_pressure_evicts_oldest_and_warns() {
    let mut s = session_with_capped_history("", 2);
    type_char(&mut s, 'a');
    type_char(&mut s, 'b');
    // both status messages so far should be info
    assert_eq!(s.status.as_ref().unwrap().kind, gobo::editor::status::StatusKind::Info);

    type_char(&mut s, 'c'); // 3rd edit, capacity 2 -> evict 'a'
    // the edit is applied
    assert_eq!(s.document.text.to_string(), "abc");
    // status is the truncation warning
    assert_eq!(s.status.as_ref().unwrap().text, "History truncated to free memory");
    assert_eq!(s.status.as_ref().unwrap().kind, gobo::editor::status::StatusKind::Warning);
    // undo stack has only 2 steps ('b','c'); undoing twice reaches "a", not ""
    undo(&mut s);
    assert_eq!(s.document.text.to_string(), "ab");
    undo(&mut s);
    assert_eq!(s.document.text.to_string(), "a");
    assert!(!s.history.can_undo()); // 'a' step was evicted
}

