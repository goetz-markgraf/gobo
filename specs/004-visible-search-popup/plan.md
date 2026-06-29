# Implementation Plan: Visible Search Popup

**Branch**: `[004-visible-search-popup]` | **Date**: 2026-06-29 | **Spec**: [spec.md](./spec.md)

## Summary

Fix the invisible search prompt displayed via `Search: ` on the bottom line and add a `FindNext` command bound to Ctrl+G for navigating to subsequent matches. The search currently renders as plain text through `status::search_prompt()` but is never shown because popups always occupy the bottom area. The fix involves wiring the search prompt into the render loop with proper yellow foreground styling, handling the popup-vs-search prompt mutual exclusion in `popup_view()`, adding Ctrl+G as new `EditorCommand::FindNext` variant, distinct from Tab-key-driven `NextChoice`, and extending `handle_search_command()` to support the next-match navigation from the confirmed cursor position with wrap-around behavior. The implementation stays within the existing crate — no new dependencies, no state machine changes beyond adding the command variant.

## Technical Context

**Language/Version**: Rust 2024 edition (Rust 1.85+)

**Primary Dependencies**: 
- `crossterm` 0.28 — terminal events (key input, raw mode, alternate screen)
- `ratatui` 0.29 — text rendering widgets, layout, styles (`Style`, `Color`, `Modifier`)
- `ropey` 1.6 — rope-based text buffer and character indexing
- `unicode-segmentation` 1.12 — grapheme cluster detection for search matching
- `unicode-width` 0.2 — grapheme width measurement

**Storage**: None (pure in-memory editor state)

**Testing**: Rust `#[test]` in `tests/unit/`, integration tests in `tests/integration/` driven by `cargo test`

**Target Platform**: macOS / Linux / BSD terminal emulators supporting ANSI color codes

**Project Type**: Desktop single-binary text editor (CLI application)

**Performance Goals**: Search prompt visible within one render frame (< 16ms at 60 Hz); find_next runs in O(n) over document length using rope-based iteration; wrap-around uses a single additional pass.

**Constraints**: Must stay within the existing crate structure (`src/app.rs`, `src/editor/search.rs`, `src/editor/status.rs`, `src/main.rs` draw loop). No new dependencies. Terminal must support basic ANSI color codes (already assumed in current codebase for yellow foreground styling).

**Scale/Scope**: Single document editing session; search queries up to the terminal width; documents of typical size (< 100K lines for interactive use).

## Constitution Check

- **Readability Gate**: The main modules are `app.rs` (session state + command handling), `editor/search.rs` (query matching logic), `editor/status.rs` (status line + popup + search prompt formatting), and `main.rs` (terminal render loop). Responsibilities stay cleanly separated: state machine in app, text ops in search, output in status and main. The design stays easy to read because the new code touches only these four files with minimal logic.
- **Maintainability Gate**: Boundary lines are: `editor/search.rs` owns query finding; `editor/status.rs` owns bottom-line / popup formatting; `app.rs` owns keyboard dispatch; `main.rs` owns rendering. No new abstractions needed — the change is a straightforward wiring of existing components. One small addition: the `prompt_lines()` method must return 2 when `SearchInput` mode (already done) and the render loop in `main.rs` must expand from 1 to 2 bottom constraint lines for search.
- **Security Gate**: No user-data I/O risks introduced. Search input is a plain string stored in memory; no files are read or written during search. The fail-safe behavior: if query contains UTF-8 grapheme sequences, `ropey` handles them safely. Empty queries exit silently (no panic).
- **Verification Gate**: Automated tests added to `tests/unit/search.rs`: (1) case-insensitive next-match finds the second occurrence after first confirmed position, (2) wrap-around from end-of-document returns first match, (3) no-match when search query has zero instances. Additional `editing_session` integration test in `tests/integration/` verifies keyboard-driven Ctrl+G navigation flow.
- **Scope Gate**: The feature fits entirely within the approved product scope — a focused single-document editor with basic search and navigate commands. No constitutional exception required.

## Project Structure

### Documentation (this feature)

```text
specs/004-visible-search-popup/
├── plan.md               # This file
├── research.md           # Phase 0 output
├── data-model.md         # Phase 1 output
├── quickstart.md         # Phase 1 output
├── contracts/            # Phase 1 output
└── tasks.md              # Phase 2 output (NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── app.rs                # EditingSession + handle_search_command() + new FindNext branch
├── main.rs               # draw() loop: bottom-line expansion for SearchInput mode; styled yellow search prompt
├── cli.rs                # No changes needed
├── document.rs           # No changes needed
├── lib.rs                # No changes needed
└── editor/
    ├── mod.rs            # re-exports unchanged
    ├── buffer.rs         # No changes needed
    ├── cursor.rs         # No changes needed
    ├── input.rs          # New EditorCommand::FindNext variant (distinct from NextChoice; no reuse needed)
    ├── search.rs         # find_next() wrap-around from given position, handle "no matches" state
    ├── status.rs         # popup_view() mutual exclusion with search_prompt(); styled output
    └── render.rs         # No changes needed

tests/
├── unit/
│   └── search.rs         # New tests: next-match, wrap-around, empty-query, no-match persistence
└── integration/
    └── search_and_resize.rs  # Extension: Ctrl+G navigation from confirmed cursor position
```

**Structure Decision**: No new files or modules. The four touched files are the ones that already own the relevant responsibilities (session state, input mapping, search logic, status rendering). This adheres to Constitution Principle V — simplest design with fewest moving parts.

## Complexity Tracking

> Not applicable — no constitutional violations or complex abstractions needed.

## Detailed Design

### 1. Render Loop Fix (`main.rs`)

**Problem**: The `search_prompt()` returns `Some(String)`, but the bottom-line block in `draw()` only renders when `view.bottom_line.is_some()`. However, `render_view()` in `render.rs` sets `bottom_line = None` when `popup.is_some()`. The default state of `session.pending_prompt` is `None`, yet a new session with `SessionMode::Editing` has `search: None` (no search started), so the bottom line should only appear during `SearchInput` mode.

The deeper issue is that the existing render loop creates two bottom constraints:
```rust
// Current: one status line + zero or one optional bottom
constraints: [Min(1), Length(1), Length(prompt_height)]
```

For `SessionMode::SearchInput`, `prompt_lines()` returns 2, meaning we need **two** lines below the body (one for status, one for search prompt). But currently the last constraint uses `prompt_height` as a single block. The fix: when in `SearchInput` mode, expand to `Length(prompt_height)` where `prompt_height = 2`, and render the status line + search prompt in stacked blocks within that height.

Actually — looking more carefully at `render_view()`:
- `bottom_line` is set for `SessionMode::SearchInput` (since no popup) 
- But there's actually an issue: the current code always sets bottom_line = Some(...) or None based only on popup presence

The existing flow works for SearchInput but has no visual styling. The key change: in `draw()`, when rendering `view.bottom_line`, use yellow foreground style (already done, so this is fine). The real fix needed: verify the layout constraint properly handles `prompt_height == 2`.

**Changes to `main.rs` draw function**:
```rust
// When bottom_line exists and prompt_height >= 2, show status + search stacked
let prompts_len = if view.bottom_line.is_some() { 
    session.prompt_lines() 
} else { 
    1 
};
let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Min(1),           // body
        Constraint::Length(1),        // status line (always)
        Constraint::Length(prompts_len),  // expanded for search or minimal otherwise
    ])
    .split(frame.area());

// Keep existing popup rendering as-is; it overlays on top.
if let Some(prompt_line) = view.bottom_line {
    let prompt = Paragraph::new(prompt_line)
        .style(Style::default().fg(Color::Yellow))  // <-- KEY: explicit yellow foreground
        .block(Block::default().borders(Borders::TOP));
    frame.render_widget(prompt, chunks[2]);
}
```

### 2. Status Line Fixes (`editor/status.rs`)

**Changes to `popup_view()`**: Return `None` from popup when in SearchInput mode and no pending_prompt is set. Actually the current code already checks `session.pending_prompt.as_ref()`, which returns `None` for a fresh search session. The real problem: check if the search is started.

Looking again at the flow:
1. User enters `SearchInput` mode → `session.search = Some(SearchState)` (via `begin_search`)
2. `popup_view()` checks `session.pending_prompt.as_ref()` — returns `None` 
3. Since popup is `None`, `render_view()` sets `bottom_line = Some(search_prompt(session))`

So actually the mechanism *should* work: popup returns None for search sessions, bottom_line becomes Some(...). The **real** problem may be that the render loop already handles it but the terminal has a styling issue (text transparent) OR the current code doesn't correctly check popup state vs. search mode mutual exclusion.

Looking at `search_prompt()`: it checks `session.mode != SessionMode::SearchInput` and returns None — this is correct. The result string `"Search: {}"` is plain text with no formatting passed through.

The fix in `status.rs`: ensure `popup_view()` respects the search mode priority correctly. Since both `pending_prompt` and `search` coexist, we need to verify that:
- When `mode == SearchInput` and `pending_prompt` is None → popup = None, bottom_line = Some("Search: ...")  
- When user has a pending prompt (ConfirmQuit, SaveConflict) even if mode == Edit → popup = Some(...), bottom_line = None

This already works. The visual fix should be in **render.rs**:
```rust
// Pass terminal_size to search_prompt so it can style output
pub fn search_prompt(session: &EditingSession, size: TerminalSize) -> Option<String> {
    if session.mode != SessionMode::SearchInput { return None; }
    let query = session.search.as_ref().map(|s| s.query.as_str()).unwrap_or("");
    // Currently returns just format!("Search: {}", query) — text color applied in main.rs renderer
    Some(format!("Search: {}", query))
}
```

### 3. Search Command Addition (`editor/input.rs`)

Add a new `FindNext` variant to `EditorCommand` (Ctrl+G — next match search). The existing `NextChoice` is used exclusively for popup/quickfix Tab navigation; this is a distinct command on a different key with no collision risk.

```rust
pub enum EditorCommand {
    // ... existing variants ...
    NextChoice,           // Existing: popup/quickfix Tab navigation (unchanged)
    FindNext,             // NEW: Ctrl+G - next match search from cursor position
}
```

Then in the event mapper:
```rust
(_, KeyCode::Char('g')) if key.modifiers == KeyModifiers::CONTROL => Some(EditorCommand::FindNext),
```

### 4. Search Session Handling (`app.rs`)

In `handle_search_command()`, add a new branch for the `FindNext` command:

```rust
EditorCommand::FindNext => {
    match search.query.is_empty() {
        true => {
            self.status = Some(StatusMessage::info("No match for (empty)"));
        }
        false => {
            let result = search.find_next(&self.document.text, self.cursor.char_index);
            match result {
                Some((start, end)) => {
                    self.cursor.char_index = start;
                    self.cursor.preferred_column = cursor::visual_column(&self.document.text, start);
                    self.status = Some(StatusMessage::success(format!(
                        "Match at {}..{}", start, end
                    )));
                }
                None => {
                    self.status = Some(StatusMessage::warning("No match"));
                }
            }
        }
    }
}
```

Also update `handle_editing_command()` to handle `FindNext`:
```rust
EditorCommand::FindNext => {}  // no-op in editing mode (search not active)
```

### 5. Search Logic Enhancement (`editor/search.rs`)

The existing `find_next()` already does:
1. Search forward from current position (char_index) with wrap-around to the beginning if nothing found after the cursor.

This logic is correct per the spec! The key behavioral fix needed: verify that when called during search confirmation (Enter), the cursor moves to match start and the session returns to Editing mode — then **Ctrl+G** starts a *new* find operation from the current position.

One edge case to handle: if `find_next` is called with an empty query, return `None` immediately without searching. This matches FR-007 (no matches → no cursor movement).

### 6. Rendering Styling Fix

The core issue identified in spec input: "the prompt for the search text is invisible." Looking at the existing main.rs render code:

```rust
let prompt = Paragraph::new(prompt_line)
    .style(Style::default().fg(Color::Yellow))
    .block(Block::default().borders(Borders::TOP));
```

This already has yellow foreground. The issue is likely that the `Block` borders and background colors are blending — or the search prompt path was never being taken because popups took priority. Let me check: for a fresh session with SearchInput mode, `pending_prompt` is `None`, so `popup_view` returns `None`, so `bottom_line = Some("Search: ...")`.

**The real fix**: ensure that when the search prompt renders, it explicitly has a contrasting style. The current code already uses yellow foreground; adding a background would help but adds complexity. Since the spec says "distinct foreground color... contrasts clearly against terminal background", yellow is appropriate.


**Note on FR-008 (visible yellow foreground):** Automated color contrast testing is not feasible for this editor context. Verification will be manual during implementation review; the code explicitly applies `Color::Yellow` making human verification reliable.
**Note on FR-010 (UTF-8 / multi-byte graphemes):** Unit test coverage in T002 tests CJK/emoji grapheme matching and invalid UTF-8 resilience. No separate integration test needed since find_next() logic is tested at unit level.
Let me check if there's something in `render_view()` that prevents this path:
```rust
let bottom_line = if popup.is_some() {
    None
} else {
    status::search_prompt(session)
};
```

This is correct — only hides search when popups exist. For pure SearchInput mode, both popup=None and bottom_line=Some("Search: ...").

### Summary of Changes

| File | Change |
|------|--------|
| `src/main.rs` | Render loop: ensure yellow foreground styling on search prompt; handle constraint for 2-line mode properly in SearchInput |
| `src/editor/input.rs` | Add `FindNext` variant to `EditorCommand`; add Ctrl+G mapping |
| `src/editor/search.rs` | Ensure empty query handling returns NoMatch; verify wrap-around logic in find_next() |
| `src/editor/status.rs` | Minor: pass styled output through for search_prompt |
| `src/app.rs` | Add `FindNext` branch to `handle_search_command()` and `handle_editing_command()` |

## Extension Hooks

The `after_plan` hook is registered in `.specify/extensions.yml`:

- **agent-context** (optional): `speckit.agent-context.update` — refreshes the AGENTS.md file to point to this plan. This should be run after all planning artifacts are generated.
