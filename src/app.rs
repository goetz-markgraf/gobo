//! High-level editing session state and command handling.

use crate::document::{DocumentBuffer, DocumentError, SaveResult};
use crate::editor::buffer;
use crate::editor::clipboard;
use crate::editor::cursor::{self, CursorState, Selection};
use crate::editor::history::EditStep;
use crate::editor::indent;
use crate::editor::input::EditorCommand;
use crate::editor::render::{self, RenderView, TerminalSize, ViewportState};
use crate::editor::search::SearchState;
use crate::editor::status::{StatusMessage, help_max_offset};
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
    /// Static help dialog showing all active Ctrl-key shortcuts.
    HelpDialog,
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
    /// Active text selection, or `None` when no text is selected. Session-bound,
    /// in-memory, never persisted. Seeded by `MoveSelect*`, cleared by plain
    /// moves / edits / undo-redo (FR-001/FR-013).
    pub selection: Option<Selection>,
    /// Vertical scroll offset for HelpDialog popup when open.
    pub help_scroll_offset: usize,
    /// When a help dialog is open on top of another pending prompt, preserves
    /// the underlying session state so it can be restored on close.
    pub help_preserved_state: Option<(SessionMode, Option<PromptState>)>,
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
            selection: None,
            help_scroll_offset: 0,
            help_preserved_state: None,
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
            // Clamp help_scroll_offset so resize doesn't create a dead scroll zone.
            let max_off = help_max_offset(self.terminal_size);
            if matches!(self.pending_prompt, Some(PromptState::HelpDialog)) {
                self.help_scroll_offset = self.help_scroll_offset.min(max_off);
            }
            self.status = Some(StatusMessage::info(format!(
                "Resized to {}x{}",
                size.width, size.height
            )));
            return Ok(());
        }

        if self.mode == SessionMode::Exiting {
            return Ok(());
        }

        // Help dialog is a global overlay — always opens regardless of current mode.
        if let EditorCommand::ShowHelp = command {
            let preserved = (self.mode.clone(), self.pending_prompt.take());
            self.help_preserved_state = Some(preserved);
            self.pending_prompt = Some(PromptState::HelpDialog);
            // Clamp scroll offset when first opening to avoid stale values.
            let max_off = help_max_offset(self.terminal_size);
            self.help_scroll_offset = self.help_scroll_offset.min(max_off);
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
            EditorCommand::InsertChar(c) => self.replace_or_insert(&c.to_string()),
            EditorCommand::Enter => self.apply_indent_command(EditorCommand::Enter),
            EditorCommand::Tab => self.apply_indent_command(EditorCommand::Tab),
            EditorCommand::Backspace => self.apply_indent_command(EditorCommand::Backspace),
            EditorCommand::Delete => self.delete_or_backspace(true),
            EditorCommand::MoveSelectLeft => {
                cursor::move_select_left(&mut self.selection, &mut self.cursor, &self.document.text)
            }
            EditorCommand::MoveSelectRight => cursor::move_select_right(
                &mut self.selection,
                &mut self.cursor,
                &self.document.text,
            ),
            EditorCommand::MoveSelectUp => {
                cursor::move_select_up(&mut self.selection, &mut self.cursor, &self.document.text)
            }
            EditorCommand::MoveSelectDown => cursor::move_select_down(
                &mut self.selection,
                &mut self.cursor,
                &self.document.text,
            ),
            EditorCommand::MoveLeft => {
                // FR-004: a plain move collapses any active selection before motion.
                self.selection = None;
                cursor::move_left(&mut self.cursor, &self.document.text);
            }
            EditorCommand::MoveRight => {
                self.selection = None;
                cursor::move_right(&mut self.cursor, &self.document.text);
            }
            EditorCommand::MoveUp => {
                self.selection = None;
                cursor::move_up(&mut self.cursor, &self.document.text);
            }
            EditorCommand::MoveDown => {
                self.selection = None;
                cursor::move_down(&mut self.cursor, &self.document.text);
            }
            EditorCommand::Save => {
                let _ = self.save_document(None)?;
            }
            EditorCommand::Quit => self.request_quit(),
            EditorCommand::Search => self.begin_search(),
            EditorCommand::Cancel | EditorCommand::FindNext => {}
            EditorCommand::Undo => self.undo(),
            EditorCommand::Redo => self.redo(),
            EditorCommand::Copy => self.copy(),
            EditorCommand::Cut => self.cut(),
            EditorCommand::Paste => self.paste(),
            EditorCommand::ShowHelp => {
                // Preserve current mode + pending prompt, then open the help dialog.
                let preserved = (self.mode.clone(), self.pending_prompt.take());
                self.help_preserved_state = Some(preserved);
                // Help content will be generated by status::popup_view() when needed.
            }
            // FR-011: Search/FindNext/Save/Quit and non-editing commands preserve the
            // selection; Undo/Redo clear it (see `undo`/`redo`).
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
            EditorCommand::Copy => self.copy(),
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
            | EditorCommand::MoveSelectLeft
            | EditorCommand::MoveSelectRight
            | EditorCommand::MoveSelectUp
            | EditorCommand::MoveSelectDown
            | EditorCommand::Delete
            | EditorCommand::Save
            | EditorCommand::Search
            | EditorCommand::Undo
            | EditorCommand::Redo
            | EditorCommand::Cut
            | EditorCommand::Paste
            | EditorCommand::ShowHelp
            | EditorCommand::Tab
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
                EditorCommand::MoveRight | EditorCommand::Tab | EditorCommand::NextChoice => {
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
                EditorCommand::MoveRight | EditorCommand::Tab | EditorCommand::NextChoice => {
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
                        // FR-011: reload resets the cursor, so the selection is
                        // cleared as part of the cursor reset (no split clusters).
                        self.selection = None;
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
            PromptState::HelpDialog => {
                // Help dialog: up/down scroll, enter/escape close.
                match command {
                    EditorCommand::MoveDown => {
                        // Clamp to max_offset so the offset can never drift past
                        // the last scrollable row — otherwise pressing Up again
                        // after overshoot shows no visible change (dead zone).
                        let max_off = help_max_offset(self.terminal_size);
                        self.help_scroll_offset =
                            (self.help_scroll_offset + 1).min(max_off);
                    }
                    EditorCommand::MoveUp => {
                        self.help_scroll_offset = self.help_scroll_offset.saturating_sub(1);
                    }
                    EditorCommand::Enter | EditorCommand::Cancel => {
                        if let Some((mode, prev_prompt)) = self.help_preserved_state.take() {
                            self.pending_prompt = None;
                            self.mode = mode;
                            self.pending_prompt = prev_prompt;
                        } else {
                            self.dismiss_prompt();
                        }
                    }
                    _ => {
                        // All other keys silently ignored while help is open.
                    }
                }
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

    /// Selection-aware insert path (FR-005/FR-007/FR-008). When a non-empty
    /// selection exists, perform one atomic `EditStep::Replace` (removed = the
    /// selected text, inserted = `text`), clear the selection, land the cursor
    /// after the inserted text. Otherwise fall back to the existing `insert_text`
    /// seam (FR-008). Read-only documents are blocked (constitution III).
    fn replace_or_insert(&mut self, text: &str) {
        if let Some(selection) = self.selection
            && !selection.is_empty()
        {
            self.replace_selection(text);
            return;
        }
        self.insert_text(text);
    }

    fn apply_indent_command(&mut self, command: EditorCommand) {
        let selection_range = self
            .selection
            .filter(|selection| !selection.is_empty())
            .map(|selection| selection.range());

        let Some(plan) = self.indent_action_plan(&command) else {
            match command {
                EditorCommand::Backspace => {
                    if let Some(range) = selection_range {
                        let start = range.start;
                        let delete_start = start.saturating_sub(1);
                        self.apply_atomic_edit(delete_start, range.end, "");
                    } else {
                        self.backspace();
                    }
                }
                EditorCommand::Enter => self.replace_or_insert("\n"),
                EditorCommand::Tab => {}
                _ => {}
            }
            return;
        };

        self.apply_atomic_edit(plan.replace_start, plan.replace_end, &plan.inserted_text);
    }

    fn indent_action_plan(&self, command: &EditorCommand) -> Option<indent::IndentActionPlan> {
        let (replace_start, replace_end) = if let Some(selection) = self.selection
            && !selection.is_empty()
        {
            let range = selection.range();
            (range.start, range.end)
        } else {
            (self.cursor.char_index, self.cursor.char_index)
        };

        match command {
            EditorCommand::Tab => Some(indent::plan_tab(
                &self.document.text,
                replace_start,
                replace_end,
            )),
            EditorCommand::Enter => Some(indent::plan_enter(
                &self.document.text,
                replace_start,
                replace_end,
            )),
            EditorCommand::Backspace => indent::plan_backspace(
                &self.document.text,
                replace_start,
                replace_end,
            ),
            _ => None,
        }
    }

    /// Selection-aware delete path for `Backspace` (`forward = false`) and
    /// `Delete` (`forward = true`) (FR-006/FR-007/FR-008). When a non-empty
    /// selection exists, both route to the same atomic delete-selection, landing
    /// the cursor at the selection start. Otherwise fall back to the existing
    /// single-char `backspace` / `delete` seams (FR-008).
    fn delete_or_backspace(&mut self, _forward: bool) {
        if let Some(selection) = self.selection
            && !selection.is_empty()
        {
            self.replace_selection("");
            return;
        }
        if _forward {
            self.delete();
        } else {
            self.backspace();
        }
    }

    fn apply_atomic_edit(&mut self, start: usize, end: usize, inserted: &str) {
        if self.document.is_read_only() {
            self.status = Some(StatusMessage::warning("Read-only: edits are blocked"));
            return;
        }

        let clamped_start = start.min(self.document.text.len_chars());
        let clamped_end = end.min(self.document.text.len_chars()).max(clamped_start);
        let removed = self.document.text.slice(clamped_start..clamped_end).to_string();
        let next_index = buffer::replace_range(
            &mut self.document.text,
            clamped_start..clamped_end,
            inserted,
        );
        self.cursor.char_index = next_index;
        self.cursor.preferred_column = cursor::visual_column(&self.document.text, next_index);
        self.document.mark_dirty();

        let step = if removed.is_empty() {
            EditStep::Insert {
                index: clamped_start,
                text: inserted.to_string(),
            }
        } else {
            EditStep::Replace {
                index: clamped_start,
                removed,
                inserted: inserted.to_string(),
            }
        };
        let outcome = self.history.record(step);
        self.status = Some(history_status(outcome, "Replaced text"));
        self.selection = None;
    }

    /// Atomic replace of the active selection range with `inserted` (may be
    /// empty for a pure delete). Records exactly one `EditStep::Replace`, clears
    /// the selection, lands the cursor at `start + inserted.chars().count()`,
    /// and marks the document dirty. Waits behind a read-only guard.
    fn replace_selection(&mut self, inserted: &str) {
        if self.document.is_read_only() {
            self.status = Some(StatusMessage::warning("Read-only: edits are blocked"));
            return;
        }

        let selection = self
            .selection
            .expect("replace_selection called only with a selection");
        let range = selection.range();
        let start = range.start;
        let end = range.end.min(self.document.text.len_chars());
        // Capture the to-be-removed text before mutation.
        let removed: String = self.document.text.slice(start..end).to_string();

        let next_index = buffer::replace_range(&mut self.document.text, range, inserted);
        self.cursor.char_index = next_index;
        self.cursor.preferred_column = cursor::visual_column(&self.document.text, next_index);
        self.document.mark_dirty();
        // `removed` is non-empty by the `is_empty()` guard, so a `Replace` step is
        // always recorded here (non-degeneracy, FR-008). `record` clears `redo`.
        let step = crate::editor::history::EditStep::Replace {
            index: start,
            removed,
            inserted: inserted.to_string(),
        };
        let outcome = self.history.record(step);
        self.status = Some(history_status(outcome, "Replaced text"));
        self.selection = None;
    }

    // ---- Clipboard: Copy (spec 009, FR-001) -----------------------------------
    /// Ctrl-C: write selection or single grapheme to OS clipboard. Buffer,
    /// cursor, and selection are **not** modified (FR-001). Safe in read-only.
    fn copy(&mut self) {
        let Some((text, _)) = self.clipboard_source() else {
            self.status = Some(StatusMessage::info("Nothing to copy"));
            return;
        };
        let n = text.chars().count();
        match clipboard::write_text(&text) {
            Ok(()) => self.status = Some(StatusMessage::info(format!("Copied {n} chars"))),
            Err(msg) => self.status = Some(StatusMessage::warning(format!("Failed to copy: {msg}"))),
        }
        // copy never clears the selection (FR-001).
    }

    // ---- Clipboard: Cut (spec 009, FR-002/FR-003/FR-005) ----------------------
    /// Ctrl-X: write selection or single grapheme to OS clipboard, then delete
    /// it from the buffer in **one atomic undo step** (FR-005). Blocked in
    /// read-only (constitution III). Clipboard is NOT updated if the OS write
    /// fails, and the buffer is NOT mutated (fail-safe, FR-012).
    fn cut(&mut self) {
        if self.document.is_read_only() {
            self.status = Some(StatusMessage::warning("Read-only: edits are blocked"));
            return;
        }
        let Some((text, start)) = self.clipboard_source() else {
            self.status = Some(StatusMessage::info("Nothing to cut"));
            return;
        };
        let n = text.chars().count();
        // Write clipboard first; don't mutate buffer if the OS write fails.
        if let Err(msg) = clipboard::write_text(&text) {
            self.status = Some(StatusMessage::warning(msg));
            return;
        }
        if let Some(sel) = self.selection
            && !sel.is_empty()
        {
            // Cut with selection — one atomic Replace (FR-002/FR-010).
            let range = sel.range();
            let clamped_end = range.end.min(self.document.text.len_chars());
            let removed: String = self.document.text.slice(range.start..clamped_end).to_string();
            let next = buffer::replace_range(&mut self.document.text, range.clone(), "");
            self.cursor.char_index = next;
            self.cursor.preferred_column = cursor::visual_column(&self.document.text, next);
            self.document.mark_dirty();
            let step = EditStep::Replace {
                index: range.start,
                removed,
                inserted: String::new(),
            };
            let outcome = self.history.record(step);
            self.selection = None;
            self.status = Some(cut_status(outcome, n));
        } else {
            // Cut single grapheme — one atomic Delete (FR-003/FR-011).
            let end = start + n;
            let next = buffer::replace_range(&mut self.document.text, start..end, "");
            self.cursor.char_index = next; // stays at `start` (next == start)
            self.cursor.preferred_column = cursor::visual_column(&self.document.text, next);
            self.document.mark_dirty();
            let step = EditStep::Delete { index: start, text };
            let outcome = self.history.record(step);
            self.status = Some(cut_status(outcome, n));
        }
    }

    // ---- Clipboard: Paste (spec 009, FR-004/FR-005/FR-007/FR-009) -------------
    /// Ctrl-V: insert OS clipboard text at cursor, or replace active selection.
    /// Silent no-op when clipboard is empty or non-text (FR-009). Warns and
    /// aborts when clipboard content exceeds 1 MB (FR-013). One atomic undo step
    /// (FR-005). Cursor lands after the inserted text; selection cleared (FR-007).
    fn paste(&mut self) {
        let clip = match clipboard::read_text() {
            None => return, // binary / empty / unavailable — silent no-op (FR-009)
            Some(t) => t,
        };
        if clip.is_empty() {
            return; // empty string — silent no-op (FR-009)
        }
        if !clipboard::fits_size_limit(clip.len()) {
            self.status = Some(StatusMessage::warning(
                "Clipboard content too large (>1 MB)",
            ));
            return;
        }
        if self.document.is_read_only() {
            self.status = Some(StatusMessage::warning("Read-only: edits are blocked"));
            return;
        }
        let n = clip.chars().count();
        if let Some(sel) = self.selection
            && !sel.is_empty()
        {
            // Paste over selection — one atomic Replace (FR-004/FR-005).
            let range = sel.range();
            let clamped_end = range.end.min(self.document.text.len_chars());
            let removed: String = self.document.text.slice(range.start..clamped_end).to_string();
            let next = buffer::replace_range(&mut self.document.text, range.clone(), &clip);
            self.cursor.char_index = next;
            self.cursor.preferred_column = cursor::visual_column(&self.document.text, next);
            self.document.mark_dirty();
            let step = EditStep::Replace {
                index: range.start,
                removed,
                inserted: clip,
            };
            let outcome = self.history.record(step);
            self.selection = None;
            self.status = Some(paste_status(outcome, n));
        } else {
            // Paste at cursor — one atomic Insert (FR-004/FR-005).
            let index = self.cursor.char_index;
            let next = buffer::insert_text(&mut self.document.text, index, &clip);
            self.cursor.char_index = next;
            self.cursor.preferred_column = cursor::visual_column(&self.document.text, next);
            self.document.mark_dirty();
            let step = EditStep::Insert { index, text: clip };
            let outcome = self.history.record(step);
            self.selection = None; // FR-007: clear any residual empty selection
            self.status = Some(paste_status(outcome, n));
        }
    }

    /// The text (and its start char index) that a no-selection copy/cut operates
    /// on: the active selection range when non-empty, else the single grapheme
    /// cluster at/after the cursor (spec 009, FR-001/FR-002/FR-003).
    fn clipboard_source(&self) -> Option<(String, usize)> {
        if let Some(sel) = self.selection
            && !sel.is_empty()
        {
            let r = sel.range();
            let clamped_end = r.end.min(self.document.text.len_chars());
            let text = self.document.text.slice(r.start..clamped_end).to_string();
            return Some((text, r.start));
        }
        cursor::grapheme_at_cursor(&self.document.text, self.cursor.char_index)
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
        // FR-015: undo clears any active selection.
        self.selection = None;
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
        // FR-015: redo clears any active selection.
        self.selection = None;
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

/// Status for a completed cut; warns on history eviction.
fn cut_status(
    outcome: crate::editor::history::RecordOutcome,
    n_chars: usize,
) -> StatusMessage {
    if outcome.oldest_dropped {
        StatusMessage::warning("History truncated to free memory")
    } else {
        StatusMessage::info(format!("Cut {n_chars} chars"))
    }
}

/// Status for a completed paste; warns on history eviction.
fn paste_status(
    outcome: crate::editor::history::RecordOutcome,
    n_chars: usize,
) -> StatusMessage {
    if outcome.oldest_dropped {
        StatusMessage::warning("History truncated to free memory")
    } else {
        StatusMessage::info(format!("Pasted {n_chars} chars"))
    }
}
