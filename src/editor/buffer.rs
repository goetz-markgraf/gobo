use ropey::Rope;
use std::ops::Range;

pub fn clamp_char_index(text: &Rope, char_index: usize) -> usize {
    char_index.min(text.len_chars())
}

pub fn insert_text(text: &mut Rope, char_index: usize, input: &str) -> usize {
    let char_index = clamp_char_index(text, char_index);
    text.insert(char_index, input);
    char_index + input.chars().count()
}

pub fn remove_char_before(text: &mut Rope, char_index: usize) -> Option<usize> {
    let char_index = clamp_char_index(text, char_index);
    if char_index == 0 {
        return None;
    }

    text.remove(char_index - 1..char_index);
    Some(char_index - 1)
}

pub fn delete_char_at(text: &mut Rope, char_index: usize) -> bool {
    let char_index = clamp_char_index(text, char_index);
    if char_index >= text.len_chars() {
        return false;
    }

    text.remove(char_index..char_index + 1);
    true
}

pub fn replace_range(text: &mut Rope, range: Range<usize>, replacement: &str) -> usize {
    let start = clamp_char_index(text, range.start);
    let end = clamp_char_index(text, range.end.max(start));
    text.remove(start..end);
    text.insert(start, replacement);
    start + replacement.chars().count()
}

pub fn line_count(text: &Rope) -> usize {
    text.len_lines().max(1)
}

pub fn line_of_char(text: &Rope, char_index: usize) -> usize {
    if text.len_chars() == 0 {
        return 0;
    }

    let char_index = clamp_char_index(text, char_index);
    let probe = if char_index == text.len_chars() {
        char_index.saturating_sub(1)
    } else {
        char_index
    };

    text.char_to_line(probe)
}

pub fn line_start_char(text: &Rope, line_index: usize) -> usize {
    if text.len_lines() == 0 {
        return 0;
    }

    let line_index = line_index.min(text.len_lines().saturating_sub(1));
    text.line_to_char(line_index)
}

pub fn line_content(text: &Rope, line_index: usize) -> String {
    if line_index >= text.len_lines() {
        return String::new();
    }

    let mut line = text.line(line_index).to_string();
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
    line
}

pub fn line_len_chars(text: &Rope, line_index: usize) -> usize {
    line_content(text, line_index).chars().count()
}

pub fn char_index_from_line_column(text: &Rope, line_index: usize, column_chars: usize) -> usize {
    let start = line_start_char(text, line_index);
    start + line_len_chars(text, line_index).min(column_chars)
}
