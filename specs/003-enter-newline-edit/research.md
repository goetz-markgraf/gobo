# Research: Enter Key Newline Editing

## Unknowns Resolved

### 1. How should newline insertion interact with the Rope-based text storage?

**Decision**: Use `ropey::Rope::insert(char_index, "\n")` to insert a newline character at the cursor position. The Rope natively treats '\n' as a line separator, so all existing line-tracking helpers (`line_of_char`, `line_start_char`, `line_len_chars`) work correctly without modification.

**Rationale**: `ropey::Rope` is already the document storage backend. Its `insert` method handles Unicode grapheme boundaries automatically. Line separation by '\n' characters means everything after the inserted newline naturally becomes a new line for all downstream operations (rendering, cursor movement, save).

**Alternatives considered**:
- Adding a dedicated `buffer::insert_newline` wrapper — unnecessary since `insert_text` suffices.
- Using an internal `\r\n` representation — no value over plain `\n` on any platform; the Rope handles display correctly.

### 2. Where does the cursor land after newline insertion?

**Decision**: Cursor moves to position 0 of the newly created line — i.e., at the inserted '\n' character (or equivalently, right after the newline char). The `visual_column` will be 0 since no characters precede it on the new line.

**Rationale**: This matches standard text editor behavior and matches what the spec requires: "a new blank line is created below and the cursor moves to the beginning of that new line."

### 3. How to handle Enter in an empty document (no lines)?

**Decision**: When the Rope has zero length, inserting '\n' creates exactly one line (the Rope treats "\n" as a single empty line). Cursor position is 0 at the start of this line. This satisfies "An initial empty line structure is established and cursor moves to allow content entry on a new first line."

**Rationale**: `ropey::Rope` handles this naturally — `"\n".len_lines()` returns 1, confirming exactly one line exists after the newline insertion.

### 4. Trailing whitespace preservation during split?

**Decision**: No special handling needed. Inserting '\n' at cursor position preserves everything before and after the insertion point as-is. If the cursor is in the middle of a line with trailing spaces like `"Hello   "`, splitting produces `"Hello   "` (first line) and `""` (second line). The whitespace before the cursor stays on line 0, anything after goes to line 1.

**Rationale**: Rope's insert does not trim — this is direct character insertion. Matches spec requirement: "Trailing whitespace is preserved exactly as-is on the top line."

## Dependencies Resolved

### ropey::Rope best practices

- `insert(char, &str)` inserts at a codepoint (char) index
- Line boundaries defined by `'\n'` characters — Rope tracks line indices automatically
- `len_lines()` returns count of lines for a zero-length Rope: 0 (but after inserting '\n', it's 1)

### crossterm key events for Enter

- In raw mode, Enter (`KeyCode::Enter`) emits as a standard key event, not modifiable.
- Currently mapped to `EditorCommand::Confirm` in `input.rs`; will be changed to `EditorCommand::Enter`.
