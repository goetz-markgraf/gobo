# Implementation Plan: Clipboard Cut, Copy & Paste

**Branch**: `009-clipboard-cut-copy-paste` | **Date**: 2026-07-03 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/009-clipboard-cut-copy-paste/spec.md`

## Summary

Add clipboard support with `Ctrl-C` (copy), `Ctrl-X` (cut), and `Ctrl-V` (paste) using the
system clipboard (via `arboard`). Copy/cut without selection operates on the single grapheme
cluster under/after the cursor, mirroring normal editor behavior. All three commands are
single-step undo operations. The editor delegates all clipboard state to the OS — no internal
clipboard copy is maintained. A 1 MB hard limit rejects oversized pastes with a non-blocking
status message.

## Technical Context

**Language/Version**: Rust 2024 edition

**Primary Dependencies**: Add `arboard = "3"` for cross-platform system clipboard (macOS/i18n/Linux). Existing deps: `crossterm`, `ratatui`, `ropey`, `clap`, `unicode-segmentation`, `unicode-width`.

**Storage**: System clipboard only; no persistent storage. Clipboard content is managed entirely by the OS via `arboard`'s `DisplayClipboard`.

**Testing**: Integration tests driving real `arboard` clipboard through temp sessions (mocking impossible). Unit tests for size-limit logic and edge cases on clipboard data parsing. 1 MB enforcement tested with synthetic large inputs.

**Target Platform**: macOS (primary), Linux (Secondary, X11/Wayland via arboard's backend selection). Windows not in scope for this iteration but `arboard` supports it.

**Project Type**: CLI desktop application (single-file text editor)

**Performance Goals**: Clipboard round-trip < 50ms for text up to 1 MB; paste of large text must render within one frame without blocking the event loop.

**Constraints**: Must use only the OS-provided clipboard API — no internal clipboard copy cache. `arboard` is the lowest-friction cross-platform path; alternative (platform-specific FFI) would violate Constitution V.

**Scale/Scope**: Single feature: three key bindings (Ctrl-C/X/V) + one new dependency (`arboard`). No new modules beyond `editor/clipboard.rs`.

## Constitution Check

- **Readability Gate**: One new module (`editor/clipboard.rs`), two new `EditorCommand` variants per operation (already counted — Copy, Cut, Paste = 3 commands). Each function is pure or has side effects clearly scoped. Clipboard wiring in `input.rs` and the dispatch in `app.rs`, no hidden state.

- **Maintainability Gate**: Boundary lines:
  - `editor/clipboard.rs` — clipboard I/O (read/write OS display), size limiting, text-only filtering. No dependency on editor state.
  - `app.rs` — dispatches commands to `clipboard::write_clipboard()` / `clipboard::read_clipboard()`. Never stores clipboard data internally beyond the local variable in the handler.
  - `editor/input.rs` — key binding table extension.
  - One new dependency (`arboard`) justified: solves cross-platform clipboard where raw FFI would be more work and harder to maintain.

- **Security Gate**:
  - User-data risk: clipboard may contain arbitrary data from other processes. Mitigation: `arboard::get_text()` returns `Option<String>`; only `Some(text)` proceeds, all other cases (None/binary) → no-op with no status change.
  - Input validation: size limit enforced at read and write — rejects >1 MB silently via status message.
  - Destructive action safeguard: Cut already requires explicit Ctrl-X intent; paste never clobbers without a pre-existing selection.
  - Dependency risk: `arboard` is well-maintained (3.x), no known security issues per audit history. Declared in constitution II as acceptable since the concrete problem (clipboard IPC) is harder to solve safely in project code.

- **Verification Gate**: Automated tests covering: copy with selection, copy without selection (single char), cut with selection, cut without selection, paste without selection, paste over selection, undo after all three operations, empty clipboard paste, oversized paste (>1 MB), binary data paste (no-op). All via integration tests through `handle_command()`.

- **Scope Gate**: Fits squarely within single-document editor scope. No network, no background service, no plugin system. One dependency; one new module; additions to existing files bounded to ~50 lines each. No constitutional exception needed.

## Project Structure

### Documentation (this feature)

```text
specs/009-clipboard-cut-copy-paste/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── contracts/           # Phase 1 output
```

### Source Code (repository root)

```text
src/
├── editor/
│   ├── clipboard.rs     NEW: clipboard read/write, size limit, text filters
│   ├── input.rs         MODIFIED: add Ctrl-C/X/V mappings
│   └── app.rs           MODIFIED: handle Copy/Cut/Paste commands

tests/
├── integration/
│   └── clipboard_features.rs  NEW: copy/cut/paste integration tests
```

**Structure Decision**: Keep within existing `editor/` sub-directory for all clipboard logic.
New crate `arboard` in `[dependencies]`. New integration test file following the established pattern (standalone binary, inline helpers). No new top-level modules — the feature is narrow enough that splitting further would increase coupling via extra re-exports.

## Complexity Tracking

No constitutional violations identified. All requirements fit within existing architecture
patterns. The single new dependency (`arboard`) is the only complexity addition and is
justified under Constitution II (solves a concrete external I/O problem) and V (keeps
scope narrow).
