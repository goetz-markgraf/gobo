# Editor Command Contract: Enter / Newline Editing

## Interface

**Module**: `src/editor/input.rs`

```rust
pub enum EditorCommand {
    // ... existing variants ...
    Enter,   // Insert newline / split line at cursor
}
```

## Behavior contract

When `EditorCommand::Enter` is dispatched through `EditingSession::handle_command`:

1. **Precondition**: Session mode must be `SessionMode::Editing` or `SessionMode::SearchInput`. In any prompt mode (`ConfirmQuit`, `SaveConflictPrompt`), Enter maps to `Confirm` instead (no change in behavior).
2. **Read-only check**: If `document.is_read_only()` returns true, the edit is rejected and a status message "Read-only: edits are blocked" is shown. No state changes occur.
3. **Normal path** (editable document):
   a. Extract text at cursor from `self.cursor.char_index`.
   b. Insert the sequence `"\n"` at that position using `buffer::insert_text`.
   c. Update `self.cursor.char_index` to the end of the inserted `\n` (position 1 after insertion point, i.e., start of the new second line).
   d. Update `self.cursor.preferred_column` to 0 (cursor is at column 0 of the new line).
   e. Call `document.mark_dirty()`.
   f. Set status to `StatusMessage::info("New line inserted")` or similar.

## Edge cases

| Condition                              | Behavior                                           |
|----------------------------------------|----------------------------------------------------|
| Cursor at position 0 of empty doc     | Creates one `\n`; cursor lands at position 1       |
| Cursor at very end of document        | Appends `\n` after last char; cursor moves +2      |
| Cursor mid-line (splitting)           | Lines split at cursor; top keeps text before, bottom gets text after |
| Document has single empty line + Enter| Creates two new lines: `"\n\n"`                    |
| Read-only document                   | Blocked; no state change                           |

## Status line updates

After successful newline insertion, the editor sets a status message. The session layer handles clearing/replacing previous status messages in its standard flow (the `sync_viewport` call resets any transient status after the edit completes).

## Testing entry points

- Unit: test `EditorCommand::Enter` in isolation through `EditingSession` for each edge case above.
- Integration: verify document state before/after Enter, validate cursor position and visible line rendering (via visual inspection during manual test).
