use crate::app::{ConflictChoice, EditingSession, PromptState, SessionMode, UnsavedChoice};
use crate::document::AccessMode;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StatusKind {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusMessage {
    pub kind: StatusKind,
    pub text: String,
}

impl StatusMessage {
    pub fn info(text: impl Into<String>) -> Self {
        Self {
            kind: StatusKind::Info,
            text: text.into(),
        }
    }

    pub fn success(text: impl Into<String>) -> Self {
        Self {
            kind: StatusKind::Success,
            text: text.into(),
        }
    }

    pub fn warning(text: impl Into<String>) -> Self {
        Self {
            kind: StatusKind::Warning,
            text: text.into(),
        }
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self {
            kind: StatusKind::Error,
            text: text.into(),
        }
    }
}

pub fn format_status_line(session: &EditingSession) -> String {
    let path = session.document.path.display();
    let access = match session.document.access_mode {
        AccessMode::Editable => "EDIT",
        AccessMode::ReadOnly => "READ-ONLY",
    };
    let dirty = if session.document.dirty { "DIRTY" } else { "CLEAN" };
    let mode = match session.mode {
        SessionMode::Editing => "editing",
        SessionMode::SearchInput => "search",
        SessionMode::ConfirmQuit => "confirm-quit",
        SessionMode::SaveConflictPrompt => "save-conflict",
        SessionMode::Exiting => "exiting",
    };
    let message = session
        .status
        .as_ref()
        .map(|status| status.text.as_str())
        .unwrap_or("Ready");

    format!("{} | {} | {} | {} | {}", path, access, dirty, mode, message)
}

pub fn prompt_line(prompt: &PromptState) -> String {
    match prompt {
        PromptState::UnsavedChanges { focus, .. } => format!(
            "Unsaved changes: {} {} {}  (←/→, Enter, Esc)",
            focus_label("Save", matches!(focus, UnsavedChoice::Save)),
            focus_label("Discard", matches!(focus, UnsavedChoice::Discard)),
            focus_label("Cancel", matches!(focus, UnsavedChoice::Cancel)),
        ),
        PromptState::SaveConflict { focus, .. } => format!(
            "File changed on disk: {} {} {}  (←/→, Enter, Esc)",
            focus_label("Reload", matches!(focus, ConflictChoice::Reload)),
            focus_label("Overwrite", matches!(focus, ConflictChoice::Overwrite)),
            focus_label("Cancel", matches!(focus, ConflictChoice::Cancel)),
        ),
    }
}

pub fn search_prompt(session: &EditingSession) -> Option<String> {
    if session.mode != SessionMode::SearchInput {
        return None;
    }

    let query = session
        .search
        .as_ref()
        .map(|search| search.query.as_str())
        .unwrap_or("");
    Some(format!("Search: {}", query))
}

fn focus_label(label: &str, focused: bool) -> String {
    if focused {
        format!("[{}]", label)
    } else {
        label.to_string()
    }
}
