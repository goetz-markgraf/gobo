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
    // Select "llo" -> chars [2,5) on line 0.
    session.selection = Some(Selection { anchor: 2, head: 5 });

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
