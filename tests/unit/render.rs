// Unit tests for editor::render footer formatting (revision 2026-07-02).
// The footer is a single row: filename + optional ` (*)` on the LEFT,
// status message on the RIGHT, padded to terminal width.
// Covers FR-001 (filename left), FR-003 (message right), FR-002 (1 row),
// and the edge cases named in spec 005 (truncation, small terminals).

use gobo::editor::render::format_footer_line;

/// A short clean name fills the row, name on the left, message on the right.
#[test]
fn clean_name_left_message_right() {
    let footer = format_footer_line("something.txt", false, "Ready", 40);
    assert!(footer.starts_with("something.txt"));
    assert!(footer.ends_with("Ready"));
    assert_eq!(footer.len(), 40);
}

/// A dirty file appends the ` (*)` marker immediately after the path (left).
#[test]
fn dirty_path_appends_dirty_marker_on_left() {
    let footer = format_footer_line("something.txt", true, "Ready", 80);
    assert!(footer.starts_with("something.txt (*)"));
    assert!(footer.contains("(*)"));
}

/// Spec US1 scenario 2: a relative path is shown verbatim on the left.
#[test]
fn relative_path_shown_verbatim_left() {
    let footer = format_footer_line("somedir/something.txt", false, "Ready", 80);
    assert!(footer.starts_with("somedir/something.txt"));
}

/// When the path (+optional marker) exceeds terminal width, the path is
/// truncated from the left with a `...` prefix while the dirty marker stays.
#[test]
fn long_dirty_path_truncates_left_and_keeps_marker() {
    let path = "very/long/path/that/does/not/fit/at/all/example.txt";
    let footer = format_footer_line(path, true, "Ready", 30);
    assert!(footer.starts_with("..."));
    assert!(footer.ends_with(" (*)"));
}

/// A clean (non-dirty) overflowing path also truncates from the left.
#[test]
fn long_clean_path_truncates_left() {
    let path = "very/long/path/that/does/not/fit/at/all/example.txt";
    let footer = format_footer_line(path, false, "Ready", 30);
    assert!(footer.starts_with("..."));
    assert!(!footer.contains("(*)"));
}

/// The computed visual width of the footer never exceeds terminal width,
/// even for long names and/or messages.
#[test]
fn footer_width_never_exceeds_terminal() {
    use unicode_width::UnicodeWidthStr;
    let path = "x".repeat(500);
    for dirty in [false, true] {
        let footer = format_footer_line(&path, dirty, "a long status message", 40);
        assert!(
            UnicodeWidthStr::width(footer.as_str()) <= 40,
            "footer width overflowed terminal for dirty={dirty}: {footer:?}"
        );
    }
}

/// Narrow-terminal edge case: terminal at least fits the `...` prefix plus
/// the dirty marker — the footer should still fit cleanly without panic.
#[test]
fn very_narrow_terminal_does_not_panic_and_fits() {
    use unicode_width::UnicodeWidthStr;
    let footer = format_footer_line("name.txt", true, "Ready", 7);
    assert_eq!(
        UnicodeWidthStr::width(footer.as_str()),
        7,
        "narrow terminal footer should fit exactly, got: {footer:?}"
    );
}

/// When the message is too long to fit beside the filename it is dropped,
/// leaving the (possibly padded) name alone on the row.
#[test]
fn overlong_message_is_dropped_name_remains() {
    use unicode_width::UnicodeWidthStr;
    let footer = format_footer_line(
        "note.md",
        false,
        "this is an extremely long message that cannot possibly fit",
        20,
    );
    // Name still on the left, message not present.
    assert!(footer.starts_with("note.md"));
    assert!(!footer.contains("extremely"));
    assert_eq!(UnicodeWidthStr::width(footer.as_str()), 20);
}

/// An empty path renders the message on the right (left filled with padding).
#[test]
fn empty_path_renders_message_right() {
    let footer = format_footer_line("", false, "Ready", 10);
    assert!(footer.ends_with("Ready"));
    assert_eq!(footer.len(), 10);
}

/// Spec 005 US1 scenario 3: an absolute path is shown verbatim on the left.
/// `document.path` is stored without canonicalization, so `.display()` reproduces
/// the argument the user typed on the command line.
#[test]
fn absolute_path_shown_verbatim_left() {
    use unicode_width::UnicodeWidthStr;
    let footer = format_footer_line("/Users/foo/bar.txt", false, "Ready", 80);
    assert!(
        footer.starts_with("/Users/foo/bar.txt"),
        "absolute path should appear verbatim on the left: {footer:?}"
    );
    assert!(footer.ends_with("Ready"));
    assert_eq!(UnicodeWidthStr::width(footer.as_str()), 80);
}

/// Spec 005 US1 scenario 6 (secondary states): a non-`Ready` status message —
/// e.g. a search outcome like "No match for foo" — must appear on the RIGHT of
/// the footer row alongside the filename on the left.
#[test]
fn non_ready_message_appears_on_right() {
    use unicode_width::UnicodeWidthStr;
    let footer =
        format_footer_line("bar.txt", false, "No match for foo", 80);
    assert!(
        footer.starts_with("bar.txt"),
        "filename still leads on the left: {footer:?}"
    );
    assert!(
        footer.ends_with("No match for foo"),
        "non-Ready status message must surface on the right: {footer:?}"
    );
    assert_eq!(UnicodeWidthStr::width(footer.as_str()), 80);
}

// ---- Centered "Ctrl-Q: Quit, Ctrl-H: Help" hint (width-dependent) ----
//
// When the terminal is wide enough, a centered hint is shown BETWEEN the
// filename (left) and the status message (right). Design rules pinned by the
// tests below:
//   * hint text  = `Ctrl-Q: Quit, Ctrl-H: Help`  (visual width = 26)
//   * centering  = full terminal width: hint_start == (width - 26) / 2
//   * fits IFF   hint_start >= name_width + 1
//                 AND hint_start + 26 + 1 <= width - msg_width
//                 (at least one space of separation on each side)
//   * drop order = the hint is dropped FIRST (no replacement); LEFT (name)
//                  and RIGHT (message) keep today's two-region layout.

/// On a wide terminal the hint appears centered between filename and message.
#[test]
fn wide_terminal_shows_centered_hint_between_name_and_message() {
    use unicode_width::UnicodeWidthStr;
    let footer = format_footer_line("foo.txt", false, "Ready", 80);
    assert!(
        footer.contains("Ctrl-Q: Quit, Ctrl-H: Help"),
        "centered hint missing: {footer:?}"
    );
    assert!(
        footer.starts_with("foo.txt"),
        "filename must still lead on the left: {footer:?}"
    );
    assert!(
        footer.ends_with("Ready"),
        "status message must still end the row on the right: {footer:?}"
    );
    assert_eq!(
        UnicodeWidthStr::width(footer.as_str()),
        80,
        "footer must fill the terminal width exactly: {footer:?}"
    );
    // Hint is centered in the full width: (80 - 26) / 2 == 27.
    assert_eq!(
        footer.find("Ctrl-Q: Quit, Ctrl-H: Help"),
        Some(27),
        "hint must be centered in the full terminal width: {footer:?}"
    );
}

/// Pin the exact 80-col clean layout: name | pad | hint | pad | message.
#[test]
fn wide_footer_exact_layout_name_hint_message() {
    let expected = format!(
        "{}{}{}{}{}",
        "foo.txt",
        " ".repeat(20),
        "Ctrl-Q: Quit, Ctrl-H: Help",
        " ".repeat(22),
        "Ready"
    );
    let footer = format_footer_line("foo.txt", false, "Ready", 80);
    assert_eq!(footer, expected);
}

/// The dirty marker stays glued to the filename on the left; the hint still fits.
#[test]
fn wide_terminal_shows_hint_alongside_dirty_marker() {
    use unicode_width::UnicodeWidthStr;
    let footer = format_footer_line("foo.txt", true, "Ready", 80);
    assert!(
        footer.starts_with("foo.txt (*)"),
        "dirty marker must stay glued to the filename: {footer:?}"
    );
    assert!(footer.contains("Ctrl-Q: Quit, Ctrl-H: Help"));
    assert!(footer.ends_with("Ready"));
    assert_eq!(UnicodeWidthStr::width(footer.as_str()), 80);
}

/// Centering tracks the terminal width: a 100-col terminal centers the hint at col 37.
#[test]
fn hint_center_position_tracks_terminal_width() {
    let footer = format_footer_line("foo.txt", false, "Ready", 100);
    assert_eq!(footer.find("Ctrl-Q: Quit, Ctrl-H: Help"), Some(37));
}

/// Boundary: the smallest width where the hint still fits for this name/message.
/// "foo.txt" (7) + 1 gap + hint (26) + 1 gap + "Ready" (5) needs >= 42 cols;
/// at width 42 the hint starts at col (42 - 26) / 2 == 8.
#[test]
fn hint_just_fits_at_width_boundary() {
    let footer = format_footer_line("foo.txt", false, "Ready", 42);
    assert!(
        footer.contains("Ctrl-Q: Quit, Ctrl-H: Help"),
        "hint should still fit at width 42: {footer:?}"
    );
    assert_eq!(footer.find("Ctrl-Q: Quit, Ctrl-H: Help"), Some(8));
}

/// One column below the boundary the hint is dropped, leaving name | message.
#[test]
fn hint_dropped_one_column_below_boundary() {
    let footer = format_footer_line("foo.txt", false, "Ready", 41);
    assert!(
        !footer.contains("Ctrl-Q"),
        "hint should be dropped at width 41: {footer:?}"
    );
    assert!(footer.starts_with("foo.txt"));
    assert!(footer.ends_with("Ready"));
}

/// On a narrow terminal the hint is dropped; today's two-region layout remains.
#[test]
fn narrow_terminal_drops_hint_keeps_name_and_message() {
    let footer = format_footer_line("foo.txt", false, "Ready", 40);
    assert!(!footer.contains("Ctrl-Q"));
    assert!(footer.starts_with("foo.txt"));
    assert!(footer.ends_with("Ready"));
}

/// Even on a wide-ish terminal, a long message colliding with the centered
/// hint causes the hint to be dropped (the message stays on the right).
#[test]
fn long_message_colliding_with_hint_drops_hint() {
    use unicode_width::UnicodeWidthStr;
    let msg = "x".repeat(20);
    let footer = format_footer_line("a.txt", false, &msg, 50);
    assert!(
        !footer.contains("Ctrl-Q"),
        "hint should be dropped when the message collides with it: {footer:?}"
    );
    assert!(footer.starts_with("a.txt"));
    assert!(footer.ends_with(&msg));
    assert_eq!(UnicodeWidthStr::width(footer.as_str()), 50);
}

// ---- Selection highlight projection (spec 007 T013) ----

use gobo::app::EditingSession;
use gobo::editor::cursor::Selection;
use gobo::editor::render::{HighlightSpan, TerminalSize};

#[test]
fn render_view_emits_highlight_spans_for_visible_selection() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("hl.txt");
    std::fs::write(&path, "Hallo Welt\n").unwrap();
    std::mem::forget(dir);
    let mut session = EditingSession::open(&path, TerminalSize::new(40, 8)).unwrap();
    // Select "llo" -> chars [2,5): anchor=2, head=4 (inclusive, block-cursor model).
    session.selection = Some(Selection { anchor: 2, head: 4 });

    let view = session.render_view();
    assert!(!view.body_lines.is_empty());
    assert_eq!(
        view.body_lines[0].highlights,
        vec![HighlightSpan { start_col: 2, end_col: 5 }]
    );
    // Other visible lines have no highlights.
    for line in view.body_lines.iter().skip(1) {
        assert!(line.highlights.is_empty(), "unexpected highlight: {:?}", line.highlights);
    }
}

#[test]
fn render_view_selection_backward_maps_to_same_forward_span() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("hl2.txt");
    std::fs::write(&path, "abcdef\n").unwrap();
    std::mem::forget(dir);
    let mut session = EditingSession::open(&path, TerminalSize::new(40, 8)).unwrap();
    // Backward selection over [1,4).
    session.selection = Some(Selection { anchor: 4, head: 1 });

    let view = session.render_view();
    assert_eq!(
        view.body_lines[0].highlights,
        vec![HighlightSpan { start_col: 1, end_col: 4 }]
    );
}

#[test]
fn render_view_no_selection_means_empty_highlights() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("hl3.txt");
    std::fs::write(&path, "abc\n").unwrap();
    std::mem::forget(dir);
    let session = EditingSession::open(&path, TerminalSize::new(40, 8)).unwrap();

    let view = session.render_view();
    for line in &view.body_lines {
        assert!(line.highlights.is_empty());
    }
}

#[test]
fn render_view_empty_selection_emits_no_highlight() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("hl4.txt");
    std::fs::write(&path, "abcdef\n").unwrap();
    std::mem::forget(dir);
    let mut session = EditingSession::open(&path, TerminalSize::new(40, 8)).unwrap();
    // Empty selection (anchor == head) -> no highlight (FR-008).
    session.selection = Some(Selection { anchor: 2, head: 2 });

    let view = session.render_view();
    assert!(view.body_lines[0].highlights.is_empty());
}

// ---- Spec 008 / User Story 1: render draws the cursor on the empty trailing line ----
// `render_view` derives `cursor_y` from `buffer::line_of_char` and `cursor_x`
// from `cursor::visual_column` (which itself uses `line_of_char`), so the
// `line_of_char` fix cascades to the drawn cursor with no render-side change.

/// At end-of-doc of a file ending in `\n`, `render_view` draws the cursor at
/// column 0 of the empty trailing line -- not at the end of the `abc` line
/// (FR-001). The next insert therefore lands where the cursor is shown.
#[test]
fn render_view_cursor_at_doc_end_after_trailing_newline_is_on_empty_line() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("trailing.txt");
    std::fs::write(&path, "abc\n").unwrap();
    std::mem::forget(dir);
    let mut session = EditingSession::open(&path, TerminalSize::new(40, 8)).unwrap();
    // Cursor exactly behind the trailing newline (end of document).
    session.cursor.char_index = 4;
    session.cursor.preferred_column = 0;

    let view = session.render_view();

    assert_eq!(view.cursor_y, 1, "cursor should be on the empty trailing line (line 1)");
    assert_eq!(view.cursor_x, 0, "cursor column should be 0 of the empty trailing line");
}
