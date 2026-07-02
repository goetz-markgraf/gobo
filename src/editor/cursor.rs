use crate::editor::buffer;
use crate::editor::render::ViewportState;
use ropey::Rope;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

// ---- Selection state (spec 007, FR-013 responsibility boundary) ------------
// `cursor.rs` owns the `Selection` type and the `MoveSelect*` selection-motion
// functions (FR-013). Text mutation lives in `buffer.rs`, undo steps in
// `history.rs` (`EditStep::Replace`), dispatch in `app.rs`, and the highlight
// projection in `render.rs`. Anchored to Gobo constitution I/II (readable,
// clear module boundaries).

/// A text selection: a fixed **anchor** (where the user first pressed
/// Shift+Arrow) plus a moving **head** (the live cursor). All indices are
/// character indices into the `Rope`, identical to the existing text model.
/// Session-bound, in-memory, never persisted (FR-001/FR-013).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Selection {
    /// Fixed char index where the selection started.
    pub anchor: usize,
    /// Moving char index of the live cursor end.
    pub head: usize,
}

impl Selection {
    /// Half-open char range actually covered: `[min(anchor, head), max(anchor, head))`.
    /// Always safe for `Rope::remove` / `buffer::replace_range`.
    pub fn range(self) -> std::ops::Range<usize> {
        self.anchor.min(self.head)..self.anchor.max(self.head)
    }

    /// `true` iff `anchor == head` (no visible selection). Per FR-008 an empty
    /// selection behaves as "no selection" for every edit command.
    pub fn is_empty(self) -> bool {
        self.anchor == self.head
    }

    /// `true` iff the head is at or past the anchor (forward selection).
    /// Direction is derived, not stored, to avoid a third consistency field.
    pub fn is_forward(self) -> bool {
        self.head >= self.anchor
    }
}

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

// ---- Selection motions (spec 007, FR-001/FR-002/FR-003) --------------------
// Each `move_select_*` seeds the anchor from the current cursor exactly once
// (when `selection` transitions `None` -> `Some`), then reuses the matching
// plain motion for the head movement, so document-boundary clamping and
// grapheme-aware column preservation are identical to plain motion (FR-003).
// The head may cross the anchor; direction flips naturally (FR-002).
/// Shift+Left: seed anchor on first move, then move the head one char left.
pub fn move_select_left(sel: &mut Option<Selection>, cursor: &mut CursorState, text: &Rope) {
    seed_anchor(sel, cursor);
    move_left(cursor, text);
    sel.as_mut().expect("selection seeded").head = cursor.char_index;
}

/// Shift+Right: seed anchor on first move, then move the head one char right.
pub fn move_select_right(sel: &mut Option<Selection>, cursor: &mut CursorState, text: &Rope) {
    seed_anchor(sel, cursor);
    move_right(cursor, text);
    sel.as_mut().expect("selection seeded").head = cursor.char_index;
}

/// Shift+Up: seed anchor on first move, then move the head one line up.
pub fn move_select_up(sel: &mut Option<Selection>, cursor: &mut CursorState, text: &Rope) {
    seed_anchor(sel, cursor);
    move_up(cursor, text);
    sel.as_mut().expect("selection seeded").head = cursor.char_index;
}

/// Shift+Down: seed anchor on first move, then move the head one line down.
pub fn move_select_down(sel: &mut Option<Selection>, cursor: &mut CursorState, text: &Rope) {
    seed_anchor(sel, cursor);
    move_down(cursor, text);
    sel.as_mut().expect("selection seeded").head = cursor.char_index;
}

/// Seed the selection anchor from the current cursor when no selection exists.
/// After this, `sel` is `Some` with `anchor == head == cursor.char_index`.
fn seed_anchor(sel: &mut Option<Selection>, cursor: &CursorState) {
    if sel.is_none() {
        *sel = Some(Selection {
            anchor: cursor.char_index,
            head: cursor.char_index,
        });
    }
}
