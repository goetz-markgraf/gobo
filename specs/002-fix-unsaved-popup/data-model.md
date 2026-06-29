# Data Model: Fix Unsaved Popup

## Entity: QuitConfirmationPrompt

Represents the modal decision state shown when a user presses `Ctrl-Q` with unsaved changes.

### Fields
- `action: PromptAction` — the pending destructive action being confirmed; for this feature the active flow is `Quit`
- `focus: UnsavedChoice` — the currently focused option: `Save`, `Discard`, or `Cancel`
- `visible: bool` — derived from `pending_prompt` containing an unsaved-changes prompt

### Validation Rules
- The prompt is created only when the document is dirty and the user requests quit.
- When the prompt opens, `focus` must default to `Save`.
- While visible, the prompt blocks quitting until the user confirms save, discard, or cancel.
- Pressing `Esc` while visible closes the prompt and returns the session to normal editing.

### State Transitions
- `Hidden -> Visible` when `Ctrl-Q` is triggered on a dirty document.
- `Visible -> Hidden` on `Cancel`, on successful discard, or after a save attempt resolves.
- `Visible -> Hidden + Editing` when save fails, so the normal save error UI can be shown.

## Entity: QuitPromptPresentation

Represents the derived visual form of the quit-confirmation popup.

### Fields
- `variant: PromptVariant` — `Full` or `Compact`
- `title: String` — visible popup heading such as `Unsaved changes` or a shorter compact title
- `message: String` — visible explanation text when space allows
- `actions: Vec<PromptActionLabel>` — ordered labels for save, discard, and cancel, including focused state
- `help_text: String` — key hints such as `←/→, Tab, Enter, Esc`
- `popup_rect: PopupRect` — the screen area reserved for the overlay

### Validation Rules
- `variant` is derived from current terminal width and height, not stored as behavioral editor state.
- `popup_rect` must fit entirely inside the active terminal area.
- The full variant is used when the terminal can show the full title, message, actions, and help clearly.
- The compact variant is used when the terminal is too small for the full content but still must expose save, discard, and cancel.

### State Transitions
- `Full -> Compact` when resize reduces available space below the full-layout threshold.
- `Compact -> Full` when resize restores enough space.
- Presentation may change on any redraw without changing the underlying prompt behavior.

## Entity: PromptActionLabel

Represents one selectable action in the visible popup.

### Fields
- `choice: UnsavedChoice` — `Save`, `Discard`, or `Cancel`
- `label: String` — full or abbreviated visible label depending on the presentation variant
- `focused: bool` — whether this action is currently selected

### Validation Rules
- Exactly one action label must be focused while the popup is visible.
- Action ordering remains stable across redraws so keyboard navigation stays predictable.
- Compact labels may abbreviate text, but the three choices must remain distinguishable.

## Entity: PopupRect

Represents the derived on-screen area for the popup overlay.

### Fields
- `x: u16`
- `y: u16`
- `width: u16`
- `height: u16`

### Validation Rules
- Width and height must be non-zero when the popup is rendered.
- The rectangle must remain within the terminal bounds after resize.
- The rectangle should keep the popup centered when space allows.

## Entity: QuitSaveAttemptOutcome

Represents the result of choosing Save from the quit-confirmation popup.

### Fields
- `result: SaveAttemptResult` — `Saved`, `ConflictPrompted`, `BlockedReadOnly`, or `Failed`
- `error_message: Option<String>` — present only when `result` is `Failed`
- `keeps_editor_open: bool` — whether the session stays in editing mode afterward
- `preserves_dirty_state: bool` — whether unsaved edits remain in memory afterward

### Validation Rules
- `Saved` transitions to exiting only after a successful write.
- `Failed` must close the quit popup, keep the editor open, preserve unsaved changes, and surface the save error UI.
- `ConflictPrompted` may open the save-conflict prompt instead of exiting.

## Supporting Enums

### `PromptVariant`
- `Full`
- `Compact`

### `SaveAttemptResult`
- `Saved`
- `ConflictPrompted`
- `BlockedReadOnly`
- `Failed`

## Relationships
- One `EditingSession` optionally owns one `QuitConfirmationPrompt` through `pending_prompt`.
- One visible `QuitConfirmationPrompt` derives one `QuitPromptPresentation` per render pass.
- One `QuitPromptPresentation` owns three ordered `PromptActionLabel` values.
- One save attempt from the quit prompt produces one `QuitSaveAttemptOutcome`.
