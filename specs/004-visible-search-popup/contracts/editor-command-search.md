# Editor Command Contract: Search / Find-Next

## Interface

**Module**: `src/editor/input.rs`

```rust
pub enum EditorCommand {
    // ... existing variants ...
    Search,     // Ctrl+F — enter search input mode
    FindNext,   // NEW: Ctrl+G — find next match from cursor position forward
}

pub fn map_key_event(key: KeyEvent) -> Option<EditorCommand> {
    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('f')) => Some(EditorCommand::Search),
        (KeyModifiers::CONTROL, KeyCode::Char('g')) => Some(EditorCommand::FindNext),
        // ... rest of mapper unchanged
    }
}
```

## Behavior contract for `EditorCommand::Search` (Ctrl+F)

When `EditorCommand::Search` is dispatched through `EditingSession::handle_command` in `SessionMode::Editing`:

1. **Effect**: Transition session mode to `SessionMode::SearchInput`. Initialize or reuse the existing `search: Option<SearchState>`. Display status message `"Search started"`.
2. **After entering SearchInput**: The editor enters text-input mode on the bottom line. Each character typed appends to `search.query`. Backspace removes last character. Both Enter and Esc exit search mode.

## Behavior contract for `EditorCommand::FindNext` (Ctrl+G)

When `EditorCommand::FindNext` is dispatched through `EditingSession::handle_command` in `SessionMode::SearchInput`:

1. **Precondition**: A non-empty query must exist in `search.query`. An empty query produces "no match" without moving the cursor.
2. **Operation**: Call `search.find_next()` starting from the current cursor character position (`self.cursor.char_index`). The find logic searches forward to end-of-document, then wraps to beginning if nothing found.
3. **Match found**: Move cursor to the start of the matched text. Display a success status message with match range (e.g., `"Match at 10..15"`). Cursor remains in `SearchInput` mode — the session does NOT revert to Editing.
4. **No match**: Display a warning status `"No match"`. Cursor position is unchanged. Session stays in `SearchInput` mode.

When `EditorCommand::FindNext` is dispatched through `EditingSession::handle_command` in `SessionMode::Editing`:

1. **Effect**: No operation. The command is a no-op because search input mode is not active. This prevents accidental matches when the user presses Ctrl+G outside of search context. (Per discussion: if there's a prior query stored, it could enter SearchInput — but this spec requires FindNext only in SearchInput.)

## Edge cases

| Condition | Behavior |
|---|---|
| Empty query + Enter | Exit SearchInput silently; no message displayed; cursor unchanged |
| Empty query + Ctrl+G | Display "no match"; do not move cursor |
| Single-char document, search for that char | Match found at position 0..1 |
| Search term longer than document | Never matches → NoMatch |
| Query with UTF-8 multi-byte characters | Matches correctly via ropey grapheme-aware indexing |
| Document wrapped around zero times (cursor past last potential match) | Wrap to beginning of document, find first occurrence if any |

## Status line updates

- Enter confirm with match: `StatusMessage::success("Match found at <start>..<end>")`
- Enter confirm without match: `StatusMessage::warning("No match for <query>")`
- Ctrl+G with match: `StatusMessage::success("Match at <start>..<end>")`
- Ctrl+G without match: `StatusMessage::warning("No match")`

## Testing entry points

- **Unit**: Add tests to `tests/unit/search.rs` validating `find_next()` behavior for wrap-around, case-insensitive match, and no-match cases. Add unit-level tests for `EditingSession::handle_search_command()` with FindNext dispatch.
- **Integration**: Add scenario in `tests/integration/search_and_resize.rs` verifying the full flow: Ctrl+F → type query → Enter → verify match → Ctrl+G multiple times → verify wrap-around → Esc to cancel.
- **Visual**: Verify search prompt text is visibly yellow on last screen line; no transparent or blended rendering.
