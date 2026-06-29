# gobo

`gobo` is a lightweight shell text editor for one UTF-8 document per session.

## Usage

```bash
cargo run -- <path>
# or
./target/debug/gobo <path>
```

## Keybindings

- `Arrow keys`: move cursor
- `Ctrl-S`: save
- `Ctrl-Q`: quit
- `Ctrl-F`: search
- `Enter`: confirm active prompt
- `Esc`: cancel active prompt or search
- `Tab` / `Shift-Tab`: move between prompt choices

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
