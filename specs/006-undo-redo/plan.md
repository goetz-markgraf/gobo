# Implementation Plan: Undo / Redo

**Branch**: `006-undo-redo` | **Date**: 2026-07-02 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/006-undo-redo/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Add per-keystroke Undo (Ctrl-Z) and Redo (Ctrl-Y) to the editor. The `EditingSession` gains two unbounded in-memory stacks (undo, redo). Every text mutation (insert, backspace, delete, enter/newline — the editor has no overwrite mode, so these exhaust the mutation kinds) records a single diff step onto the undo stack and clears the redo stack. Ctrl-Z pops the top undo step, applies its reverse diff, and pushes the forward diff onto the redo stack; Ctrl-Y performs the symmetric operation. Stacks live only for the session (never persisted). Behavior is gated to editing mode only. Storage exhaustion on push is handled by dropping the oldest undo step and informing the user; the in-progress edit is always applied.

## Technical Context

**Language/Version**: Rust 2024 edition (`Cargo.toml` → `edition = "2024"`)

**Primary Dependencies**: existing crate deps — `ropey` 1.6 (text buffer), `crossterm` 0.28 (key events), `ratatui` 0.29 (render), `clap` (CLI). No new dependencies.

**Storage**: in-memory only (`Vec`-backed stacks on the session). No persistence. N/A to disk.

**Testing**: `cargo test` — unit tests in `tests/unit/`, integration tests driving `EditingSession::open()` + `handle_command()` in `tests/integration/`. Each integration file is a standalone `[[test]]` target in `Cargo.toml`.

**Target Platform**: local terminal (macOS/Linux), single binary `gobo`.

**Project Type**: desktop-app (terminal editor).

**Performance Goals**: each Undo/Redo step applies a single small diff in O(n) at worst (rope removal/insert is O(log n)); recording a step is O(step size). No 60fps budget impact — operations are user-paced per keystroke.

**Constraints**: single-document, single-binary, no external services. Perkeystroke granularity (no grouping). No overwrite/replace mode — all text changes are insert or delete (spec clarification), so three record seams suffice and `EditStep` has exactly two variants. Stacks bounded only by available memory; OOM handled by dropping oldest undo step.

**Scale/Scope**: single local file; unbounded history length in memory.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Readability Gate**: A new `editor/history.rs` module owns the `History` struct (undo + redo stacks) and the `EditStep` type. `EditingSession` gains a single `history: History` field. Diff representation is one explicit enum with exactly two variants (`EditStep::Insert { index, text }` / `EditStep::Delete { index, text }`) — no `Replace` variant, because the editor has no overwrite mode (spec clarification 2026-07-02; all edits reduce to insert or delete). Intent-revealing names, straightforward control flow, no clever merge/grouping logic. The recording hook is added exactly where each existing mutation helper (`insert_text`, `backspace`, `delete`) already centralizes text changes in `app.rs`, so reviewers see one record point per mutation kind — and these three seams exhaust the editor's mutation surface.

- **Maintainability Gate**: Boundary lines preserved: `history.rs` holds pure history state and apply/undo/redo operations on a `Rope`; `app.rs` records steps at its existing mutation seams; `input.rs` adds only two key bindings (Ctrl-Z → `Undo`, Ctrl-Y → `Redo`); `render.rs` is unaffected (status messages already project from session). No new abstraction beyond the `History` struct itself, which is the minimum needed to keep the two stacks and their invariants (clear-redo-on-push, reverse-diff symmetry) in one place rather than scattered across `app.rs`.

- **Security Gate**: Risks: (a) memory exhaustion from unbounded stacks — fail-safe by dropping the oldest undo step and surfacing a status message; the user's last keystroke edit is never lost and the document text is never corrupted (FR-006). (b) destructive Undo/Redo overwriting unsaved external changes — undo/redo only touches the in-memory `Rope`, never disk, so it cannot destroy on-disk data; the save/conflict flow remains the only disk write path. (c) read-only mode — undo/redo are *not* edits; they restore previously-recorded states and are allowed even when read-only (recording is already blocked in read-only mode, so stacks stay empty there in practice). No new user-data exposure, logging, or network paths.

- **Verification Gate**: Automated tests via `EditingSession` interface covering: build history then repeated Undo back to origin; repeated Redo back to latest; Redo-empties-on-new-edit (Ctrl-Y no-op after new input); session lifetime (history empty after reopen — covered by fresh session having empty stacks); Unicode + newline steps; stack-end no-ops (Undo at empty stack, Redo at empty stack); OOM path (oldest undo dropped, edit applied, user informed). Unit tests on `History` cover reverse/forward diff symmetry directly on `Rope`. Terminal-UX rendering of OOM warning is checked manually (status message already auto-rendered).

- **Scope Gate**: Feature fits the approved single-document, single-binary, local-first scope. No persistent services, network, plugins, or multi-document orchestration. No constitutional exception required.

## Project Structure

### Documentation (this feature)

```text
specs/006-undo-redo/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── main.rs          # event loop: map_key_event → handle_command → draw (unchanged shape)
├── lib.rs           # re-exports modules (unchanged)
├── cli.rs           # CLI parsing (unchanged)
├── document.rs      # file I/O + save/conflict (unchanged)
├── app.rs           # EditingSession: adds history field, records steps at mutation seams,
│                    #   dispatches Undo/Redo commands in editing mode only
└── editor/
    ├── mod.rs       # re-exports new history module
    ├── buffer.rs    # Rope mutations (unchanged signatures)
    ├── cursor.rs    # cursor state + motions (unchanged)
    ├── input.rs     # adds Ctrl-Z → Undo, Ctrl-Y → Redo bindings
    ├── render.rs    # render projection (unchanged)
    ├── search.rs    # search state (unchanged)
    ├── status.rs    # status messages (adds history/OOM notifications)
    └── history.rs   # NEW: History { undo, redo } + EditStep + apply/undo/redo/record

tests/
├── unit/
│   ├── buffer.rs
│   ├── cursor.rs
│   ├── search.rs
│   ├── render.rs
│   └── history.rs   # NEW: EditStep reverse/forward symmetry, clear-redo, OOM drop
└── integration/
    ├── open_and_save.rs
    ├── unsaved_guards.rs
    ├── readonly_and_conflict.rs
    ├── search_and_resize.rs
    ├── enter_newline.rs
    └── undo_redo.rs  # NEW: full session-level Undo/Redo, clear-redo, session lifetime,
                     #        Unicode/newline, stack-end no-ops, OOM path
```

**Structure Decision**: Single-project layout retained (existing `src/` + `tests/`). The only new source module is `src/editor/history.rs`, owned by `editor/` alongside the other pure editor sub-modules. The only new tests are `tests/unit/history.rs` and `tests/integration/undo_redo.rs`, both registered as standalone `[[test]]` targets in `Cargo.toml` following the existing convention.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations — all constitution gates pass. Table intentionally left empty.
