# Research: Visible Search Popup & Ctrl+G Find-Next

## Decision 1: How to fix the invisible search prompt

**Decision**: The search prompt text is rendered via `status::search_prompt()` which returns a plain `String`. In `main.rs` draw loop, this string is displayed using `Paragraph` with `.style(Style::default().fg(Color::Yellow))`. The existing code already had yellow foreground styling — the invisibility issue arises from the interplay between popup and search prompt mutual exclusion.

**Rationale**: 
- `popup_view()` in `status.rs` checks `session.pending_prompt.as_ref()`. For a fresh SearchInput session, `pending_prompt` is `None`, so popup returns `None`.
- When popup is None, `render_view()` sets `bottom_line = Some(search_prompt(session))`, which should display the search line.
- The visual bug may stem from: (a) terminal background blending with yellow on certain themes, (b) the render constraint not expanding to 2 lines for SearchInput mode, or (c) the status line consuming both the "Search: ..." text and the status message simultaneously.
- By explicitly rendering search in a separate block below the status line with `Color::Yellow` foreground, we ensure separation.

**Alternatives considered**:
1. Inline style directly inside `search_prompt()` — rejected because `search_prompt` has no access to terminal theme / color definitions; keeping styling in the renderer is cleaner.
2. Render search as a ratatui widget with explicit background — acceptable but adds complexity; yellow fg on default terminal bg (black) works fine per FR-008.

## Decision 2: Find-next keyboard binding strategy

**Decision**: Add a new `FindNext` variant to `EditorCommand` enum and map `Ctrl+G` in `map_key_event()`. This avoids clashing with the existing `NextChoice` which is already used for `Tab`/`BackTab` prompt navigation.

**Rationale**: 
- `NextChoice` on `Tab` drives cursor choice between "Save / Discard / Cancel" options in unsaved-changes and save-conflict prompts. Reusing it for search would cause ambiguous dispatch.
- The spec explicitly states Ctrl+G should be a dedicated `FindNext` command, separate from Enter.
- No new dependencies needed; the existing event loop already dispatches all `EditorCommand` variants through `handle_command()`.

**Alternatives considered**:
1. Reuse `NextChoice` for both purposes — rejected because Tab (via prompt navigation) and Ctrl+G (search next) are distinct contexts.
2. Add a mode-aware dispatch in the key handler — rejected by Constitution principle II (simpler code over cleverness).

## Decision 3: Wrap-around behavior in search.rs

**Decision**: The existing `find_next()` implementation already performs wrap-around by doing:
1. `haystack_cmp[start_byte..]` — search forward from cursor position to end
2. `.or_else(|| haystack_cmp[..start_byte].find(&needle_cmp))` — if nothing found after cursor, wrap to beginning

This matches the spec precisely: "wrap around to the beginning of the document, continuing the search until returning to the original position."

**Rationale**: The two-part search (forward then from-beginning) is the correct algorithm for a circular find. No changes needed to `search.rs` logic itself — only ensuring the caller (`app.rs`) passes cursor position correctly and handles the returned match state properly.

**Alternatives considered**:
1. Pre-compute all match positions into a Vec — rejected by Constitution V (more moving parts, more memory for typical single-use search).
2. Use `ropey::StrFind` with iterator — cleaner but requires rope-specific API changes; the current string-based approach works and is already tested.

## Decision 4: Case-insensitive search behavior (already implemented)

**Decision**: The existing `case_mode` field on `SearchState` defaults to `CaseMode::Insensitive`. The `normalize()` function lowercases both query and haystack before matching. This satisfies FR-008 (case-sensitive by default is NOT needed per the spec — "finds matches insensitive to case by default").

**Rationale**: Per User Story 3, "search respects the document text casing: it finds matches insensitive to case by default." The existing code already lowercases everything. No changes needed.

## Decision 5: Search prompt rendering in 2-line bottom area

**Decision**: When `SessionMode::SearchInput`, `prompt_lines()` returns 2, meaning the render loop allocates 2 rows for search prompts. The first row shows the status line (`status` block), and the second row shows the search prompt (`bottom_line` block). This matches the existing architecture where `status_line` is always rendered in its own constraint and `bottom_line` fills additional room.

The key render fix:
```rust
let prompts_len = if view.bottom_line.is_some() {
    session.prompt_lines()  // 2 when SearchInput, 1 otherwise
} else {
    1  // minimum for status line alone
};
constraints: [Min(1), Length(1), Length(prompts_len)]
```

**Rationale**: This preserves the existing 3-constraint layout but expands the search block from `Length(1)` to `Length(2)`. The status line stays at 1 row always. No restructuring needed.

**Alternatives considered**:
1. Collapse status line into bottom search area — rejected by FR-009 (status MUST remain at 1 line, search prompt occupies 1 additional line only).
2. Use ratatui layouts with `split()` for fine-grained positioning — rejects simplicity.

## Decision 6: No persistent highlights during editing

**Rationale**: Per the spec, "No persistent highlights at all." The current architecture has no match-highlighting code at all. Search results are applied immediately as the cursor moves (during Enter/Confirm or Ctrl+G) and disappear when the session returns to Editing mode because there's no persistent highlight state. This is already compliant — nothing changes.
