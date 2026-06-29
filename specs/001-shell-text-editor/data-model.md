# Data Model: Shell Text Editor

## Entity: DocumentBuffer

Represents the single UTF-8 document loaded for the session.

### Fields
- `path: PathBuf` — target file path for open/save
- `text: Rope` — in-memory UTF-8 document content
- `access_mode: AccessMode` — `Editable` or `ReadOnly`
- `dirty: bool` — whether in-memory text differs from the last persisted snapshot
- `exists_on_disk: bool` — whether the target file existed at open time
- `disk_snapshot: DiskSnapshot | None` — metadata captured after open/save for conflict detection
- `last_saved_at: SystemTime | None` — successful save timestamp for status feedback

### Validation Rules
- Path must resolve to a file target, not a directory.
- Existing files must decode as valid UTF-8 or open must fail with a clear error.
- If the file is readable but not writable, `access_mode` must be `ReadOnly`.
- New files remain editable and are created on first successful save.

### State Transitions
- `Clean -> Dirty` when text changes.
- `Dirty -> Clean` after a successful save.
- `Editable -> ReadOnly` only at open time for non-writable existing files.
- `ConflictPending` is reached indirectly when save detects a changed `disk_snapshot`.

## Entity: DiskSnapshot

Tracks the persisted state used to detect save conflicts.

### Fields
- `modified_at: SystemTime | None` — last modification time observed on disk
- `size_bytes: u64 | None` — file size at open/save time
- `content_fingerprint: String | None` — optional digest for stronger conflict checks

### Validation Rules
- Snapshot is `None` for unsaved new files.
- Snapshot must be refreshed after every successful save or reload from disk.
- A mismatch between current on-disk state and `disk_snapshot` before save triggers the conflict prompt.

## Entity: EditingSession

Owns the active interactive state for exactly one open document.

### Fields
- `document: DocumentBuffer`
- `cursor: CursorState`
- `viewport: ViewportState`
- `mode: SessionMode`
- `search: SearchState | None`
- `status: StatusMessage | None`
- `pending_prompt: PromptState | None`
- `terminal_size: TerminalSize`

### Validation Rules
- Exactly one `DocumentBuffer` per session.
- Session must always know the current terminal size.
- A prompt that can discard work must block the destructive action until the user confirms or cancels.

### State Transitions
- `Browsing/Editing -> SearchInput` when search starts.
- `Browsing/Editing -> ConfirmQuit` when quitting with unsaved changes.
- `Browsing/Editing -> SaveConflictPrompt` when the disk snapshot changed before save.
- `Any -> Exiting` only after safe quit or explicit discard.

## Entity: CursorState

Tracks the logical editing location.

### Fields
- `char_index: usize` — current logical insertion point in the rope
- `preferred_column: usize` — desired visual column when moving vertically
- `selection: Option<SelectionRange>` — reserved for none in v1 unless a future need emerges

### Validation Rules
- `char_index` must remain on a valid character boundary.
- Vertical movement must clamp to the available line length.
- Read-only mode may move the cursor but must not mutate text.

## Entity: ViewportState

Tracks what part of the document is visible.

### Fields
- `top_line: usize`
- `left_column: usize`
- `visible_height: u16`
- `visible_width: u16`

### Validation Rules
- Visible dimensions must update after terminal resize.
- Viewport must keep the cursor visible after navigation, search jumps, and resize.

## Entity: SearchState

Represents the current in-editor search interaction.

### Fields
- `query: String`
- `case_mode: CaseMode` — initial value `Insensitive`
- `last_match_char_range: Option<(usize, usize)>`
- `last_result: SearchResultState`

### Validation Rules
- Empty query does not trigger a match jump.
- Initial release defaults to case-insensitive matching.
- No-match results must surface a visible status message.

## Supporting Enums

### `AccessMode`
- `Editable`
- `ReadOnly`

### `SessionMode`
- `Editing`
- `SearchInput`
- `ConfirmQuit`
- `SaveConflictPrompt`
- `Exiting`

### `PromptState`
- `UnsavedChanges { action: Quit | ReloadDocument }`
- `SaveConflict { options: Reload | Overwrite | Cancel }`

### `SearchResultState`
- `Idle`
- `MatchFound`
- `NoMatch`

## Relationships
- One `EditingSession` owns exactly one `DocumentBuffer`.
- One `EditingSession` owns one `CursorState` and one `ViewportState`.
- One `DocumentBuffer` optionally owns one `DiskSnapshot`.
- One `EditingSession` optionally owns one active `SearchState` and one `PromptState`.
