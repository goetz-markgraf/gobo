# Data Model: Enter Key Newline Editing

## Overview

This feature does not introduce new persistent data structures. It modifies runtime editing behavior of the existing `ropey::Rope`-backed document buffer by inserting a newline character (`'\n'`) at the cursor position, which naturally creates or splits lines.

## Existing Types (Unchanged)

### `DocumentBuffer` (src/document.rs)

| Field            | Type             | Notes                         |
| ----------------- | ---------------- | ----------------------------- |
| `path`           | `PathBuf`        | File path on disk             |
| `text`           | `Rope`           | All lines separated by `\n`  |
| `access_mode`    | `AccessMode`     | Editable / ReadOnly          |
| `dirty`          | `bool`           | True if unsaved changes      |

### `CursorState` (src/editor/cursor.rs)

| Field            | Type             | Notes                          |
| ----------------- | ---------------- | ----------------------------- |
| `char_index`     | `usize`          | Codepoint offset in Rope       |
| `preferred_column`| `usize`         | Visual column for up/down    |

### `EditorCommand` (src/editor/input.rs) — **Modified**

Add variant:

```rust
pub enum EditorCommand {
    // ... existing variants ...
    Enter,       // NEW: insert newline at cursor position
}
```

## Line-Structure Invariants (Maintained)

In the Rope-based text model, lines are separated by `'\n'` characters:

- **Line 0**: Characters from index 0 to first `'\n'` (or end of text if no newlines)
- **Line N** (N > 0): Characters after the `(N-1)`th `'\n'` up to (but not including) the Nth `'\n'`

After a newline insertion at cursor position `C`:

- Line containing `C` is split into two: characters `[line_start..C]` on top line, `[C+1..next_newline]` on bottom line
- Cursor position advances to `C + 1` (the first character of the new second line)
- All other lines in the document remain at their original character offsets

## Validation Rules

1. **Edit guard**: If `DocumentBuffer::is_read_only()` returns true, newline insertion is blocked (handled in session layer).
2. **UTF-8 safety**: Cursor `char_index` is a codepoint offset; `\n` is always one codepoint / zero bytes for ASCII-positioned content. Rope handles Unicode grapheme boundaries natively.
3. **Empty document**: If `text.len_chars() == 0`, inserting `"\n"` creates one empty line.
4. **End-of-line**: If cursor is at the last character of a line (or at `len_chars()`), newline is appended after all existing content of that line.
