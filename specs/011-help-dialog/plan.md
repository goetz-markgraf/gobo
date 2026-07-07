# Implementation Plan: Help Dialog

**Branch**: `011-help-dialog` | **Date**: 2026-07-07 | **Spec**: [/specs/011-help-dialog/spec.md](/specs/011-help-dialog/spec.md)

**Input**: Feature specification from `/specs/[###-feature-name]/spec.md`

## Summary

Add a Help Dialog (Ctrl-H) that displays all active Ctrl-key shortcuts during text editing in a scrollable popup. Reuses the existing `PopupView`/`pending_prompt` mechanism from `ConfirmQuit`. The dialog lists keybindings as a flat table, supports line-by-line arrow-key scrolling, and closes with Enter/Escape — no state side effects.

## Technical Context

**Language/Version**: Rust 2024 edition (confirmed in Cargo.toml)

**Primary Dependencies**: `crossterm` 0.28 (raw terminal events), `ratatui` 0.29 (popup rendering), existing `pending_prompt` state machine

**Storage**: None — purely in-memory dialog rendered via ratatui overlay.

**Testing**: `cargo test` — unit tests for flat list data and scroll boundary logic; integration test covering Ctrl-H open, arrow-key scroll, Enter/Escape close, and no-side-effect guarantee.

**Target Platform**: Unix-like systems (macOS, Linux) where crossterm alternate screen + raw mode works.

**Project Type**: CLI single-document text editor (`gobo` crate)

**Performance Goals**: Dialog opens in < 10ms (O(1) — one-shot static data structure construction); scroll at any height with zero layout overhead.

**Constraints**: Must reuse the `pending_prompt` popup mechanism exactly as ConfirmQuit does — no new widget types. Modal: no key events pass through except up/down/Enter/Escape. Terminal resize while open → re-measure and re-layout automatically via existing PopupView logic.

**Scope**: flat list of all active Ctrl-command bindings (9 entries) across a full help dialog.

## Constitution Check

### I. Readability Gate
**PASSES.** Three focused changes in existing modules:
- `src/editor/input.rs`: Add one binding (`Ctrl-H → ShowHelp`) — a single line in the keymap table.
- `src/app.rs`: Add `ShowHelp` variant to `EditorCommand`, add dialog state fields, and a mode-gated handler — all adjacent to existing prompt handling code.
- `src/main.rs`: Render the popup overlay when `pending_prompt = Some(Prompt::HelpDialog)` — follows the same pattern as ConfirmQuit/SaveConflictPrompt rendering.

No new modules needed. No new abstractions. The help data is static and defined once in a helper function.

### II. Maintainability Gate
**PASSES.** Boundaries preserved:
- Input mapping stays in `editor/input.rs` (one binding added).
- Dialog state lives on `EditingSession` alongside existing prompt state (`pending_prompt`).
- All content is static (9 rows from the flat contract list) — no dynamic logic, no I/O, no persistence.
- No new dependencies; reuses `PopupView` and existing rendering infrastructure.

### III. Security Gate
**PASSES.** Zero user-data, input-validation, permission, or destructive-action risks:
- Help dialog is purely informational — no file writes, no clipboard access, no sensitive data exposure.
- Modal mode ensures no keypresses leak through to the editor, preventing accidental state changes.
- No external inputs received by the feature.

### IV. Verification Gate
**PASSES (with planned tests).** Tests defined in Implementation Plan below:
- Unit tests for flat list data construction and scroll boundary logic.
- Integration test for Ctrl-H → dialog open, arrow-key scroll, Enter/Escape → close, and mode restoration.
- Edge case test for very small terminal dimensions (compact layout).
- Manual verification: open in 20-line terminal, verify all shortcuts visible via scrolling; open mid-search to confirm state preservation.

### V. Scope Gate
**PASSES.** Help dialog is a read-only informational overlay — fits cleanly within the "focused single-document editor" scope. No external services, no plugins, no multi-file orchestration. Simplest valid design: popup over existing `pending_prompt`.

## Project Structure

### Documentation (this feature)

```text
specs/011-help-dialog/
├── plan.md              # This file
├── research.md          # Phase 0 output (no research needed — all known)
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (key contract doc)
└── tasks.md             # Phase 2 output (NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── app.rs               ← EditingSession: show_help field, PendingPrompt::HelpDialog variant
├── editor/
│   ├── input.rs         ← Ctrl-H → ShowHelp binding addition
│   └── status.rs        ← Flat list row type + build_list() in existing editor module
main.rs                    ← Draw handler: render popup when pending_prompt = Some(Prompt::HelpDialog)
tests/
├── unit/
│   └── help_dialog.rs    ← New: static content + scrolling tests
└── integration/
    └── help_dialog.rs     ← New: Ctrl-H flow, scroll, close, state preservation

```

**Structure Decision**: Minimal changes across three existing files (`app.rs`, `input.rs`, `main.rs`) plus one helper function (`help_view` into `editor/status.rs`) and two new test files. No new modules — help dialog row type lives alongside the existing popup/prompt infrastructure.

## Complexity Tracking

No constitutional violations to track. The simplest valid design (reuse PopupView + pending_prompt, static data structures, no new dependencies) was chosen and adopted.
