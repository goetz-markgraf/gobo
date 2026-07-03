# Implementation Plan: Fix Trailing Newline Cursor Position

**Branch**: `008-fix-trailing-newline-cursor` | **Date**: 2026-07-03 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/008-fix-trailing-newline-cursor/spec.md`

## Summary

The editor reports the wrong *visual* line for the cursor when the cursor sits
exactly at the end of a document that ends with one or more `\n` characters.
Symptom: opening `"abc\n"` and moving the cursor to the document end leaves it
drawn at the end of the `abc` line; only an inserted character "snaps" it to the
correct empty trailing line. Root cause is in `buffer::line_of_char`: for
`char_index == text.len_chars()` it probes `char_index - 1`, and ropey's
`char_to_line('\n')` returns the line *containing* the newline rather than the
following empty line, so the cursor's visual line is one too low.

Fix: in `buffer::line_of_char`, when the cursor is at `len_chars()` AND the
final character of the document is a `\n`, return `probe_line + 1` (i.e. the
empty trailing line). This single change cascades through `cursor::visual_column`
and `render::render_view` (both derive the line via `line_of_char`), so the
visual cursor, its column, and the effective insert position all become
consistent. The logical `char_index` is **not** changed, so inserts already
land at the correct position; only the display/line mapping is corrected. No
change to persistence (out-of-scope per clarification).

## Technical Context

**Language/Version**: Rust 2024 edition (`Cargo.toml`, edition = "2024").

**Primary Dependencies**: ropey 1.6 (text model, char indices), ratatui 0.29,
crossterm 0.28, unicode-segmentation / unicode-width (grapheme-aware columns).

**Storage**: single file, atomic save (temp+rename); unchanged by this feature.

**Testing**: `cargo test` — unit tests in `tests/unit/` plus integration tests
under `tests/integration/`. New coverage is unit-level (pure functions, no
terminal).

**Target Platform**: local terminal editor (macOS/Linux raw mode).

**Project Type**: single-binary local-first CLI editor (desktop-app/CLI).

**Performance Goals**: none new; cursor mapping is O(line lookup) already.

**Constraints**: no new dependencies; no change to persisted content; fix must
not regress the already-correct cases (no trailing newline, cursor before the
trailing newline, empty document).

**Scale/Scope**: single document, no network, no plugins.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Readability Gate**: The fix lives entirely in `buffer::line_of_char`, the
  single function responsible for char→line mapping. The added branch is a few
  lines with a clarifying comment explaining the ropey `\n`/end-of-document
  quirk. No new module, no new abstraction; reviewers understand the bug from
  the comment + failing test. ✓
- **Maintainability Gate**: Boundaries already exist: `buffer.rs` owns char/line
  mapping, `cursor.rs` consumes it, `render.rs` consumes it for display. The
  change touches only the mapping producer, preserving every boundary. No new
  dependency or shared state. ✓
- **Security Gate**: No user-data, input-validation, permission, or
  destructive-action surface changes. The change is read-only mapping logic;
  it cannot cause data loss, and persistence is explicitly out-of-scope
  (clarification 2026-07-03). The trailing `\n` is preserved on save
  unchanged. ✓
- **Verification Gate**: Automated tests are the primary deliverable:
  - Reproducing failing test for `"abc\n"` cursor-at-end (the bug).
  - Edge cases: `"abc\n\n"`, `"abc\n\n\n"`, `"\n"`, `""`, `"abc"` (no newline),
    cursor *before* the trailing newline (no regression).
  - Visual-column consistency (`visual_column`) and end-to-end `render_view`
    cursor `(x,y)` for the trailing-newline case.
  These are pure-function unit tests — repeatable, no manual UX required. ✓
- **Scope Gate**: Stays within the approved single-document local editor scope.
  No new services, network, plugins, or persistence changes. ✓

## Project Structure

### Documentation (this feature)

```text
specs/008-fix-trailing-newline-cursor/
├── plan.md              # This file
├── research.md           # Phase 0 output
├── data-model.md         # Phase 1 output
├── quickstart.md         # Phase 1 output
├── contracts/
│   └── cursor-line-mapping.md
└── tasks.md              # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
src/editor/
├── buffer.rs   # FIX: line_of_char end-of-doc trailing-newline mapping
├── cursor.rs   # unchanged (consumes line_of_char via visual_column)
└── render.rs   # unchanged (consumes line_of_char for cursor_y)

tests/unit/
├── buffer.rs   # ADD: line_of_char trailing-newline edge cases
└── cursor.rs   # ADD: visual_column / move_right at doc-end consistency
```

**Structure Decision**: Single-project layout (Option 1). The change is a
localized fix in `src/editor/buffer.rs` plus unit tests; no new files or
modules.

## Complexity Tracking

> No constitution violations. Table left empty.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| — | — | — |
