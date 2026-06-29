# Quickstart: Fix Unsaved Popup Validation

## Prerequisites

- Rust toolchain installed (`rustc`, `cargo`)
- ANSI-capable terminal on macOS or Linux
- Repository checked out with the feature artifacts in `specs/002-fix-unsaved-popup/`

## Build and test

```bash
cargo test
cargo run -- /tmp/gobo-popup.txt
```

## Scenario 1: Visible unsaved-quit popup in a normal terminal

1. Create a sample file:
   ```bash
   printf 'alpha\n' > /tmp/gobo-popup.txt
   ```
2. Open it:
   ```bash
   cargo run -- /tmp/gobo-popup.txt
   ```
3. Type one character to make the document dirty.
4. Press `Ctrl-Q`.

**Expected outcome**:
- A visible, centered quit-confirmation popup appears before the editor exits.
- The popup shows save, discard, and cancel actions.
- **Save** is focused by default.
- The normal bottom prompt line is not used while the popup is visible.

## Scenario 2: Cancel with `Esc`

1. Trigger the unsaved-quit popup as above.
2. Press `Esc`.

**Expected outcome**:
- The popup closes.
- The editor returns to normal editing.
- The document remains dirty and the session does not quit.

## Scenario 3: Long path or status text does not hide the popup

1. Open a file with a long path, for example:
   ```bash
   mkdir -p /tmp/gobo/very/long/path/for/popup/visibility/check
   printf 'beta\n' > /tmp/gobo/very/long/path/for/popup/visibility/check/example.txt
   cargo run -- /tmp/gobo/very/long/path/for/popup/visibility/check/example.txt
   ```
2. Make the document dirty.
3. Press `Ctrl-Q`.

**Expected outcome**:
- The quit-confirmation popup remains clearly visible.
- Long path or status text does not obscure the available actions.

## Scenario 4: Compact popup in a constrained terminal

1. Open a dirty document in `gobo`.
2. Resize the terminal to a narrow and/or short layout.
3. Press `Ctrl-Q`, or resize while the popup is already open.

**Expected outcome**:
- The prompt stays visible after redraw and after live resize.
- The editor shows a compact popup variant when the terminal is too small for the full text.
- The selected action stays focused across the resize.
- Save, discard, and cancel remain usable.

## Scenario 5: Save failure from the quit popup

1. On Unix-like systems, create a file and open it:
   ```bash
   printf 'stable\n' > /tmp/gobo-save-fail.txt
   cargo run -- /tmp/gobo-save-fail.txt
   ```
2. Make the document dirty.
3. Before saving, remove write permission from another shell:
   ```bash
   chmod a-w /tmp/gobo-save-fail.txt
   ```
4. In `gobo`, press `Ctrl-Q` and confirm the default **Save** action.

**Expected outcome**:
- The editor does not quit.
- The quit popup closes.
- Unsaved changes remain in memory.
- The existing save error message UI is shown after the popup closes.
- The file on disk still contains the pre-failure content.

## Scenario 6: Automated regression coverage

Run the automated suite:

```bash
cargo test --test unsaved_guards
cargo test --test search_and_resize
cargo test --test readonly_and_conflict
```

**Expected outcome**:
- Tests cover the visible quit-confirmation flow, `Esc` cancellation, resize while prompted, constrained-layout rendering, long competing text, and save failure while quitting.

## References

- Plan: [plan.md](plan.md)
- Research: [research.md](research.md)
- Data model: [data-model.md](data-model.md)
- UI contract: [contracts/quit-confirmation-popup.md](contracts/quit-confirmation-popup.md)
