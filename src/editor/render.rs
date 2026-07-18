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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PromptVariant {
    Full,
    Compact,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopupRect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromptActionLabel {
    pub label: String,
    pub focused: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopupView {
    // Popup presentation stays in the render layer so prompt behavior remains in app state.
    pub variant: PromptVariant,
    pub title: String,
    pub message: Option<String>,
    pub actions: Vec<PromptActionLabel>,
    pub help_text: String,
    pub rect: PopupRect,
    // Row data for non-action-popups (HelpDialog). Empty for other prompts.
    pub popup_rows: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderView {
    pub body_lines: Vec<BodyLine>,
    pub footer_line: String, // bottom row: filename (left) + status message (right)
    pub bottom_line: Option<String>,
    // Overlay prompts render independently from the bottom-line status/search surfaces.
    pub popup: Option<PopupView>,
    pub cursor_x: u16,
    pub cursor_y: u16,
}

/// One visible body line plus its selection highlight spans (spec 007, FR-010).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BodyLine {
    pub text: String,
    /// Visual-column highlight spans (line-local, clipped to the viewport).
    /// Empty when no selection intersects this line.
    pub highlights: Vec<HighlightSpan>,
}

/// A run of highlighted visual columns within a single body line (spec 007).
/// Geometry only — styling is applied in `main.rs::draw` (constitution II).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HighlightSpan {
    pub start_col: usize,
    pub end_col: usize,
}

/// Format the single footer row: the filename (CLI path) plus optional
/// ` (*)` dirty marker on the LEFT, the status `message` on the RIGHT,
/// and — when the terminal is wide enough — a centered `Ctrl-Q: Quit,
/// Ctrl-H: Help` hint BETWEEN the two. The hint is centered over the full
/// terminal width and is shown only when at least one space of separation
/// remains on both sides; otherwise it is dropped (no replacement) and the
/// two-region layout (name | message) from spec 005 applies. If the filename
/// is too long it is truncated from the left with a `...` prefix; the message
/// (which is less critical) is dropped first when space is tight.
/// Spec 005 FR-001 / FR-003 (revision 2026-07-02).
pub fn format_footer_line(path: &str, dirty: bool, message: &str, terminal_width: u16) -> String {
    const FOOTER_HINT: &str = "Ctrl-Q: Quit, Ctrl-H: Help";
    let hint_width = UnicodeWidthStr::width(FOOTER_HINT);

    let name = if dirty {
        format!("{} (*)", path)
    } else {
        path.to_string()
    };
    let width = terminal_width as usize;
    let name_width = UnicodeWidthStr::width(name.as_str());

    // No room for the message at all: show just the (possibly truncated) name.
    if name_width >= width {
        return truncate_name(&name, terminal_width, dirty);
    }

    let msg_width = UnicodeWidthStr::width(message);

    // Centered hint: placed at (width - hint_width) / 2 over the full width, and
    // shown only when at least one space of separation remains on both sides.
    // When it does not fit it is dropped entirely (no replacement); the
    // two-region layout below takes over.
    let hint_start = width.saturating_sub(hint_width) / 2;
    let hint_end = hint_start + hint_width;
    let hint_fits = hint_width <= width
        && hint_start > name_width
        && hint_end < width.saturating_sub(msg_width);
    if hint_fits {
        let left_pad = hint_start - name_width;
        let right_pad = width - msg_width - hint_end;
        return format!(
            "{}{}{}{}{}",
            name,
            " ".repeat(left_pad),
            FOOTER_HINT,
            " ".repeat(right_pad),
            message
        );
    }

    let gap_width = width.saturating_sub(name_width);
    // Need at least one space of separation between name and message.
    if msg_width < gap_width {
        let pad = gap_width - msg_width;
        return format!("{}{}{}", name, " ".repeat(pad), message);
    }

    // Message won't fit beside the name: drop it and show only the name, padded
    // right with spaces to fill the row. If even the name overflows, truncate.
    if name_width <= width {
        return format!("{}{}", name, " ".repeat(gap_width));
    }
    truncate_name(&name, terminal_width, dirty)
}

/// Truncate `name` (a path plus optional ` (*)` suffix) to `terminal_width`
/// columns, keeping the filename end and prepending `...`.
fn truncate_name(name: &str, terminal_width: u16, dirty: bool) -> String {
    let suffix_len = if dirty { 4usize } else { 0 }; // len(" (*)")
    let available = terminal_width
        .saturating_sub(3)
        .saturating_sub(suffix_len as u16);
    if dirty {
        let path_part = name.trim_end_matches(" (*)");
        truncate_left(path_part, available) + " (*)"
    } else {
        truncate_left(name, available)
    }
}

/// Truncate a string to fit `max_width` columns by dropping characters from
/// the **left** and prepending "...", so the right-most part (e.g. the
/// filename at the end of a path) is preserved. Spec 005 FR-001.
fn truncate_left(s: &str, max_width: u16) -> String {
    if UnicodeWidthStr::width(s) <= max_width as usize {
        return format!("...{}", s);
    }

    // Walk graphemes from the right, accumulating until the budget is full.
    let graphemes: Vec<&str> =
        unicode_segmentation::UnicodeSegmentation::graphemes(s, true).collect();
    let mut kept: Vec<&str> = Vec::new();
    let mut width = 0usize;
    for g in graphemes.iter().rev() {
        let gw = UnicodeWidthStr::width(*g);
        if width + gw > max_width as usize {
            break;
        }
        kept.push(g);
        width += gw;
    }
    kept.reverse();
    format!("...{}", kept.concat())
}

pub fn render_view(session: &EditingSession) -> RenderView {
    let mut body_lines = Vec::new();
    let width = session.viewport.visible_width as usize;
    let height = session.viewport.visible_height as usize;
    let left_column = session.viewport.left_column;
    let sel = session.selection;

    for row in 0..height {
        let line_index = session.viewport.top_line + row;
        let line_text = buffer::line_content(&session.document.text, line_index);
        let line_str = slice_visible_columns(
            &line_text,
            left_column,
            width,
        );
        let highlights = highlight_spans_for_line(
            &session.document.text,
            line_index,
            &line_text,
            sel,
            left_column,
            width,
        );
        body_lines.push(BodyLine {
            text: line_str,
            highlights,
        });
    }

    let message = status::current_message(session);
    let popup = status::popup_view(session, session.terminal_size);
    let bottom_line = if popup.is_some() {
        None
    } else {
        status::search_prompt(session)
    };

    let footer_line = format_footer_line(
        session.document.path.display().to_string().as_str(),
        session.document.dirty,
        &message,
        session.terminal_size.width,
    );

    let cursor_line = buffer::line_of_char(&session.document.text, session.cursor.char_index);
    let cursor_column = cursor::visual_column(&session.document.text, session.cursor.char_index);
    let cursor_y = cursor_line.saturating_sub(session.viewport.top_line) as u16;
    let cursor_x = cursor_column.saturating_sub(session.viewport.left_column) as u16;

    RenderView {
        body_lines,
        footer_line,
        bottom_line,
        popup,
        cursor_x,
        cursor_y,
    }
}

/// Compute the visual-column highlight spans for one line given an optional
/// selection (spec 007, FR-010). Returns an empty vector when there is no
/// selection or the selection does not intersect the line's visible content.
/// Spans are line-local visual columns, clipped to the visible viewport.
fn highlight_spans_for_line(
    text: &ropey::Rope,
    line_index: usize,
    line_text: &str,
    sel: Option<crate::editor::cursor::Selection>,
    left_column: usize,
    width: usize,
) -> Vec<HighlightSpan> {
    if width == 0 {
        return Vec::new();
    }
    let sel = match sel {
        Some(s) if !s.is_empty() => s,
        _ => return Vec::new(),
    };

    let range = sel.range();
    let line_start = buffer::line_start_char(text, line_index);
    let line_chars = line_text.chars().count();
    // The line's visible content occupies char indices
    // [line_start, line_start + line_chars) (excluding the trailing newline).
    let line_end = line_start + line_chars;
    if range.start >= line_end || range.end <= line_start {
        return Vec::new();
    }

    // Clamp the selection intersection to the line's char range.
    let intersection_start = range.start.max(line_start);
    let intersection_end = range.end.min(line_end);
    if intersection_start >= intersection_end {
        return Vec::new();
    }

    // Map the char intersection to visual columns relative to the line start.
    let prefix_start: String = line_text
        .chars()
        .take(intersection_start.saturating_sub(line_start))
        .collect();
    let col_start = unicode_width::UnicodeWidthStr::width(prefix_start.as_str());
    let prefix_end: String = line_text
        .chars()
        .take(intersection_end.saturating_sub(line_start))
        .collect();
    let col_end = unicode_width::UnicodeWidthStr::width(prefix_end.as_str());

    // Clip the single contiguous span to the visible viewport columns.
    let vis_start = col_start.max(left_column);
    let vis_end = col_end.min(left_column + width);
    if vis_start >= vis_end {
        return Vec::new();
    }
    vec![HighlightSpan {
        start_col: vis_start - left_column,
        end_col: vis_end - left_column,
    }]
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
