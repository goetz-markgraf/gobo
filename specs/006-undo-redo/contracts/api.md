# Contracts: Undo / Redo

**Branch**: `006-undo-redo` | **Date**: 2026-07-02 | **Spec**: [spec.md](./spec.md)

`gobo` is a single-binary local terminal editor. It exposes no network/RPC surface. The contracts below are the **public, programmable interfaces** that this feature adds or extends, and which automated tests and any embedding harness rely on.

---

## 1. Programmatic contract — `gobo::editor::history`

Stable public API of the new `history` module (full type details in [data-model.md](../data-model.md)).

### Types

```rust
pub enum EditStep { Insert { index: usize, text: String }, Delete { index: usize, text: String } }
pub struct RecordOutcome { pub oldest_dropped: bool }
pub struct History { /* undo, redo, undo_capacity */ }
```

### Operations

| Signature | Postcondition | Failure / no-op behavior |
|-----------|---------------|--------------------------|
| `History::new() -> Self` | empty undo and redo; `undo_capacity == usize::MAX` | — |
| `History::with_capacity(cap: usize) -> Self` | empty stacks; `undo_capacity == cap` (test hook) | — |
| `History::record(&mut self, step: EditStep) -> RecordOutcome` | after: `self.undo.last() == Some(&step)`; `self.redo` is empty. If `undo.len()` reached `undo_capacity`, the **oldest** step was evicted and `RecordOutcome.oldest_dropped == true`. | A `step` with empty `text` MUST NOT be recorded (caller contract; `app.rs` never calls record for a no-op mutation). The rope is NOT mutated by `record`. |
| `History::undo(&mut self, text: &mut Rope) -> Option<usize>` | If undo non-empty: pops top `s`, applies `s`'s reverse diff to `text`, pushes `s` onto `redo`, returns `Some(s.before_cursor())`. | If undo empty: returns `None`, `text` and stacks unchanged. |
| `History::redo(&mut self, text: &mut Rope) -> Option<usize>` | If redo non-empty: pops top `s`, applies `s`'s forward diff to `text`, pushes `s` onto `undo`, returns `Some(s.after_cursor())`. | If redo empty: returns `None`, `text` and stacks unchanged. |
| `History::can_undo(&self) -> bool` | `!self.undo.is_empty()` | — |
| `History::can_redo(&self) -> bool` | `!self.redo.is_empty()` | — |
| `History::clear(&mut self)` | both stacks empty | — |

**Determinism invariant** (FR-012): for any sequence of `record` calls producing a sequence of steps, repeated `undo` until `None` followed by repeated `redo` until `None` restores the rope to byte-identical content and the cursor to the post-last-edit index.

**Reverse-diff symmetry invariant**: `undo` followed immediately by `redo` (and vice versa) is a no-op on the rope content and leaves the cursor where the paired operation left it.

---

## 2. Programmatic contract — `gobo::app::EditingSession`

The `EditingSession` is the existing primary test harness entry point (per `architecture.md` → Testing Convention). Undo/Redo extend it without changing any existing public signature.

### New field

```rust
pub struct EditingSession {
    // ... existing fields ...
    pub history: History,   // NEW; initialized empty in new()
}
```

### New command handling (editing mode only)

`handle_command(EditorCommand::Undo)` and `handle_command(EditorCommand::Redo)`:

| Pre-state | Post-state |
|-----------|------------|
| `SessionMode::Editing` and `pending_prompt.is_none()` and undo/redo non-empty | rope updated by the relevant diff; `cursor.char_index` = step's `before_cursor()` (Undo) / `after_cursor()` (Redo); `cursor.preferred_column` recomputed; `document.dirty = true`; `status = Some(StatusMessage::info("Undo"\|"Redo"))`. |
| Same but undo/redo **empty** | complete no-op: rope, cursor, dirty flag, status all unchanged. |
| Any prompt active (`pending_prompt.is_some()`) | ignored — `handle_command` routes to `handle_prompt_command`, which has no Undo/Redo arms. |
| `SessionMode::SearchInput` | ignored — `handle_search_command` has no Undo/Redo arms. |
| `SessionMode::ConfirmQuit`, `SaveConflictPrompt`, `Exiting` | ignored. |

### Recording contract (every text mutation)

| `EditorCommand` in `Editing` mode | Records a step? | Step kind |
|----|----|----|
| `InsertChar(c)` | yes (always — single char is non-empty) | `Insert { index: pre-insert cursor, text: c.to_string() }` |
| `Enter` | yes | `Insert { index: pre-insert cursor, text: "\n" }` |
| `Backspace` | only if a char was actually removed | `Delete { index: post-remove cursor, text: <removed char> }` |
| `Delete` | only if a char was actually removed | `Delete { index: pre-remove cursor, text: <removed char> }` |
| `Move*` | never | — |
| `Save` | never (FR-013) | — |
| `Undo` / `Redo` | never (they move steps between stacks, not record) | — |
| `Search`, `FindNext`, `Cancel`, `NextChoice`, `PreviousChoice`, `Resize` | never | — |

**Redo-clear invariant** (FR-007/SC-003): immediately after any successful `record`, `session.history.redo` is empty.

**Session-lifetime invariant** (FR-008/SC-004): a freshly constructed or freshly `open()`-ed `EditingSession` has `history.undo.is_empty() && history.redo.is_empty()`.

**Memory-pressure contract** (FR-006/SC-007): when `record` returns `RecordOutcome { oldest_dropped: true }`, the applying helper (`insert_text`/`backspace`/`delete`) sets `status = Some(StatusMessage::warning("History truncated to free memory"))`. The text edit that triggered the record is **always** applied to the rope regardless of the outcome.

**Save interaction** (FR-013): `save_document`/`save`/`overwrite_save` do not call `record` or `clear`; history is preserved across a save.

---

## 3. Input contract — `gobo::editor::input`

`EditorCommand` enum gains `Undo` and `Redo`. `map_key_event` mapping additions:

| Key event | Command |
|-----------|---------|
| `Ctrl-Z` (Control + Char('z')) | `EditorCommand::Undo` |
| `Ctrl-Y` (Control + Char('y')) | `EditorCommand::Redo` |

Precedence: these arms are matched **before** the bare-printable `Char(c)` catch-all, and that catch-all already excludes `KeyModifiers::CONTROL`, so `Ctrl-Z`/`Ctrl-Y` are never aliased to `InsertChar('z')`/`InsertChar('y')`.

Existing key bindings (Ctrl-S/Ctrl-Q/Ctrl-F/Ctrl-G, arrows, Enter, Esc, Tab, Backspace, Delete, printable) are unchanged.

---

## 4. CLI contract — `gobo::cli`

Unchanged. The single positional `path: PathBuf` argument and `parse_args()` helper are unaffected by this feature.
