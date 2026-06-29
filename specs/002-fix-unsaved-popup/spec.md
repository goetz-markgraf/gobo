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

1. **Given** an open document with unsaved changes, **When** the user presses `Ctrl-Q`, **Then** a visible quit-confirmation prompt appears in the terminal before the editor exits.
2. **Given** the quit-confirmation prompt is visible, **When** the user chooses save, discard, or cancel, **Then** the editor performs the chosen action and the prompt no longer blocks the screen.
3. **Given** the quit-confirmation prompt has just opened, **Then** the Save action is focused by default.
4. **Given** the quit-confirmation prompt is visible, **When** the user presses `Esc`, **Then** the prompt closes and the editor returns to normal editing without quitting.
5. **Given** the user chooses Save from the quit-confirmation prompt and the save operation fails, **Then** the editor remains open, preserves the unsaved changes, closes the quit-confirmation prompt, and shows the existing save error message UI instead of quitting.

---

### User Story 2 - Use the prompt in constrained terminal layouts (Priority: P2)

When the terminal is narrow, short, or contains long status text, the user can still see and use the quit-confirmation prompt.

**Why this priority**: Prompt visibility must remain reliable in realistic shell conditions, not only in ideal terminal sizes.

**Independent Test**: Can be fully tested by triggering the quit-confirmation prompt in multiple terminal sizes and with long file names or status text, then confirming that the prompt remains visible and actionable.

**Acceptance Scenarios**:

1. **Given** a document with unsaved changes in a narrow or short terminal, **When** the user presses `Ctrl-Q`, **Then** the prompt remains visible enough to show that a decision is required and what actions are available.
2. **Given** the prompt is already visible, **When** the terminal is resized, **Then** the prompt stays visible after redraw and still allows the user to complete the quit decision.
3. **Given** long status or file-path text would otherwise compete with the prompt area, **When** the user presses `Ctrl-Q`, **Then** the editor shows the quit-confirmation prompt as a popup so the available actions remain clearly visible.
4. **Given** the terminal is too small to fit the full prompt text, **When** the user presses `Ctrl-Q`, **Then** the editor shows a compact confirmation prompt with abbreviated actions that still allows save, discard, or cancel.

### Edge Cases

- If the terminal is too small for the full prompt text, the editor shows a compact confirmation prompt with abbreviated actions that still allows save, discard, or cancel.
- If the current file path or status text would compete for the same screen area, the editor shows the quit-confirmation prompt as a popup so the available actions remain clearly visible.
- If the terminal is resized while the quit-confirmation prompt is already open, the prompt stays visible after redraw and still allows the user to complete the quit decision.
- Pressing `Esc` while the quit-confirmation prompt is open cancels the prompt and returns to normal editing without quitting.
- If the user chooses Save from the quit-confirmation prompt and the save operation fails, the editor remains open, preserves the unsaved changes, closes the quit-confirmation prompt, and shows the existing save error message UI.
- Pressing `Ctrl-Q` with no unsaved changes exits the editor without showing the unsaved-changes prompt.
- If another transient UI element was active just before `Ctrl-Q`, the unsaved-changes prompt takes precedence while the quit decision is pending.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST show a visible quit-confirmation prompt whenever a user attempts to quit a document that has unsaved changes.
- **FR-002**: The quit-confirmation prompt MUST be visible within the active terminal view and MUST not depend on hidden or off-screen content.
- **FR-003**: The quit-confirmation prompt MUST present the available actions to save, discard, or cancel.
- **FR-004**: The quit-confirmation prompt MUST indicate which action is currently focused so the user can understand what will happen on confirmation.
- **FR-004a**: When the quit-confirmation prompt opens, the **Save** action MUST be focused by default.
- **FR-005**: While the quit-confirmation prompt is active, the system MUST prevent the editor from quitting until the user explicitly chooses save, discard, or cancel.
- **FR-006**: The system MUST keep the quit-confirmation prompt visible and actionable after terminal redraw events, including terminal resize.
- **FR-007**: The system MUST keep the quit-confirmation prompt visible even when the current file path, status text, or other visible editor text is long.
- **FR-007a**: When long visible editor text would otherwise compete with the prompt area, the system MUST show the quit-confirmation prompt as a popup so the available actions remain clearly visible.
- **FR-008**: If the terminal is too small to show the full prompt text, the system MUST show a compact visible confirmation prompt with abbreviated actions and MUST preserve a usable path to save, discard, or cancel.
- **FR-009**: When the quit-confirmation prompt is active, it MUST take precedence over other transient editor UI so that the unsaved-changes decision remains clear.
- **FR-009a**: While the quit-confirmation prompt is active, pressing `Esc` MUST cancel the prompt and return the user to normal editing without quitting.
- **FR-009b**: If the user chooses Save from the quit-confirmation prompt and the save operation fails, the system MUST keep the editor open, preserve the unsaved changes, close the quit-confirmation prompt, and surface the existing save error message UI clearly.
- **FR-010**: The system MUST continue to allow immediate quit without the unsaved-changes prompt when the current document has no unsaved changes.
- **FR-011**: The system MUST provide automated test coverage for the visible quit-confirmation flow and the relevant edge cases, including small terminal sizes, resize while prompted, long visible status content, and save failure while quitting.

### Key Entities *(include if feature involves data)*

- **Quit Confirmation Prompt**: The visible decision state shown when a user tries to exit with unsaved changes, including the available actions and the currently focused action; it defaults focus to Save when opened, may appear as a popup when competing editor text would otherwise obscure it, and closes if a save attempt fails so the existing save error message UI can be shown.
- **Editor View**: The currently visible terminal content presented to the user, including document area, status information, and any active prompt.
- **Quit Decision**: The user’s selected outcome when unsaved changes are present: save, discard, or cancel.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In validation of unsaved documents, 100% of `Ctrl-Q` attempts display a visible quit-confirmation prompt before the editor exits.
- **SC-002**: In validation across at least three terminal sizes, including one constrained layout, the quit-confirmation prompt remains visible and actionable in 100% of tested cases.
- **SC-003**: In validation with long visible file-path or status content, users can still identify the available quit actions on first attempt in at least 95% of test runs.
- **SC-004**: In validation runs where the terminal is resized while the prompt is open, the prompt remains usable without restarting the session in 100% of tested cases.
- **SC-005**: Automated tests cover the primary unsaved-quit flow and the defined prompt-visibility edge cases for this feature, including save failure while quitting.

## Assumptions

- The existing unsaved-changes flow already defines the intended choices as save, discard, and cancel.
- The feature scope is limited to making the quit-confirmation prompt visible and reliably usable, not redesigning the broader editor workflow.
- The editor continues to use keyboard-only interaction for prompt navigation and confirmation.
- The same visible prompt area is expected to remain the standard place for user decisions during editing sessions.
