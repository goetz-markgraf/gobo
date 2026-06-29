# Quickstart: Shell Text Editor Validation

## Prerequisites

- Rust toolchain installed (`rustc`, `cargo`)
- ANSI-capable terminal on macOS or Linux
- Repository checked out on branch `001-shell-text-editor`

## Build

```bash
cargo build
```

## Run

```bash
cargo run -- <path>
# or after building
./target/debug/gobo <path>
```

## Validation Scenario 1: Create a new file and save it

1. Start the editor with a path that does not exist yet:
   ```bash
   cargo run -- /tmp/gobo-new.txt
   ```
2. Enter text into the empty buffer.
3. Confirm the UI shows the document as modified.
4. Save with `Ctrl-S`.
5. Exit with `Ctrl-Q`.
   - If a prompt appears, use `Tab` / `Shift-Tab` or arrow keys to change the focused action, `Enter` to confirm, and `Esc` to cancel.
6. Verify the file content:
   ```bash
   cat /tmp/gobo-new.txt
   ```

**Expected outcome**:
- The file is created only on save.
- The saved content matches the typed text.
- Dirty state returns to clean after save.

## Validation Scenario 2: Unsaved-change protection on quit

1. Create a sample file:
   ```bash
   printf 'alpha\nbeta\n' > /tmp/gobo-quit.txt
   ```
2. Open it:
   ```bash
   cargo run -- /tmp/gobo-quit.txt
   ```
3. Make an edit.
4. Press `Ctrl-Q`.
5. Cancel the quit warning.
6. Save, then quit again.

**Expected outcome**:
- The first quit attempt does not discard changes immediately.
- A visible prompt offers save/discard/cancel behavior, with keyboard navigation between actions.
- Cancel returns to the editor with content intact.

## Validation Scenario 3: Search and no-match feedback

1. Create a sample file:
   ```bash
   printf 'First line\nsecond line\nTHIRD line\n' > /tmp/gobo-search.txt
   ```
2. Open it:
   ```bash
   cargo run -- /tmp/gobo-search.txt
   ```
3. Press `Ctrl-F` and search for `third`.
4. Repeat with a query that does not exist, such as `missing-value`.

**Expected outcome**:
- `third` matches `THIRD` because search is case-insensitive by default.
- The editor jumps to or highlights the match.
- A no-match query shows clear status feedback.

## Validation Scenario 4: Read-only file behavior

1. Create a file and remove write permission:
   ```bash
   printf 'locked\n' > /tmp/gobo-readonly.txt
   chmod a-w /tmp/gobo-readonly.txt
   ```
2. Open it:
   ```bash
   cargo run -- /tmp/gobo-readonly.txt
   ```
3. Attempt to edit and save.

**Expected outcome**:
- The editor clearly indicates read-only mode.
- Mutating actions are blocked or ignored with visible feedback.
- Save is not performed.

## Validation Scenario 5: External-change save conflict

1. Create a sample file:
   ```bash
   printf 'base\n' > /tmp/gobo-conflict.txt
   ```
2. Open it in `gobo`:
   ```bash
   cargo run -- /tmp/gobo-conflict.txt
   ```
3. While `gobo` is still open, change the file from another shell:
   ```bash
   printf 'changed elsewhere\n' > /tmp/gobo-conflict.txt
   ```
4. Return to `gobo` and attempt to save.

**Expected outcome**:
- The editor detects the on-disk change before writing.
- A prompt offers reload, overwrite, or cancel, with keyboard navigation between actions.
- Choosing cancel preserves the in-memory unsaved state.

## Validation Scenario 6: Terminal resize resilience

1. Open any multi-line file in `gobo`.
2. Resize the terminal narrower and wider while navigating.

**Expected outcome**:
- The screen redraws for the new size.
- Editing can continue without restarting the process.
- Cursor and status information remain visible and coherent.

## Validation Scenario 7: Measurable 1 MB performance protocol

1. Create a representative UTF-8 file close to 1 MB:
   ```bash
   python3 - <<'PY'
from pathlib import Path
line = "αbeta gamma delta 12345\n"
text = line * 40000
Path('/tmp/gobo-1mb.txt').write_text(text[:1_000_000], encoding='utf-8')
PY
   ```
2. Measure startup/open time:
   ```bash
   /usr/bin/time -p cargo run -- /tmp/gobo-1mb.txt
   ```
   Quit immediately with `Ctrl-Q` after the first render.
3. Re-open the file, search for `gamma`, save once with `Ctrl-S`, and note elapsed wall-clock time for search response and save completion.
4. Resize the terminal during the session and continue moving through the file.

**Expected outcome**:
- Opening the 1 MB file completes within 1 second on a local development machine.
- A representative search completes within 1 second.
- Saving the 1 MB file completes within 1 second.
- Resize does not require restarting the session.

## References

- Data model: [data-model.md](data-model.md)
- CLI contract: [contracts/cli-contract.md](contracts/cli-contract.md)
- Research decisions: [research.md](research.md)
