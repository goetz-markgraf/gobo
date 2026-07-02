# Contracts: Text Selection

**Branch**: `007-text-selection` | **Date**: 2026-07-02 | **Spec**: [spec.md](./spec.md)

`gobo` is a single-binary local terminal editor with no network/RPC surface. The contracts below are the **public, programmable interfaces** that this feature adds or extends and which automated tests (and any embedding harness) rely on. Type details live in [data-model.md](../data-model.md); design rationale in [research.md](../research.md).

---

## 1. Programmatic contract — `gobo::editor::cursor`

New `Selection` type plus four selection-motion functions, all in the existing `cursor` module.

### Types

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Selection {
    pub anchor: usize,
    pub head: usize,
}

impl Selection {
    /// Half-open char range actually covered: [min(anchor,head), max(anchor,head)).
    pub fn range(self) -> std::ops::Range<usize> { /* ... */ }
    /// `true` iff `anchor == head` (no visible selection).
    pub fn is_empty(self) -> bool { /* ... */ }
    /// `forward` iff `head >= anchor`.
    pub fn is_forward(self) -> bool { /* ... */ }
}
```

### Selection-motion operations

| Signature | Postcondition | Boundary / no-op behavior |
|-----------|---------------|---------------------------|
| `move_select_left(sel: &mut Option<Selection>, cursor: &mut CursorState, text: &Rope)` | If `sel` was `None`, seed `anchor = cursor.char_index`. Move head one char left. `sel = Some`; `cursor.char_index` = new head; `preferred_column` recomputed. | At doc start (`char_index == 0`): head stays 0, anchor seeded if needed; `Selection` may be empty. |
| `move_select_right(sel, cursor, text)` | Symmetric to left, moving right. | At doc end (`char_index == len_chars`): head stays clamped. |
| `move_select_up(sel, cursor, text)` | Seed anchor if needed; move head one line up using the existing `move_up` logic (honors `preferred_column`). | On top line: head stays, `preferred_column` recomputed; no overflow. |
| `move_select_down(sel, cursor, text)` | Symmetric to up. | On last line: head stays clamped. |

**Reused-motion invariant (FR-003/FR-012)**: all four functions reuse the existing `cursor::move_left/right/up/down` for the head movement, so document-bound clamping and grapheme-aware column preservation are identical to plain motion. The anchor is seeded exactly once (when `sel` transitions `None` → `Some`) and never moved by these functions.

**Direction-flip invariant (FR-002, edge case 2)**: the head may move across the anchor; `Selection` represents this naturally (`is_forward()` flips). No special-case code.

---

## 2. Programmatic contract — `gobo::editor::history`

`EditStep` gains one variant. `History` itself is unchanged.

### New `EditStep` variant

```rust
pub enum EditStep {
    Insert { index: usize, text: String },                 // existing, unchanged
    Delete { index: usize, text: String },                 // existing, unchanged
    Replace { index: usize, removed: String, inserted: String },  // NEW
}
```

| Method | `Insert` | `Delete` | `Replace` (NEW) |
|--------|----------|----------|------------------|
| `index()` | `index` | `index` | `index` |
| `len_chars()` | `text.chars().count()` | `text.chars().count()` | `removed.chars().count()` (covers the deleted range) |
| `end_index()` | `index + len_chars` | `index + len_chars` | `index + removed.chars().count()` (end of the removed range) |
| `before_cursor()` | `index` | `end_index()` | `index` (selection start) |
| `after_cursor()` | `end_index()` | `index` | `index + inserted.chars().count()` (cursor after inserted text; equals `index` when `inserted == ""`) |
| `apply_forward(text)` | insert `text` at `index` | remove `[index, end)` | remove `[index, end)` then insert `inserted` at `index` |
| `apply_reverse(text)` | remove `[index, end)` | insert `text` at `index` | remove `[index, index + inserted.chars().count())` then insert `removed` at `index` |

### `History` operations (unchanged signatures, now also accept `Replace`)

`record`, `undo`, `redo`, `can_undo`, `can_redo`, `clear` — all unchanged in signature and postconditions (see the undo-redo contract). A `Replace` step flows through them identically:

- `record(Replace{..})`: push onto `undo`, clear `redo`, evict oldest if at capacity (same `RecordOutcome`).
- `undo`: pop, `apply_reverse`, push onto `redo`, return `Some(before_cursor()) == Some(index)`.
- `redo`: pop, `apply_forward`, push onto `undo`, return `Some(after_cursor())`.

### Carried invariants

- **Atomicity (FR-007)**: one `Replace` step = one undo unit; exactly one Ctrl-Z restores the pre-edit rope and cursor.
- **Clear-redo-on-record**: unchanged — recording a `Replace` empties `redo`.
- **Reverse-diff symmetry**: `undo` ↔ `redo` is a no-op on rope content; cursor returns to `after_cursor()`.
- **Non-degeneracy**: `app.rs` only records a `Replace` when `removed` is non-empty (a real selection existed). An empty selection routes to the existing `Insert`/`Delete` path (FR-008).

---

## 3. Programmatic contract — `gobo::app::EditingSession`

### New field

```rust
pub struct EditingSession {
    // ... existing fields ...
    pub selection: Option<Selection>,   // NEW; None on new()/open()
}
```

### Editing-mode command handling (extends existing `handle_editing_command`)

| `EditorCommand` | `selection == None` | `selection == Some(non-empty)` |
|-----------------|----------------------|--------------------------------|
| `MoveSelectLeft/Right/Up/Down` (NEW) | seed anchor = cursor; move head; `selection = Some` | move head only; `selection` stays `Some` with fixed anchor |
| `MoveLeft/Right/Up/Down` | existing motion | existing motion **+ `selection = None`** |
| `InsertChar(c)` | existing `insert_text` + `Insert` step | atomic `Replace { removed=<selection text>, inserted=c }`; clear selection; FR-005 |
| `Enter` | existing newline insert + `Insert` step | atomic `Replace { removed=<selection text>, inserted="\n" }`; clear selection |
| `Backspace` | existing single-char-back delete + `Delete` step | atomic `Replace { removed=<selection text>, inserted="" }`; cursor at `index`; clear selection; FR-006 (same effect as `Delete`) |
| `Delete` | existing single-char-fwd delete + `Delete` step | atomic `Replace { removed=<selection text>, inserted="" }`; cursor at `index`; clear selection; FR-006 |
| `Undo` / `Redo` | existing (may be no-op) | existing behavior **+ `selection = None`** |
| `Search` / `FindNext` / `Save` / `Quit` | unchanged | unchanged; **selection preserved** (FR-011) |
| `Cancel` / `NextChoice` / `PreviousChoice` / `Resize` | unchanged | selection preserved (non-editing) |

**Atomic-edit postcondition** (FR-005/FR-006/FR-007): after `InsertChar`/`Enter`/`Backspace`/`Delete` on a non-empty selection, the rope equals `original with [start,end) replaced by inserted`, `cursor.char_index = start + inserted.chars().count()`, `selection == None`, `document.dirty == true`, exactly **one** new `History.undo` step of kind `Replace`, `history.redo` empty, and `status` shows the usual info/warning per `record` outcome.

**Read-only interaction (constitution III)**: when `document.is_read_only()`, the four edit commands on a non-empty selection are blocked with `status = Some(StatusMessage::warning("Read-only: edits are blocked"))`; the rope, selection, cursor, dirty flag, and history are unchanged. Selection building/collapsing (Shift+arrows, plain arrows) still works in read-only mode.

**Empty-selection fallthrough (FR-008)**: when `selection.is_none()` (or `Some` with `is_empty()`), all commands behave exactly as today — no `Replace` step is recorded; selection-aware branches are skipped.

**Session-lifetime invariant**: a freshly constructed or `open()`-ed `EditingSession` has `selection == None`.

### Prompt/search/search-mode interaction

- `EditorCommand::Search`/`FindNext`/`Save`/`Quit` and `Resize` do not clear `selection` (FR-011).
- Entering `SearchInput` mode does not clear the selection; the search flow ignores it. (Copy-style use is out of scope but selection persistence is preserved.)
- `Undo`/`Redo` are still ignored in `SearchInput`/prompt modes (unchanged).
- `SaveConflictPrompt` → `Reload` resets the cursor; the caller clears `selection` as part of the cursor reset (consistent, no split clusters). `Overwrite`/`Cancel` leave `selection` as-is.

---

## 4. Input contract — `gobo::editor::input`

### New `EditorCommand` variants

```rust
pub enum EditorCommand {
    // ... existing 16 variants unchanged ...
    MoveSelectLeft,   // NEW
    MoveSelectRight,  // NEW
    MoveSelectUp,     // NEW
    MoveSelectDown,   // NEW
}
```

### New `map_key_event` arms

| Key event | Command |
|-----------|---------|
| `(KeyModifiers::SHIFT, KeyCode::Left)` | `MoveSelectLeft` |
| `(KeyModifiers::SHIFT, KeyCode::Right)` | `MoveSelectRight` |
| `(KeyModifiers::SHIFT, KeyCode::Up)` | `MoveSelectUp` |
| `(KeyModifiers::SHIFT, KeyCode::Down)` | `MoveSelectDown` |

**Precedence**: these four arms are matched **before** the existing `(_, KeyCode::Left/Right/Up/Down)` arms (which currently use the wildcard modifier `_` and would otherwise catch Shift+arrows as plain moves). The existing bare-printable catch-all already excludes `KeyModifiers::CONTROL` and is unaffected — Shift+letter arrives as the upper-case `Char` and still routes to `InsertChar` (desired). All existing bindings (Ctrl-S/Q/F/G/Z/Y, plain arrows, Enter, Esc, Tab, Backspace, Delete, printable) are unchanged.

### Unchanged dispatch arms

`handle_search_command` and `handle_prompt_command` add no `MoveSelect*` arms — selection moves are Editing-mode only (Shift+arrow in a prompt/search textbox is out of scope and currently unmapped, returning `None`, same as today for any unmapped shift combo).

---

## 5. Render contract — `gobo::editor::render`

`RenderView` and `render_view` extend so the selection is visible (FR-010), preserving the pure-projection property (no side effects, no ratatui types).

### `RenderView` extension

Each visible body line entry carries, in addition to its string, a list of **visual-column highlight spans** indicating intersection with the selection:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HighlightSpan {
    pub start_col: usize,  // visual column, line-local
    pub end_col: usize,    // exclusive
}

pub struct RenderView {
    pub body_lines: Vec<BodyLine>,   // CHANGED from Vec<String>
    pub footer_line: String,
    pub bottom_line: Option<String>,
    pub popup: Option<PopupView>,
    pub cursor_x: u16,
    pub cursor_y: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BodyLine {
    pub text: String,
    pub highlights: Vec<HighlightSpan>,  // empty when no selection intersects this line
}
```

### `render_view` postcondition

- For each visible line `L`, if `session.selection` is `Some(s)` and `s.range()` intersects `L`'s char range, `highlights` contains the corresponding visual-column span(s) (mapped via the existing grapheme-aware `visual_column` math, one span per contiguous intersection; clipped to the visible viewport columns). Otherwise `highlights` is empty.
- All other `RenderView` fields are computed exactly as today (footer, bottom_line, popup, cursor position unchanged in formula).

### `main.rs::draw` postcondition

`draw()` (the existing ratatui-assembly layer, out of `render.rs`) applies `Style::default().reversed()` to each `HighlightSpan` on each body line. No styling decision lives in `render.rs` — only the geometry (constitution II boundary).

**Manual check (constitution IV)**: the inverse-color appearance of the selected text is asserted by a documented manual procedure in [quickstart.md](../quickstart.md), since terminal styling cannot be asserted from automation beyond the `RenderView` projection.

---

## 6. CLI contract — `gobo::cli`

Unchanged. The single positional `path: PathBuf` and `parse_args()` are unaffected by this feature.
