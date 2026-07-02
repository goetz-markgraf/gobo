use gobo::editor::history::{EditStep, History, RecordOutcome};
use ropey::Rope;

/// Build an Insert step for `text` at `index`.
fn ins(index: usize, text: &str) -> EditStep {
    EditStep::Insert {
        index,
        text: text.to_string(),
    }
}

/// Build a Delete step for `text` at `index`.
fn del(index: usize, text: &str) -> EditStep {
    EditStep::Delete {
        index,
        text: text.to_string(),
    }
}

#[test]
fn new_history_is_empty_and_unbounded() {
    let h = History::new();
    assert!(!h.can_undo());
    assert!(!h.can_redo());
    assert!(h.undo.is_empty());
    assert!(h.redo.is_empty());
    assert_eq!(h.undo_capacity(), usize::MAX);
}

#[test]
fn with_capacity_sets_capacity_for_test_injection() {
    let h = History::with_capacity(2);
    assert_eq!(h.undo_capacity(), 2);
    assert!(h.undo.is_empty());
}

#[test]
fn record_pushes_to_undo_and_clears_redo() {
    let mut h = History::new();
    let mut text = Rope::from_str("");
    text.insert(0, "a");
    h.record(ins(0, "a"));
    assert_eq!(h.undo.len(), 1);
    assert!(h.redo.is_empty());
    assert!(h.can_undo());

    // populate redo via one undo (removes 'a' -> empty rope)
    assert_eq!(h.undo(&mut text), Some(0));
    assert_eq!(text.to_string(), "");
    assert_eq!(h.redo.len(), 1);

    // a new record must clear redo. Simulate the new edit on the rope too.
    text.insert(0, "b");
    let outcome = h.record(ins(0, "b"));
    assert_eq!(outcome, RecordOutcome { oldest_dropped: false });
    assert!(h.redo.is_empty());
    assert_eq!(h.undo.len(), 1);
    // the cleared redo was the old 'a' step; undo stack now only holds 'b'
    assert_eq!(h.undo[0], ins(0, "b"));
    // record never mutates rope; rope holds 'b' from our explicit insert
    assert_eq!(text.to_string(), "b");
}

#[test]
fn record_at_capacity_evicts_oldest_and_reports_dropped() {
    let mut h = History::with_capacity(2);
    h.record(ins(0, "a"));
    h.record(ins(1, "b"));
    assert_eq!(h.undo.len(), 2);

    let outcome = h.record(ins(2, "c"));
    assert_eq!(outcome, RecordOutcome { oldest_dropped: true });
    // oldest ('a') evicted, newest kept
    assert_eq!(h.undo.len(), 2);
    assert_eq!(h.undo[0], ins(1, "b"));
    assert_eq!(h.undo[1], ins(2, "c"));
}

#[test]
fn insert_undo_then_redo_is_round_trip_identity_on_rope() {
    let mut h = History::new();
    h.record(ins(0, "hello"));
    let mut text = Rope::from_str("hello");

    // undo: removes "hello" -> empty rope; cursor goes to before_cursor (0)
    let c = h.undo(&mut text).unwrap();
    assert_eq!(c, 0);
    assert_eq!(text.to_string(), "");
    assert!(h.undo.is_empty());
    assert_eq!(h.redo.len(), 1);

    // redo: re-inserts "hello"
    let c = h.redo(&mut text).unwrap();
    assert_eq!(c, 5);
    assert_eq!(text.to_string(), "hello");
}

#[test]
fn delete_reverse_is_insert_and_vice_versa() {
    let mut h = History::new();
    // simulate: original edit deleted "llo" starting at index 2 (text was "hello" -> "he")
    h.record(del(2, "llo"));
    let mut text = Rope::from_str("he");

    // undo of delete = re-insert "llo"
    let c = h.undo(&mut text).unwrap();
    // before_cursor for Delete = end_index = 2 + 3 = 5
    assert_eq!(c, 5);
    assert_eq!(text.to_string(), "hello");

    // redo of delete = remove "llo" again
    let c = h.redo(&mut text).unwrap();
    // after_cursor for Delete = index = 2
    assert_eq!(c, 2);
    assert_eq!(text.to_string(), "he");
}

#[test]
fn undo_then_redo_noop_on_rope_and_cursor() {
    let mut h = History::new();
    h.record(ins(0, "abc"));
    let mut text = Rope::from_str("abc");
    let before = text.clone();

    let u = h.undo(&mut text).unwrap();
    let r = h.redo(&mut text).unwrap();
    // redo returns after_cursor (3); undo returned before_cursor (0).
    assert_eq!(u, 0);
    assert_eq!(r, 3);
    assert_eq!(text.to_string(), before.to_string());
    // back to one undo step, empty redo
    assert_eq!(h.undo.len(), 1);
    assert!(h.redo.is_empty());
}

#[test]
fn empty_undo_returns_none_and_mutates_nothing() {
    let mut h = History::new();
    let mut text = Rope::from_str("xyz");
    let snapshot = text.clone();
    assert_eq_or_none(h.undo(&mut text), ());
    assert_eq!(text.to_string(), snapshot.to_string());
    assert!(h.undo.is_empty());
    assert!(h.redo.is_empty());
}

#[test]
fn empty_redo_returns_none_and_mutates_nothing() {
    let mut h = History::new();
    let mut text = Rope::from_str("xyz");
    let snapshot = text.clone();
    assert_eq!(h.redo(&mut text), None);
    assert_eq!(text.to_string(), snapshot.to_string());
    assert!(h.undo.is_empty());
    assert!(h.redo.is_empty());
}

#[test]
fn determinism_full_undo_then_full_redo_restores_rope() {
    let mut h = History::new();
    let mut text = Rope::from_str("");
    let steps = [ins(0, "a"), ins(1, "b"), ins(2, "c")];
    for s in &steps {
        text.insert(step_index(s), step_text(s));
        h.record(s.clone());
    }
    assert_eq!(text.to_string(), "abc");

    // undo all the way to None
    while h.undo(&mut text).is_some() {}
    assert_eq!(text.to_string(), "");
    assert!(!h.can_undo());

    // redo all the way to None restores content + cursor
    let mut last_cursor = 0;
    while let Some(c) = h.redo(&mut text) {
        last_cursor = c;
    }
    assert_eq!(text.to_string(), "abc");
    assert_eq!(last_cursor, 3);
}

#[test]
fn clear_empties_both_stacks() {
    let mut h = History::new();
    h.record(ins(0, "a"));
    let mut text = Rope::from_str("a");
    h.undo(&mut text).unwrap();
    assert!(!h.undo.is_empty() || !h.redo.is_empty()); // one is non-empty
    h.clear();
    assert!(h.undo.is_empty());
    assert!(h.redo.is_empty());
    assert!(!h.can_undo());
    assert!(!h.can_redo());
}

// Helper trait impls used above.

/// Small helper: compare an Option<usize> to None without using assert_eq on the
/// `None` literal directly (keeps a single clear failure message).
fn assert_eq_or_none<T: PartialEq + std::fmt::Debug>(actual: Option<T>, _none_marker: ()) {
    assert!(actual.is_none(), "expected None, got {:?}", actual);
}

fn step_index(s: &EditStep) -> usize {
    match s {
        EditStep::Insert { index, .. } | EditStep::Delete { index, .. } => *index,
    }
}

fn step_text(s: &EditStep) -> &str {
    match s {
        EditStep::Insert { text, .. } | EditStep::Delete { text, .. } => text.as_str(),
    }
}
