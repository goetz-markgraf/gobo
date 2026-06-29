# Quickstart Validation Guide: Enter Key Newline Editing

## Prerequisites

- Rust toolchain installed (1.96+).
- gobo project cloned and builds: `cargo build --release` succeeds.
- A writable test file for manual interaction: `echo "Hello" > /tmp/test-newline.md`.

## Automated Validation

Run the following test suites to verify newline editing behavior:

```bash
# All existing tests must still pass
cargo test --lib
cargo test --test integration

# After implementation, new tests added in tests/integration/enter_newline.rs
cargo test --test enter_newline
```

## Manual Validation Scenarios

Each scenario below describes a manual edit flow you can reproduce with the built editor. Reference `contracts/editor-command-enter.md` for expected behavior and `data-model.md` for document-state invariants.

### Scenario 1: Enter at End of Line (Create New Blank Line)

**Setup**: Open or create a file containing `Hello`. Place cursor after `o` (end of line, position 5).

**Action**: Press **Enter**.

**Expected**:
- A new blank line appears below `Hello`.
- Cursor moves to the first column of the new blank line.
- The character `H-e-l-l-o-<newline>` is visible in the file; the document now has 2 lines.
- Status shows "New line inserted" (or equivalent info message).

**Data model invariant**: Document Rope content is `"Hello\n"` with cursor at index 6.

### Scenario 2: Enter Mid-Line (Split Line)

**Setup**: Open a file containing `Hello World`. Move cursor to position after `Hello ` (after the space, before `W`).

**Action**: Press **Enter**.

**Expected**:
- The original line is split: `Hello ` remains on line 1, `World` moves to line 2.
- Cursor is at the start of line 2 (first column, after the `\n`, before `W`).
- Document Rope content is `"Hello \nWorld"`.

**Data model invariant**: Line 0 = `"Hello "`, Line 1 = `"World"`.

### Scenario 3: Empty Line + Enter

**Setup**: Open a file with one empty line (cursor at position 0, or document just has `\n`).

**Action**: Press **Enter**.

**Expected**: Document Rope content becomes `"\n\n"` — two blank lines. Cursor is between them.

### Scenario 4: Empty Document + Enter

**Setup**: Open a non-existent file, so the editor starts with zero characters.

**Action**: Press **Enter**.

**Expected**: Document Rope content is `"\n"`. One empty line visible; cursor at position 1.

### Scenario 5: End-of-Last-Line in Multi-line Document

**Setup**: File with two lines: `First` on line 0, `Second` on line 1. Cursor at end of `Second`.

**Action**: Press **Enter**.

**Expected**: A third blank line is created below `Second`. Document Rope = `"First\nSecond\n"`.

### Scenario 6: Read-only File + Enter (Edge Case)

**Setup**: Open a read-only file. Ensure cursor is somewhere on the document.

**Action**: Press **Enter**.

**Expected**: No file changes. Status shows "Read-only: edits are blocked". Document state unchanged.

## Traceability

| Requirement | Scenario(s) |
|-------------|-------------|
| FR-001 (end-of-line newline) | 1, 5 |
| FR-002 (mid-line split) | 2 |
| FR-003 (preserving other lines) | 2, 5 |
| FR-004 (cursor repositioning) | All scenarios |
| SC-001 (multi-line documents) | 1, 2, 5 |
| SC-002 (split correctness) | 2, 3 |
| SC-003 (valid document after newline) | All |
