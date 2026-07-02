# Feature Specification: Filename in Footer Row

**Feature Branch**: `[005-filename-in-footer]`

**Created**: 2026-07-01

**Status**: Draft

**Input**: User description: "Ich möchte in der Fußzeile den Namen der aktuell bearbeiteten Datei anzeigen. (gobo something.txt oder gobo somedir/something.txt) Rechtsbündig in der untersten Zeile die heute vor allem aus '-' besteht."

## Clarifications

### Session 2026-07-01

- Q: Should the filename be displayed as absolute path, relative path, or CLI argument as-is? → A: Display the path exactly as given on the command line (e.g., `something.txt` or `somedir/something.txt`).
- Q: Should additional info (access mode, dirty state) appear in this footer row alongside the filename? → A: Only the dirty state — append ` (*)` (blank + asterisk in parentheses) when the file has unsaved changes. Access mode and editing mode are NOT shown.
- Q: What about newly created files (not yet saved)? → A: Still show the CLI argument path — same behavior, no change needed since `document.path` stores the original argument even for new files.

### Session 2026-07-02 (design simplification)

- The previous design kept a separate status line (mode/access/message) above a footer line. In practice on wide terminals the right-aligned filename in the lower row was hard to perceive, and the mode/access information was not requested by the user.
- Decision: collapse to a SINGLE bottom row. It shows the filename plus optional ` (*)` on the LEFT, and the current status message on the RIGHT. There is no longer a separate status-line row, and no mode/access display.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Display filename and message in footer row (Priority: P1)

A user opens a file with gobo and sees the file path (with an optional ` (*)` dirty marker) on the LEFT of the bottommost screen row, and the current status message on the RIGHT of the same row. There is no separate status line and no mode/access display.

**Why this priority**: File identification is fundamental — users need to know which file they are editing at all times. This is a persistent UI element visible on every render frame.

**Independent Test**: Can be tested by opening gobo with any file and verifying the bottom row displays the path on the left and the status message on the right.

**Acceptance Scenarios**:

1. **Given** the user runs `gobo something.txt`, **When** the editor renders, **Then** the bottommost screen row displays `something.txt` on the left
2. **Given** the user runs `gobo somedir/something.txt`, **When** the editor renders, **Then** the bottommost screen row displays `somedir/something.txt` on the left
3. **Given** the user opens a file with an absolute path `/Users/foo/bar.txt`, **When** the editor renders, **Then** the bottommost screen row displays `/Users/foo/bar.txt` on the left
4. **Given** the user opens a clean file `something.txt` and makes an edit, **When** the editor renders, **Then** the bottom row displays `something.txt (*)` on the left (with one blank between name and mark)
5. **Given** a dirty file that the user saves, **When** the editor re-renders, **Then** the `(*)` mark is removed and only the plain filename appears again on the left
6. **Given** the editor shows a status message (e.g. "Ready", "Match found", "No match", "Search cancelled"), **When** the editor renders, **Then** the bottommost row displays that message on the right, alongside the filename on the left

---

## Functional Requirements

- **FR-001**: The editor MUST display the original CLI argument path (as stored in `document.path`) on the LEFT of a single footer row at the bottom of the screen. When the file is dirty, append a blank and a `(*)` behind the filename.
- **FR-002**: The footer row is always visible — it takes 1 line from the viewport budget regardless of mode
- **FR-003**: The current status message (e.g. "Ready", "Match found", "No match", "Search cancelled") MUST be displayed on the RIGHT of the same footer row. There is NO separate status-line row and NO mode/access display.
- **FR-004**: The current layout artifact producing stray empty/hyphen rows MUST be eliminated — exactly 2 visible areas render: body | footer (or body | search-prompt | footer during SearchInput mode)
- **FR-005**: During `SearchInput` mode, the layout expands to: body | search-prompt | footer — the footer still shows the filename on the left and the status message on the right

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The bottommost screen row always displays the file path on the left in all session modes
- **SC-002**: The bottommost screen row always displays the current status message on the right in all session modes
- **SC-003**: No stray empty rows or `-` filler artifacts appear in the layout
- **SC-004**: The footer text does not overlap with body content, the search prompt, or popups
- **SC-005**: Existing integration tests compile and pass — no behavioral regressions

## Assumptions

- `document.path` holds the original CLI argument without canonicalization (already true)
- The path will never exceed terminal width in normal use; if it does, truncate from the left with `...` prefix
- No changes to `cli.rs`, `document.rs`, or `editor/input.rs` are required
