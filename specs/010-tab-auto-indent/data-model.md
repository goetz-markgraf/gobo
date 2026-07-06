# Data Model: Tab Support and Auto-Indent

## Context

This feature adds no persisted domain objects. The relevant data is transient editor state used to decide how many spaces to insert or remove for Tab, Enter, and Backspace.

## Entities

### Cursor Column

The zero-based number of characters to the left of the cursor within the current line.

| Field | Type | Description |
|---|---|---|
| `line_start_char` | `usize` | Char index of the current line start |
| `cursor_char` | `usize` | Current cursor char index |
| `column_chars` | `usize` | `cursor_char - line_start_char` |

**Validation / invariants**:
- `column_chars >= 0`
- Computed from raw char counts, not display width
- Used only for Tab and special Backspace width decisions

### Leading Indentation

The contiguous run of ASCII space characters at the beginning of a line.

| Field | Type | Description |
|---|---|---|
| `space_count` | `usize` | Number of leading `' '` characters |
| `line_text` | `String` | Current line content without trailing newline |

**Validation / invariants**:
- Only `' '` counts as indentation for this feature
- Tabs and other characters do not count as auto-indent
- Used by Enter to build the new line prefix

### Indent Action Plan

A transient, computed description of the edit to perform for one command.

| Field | Type | Description |
|---|---|---|
| `command` | `Tab | Enter | Backspace` | The triggering editing action |
| `replace_start` | `usize` | Start char index of the text range to replace/delete |
| `replace_end` | `usize` | End char index of the text range to replace/delete |
| `inserted_text` | `String` | Text inserted at `replace_start` after removing `replace_start..replace_end` |

**Validation / invariants**:
- `replace_start <= replace_end`
- For Tab without selection: `replace_start == replace_end == cursor`
- For Enter without selection: `inserted_text == "\n" + leading_spaces`
- For special Backspace without selection: `inserted_text == ""` and the replaced range spans 1 or 2 spaces to the left of the cursor
- For selection cases: the plan already includes both selection removal and the follow-up indent behavior so one history step is enough

### Selection Replacement Context

Transient context used when Tab, Enter, or Backspace is invoked with a non-empty selection.

| Field | Type | Description |
|---|---|---|
| `selection_start` | `usize` | Normalized selection start |
| `selection_end` | `usize` | Normalized selection end |
| `insertion_point` | `usize` | Cursor position after removing the selection; equals `selection_start` |
| `line_prefix_before_insertion` | `String` | Text from line start up to `insertion_point`, used for Tab/Backspace/Enter decisions |

**Validation / invariants**:
- Selection is cleared after the action completes
- The resulting edit is recorded as exactly one `EditStep`
- Backspace may expand `replace_start` left of `selection_start` when the post-selection line prefix is all spaces

## Relationships

```text
KeyEvent
  -> EditorCommand::{Tab, Enter, Backspace}
  -> Indent Action Plan
  -> buffer mutation (insert / replace_range)
  -> history::EditStep::{Insert, Delete, Replace}
  -> updated cursor + cleared selection
```

## State Transitions

### Tab
1. Determine insertion point (selection start or cursor)
2. Compute current line column in chars
3. Insert 2 spaces on even column, 1 space on odd column
4. Record one history step
5. Move cursor to end of inserted spaces

### Enter
1. Determine insertion point (selection start or cursor)
2. Read current line’s leading spaces
3. Insert `\n` followed by exactly that many spaces
4. Preserve text that was right of the cursor after the inserted prefix
5. Record one history step
6. Move cursor after the copied indentation

### Backspace
1. Determine insertion point (selection start or cursor after selection removal)
2. If line prefix before insertion point is all spaces, delete 1 or 2 spaces to reach previous even column
3. Otherwise use existing single-char backspace behavior
4. Record one history step
5. Move cursor to the new position
