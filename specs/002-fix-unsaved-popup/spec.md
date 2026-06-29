# Feature Specification: Fix Unsaved Popup

**Feature Branch**: `[002-fix-unsaved-popup]`

**Created**: 2026-06-29

**Status**: Draft

**Input**: User description: "Das Popup zum Speichern, das kommt, wenn man CTRL-Q drückt und ungespeicherte Änderungen hat, ist nicht sichtbar."

## Clarifications

### Session 2026-06-29

- Q: Which action should be focused by default when the unsaved-quit prompt opens? → A: Save
- Q: How should the editor behave if the terminal is too small for the full quit prompt? → A: Show a compact confirmation prompt with abbreviated actions.
- Q: How should `Esc` behave while the quit-confirmation prompt is open? → A: `Esc` cancels the quit prompt and returns to editing.
- Q: How should the editor handle long status or file-path text that competes with the quit prompt area? → A: Prompt as a popup.
- Q: What should happen if saving fails after the user chooses Save from the unsaved-quit prompt? → A: Stay in the editor, keep unsaved changes, and show the save error.
- Q: Where should the save error be shown after Save fails from the unsaved-quit prompt? → A: Show the existing save error message UI after the prompt closes.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - See the unsaved-changes quit prompt (Priority: P1)

When a user has unsaved changes and presses `Ctrl-Q`, the editor shows a clearly visible confirmation prompt so the user can decide whether to save, discard, or cancel.

**Why this priority**: This protects user data. If the confirmation prompt is not visible, users cannot make a safe decision before quitting.

**Independent Test**: Can be fully tested by editing a document, pressing `Ctrl-Q`, and verifying that the confirmation prompt is visible in the terminal and lists the available actions.

**Acceptance Scenarios**:

1. **Given** an open document with unsaved changes, **When** the user presses `Ctrl-Q`, **Then** a visible quit-confirmation popup appears in the terminal before the editor exits.
2. **Given** the quit-confirmation popup is visible, **When** the user chooses save, discard, or cancel, **Then** the editor performs the chosen action and the popup no longer blocks the screen.
3. **Given** the quit-confirmation popup has just opened, **Then** the Save action is focused by default.
4. **Given** the quit-confirmation popup is visible, **When** the user presses `Esc`, **Then** the popup closes and the editor returns to normal editing without quitting.
5. **Given** the user chooses Save from the quit-confirmation popup and the save operation fails, **Then** the editor remains open, preserves the unsaved changes, closes the quit-confirmation popup, and shows the existing save error message UI instead of quitting.

---

### User Story 2 - Use the prompt in constrained terminal layouts (Priority: P2)

When the terminal is narrow, short, or contains long status text, the user can still see and use the quit-confirmation popup.

**Why this priority**: Prompt visibility must remain reliable in realistic shell conditions, not only in ideal terminal sizes.

**Independent Test**: Can be fully tested by triggering the quit-confirmation popup in multiple terminal sizes and with long file names or status text, then confirming that the popup remains visible and actionable.

**Acceptance Scenarios**:

1. **Given** a document with unsaved changes in a narrow or short terminal, **When** the user presses `Ctrl-Q`, **Then** the popup remains visible enough to show that a decision is required and what actions are available.
2. **Given** the popup is already visible, **When** the terminal is resized, **Then** the popup stays visible after redraw and still allows the user to complete the quit decision.
3. **Given** long status or file-path text would otherwise compete with the popup area, **When** the user presses `Ctrl-Q`, **Then** the editor shows the quit-confirmation popup as an overlay so the available actions remain clearly visible. (see also: [Visibility Contract](../contracts/quit-confirmation-popup.md#visibility-contract))
4. **Given** the terminal is too small to fit the full popup text, **When** the user presses `Ctrl-Q`, **Then** the editor shows a compact confirmation popup with abbreviated actions that still allows save, discard, or cancel.

### Edge Cases

**Note:** Most edge cases below are already captured by the functional requirements and acceptance scenarios above.
Pointers to the relevant FR/US for each:

- Too-small terminal → compact popup: **FR-008**, US2S4
- Long status/path text competes → overlay: **FR-007**, US2S3
- Terminal resize while popup open → redraw handling: **FR-006**, US2S2
- Esc cancels prompt → normal editing: **FR-009**, US1S4
- Save fails → editor stays open, shows error UI: **FR-009b**, US1S5
- No unsaved changes → immediate quit: **FR-010**
- Transient UI takes precedence → popup: **FR-009**

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST show a visible quit-confirmation popup whenever a user attempts to quit a document that has unsaved changes.
- **FR-002**: The quit-confirmation popup MUST be visible within the active terminal view and MUST not depend on hidden or off-screen content.
- **FR-003**: The quit-confirmation popup MUST present the available actions to save, discard, or cancel.
- **FR-004**: The quit-confirmation popup MUST indicate which action is currently focused so the user can understand what will happen on confirmation. **Save** must be focused by default when the popup opens.
- **FR-005**: While the quit-confirmation popup is active, the system MUST prevent the editor from quitting until the user explicitly chooses save, discard, or cancel.
- **FR-006**: The system MUST keep the quit-confirmation popup visible and actionable after terminal redraw events, including terminal resize.
- **FR-007**: The system MUST keep the quit-confirmation popup visible even when the current file path, status text, or other visible editor text is long. When long editor text would otherwise compete with the popup area, the popup MUST render as an overlay so the available actions remain clearly visible.
- **FR-008**: If the terminal is too small to show the full popup text, the system MUST show a compact visible popup with abbreviated actions and MUST preserve a usable path to save, discard, or cancel.
- **FR-009**: When the quit-confirmation popup is active, it MUST take precedence over other transient editor UI so that the unsaved-changes decision remains clear. Pressing `Esc` while the popup is active cancels it and returns the user to normal editing without quitting.
- **FR-009b**: If the user chooses Save from the quit-confirmation popup and the save operation fails, the system MUST keep the editor open, preserve the unsaved changes, close the popup, and surface the existing save error message UI clearly.
- **FR-010**: The system MUST continue to allow immediate quit (no popup) without the unsaved-changes prompt when the current document has no unsaved changes. For test writers: the clean-quit path completes in a single keypress pass—no popup appears at all.
- **FR-011**: The system MUST provide automated test coverage for the visible quit-confirmation flow and the relevant edge cases, including small terminal sizes, resize while prompted, long visible status content, and save failure while quitting.

### Key Entities *(include if feature involves data)*

- **Quit Confirmation Popup**: The visible decision state shown when a user tries to exit with unsaved changes, including the available actions and the currently focused action; it defaults focus to Save when opened, may appear as an overlay when competing editor text would otherwise obscure it, and closes if a save attempt fails so the existing save error message UI can be shown.
- **Editor View**: The currently visible terminal content presented to the user, including document area, status information, and any active popup.
- **Quit Decision**: The user's selected outcome when unsaved changes are present: save, discard, or cancel.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In validation of unsaved documents, 100% of `Ctrl-Q` attempts display a visible quit-confirmation popup before the editor exits.
- **SC-002**: In validation across at least three terminal sizes (concrete examples: 80×24 standard, 160×50 wide, and ~30×5 severely constrained), including one constrained layout, the quit-confirmation popup remains visible and actionable in 100% of automated test cases.
- **SC-003**: In validation with long visible file-path or status content, all automated tests confirm that the available quit actions are clearly identifiable on first display (replacing the previous 95% target—which is difficult to automate—with a deterministic pass/fail assertion). The remaining 95% manual-reporting figure may be retained for user-study metrics.
- **SC-004**: In validation runs where the terminal is resized while the popup is open, the popup remains usable without restarting the session in 100% of automated test cases. For purposes of this criterion, "usable" means the user can select at least one action within 3 keypresses or, at minimum, see one fully visible action label after redraw.
- **SC-011**: Automated tests cover the primary unsaved-quit flow and the defined popup-visibility edge cases for this feature, including save failure while quitting.

## Assumptions

- The existing unsaved-changes flow already defines the intended choices as save, discard, and cancel.
- The feature scope is limited to making the quit-confirmation popup visible and reliably usable, not redesigning the broader editor workflow.
- The editor continues to use keyboard-only interaction for prompt navigation and confirmation.
- The same visible popup area is expected to remain the standard place for user decisions during editing sessions.
