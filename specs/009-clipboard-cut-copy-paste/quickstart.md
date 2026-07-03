# Quickstart: Clipboard Cut, Copy & Paste Validation Guide

## Prerequisites

- `arboard` added to `[dependencies]` in `Cargo.toml`
- Build succeeds: `cargo build`
- Test suite green: `cargo test`

## Validation Scenarios

### Scenario 1: Copy with Selection

**Setup**: `gedito.txt` containing `Hello World`, start editor, mark `World` (Shift+arrows).
**Action**: Press Ctrl-C.
**Expected Status**: `"Copied 5 chars"`
**Verification**: Open a terminal and run `pbpaste` — output should be `World`. Editor text unchanged: `Hello World`. Selection still visible.
**Reference**: [Contract §Copy](contracts/cut-copy-paste.md)

### Scenario 2: Copy without Selection (Single Char)

**Setup**: `gedito.txt` containing `Hello`, cursor between `o` and ` ` (space).
**Action**: Press Ctrl-C.
**Expected Status**: `"Copied 1 chars"`
**Verification**: `pbpaste` outputs a single space (` `). Text unchanged.
**Reference**: [Contract §Copy](contracts/cut-copy-paste.md)

### Scenario 3: Cut with Selection

**Setup**: `gedito.txt` containing `Hello World`, select `World`.
**Action**: Press Ctrl-X.
**Expected Status**: `"Cut 5 chars"`
**Verification**: Editor text is now `Hello `. Undo (Ctrl-Z): `Hello World` restored. Clipboard still contains `World` — pressing Ctrl-V again (after redo) replaces the empty space with `World`.
**Reference**: [Contract §Cut](contracts/cut-copy-paste.md)

### Scenario 4: Cut without Selection (Single Char)

**Setup**: `gedito.txt` containing `Hello`, cursor at index 5 (end of file, after last char).
**Action**: Press Ctrl-X.
**Expected Status**: `"Cut 1 chars"`
**Verification**: If text is exactly `Foo\n` with cursor before `\n` at end: the newline is cut. Undo restores it. Text at end of document minus trailing newline should show correct length.

### Scenario 5: Paste into Editor

**Setup**: Copy `Test` (Ctrl-C on selected text in any editor), return to gobo, position cursor in middle of existing text.
**Action**: Press Ctrl-V.
**Expected Status**: `"Pasted 4 chars"`
**Verification**: Inserted text visible at cursor position. Selection is cleared if one was active.
**Reference**: [Contract §Paste](contracts/cut-copy-paste.md)

### Scenario 6: Paste over Selection

**Setup**: `gedito.txt` containing `Hello World`, select `World`. Clipboard contains `Earth`.
**Action**: Press Ctrl-V.
**Expected Status**: `"Pasted 5 chars"`
**Verification**: Editor text is now `Hello Earth`. Undo restores `Hello World`.
**Reference**: [Contract §Paste](contracts/cut-copy-paste.md)

### Scenario 7: Empty Clipboard Paste

**Setup**: `gedito.txt` containing `Hello`, cursor in middle. Clear clipboard (e.g., `echo -n "" | pbcopy`).
**Action**: Press Ctrl-V.
**Expected Status**: Silent — no status message, no undo entry created. Text unchanged.
**Reference**: [FR-009](specs/009-clipboard-cut-copy-paste/spec.md), [Contract §ClipboardState](data-model.md#clipboardstate-external--not-managed-by-editor)

### Scenario 8: Large Clipboard (>1 MB) — Manual

**Setup**: Run `python3 -c "print('A' * (1024*1024 + 1))" | pbcopy` to paste >1 MB text.
**Action**: Press Ctrl-V in gobo with cursor in middle of any text.
**Expected Status**: Warning `"Clipboard content too large (>1 MB)"`. Text unchanged.

### Scenario 9: Binary Clipboard Content — Manual

**Setup**: Run `printf '\x00\x01\x02\xff' | pbcopy` to set binary clipboard.
**Action**: Press Ctrl-V in gobo.
**Expected Status**: Silent no-op (same as empty clipboard). Text unchanged.
**Reference**: [FR-009](specs/009-clipboard-cut-copy-paste/spec.md)

### Scenario 10: Cut then Undo (Clipboard Persists)

**Setup**: `gedito.txt` containing `TestWord`, select `ord`. Press Ctrl-X. Clipboard = `ord`.
**Action**: Press Ctrl-Z → undo cut. Then press Ctrl-V again → paste from clipboard.
**Expected Status**: After undo, text restored to `TestWord`. After paste, same text `TestWorld` with `old` replaced by `ord` (undo of the original cut makes the text as if Cut was undone; second paste re-inserts the clip content at cursor position).
**Key Check**: The clipboard still contains `ord` after undo — it was never cleared.
**Reference**: [FR-006](specs/009-clipboard-cut-copy-paste/spec.md)

## Automated Test Execution

All scenarios above are covered by automated integration tests:

```bash
cargo test --test integration_clipboard_features
cargo test unit_clipboard
```

Focus on these test categories in `tests/integration/clipboard_features.rs`:
- `copy_with_selection` / `copy_without_selection`
- `cut_with_selection` / `cut_without_selection`
- `paste_no_selection` / `paste_over_selection`
- `empty_clipboard_is_noop`
- `large_clipboard_rejected`
- `undo_after_cut_preserves_clipboard`
- `multi_line_cut_restore`
