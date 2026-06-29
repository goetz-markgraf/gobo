# Feature Specification: Visible Search Popup

**Feature Branch**: `[004-visible-search-popup]`

**Created**: 2026-06-29

**Status**: Draft

**Input**: User description: "When I select CTRL-F, the prompt for the search text is invisible. Make it a popup instead on the last line. Also, make Ctrl-G find the next occurrence from where the cursor is."

## Clarifications

### Session 2026-06-29

- Q: How should "next" behavior work when no match is found and the user presses again — wrap around to the beginning of the document or stop? → A: Wrap around to the beginning of the document, continuing the search until returning to the original position (standard editor behavior).
- Q: What keybinding was originally used to trigger "find next"? The spec mentions Ctrl-G — is this a new binding or replacement? → A: Use a dedicated `FindNext` command bound to Ctrl+G, separate from the initial search. Pressing Enter in the current search still confirms and jumps to the found match.
- Q: For the popup display style, should it look similar to existing popups (e.g., unsaved-changes confirmation) or be a simpler bottom-line prompt? → A: Use an inline bottom-line prompt on the last screen line — simpler and consistent with how status lines already render at the edge of the editor view.
- Q: What should happen when the user presses Enter in search mode with an **empty query**? → A: Exit search mode silently without any message or cursor movement.
- Q: After confirming a search and returning to editing mode, should any text be visually altered for the user? → A: Nothing on the document text is visually altered. The only changes visible to the user are the cursor position updating and the bottom-line status message.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Search for text in document (Priority: P1)

A user presses Ctrl+F to enter search mode, types a query on the visible last line of the screen. There are no visual indicators on the document text. The user can confirm the search (Enter), cancel it (Esc), or continue typing the query to update results live.

**Why this priority**: Without a visible search interface, the search feature is unusable. This is the core interaction that enables all other search behaviors.

**Independent Test**: Can be fully tested by starting the editor, pressing Ctrl+F, typing a known string present in the document, and verifying the prompt text is clearly visible on the last screen line with query characters appearing as typed.

**Acceptance Scenarios**:

1. **Given** the editor is in editing mode with a document open, **When** the user presses Ctrl+F, **Then** the cursor switches to search input mode and the bottom of the screen displays `Search: ` with a visible text cursor
2. **Given** the user is in search input mode, **When** the user types characters, **Then** each character appears appended after `Search: ` on the last screen line, updated immediately
3. **Given** the user is in search input mode and has entered a query, **When** the user presses Enter, **Then** the editor finds the first match (after cursor position), places the cursor at the start of that match, displays the position briefly, and returns to editing mode
4. **Given** the user is in search input mode, **When** the user presses Esc, **Then** search mode exits without modifying cursor position or applying any search
5. **Given** the user has entered a query that does not match anything, **When** the user presses Enter, **Then** a clear message indicates no matches were found and the editor returns to editing mode

---

### User Story 2 - Find next occurrence (Priority: P2)

A user who already performed a search can navigate through multiple matches by pressing Ctrl+G. Each press jumps to the next match from the current cursor position, wrapping around the document when reaching the end.

**Why this priority**: Finding all occurrences of a term is essential for common editing workflows like "find and replace" readiness or quickly scanning for repeated patterns.

**Independent Test**: Can be tested by entering a search term with multiple matches, pressing Enter to accept the search, then repeatedly pressing Ctrl+G to verify each invocation advances the cursor to the next match in order, wrapping from end-of-document back to the first match.

**Acceptance Scenarios**:

1. **Given** a document contains multiple instances of a search term and the user has just confirmed a search (via Enter) with the cursor at the first match, **When** the user presses Ctrl+G, **Then** the cursor jumps to the next occurrence after the current position
2. **Given** the cursor is positioned at or after the last match in the document, **When** the user presses Ctrl+G, **Then** the cursor wraps around and jumps to the first match from the beginning of the document
3. **Given** a search query has no matches at all, **When** the user presses Ctrl+G, **Then** the editor displays "no matches" and does not move the cursor

---

### User Story 3 - Search with case mode awareness (Priority: P3)

The search respects the document text casing: it finds matches insensitive to case by default. The display shows the query exactly as typed, and the status line briefly indicates match position or no-match status.

**Why this priority**: Case-insensitive search is the most common user expectation for a simple editor; making this behavior explicit reduces confusion about why some matches are or aren't found.

**Independent Test**: Can be tested by searching for an uppercase word that exists only in lowercase form in the document, verifying the match is found and reported correctly.

**Acceptance Scenarios**:

1. **Given** a document contains the text "hello world" and the user searches for "HELLO", **When** the user presses Enter to confirm, **Then** the match for "hello" is found (case-insensitive)
2. **Given** the editor cannot find any match for the query, **When** the search mode exits after Enter, **Then** a status message clearly indicates no matches without suggesting incorrect behavior

---

## Functional Requirements

- **FR-001**: When the user presses Ctrl+F, the editor MUST switch to search input mode and display `Search: ` followed by the current query text on the last screen line with clear visible text (not transparent or blended into the background). The cursor is placed at the **start** of the first match; no text on the document is visually altered for the user beyond cursor positioning.
- **FR-002**: During search input mode, each typed character MUST be appended to the query string and displayed immediately after `Search: ` on the bottom line
- **FR-003**: Pressing Enter while in search input mode MUST find the first match (searching from cursor position forward, wrapping to the beginning), place the cursor at the **start** of that match, display the match position briefly (e.g., "Match at (10..15)" for ~2 seconds or until any key is pressed), then return to editing mode
- **FR-004**: Pressing Esc while in search input mode MUST cancel the search, discard any partial query, and return to editing mode without moving the cursor
- **FR-004a**: Pressing Enter while in search input mode with an **empty query** MUST exit search mode silently without moving the cursor or displaying any message
- **FR-005**: The editor MUST support a new Ctrl+G keybinding that finds the next occurrence of the current search query from the cursor position forward
- **FR-006**: Pressing Ctrl+G MUST wrap around from end-of-document to the first match when no further matches remain after the cursor.
- **FR-007**: If a search query has zero matches in the document, pressing Enter or Ctrl+G MUST display "no matches" and not move the cursor
- **FR-008**: The search input line MUST use a distinct foreground color (e.g., yellow) that contrasts clearly against the terminal background so the prompt text and typed query are always readable
- **FR-009**: The status area above the search prompt MUST remain at 1 line; the search prompt occupies 1 additional line only when in SearchInput mode (total of 2 bottom areas during search)
- **FR-010**: All search-related behaviors MUST work correctly with UTF-8 encoded text and multi-byte graphemes

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: When Ctrl+F is pressed in any editing session, the search prompt `Search: ` appears on the screen within one frame render cycle (< 16ms at standard terminal refresh rates)
- **SC-002**: The query text typed in search mode remains visible and legible (foreground color contrasts with background; no zero-opacity rendering) in every observed terminal theme tested
- **SC-003**: Users can type, confirm a search, navigate to the next match, and cancel the search — all within 5 seconds on a short document (< 50 lines) without errors or hidden UI elements
- **SC-004**: Ctrl+G correctly finds and jumps to the next match from the current cursor position in over 95% of cases across documents with at least 10 matches
- **SC-005**: Automated tests cover search visibility, character input display, next-match navigation, wrap-around behavior, no-match feedback, cancel operation, and case-insensitive matching

## Assumptions

- The document remains open during the entire search interaction; closing or switching files exits search mode and **clears the stored query** so the next file starts with an empty `Search: ` prompt
- The terminal is assumed to support basic ANSI color codes (yellow foreground)
- Search results are applied immediately as the user types in the editor (live feedback), rather than requiring a separate "refresh" action
- The popup style requested by the user refers to an on-screen inline prompt on the last line — not an overlay dialog box — consistent with how status lines already render at screen edges
