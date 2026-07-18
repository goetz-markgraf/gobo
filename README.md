# gobo

`gobo` is a lightweight shell text editor for one UTF-8 document per session.

## Installation

On macOS (Apple Silicon) install via Homebrew:

```bash
brew install goetz-markgraf/tap/gobo
```

This pulls the prebuilt binary from the [GitHub release](https://github.com/goetz-markgraf/gobo/releases) and makes `gobo` available in your `PATH`. Later updates:

```bash
brew upgrade gobo
```

> Note: the fully qualified name `goetz-markgraf/tap/gobo` is required because a
> different `gobo` (Gobo Eiffel) already exists in the default Homebrew tap.

## Usage

```bash
gobo <path>
```

For development:

```bash
cargo run -- <path>
# or
./target/debug/gobo <path>
```

## Keybindings

## Editing

| Key | Action |
|---|---|
| Arrow keys | Move cursor (Plain → collapses any selection) |
| **Shift**+Arrow keys | Move and extend selection |
| `Backspace` / `Delete` | Delete character(s) |
| Printable chars | Insert at cursor |
| `Tab` (no prompt active) | Insert spaces (auto-indent aware) |

## Commands

| Key | Action |
|---|---|
| `Ctrl-S` | Save |
| `Ctrl-Q` | Quit (prompts if dirty) |
| `Ctrl-F` | Search (case-insensitive, query persists across modes) |
| `Ctrl-G` | Find next match in current search query |
| `Ctrl-Z` / `Ctrl-Y` | Undo / Redo |
| `Ctrl-C` / `Ctrl-X` / `Ctrl-V` | Copy / Cut / Paste |
| `Ctrl-H` | **Show help** (always available, opens keybinding reference) |

## Prompts & Search

| Key | Action |
|---|---|
| `Enter` | Confirm active prompt choice or search match |
| `Esc` | Cancel active prompt or search |
| `Tab` / `Shift-Tab` | Move between prompt choices |

## Behavior

- Opens one file path per session
- Missing files start as empty buffers and are created on first save
- Existing non-writable files open in read-only mode
- Search is case-insensitive by default
- Save detects on-disk conflicts and prompts for reload, overwrite, or cancel
- Unsaved quit prompts for save, discard, or cancel
- Terminal resize redraws the current view

## Non-goals in this release

- No multi-document sessions
- No automatic crash recovery
- No non-UTF-8 encodings
- No plugins or scripting

## Development

```bash
cargo test
cargo run -- /tmp/example.txt
```

### Git Hooks

A pre-commit hook verifies that every `.rs` file under `tests/` is registered
as a `[[test]]` target in `Cargo.toml`.  This prevents "silent failures" where
a new test file is added but never compiled or run because it was forgotten in
the manifest.

Activate it once per clone:

```bash
git config core.hooksPath hooks
```

Files whose name contains `util` (e.g. `tests/unit/test_util_helpers.rs`) are
exempt — they are intended as shared helper modules without their own `#[test]`
functions and therefore don't need a test target.
