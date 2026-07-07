# Quickstart: Help Dialog Validation

**Generated**: 2026-07-07

## Prerequisites

- `gobo` built from the `011-help-dialog` branch: `cargo build --release`
- A terminal at least 44 columns × 8 rows (full mode)
- A test file for editing context: `touch /tmp/gobo-test.txt`

## Validation Scenarios

### S1: Help Dialog Opens (Primary Flow)

**Setup**: Open any file in Gobo.

```bash
./target/release/gobo /tmp/gobo-test.txt
```

**Steps**:
1. Press **Ctrl-H** while in editing mode.
2. Verify the centered dialog appears with title "Keyboard Shortcuts".
3. Verify all 9 shortcut entries from `{@link contracts/key-bindings.md}` are listed in the flat dialog body, matching the contract order.

**Expected**: Dialog visible; every active binding shown without truncation (in a ≥44×8 terminal).

### S2: Arrow-Key Scrolling

**Setup**: Help dialog is open (from S1), in a terminal tall enough to require scrolling (use a short terminal or many shortcuts).

**Steps**:
1. Press **↓** — list scrolls down one line, revealing new entries at bottom.
2. Continue pressing ↓ until the bottom of the list is reached.
3. Press ↓ again — no scroll occurs (stays at bottom).
4. Press **↑** — list scrolls up one line per press.
5. Continue ↑ until top is reached.
6. Press ↑ again — no scroll above top.

**Expected**: Smooth one-line-per-press scrolling with correct boundary behavior. See `{@link data-model.md#ScrollState}` for invariants.

### S3: Close Dialog (Enter)

**Setup**: Help dialog is open.

**Steps**:
1. Press **Enter**.

**Expected**: Dialog closes immediately; cursor and document state are identical to before opening. Return to editing mode.

### S4: Close Dialog (Escape)

**Setup**: Help dialog is open.

**Steps**:
1. Press **Escape**.

**Expected**: Same as S3 — dialog closes, full state restoration.

### S5: Open During Search Mode

**Setup**: Open a file, press Ctrl-F to enter search mode, type a partial query.

**Steps**:
1. Press **Ctrl-H**.
2. Verify help dialog opens centered on screen.
3. The search input below the line remains intact (unchanged).
4. Close with Escape.
5. Verify search state is fully restored — cursor in search bar, query still present.

**Expected**: Search mode preserved; help does not clear or corrupt pending operations. See spec FR-003.

### S6: No Mode Interference During Help

**Setup**: Help dialog is open (from any scenario above).

**Steps**:
1. Type arbitrary printable characters while the dialog is visible.

**Expected**: Characters are silently ignored — no insertion into document, no buffer modification, no cursor movement, no command execution. See spec FR-006.

### S7: Small Terminal (Compact Layout)

**Setup**: Open Gobo in a terminal narrower than 44 columns or shorter than 8 rows.

**Steps**:
1. Press **Ctrl-H**.

**Expected**: Dialog still renders correctly in compact mode — title on one line, no footer shown, content maximized for available space. Shortcut keys are always visible even if truncated.

### S8: Help Over Existing Prompt

**Setup**: Trigger the Quit confirmation prompt (press Ctrl-Q on a dirty file).

**Steps**:
1. While the quit prompt is visible, press **Ctrl-H**.
2. Verify help dialog opens on top of/layered over the quit prompt.
3. Close help with Escape or Enter.
4. Verify the quit prompt returns to focus with its state intact.

**Expected**: Layered popup behavior works; underlying prompt unharmed. See spec Edge Cases section.

## Automated Test Coverage

Run the following to verify implementation:

```bash
# Unit tests for help dialog data and scrolling logic
cargo test --test unit_help_dialog

# Integration test for Ctrl-H flow, scroll, close, state preservation
cargo test --test integration_help_dialog
```
