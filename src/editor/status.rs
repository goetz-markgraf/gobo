use crate::app::{ConflictChoice, EditingSession, PromptState, SessionMode, UnsavedChoice};
use crate::editor::render::{PopupRect, PopupView, PromptActionLabel, PromptVariant, TerminalSize};

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

/// The human-readable status message for the current session, shown on the
/// right of the footer row. Defaults to `Ready` when no status is set.
pub fn current_message(session: &EditingSession) -> String {
    session
        .status
        .as_ref()
        .map(|status| status.text.clone())
        .unwrap_or_else(|| "Ready".to_string())
}

pub fn popup_view(session: &EditingSession, terminal_size: TerminalSize) -> Option<PopupView> {
    let prompt = session.pending_prompt.as_ref()?;
    Some(match prompt {
        PromptState::UnsavedChanges { focus, .. } => {
            let variant = popup_variant(terminal_size);
            let (title, message, labels) = match variant {
                PromptVariant::Full => (
                    "Unsaved changes".to_string(),
                    Some("Save before quitting?".to_string()),
                    ["Save", "Discard", "Cancel"],
                ),
                PromptVariant::Compact => ("Unsaved".to_string(), None, ["Save", "Drop", "Back"]),
            };
            let rect = popup_rect(terminal_size, variant, message.is_some());
            PopupView {
                variant,
                title,
                message,
                actions: vec![
                    PromptActionLabel {
                        label: focus_label(labels[0], matches!(focus, UnsavedChoice::Save)),
                        focused: matches!(focus, UnsavedChoice::Save),
                    },
                    PromptActionLabel {
                        label: focus_label(labels[1], matches!(focus, UnsavedChoice::Discard)),
                        focused: matches!(focus, UnsavedChoice::Discard),
                    },
                    PromptActionLabel {
                        label: focus_label(labels[2], matches!(focus, UnsavedChoice::Cancel)),
                        focused: matches!(focus, UnsavedChoice::Cancel),
                    },
                ],
                help_text: "←/→, Tab, Enter, Esc".to_string(),
                rect,
            }
        }
        PromptState::SaveConflict { focus, .. } => {
            let variant = popup_variant(terminal_size);
            let (title, message, labels) = match variant {
                PromptVariant::Full => (
                    "File changed on disk".to_string(),
                    Some("Choose how to continue saving.".to_string()),
                    ["Reload", "Overwrite", "Cancel"],
                ),
                PromptVariant::Compact => {
                    ("Conflict".to_string(), None, ["Reload", "Write", "Back"])
                }
            };
            let rect = popup_rect(terminal_size, variant, message.is_some());
            PopupView {
                variant,
                title,
                message,
                actions: vec![
                    PromptActionLabel {
                        label: focus_label(labels[0], matches!(focus, ConflictChoice::Reload)),
                        focused: matches!(focus, ConflictChoice::Reload),
                    },
                    PromptActionLabel {
                        label: focus_label(labels[1], matches!(focus, ConflictChoice::Overwrite)),
                        focused: matches!(focus, ConflictChoice::Overwrite),
                    },
                    PromptActionLabel {
                        label: focus_label(labels[2], matches!(focus, ConflictChoice::Cancel)),
                        focused: matches!(focus, ConflictChoice::Cancel),
                    },
                ],
                help_text: "←/→, Tab, Enter, Esc".to_string(),
                rect,
            }
        }
    })
}

fn popup_variant(size: TerminalSize) -> PromptVariant {
    if size.width < 44 || size.height < 8 {
        PromptVariant::Compact
    } else {
        PromptVariant::Full
    }
}

fn popup_rect(size: TerminalSize, variant: PromptVariant, has_message: bool) -> PopupRect {
    let desired_width = match variant {
        PromptVariant::Full => size.width.min(52),
        PromptVariant::Compact => size.width.min(30),
    }
    .max(18);
    let desired_height = match (variant, has_message) {
        (PromptVariant::Full, true) => 7,
        (PromptVariant::Full, false) => 6,
        (PromptVariant::Compact, _) => 5,
    }
    .min(size.height.max(1));

    PopupRect {
        x: size.width.saturating_sub(desired_width) / 2,
        y: size.height.saturating_sub(desired_height) / 2,
        width: desired_width,
        height: desired_height,
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
