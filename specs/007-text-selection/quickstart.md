# Quickstart: Text Selection

**Branch**: `007-text-selection` | **Date**: 2026-07-02 | **Spec**: [spec.md](./spec.md)

This is a **validation run guide**, not an implementation reference. Implementation details live in `tasks.md` (Phase 2) and the source. Type/behavior contracts live in [contracts/api.md](./contracts/api.md); data shapes in [data-model.md](./data-model.md).

---

## Prerequisites

- Rust toolchain matching `Cargo.toml` (edition 2024). `cargo --version` works.
- A writable temp dir (integration tests use `tempfile`'s `tempdir()`).
- No external services, no network — purely local, per constitution V.

## Build

```bash
cargo build
cargo test --no-run      # compiles all [[test]] targets, including the new ones
```

## Automated Validation

Run the automated suites that prove the feature end-to-end through the real `EditingSession` (constitution IV — logic that can be tested repeatably MUST be automated).

### 1. Unit tests (pure functions)

```bash
cargo test --test unit_cursor       # Selection geometry + MoveSelect* motions, clamping, direction flip
cargo test --test unit_history      # EditStep::Replace forward/reverse symmetry, clear-redo, determinism
cargo test --test unit_render       # RenderView highlight-span projection (line/char→column mapping)
```

**Expected**: all pass. These cover the pure seams: `Selection::range/is_empty/is_forward`, the four `move_select_*` functions (incl. document-boundary clamp and anchor-cross direction flip), and `EditStep::Replace` `apply_forward`/`apply_reverse`/cursor helpers, plus the render-column highlights.

### 2. Integration tests (full session flows)

```bash
cargo test --test integration_text_selection
```

**Expected**: all pass. This new target drives `EditingSession::open()` + `handle_command()` to cover, per FR-014 / SC-005:

| Scenario group | What it proves |
|----------------|----------------|
| **Build / shrink / reverse** (Shift+Left/Right/Up/Down sequences) | FR-001, FR-002, SC-001: selection equals `[anchor, head)` after each move; grows/shrinks; head can cross anchor |
| **Collapse on plain move** (Left/Right/Up/Down without Shift) | FR-004, SC-002: `selection == None` after any plain move; cursor at the motion's natural position |
| **Replace by typing** (type one char / several chars over a selection) | FR-005, FR-007, SC-003: text replaced; one Ctrl-Z restores original + removes typed char(s); selection cleared; cursor after inserted text |
| **Delete selection** (Del and Backspace over a selection, including multi-line) | FR-006, FR-009, SC-003/SC-004: removed text gone; cursor at selection start; one Ctrl-Z restores; Del == Backspace effect; multi-line removes intervening newlines |
| **Empty / no selection fallthrough** (edit with `selection == None`) | FR-008: indistinguishable from today's single-char behavior; records the normal Insert/Delete step |
| **Boundaries & edge content** (selection at doc start/end; newline-only range; CRLF range; multi-grapheme cluster) | FR-003, FR-012, SC-004: no out-of-range removes; no halved clusters; `\r\n` removed/restored as a pair |
| **Non-editing commands preserve selection** (Search, FindNext, Save) | FR-011, edge case 5: `selection` unchanged by Search/FindNext/Save |
| **Undo/Redo around selection** (build selection, edit, Ctrl-Z, Ctrl-Y) | FR-007: round-trip restores rope content and cursor; selection cleared on Undo/Redo |

### 3. Full suite (no regressions)

```bash
cargo test
```

**Expected**: the entire existing suite continues to pass — no existing public signature changes; the new `RenderView.body_lines` type change is internal and the existing render unit test is updated in lockstep.

## Manual Validation (terminal UX only)

Constitution IV permits manual checks for terminal UX that cannot be asserted repeatably in automation. These supplement, not replace, the automated tests above.

1. `cargo run -- <some file>` in a real terminal (≥ 44×8 for the Full popup variant).
2. Type some text, e.g. `Hallo Welt`.
3. Place the cursor at line end, press **Shift+Left** three times → confirm the three chars appear **inverted/highlighted** (FR-010).
4. Press **Shift+Right** once → confirm the selection shrinks by one char from the right (FR-002).
5. Press **Right** (no Shift) → confirm the highlight disappears and the cursor moves normally (FR-004).
6. Build a selection over `llo`, type `x` → confirm the text reads `Hax…`, no highlight remains, cursor sits after `x`; **Ctrl-Z** → `llo` restored and the `x` gone, in one step (FR-005/FR-007).
7. Select `llo`, press **Delete** → `Ha` remains, cursor after `a`; **Ctrl-Z** → `Hallo` restored (FR-006).
8. Select across two lines, press **Backspace** → both lines plus the newline gone; **Ctrl-Z** → restored (FR-009).
9. Build a selection, press **Ctrl-F** (search), type a query, **Esc** → confirm the selection is still present afterward (FR-011).

## Expected Outcomes (Success Criteria mapping)

- **SC-001**: Step 3–4 — selection matches `[anchor, head)` at every move.
- **SC-002**: Step 5 — 100 % of plain moves clear the selection.
- **SC-003**: Steps 6–7 — one Ctrl-Z per atomic replace/delete.
- **SC-004**: Step 8 and the CRLF/multi-grapheme automated cases — no data loss, no halved clusters.
- **SC-005**: the `cargo test` run above is green.
