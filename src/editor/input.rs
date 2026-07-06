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
    Cancel,
    Enter,
    Tab,
    Undo,
    Redo,
    Copy,
    Cut,
    Paste,
    NextChoice,
    PreviousChoice,
    FindNext,
    Resize(TerminalSize),
    MoveSelectLeft,
    MoveSelectRight,
    MoveSelectUp,
    MoveSelectDown,
}

pub fn map_key_event(key: KeyEvent) -> Option<EditorCommand> {
    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('s')) => Some(EditorCommand::Save),
        (KeyModifiers::CONTROL, KeyCode::Char('q')) => Some(EditorCommand::Quit),
        (KeyModifiers::CONTROL, KeyCode::Char('f')) => Some(EditorCommand::Search),
        (KeyModifiers::CONTROL, KeyCode::Char('g')) => Some(EditorCommand::FindNext),
        (KeyModifiers::CONTROL, KeyCode::Char('z')) => Some(EditorCommand::Undo),
        (KeyModifiers::CONTROL, KeyCode::Char('y')) => Some(EditorCommand::Redo),
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => Some(EditorCommand::Copy),
        (KeyModifiers::CONTROL, KeyCode::Char('x')) => Some(EditorCommand::Cut),
        (KeyModifiers::CONTROL, KeyCode::Char('v')) => Some(EditorCommand::Paste),
        (KeyModifiers::SHIFT, KeyCode::Left) => Some(EditorCommand::MoveSelectLeft),
        (KeyModifiers::SHIFT, KeyCode::Right) => Some(EditorCommand::MoveSelectRight),
        (KeyModifiers::SHIFT, KeyCode::Up) => Some(EditorCommand::MoveSelectUp),
        (KeyModifiers::SHIFT, KeyCode::Down) => Some(EditorCommand::MoveSelectDown),
        (_, KeyCode::Left) => Some(EditorCommand::MoveLeft),
        (_, KeyCode::Right) => Some(EditorCommand::MoveRight),
        (_, KeyCode::Up) => Some(EditorCommand::MoveUp),
        (_, KeyCode::Down) => Some(EditorCommand::MoveDown),
        (_, KeyCode::Backspace) => Some(EditorCommand::Backspace),
        (_, KeyCode::Delete) => Some(EditorCommand::Delete),
        (_, KeyCode::Enter) => Some(EditorCommand::Enter),
        (_, KeyCode::Esc) => Some(EditorCommand::Cancel),
        (_, KeyCode::Tab) => Some(EditorCommand::Tab),
        (_, KeyCode::BackTab) => Some(EditorCommand::PreviousChoice),
        (_, KeyCode::Char(c)) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(EditorCommand::InsertChar(c))
        }
        _ => None,
    }
}
