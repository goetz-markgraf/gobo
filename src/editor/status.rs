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

/// A single row in the help dialog overlay: key binding + label.
#[derive(Clone, Debug)]
pub struct HelpDialogRow {
    pub key: String,
    pub label: String,
}

/// Build the flat list of all 9 active Ctrl-key bindings for the Help Dialog.
/// Display order matches contracts/key-bindings.md EXACTLY.
pub fn build_help_rows() -> Vec<HelpDialogRow> {
    vec![
        HelpDialogRow { key: String::from("Ctrl-F"), label: String::from("Find in document") },
        HelpDialogRow { key: String::from("Ctrl-G"), label: String::from("Find next match") },
        HelpDialogRow { key: String::from("Ctrl-S"), label: String::from("Save document") },
        HelpDialogRow { key: String::from("Ctrl-Z"), label: String::from("Undo last edit") },
        HelpDialogRow { key: String::from("Ctrl-Y"), label: String::from("Redo last undone edit") },
        HelpDialogRow { key: String::from("Ctrl-C"), label: String::from("Copy selection to clipboard") },
        HelpDialogRow { key: String::from("Ctrl-X"), label: String::from("Cut selection to clipboard") },
        HelpDialogRow { key: String::from("Ctrl-V"), label: String::from("Paste from clipboard") },
        HelpDialogRow { key: String::from("Ctrl-Q"), label: String::from("Quit (clean) / save prompt (dirty)") },
    ]
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
                popup_rows: vec![],
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
                popup_rows: vec![],
            }
        }
        PromptState::HelpDialog => build_help_popup(terminal_size, session),
    })
}

fn build_help_popup(size: TerminalSize, session: &EditingSession) -> PopupView {
    let rows = build_help_rows();
    let mut variant = PromptVariant::Full;

    // Build all data rows with fixed-width key column (14 chars right-padded)
    let all_rows: Vec<String> = rows
        .iter()
        .map(|r| {
            let klen = unicode_width::UnicodeWidthStr::width(r.key.as_str());
            if klen < 14 {
                format!("{}{}", r.key, " ".repeat(14 - klen)) + " " + &r.label
            } else {
                format!("{}{}", r.key, " ") + &r.label
            }
        })
        .collect();

    let rows_len = all_rows.len();

    // Popup height = content rows + title/blank/footer hint (constant 4 extra rows).
    // Clamped to available space so it never overflows the terminal.
    let desired_height: usize = help_desired_height(size);

    if size.height < 14 {
        variant = if size.height < 8 { PromptVariant::Compact } else { PromptVariant::Full };
    }

    // Available body slots + max scroll offset share one source of truth so the
    // scroll clamping in `app.rs` can never diverge from the rendered layout.
    let body_capacity: usize = help_body_capacity(size);
    let max_offset: usize = help_max_offset(size);
    let offset = session.help_scroll_offset.min(max_offset);
    let end = (offset + body_capacity).min(rows_len);

    let popup_rows: Vec<String> = if offset < end {
        all_rows[offset..end].to_vec()
    } else {
        vec![]
    };

    PopupView {
        variant,
        title: "Keyboard Shortcuts".to_string(),
        message: None,
        actions: vec![], // no buttons for help dialog
        help_text: "↑/↓ Scroll · Enter/Esc Close".to_string(),
        rect: PopupRect {
            x: size.width.saturating_sub(52.min(size.width)) / 2,
            y: (size.height - desired_height as u16).saturating_sub(1) / 2,
            width: 52.min(size.width),
            height: desired_height as u16,
        },
        popup_rows,
    }
}


fn popup_variant(size: TerminalSize) -> PromptVariant {
    if size.width < 44 || size.height < 8 {
        PromptVariant::Compact
    } else {
        PromptVariant::Full
    }
}

/// Total popup height (incl. borders) for the help dialog at a given terminal
/// size. Shared by `build_help_popup` (rect height) and `help_body_capacity`
/// so they can never diverge.
pub fn help_desired_height(size: TerminalSize) -> usize {
    let rows_len = 9usize; // always 9 key-binding rows (FR-011)
    // rows + 6 = borders(2) + title(1) + blank(1) + blank(1) + footer(1).
    (rows_len + 6).min(size.height.max(1) as usize - 1)
}

/// Number of key-binding rows visible inside the help popup for a given
/// terminal size. Header (title) and footer (Enter/Esc hint) are ALWAYS
/// visible — only the shortcut rows scroll (FR-004). Min 1 row so even on a
/// very small terminal at least one shortcut stays visible.
pub fn help_body_capacity(size: TerminalSize) -> usize {
    // inner height = desired_height - 2 (top+bottom borders); reserve 4 lines
    // for title, the two blank separators and the footer hint.
    help_desired_height(size).saturating_sub(6).max(1)
}

/// Maximum valid scroll offset for the help dialog given a terminal size.
/// Derived from `help_body_capacity` so clamping in `app.rs` matches the
/// rendered popup exactly (prevents dead scroll zones after resize).
pub fn help_max_offset(size: TerminalSize) -> usize {
    let rows_len = 9usize;
    let body_capacity = help_body_capacity(size);
    rows_len.saturating_sub(body_capacity)
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
