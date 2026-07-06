# Contract: Tab Support and Auto-Indent Interface

## Contract Type

UI / Key-Binding Contract — defines how Tab, Enter, and Backspace behave in the editor for indentation-related editing.

## Interface Boundary

The boundary is `src/editor/input.rs` for key mapping and `src/app.rs::handle_command()` for mode-aware command execution.

## Key Bindings

| Key | Editing Mode | Prompt Modes | Effect |
|---|---|---|---|
| `Tab` | Indent command | Next choice | Inserts 1 or 2 spaces so the cursor lands on the next even column |
| `Enter` | Auto-indent newline | Confirm action | Inserts newline plus copied leading spaces from the current line |
| `Backspace` | Smart outdent / normal backspace | Ignored unless prompt already handles it | Deletes 1 or 2 spaces only when the line prefix before the cursor is all spaces; otherwise keeps normal backspace behavior |
| `Shift-Tab` | No new editing behavior | Previous choice | Prompt navigation remains unchanged |

## Behavior Contract

### Tab

**Precondition**: Session is in editing mode.

**Action**:
1. If a non-empty selection exists, delete the selection first and use the selection start as the insertion point.
2. Compute the current line column as the raw char count from line start to insertion point.
3. Insert exactly 2 spaces when the column is even.
4. Insert exactly 1 space when the column is odd.
5. Clear the selection.
6. Record the whole action as one undo step.

**Postcondition**:
- No tab character is inserted into the document.
- Cursor lands on the next even column.

### Enter

**Precondition**: Session is in editing mode.

**Action**:
1. If a non-empty selection exists, delete the selection first and use the selection start as the insertion point.
2. Determine the leading-space count of the current line.
3. Insert `"\n"` followed by exactly that many spaces.
4. Keep the text originally to the right of the cursor after the inserted indentation prefix.
5. Clear the selection.
6. Record the whole action as one undo step.

**Postcondition**:
- A new line is created.
- The new line begins with the same number of leading spaces as the source line.

### Backspace

**Precondition**: Session is in editing mode.

**Action**:
1. If a non-empty selection exists, delete the selection first and use the selection start as the insertion point.
2. Inspect the text from the current line start up to the insertion point.
3. If that prefix consists only of spaces:
   - remove 2 spaces when the prefix length is even and greater than 0
   - remove 1 space when the prefix length is odd
   - remove nothing when the prefix length is 0
4. Otherwise perform the existing normal backspace behavior.
5. Clear the selection if one existed.
6. Record the whole action as one undo step.

**Postcondition**:
- Smart outdent happens only inside all-space indentation prefixes.
- Mixed content before the cursor preserves normal backspace behavior.

## Undo Contract

Each individual Tab, Enter, and Backspace action — including selection replacement plus follow-up indentation logic — produces exactly one undo step.

## Failure / Safety Contract

| Situation | Expected behavior |
|---|---|
| Read-only document | No text mutation; existing read-only warning behavior stays intact |
| Cursor at column 0 + Backspace | No negative indentation; nothing removed by smart outdent |
| Current line begins with tabs or non-space chars | Only leading spaces are copied for Enter; smart Backspace does not treat non-space prefixes as indentation |
| Active prompt | Tab and Shift-Tab keep prompt navigation semantics |
