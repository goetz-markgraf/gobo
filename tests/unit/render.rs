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
