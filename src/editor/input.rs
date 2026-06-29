use crate::editor::render::TerminalSize;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorCommand {
    InsertChar(char),
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    Backspace,
    Delete,
    Save,
    Quit,
    Search,
    Confirm,
    Cancel,
    NextChoice,
    PreviousChoice,
    Resize(TerminalSize),
}

pub fn map_key_event(key: KeyEvent) -> Option<EditorCommand> {
    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('s')) => Some(EditorCommand::Save),
        (KeyModifiers::CONTROL, KeyCode::Char('q')) => Some(EditorCommand::Quit),
        (KeyModifiers::CONTROL, KeyCode::Char('f')) => Some(EditorCommand::Search),
        (_, KeyCode::Left) => Some(EditorCommand::MoveLeft),
        (_, KeyCode::Right) => Some(EditorCommand::MoveRight),
        (_, KeyCode::Up) => Some(EditorCommand::MoveUp),
        (_, KeyCode::Down) => Some(EditorCommand::MoveDown),
        (_, KeyCode::Backspace) => Some(EditorCommand::Backspace),
        (_, KeyCode::Delete) => Some(EditorCommand::Delete),
        (_, KeyCode::Enter) => Some(EditorCommand::Confirm),
        (_, KeyCode::Esc) => Some(EditorCommand::Cancel),
        (_, KeyCode::Tab) => Some(EditorCommand::NextChoice),
        (_, KeyCode::BackTab) => Some(EditorCommand::PreviousChoice),
        (_, KeyCode::Char(c)) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(EditorCommand::InsertChar(c))
        }
        _ => None,
    }
}
