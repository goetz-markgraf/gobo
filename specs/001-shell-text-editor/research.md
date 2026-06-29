# Research: Shell Text Editor

## Decision 1: Target stable Rust with Edition 2024
- **Decision**: Implement `gobo` in stable Rust using the locally available toolchain (`rustc 1.96.0`) and Edition 2024.
- **Rationale**: The feature explicitly requires Rust and a single binary. Stable Rust keeps distribution simple, avoids nightly-only maintenance, and fits a terminal application with no platform-specific GUI dependencies.
- **Alternatives considered**:
  - **Nightly Rust**: Rejected because the editor does not need unstable features and nightly would make adoption and reproducible builds harder.
  - **Older pinned Rust baseline**: Rejected for now because the repository is greenfield and the local toolchain is already current stable.

## Decision 2: Use `ratatui` + `crossterm` for the terminal UI loop
- **Decision**: Build the interface with `ratatui` for layout/rendering and `crossterm` for raw mode, alternate screen management, keyboard input, and resize events.
- **Rationale**: Current Ratatui docs recommend the main `ratatui` crate for applications using the Crossterm backend, and Crossterm provides the event primitives needed for key handling and terminal resize support. This pairing gives enough structure for a lightweight editor while preserving control over behavior that should differ from other editors.
- **Alternatives considered**:
  - **Raw `crossterm` only**: Rejected because full-screen layout, status bars, and redraw management would require more boilerplate with no user-facing benefit.
  - **Higher-level editor widget crates**: Rejected because the product goal is “simple, but different in detail,” so the editor should own its editing model instead of being constrained by a prebuilt text-area abstraction.

## Decision 3: Use `ropey` as the document buffer
- **Decision**: Store the open document in a `ropey::Rope`.
- **Rationale**: Ropey is explicitly designed as a UTF-8 editor backing buffer and supports insert/remove operations and line/character conversions efficiently. That matches the requirement to stay responsive while editing common files up to 1 MB and simplifies line-based viewport rendering.
- **Alternatives considered**:
  - **Single `String` buffer**: Rejected because repeated inserts/deletes in the middle of a document become awkward and less scalable as the file grows.
  - **`Vec<String>` by line**: Rejected because cross-line edits, cursor math, and save serialization become more error-prone.

## Decision 4: Keep the public CLI minimal with `clap`
- **Decision**: Expose `gobo` as `gobo <path>` with a single required positional file path parsed via `clap` derive.
- **Rationale**: The feature spec defines exactly one target path per session. `clap` provides a polished CLI, help output, and validation with minimal code, while keeping the external surface intentionally small.
- **Alternatives considered**:
  - **Manual `std::env::args()` parsing**: Rejected because it adds avoidable boilerplate and makes error/help behavior less consistent.
  - **A richer option set in v1**: Rejected because it conflicts with the simplicity goal and the spec only requires direct shell invocation for one file.

## Decision 5: Separate editor core from terminal adapter for testing
- **Decision**: Design the application around a pure editor state/core that accepts high-level input events and produces state updates plus render-ready view data; keep terminal setup and actual drawing in a thin adapter layer.
- **Rationale**: Full PTY-driven TUI tests are brittle. A split architecture supports deterministic unit and integration tests for editing, search, unsaved warnings, read-only behavior, save conflicts, and resize handling without depending on an interactive terminal in every test.
- **Alternatives considered**:
  - **Only subprocess/PTY end-to-end tests**: Rejected because they are slower, flakier, and harder to diagnose for a greenfield editor.
  - **No integration boundary at all**: Rejected because the feature has many session-level behaviors that must be validated beyond isolated helper functions.

## Decision 6: Use Unicode-aware editing boundaries at the cursor/render layer
- **Decision**: Represent internal editing positions using Ropey character indices, and use `unicode-segmentation` plus `unicode-width` when translating cursor movement and rendering across visible grapheme boundaries and terminal cell widths.
- **Rationale**: The initial release is UTF-8 only. Character-safe and display-aware movement reduces broken cursor behavior around multi-byte and multi-cell characters while keeping the implementation manageable.
- **Alternatives considered**:
  - **Byte-based indexing**: Rejected because it risks invalid UTF-8 boundaries and poor editing behavior.
  - **Full advanced Unicode editor semantics everywhere**: Rejected for v1 because the first release only needs robust basic editing, not the most exhaustive possible text model.

## Decision 7: Detect external file changes with a saved disk snapshot checked before write
- **Decision**: Capture file metadata when opening or saving a document and re-check the on-disk file before saving. If the on-disk state changed, enter an explicit conflict prompt with `reload`, `overwrite`, or `cancel` outcomes.
- **Rationale**: The spec requires warning users before saving over a file that changed on disk. For a 1 MB local-file scope, a save-time disk check is simple, predictable, and cheap enough.
- **Alternatives considered**:
  - **Continuous file watching**: Rejected because it adds complexity and platform-specific behavior not required for the initial release.
  - **Blind overwrite on save**: Rejected because it violates FR-014.

## Decision 8: Define a conventional first-release keymap
- **Decision**: Use conventional keyboard controls in the initial design: arrow keys for movement, `Ctrl-S` to save, `Ctrl-Q` to quit, `Ctrl-F` to search, `Enter` to confirm prompts, and `Esc` to cancel prompts.
- **Rationale**: The spec requires keyboard-only operation but does not prescribe exact bindings. Conventional defaults minimize onboarding cost and support the success criteria for first-use editing.
- **Alternatives considered**:
  - **Modal/Vim-like bindings only**: Rejected because they raise the learning curve for a general-purpose lightweight editor.
  - **Function-key-heavy controls**: Rejected because terminal portability is less predictable across environments.
