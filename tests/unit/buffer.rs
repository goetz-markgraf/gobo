use gobo::editor::buffer::{delete_char_at, insert_text, line_content, line_count, line_of_char, remove_char_before, replace_range};
use ropey::Rope;

// # Fix Trailing Newline Cursor Position
//
// Spec 008: the visual line for a cursor sitting exactly at the end of a
// document that ends with `\n`. Per contracts/cursor-line-mapping.md a cursor
// at `len_chars()` after a trailing `\n` belongs to the empty trailing line
// created by that newline (ropey models `"abc\n"` as 2 lines). The pre-fix
// `line_of_char` probed the preceding `\n` and ropey's `char_to_line('\n')`
// returned the line *containing* the newline, so the cursor was drawn one line
// too high.

#[test]
fn insert_delete_and_replace_work_on_rope_buffer() {
    let mut text = Rope::from_str("hello");

    let cursor = insert_text(&mut text, 5, " world");
    assert_eq!(cursor, 11);
    assert_eq!(text.to_string(), "hello world");

    let cursor = remove_char_before(&mut text, cursor).unwrap();
    assert_eq!(cursor, 10);
    assert_eq!(text.to_string(), "hello worl");

    assert!(delete_char_at(&mut text, 5));
    assert_eq!(text.to_string(), "helloworl");

    let cursor = replace_range(&mut text, 5..9, " rust");
    assert_eq!(cursor, 10);
    assert_eq!(text.to_string(), "hello rust");
}

#[test]
fn line_helpers_track_lines_and_columns() {
    let text = Rope::from_str("alpha\nbeta\n");

    assert_eq!(line_count(&text), 3);
    assert_eq!(line_content(&text, 0), "alpha");
    assert_eq!(line_content(&text, 1), "beta");
    assert_eq!(line_of_char(&text, 0), 0);
    assert_eq!(line_of_char(&text, 6), 1);
}

// ---- Spec 008: Phase 2 foundational regression test (reproduces the bug) ----

/// A cursor at `len_chars()` of a document ending in `\n` MUST map to the empty
/// trailing line. These four rows are the `[BUG]` cases from research.md and
/// the corrected mapping table in data-model.md; this test FAILS before the
/// `line_of_char` fix and PASSES after.
#[test]
fn line_of_char_bug_cursor_after_trailing_newline() {
    assert_eq!(line_of_char(&Rope::from_str("abc\n"), 4), 1);
    assert_eq!(line_of_char(&Rope::from_str("abc\n\n"), 5), 2);
    assert_eq!(line_of_char(&Rope::from_str("abc\n\n\n"), 6), 3);
    assert_eq!(line_of_char(&Rope::from_str("\n"), 1), 1);
}

// ---- Spec 008 / User Story 1: end-of-doc trailing-newline mapping (FR-006) ----

/// The corrected char->line mapping for the end-of-document trailing-newline
/// cases, per the table in data-model.md. Independent of the bug-repro test
/// above, this anchors User Story 1's contract: the cursor sits on the empty
/// trailing line for one or many trailing newlines (FR-006).
#[test]
fn line_of_char_end_of_doc_trailing_newline_maps_to_empty_line() {
    // One trailing newline -> the single empty trailing line.
    assert_eq!(line_of_char(&Rope::from_str("abc\n"), 4), 1);
    // Two trailing newlines -> the second empty trailing line.
    assert_eq!(line_of_char(&Rope::from_str("abc\n\n"), 5), 2);
    // Three trailing newlines -> the third empty trailing line.
    assert_eq!(line_of_char(&Rope::from_str("abc\n\n\n"), 6), 3);
    // A document consisting of a single newline -> the empty second line.
    assert_eq!(line_of_char(&Rope::from_str("\n"), 1), 1);
}

// ---- Spec 008: total + monotonicity sanity (FR-005 regression safety) ----

/// `line_of_char` is total and monotonically non-decreasing across the whole
/// cursor range `0..=len_chars()` for `"abc\n"`: moving the cursor right never
/// reports a lower line, and no in-range index panics (contract "Required
/// invariants"). This guards the end-of-doc branch against breaking ordering.
#[test]
fn line_of_char_is_total_and_non_decreasing_across_cursor_range() {
    let text = Rope::from_str("abc\n");
    let len = text.len_chars();

    let mut prev = line_of_char(&text, 0);
    for idx in 0..=len {
        let line = line_of_char(&text, idx);
        assert!(
            line >= prev,
            "line_of_char decreased at idx {idx}: {line} < {prev}"
        );
        prev = line;
    }
}

// ---- Spec 008 / User Story 2: already-correct cases MUST NOT change (FR-005) ----

/// The new end-of-doc branch fires only when `char_index == len_chars()` AND
/// the final character is `\n`. Every other index keeps the pre-fix mapping,
/// so these already-correct cases are a regression guard (FR-005).
#[test]
fn line_of_char_unaffected_cases_are_regression_guard() {
    // No trailing newline: end-of-doc maps to the only line.
    assert_eq!(line_of_char(&Rope::from_str("abc"), 3), 0);
    // Empty document: the single virtual line 0.
    assert_eq!(line_of_char(&Rope::from_str(""), 0), 0);
    // Trailing newline present, but the cursor is ON the `\n` (end of `abc`,
    // not past it): still line 0.
    assert_eq!(line_of_char(&Rope::from_str("abc\n"), 3), 0);
    // Two trailing newlines, cursor on the middle `\n` (not end-of-doc):
    // unchanged line 1.
    assert_eq!(line_of_char(&Rope::from_str("abc\n\n"), 4), 1);
}

