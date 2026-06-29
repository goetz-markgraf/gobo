# Implementation Plan: Fix Unsaved Popup

**Branch**: `[002-fix-unsaved-popup]` | **Date**: 2026-06-29 | **Spec**: [specs/002-fix-unsaved-popup/spec.md](specs/002-fix-unsaved-popup/spec.md)

**Input**: Feature specification from `/specs/002-fix-unsaved-popup/spec.md`

## Summary

Fix the unsaved-changes quit flow so `Ctrl-Q` always shows a clearly visible confirmation popup before quitting a dirty document. The implementation will keep the existing modal prompt state in `src/app.rs`, move prompt presentation to a centered popup overlay rendered from `src/editor/render.rs` and `src/main.rs`, provide a compact popup variant for constrained terminal sizes, preserve the default **Save** focus, and route save failures back into the existing error status UI after the prompt closes instead of exiting the editor.

## Technical Context

**Language/Version**: Rust 1.96.0 stable, Edition 2024

**Primary Dependencies**: `ratatui` for layout and popup rendering; `crossterm` for input and resize events; existing project modules in `src/app.rs`, `src/editor/render.rs`, and `src/editor/status.rs`; `tempfile` for integration tests

**Storage**: Local filesystem only; in-memory `ropey::Rope` document buffer already used by the editor

**Testing**: `cargo test` with integration tests focused on session state, derived render output, resize handling, and save-failure regression coverage

**Target Platform**: ANSI-capable terminal sessions on macOS and Linux

**Project Type**: Standalone Rust CLI/TUI application

**Performance Goals**: Popup rendering and resize redraw remain immediate during interactive editing, with no noticeable regression from the existing single-document workflow on typical files up to 1 MB

**Constraints**: Exactly one open document per session; unsaved quit prompt must render inside the visible terminal area; long status/path text must not obscure the prompt; very small terminals must fall back to a compact prompt; `Esc` must cancel the prompt; save failure from the prompt must keep the session open and preserve unsaved edits

**Scale/Scope**: Targeted bug fix to the existing unsaved-quit flow and related rendering/tests only; no new commands, no multi-document work, no plugin or background-service changes

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Initial Gate Assessment

- **Readability Gate**: PASS. The change stays concentrated in the existing session-state, render, status-formatting, and draw-path files, with popup presentation derived from prompt state instead of adding a new behavioral subsystem.
- **Maintainability Gate**: PASS. Prompt behavior remains in `src/app.rs`, while popup layout and compact/full rendering remain in the render layer. No new dependency or architecture tier is needed.
- **Security Gate**: PASS. The change strengthens destructive-action safety by making the unsaved-quit decision visible, preserving explicit user intent, and ensuring save failures do not silently quit or discard edits.
- **Verification Gate**: PASS. The plan includes automated coverage for prompt visibility state, default focus, `Esc` cancel, resize while prompted, long competing status/path content, compact rendering in constrained terminals, and save failure after choosing Save.
- **Scope Gate**: PASS. This is a contained bug fix in the current single-binary editor and does not expand product scope.

### Post-Design Re-check

- The Phase 1 design keeps prompt presentation separate from prompt behavior, which preserves readability and maintainability.
- The design preserves safe destructive-action handling by keeping the editor open on save failure and by keeping the prompt modal while active.
- The planned automated tests cover the primary regression and the specified edge cases.
- No constitutional exception is required.
- Result: **PASS**.

## Project Structure

### Documentation (this feature)

```text
specs/002-fix-unsaved-popup/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── quit-confirmation-popup.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── app.rs
├── main.rs
└── editor/
    ├── render.rs
    └── status.rs

tests/
└── integration/
    ├── unsaved_guards.rs
    ├── search_and_resize.rs
    └── readonly_and_conflict.rs
```

**Structure Decision**: Reuse the current single-crate layout. Keep prompt lifecycle and save/quit decision logic in `src/app.rs`, represent popup layout and full-text rendering in `src/editor/render.rs`, render status labels (`Save`/`Discard`/`Cancel`) from `src/editor/status.rs`, render the overlay in `src/main.rs`, and prove behavior through focused integration tests in `tests/integration/`.

## Complexity Tracking

No constitution violations or complexity exceptions identified.
