use gobo::editor::indent;
use ropey::Rope;

#[test]
fn tab_width_depends_on_column_parity() {
    assert_eq!(indent::tab_width_for_column(0), 2);
    assert_eq!(indent::tab_width_for_column(1), 1);
    assert_eq!(indent::tab_width_for_column(2), 2);
    assert_eq!(indent::tab_width_for_column(3), 1);
}

#[test]
fn line_start_column_counts_raw_chars_from_line_start() {
    let text = Rope::from_str("ab\nxyz\n");
    assert_eq!(indent::line_start_column(&text, 0), 0);
    assert_eq!(indent::line_start_column(&text, 1), 1);
    assert_eq!(indent::line_start_column(&text, 2), 2);
    assert_eq!(indent::line_start_column(&text, 3), 0);
    assert_eq!(indent::line_start_column(&text, 5), 2);
}

#[test]
fn plan_tab_inserts_spaces_to_next_even_column() {
    let text = Rope::from_str("abc");

    let even = indent::plan_tab(&text, 0, 0);
    assert_eq!(even.replace_start, 0);
    assert_eq!(even.replace_end, 0);
    assert_eq!(even.inserted_text, "  ");

    let odd = indent::plan_tab(&text, 1, 1);
    assert_eq!(odd.replace_start, 1);
    assert_eq!(odd.replace_end, 1);
    assert_eq!(odd.inserted_text, " ");
}

#[test]
fn leading_spaces_only_count_ascii_spaces() {
    assert_eq!(indent::leading_spaces("    hello"), 4);
    assert_eq!(indent::leading_spaces("hello"), 0);
    assert_eq!(indent::leading_spaces("\thello"), 0);
    assert_eq!(indent::leading_spaces("  \thello"), 2);
}

#[test]
fn plan_enter_uses_current_lines_leading_spaces() {
    let text = Rope::from_str("    hello\n  world");

    let first_line = indent::plan_enter(&text, 6, 6);
    assert_eq!(first_line.inserted_text, "\n    ");

    let second_line = indent::plan_enter(&text, 14, 14);
    assert_eq!(second_line.inserted_text, "\n  ");
}

#[test]
fn all_space_prefix_detection_and_smart_backspace_width_follow_rules() {
    assert!(indent::is_all_spaces_prefix(""));
    assert!(indent::is_all_spaces_prefix("   "));
    assert!(!indent::is_all_spaces_prefix("  a"));

    assert_eq!(indent::smart_backspace_width(""), 0);
    assert_eq!(indent::smart_backspace_width("   "), 1);
    assert_eq!(indent::smart_backspace_width("    "), 2);
    assert_eq!(indent::smart_backspace_width("  a"), 0);
}

#[test]
fn plan_backspace_removes_to_previous_even_column() {
    let text = Rope::from_str("    hello");

    let odd = indent::plan_backspace(&text, 3, 3).expect("odd indent plan");
    assert_eq!(odd.replace_start, 2);
    assert_eq!(odd.replace_end, 3);
    assert_eq!(odd.inserted_text, "");

    let even = indent::plan_backspace(&text, 4, 4).expect("even indent plan");
    assert_eq!(even.replace_start, 2);
    assert_eq!(even.replace_end, 4);
    assert_eq!(even.inserted_text, "");
}
