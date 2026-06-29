# Implementation Plan: Shell Text Editor

**Branch**: `[001-shell-text-editor]` | **Date**: 2026-06-29 | **Spec**: [specs/001-shell-text-editor/spec.md](specs/001-shell-text-editor/spec.md)

**Input**: Feature specification from `/specs/001-shell-text-editor/spec.md`

## Summary

Build `gobo` as a standalone Rust terminal text editor for one UTF-8 document per session. The implementation will use a single-binary CLI/TUI architecture: `clap` for startup argument parsing, `ratatui` + `crossterm` for terminal rendering and event handling, and `ropey` for an efficient UTF-8 text buffer that stays responsive on typical files up to 1 MB. The design centers on safe editing workflows: explicit unsaved-change guards, read-only mode for non-writable files, terminal-resize support, case-insensitive search by default, and conflict prompts when the underlying file changes on disk.

## Technical Context

**Language/Version**: Rust 1.96.0 stable, Edition 2024

**Primary Dependencies**: `clap` for CLI parsing; `ratatui` with the Crossterm backend for terminal UI; `crossterm` for raw mode, screen management, and input/resize events; `ropey` for the UTF-8 text buffer; `unicode-segmentation` and `unicode-width` for cursor/display correctness

**Storage**: Local filesystem only; in-memory rope buffer for the open document; no database

**Testing**: `cargo test` with unit tests for editor state transitions and integration tests that drive the app core with synthesized events and temporary files

**Target Platform**: ANSI-capable terminal sessions on macOS and Linux first; packaged as a single local CLI binary

**Project Type**: Standalone CLI/TUI application

**Performance Goals**: Startup feels immediate for typical files; interactive editing/search remains responsive without perceptible lag on UTF-8 files up to 1 MB; local open/save operations for 1 MB files complete within about 1 second

**Constraints**: Exactly one open document per session; UTF-8 text files only; no automatic crash recovery in the initial release; must warn before destructive actions and before saving over externally changed files; must support terminal resize; must clearly indicate read-only mode when a file cannot be written

**Scale/Scope**: Single-user local workflow; one process, one document, one viewport; initial release targets notes/config/source files up to 1 MB and excludes tabs, splits, plugins, and collaborative features

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Initial Gate Assessment

- **Readability Gate**: PASS. The planned module split keeps CLI, app lifecycle, document state, rendering, cursor logic, search, and status feedback in focused files with clear responsibilities.
- **Maintainability Gate**: PASS. Terminal I/O and rendering stay isolated from editor state so behavior can be tested without a live terminal and future changes stay localized.
- **Security Gate**: PASS. The feature scope includes UTF-8 validation, read-only handling for non-writable files, unsaved-change guards, and explicit conflict prompts before overwriting externally changed files.
- **Verification Gate**: PASS. The plan already requires unit tests for core editor logic and integration tests for open/save, unsaved-change protection, read-only/conflict handling, and search/resize behavior.
- **Scope Gate**: PASS. The design remains a single-binary, single-document local editor and does not introduce plugins, network services, or other out-of-scope complexity.

### Post-Design Re-check

- The Phase 1 design still satisfies all five constitutional gates above.
- No constitutional exception is required.
- Result: **PASS**.

## Project Structure

### Documentation (this feature)

```text
specs/001-shell-text-editor/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── cli-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
Cargo.toml
src/
├── main.rs
├── cli.rs
├── app.rs
├── document.rs
└── editor/
    ├── mod.rs
    ├── buffer.rs
    ├── cursor.rs
    ├── input.rs
    ├── render.rs
    ├── search.rs
    └── status.rs

tests/
├── integration/
│   ├── open_and_save.rs
│   ├── unsaved_guards.rs
│   ├── readonly_and_conflict.rs
│   └── search_and_resize.rs
└── unit/
    ├── buffer.rs
    ├── cursor.rs
    └── search.rs
```

**Structure Decision**: Use a single Cargo binary crate. Keep terminal I/O and rendering isolated from editor state so unit tests can exercise buffer/cursor/search logic without a live terminal, while integration tests validate end-to-end flows against temporary files.

## Complexity Tracking

No constitution violations or complexity exceptions identified.
