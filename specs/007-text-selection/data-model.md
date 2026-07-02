# Data Model: Text Selection

**Branch**: `007-text-selection` | **Date**: 2026-07-02 | **Spec**: [spec.md](./spec.md)

This feature adds two data entities (`Selection`, and the `EditStep::Replace` variant) and extends one existing struct (`EditingSession`). All indices are **character indices** into the `Rope` (`usize`), identical to the existing text model (see `architecture.md` → Text Model Details). No persistence is involved; everything is session-bound and in-memory.

---

## Entity 1 — `Selection`

**Location**: `src/editor/cursor.rs` (the existing cursor-state module).

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Selection {
    /// Fixed anchor: the cursor position where the user first pressed
    /// Shift+Arrow. Never moved by `MoveSelect*`.
    pub anchor: usize,
    /// Moving head: the current cursor position. Equal to the live
    /// `EditingSession.cursor.char_index` while a selection is active.
    pub head: usize,
}
```

### Fields

| Field | Type | Meaning | Constraints |
|-------|------|---------|-------------|
| `anchor` | `usize` | Char index where the selection started | `0 <= anchor <= text.len_chars()` (clamped via motion functions) |
| `head` | `usize` | Char index of the live cursor end | `0 <= head <= text.len_chars()` |

### Derived geometry

- **Direction** (FR-002): `forward` iff `head >= anchor`, else `backward`. Not stored — derived on demand to avoid a third consistency-maintained field.
- **Acted-on range**: `[start, end)` where `start = anchor.min(head)`, `end = anchor.max(head)`. Always a half-open char range, safe for `Rope::remove`.
- **Length**: `end - start`. **Empty** iff `length == 0` (i.e. `anchor == head`); per spec edge case 3 / FR-008 an empty selection behaves as "no selection."

### State transitions

| From | Trigger | To | Notes |
|------|---------|----|-------|
| `None` (no selection) | `MoveSelect*` | `Some(Selection { anchor: pre-move cursor, head: post-move cursor })` | Anchor seeded from current cursor (R5) |
| `Some(s)` | `MoveSelect*` (same or different direction) | `Some(Selection { anchor: s.anchor /* unchanged */, head: new cursor })` | Head may cross anchor → direction flips naturally (edge case 2) |
| `Some(s)` | any plain `Move*` (no Shift) | `None` | FR-004 collapse |
| `Some(s)` | `InsertChar`/`Enter` on non-empty selection | `None` + rope mutated, one `Replace` step recorded | FR-005 |
| `Some(s)` | `Backspace`/`Delete` on non-empty selection | `None` + rope mutated, one `Replace` step (empty `inserted`) recorded | FR-006 |
| `Some(s)` | `Undo`/`Redo` | `None` | restored text has no meaningful selection |
| `Some(s)` | `Search`/`FindNext`/`Save`/`Quit`→prompt | `Some(s)` unchanged | FR-011 / edge case 5 |

### Validation rules (from requirements)

- **FR-003**: anchor/head cannot exceed document bounds — enforced by reusing the existing clamping motion functions (`move_left` saturates, `move_right`/up/down clamp).
- **FR-012**: boundaries, empty lines, newline-only ranges, CRLF, multi-grapheme clusters — handled by char-index + existing grapheme-aware column math; no partial clusters.
- **Empty selection** (FR-008): when `anchor == head`, edit commands route to the existing single-char paths unchanged.

### Relationships

- Owned by `EditingSession.selection: Option<Selection>` (new field).
- Overlays `CursorState` — `head` mirrors `cursor.char_index` while active; `cursor` remains the single source of truth for the live cursor, and `head` is kept in sync by the `MoveSelect*` handlers writing both.
- Consumed by `render.rs` (highlight projection) and `app.rs` (atomic edit/delete).

---

## Entity 2 — `EditStep::Replace` (new variant)

**Location**: `src/editor/history.rs` (extends the existing `EditStep` enum).

```rust
pub enum EditStep {
    Insert { index: usize, text: String },            // existing
    Delete { index: usize, text: String },            // existing
    /// Atomic delete-then-insert over the char range
    /// `[index, index + removed.chars().count())`. `removed`
    /// holds the deleted content; `inserted` holds what replaced it
    /// (empty for a pure delete-selection). FR-005 / FR-006 / FR-007.
    Replace {
        index: usize,
        removed: String,
        inserted: String,
    },
}
```

### Fields

| Field | Type | Meaning |
|------|------|---------|
| `index` | `usize` | Start char index of the replaced range (= `min(anchor, head)`). |
| `removed` | `String` | The full deleted substring (may span lines incl. `\n` / `\r\n`). Non-empty by construction. |
| `inserted` | `String` | The replacement text; empty for delete-selection; the typed chars for replace-by-typing. May be empty. |

### Behavior

| Operation | Forward diff (Redo) | Reverse diff (Undo) |
|-----------|---------------------|---------------------|
| `Replace` | `text.remove(index .. index + removed.chars().count()); text.insert(index, &inserted);` | `text.remove(index .. index + inserted.chars().count()); text.insert(index, &removed);` |
| `before_cursor()` | `index` (selection start) | |
| `after_cursor()` | `index + inserted.chars().count()` (cursor after typed text; equals `index` when `inserted` is empty, matching FR-006's "cursor at selection start") | |

### Invariants (constitution II/IV)

- **Non-degeneracy**: a `Replace` step is only recorded when `removed` is non-empty (i.e. there was an actual selection to remove). A replace over an empty selection is not a `Replace` — it falls back to the existing `Insert` path (FR-008).
- **Atomicity** (FR-007): one `record` call → one undo step → one Ctrl-Z restores the pre-edit text and cursor. `record` still clears the redo stack (existing invariant).
- **Reverse-diff symmetry**: `undo` then `redo` (or vice versa) is a no-op on rope content and leaves the cursor at `after_cursor()`, exactly like the existing `Insert`/`Delete` arms.
- **Multi-line** (FR-009): `removed` carries the whole deleted span including intervening `\n`; undo re-inserts it verbatim.

### Relationships

- Recorded by `EditingSession` from the new atomic replace/delete selection path in `handle_editing_command`.
- Lives on the existing `History.undo`/`redo` stacks; no schema change to `History` itself.

---

## Entity 3 — `EditingSession` (extended)

**Location**: `src/app.rs`.

### New field

```rust
pub struct EditingSession {
    // ... existing fields unchanged ...
    /// Active text selection, or `None` when no text is selected.
    /// Session-bound, in-memory, never persisted. FR-001/FR-013.
    pub selection: Option<Selection>,
}
```

### Lifecycle

- `EditingSession::new` / `open`: `selection = None` (no selection at open).
- Cleared on: plain `Move*`, edit commands after consuming the selection, `Undo`/`Redo`.
- Preserved across: `Search`, `FindNext`, `Save`, `Resize`, and prompt entry (FR-011).
- Not persisted: save/reload leave it untouched; reload (currently) resets cursor to 0 and so the selection would be cleared by the caller as part of cursor reset (consistent, no split clusters).

### New behaviors (summary; full contract in [contracts/api.md](./contracts/api.md))

| Command (Editing mode) | With `None` selection | With non-empty `Some` selection |
|---|---|---|
| `MoveSelect{Left,Right,Up,Down}` | seed anchor = cursor, move head | move head only (anchor fixed) |
| `Move{Left,Right,Up,Down}` | existing motion | existing motion **+ clear selection** |
| `InsertChar(c)` / `Enter` | existing insert path + `Insert` step | atomic `Replace` (removed=selection, inserted=typed) → clear selection |
| `Backspace` / `Delete` | existing single-char delete path | atomic `Replace` (removed=selection, inserted="") → clear selection |
| `Undo` / `Redo` | existing | existing **+ clear selection** |
| `Search`/`FindNext`/`Save`/`Quit` | unchanged | unchanged, selection preserved |

### Validation rules carried over

- Read-only document: replace-by-typing and delete-selection are blocked with the same "Read-only: edits are blocked" status as the existing edit seams (constitution III). Plain and Shift+arrow motion still works.
