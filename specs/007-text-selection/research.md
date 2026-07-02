# Research: Text Selection

**Branch**: `007-text-selection` | **Date**: 2026-07-02 | **Spec**: [spec.md](./spec.md)

The Technical Context in [plan.md](./plan.md) had no NEEDS CLARIFICATION items — the language, dependencies, testing, platform, and project type are all fixed by the existing repo (`architecture.md`, `Cargo.toml`). The research below resolves the **design-decision unknowns** raised by the spec's edge cases and assumptions (anchor-vs-head model, undo atomicity, CRLF/Unicode handling, rendering approach, and key-binding precedence), so Phase 1 can proceed.

---

## R1: Selection representation — anchor + head, not start + end

**Decision**: Model selection as a fixed **anchor** (the position where the user first pressed Shift+arrow) plus a moving **head** (the live cursor position). Direction is derived: forward when `head >= anchor`, backward when `head < anchor`. The visible/acted-upon range is always `[min(anchor, head), max(anchor, head))`.

**Rationale**: The spec's Key Entity "Selection" explicitly names Anchor and Head as the two key attributes, and edge case 2 ("cursor runs past the anchor") requires the anchor to stay fixed while the head crosses it — this is exactly the anchor/head model, not a start/end model (which would lose direction information). It also matches the mental model of every common editor (the cursor is the moving end; the anchor is the dropped pin).

**Alternatives considered**:
- *Start/end pair with a separate `direction` enum*: forces three fields to stay consistent (start, end, direction) and makes "run past the anchor" a state-transform bug magnet. Rejected.
- *Storing only the range + a "which end is the cursor" flag*: equivalent to anchor/head but with a less intent-revealing name. Rejected on readability grounds (constitution I).

## R2: Where selection state lives

**Decision**: Add a `pub selection: Option<Selection>` field on `EditingSession` in `app.rs`. `Selection { anchor: usize, head: usize }` (char indices) is a plain `Copy` struct defined in `cursor.rs` alongside `CursorState` (the existing cursor-state module). `None` ⇔ empty/no selection (length-zero or collapsed).

**Rationale**: The constitution's Maintainability gate (II) requires clear boundaries and "stateful structs only for compound concerns" — `EditingSession` is already the compound-state owner (it owns `cursor`, `viewport`, `search`, `history`). Placing selection there keeps one reason-to-change for session state. Defining the type in `cursor.rs` co-locates it with the motion functions that operate on it. `None` for empty avoids a degenerate "anchor==head but still Some" representation that the spec's edge case 3 explicitly calls out as "no visible selection."

**Alternatives considered**:
- *New `selection.rs` module*: adds a file for one ~10-line struct + four motion functions. Over-modularization; rejected per scope gate (V).
- *Encode selection inline as `(Option<usize>, Option<usize>)` on the session*: harder to read and to invariant-check. Rejected.

## R3: Atomic undo for replace-by-typing and delete-selection

**Decision**: Add one new `EditStep::Replace { index: usize, removed: String, inserted: String }` variant to `history.rs`. Its forward diff = remove `[index, index + removed.chars().count())` then insert `inserted` at `index`; its reverse diff = remove the inserted text then re-insert `removed`. `before_cursor` = `index` (the selection start); `after_cursor` = `index + inserted.chars().count()` (cursor lands after the typed text). Both delete-selection (FR-006) and replace-by-typing (FR-005) record a single `Replace` step where `inserted` is empty (delete) or the typed string (replace).

**Rationale**: The spec demands FR-007 "single step in the undo history" for both actions, and the Assumptions section explicitly anticipates extending the `EditStep` model. One `Replace` variant covers both delete-selection (`inserted == ""`) and replace-by-typing (`inserted != ""`), avoiding two near-identical variants. It preserves the existing `History` invariants (clear-redo-on-record, reverse-diff symmetry) because it is just one more enum arm with paired forward/reverse diffs. Multi-line selections work naturally because `removed` is the full deleted substring (including `\n`), exactly as the existing `Delete` variant already stores full removed text.

**Alternatives considered**:
- *Two separate edits: record a `Delete` then an `Insert` in `record`*: produces two undo steps → violates FR-007 (would need two Ctrl-Z). Rejected.
- *A "group/transaction" wrapper around existing steps*: a far heavier change to `History` and a new concept, for a single use case. Rejected by scope gate (V).
- *Only a `Delete` variant for deletion, plus a `Replace` variant only for replace*: two variants that share an identical forward/reverse structure. Rejected for readability/duplication (constitution I/II).

## R4: Key-binding precedence and Shift detection

**Decision**: Add four `EditorCommand` variants — `MoveSelectLeft`, `MoveSelectRight`, `MoveSelectUp`, `MoveSelectDown` — mapped in `input.rs::map_key_event` from `(KeyModifiers::SHIFT, KeyCode::Left/Right/Up/Down)`. Match these arms **before** the existing bare-arrow arms (which already use `(_, KeyCode::Arrow)` and so would otherwise swallow Shift+arrows). Plain arrows stay mapped to existing `Move*`. The bare-printable catch-all already excludes `KeyModifiers::CONTROL`; it does not need a `SHIFT` exclusion because shift+letter typically arrives as a different (upper-case) `Char`, still routed to `InsertChar` — desired behavior.

**Rationale**: `crossterm` already surfaces `KeyModifiers::SHIFT` on `KeyEvent`; no new dependency. Putting the mapping in `input.rs` preserves the established "single source of truth for key bindings" pattern (architecture.md → Key Design Patterns). Ordering the new arms first mirrors the existing Ctrl-Z/Ctrl-Y-before-catch-all precedence trick (per undo-redo contract §3).

**Alternatives considered**:
- *Pass `KeyModifiers` through and let `app.rs` decide*: violates the input-mapping isolation pattern. Rejected.
- *One `MoveSelect(Motion)` variant*: more compact but harder to read in the `match` and loses the one-variant-per-key clarity. Rejected (constitution I).

## R5: Cursor motion semantics on Shift+arrows

**Decision**: Each `MoveSelect*` first ensures a selection exists (if `selection.is_none()`, seed it with `anchor = cursor.char_index`), then moves the head by calling the **existing** `cursor::move_left/right/up/down` on a throwaway `CursorState` mirror of the head, then writes the resulting index back as the new head and updates the real `cursor`. The anchor is never moved by `MoveSelect*`. `preferred_column` is honored on `MoveSelectUp/Down` exactly as for plain Up/Down (per Assumptions).

**Rationale**: Reusing the existing, tested motion functions (rather than reimplementing bounded moves) satisfies "do not split grapheme clusters / respect document bounds" for free — `move_up/down` already clamp to line range and `move_left/right` already `saturating_sub`/`.min(len_chars)`. This directly satisfies FR-003 (no growth past document bounds). Reusing the functions is the simplest design (constitution V) and avoids drift between plain and selection moves.

**Alternatives considered**:
- *Reimplement moves inline in selection handlers*: duplicates the clamp logic and is a future divergence bug source. Rejected.

## R6: Collapse-on-plain-move and edit actions

**Decision**: In `handle_editing_command`, every `Move{Left,Right,Up,Down}` (non-shift), `InsertChar`, `Enter`, `Backspace`, `Delete`, `Undo`, `Redo`, `Search`, `FindNext`, `Quit`, `Cancel` first clears `selection = None` — *except* `Search`/`FindNext`/`Save`/`Quit`-prompt which the spec's edge case 5 / FR-011 say must leave the selection untouched unless the mode change demands otherwise. For `InsertChar`/`Enter`/`Backspace`/`Delete`: if a non-empty selection exists, route through the new atomic replace/delete path; otherwise fall back to the existing single-char path (FR-008). `Undo`/`Redo` clear the selection (the restored text state has no meaningful selection).

**Rationale**: FR-004 mandates collapse on plain cursor motion. FR-005/FR-006 mandate atomic edit-while-selected. FR-008 mandates unchanged behavior when no selection exists. FR-011 protects the selection from non-editing commands. The single `handle_editing_command` match is the existing dispatch choke point, so adding the clear-and-route there keeps one readable place for all cursor/edit entry behavior.

**Alternatives considered**:
- *Clear selection lazily on next plain move only*: leaves the selection live during editing commands other than the four edit variants, contradicting FR-004's "immediately." Rejected.

## R7: Rendering the selection highlight

**Decision**: Extend `RenderView` so each rendered body line carries, where it intersects the selection, the substring ranges that should be highlighted (inversely). Concretely: compute, for each visible line row, the `[char_start, char_end)` intersection of the selection with that line's character range, map to visual columns, and emit a `Vec<HighlightSpan>` (column range) on the line entry. `main.rs::draw()` already owns the ratatui widget assembly; it applies `Style::default().reversed()` to those spans. The pure-projection property of `render.rs` is preserved (no side effects, no styling decisions beyond "which columns are selected").

**Rationale**: FR-010 requires the selected range to be visibly highlighted. Keeping the *which columns* decision in `render.rs` (pure) and the *how to style* decision in `main.rs::draw` follows the existing "Render split across layers" pattern (architecture.md). Exposing column ranges (not ratatui `Style`) keeps the unit test for render pure and assertion-friendly.

**Alternatives considered**:
- *Push a full `ratatui::Style` into `RenderView`*: couples the pure render projection to ratatui types, breaking the existing layer split and making unit tests depend on the widget crate. Rejected (constitution II — boundary lines).
- *Highlight only in `main.rs`, recomputing selection geometry there*: duplicates the line/char→column math already in `render.rs`/`cursor.rs`. Rejected.

## R8: Unicode / grapheme / CRLF correctness

**Decision**: Selection operates purely on **character indices** (`usize` into the `Rope`), identical to every existing buffer/cursor/history operation (architecture.md → Text Model Details). Because a `Rope` char is a Unicode scalar value and the grapheme-aware helpers (`graphemes(true)`, `UnicodeWidthStr`) are already used in `cursor.rs`/`render.rs` for column math, a selection never splits a grapheme cluster: the anchor/head are always valid char indices produced by the existing motion functions, and deletion uses `text.remove(start..end)` over whole chars. CRLF (`\r\n`) is two chars; selecting/deleting across them removes both whole, exactly as the existing `Delete` step already stores `"\r\n"` faithfully when present.

**Rationale**: The spec's edge case 7 / FR-012 require no halved clusters. By reusing the char-index + grapheme-aware-column model already proven in the codebase, the feature inherits the guarantees rather than re-deriving them. No new unicode logic is needed.

**Alternatives considered**:
- *Grapheme-offset-based selection*: would diverge from the Rope's char-index API and the entire existing codebase. Rejected.

## R9: Document-boundary and empty-line / newline-only selections

**Decision**: Clamping is inherited from the reused motion functions (R5): `move_left` `saturating_sub`s at 0, `move_right`/up/down clamp to line/end. The selection range fed to `buffer::remove`/`replace_range` is `min(anchor,head) .. max(anchor,head)`, further clamped by `buffer::clamp_char_index` (already used by `replace_range`). A selection spanning only `\n` chars or a whole empty line works because deletion is a plain char-range remove; `removed` stores the exact bytes (including `\n`, optionally `\r\n`), so undo restores them precisely.

**Rationale**: Edge cases 1 (boundary), 4 (newline-only / empty-line), and 5 are all handled by composing existing clamps with a `min/max` range — no new boundary code.

## R10: Read-only interaction

**Decision**: When the document is read-only, replace-by-typing and delete-selection are blocked exactly like the existing `insert_text`/`backspace`/`delete` seams (they already early-return with a "Read-only" status). Selection *building and collapsing* (Shift+arrows, plain arrows) still works in read-only mode — selection is non-destructive and useful for, e.g., copy scenarios later.

**Rationale**: Consistent with constitution III (fail safely; destructive actions require non-read-only) and the existing read-only guards. No new risk introduced.
