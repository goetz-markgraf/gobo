# gobo – Architecture Overview

## Tech Stack

| Layer | Crate | Purpose |
|---|---|---|
| Language | Rust 2024 edition | `Cargo.toml` → `[package] edition = "2024"` |
| CLI parsing | `clap` (derive) | Parses single positional `path: PathBuf`; exposed as `gobo::cli::parse_args()` |
| Terminal I/O | `crossterm` 0.28 | Raw-mode events (`Key`, `Resize`), alternate screen, cursor control |
| TUI rendering | `ratatui` 0.29 | Layout → widgets (`Paragraph`, `Block`). Drawn each loop iteration |
| Text buffer | `ropey` 1.6 | All text is `Rope`. Every function works in **character indices** (never byte offsets) |
| Unicode helpers | `unicode-segmentation`, `unicode-width` | Grapheme-aware visual column calculations |
| Temp-files (tests) | `tempfile` 3.20 | Integration tests use `tempdir()` for isolated file systems |

## Module Map

```
src/
├── main.rs          binary entry: parse args → open session → raw-mode event loop → draw
├── lib.rs           re-exports 4 public modules
├── cli.rs           clap struct + parse_args() helper
├── document.rs      file I/O, access-mode detection, atomic save (temp+rename), conflict snapshots
├── app.rs           EditingSession – top-level state machine and command dispatch
└── editor/          pure editor sub-modules (stateless functions where possible)
    ├── mod.rs
    ├── buffer.rs    Rope mutations: insert / remove / delete / replace_range; line helpers
    ├── cursor.rs    CursorState + motion functions + viewport clamping + Selection type + MoveSelect* selection motions
    ├── input.rs     KeyEvent → EditorCommand mapping (key bindings table; incl. Shift+arrows → MoveSelect*)
    ├── render.rs    EditingSession → RenderView; viewport slicing, column clipping, selection highlight spans
    ├── search.rs    SearchState: case-insensitive find_next() with wrap-around
    ├── status.rs    StatusMessage → footer message, popup view construction
    └── history.rs   Undo/Redo: History { undo, redo } + EditStep (Insert/Delete/Replace) + record/undo/redo
```

The `EditingSession` owns a `history: History` field (in-memory, session-bound,
never persisted) and a `selection: Option<Selection>` field (in-memory,
session-bound, never persisted; FR-001/FR-013). Ctrl-Z dispatches
`EditorCommand::Undo`, Ctrl-Y dispatches `EditorCommand::Redo`; both are wired
in `handle_editing_command` and silently ignored in search/prompt modes. Every
text mutation (`insert_text`, `backspace`, `delete`, and the selection-aware
`replace_or_insert` / `delete_or_backspace` → `replace_selection`) records one
`EditStep` via `history.record`, which clears the redo stack.

### Data Flow (Per Event Loop Tick)

1. `crossterm::event::poll()` → `Event::Key` or `Event::Resize`
2. `map_key_event(key)` → `Option<EditorCommand>` (or `Resize(TerminalSize)`)
3. `session.handle_command(command)` → mutates session state
4. `session.render_view()` → `RenderView` (body lines as `BodyLine { text, highlights }` with optional `HighlightSpan`s, footer line with filename + status message, optional popup/bottom-line)
5. `draw(terminal, &view)` → ratatui frame render + cursor position

## State Machine (`SessionMode`)

```
Editing ──Ctrl-Q(dirty)──► ConfirmQuit ──Save─────────► Exiting
  │                        │           │
  │                        │          Overwrite
  │                        ├──────────┘
  │                        └──Cancel──► Editing
  │
  Ctrl-F                 SearchInput ──Enter(match)──► Editing (cursor jumps)
      │                       │       │
      │                       ├──────┘ (no match → status msg, back to Editing)
      │                       └Cancel──► Editing (search state persists)
  │
  Ctrl-S + conflict    SaveConflictPrompt ──Reload/Overwrite/Cancel─→ Editing
```

**Key rule:** `handle_command()` first checks prompts, then dispatches by current `mode`.
Prompts always take priority; most commands in prompt/search modes are silently ignored.

## Popup Precedence

When `pending_prompt` is `Some`, the render layer sets `bottom_line = None` –
the popup overlay (drawn as separate ratatui widget) takes full visual precedence.

Popup variant switches automatically: terminal < 44×8 → `Compact`, else `Full`.

## Text Model Details

- All positions are **character indices** (`usize`), not byte offsets or grapheme offsets
- Visual columns = sum of `unicode_width` of all graphemes from line start
- Moving up/down preserves `preferred_column` (visual), re-resolved per target line
- `buffer::line_content()` strips trailing `\n` and `\r\n`; `rope_to_string()` reassembles with newlines
- `buffer::line_of_char()` maps a cursor at `len_chars()` of a document ending in `\n` to the empty trailing line (ropey `char_to_line` returns the line *containing* a trailing `\n`; spec 008)

## Document I/O

- **Atomic save:** write to `<path>.tmp` → `std::fs::rename()` into place
- **Conflict detection:** on `save()`, compare current disk snapshot (mtime + size + content hash) against last-saved snapshot
- **Read-only detection:** flags at open time may be unreliable on non-unix; current code checks `permissions().readonly()` only
- Missing file → opens empty buffer, created on first save

## Testing Convention

### Unit Tests (`tests/unit/`)

Test pure functions directly with constructed inputs; no `EditingSession` involved.

| File | What it covers | Key types imported |
|---|---|---|
| `buffer.rs` | Insert / remove / delete / replace / line helpers | `Rope`, buffer module funcs |
| `cursor.rs` | Motion (left/right/up/down), viewport clamp, visual column math | `CursorState`, `ViewportState` |
| `search.rs` | Case-insensitive match, edge cases (empty query, wrap-around, multi-byte) | `SearchState` |
| `history.rs` | `EditStep` reverse/forward symmetry, clear-redo, OOM eviction, empty-stack no-ops | `History`, `EditStep`, `Rope` |

### Integration Tests (`tests/integration/`)

Always drive real behavior through `EditingSession::open()` + `handle_command()`.
Each file groups tests by topic with a shared helper function at the top:

| File | Focus | Helper pattern |
|---|---|---|
| `open_and_save.rs` | File open/edit/save/create lifecycle | inline `tempdir()` per test |
| `unsaved_guards.rs` | Quit popup flow, clean vs dirty exit | `dirty_session()` helper |
| `readonly_and_conflict.rs` | Read-only guards + external-change conflict prompt | inline `tempdir()`, `#[cfg(unix)]` for chmod |
| `search_and_resize.rs` | Full search flow (type→confirm), resize while prompted | `dirty_session_with_size()` |
| `enter_newline.rs` | Enter key text insertion at every edge position | `make_session()` + `assert_enter_text()` |
| `undo_redo.rs` | Full Undo/Redo, clear-redo-on-edit, session lifetime, mode gating, OOM, save | `session_with_seed()` / `session_with_capped_history()` |

### How to Write Tests

1. **Unit:** import from `gobo::editor::*`, construct inputs manually, assert return values / mutated state
2. **Integration:** create temp dir → write seed file → `EditingSession::open()` → `handle_command()` sequence → assert session fields and/or disk content
3. **No mocking.** Real files on real temp dirs. No fakes for `Rope` or `crossterm`
4. **Standalone bins:** Each test file is a separate `[[test]]` target in `Cargo.toml`. Run with
   `cargo test --test unit_buffer` or `cargo test --test integration_search_and_resize`.
5. **No shared harness:** No `tests/mod.rs` or common fixture module – each integration file is self-contained

## Key Design Patterns

- **Stateless functions where possible:** `buffer/` functions take `&mut Rope` + indices, return new index; no hidden state
- **Stateful structs only for compound concerns:** `EditingSession`, `SearchState`, `DiskSnapshot`
- **Render = pure projection:** `render_view()` derives `RenderView` from session snapshot – no side effects
- **Input mapping isolated in one place:** `editor/input.rs` is the single source of truth for key bindings
  (including Ctrl-Z → Undo, Ctrl-Y → Redo, placed before the printable-char catch-all so the `!CONTROL`
  guard prevents aliasing to `InsertChar('z')/'y')`)
- **Render split across layers:** `editor/render.rs` produces a data struct (`RenderView` with `BodyLine`/`HighlightSpan`); actual widget rendering with layout constraints and `Modifier::REVERSED` highlight styling lives in `main.rs::draw()` (spec 007) – two separate concerns

## Commands (`EditorCommand`)

19 variants. Dispatched via match on `(KeyModifiers, KeyCode)`. Unmapped keys → `None` (ignored).

| Key | Command |
|---|---|
| Arrows | `MoveLeft/Right/Up/Down` |
| Shift+Arrows | `MoveSelectLeft/Right/Up/Down` (seed/extend selection; spec 007) |
| Ctrl-S | `Save` |
| Ctrl-S | `Save` |
| Ctrl-Q | `Quit` |
| Ctrl-F | `Search` |
| Ctrl-G | `FindNext` |
| Enter | `Enter` (newline in edit mode; confirm in prompt/search) |
| Esc | `Cancel` |
| Tab / Shift-Tab | `NextChoice` / `PreviousChoice` |
| Backspace / Delete | `Backspace` / `Delete` |
| Printable (no mod) | `InsertChar(char)` |

## Search State Persistence

- `SearchState` lives on `EditingSession.search` (behind `Option`). It is **not cleared** when leaving
  `SearchInput` mode – the query persists so `Ctrl-G` (`FindNext`) continues working from editing mode.
- Cancelling with `Esc` returns to editing but keeps the query intact.
- Confirming with `Enter` jumps cursor to first match and switches to editing mode, keeping search alive.
- Internally `find_next()` is O(n×m) – collects all matches into a `Vec<char>` before selection. Fine for
  single-file use case; not suitable for multi-MB documents.

## Spec-Driven Development

Feature specs live under `specs/<N>-<name>/`. Each spec contains:
`spec.md`, `plan.md`, `tasks.md`, `data-model.md`, contracts, and checklists.
The active spec path is referenced in `.pi/agent/AGENTS.md` via the SPECKIT comment.
