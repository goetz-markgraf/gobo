# Implementation Plan: Tab Support and Auto-Indent

**Branch**: `010-tab-auto-indent` | **Date**: 2026-07-06 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/010-tab-auto-indent/spec.md`

## Summary

Add real Tab-key indentation and indentation-aware Enter/Backspace behavior. In editing mode, Tab inserts only spaces so the cursor lands on the next even column, Enter inserts a newline plus the current line’s leading spaces, and Backspace removes 1 or 2 spaces only when the text from line start to cursor is all spaces. Selection-aware Tab, Enter, and Backspace must first replace the selection and then apply their normal logic at the insertion point, all as one undo step.

## Technical Context

**Language/Version**: Rust 2024 edition

**Primary Dependencies**: Existing crates only: `crossterm`, `ratatui`, `ropey`, `clap`, `unicode-segmentation`, `unicode-width`, `arboard`

**Storage**: In-memory `ropey::Rope` document state only; no new persisted storage

**Testing**: `cargo test` with standalone integration/unit test targets declared in `Cargo.toml`

**Target Platform**: Local terminal editor on macOS and Linux

**Project Type**: Single-binary CLI desktop text editor

**Performance Goals**: Each Tab/Enter/Backspace action completes within one event-loop tick; no new work beyond local rope edits on a single document

**Constraints**: Never insert literal tab characters for this feature; prompt navigation with Tab/Shift-Tab must keep working; only leading spaces count as indentation; each Tab/Enter/Backspace action must remain exactly one undo step

**Scale/Scope**: One feature touching key mapping, edit orchestration, and focused tests; no new dependencies; one small helper module is acceptable if it keeps indentation logic isolated

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Readability Gate**: Keep physical key mapping in `src/editor/input.rs`, pure indentation calculations in `src/editor/indent.rs`, and stateful rope/history/session orchestration in `src/app.rs`. This prevents the three behaviors from being hidden inside one large match arm or spread across unrelated files.
- **Maintainability Gate**: Boundaries remain clear: `input.rs` maps keys to commands, `indent.rs` computes widths/ranges/inserted strings, `app.rs` performs mode-aware command handling and records history, `buffer.rs` stays a rope primitive layer, and tests stay split between pure helper coverage and `EditingSession` integration flows. No new dependency is introduced.
- **Security Gate**: This feature touches only local in-memory editing. Relevant fail-safe behavior: read-only documents still block mutation, Backspace at column 0 removes nothing, mixed-content prefixes do not trigger special outdent, and prompt mode must not accidentally mutate the document when Tab is pressed.
- **Verification Gate**: Automated coverage must include Tab on even/odd columns, Enter with and without leading spaces, Enter splitting lines, Backspace on all-space prefixes vs mixed prefixes, selection replacement for all three commands, prompt navigation preservation, read-only blocking, and one-step undo for every command path.
- **Scope Gate**: The feature fits the approved single-document local editor scope. No network, background services, plugin hooks, or persistent state changes are introduced.

## Project Structure

### Documentation (this feature)

```text
specs/010-tab-auto-indent/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
└── contracts/
    └── tab-auto-indent.md
```

### Source Code (repository root)

```text
src/
├── app.rs                  MODIFIED: editing-mode dispatch and atomic indent edits
├── editor/
│   ├── input.rs            MODIFIED: dedicated Tab command mapping
│   ├── indent.rs           NEW: pure indentation calculations
│   └── mod.rs              MODIFIED: export indent module

tests/
├── integration/
│   ├── enter_newline.rs    MODIFIED: auto-indent expectations for Enter
│   └── tab_auto_indent.rs  NEW: Tab / Backspace / selection / prompt flows
└── unit/
    └── indent.rs           NEW: pure indent helper boundary cases

Cargo.toml                  MODIFIED: register new standalone test targets
```

**Structure Decision**: Add one small helper module, `src/editor/indent.rs`, because the feature introduces several pure indentation rules that should not live directly inside `src/app.rs`. Keep all rope mutation and history recording in `src/app.rs` so the helper stays side-effect free. Extend the existing standalone test-target pattern in `Cargo.toml`.

## Phase 0: Research Summary

Research resolved the implementation shape without leaving open clarifications:
- Use raw char-count columns, not visual-width columns, for indentation rules
- Introduce `EditorCommand::Tab` instead of overloading `NextChoice`
- Build Enter/selection-aware edits as one atomic inserted/replaced text operation
- Keep smart Backspace restricted to all-space prefixes only
- Cover the feature with one new integration file plus one new unit test file

See [research.md](research.md).

## Phase 1: Design Summary

- The transient data model is centered on `Cursor Column`, `Leading Indentation`, and an `Indent Action Plan` that describes the single edit to apply.
- The user-facing interface contract is documented as a TUI key-binding contract in [`contracts/tab-auto-indent.md`](contracts/tab-auto-indent.md).
- Validation scenarios and test commands are documented in [quickstart.md](quickstart.md).
- Agent context is updated to point to this plan.

## Post-Design Constitution Re-check

- **Readability Gate**: Still passes. The design keeps indentation math in a dedicated helper instead of hiding it inside `handle_editing_command`.
- **Maintainability Gate**: Still passes. No new dependency; one new module has one reason to change: indentation rules.
- **Security Gate**: Still passes. Failure paths remain local and safe: read-only is blocked, prompt mode does not mutate text, and smart Backspace is narrowly scoped.
- **Verification Gate**: Passes in design. The planned unit and integration tests cover the feature behaviors and the relevant edge cases named in the spec.
- **Scope Gate**: Still passes. The work remains within a focused editor behavior change.

## Complexity Tracking

No constitutional violations identified. The only structural addition is `src/editor/indent.rs`, justified to keep the new indentation rules readable and independently testable.
