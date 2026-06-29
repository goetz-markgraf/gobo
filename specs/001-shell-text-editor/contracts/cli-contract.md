# CLI Contract: `gobo`

## Purpose

`gobo` starts one shell-based text editing session for exactly one target file path.

## Invocation

```bash
gobo <path>
```

## Arguments

- `path` *(required positional)*: target file path for the document to open or create on first save

## Startup Contract

### Existing writable UTF-8 file
- Opens the file content in editable mode.
- Shows the current path and clean/dirty state in the visible UI status area.

### Existing readable but non-writable UTF-8 file
- Opens the file in read-only mode.
- Clearly indicates that editing is disabled.
- Save attempts are blocked with visible feedback.

### Missing file path
- Starts an empty editable buffer bound to that target path.
- Does not create the file until the first successful save.

### Invalid startup targets
The command must fail fast with a non-zero exit status and a clear terminal-visible error when:
- the target path is a directory
- the target file cannot be read
- the target file is not valid UTF-8 text
- the path cannot be resolved or opened for another OS-level reason

## Interactive Contract

The initial release exposes the following keyboard contract:

- `Arrow keys`: move cursor
- `Ctrl-S`: save current document
- `Ctrl-Q`: quit editor; if dirty, open unsaved-changes prompt instead of exiting immediately
- `Ctrl-F`: open search input
- `Enter`: confirm the focused prompt action
- `Esc`: cancel active prompt/search input

## Required Interactive Behaviors

- The editor must maintain exactly one open document per process.
- The editor must visibly indicate whether the document is dirty.
- Search is case-insensitive by default.
- A search miss must produce clear visible feedback.
- Terminal resize events must update the visible layout without restarting the session.
- If the file changed on disk since open/save, the next save flow must prompt for `reload`, `overwrite`, or `cancel`.

## Exit Contract

- Exit code `0`: normal editor exit
- Exit code non-zero: startup or unrecoverable runtime error

## Non-Goals for This Contract

The initial contract does **not** include:
- multi-document sessions
- automatic crash recovery
- non-UTF-8 encodings
- mouse-only workflows
- plugin or scripting interfaces
