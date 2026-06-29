use crate::editor::buffer;
use crate::editor::render::ViewportState;
use ropey::Rope;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct CursorState {
    pub char_index: usize,
    pub preferred_column: usize,
}

pub fn clamp_cursor(cursor: &mut CursorState, text: &Rope) {
    cursor.char_index = buffer::clamp_char_index(text, cursor.char_index);
    cursor.preferred_column = visual_column(text, cursor.char_index);
}

pub fn move_left(cursor: &mut CursorState, text: &Rope) {
    cursor.char_index = cursor.char_index.saturating_sub(1);
    cursor.preferred_column = visual_column(text, cursor.char_index);
}

pub fn move_right(cursor: &mut CursorState, text: &Rope) {
    cursor.char_index = (cursor.char_index + 1).min(text.len_chars());
    cursor.preferred_column = visual_column(text, cursor.char_index);
}

pub fn move_up(cursor: &mut CursorState, text: &Rope) {
    let current_line = buffer::line_of_char(text, cursor.char_index);
    if current_line == 0 {
        cursor.preferred_column = visual_column(text, cursor.char_index);
        return;
    }

    let target_line = current_line - 1;
    cursor.char_index = char_index_for_visual_column(text, target_line, cursor.preferred_column);
}

pub fn move_down(cursor: &mut CursorState, text: &Rope) {
    let current_line = buffer::line_of_char(text, cursor.char_index);
    let last_line = buffer::line_count(text).saturating_sub(1);
    if current_line >= last_line {
        cursor.preferred_column = visual_column(text, cursor.char_index);
        return;
    }

    let target_line = current_line + 1;
    cursor.char_index = char_index_for_visual_column(text, target_line, cursor.preferred_column);
}

pub fn ensure_cursor_in_view(text: &Rope, cursor: &CursorState, viewport: &mut ViewportState) {
    let line = buffer::line_of_char(text, cursor.char_index);
    let column = visual_column(text, cursor.char_index);

    if line < viewport.top_line {
        viewport.top_line = line;
    }

    let visible_height = viewport.visible_height.max(1) as usize;
    if line >= viewport.top_line + visible_height {
        viewport.top_line = line + 1 - visible_height;
    }

    if column < viewport.left_column {
        viewport.left_column = column;
    }

    let visible_width = viewport.visible_width.max(1) as usize;
    if column >= viewport.left_column + visible_width {
        viewport.left_column = column + 1 - visible_width;
    }
}

pub fn visual_column(text: &Rope, char_index: usize) -> usize {
    let line = buffer::line_of_char(text, char_index);
    let line_start = buffer::line_start_char(text, line);
    let within_line = char_index.saturating_sub(line_start);
    let line_text = buffer::line_content(text, line);
    let prefix: String = line_text.chars().take(within_line).collect();
    UnicodeWidthStr::width(prefix.as_str())
}

pub fn char_index_for_visual_column(text: &Rope, line_index: usize, desired_column: usize) -> usize {
    let line_start = buffer::line_start_char(text, line_index);
    let line_text = buffer::line_content(text, line_index);

    let mut visual = 0;
    let mut char_offset = 0;
    for grapheme in line_text.graphemes(true) {
        let grapheme_width = UnicodeWidthStr::width(grapheme);
        if visual + grapheme_width > desired_column {
            break;
        }
        visual += grapheme_width;
        char_offset += grapheme.chars().count();
    }

    line_start + char_offset
}
