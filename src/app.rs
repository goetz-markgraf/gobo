//! High-level editing session state and command handling.

use crate::document::{DocumentBuffer, DocumentError, SaveResult};
use crate::editor::buffer;
use crate::editor::cursor::{self, CursorState};
use crate::editor::input::EditorCommand;
use crate::editor::render::{self, RenderView, TerminalSize, ViewportState};
use crate::editor::search::SearchState;
use crate::editor::status::StatusMessage;
use crate::editor::history::History;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionMode {
    Editing,
    SearchInput,
    ConfirmQuit,
    SaveConflictPrompt,
    Exiting,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PromptAction {
    Quit,
    ReloadDocument,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnsavedChoice {
    Save,
    Discard,
    Cancel,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConflictChoice {
    Reload,
    Overwrite,
    Cancel,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PromptState {
    UnsavedChanges {
        action: PromptAction,
        focus: UnsavedChoice,
    },
    SaveConflict {
        focus: ConflictChoice,
        resume_action: Option<PromptAction>,
    },
}

#[derive(Debug)]
pub struct EditingSession {
    pub document: DocumentBuffer,
    pub cursor: CursorState,
    pub viewport: ViewportState,
    pub mode: SessionMode,
    pub search: Option<SearchState>,
    pub status: Option<StatusMessage>,
    pub pending_prompt: Option<PromptState>,
    pub terminal_size: TerminalSize,
    /// Undo/redo history. Session-bound: in-memory only, never persisted,
    /// and initialized empty on every `new()`/`open()` (FR-008).
    pub history: History,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SaveDisposition {
    Saved,
    ReadOnlyBlocked,
    ConflictPrompted,
}

impl EditingSession {
    pub fn open(
        path: impl AsRef<Path>,
        terminal_size: TerminalSize,
    ) -> Result<Self, DocumentError> {
        let document = DocumentBuffer::open(path.as_ref().to_path_buf())?;
        Ok(Self::new(document, terminal_size))
    }

    pub fn new(document: DocumentBuffer, terminal_size: TerminalSize) -> Self {
        let mut session = Self {
            document,
            cursor: CursorState::default(),
            viewport: ViewportState::from_terminal(terminal_size, 1),
            mode: SessionMode::Editing,
            search: None,
            status: Some(StatusMessage::info("Ready")),
            pending_prompt: None,
            terminal_size,
            history: History::new(),
        };
        session.sync_viewport();
        if session.document.is_read_only() {
            session.status = Some(StatusMessage::warning("Opened in read-only mode"));
        }
        session
    }

    pub fn is_exiting(&self) -> bool {
        self.mode == SessionMode::Exiting
    }

    pub fn render_view(&self) -> RenderView {
        render::render_view(self)
    }

    pub fn handle_command(&mut self, command: EditorCommand) -> Result<(), DocumentError> {
        if let EditorCommand::Resize(size) = command {
            self.terminal_size = size;
            self.sync_viewport();
            self.status = Some(StatusMessage::info(format!(
                "Resized to {}x{}",
                size.width, size.height
            )));
            return Ok(());
        }

        if self.mode == SessionMode::Exiting {
            return Ok(());
        }

        if self.pending_prompt.is_some() {
            return self.handle_prompt_command(command);
        }

        match self.mode {
            SessionMode::Editing => self.handle_editing_command(command),
            SessionMode::SearchInput => self.handle_search_command(command),
            SessionMode::ConfirmQuit | SessionMode::SaveConflictPrompt | SessionMode::Exiting => {
                Ok(())
            }
        }
    }

    fn handle_editing_command(&mut self, command: EditorCommand) -> Result<(), DocumentError> {
        match command {
            EditorCommand::InsertChar(c) => self.insert_text(&c.to_string()),
            EditorCommand::Backspace => self.backspace(),
            EditorCommand::Delete => self.delete(),
            EditorCommand::MoveLeft => cursor::move_left(&mut self.cursor, &self.document.text),
            EditorCommand::MoveRight => cursor::move_right(&mut self.cursor, &self.document.text),
            EditorCommand::MoveUp => cursor::move_up(&mut self.cursor, &self.document.text),
            EditorCommand::MoveDown => cursor::move_down(&mut self.cursor, &self.document.text),
            EditorCommand::Save => {
                let _ = self.save_document(None)?;
            }
            EditorCommand::Quit => self.request_quit(),
            EditorCommand::Enter => self.insert_text("\n"),
            EditorCommand::Search => self.begin_search(),
            EditorCommand::Cancel | EditorCommand::FindNext => {}
            EditorCommand::Undo => self.undo(),
            EditorCommand::Redo => self.redo(),
            EditorCommand::NextChoice
            | EditorCommand::PreviousChoice
            | EditorCommand::Resize(_) => {}
        }

        // Handle FindNext from editing mode (cursor jumps to next search match)
        if let EditorCommand::FindNext = command {
            if let Some(ref mut search) = self.search {
                match search.query.is_empty() {
                    true => self.status = Some(StatusMessage::warning("No match")),
                    false => match search.find_next(&self.document.text, self.cursor.char_index) {
                        Some((start, end)) => {
                            self.cursor.char_index = start;
                            self.cursor.preferred_column =
                                cursor::visual_column(&self.document.text, start);
                            self.status =
                                Some(StatusMessage::info(format!("Match at {}..{}", start, end)));
                        }
                        None => {
                            self.status = Some(StatusMessage::warning("No match"));
                        }
                    },
                };
            } else {
                self.status = Some(StatusMessage::warning("No match"));
            }
        }

        self.sync_viewport();
        Ok(())
    }

    fn handle_search_command(&mut self, command: EditorCommand) -> Result<(), DocumentError> {
        let search = self.search.get_or_insert_with(SearchState::default);

        match command {
            EditorCommand::InsertChar(c) => {
                search.query.push(c);
                self.status = Some(StatusMessage::info("Search query updated"));
            }
            EditorCommand::Backspace => {
                search.query.pop();
                self.status = Some(StatusMessage::info("Search query updated"));
            }
            EditorCommand::Enter => {
                if search.query.is_empty() {
                    self.status = Some(StatusMessage::info("Search cancelled"));
                    self.mode = SessionMode::Editing;
                } else if let Some((start, end)) =
                    search.find_next(&self.document.text, self.cursor.char_index)
                {
                    self.cursor.char_index = start;
                    self.cursor.preferred_column =
                        cursor::visual_column(&self.document.text, start);
                    self.status = Some(StatusMessage::success(format!(
                        "Match found at {}..{}",
                        start, end
                    )));
                    self.mode = SessionMode::Editing;
                } else {
                    self.status = Some(StatusMessage::warning(format!(
                        "No match for {}",
                        search.query
                    )));
                    self.mode = SessionMode::Editing;
                }
            }
            EditorCommand::Cancel => {
                self.status = Some(StatusMessage::info("Search cancelled"));
                self.mode = SessionMode::Editing;
            }
            EditorCommand::Quit => self.request_quit(),
            EditorCommand::FindNext => {
                if search.query.is_empty() {
                    self.status = Some(StatusMessage::warning("No match"));
                } else {
                    let result = search.find_next(&self.document.text, self.cursor.char_index);
                    match result {
                        Some((start, end)) => {
                            self.cursor.char_index = start;
                            self.cursor.preferred_column =
                                cursor::visual_column(&self.document.text, start);
                            self.status =
                                Some(StatusMessage::info(format!("Match at {}..{}", start, end)));
                        }
                        None => {
                            self.status = Some(StatusMessage::warning("No match"));
                        }
                    }
                }
            }
            EditorCommand::MoveLeft
            | EditorCommand::MoveRight
            | EditorCommand::MoveUp
            | EditorCommand::MoveDown
            | EditorCommand::Delete
            | EditorCommand::Save
            | EditorCommand::Search
            | EditorCommand::Undo
            | EditorCommand::Redo
            | EditorCommand::NextChoice
            | EditorCommand::PreviousChoice
            | EditorCommand::Resize(_) => {}
        }
        self.sync_viewport();
        Ok(())
    }

    fn handle_prompt_command(&mut self, command: EditorCommand) -> Result<(), DocumentError> {
        let prompt = self.pending_prompt.clone().expect("prompt exists");

        match prompt {
            PromptState::UnsavedChanges { action, focus } => match command {
                EditorCommand::MoveLeft | EditorCommand::PreviousChoice => {
                    self.pending_prompt = Some(PromptState::UnsavedChanges {
                        action,
                        focus: previous_unsaved_choice(&focus),
                    });
                }
                EditorCommand::MoveRight | EditorCommand::NextChoice => {
                    self.pending_prompt = Some(PromptState::UnsavedChanges {
                        action,
                        focus: next_unsaved_choice(&focus),
                    });
                }
                EditorCommand::Enter => match focus {
                    UnsavedChoice::Save => {
                        if matches!(
                            self.save_document(Some(action.clone()))?,
                            SaveDisposition::Saved
                        ) {
                            self.mode = SessionMode::Exiting;
                        }
                    }
                    UnsavedChoice::Discard => {
                        self.pending_prompt = None;
                        self.mode = SessionMode::Exiting;
                    }
                    UnsavedChoice::Cancel => self.dismiss_prompt(),
                },
                EditorCommand::Cancel => self.dismiss_prompt(),
                _ => {}
            },
            PromptState::SaveConflict {
                focus,
                resume_action,
            } => match command {
                EditorCommand::MoveLeft | EditorCommand::PreviousChoice => {
                    self.pending_prompt = Some(PromptState::SaveConflict {
                        focus: previous_conflict_choice(&focus),
                        resume_action,
                    });
                }
                EditorCommand::MoveRight | EditorCommand::NextChoice => {
                    self.pending_prompt = Some(PromptState::SaveConflict {
                        focus: next_conflict_choice(&focus),
                        resume_action,
                    });
                }
                EditorCommand::Enter => match focus {
                    ConflictChoice::Reload => {
                        self.document.reload_from_disk()?;
                        self.cursor.char_index = 0;
                        self.cursor.preferred_column = 0;
                        self.pending_prompt = None;
                        self.mode = if matches!(resume_action, Some(PromptAction::Quit)) {
                            SessionMode::Exiting
                        } else {
                            SessionMode::Editing
                        };
                        self.status = Some(StatusMessage::warning("Reloaded from disk"));
                    }
                    ConflictChoice::Overwrite => match self.document.overwrite_save()? {
                        SaveResult::Saved => {
                            self.pending_prompt = None;
                            self.mode = if matches!(resume_action, Some(PromptAction::Quit)) {
                                SessionMode::Exiting
                            } else {
                                SessionMode::Editing
                            };
                            self.status = Some(StatusMessage::success("Overwrite save complete"));
                        }
                        SaveResult::BlockedReadOnly => {
                            self.status =
                                Some(StatusMessage::error("Cannot overwrite in read-only mode"));
                        }
                        SaveResult::ConflictDetected => {
                            self.status = Some(StatusMessage::warning("Conflict still present"));
                        }
                    },
                    ConflictChoice::Cancel => self.dismiss_prompt(),
                },
                EditorCommand::Cancel => self.dismiss_prompt(),
                _ => {}
            },
        }

        self.sync_viewport();
        Ok(())
    }

    fn insert_text(&mut self, text: &str) {
        if self.document.is_read_only() {
            self.status = Some(StatusMessage::warning("Read-only: edits are blocked"));
            return;
        }

        let index = self.cursor.char_index;
        let next_index = buffer::insert_text(&mut self.document.text, index, text);
        self.cursor.char_index = next_index;
        self.cursor.preferred_column = cursor::visual_column(&self.document.text, next_index);
        self.document.mark_dirty();
        let step = crate::editor::history::EditStep::Insert {
            index,
            text: text.to_string(),
        };
        let outcome = self.history.record(step);
        self.status = Some(history_status(outcome, "Inserted text"));
    }

    fn backspace(&mut self) {
        if self.document.is_read_only() {
            self.status = Some(StatusMessage::warning("Read-only: edits are blocked"));
            return;
        }

        let char_index = self.cursor.char_index;
        // Capture the char that will be removed *before* the mutation.
        let removed = if char_index > 0 {
            Some(self.document.text.char(char_index - 1).to_string())
        } else {
            None
        };

        if let Some(next_index) =
            buffer::remove_char_before(&mut self.document.text, char_index)
        {
            self.cursor.char_index = next_index;
            self.cursor.preferred_column = cursor::visual_column(&self.document.text, next_index);
            self.document.mark_dirty();
            let step = crate::editor::history::EditStep::Delete {
                index: next_index,
                text: removed.expect("removed char exists when remove_char_before succeeds"),
            };
            let outcome = self.history.record(step);
            self.status = Some(history_status(outcome, "Deleted text"));
        }
    }

    fn delete(&mut self) {
        if self.document.is_read_only() {
            self.status = Some(StatusMessage::warning("Read-only: edits are blocked"));
            return;
        }

        let char_index = self.cursor.char_index;
        // Capture the char at the cursor *before* the mutation.
        let removed = if char_index < self.document.text.len_chars() {
            Some(self.document.text.char(char_index).to_string())
        } else {
            None
        };

        if buffer::delete_char_at(&mut self.document.text, char_index) {
            self.document.mark_dirty();
            let step = crate::editor::history::EditStep::Delete {
                index: char_index,
                text: removed.expect("removed char exists when delete_char_at succeeds"),
            };
            let outcome = self.history.record(step);
            self.status = Some(history_status(outcome, "Deleted text"));
        }
    }

    fn begin_search(&mut self) {
        self.mode = SessionMode::SearchInput;
        self.search.get_or_insert_with(SearchState::default);
        self.status = Some(StatusMessage::info("Search started"));
    }

    fn request_quit(&mut self) {
        if self.document.dirty {
            self.mode = SessionMode::ConfirmQuit;
            self.pending_prompt = Some(PromptState::UnsavedChanges {
                action: PromptAction::Quit,
                focus: UnsavedChoice::Save,
            });
            self.status = Some(StatusMessage::warning("Unsaved changes"));
        } else {
            self.mode = SessionMode::Exiting;
        }
    }

    fn save_document(
        &mut self,
        resume_action: Option<PromptAction>,
    ) -> Result<SaveDisposition, DocumentError> {
        match self.document.save() {
            Ok(SaveResult::Saved) => {
                self.pending_prompt = None;
                self.mode = SessionMode::Editing;
                self.status = Some(StatusMessage::success("Saved"));
                Ok(SaveDisposition::Saved)
            }
            Ok(SaveResult::BlockedReadOnly) => {
                self.pending_prompt = None;
                self.mode = SessionMode::Editing;
                self.status = Some(StatusMessage::error("Read-only: save blocked"));
                Ok(SaveDisposition::ReadOnlyBlocked)
            }
            Ok(SaveResult::ConflictDetected) => {
                self.pending_prompt = Some(PromptState::SaveConflict {
                    focus: ConflictChoice::Cancel,
                    resume_action,
                });
                self.mode = SessionMode::SaveConflictPrompt;
                self.status = Some(StatusMessage::warning("File changed on disk"));
                Ok(SaveDisposition::ConflictPrompted)
            }
            Err(error) => {
                self.pending_prompt = None;
                self.mode = SessionMode::Editing;
                self.status = Some(StatusMessage::error(format!("Save failed: {error}")));
                if resume_action.is_some() {
                    Ok(SaveDisposition::ReadOnlyBlocked)
                } else {
                    Err(error)
                }
            }
        }
    }

    fn dismiss_prompt(&mut self) {
        self.pending_prompt = None;
        self.mode = SessionMode::Editing;
        self.status = Some(StatusMessage::info("Prompt cancelled"));
    }

    /// Undo the last recorded edit. Restores the rope's pre-edit state, moves the
    /// cursor to the step's `before_cursor`, marks the document dirty, and shows
    /// an "Undo" status. No-op (rope/cursor/dirty/status untouched) when the undo
    /// stack is empty.
    fn undo(&mut self) {
        if let Some(idx) = self.history.undo(&mut self.document.text) {
            self.cursor.char_index = idx;
            self.cursor.preferred_column = cursor::visual_column(&self.document.text, idx);
            self.document.mark_dirty();
            self.status = Some(StatusMessage::info("Undo"));
        }
        self.sync_viewport();
    }

    /// Redo the last undone edit. Re-applies the step's forward diff, moves the
    /// cursor to the step's `after_cursor`, marks the document dirty, and shows a
    /// "Redo" status. No-op when the redo stack is empty.
    fn redo(&mut self) {
        if let Some(idx) = self.history.redo(&mut self.document.text) {
            self.cursor.char_index = idx;
            self.cursor.preferred_column = cursor::visual_column(&self.document.text, idx);
            self.document.mark_dirty();
            self.status = Some(StatusMessage::info("Redo"));
        }
        self.sync_viewport();
    }

    fn sync_viewport(&mut self) {
        self.viewport
            .update_for_terminal(self.terminal_size, self.prompt_lines());
        cursor::clamp_cursor(&mut self.cursor, &self.document.text);
        cursor::ensure_cursor_in_view(&self.document.text, &self.cursor, &mut self.viewport);
    }

    pub fn prompt_lines(&self) -> u16 {
        // The single footer row is always present; the search prompt adds one more
        // row only while in SearchInput mode. There is no separate status line.
        match self.mode == SessionMode::SearchInput {
            true => 2,  // search-prompt + footer
            false => 1, // footer only
        }
    }
}

fn next_unsaved_choice(choice: &UnsavedChoice) -> UnsavedChoice {
    match choice {
        UnsavedChoice::Save => UnsavedChoice::Discard,
        UnsavedChoice::Discard => UnsavedChoice::Cancel,
        UnsavedChoice::Cancel => UnsavedChoice::Save,
    }
}

fn previous_unsaved_choice(choice: &UnsavedChoice) -> UnsavedChoice {
    match choice {
        UnsavedChoice::Save => UnsavedChoice::Cancel,
        UnsavedChoice::Discard => UnsavedChoice::Save,
        UnsavedChoice::Cancel => UnsavedChoice::Discard,
    }
}

fn next_conflict_choice(choice: &ConflictChoice) -> ConflictChoice {
    match choice {
        ConflictChoice::Reload => ConflictChoice::Overwrite,
        ConflictChoice::Overwrite => ConflictChoice::Cancel,
        ConflictChoice::Cancel => ConflictChoice::Reload,
    }
}

fn previous_conflict_choice(choice: &ConflictChoice) -> ConflictChoice {
    match choice {
        ConflictChoice::Reload => ConflictChoice::Cancel,
        ConflictChoice::Overwrite => ConflictChoice::Reload,
        ConflictChoice::Cancel => ConflictChoice::Overwrite,
    }
}

/// Pick the status message after a recording seam: warn on memory-pressure
/// eviction, otherwise show the usual info `default` message.
fn history_status(
    outcome: crate::editor::history::RecordOutcome,
    default: &str,
) -> StatusMessage {
    if outcome.oldest_dropped {
        StatusMessage::warning("History truncated to free memory")
    } else {
        StatusMessage::info(default)
    }
}
