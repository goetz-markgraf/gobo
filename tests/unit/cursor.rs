use gobo::editor::cursor::{char_index_for_visual_column, ensure_cursor_in_view, move_down, move_left, move_right, move_up, visual_column, CursorState, Selection, move_select_left, move_select_right, move_select_up, move_select_down};
use gobo::editor::buffer::line_of_char;
use gobo::editor::render::{TerminalSize, ViewportState};
use ropey::Rope;

#[test]
fn vertical_motion_preserves_preferred_column_when_possible() {
    let text = Rope::from_str("abcd\nxy\nabcdef\n");
    let mut cursor = CursorState {
        char_index: 3,
        preferred_column: 3,
    };

    move_down(&mut cursor, &text);
    assert_eq!(cursor.char_index, 7);

    move_down(&mut cursor, &text);
    assert_eq!(cursor.char_index, 11);

    move_up(&mut cursor, &text);
    assert_eq!(cursor.char_index, 7);
}

#[test]
fn viewport_clamps_to_keep_cursor_visible() {
    let text = Rope::from_str("one\ntwo\nthree\nfour\nfive\n");
    let mut cursor = CursorState::default();
    move_right(&mut cursor, &text);
    move_right(&mut cursor, &text);
    cursor.char_index = 12;
    let mut viewport = ViewportState::from_terminal(TerminalSize::new(4, 3), 1);

    ensure_cursor_in_view(&text, &cursor, &mut viewport);

    assert_eq!(viewport.top_line, 1);
    assert_eq!(viewport.left_column, 1);
}

#[test]
fn grapheme_width_is_used_for_visual_columns() {
    let text = Rope::from_str("a界b\n");

    assert_eq!(visual_column(&text, 0), 0);
    assert_eq!(visual_column(&text, 1), 1);
    assert_eq!(visual_column(&text, 2), 3);
    assert_eq!(char_index_for_visual_column(&text, 0, 3), 2);
}

// ---- Selection geometry (spec 007 T007) ----

#[test]
fn selection_range_is_min_max_half_open() {
    let s = Selection { anchor: 2, head: 5 };
    assert_eq!(s.range(), 2..5);
    let reversed = Selection { anchor: 5, head: 2 };
    assert_eq!(reversed.range(), 2..5);
}

#[test]
fn selection_is_empty_when_anchor_equals_head() {
    assert!(Selection { anchor: 3, head: 3 }.is_empty());
    assert!(!Selection { anchor: 3, head: 4 }.is_empty());
    assert!(!Selection { anchor: 4, head: 3 }.is_empty());
}

#[test]
fn selection_is_forward_when_head_ge_anchor() {
    assert!(Selection { anchor: 2, head: 5 }.is_forward());
    assert!(Selection { anchor: 3, head: 3 }.is_forward());
    assert!(!Selection { anchor: 5, head: 2 }.is_forward());
}

#[test]
fn selection_default_is_empty_and_zero_range() {
    let s = Selection::default();
    assert_eq!(s.anchor, 0);
    assert_eq!(s.head, 0);
    assert!(s.is_empty());
    assert_eq!(s.range(), 0..0);
}

// ---- MoveSelect* motions (spec 007 T011) ----

#[test]
fn move_select_seeds_anchor_on_first_move_then_fixes_it() {
    let text = Rope::from_str("Hallo");
    let mut cursor = CursorState { char_index: 2, preferred_column: 2 };
    let mut sel: Option<Selection> = None;

    move_select_right(&mut sel, &mut cursor, &text);
    assert_eq!(sel, Some(Selection { anchor: 2, head: 3 }));
    assert_eq!(cursor.char_index, 3);

    // Anchor stays fixed; head advances.
    move_select_right(&mut sel, &mut cursor, &text);
    assert_eq!(sel, Some(Selection { anchor: 2, head: 4 }));
    assert_eq!(cursor.char_index, 4);
}

#[test]
fn move_select_direction_flips_when_head_crosses_anchor() {
    let text = Rope::from_str("Hallo");
    let mut cursor = CursorState { char_index: 2, preferred_column: 2 };
    let mut sel: Option<Selection> = None;

    move_select_left(&mut sel, &mut cursor, &text);
    assert_eq!(sel, Some(Selection { anchor: 2, head: 1 }));
    assert!(!sel.unwrap().is_forward());

    // Cross back to the right of the anchor -> forward again.
    move_select_right(&mut sel, &mut cursor, &text);
    move_select_right(&mut sel, &mut cursor, &text);
    assert_eq!(sel, Some(Selection { anchor: 2, head: 3 }));
    assert!(sel.unwrap().is_forward());
}

#[test]
fn move_select_can_shrink_to_empty_at_anchor() {
    let text = Rope::from_str("Hallo");
    let mut cursor = CursorState { char_index: 2, preferred_column: 2 };
    let mut sel: Option<Selection> = None;

    move_select_right(&mut sel, &mut cursor, &text); // head 3
    move_select_left(&mut sel, &mut cursor, &text); // head 2 -> empty
    assert_eq!(sel, Some(Selection { anchor: 2, head: 2 }));
    assert!(sel.unwrap().is_empty());
}

// ---- MoveSelect* document-boundary clamping (spec 007 T012) ----

#[test]
fn move_select_left_at_doc_start_clamps_and_seeds_anchor() {
    let text = Rope::from_str("Hallo");
    let mut cursor = CursorState { char_index: 0, preferred_column: 0 };
    let mut sel: Option<Selection> = None;
    move_select_left(&mut sel, &mut cursor, &text);
    assert_eq!(cursor.char_index, 0);
    assert_eq!(sel, Some(Selection { anchor: 0, head: 0 }));
}

#[test]
fn move_select_right_at_doc_end_clamps() {
    let text = Rope::from_str("Hi");
    let mut cursor = CursorState { char_index: 2, preferred_column: 2 };
    let mut sel: Option<Selection> = None;
    move_select_right(&mut sel, &mut cursor, &text);
    assert_eq!(cursor.char_index, 2);
    assert_eq!(sel, Some(Selection { anchor: 2, head: 2 }));
}

#[test]
fn move_select_up_at_top_line_clamps_keeps_column() {
    let text = Rope::from_str("abc\ndefgh\n");
    let mut cursor = CursorState { char_index: 1, preferred_column: 1 };
    let mut sel: Option<Selection> = None;
    move_select_up(&mut sel, &mut cursor, &text);
    // Already on line 0; head stays.
    assert_eq!(cursor.char_index, 1);
    assert_eq!(sel, Some(Selection { anchor: 1, head: 1 }));
}

#[test]
fn move_select_down_at_last_line_clamps() {
    let text = Rope::from_str("abc\ndefgh");
    // last line content "defgh" starts at char 4
    let mut cursor = CursorState { char_index: 5, preferred_column: 1 };
    let mut sel: Option<Selection> = None;
    move_select_down(&mut sel, &mut cursor, &text);
    // Already on the last line; head stays.
    assert_eq!(cursor.char_index, 5);
    assert_eq!(sel, Some(Selection { anchor: 5, head: 5 }));
}

// ---- Spec 008 / User Story 1: cursor at end-of-doc after a trailing newline ----
// `visual_column` and `move_right` both derive the line from `buffer::line_of_char`,
// so once that mapping is corrected the visible cursor lands at column 0 of the
// empty trailing line (FR-002, FR-003). These FAIL against the pre-fix mapping.

/// A cursor at `len_chars()` after a trailing `\n` is at the start of the
/// empty trailing line, so its visual column is 0 -- not the width of the
/// previous (non-empty) line (FR-002).
#[test]
fn visual_column_is_zero_at_doc_end_after_trailing_newline() {
    assert_eq!(visual_column(&Rope::from_str("abc\n"), 4), 0);
    assert_eq!(visual_column(&Rope::from_str("abc\n\n"), 5), 0);
    assert_eq!(visual_column(&Rope::from_str("abc\n\n\n"), 6), 0);
    assert_eq!(visual_column(&Rope::from_str("\n"), 1), 0);
}

/// Moving right from the end of the `abc` line (index 3) over the trailing
/// `\n` onto end-of-doc places the cursor at column 0 of the empty trailing
/// line: the visible cursor is exactly where the next insert lands (FR-003).
#[test]
fn move_right_to_doc_end_lands_on_empty_trailing_line() {
    let text = Rope::from_str("abc\n");
    let mut cursor = CursorState { char_index: 3, preferred_column: 3 };

    move_right(&mut cursor, &text);

    assert_eq!(cursor.char_index, 4, "right move crosses the trailing newline to end-of-doc");
    assert_eq!(line_of_char(&text, cursor.char_index), 1, "cursor is on the empty trailing line");
    assert_eq!(visual_column(&text, cursor.char_index), 0, "cursor column is 0 of the empty line");
}

// ---- Spec 008 / User Story 2: arrow nav across the trailing newline (FR-004) ----

/// Arrow navigation across a trailing `\n` is consistent in both directions
/// (FR-004): right from the end of `abc` crosses onto the empty trailing line,
/// and left from end-of-doc crosses back onto the end of `abc` (before the
/// newline). The round trip returns to the start, proving no drift.
#[test]
fn arrow_nav_across_trailing_newline_is_consistent_both_directions() {
    let text = Rope::from_str("abc\n");
    let mut cursor = CursorState { char_index: 3, preferred_column: 3 };

    // Right over the trailing newline -> empty trailing line.
    move_right(&mut cursor, &text);
    assert_eq!(cursor.char_index, 4);
    assert_eq!(line_of_char(&text, cursor.char_index), 1);
    assert_eq!(visual_column(&text, cursor.char_index), 0);

    // Left back over the trailing newline -> end of the `abc` line.
    move_left(&mut cursor, &text);
    assert_eq!(cursor.char_index, 3);
    assert_eq!(line_of_char(&text, cursor.char_index), 0);
    assert_eq!(visual_column(&text, cursor.char_index), 3);
}
