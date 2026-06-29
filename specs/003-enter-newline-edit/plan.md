# Implementation Plan: Enter Key Newline Editing

**Branch**: `003-enter-newline-edit` | **Date**: 2026-06-29 | **Spec**: [specs/003-enter-newline-edit/spec.md](specs/003-enter-newline-edit/spec.md)

**Input**: Feature specification from `/specs/003-enter-newline-edit/spec.md`

## Summary

Add Enter key functionality to the text editor so that pressing Enter with the cursor at the end of a line creates a new blank line below, and pressing Enter mid-line splits the current line into two — text before the cursor stays on the top line, text after moves to a new bottom line. The implementation adds an `EditorCommand::Enter` variant, maps the Enter key to it, and handles newline insertion using existing buffer utilities on the Rope-based document model.

## Technical Context

**Language/Version**: Rust 1.96.0 stable, Edition 2024

**Primary Dependencies**: Existing project modules — `src/editor/input.rs` (command enum + key mapping), `src/editor/buffer.rs` (Rope insertion utilities), `src/editor/cursor.rs` (cursor positioning), `src/app.rs` (session command dispatch); `ropey::Rope` as the underlying document buffer

**Storage**: In-memory `ropey::Rope`; lines separated by '\n' characters. No external storage changes needed for this feature.

**Testing**: `cargo test` with focus on the editing session commands and buffer-level newline insertion behavior. New integration tests in `tests/integration/` to cover end-of-line create-newline and mid-line split flows.

**Target Platform**: ANSI-capable terminal sessions on macOS and Linux (crossterm raw mode via ratatui).

**Project Type**: Standalone Rust CLI/TUI single-document text editor.

**Performance Goals**: O(1) for newline insertion at arbitrary position in the Rope; no performance regression from existing character insert/delete/arrow operations.

**Constraints**: Must work correctly with UTF-8 (grapheme-safe) cursor positions — line boundaries and cursor positions are measured by Unicode codepoint offset, not byte offset. Trailing whitespace on split lines must be preserved exactly as-is.

**Scale/Scope**: Minimal — one new command variant, one mapping change in `map_key_event`, one handler addition in `handle_editing_command`, plus tests covering the spec's acceptance scenarios and edge cases. No new modules, no changes to save/draw/render paths.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Initial Gate Assessment

- **Readability Gate**: PASS. The change is narrowly scoped to three files (`input.rs`, `app.rs`, plus tests). Naming directly reflects intent (`Enter` command, newline insertion). No new modules or abstractions are introduced.
- **Maintainability Gate**: PASS. The buffer layer already supports newline insertion via `insert_text`. Cursor position updates use existing helpers from `cursor.rs`. No new dependency or architectural boundary is created.
- **Security Gate**: PASS. The feature only inserts a newline character into an in-memory Rope. No file I/O, no external input handling, no destructive actions beyond normal editing. Safe by default — nothing can leak data or cause crashes beyond normal edit bounds.
- **Verification Gate**: PASS. Automated tests will cover: end-of-line newline (creating blank line below), mid-line split (text before/after cursor separated correctly), empty line with whitespace preservation, multi-line document where only one line splits, empty-document Enter creating initial structure, and cursor repositioning after each operation.
- **Scope Gate**: PASS. This is a single-editor behavior change fitting within the approved scope of a focused, single-binary, local-first editor. No plugin system, no network dependence, no multi-document work added.

### Post-Design Re-check

- The design keeps all changes in the existing editing pipeline (input → command dispatch → buffer/Cursor update) with no extraneous abstraction.
- Read-only documents correctly block edits via existing `is_read_only()` guards on the buffer path.
- No constitutional exception is required.
- Result: **PASS**.

## Project Structure

### Documentation (this feature)

```text
specs/003-enter-newline-edit/
├── plan.md               # This file
├── research.md           # Phase 0 output (trivial — no unknowns)
├── data-model.md         # Phase 1 output (rope structure unchanged)
├── quickstart.md         # Phase 1 output (validation guide for enter key behaviors)
├── contracts/            # Phase 1 output (editor command contract)
│    └── editor-command-enter.md
└── tasks.md              # Phase 2 output (/speckit.tasks — not created here)
```

### Source Code (repository root)

```text
src/
├── app.rs           # handle_editing_command: new match arm for Enter
├── editor/
│    ├── input.rs     # EditorCommand::Enter variant + key mapping
│    └── buffer.rs    # reuse existing insert_text / char_index helpers
tests/integration/
└── enter_newline.rs  # new integration tests for newline editing flows
```

**Structure Decision**: Reuse the existing single-crate layout. Add `EditorCommand::Enter` in `input.rs`, handle it in `app.rs::handle_editing_command` using `buffer::insert_text` with a `'\n'` string, and update cursor position post-insertion. All changes are additive (no renames that break existing tests). The new test file captures the functional requirements directly.

## Complexity Tracking

No constitution violations or complexity exceptions identified.
