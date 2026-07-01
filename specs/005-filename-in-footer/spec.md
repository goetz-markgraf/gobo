# Feature Specification: Filename in Footer Row

**Feature Branch**: `[005-filename-in-footer]`

**Created**: 2026-07-01

**Status**: Draft

**Input**: User description: "Ich möchte in der Fußzeile den Namen der aktuell bearbeiteten Datei anzeigen. (gobo something.txt oder gobo somedir/something.txt) Rechtsbündig in der untersten Zeile die heute vor allem aus '-' besteht."

## Clarifications

### Session 2026-07-01

- Q: Should the filename be displayed as absolute path, relative path, or CLI argument as-is? → A: Display the path exactly as given on the command line (e.g., `something.txt` or `somedir/something.txt`).
- Q: Should additional info (access mode, dirty state) appear in this footer row alongside the filename? → A: Only the dirty state — append ` (*)` (blank + asterisk in parentheses) when the file has unsaved changes. Access mode stays in the status line.
- Q: What about newly created files (not yet saved)? → A: Still show the CLI argument path — same behavior, no change needed since `document.path` stores the original argument even for new files.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Display filename in footer row (Priority: P1)

A user opens a file with gobo and sees the file path right-aligned in the bottommost row of the screen, separate from the status line above it that shows mode/dirty/message information.

**Why this priority**: File identification is fundamental — users need to know which file they are editing at all times. This is a persistent UI element visible on every render frame.

**Independent Test**: Can be tested by opening gobo with any file and verifying the bottom row displays the path right-aligned, without duplication of information already shown in the status line.

**Acceptance Scenarios**:

1. **Given** the user runs `gobo something.txt`, **When** the editor renders, **Then** the bottommost screen row displays `something.txt` right-aligned
2. **Given** the user runs `gobo somedir/something.txt`, **When** the editor renders, **Then** the bottommost screen row displays `somedir/something.txt` right-aligned
3. **Given** the user opens a file with an absolute path `/Users/foo/bar.txt`, **When** the editor renders, **Then** the bottommost screen row displays `/Users/foo/bar.txt` right-aligned
4. **Given** the user opens a clean file `something.txt` and makes an edit, **When** the editor renders, **Then** the bottom row displays `something.txt (*)` right-aligned (with one blank between name and mark)
5. **Given** a dirty file that the user saves, **When** the editor re-renders, **Then** the `(*)` mark is removed and only the plain filename appears again

---

## Functional Requirements

- **FR-001**: The editor MUST display the original CLI argument path (as stored in `document.path`) right-aligned in a dedicated footer row at the bottom of the screen. When the file is dirty, append a blank and a `(*)` behind the filename.
- **FR-002**: The footer row is always visible — it takes 1 line from the viewport budget regardless of mode
- **FR-003**: The status line content (mode, dirty state, access mode, message) moves out of duplication with the footer and remains on its own row above the footer
- **FR-004**: The current layout artifact producing stray empty/hyphen rows MUST be eliminated — exactly 3 visible areas render: body | status-line | footer (or body | status-line | search-prompt during SearchInput mode, plus footer)
- **FR-005**: During `SearchInput` mode, the layout expands to: body | status-line | search-prompt | footer — the footer still shows the filename

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The bottommost screen row always displays the file path right-aligned in all session modes
- **SC-002**: No stray empty rows or `-` filler artifacts appear in the layout
- **SC-003**: The footer path text does not overlap with body content, status line, or popups
- **SC-004**: Existing integration tests compile and pass — no behavioral regressions

## Assumptions

- `document.path` holds the original CLI argument without canonicalization (already true)
- The path will never exceed terminal width in normal use; if it does, truncate from the left with `...` prefix
- No changes to `cli.rs`, `document.rs`, or `editor/input.rs` are required
