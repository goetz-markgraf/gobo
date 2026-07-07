# Feature Specification: Help Dialog

**Feature Branch**: `[011-help-dialog]`

**Created**: 2026-07-07

**Status**: Draft

**Input**: User description: "Als User möchte ich bei Aufruf von Ctrl-H eine Hilfeseite angezeigt bekommen, um zu erfahren, welche Shortcuts das Programm unterstützt. Die Hilfe-Seite soll ein Dialog sein in der Art des Quit-Confirm-Dialoges. Er soll eine Liste aller im Program verwendeten Ctrl-<x> Shortcuts enthalten. Mit ENTER oder ESC kann man den Dialog schließen. Wenn der Bildschirm nicht ausreicht für alle Zeilen, soll vertikal gescrollt werden mit den Pfeiltasten."

## User Scenarios & Testing

### User Story 1 - View All Keyboard Shortcuts (Priority: P1)

A user opens a file in Gobo and wants to discover which keyboard shortcuts are available. They press Ctrl-H and see a centered dialog listing all active **Ctrl-key** shortcuts (9 entries) along with their descriptions, displayed as a flat list. The dialog shows exactly which keys do what — no guessing, no man-page hunting.

**Why this priority**: Discoverability is the foundational onboarding experience for any TUI application. Without knowing short-cuts, users cannot efficiently navigate the editor. This feature directly reduces the learning curve and increases productivity.

**Independent Test**: Can be fully tested by opening Gobo, pressing Ctrl-H, verifying the dialog appears with all shortcuts listed clearly, and confirming that Enter/Escape closes it without side effects. The value is delivered in one action: instant shortcut reference.

**Acceptance Scenarios**:

1. **Given** Gobo is open with any file loaded, **When** the user presses Ctrl-H, **Then** a Help Dialog overlay appears centered on the screen showing all keyboard shortcuts
2. **Given** the Help Dialog is displayed, **When** the user presses Enter or Escape, **Then** the dialog closes and control returns to normal editing mode
3. **Given** the Help Dialog is displayed, **When** all shortcuts for a binding have been shown on screen, **Then** additional entries are not visible until scrolled (content is truncated at the dialog boundary)

---

### User Story 2 - Scrollable Shortcut List (Priority: P1)

A user opens the Help Dialog and there are more shortcuts than fit in the visible area. All shortcuts must be accessible through vertical scrolling with arrow keys.

**Why this priority**: The number of bindings exceeds what fits on any terminal height. Without scrolling, shortcuts at the bottom would be inaccessible — defeating the purpose of showing them all. This ensures completeness is not sacrificed for compactness.

**Independent Test**: Can be fully tested by pressing Ctrl-H in a small terminal so that shortcuts overflow vertically, then using down-arrow to scroll through all entries and up-arrow to navigate back. Every shortcut must be visible at some scroll position.

**Acceptance Scenarios**:

1. **Given** the Help Dialog is displayed on a terminal with insufficient height for all entries, **When** the user presses the down-arrow key while focus is inside the dialog, **Then** the list scrolls down one line and reveals new shortcuts at the bottom
2. **Given** the scrolled view in the Help Dialog, **When** the user presses up-arrow while not already at the top of the list, **Then** the list scrolls up one line revealing previously hidden entries above
3. **Given** the user reaches the bottom of the shortcut list, **When** the down-arrow is pressed again, **Then** no scroll occurs (list stays at its current position)

---

### User Story 3 - No Mode Interference (Priority: P2)

In any editing mode (editing, search-input), Ctrl-H opens the Help Dialog. Once in the dialog, only scrolling and closing keys should work — regular typing must not be accepted as input into the dialog. The existing search state, cursor position, and document content remain unchanged while the dialog is open.

**Why this priority**: Users may invoke help mid-edit or even during a search operation. Opening help must never corrupt ongoing operations or unexpectedly change the buffer state. It should feel like a safe inspection mode.

**Independent Test**: Can be tested by opening a file, entering search mode and typing a query, pressing Ctrl-H to bring up help, verifying the search input remains intact, checking that text typed while help is active does not enter the dialog (is ignored), and closing help with Escape to verify search state is restored.

**Acceptance Scenarios**:

1. **Given** the user is in SearchInput mode with a partial query, **When** Ctrl-H is pressed, **Then** the Help Dialog opens centered on screen while the search state and query below remain unchanged
2. **Given** the Help Dialog is shown, **When** the user types printable characters inside the dialog, **Then** those characters are ignored (no effect on document or buffer)
3. **Given** the Help Dialog is open from editing or any other mode, **When** the user presses Escape to close, **Then** the session returns to the previous mode with all state restored

---

### Edge Cases

- **Very small terminal**: If the terminal height is less than the minimum dialog size (e.g., 4×8), the dialog should still be readable — use compact layout showing just titles and key labels without additional messages
- **Terminal too narrow to fit even the shortest shortcut**: Short bindings that exceed available dialog width should truncate gracefully with ellipsis, ensuring the key combo itself is always visible
- **Help invoked while a Quit or Save-Conflict prompt is already active**: The Help Dialog opens on top of the existing prompt; closing it returns focus to the underlying prompt (layered popups)
- **Undo/Redo during help open**: No undo steps are recorded, no document modifications occur — purely observational state

## Requirements

### Functional Requirements

- **FR-001**: System MUST display a Help Dialog containing all Ctrl-key shortcuts active during text editing when the user presses Ctrl-H
- **FR-002**: The dialog MUST list every Ctrl-key combination currently active during text editing (e.g. `Ctrl-S`, `Ctrl-Q`), excluding low-level keys such as Tab, Enter, Backspace, and Delete that are handled by the terminal/OS
- **FR-003**: The dialog MUST display all active Ctrl-key combinations in a flat list format (no categories or grouping headers), each showing the key binding followed by its description, consistent with the existing shortcut table style in `architecture.md`
- **FR-004**: The dialog MUST support vertical scrolling with up/down arrow keys (one line per keypress) when the total number of entries exceeds the visible area
- **FR-005**: The dialog MUST close cleanly on Enter or Escape and return control to the previous session mode without altering any state (document, cursor, selection, search query, pending prompts)
- **FR-006**: Keyboard shortcuts other than up/down arrow keys (scrolling), Enter (close), and Escape (close) while the help dialog is active MUST be silently ignored so that no underlying action executes
- **FR-007**: The Help Dialog MUST follow the same popup visual convention as other confirmation dialogs in Gobo: centered overlay with a titled block, consistent border styling, and compact/full layout variants based on terminal size — using exactly the same mechanism (PopupView via pending_prompt) as ConfirmQuit and SaveConflictPrompt
- **FR-008**: System MUST define safe handling for all modifier/key combinations currently mapped, ensuring none of them are intercepted or lost when help is displayed

### Key Entities

- **HelpDialogRow**: A single entry — `(key: String, label: String)` — representing one Ctrl-key shortcut and its purpose in a flat list format.

## Success Criteria

### Measurable Outcomes

- **SC-001**: 100% of currently mapped **Ctrl-key** shortcuts active during text editing are visible as entries in the help content, with no omissions (verifiable by cross-referencing all `map_key_event` cases for Ctrl bindings against the dialog output)
- **SC-002**: All 9 shortcut entries are immediately visible without scrolling in a standard terminal; total open-to-close cycle is ≤ 5 seconds (Enter close = ~1 second)
- **SC-003**: Opening the Help Dialog has zero side effects on document content, cursor position, selection state, or pending prompts (verified by checking session fields before and after open/close)

## Clarifications

### Session 2026-07-07

- Q: Should the help dialog support page-level scrolling or only line-by-line? → A: Line-by-line arrow keys only — minimal sufficient, no extra bindings
- Q: What happens to other key events during help dialog open? → A: Ignored entirely (Enter/Escape close; all non-scrolling keys pass through as no-op)
- Q: Should the help dialog include an internal search/filter input? → A: No — read-only scroll view sufficient for 9 entries
- Q: What shortcut scope should the help dialog display? → A: Only Ctrl-bindings active during text editing (Ctrl-N, Ctrl-S, Ctrl-Q, etc.) — excludes Tab/Enter/Backspace/Del handled at terminal level

## Assumptions

- The dialog will reuse the existing `PopupView` / `pending_prompt` mechanism established by the QuitConfirm pattern — no new widget type needed. Entries are rendered as text lines inside the popup body.
- No separate help system (man page, --help flag) is in scope — this is strictly an interactive in-dialog shortcut reference.
- Help dialog entries are static (built once from the binding table) — not dynamically generated per-session. The list does not change based on mode context.
