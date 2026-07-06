use crate::editor::buffer;
use ropey::Rope;
use std::ops::Range;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IndentCommand {
    Tab,
    Enter,
    Backspace,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndentActionPlan {
    pub command: IndentCommand,
    pub replace_start: usize,
    pub replace_end: usize,
    pub inserted_text: String,
}

impl IndentActionPlan {
    pub fn replace_range(&self) -> Range<usize> {
        self.replace_start..self.replace_end
    }
}

pub fn line_start_column(text: &Rope, char_index: usize) -> usize {
    let clamped = buffer::clamp_char_index(text, char_index);
    let line = buffer::line_of_char(text, clamped);
    let line_start = buffer::line_start_char(text, line);
    clamped.saturating_sub(line_start)
}

pub fn tab_width_for_column(column: usize) -> usize {
    if column % 2 == 0 { 2 } else { 1 }
}

pub fn tab_text_for_column(column: usize) -> String {
    " ".repeat(tab_width_for_column(column))
}

pub fn leading_spaces(text: &str) -> usize {
    text.chars().take_while(|c| *c == ' ').count()
}

pub fn leading_indent_text(text: &Rope, char_index: usize) -> String {
    let clamped = buffer::clamp_char_index(text, char_index);
    let line = buffer::line_of_char(text, clamped);
    let line_text = buffer::line_content(text, line);
    " ".repeat(leading_spaces(&line_text))
}

pub fn enter_text(text: &Rope, char_index: usize) -> String {
    format!("\n{}", leading_indent_text(text, char_index))
}

pub fn line_prefix(text: &Rope, char_index: usize) -> String {
    let clamped = buffer::clamp_char_index(text, char_index);
    let line = buffer::line_of_char(text, clamped);
    let line_start = buffer::line_start_char(text, line);
    text.slice(line_start..clamped).to_string()
}

pub fn is_all_spaces_prefix(prefix: &str) -> bool {
    prefix.chars().all(|c| c == ' ')
}

pub fn smart_backspace_width(prefix: &str) -> usize {
    if prefix.is_empty() || !is_all_spaces_prefix(prefix) {
        return 0;
    }

    if prefix.chars().count() % 2 == 0 { 2 } else { 1 }
}

pub fn plan_tab(text: &Rope, replace_start: usize, replace_end: usize) -> IndentActionPlan {
    let replace_start = buffer::clamp_char_index(text, replace_start);
    let replace_end = buffer::clamp_char_index(text, replace_end).max(replace_start);
    let column = line_start_column(text, replace_start);
    IndentActionPlan {
        command: IndentCommand::Tab,
        replace_start,
        replace_end,
        inserted_text: tab_text_for_column(column),
    }
}

pub fn plan_enter(text: &Rope, replace_start: usize, replace_end: usize) -> IndentActionPlan {
    let replace_start = buffer::clamp_char_index(text, replace_start);
    let replace_end = buffer::clamp_char_index(text, replace_end).max(replace_start);
    IndentActionPlan {
        command: IndentCommand::Enter,
        replace_start,
        replace_end,
        inserted_text: enter_text(text, replace_start),
    }
}

pub fn plan_backspace(
    text: &Rope,
    replace_start: usize,
    replace_end: usize,
) -> Option<IndentActionPlan> {
    let replace_start = buffer::clamp_char_index(text, replace_start);
    let replace_end = buffer::clamp_char_index(text, replace_end).max(replace_start);
    let prefix = line_prefix(text, replace_start);
    let width = smart_backspace_width(&prefix);

    if width == 0 {
        if replace_start == replace_end {
            return None;
        }

        return Some(IndentActionPlan {
            command: IndentCommand::Backspace,
            replace_start: replace_start.saturating_sub(1),
            replace_end,
            inserted_text: String::new(),
        });
    }

    Some(IndentActionPlan {
        command: IndentCommand::Backspace,
        replace_start: replace_start.saturating_sub(width),
        replace_end,
        inserted_text: String::new(),
    })
}
