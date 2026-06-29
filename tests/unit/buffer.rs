use gobo::editor::buffer::{delete_char_at, insert_text, line_content, line_count, line_of_char, remove_char_before, replace_range};
use ropey::Rope;

#[test]
fn insert_delete_and_replace_work_on_rope_buffer() {
    let mut text = Rope::from_str("hello");

    let cursor = insert_text(&mut text, 5, " world");
    assert_eq!(cursor, 11);
    assert_eq!(text.to_string(), "hello world");

    let cursor = remove_char_before(&mut text, cursor).unwrap();
    assert_eq!(cursor, 10);
    assert_eq!(text.to_string(), "hello worl");

    assert!(delete_char_at(&mut text, 5));
    assert_eq!(text.to_string(), "helloworl");

    let cursor = replace_range(&mut text, 5..9, " rust");
    assert_eq!(cursor, 10);
    assert_eq!(text.to_string(), "hello rust");
}

#[test]
fn line_helpers_track_lines_and_columns() {
    let text = Rope::from_str("alpha\nbeta\n");

    assert_eq!(line_count(&text), 3);
    assert_eq!(line_content(&text, 0), "alpha");
    assert_eq!(line_content(&text, 1), "beta");
    assert_eq!(line_of_char(&text, 0), 0);
    assert_eq!(line_of_char(&text, 6), 1);
}
