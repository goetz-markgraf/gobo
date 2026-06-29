# Render Contract: Search Prompt Visibility

## Interface

**Module**: `src/editor/status.rs` — search prompt formatter
**Module**: `src/main.rs` — render loop for bottom-line rendering

## Behavior contract for `status::search_prompt()`

### Signature (conceptual)

```rust
pub fn search_prompt(session: &EditingSession, size: TerminalSize) -> Option<String> {
    // Returns Some("Search: <query>") when in SearchInput mode without active popup
    // Returns None otherwise
}
```

### Precondition

- Session mode is `SessionMode::SearchInput`.
- No pending popup overlay (i.e., `session.pending_prompt` is `None`).
- Terminal area has at least 2 rows available for the bottom portion.

### Output contract

Returns a formatted string: `"Search: <query>"` where `<query>` is the current content of `search.query` as a Rust string. If query is empty, returns `"Search: "`.

The text must be rendered in the render loop with explicit `Color::Yellow` foreground and no background override affecting visibility.

### Mutual exclusion contract

When `session.pending_prompt.is_some()`, `popup_view()` returns `Some(...)`. In this case, the render loop MUST suppress the search prompt even if mode is `SearchInput`. Popups take priority.

## Render loop contract (main.rs)

### Bottom-line rendering

```rust
// Layout constraints
let prompts_len = if view.bottom_line.is_some() {
    session.prompt_lines()   // 2 for SearchInput, else 1
} else {
    1   // minimum: status line alone occupies this constraint
};

let chunks = Layout::default()
     .direction(Direction::Vertical)
     .constraints([
         Constraint::Min(1),       // editable body
         Constraint::Length(1),    // status line
         Constraint::Length(prompts_len),   // search prompt (if any)
     ])
     .split(frame.area());

// Render bottom-line block when present
if let Some(prompt_line) = view.bottom_line {
    let prompt = Paragraph::new(prompt_line)
         .style(Style::default().fg(Color::Yellow))
         .block(Block::default().borders(Borders::TOP));
    frame.render_widget(prompt, chunks[2]);
}
```

### Constraints

1. `prompt_lines()` must return 2 when `SessionMode::SearchInput` (already done) and 1 otherwise.
2. The status-line block occupies `chunks[1]` with fixed height 1 always.
3. The search prompt block occupies `chunks[2]` which is either `Length(1)` or `Length(2)`.
4. Yellow foreground (`Color::Yellow`) applies to the entire search prompt block — no color blending with status line background (which is `Color::Black` bg, `Color::White` fg).
5. When popup is active (`view.popup.is_some()`), `chunks[2]` is still present but empty; the search prompt path is not taken because `bottom_line` is `None`.

## Edge cases for rendering

| Condition | Behavior |
|---|---|
| Terminal resized to height 1 during search | Bottom constraint clips; status line visible, search text clipped — acceptable per spec (edge case) |
| Status message persists while search prompt shows | Status line in `chunks[1]` is separate from `chunks[2]` search block; both visible stacked |
| Terminal width < 8 | Search prompt string `"Search: "` may exceed width; ratatui `Paragraph::new()` will truncate/word-wrap automatically |
| Query contains wide-width Unicode (CJK, emoji) | Graphemes render per their displayed width; no overflow panic — ratatui handles truncation |
