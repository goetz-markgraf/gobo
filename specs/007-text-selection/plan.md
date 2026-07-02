# Implementation Plan: Text Selection

**Branch**: `007-text-selection` | **Date**: 2026-07-02 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/007-text-selection/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Add a visual text-selection capability to `gobo`: a fixed **anchor** plus a moving **head** (the cursor), driven by `Shift+Arrow` movements. Non-shift cursor motion collapses the selection; typing a character or pressing `Del`/`Backspace` while a non-empty selection exists replaces or deletes the whole selected range as a **single atomic undo step**. The render path highlights the selected range. The feature reuses the existing character-index text model, the `Rope` buffer, and the `History`/`EditStep` undo machinery, extended with one new `EditStep` variant `Replace` (delete-then-insert in one step). It touches only `cursor.rs` (selection state + motions), `history.rs` (`Replace` step), `app.rs` (dispatch, atomic edit, collapse), `input.rs` (Shift-modified arrows → `MoveSelect*` commands), and `render.rs` (highlighting). No new dependencies, no new modules.

## Technical Context

**Language/Version**: Rust 2024 edition (per `Cargo.toml`).

**Primary Dependencies**: `ropey` 1.6 (Rope buffer, char-index ops), `crossterm` 0.28 (`KeyModifiers::SHIFT` already exposed on `KeyEvent`), `ratatui` 0.29 (selection highlight via styled spans), `unicode-segmentation` + `unicode-width` (grapheme-aware column math, reused).

**Storage**: N/A — selection and history are session-bound, in-memory only, never persisted (consistent with constitution III and the history module's existing session-lifetime invariant).

**Testing**: `cargo test` with standalone `[[test]]` targets. Unit tests under `tests/unit/` for pure selection/cursor functions and the new `EditStep::Replace` symmetry; integration tests under `tests/integration/` driving `EditingSession::open()` + `handle_command()` for the selection/edit/undo flows.

**Target Platform**: Local terminal (cross-platform via crossterm).

**Project Type**: Single-binary local terminal editor / desktop-app (per architecture.md).

**Performance Goals**: No new goals. Selection state is O(1) (two indices); rendering highlight is O(visible lines) like the existing render path. `History` stays in-memory and unbounded in production.

**Constraints**: Operational constraints from constitution: no network, no telemetry of document content, single-document, local-first. Selection must operate on character indices (never splitting grapheme clusters), consistent with the existing buffer model.

**Scale/Scope**: Single document, interactive editing. No background services, no plugins, no multi-document orchestration — stays inside constitution V (Scope and Simplicity Control).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Readability Gate**: Main modules touched and their responsibilities stay as documented in `architecture.md`. `cursor.rs` gains the `Selection` type (anchor + direction) and the `MoveSelect*` functions alongside existing motion functions — same responsibility (cursor/selection state and motion), readable co-location, stateless where possible. `history.rs` gains one `EditStep::Replace` variant — clear extension of the existing enum, with the same forward/reverse-diff symmetry documentation pattern. `app.rs` gains the selection-collapsing and atomic replace/delete branches in `handle_editing_command`, mirroring existing `insert_text`/`backspace`/`delete` seams. `input.rs` adds four `MoveSelect*` arms in the single source-of-truth key-binding table. `render.rs` adds the selection-highlight projection (no side effects, pure). Names reveal intent (`selection`, `anchor`, `head`, `MoveSelectLeft`, `EditStep::Replace`). No hidden state, no dense abstractions.

- **Maintainability Gate**: Boundary lines preserved — selection state lives on `EditingSession` (the existing compound-state owner), not scattered. Text mutation stays in `buffer.rs` (`replace_range` already exists). Undo recording stays in `history.rs`. Rendering stays a pure projection in `render.rs`. Input mapping stays isolated in `input.rs`. The only new abstraction is `EditStep::Replace` + the `Selection` struct, each justified: `EditStep::Replace` is required to make delete-then-insert one undo step (FR-007); `Selection` is required to hold anchor+direction (FR-002, FR-014). No new dependencies. No new crates.

- **Security Gate**: User-data risks: a multi-line selection deletion is a destructive action — mitigated by the atomic undo guarantee (FR-007/SC-003), exactly one Ctrl-Z restores the full content. Validation: selection indices are clamped to document bounds (FR-003), never exceeding `text.len_chars()`; the minimum of anchor/head defines the deletion start, preventing out-of-range removes. Input: shift-arrow mapping is additive and cannot alias to printable insertion (control modifier guard already excludes control; shift alone is fine). File writes unchanged — no silent overwrite risk introduced; `save` does not touch selection or history (per existing FR-013). No credentials, telemetry, or external content transmission added.

- **Verification Gate**: Automated tests planned (FR-014/SC-005): unit tests for `Selection` direction/shrink/reverse-over-anchor and for `EditStep::Replace` forward/reverse symmetry; integration tests for (a) build/shrink/reverse selection via Shift+arrows, (b) collapse on non-shift motion, (c) replace-on-type with single Ctrl-Z restore, (d) Del/Backspace deletion with single Ctrl-Z restore, (e) multi-line selection delete, (f) document-boundary clamp, (g) empty-selection no-op (degrades to existing behavior), (h) CRLF / multi-grapheme consistency. Terminal visual highlighting cannot be asserted from automation beyond the `RenderView` projection; a documented manual check covers the inverse-color appearance (constitution IV permits manual supplement, not replacement).

- **Scope Gate**: Feature fits approved product scope — it is core single-document editing, adds no persistent services, no network, no plugins, no multi-document orchestration. One extension to the existing `EditStep` enum and one new session-owned `Selection` field. No constitutional exception required.

## Project Structure

### Documentation (this feature)

```text
specs/007-text-selection/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── api.md
└── tasks.md             # Phase 2 output (NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── main.rs              # draw(): apply selection highlight spans (render concern, minimal change)
├── lib.rs               # unchanged re-exports
├── cli.rs               # unchanged
├── document.rs          # unchanged
├── app.rs               # Selection field on EditingSession; MoveSelect* dispatch; atomic
│                        #   replace/delete; collapse-on-plain-move; SHIFT move handlers
└── editor/
    ├── mod.rs           # re-export selection module
    ├── buffer.rs        # reuse existing replace_range / remove / insert; possibly a
    │                    #   remove_range helper if cleaner than inline remove()
    ├── cursor.rs        # NEW: Selection struct + MoveSelect{Left,Right,Up,Down} functions
    ├── input.rs         # NEW commands: MoveSelect{Left,Right,Up,Down} mapped from Shift+arrows
    ├── render.rs        # RenderView gains selection range per visible line (or highlight spans)
    ├── search.rs        # unchanged
    ├── status.rs        # unchanged
    └── history.rs       # EDIT: new EditStep::Replace variant + forward/reverse apply

tests/
├── unit/
│   ├── cursor.rs        # extended: Selection direction/shrink/reverse + MoveSelect* unit tests
│   ├── history.rs       # extended: EditStep::Replace forward/reverse symmetry, clear-redo
│   └── ... (buffer, search, render unchanged or lightly extended)
└── integration/
    └── text_selection.rs # NEW: full selection/replace/delete/undo flows through EditingSession
```

**Structure Decision**: Single-project layout (Option 1 of the template), matching the existing repo. All changes are edits to existing modules plus one new integration test file and unit-test extensions, keeping the module map in `architecture.md` accurate. A new `tests/integration/text_selection.rs` standalone `[[test]]` target is added to `Cargo.toml`.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations. (Table intentionally empty.)
