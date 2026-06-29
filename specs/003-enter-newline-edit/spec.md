# Feature Specification: Enter Key Newline Editing

**Feature Branch**: `003-enter-newline-edit`

**Created**: 2026-06-29

**Status**: Draft

**Input**: User description: "Enter at the end of a line should create a new line. Enter before the end of the line should split the line. Like in any normal text editor"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Press Enter at End of Line to Start New Line (Priority: P1)

When the user is editing a document and presses the Enter key with the cursor positioned at the very end of the current line, a new blank line is created and the cursor moves to the beginning of that new line. The original line remains intact with all its content preserved.

**Why this priority**: This is the most common text editing action. Users type content and press Enter to move to a new line, just like in any standard text editor. Without this behavior, users cannot write multi-line documents at all.

**Independent Test**: Open a document, type text, position cursor at end of line, press Enter. Verify a new blank line is created and cursor appears on the next line. The original line remains unchanged.

**Acceptance Scenarios**:

1. **Given** user has typed "Hello" and cursor is after the "o", **When** user presses Enter, **Then** a new empty line is created below and cursor moves to the start of the new line.
2. **Given** user has typed text and is editing the last line of the document, **When** user presses Enter at the end of that line, **Then** content of all previous lines remains unchanged and a new blank line appears after the current line.
3. **Given** the cursor is positioned in the middle of a line containing text, **When** user presses Enter, **Then** the line is split into two lines: the left portion (text before cursor) stays on the first line, the right portion (text after cursor) moves to the new second line, and cursor sits between them.
4. **Given** a multi-line document with text on several lines, **When** user presses Enter with mid-line cursor on any line, **Then** only that specific line is split into two, all other content remains in place.

---

### User Story 2 - Press Enter Mid-Line to Split Line (Priority: P1)

When the user is editing a document and presses the Enter key with the cursor positioned somewhere in the middle of a line (not at the very end), the current line is split into two lines. The text before the cursor stays on the original line, the text after the cursor moves to a new line below, and the cursor is placed between the two halves on the new line.

**Why this priority**: Splitting a line mid-content is essential for editing existing content. Users frequently need to break up sentences or rearrange content by splitting existing lines during editing.

**Independent Test**: Open a document, type "Hello World", move cursor to position after "Hello " (before "World"), press Enter. Verify "Hello " remains on the first line and "World" appears on the new second line with cursor positioned between them.

---

### Edge Cases

- **What happens when the cursor is on an empty line and Enter is pressed?** A new blank line is created below, just like with any other line (the empty line stays empty, new one added after).
- **How does the system handle pressing Enter at the very start of a document (no existing lines)?** An initial empty line structure is established and cursor moves to allow content entry on a new first line.
- **What occurs when Enter is pressed and the document has only whitespace/blank characters (but no visible text) on the line?** The line is split as usual — whitespace before cursor stays left, after cursor goes right on new line. No special stripping or trimming.

## Functional Requirements

- **FR-001**: System MUST create a new blank line below the current line when the user presses Enter with the cursor positioned at the end of that line.
- **FR-002**: System MUST split the current line into two lines when the user presses Enter with the cursor positioned anywhere before the end of that line, placing all text after the cursor on the newly created line.
- **FR-003**: System MUST preserve all content in other lines unchanged when entering a newline at or near any specific line.
- **FR-004**: System MUST ensure the cursor always ends up positioned correctly within the editing area after an Enter key press, allowing further text entry immediately.
- **FR-005**: System MUST behave consistently with standard text editor conventions so users recognize the same Enter key behavior they use in other applications.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can enter multi-line documents without interruption by using Enter to create new lines at the end of existing content.
- **SC-002**: After pressing Enter mid-line, the user sees exactly the text they expect: original line split cleanly with content before cursor on top, content after cursor on bottom.
- **SC-003**: 100% of entered newlines result in a valid document state that can be saved, displayed, and edited further without data loss or corruption.
- **SC-004**: Automated tests cover the primary flows (end-of-line enter, mid-line split) and all documented edge cases for every behavior change.

## Assumptions

- The user is operating in a line-editing mode within the text editor where each document line is visible and editable.
- The cursor position can be determined precisely by character offset or column number.
- The Enter key maps to a newline/newline-break event in the terminal input stream.
- No special modifiers (Shift+Enter, Ctrl+Enter) are requested for this feature; only plain Enter is handled.
- Lines are stored with an internal line break representation that separates them for display and editing purposes but does not require writing external file formats at this stage.
