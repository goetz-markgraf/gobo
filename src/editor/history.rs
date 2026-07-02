//! Undo/Redo history for the editor.
//!
//! Owns two in-memory stacks of [`EditStep`] values: `undo` (grows on every text
//! edit) and `redo` (grows on `undo`, cleared on the next edit). The session
//! owns a single [`History`] via [`crate::app::EditingSession`]; the stacks live
//! only for the session and are never persisted to disk (FR-008).
//!
//! # Invariants
//!
//! - **Clear-redo-on-push**: [`History::record`] always empties `redo`. This is
//!   the FR-007 guarantee that a new edit makes any previously-undone steps
//!   unreachable via Redo.
//! - **Reverse-diff symmetry**: `undo` applies a step's *reverse* diff and
//!   pushes it onto `redo`; `redo` applies the step's *forward* diff and pushes
//!   it back onto `undo`. So `undo` immediately followed by `redo` (or vice
//!   versa) is a no-op on rope content and restores the cursor exactly.
//! - **Session-bound lifetime**: a freshly-constructed [`History`] has empty
//!   stacks, and the only way to obtain one is [`History::new`] /
//!   [`History::with_capacity`]. Dropping the owning session drops the history.
//!
//! All rope operations use [`ropey::Rope::insert`] / [`ropey::Rope::remove`] in
//! character indices, exactly like [`crate::editor::buffer`].

use ropey::Rope;

/// A single reversible text change, storing the *forward* diff (the effect of
/// the original edit) plus the cursor context derived from it.
///
/// `text` is always non-empty; a no-op mutation records no step at all.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditStep {
    /// `text` was inserted starting at char index `index`. The cursor after the
    /// edit sat at `index + text.chars().count()`.
    Insert { index: usize, text: String },
    /// The char range `[index, index + text.chars().count())` was deleted.
    /// `text` holds the removed content. The cursor after the edit sat at `index`.
    Delete { index: usize, text: String },
}

impl EditStep {
    /// Number of characters in the step's text.
    pub fn len_chars(&self) -> usize {
        match self {
            EditStep::Insert { text, .. } | EditStep::Delete { text, .. } => text.chars().count(),
        }
    }

    /// End of the affected char range: for `Insert`, the post-insert cursor; for
    /// `Delete`, the (exclusive) end of the removed range.
    pub fn end_index(&self) -> usize {
        self.index() + self.len_chars()
    }

    /// Cursor index *before* the original edit.
    /// For `Insert` the cursor was at the insertion point (`index`);
    /// for `Delete` the cursor sat at the end of the removed range, then moved to `index`.
    pub fn before_cursor(&self) -> usize {
        match self {
            EditStep::Insert { index, .. } => *index,
            EditStep::Delete { .. } => self.end_index(),
        }
    }

    /// Cursor index *after* the original edit.
    /// For `Insert`: `index + len_chars`; for `Delete`: `index`.
    pub fn after_cursor(&self) -> usize {
        match self {
            EditStep::Insert { .. } => self.end_index(),
            EditStep::Delete { index, .. } => *index,
        }
    }

    fn index(&self) -> usize {
        match self {
            EditStep::Insert { index, .. } | EditStep::Delete { index, .. } => *index,
        }
    }

    /// Apply the *forward* diff (the original edit) to `text`. Used by `Redo`.
    fn apply_forward(&self, text: &mut Rope) {
        let index = self.index();
        match self {
            EditStep::Insert { text: s, .. } => text.insert(index, s),
            EditStep::Delete { .. } => text.remove(index..self.end_index()),
        }
    }

    /// Apply the *reverse* diff (undoing the original edit) to `text`. Used by `Undo`.
    fn apply_reverse(&self, text: &mut Rope) {
        let index = self.index();
        match self {
            EditStep::Insert { .. } => text.remove(index..self.end_index()),
            EditStep::Delete { text: s, .. } => text.insert(index, s),
        }
    }
}

/// Outcome of [`History::record`], surfacing the memory-pressure path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecordOutcome {
    /// `true` when the oldest undo step was evicted to make room.
    pub oldest_dropped: bool,
}

/// Compound undo/redo state owned by [`crate::app::EditingSession`].
#[derive(Debug)]
pub struct History {
    /// Top = last element; grows on edit, shrinks on `undo`.
    pub undo: Vec<EditStep>,
    /// Top = last element; grows on `undo`, shrinks on `redo`, cleared on `record`.
    pub redo: Vec<EditStep>,
    /// Max undo steps retained. `usize::MAX` in production (unbounded);
    /// reduced in tests to exercise eviction. NOT a product cap (FR-004/SC-006).
    undo_capacity: usize,
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

impl History {
    /// New unbounded history with empty stacks (`undo_capacity == usize::MAX`).
    pub fn new() -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            undo_capacity: usize::MAX,
        }
    }

    /// New history capped at `cap` undo steps. Test injection point for the
    /// memory-pressure path; not a product cap.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            undo_capacity: cap,
        }
    }

    /// Record a forward `step`: push onto `undo`, clear `redo`. If `undo` exceeds
    /// `undo_capacity`, evict the **oldest** step and report `oldest_dropped`.
    /// Does NOT mutate the rope (the edit was already applied by the caller).
    pub fn record(&mut self, step: EditStep) -> RecordOutcome {
        self.redo.clear();

        let mut oldest_dropped = false;
        if self.undo.len() >= self.undo_capacity {
            // Evict the oldest step to make room. Always keep the new step.
            self.undo.remove(0);
            oldest_dropped = true;
        }
        self.undo.push(step);

        RecordOutcome { oldest_dropped }
    }

    /// Pop the top undo step, apply its *reverse* diff to `text`, push it onto
    /// `redo`, and return `Some(before_cursor)`. `None` (mutating nothing) when
    /// the undo stack is empty.
    pub fn undo(&mut self, text: &mut Rope) -> Option<usize> {
        let step = self.undo.pop()?;
        step.apply_reverse(text);
        let cursor = step.before_cursor();
        self.redo.push(step);
        Some(cursor)
    }

    /// Pop the top redo step, apply its *forward* diff to `text`, push it onto
    /// `undo`, and return `Some(after_cursor)`. `None` (mutating nothing) when
    /// the redo stack is empty.
    pub fn redo(&mut self, text: &mut Rope) -> Option<usize> {
        let step = self.redo.pop()?;
        step.apply_forward(text);
        let cursor = step.after_cursor();
        self.undo.push(step);
        Some(cursor)
    }

    /// `true` if at least one undo step is available.
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// `true` if at least one redo step is available.
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Empty both stacks.
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }

    /// Current undo capacity (test/inspection helper; see `with_capacity`).
    pub fn undo_capacity(&self) -> usize {
        self.undo_capacity
    }
}
