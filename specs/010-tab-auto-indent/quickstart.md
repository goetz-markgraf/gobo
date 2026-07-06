# Quickstart: Tab Support and Auto-Indent Validation Guide

## Prerequisites

- Rust toolchain installed
- Project builds successfully: `cargo build`
- Feature tests registered in `Cargo.toml`

## Automated Validation

Run the focused tests for this feature:

```bash
cargo test --test integration_tab_auto_indent
cargo test --test integration_enter_newline
cargo test --test integration_undo_redo
cargo test --test integration_text_selection
cargo test --test unit_indent
```

## Manual / End-to-End Scenarios

### Scenario 1: Tab from an even column

**Setup**: Open a document with the cursor at column 0.
**Action**: Press `Tab`.
**Expected**: Exactly two spaces are inserted and the cursor moves to column 2.
**Reference**: [Contract §Tab](contracts/tab-auto-indent.md#tab)

### Scenario 2: Tab from an odd column

**Setup**: Cursor is at column 1 in the current line.
**Action**: Press `Tab`.
**Expected**: Exactly one space is inserted and the cursor moves to column 2.
**Reference**: [Contract §Tab](contracts/tab-auto-indent.md#tab)

### Scenario 3: Enter copies leading spaces

**Setup**: Current line is `"    hello"` and the cursor is at the end of the line.
**Action**: Press `Enter`.
**Expected**: A new line is inserted starting with four spaces.
**Reference**: [Contract §Enter](contracts/tab-auto-indent.md#enter)

### Scenario 4: Enter splits a line and keeps right-side content

**Setup**: Current line is `"  hello"` and the cursor is between `he` and `llo`.
**Action**: Press `Enter`.
**Expected**: The document becomes `"  he\n  llo"`.
**Reference**: [Contract §Enter](contracts/tab-auto-indent.md#enter)

### Scenario 5: Smart Backspace inside indentation

**Setup**: Current line begins with three spaces and the cursor is after those spaces.
**Action**: Press `Backspace`.
**Expected**: Exactly one space is removed and the cursor lands on column 2.
**Reference**: [Contract §Backspace](contracts/tab-auto-indent.md#backspace)

### Scenario 6: Normal Backspace after mixed content

**Setup**: Current line is `"a  bc"` and the cursor is after the two spaces.
**Action**: Press `Backspace`.
**Expected**: Normal backspace deletes one character to the left; smart outdent does not trigger.
**Reference**: [Contract §Backspace](contracts/tab-auto-indent.md#backspace)

### Scenario 7: Selection replacement with Tab

**Setup**: Select a non-empty range in the current line.
**Action**: Press `Tab`.
**Expected**: The selection is removed, 1 or 2 spaces are inserted at the selection start according to the resulting column, the selection clears, and one undo restores the original text.
**Reference**: [Data Model §Selection Replacement Context](data-model.md#selection-replacement-context)

### Scenario 8: Selection replacement with Enter

**Setup**: Select a non-empty range in an indented line.
**Action**: Press `Enter`.
**Expected**: The selection is removed, a newline plus copied indentation is inserted, and one undo restores the original text.
**Reference**: [Contract §Enter](contracts/tab-auto-indent.md#enter)

### Scenario 9: Selection replacement with Backspace

**Setup**: Select a non-empty range with only spaces left of the selection start on the same line.
**Action**: Press `Backspace`.
**Expected**: The selection is removed and smart outdent logic then removes the additional 1 or 2 spaces needed to reach the previous even column, all in one undo step.
**Reference**: [Contract §Backspace](contracts/tab-auto-indent.md#backspace)

### Scenario 10: Prompt navigation remains intact

**Setup**: Open a quit/save conflict prompt.
**Action**: Press `Tab` and `Shift-Tab`.
**Expected**: Focus moves between prompt choices; no spaces are inserted into the document.
**Reference**: [Contract §Key Bindings](contracts/tab-auto-indent.md#key-bindings)
