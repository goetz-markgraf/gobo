use gobo::editor::cursor::{char_index_for_visual_column, ensure_cursor_in_view, move_down, move_right, move_up, visual_column, CursorState};
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
