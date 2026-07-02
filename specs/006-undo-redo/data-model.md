# Data Model: Undo / Redo

**Branch**: `006-undo-redo` | **Date**: 2026-07-02 | **Spec**: [spec.md](./spec.md)

Describes the new and changed data structures for the Undo/Redo feature. All positions are **character indices** (`usize`), consistent with the rest of `gobo` (see `architecture.md` → Text Model Details).

---

## New module: `src/editor/history.rs`

### `EditStep` (enum)

A single reversible text change. Stores the *forward* diff (the effect of the original edit) plus the resulting cursor index.

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditStep {
    /// Text `text` was inserted starting at char index `index`.
    /// Cursor after the edit sat at `index + text.chars().count()`.
    Insert { index: usize, text: String },
    /// The char range `[index, index + text.chars().count())` was deleted.
    /// `text` holds the deleted content. Cursor after the edit sat at `index`.
    Delete { index: usize, text: String },
}
```

**Derived helpers (pure functions on the step)**:
- `EditStep::len_chars(&self) -> usize` → `self.text.chars().count()`.
- `EditStep::end_index(&self) -> usize` → for `Insert`, `index + len_chars`; for `Delete`, `index + len_chars` (the end of the removed range).
- `EditStep::before_cursor(&self) -> usize` → the cursor index *before* the original edit: `Insert → index`, `Delete → index + len_chars` (the cursor was positioned at the end of what got deleted, then moved to `index`).
- `EditStep::after_cursor(&self) -> usize` → `Insert → index + len_chars`, `Delete → index`.

**Invariant**: `text` is always non-empty (a no-op edit, e.g. `Backspace` at index 0 or `Delete` past the end, records **no** step). This is enforced at the `app.rs` recording seams.

**Validation rules**:
- `index` must satisfy `0 <= index <= rope.len_chars()` at the time the step is applied.
- For `Delete`, `index + len_chars <= rope.len_chars()` at the time of *reverse* application (i.e. the stored text must still match the rope content at that range — guaranteed because Undo/Redo are strict inverses that never skip steps).

### `RecordOutcome` (struct)

Return value of `History::record`, surfacing the memory-pressure path to the caller.

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecordOutcome {
    /// True when the oldest undo step was evicted to make room.
    pub oldest_dropped: bool,
}
```

### `History` (struct)

The compound state, owned by `EditingSession`.

```rust
#[derive(Debug)]
pub struct History {
    pub undo: Vec<EditStep>, // top = last element; grows on edit
    pub redo: Vec<EditStep>, // top = last element; grows on undo, cleared on edit
    /// Max undo steps retained. `usize::MAX` in production (unbounded);
    /// reduced in tests to exercise the OOM/eviction path. NOT a product cap.
    undo_capacity: usize,
}
```

**State transitions**:

| Event | undo stack | redo stack | Effect on `Rope` | Cursor destination |
|-------|-----------|-----------|------------------|--------------------|
| New text edit | `undo.push(forward_step)`; if `undo.len() == undo_capacity` then `undo.remove(0)` and `oldest_dropped=true` | `redo.clear()` | edit already applied by `app.rs` before `record` | set by `app.rs` after the edit (unchanged from current behavior) |
| `Undo` (undo non-empty) | pop top step `s` | `redo.push(s)` | apply `s`'s reverse diff | `s.before_cursor()` |
| `Undo` (undo empty) | unchanged | unchanged | none | unchanged |
| `Redo` (redo non-empty) | `undo.push(s)` | pop top step `s` | apply `s`'s forward diff | `s.after_cursor()` |
| `Redo` (redo empty) | unchanged | unchanged | none | unchanged |
| Session reopen (new `EditingSession`) | empty | empty | fresh document text | index 0 |

**Public API**:

```rust
impl History {
    pub fn new() -> Self;                       // undo_capacity = usize::MAX
    pub fn with_capacity(usize) -> Self;        // test injection point
    pub fn record(&mut self, step: EditStep) -> RecordOutcome;
    pub fn undo(&mut self, text: &mut Rope) -> Option<usize>; // returns restored cursor index
    pub fn redo(&mut self, text: &mut Rope) -> Option<usize>; // returns restored cursor index
    pub fn can_undo(&self) -> bool;
    pub fn can_redo(&self) -> bool;
    pub fn clear(&mut self);                    // empties both stacks
}
```

**Reverse/forward application (ropey)**:
- `Undo` of `Insert { index, text }` → `text.remove(index..index + len_chars)`.
- `Undo` of `Delete { index, text }` → `text.insert(index, &text)`.
- `Redo` of `Insert { index, text }` → `text.insert(index, &text)`.
- `Redo` of `Delete { index, text }` → `text.remove(index..index + len_chars)`.

These use the existing `ropey::Rope::insert` / `Rope::remove`, operating in character indices exactly like `editor::buffer`.

---

## Changed: `src/app.rs` → `EditingSession`

### New field

```rust
pub struct EditingSession {
    // ... existing fields unchanged ...
    pub history: History,   // NEW
}
```

Initialized in `EditingSession::new()` as `History::new()` (empty stacks). Because `new()` is called on every `open()`, session-lifetime binding (FR-008) is automatic: a freshly opened document starts with empty stacks.

### Recording seams (existing mutation helpers)

| Helper | Recorded step (on success only) |
|--------|---------------------------------|
| `insert_text(&str)` | `EditStep::Insert { index: <char_index before insert>, text: <the inserted string> }` |
| `backspace()` | `EditStep::Delete { index: <next_index>, text: <the removed char as String> }` (only when a char was actually removed) |
| `delete()` | `EditStep::Delete { index: <char_index>, text: <the removed char as String> }` (only when a char was actually removed) |

Each helper already returns/has the indices it needs; `backspace` returns `Option<usize>` and `delete` returns `bool` (`buffer` module). To capture the *removed char text* for the diff, `app.rs` reads `self.document.text` at the relevant range **before** calling the `buffer` removal function. `record` is called **after** the `Rope` mutation succeeds, with `self.history.record(step)`; if `RecordOutcome.oldest_dropped` is true, `app.rs` sets a `StatusMessage::warning("History truncated to free memory")` instead of the usual "Inserted/Deleted text" info.

`Enter` (newline) routes through `insert_text("\n")` and therefore naturally records one `Insert` step (FR-011 → one step per Enter).

### New command dispatch (editing mode only)

In `handle_editing_command`:

```rust
EditorCommand::Undo => self.undo(),
EditorCommand::Redo => self.redo(),
```

with helper methods:

```rust
fn undo(&mut self) {
    if let Some(idx) = self.history.undo(&mut self.document.text) {
        self.cursor.char_index = idx;
        self.cursor.preferred_column = cursor::visual_column(&self.document.text, idx);
        self.document.mark_dirty();
        self.status = Some(StatusMessage::info("Undo"));
    }
    self.sync_viewport();
}

fn redo(&mut self) {
    if let Some(idx) = self.history.redo(&mut self.document.text) {
        self.cursor.char_index = idx;
        self.cursor.preferred_column = cursor::visual_column(&self.document.text, idx);
        self.document.mark_dirty();
        self.status = Some(StatusMessage::info("Redo"));
    }
    self.sync_viewport();
}
```

Note: an Undo/Redo that actually changes the buffer marks the document dirty (Undo of an unsaved edit must still allow the quit-guard to fire). An empty-stack Undo/Redo is a no-op and does not touch dirty/status.

### Mode gating

`Undo`/`Redo` appear **only** in `handle_editing_command`. The existing precedence in `handle_command` (prompts checked first, then mode dispatch) means they are silently ignored in `SearchInput`, `ConfirmQuit`, `SaveConflictPrompt`, and whenever a prompt is pending (FR-009). No changes to `handle_search_command` / `handle_prompt_command` are needed.

### Save interaction (FR-013)

`save_document` is unchanged with respect to history: saving never calls `record` and never calls `clear`. The history survives a save. No new field or logic is required.

---

## Changed: `src/editor/input.rs`

`EditorCommand` gains two variants:

```rust
pub enum EditorCommand {
    // ... existing ...
    Undo,
    Redo,
}
```

`map_key_event` gains two arms (placed **before** the bare `Char(c)` catch-all and alongside the other `Ctrl-*` bindings):

```rust
(KeyModifiers::CONTROL, KeyCode::Char('z')) => Some(EditorCommand::Undo),
(KeyModifiers::CONTROL, KeyCode::Char('y')) => Some(EditorCommand::Redo),
```

The existing `!key.modifiers.contains(KeyModifiers::CONTROL)` guard on the printable-char arm ensures `Ctrl-Z`/`Ctrl-Y` are not also interpreted as `InsertChar('z')` / `InsertChar('y')`.

---

## Changed: `src/editor/status.rs`

No new variants needed; `StatusMessage::info` / `StatusMessage::warning` already cover "Undo", "Redo", and the OOM "History truncated to free memory" message. The status module is unchanged in source; only new call-sites use existing constructors.

---

## Entity summary (mirrors spec → Key Entities)

| Spec entity | Code type | Lifetime |
|-------------|-----------|----------|
| Undo-Eintrag | `EditStep` | lives on `History::undo` |
| Redo-Eintrag | `EditStep` | lives on `History::redo`, created only by Undo, destroyed by next edit or by Redo |
| Undo-Verlauf | `History::undo: Vec<EditStep>` | session (`History` owned by `EditingSession`, dropped on session drop) |
| Redo-Verlauf | `History::redo: Vec<EditStep>` | session; cleared on every new edit |

All four are in-memory only; nothing is written to disk or to a sidecar file (FR-008).
