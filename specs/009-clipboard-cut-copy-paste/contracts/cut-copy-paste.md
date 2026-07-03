# Contract: Clipboard Cut, Copy & Paste Interface

## Contract Type

UI / Key-Binding Contract — defines how user input maps to editor behavior for clipboard operations.

## Interface Boundary

The boundary is the TUI event loop in `app.rs::handle_editing_command()`. The contract governs:
1. **Input**: which keypresses trigger each clipboard command
2. **Output**: what visible state changes occur and what status messages are shown
3. **Side effects**: OS clipboard read/write operations (internal, not external-facing)

## Key Bindings

| Command | Keyboard Shortcut | Condition | Effect |
|---|---|---|---|
| `Copy` | Ctrl-C | Any | Writes current selection or single grapheme to OS clipboard; text unchanged in editor |
| `Cut` | Ctrl-X | Editing mode | Writes current selection or single grapheme to OS clipboard; deletes source from buffer; undoable in one step |
| `Paste` | Ctrl-V | Editing mode | Inserts OS clipboard text at cursor (or replaces selection); no-op on empty/missing clipboard; undoable in one step |

## Behavior Contract

### Copy (Ctrl-C) — FR-001

**Precondition**: None. Editor must be in a mode that accepts commands (not Exiting).
**Action**:
1. If `selection` is Some and non-empty: use selected text as copy source.
2. Otherwise: get the first grapheme cluster starting at `cursor.char_index`.
3. Write to OS clipboard via `arboard::set_text()`.
4. **Do not** modify buffer, cursor, or selection.
5. Show status: `"Copied {N} chars"` (where N = character count of clipboard content).
**Postcondition**: Text in editor unchanged; OS clipboard updated.

### Cut (Ctrl-X) — FR-002, FR-003

**Precondition**: `selection` is Some and non-empty **OR** cursor has a valid grapheme after it.
**Action**:
1. If `selection` is Some and non-empty: source = selected text → `EditStep::Replace { removed, inserted: "" }`.
2. Otherwise: source = first grapheme cluster after cursor → `EditStep::Delete { index, text }`.
3. Record the step in history (clears redo stack).
4. Write the same text to OS clipboard via `arboard::set_text()`.
5. Update cursor: remains at delete position (FR-011).
6. Show status: `"Cut {N} chars"`.
**Postcondition**: Source text deleted; OS clipboard updated; document marked dirty. Single undo restores everything.

### Paste (Ctrl-V) — FR-004, FR-005

**Precondition**: None. Editor reads current OS clipboard content on every invocation.
**Action**:
1. Read OS clipboard via `arboard::get_text()` → `Option<String>`.
2. If `None` or empty string: no-op (no status message, no undo entry).
3. If length > 1 MB: show `"Clipboard content too large (>1 MB)"`; no edit performed.
4. Determine source text:
   - If `selection` is Some and non-empty: source = selected range → `EditStep::Replace`.
   - Otherwise: source = cursor position → `EditStep::Insert`.
5. Perform the edit (insert/replace) in buffer.
6. Clear selection. Move cursor to end of inserted text.
7. Show status: `"Pasted {N} chars"`.
**Postcondition**: Clipboard text inserted at cursor; selection cleared; document dirty; single undo restores original state.

### Undo Behavior — FR-005

All three commands produce exactly **one** `EditStep` in history, undoable via a single Ctrl-Z.
After Undo: the system clipboard is **unchanged** (FR-006) — Ctrl-V can re-inject the same content.

## Status Messages

| Event | Message Type | Format / Content |
|---|---|---|
| Copy succeeds | Info | `"Copied N chars"` |
| Cut succeeds | Info | `"Cut N chars"` |
| Paste succeeds | Info | `"Pasted N chars"` |
| Empty clipboard paste | None | (silent, no-op) |
| Over-1 MB clipboard | Warning | `"Clipboard content too large (>1 MB)"` |
| No grapheme to cut at cursor | Info | `"Nothing to cut"` |
