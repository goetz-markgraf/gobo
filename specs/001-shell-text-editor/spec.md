# Feature Specification: Shell Text Editor

**Feature Branch**: `[001-shell-text-editor]`

**Created**: 2026-06-29

**Status**: Draft

**Input**: User description: "Ich möchte einen Shell-basierten Text-Editor erstellen. Er soll ähnlich einfach sein wie nono, aber im Detail anders funktionieren. Als Techstack möchte ich mit Rust arbeiten, sodass der Editor nur ein Binary ist (\"gobo\")."

## Clarifications

### Session 2026-06-29

- Q: How many documents can one editor session handle? → A: One document per session only.
- Q: What should happen if the file changed on disk after it was opened? → A: Warn the user and let them choose whether to reload, overwrite, or cancel.
- Q: What file size should the first version handle well? → A: Typical text files up to 1 MB.
- Q: How should text search handle letter case? → A: Search is case-insensitive by default.
- Q: What text encoding should the first version support? → A: UTF-8 text files only.
- Q: What recovery behavior should the initial release provide for crashes or terminal interruptions? → A: No crash/interruption recovery beyond normal unsaved-change warnings.
- Q: How should the editor handle existing files that are readable but not writable? → A: Open them in read-only mode and clearly indicate that editing is disabled.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Create and edit plain text files (Priority: P1)

A user can open an existing text file or start a new one, change its content directly in the terminal, and save the result back to disk without leaving the editor.

**Why this priority**: This is the core value of the product. Without reliable file editing and saving, the editor does not fulfill its main purpose.

**Independent Test**: Can be fully tested by opening a sample file, inserting and deleting text, saving it, and verifying that the updated file content matches the user’s changes.

**Acceptance Scenarios**:

1. **Given** a user starts the editor with the path to an existing text file, **When** the file opens, **Then** the user sees the file content and can move through it to make edits.
2. **Given** a user has made changes to an open document, **When** the user saves the document, **Then** the file on disk is updated and the editor confirms that the save succeeded.
3. **Given** a user starts the editor with the path to a file that does not yet exist, **When** the user enters text and saves, **Then** a new file is created with the entered content.

---

### User Story 2 - Avoid accidental data loss (Priority: P2)

A user is warned before losing unsaved changes so that mistakes such as quitting too early, opening another file, or interruptions do not silently discard work.

**Why this priority**: Terminal editing is often used for quick, high-value changes. Preventing accidental loss strongly affects user trust and day-to-day usability.

**Independent Test**: Can be fully tested by editing a file, attempting to quit or replace the current document without saving, and verifying that the editor blocks destructive actions until the user confirms or saves.

**Acceptance Scenarios**:

1. **Given** a document contains unsaved changes, **When** the user tries to quit the editor, **Then** the editor warns about unsaved work and requires an explicit decision to save, discard, or cancel.
2. **Given** a document contains unsaved changes, **When** the editing session ends unexpectedly because of a crash or terminal interruption, **Then** the initial release does not restore unsaved work automatically and the user must reopen the file manually.

---

### User Story 3 - Work efficiently in a terminal-only environment (Priority: P3)

A user can perform common editing tasks from the keyboard alone, including moving through text, finding content, and understanding the current editing state without relying on a mouse or graphical interface.

**Why this priority**: The editor’s target environment is a shell session, so keyboard-driven interaction and clear terminal feedback are important for usability.

**Independent Test**: Can be fully tested by completing a short editing task in a terminal using only keyboard actions to navigate, search, edit, and save.

**Acceptance Scenarios**:

1. **Given** a user is editing a document, **When** the user invokes movement actions, **Then** the cursor position updates predictably within the visible text.
2. **Given** a user wants to find a string in the document, **When** the user starts a search and enters a query, **Then** the editor matches text case-insensitively by default, highlights or jumps to matching content, and clearly indicates when no match exists.
3. **Given** a user is actively editing, **When** the editor state changes, **Then** the interface shows enough status information for the user to understand the current file, save state, and available next action.

### Edge Cases

- What happens when the user opens a very small file, an empty file, or a file containing only one long line?
- How does the system handle attempts to open a path that does not exist, is a directory, cannot be read, or is not valid UTF-8 text?
- How does the system handle save attempts when the target location cannot be written?
- If an existing file is readable but not writable, the editor must open it in read-only mode and clearly indicate that editing is disabled.
- What happens when the terminal window is resized during editing?
- If the file changes on disk after it was opened, the editor must warn the user before saving and offer reload, overwrite, or cancel options.
- How does the system respond when a search query produces no matches?
- What happens when the user tries to quit immediately after making the first unsaved change?
- What happens when the editor process or terminal session ends unexpectedly before the user saves?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST allow a user to start an editing session from the shell for exactly one target text file path, whether the file already exists or will be created on first save.
- **FR-002**: The system MUST display the current document content in the terminal in a form the user can navigate and edit directly.
- **FR-003**: The system MUST allow users to insert text, delete text, and replace text within the current document.
- **FR-004**: The system MUST allow users to save the current document to its target file path during an editing session.
- **FR-004a**: If an existing target file is readable but not writable, the system MUST open it in read-only mode and clearly indicate that editing is disabled.
- **FR-005**: The system MUST clearly indicate whether the current document has unsaved changes.
- **FR-006**: The system MUST warn the user before any action that would discard unsaved changes.
- **FR-007**: The system MUST allow the user to cancel a destructive action after receiving an unsaved-changes warning.
- **FR-008**: The system MUST provide keyboard-based navigation for moving through the document without requiring a mouse.
- **FR-009**: The system MUST provide an in-editor search capability for locating text within the current document.
- **FR-009a**: The system MUST perform search case-insensitively by default in the initial release.
- **FR-010**: The system MUST communicate the result of important actions, including open failures, save failures, successful saves, and search misses, using clear terminal-visible feedback.
- **FR-010a**: The system MUST support UTF-8 plain-text files in the initial release and MUST show a clear error when a target file is not valid UTF-8 text.
- **FR-011**: The system MUST adapt its visible layout when the terminal size changes so that editing can continue without restarting the session.
- **FR-012**: The system MUST start and operate as a standalone command-line application that users can invoke directly from a shell session.
- **FR-013**: The system MUST preserve basic usability when editing common plain-text files used for notes, configuration, and source code.
- **FR-013a**: The system MUST support smooth editing for typical plain-text files up to 1 MB in size in the initial release.
- **FR-014**: The system MUST detect when the file on disk has changed since it was opened, warn the user before saving, and require an explicit choice to reload from disk, overwrite the file, or cancel the save.
- **FR-015**: The system MUST support only one open document per editor session in the initial release and MUST require starting a new session to edit a different file.
- **FR-016**: The system MUST NOT promise automatic recovery of unsaved work after a crash or terminal interruption in the initial release.

### Key Entities *(include if feature involves data)*

- **Document**: The text content currently being edited, including its file path, current text, save state, and whether it differs from the stored file.
- **Editing Session**: The active terminal interaction context, including the open document, cursor position, search state, current status feedback shown to the user, and whether the current document is read-only.
- **User Action**: A keyboard-driven command or text input that changes document content, navigation position, search focus, or session state.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user can open an existing text file, make a simple edit, and save it successfully in under 30 seconds on first use.
- **SC-002**: At least 90% of users completing a basic editing task can do so without needing to leave the terminal or rely on external instructions after initial onboarding.
- **SC-003**: In validation scenarios covering quit and replace flows initiated within the editor, 100% of unsaved-change cases trigger a visible warning before user work is discarded.
- **SC-004**: In representative tests with common plain-text files, at least 95% of save attempts that fail provide a clear reason and leave the previously stored file content intact.
- **SC-005**: Users can complete a find-and-edit workflow on a multi-line document using only keyboard input with a task completion rate of at least 90%.
- **SC-006**: In representative validation runs with plain-text files up to 1 MB, the editor remains responsive enough for users to navigate, edit, search, and save without perceptible input lag.

## Assumptions

- The first version allows exactly one open plain-text document per editor session.
- The primary users are developers or terminal users who prefer lightweight shell-based tools for quick edits.
- Advanced capabilities such as plugins, split views, multi-file tabs, collaborative editing, and rich text formatting are out of scope for the initial release.
- The editor is expected to run in a terminal environment that supports interactive keyboard input and screen refresh.
- Users launch the editor from a shell and provide the target file path at startup for that session.
- The product goal of being simple and lightweight is interpreted as fast startup, keyboard-first interaction, and minimal operational setup for the user.
- The initial release targets typical plain-text files up to 1 MB rather than very large-file workflows.
- The initial release supports UTF-8 plain-text files only.
- Crash recovery or automatic restoration of unsaved work is out of scope for the initial release.
- Existing files that cannot be written but can be read open in read-only mode in the initial release.
