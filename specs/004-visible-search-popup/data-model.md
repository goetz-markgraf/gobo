# Data Model: Visible Search Popup & Find-Next

## Entity: SearchState

The persistent state for an active search session. Already exists in `editor/search.rs`; extended behavior is documented here.

### Fields
- `query: String` — the text entered by the user; may be empty during typing; persists across sessions per file
- `case_mode: CaseMode` — `Insensitive` (default) or `Sensitive`; for this feature always `Insensitive`
- `last_match_char_range: Option<(usize, usize)>` — the character range (start, end) of the last match found; updated on Enter and Ctrl+G
- `last_result: SearchResultState` — current match state: `Idle`, `MatchFound`, or `NoMatch`

### Validation Rules
- `query` is never validated for length limits; characters are accepted grapheme by grapheme.
- When the document changes (cursor moves via editing), `last_match_char_range` and `last_result` remain stable until the next search operation.
- An empty query must transition `last_result` to `Idle` or produce `NoMatch` depending on caller context.

### State Transitions
- `Idle -> MatchFound` when `find_next()` locates a match starting from cursor position (with wrap-around).
- `Idle -> NoMatch` when `find_next()` exhausts all matches (including wrap-around) without finding the query.
- `MatchFound -> Idle` when an empty query is submitted.
- `MatchFound -> MatchFound` when navigating via Ctrl+G — position updates but result stays `MatchFound`.
- `NoMatch -> NoMatch` on continued failed navigation from a no-match state.
- Any state transitions to terminal `Idle` when session exits SearchInput mode (returns to editing or cancels).

## Entity: FindNextOperation

A single execution of Ctrl+G / next-match navigation during an active search session (or after confirmation in Editing mode).

### Fields
- `start_position: usize` — the character index from which to begin searching forward
- `query: String` — the search string; must be non-empty for a real search
- `wrap_around: bool` — always `true`; when reaching end-of-document, continue from position 0

### Validation Rules
- `start_position` must be ≤ document length in characters.
- An empty `query` produces no result and displays "no matches" without moving the cursor.
- The operation never modifies document content.
- Results are applied immediately: cursor moves to match start on success.

### State Transitions (caller side)
- **From Enter confirm**: user presses Enter with non-empty query → `find_next()` called, cursor jumps to first match, session returns to Editing mode. Next Ctrl+G call uses new cursor position as `start_position`.
- **From Editing mode on Ctrl+G**: not dispatched (session in Editing mode ignores FindNext), but if dispatched, it enters SearchInput mode or reports no-match depending on whether a prior query exists. Per spec: the query persists when switching files.

## Entity: SearchResultState

The match status communicated to the UI after each search operation.

### Variants
- `Idle` — no search has been performed yet in this session (empty query submitted)
- `MatchFound` — a match was located at some position in the document
- `NoMatch` — every possible position was searched; zero instances found anywhere

### Validation Rules
- The state is read only after `find_next()` returns. The caller reads `last_result`.
- Display of "Match found" vs "No match" uses `MatchFound` vs `NoMatch`.

## Entity: SearchRenderPrompt

The visual representation of the search prompt on the bottom line.

### Fields
- `text: String` — formatted as `"Search: <query>"` where `<query>` is the current session query text (may be empty)
- `visible: bool` — true when `SessionMode.SearchInput` AND no popup overlay is active
- `foreground_color: Color` — always `Yellow` per FR-008; hardcoded in render loop

### Validation Rules
- Rendered only when `mode == SearchInput` and `pending_prompt.is_none()`.
- Occupies one screen row below the status line (which always occupies its own row).
- When `prompt_lines() >= 2`, the render constraint allocates two rows: status + search.

### State Transitions
- `Hidden -> Visible` when user presses Ctrl+F entering SearchInput mode.
- `Visible -> Hidden` when user presses Esc, Enter (with or without match), or when a popup becomes active.
- Text updates on every `InsertChar` or `Backspace` while in SearchInput mode.

## Supporting Enums

### CaseMode
- `Insensitive` — lowercase both query and haystack; default
- `Sensitive` — exact byte-level comparison

(Note: for this feature only `Insensitive` is used.)

### SearchResultState (reiterated)
- `Idle`
- `MatchFound`
- `NoMatch`

## Relationships
- One `EditingSession` owns an optional `Option<SearchState>` that is created on first `Ctrl+F`.
- The search `query` string persists across session mode transitions; it is cleared when switching files.
- Each call to `find_next()` produces one `SearchResultState` and optionally a `(usize, usize)` match range.
- Zero or one `SearchRenderPrompt` is produced per render cycle (when in SearchInput mode).
