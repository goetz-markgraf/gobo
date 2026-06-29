use crate::app::EditingSession;
use crate::editor::buffer;
use crate::editor::cursor;
use crate::editor::status;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalSize {
    pub width: u16,
    pub height: u16,
}

impl TerminalSize {
    pub const fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ViewportState {
    pub top_line: usize,
    pub left_column: usize,
    pub visible_height: u16,
    pub visible_width: u16,
}

impl ViewportState {
    pub fn from_terminal(size: TerminalSize, prompt_lines: u16) -> Self {
        let mut viewport = Self {
            top_line: 0,
            left_column: 0,
            visible_height: 0,
            visible_width: 0,
        };
        viewport.update_for_terminal(size, prompt_lines);
        viewport
    }

    pub fn update_for_terminal(&mut self, size: TerminalSize, prompt_lines: u16) {
        self.visible_width = size.width;
        self.visible_height = size.height.saturating_sub(prompt_lines);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderView {
    pub body_lines: Vec<String>,
    pub status_line: String,
    pub prompt_line: Option<String>,
    pub cursor_x: u16,
    pub cursor_y: u16,
}

pub fn render_view(session: &EditingSession) -> RenderView {
    let mut body_lines = Vec::new();
    let width = session.viewport.visible_width as usize;
    let height = session.viewport.visible_height as usize;

    for row in 0..height {
        let line_index = session.viewport.top_line + row;
        let line_text = buffer::line_content(&session.document.text, line_index);
        body_lines.push(slice_visible_columns(&line_text, session.viewport.left_column, width));
    }

    let status_line = status::format_status_line(session);
    let prompt_line = session
        .pending_prompt
        .as_ref()
        .map(status::prompt_line)
        .or_else(|| status::search_prompt(session));

    let cursor_line = buffer::line_of_char(&session.document.text, session.cursor.char_index);
    let cursor_column = cursor::visual_column(&session.document.text, session.cursor.char_index);
    let cursor_y = cursor_line.saturating_sub(session.viewport.top_line) as u16;
    let cursor_x = cursor_column.saturating_sub(session.viewport.left_column) as u16;

    RenderView {
        body_lines,
        status_line,
        prompt_line,
        cursor_x,
        cursor_y,
    }
}

pub fn slice_visible_columns(line: &str, left_column: usize, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    let mut output = String::new();
    let mut visual_column = 0;

    for grapheme in line.graphemes(true) {
        let grapheme_width = UnicodeWidthStr::width(grapheme);
        let next_column = visual_column + grapheme_width;

        if next_column <= left_column {
            visual_column = next_column;
            continue;
        }

        if visual_column >= left_column + width {
            break;
        }

        output.push_str(grapheme);
        visual_column = next_column;
    }

    output
}
