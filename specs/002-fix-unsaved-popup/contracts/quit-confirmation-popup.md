# UI Contract: Quit Confirmation Popup

## Purpose

Define the visible behavior of the unsaved-changes quit confirmation shown when the user presses `Ctrl-Q` with unsaved edits.

## Trigger Contract

### Dirty document
- **Given** the current document has unsaved changes
- **When** the user presses `Ctrl-Q`
- **Then** the editor must stay open and show a visible quit-confirmation popup before any quit happens

### Clean document
- **Given** the current document has no unsaved changes
- **When** the user presses `Ctrl-Q`
- **Then** the editor exits immediately without showing the popup

## Visibility Contract

- The popup must be rendered inside the active terminal area.
- The popup must visually take precedence over document text, status text, and long file-path text.
- The popup must remain visible after redraw and terminal resize.
- When the terminal is too small for the full popup, the editor must show a compact popup variant instead of hiding the decision UI.

## Actions Contract

The popup exposes exactly three choices:
- `Save`
- `Discard`
- `Cancel`

### Focus behavior
- `Save` is focused by default when the popup opens.
- The focused action must be visually distinct.
- Keyboard navigation may use left/right arrows, `Tab`, or `Shift-Tab`.

### Confirmation behavior
- `Enter` confirms the focused action.
- `Esc` cancels the popup and returns to normal editing.

## Outcome Contract

### Save
- Attempt to save the current document.
- If save succeeds, continue the pending quit flow and exit.
- If save fails, close the popup, keep the editor open, preserve unsaved changes, and show the existing save error message UI.
- **Save-conflict flow:** When an external change conflict is detected, the editor delegates to the existing save-conflict resolution prompt (see `src/document.rs` conflict-handling path). The quit confirmation remains on hold until the user resolves the conflict—this prevents silent data loss from overwriting newer content.

### Discard
- Close the popup and exit without saving.

### Cancel
- Close the popup and return to normal editing without quitting.

## Constrained-Terminal Contract

- The compact popup must still make it clear that quitting requires a choice.
- The compact popup must still expose save, discard, and cancel.
- Compact labels may be abbreviated, but actions must remain distinguishable.
- Resize while prompted may switch between full and compact variants without changing the selected action.

## Precedence Contract

- While the popup is active, other transient editor UI must not hide or replace it.
- The popup remains the active decision surface until the user confirms an action or cancels it.
