---

description: "Task list for Enter Key Newline Editing feature"
---

# Tasks: Enter Key Newline Editing

**Input**: Design documents from `/specs/003-enter-newline-edit/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/editor-command-enter.md

**Tests**: Automated test tasks are REQUIRED. Every user story and every user-visible feature must include automated tests covering the primary flow and relevant edge cases per Constitution Section IV (Verification Before Merge).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

## Phase 1: Setup

**Purpose**: Verify the project builds and environment is ready

- [X] T001 Verify `cargo build --release` succeeds from repository root
- [X] T002 Ensure `tests/integration/` directory exists with existing test files

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Prepare the codebase for the Enter command addition — no user story behavior changes yet

- [X] T003 [P] Document module boundaries and coding conventions for `src/editor/input.rs` and `src/app.rs` in relevant file comments
- [X] T004 Identify safeguard requirements: confirm `DocumentBuffer::is_read_only()` guard already blocks edits throughout the edit pipeline (verify in `handle_editing_command`, `insert_text`, `backspace`, `delete` in src/app.rs)

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 + User Story 2 - Enter Key Newline Editing (Priority: P1) 🎯 MVP

> **Note**: US1 (end-of-line newline) and US2 (mid-line split) are two behavioral outcomes of the same underlying operation — inserting '\n' at the cursor position. They share a single implementation path and can be implemented as one cohesive task, then tested with separate test files for each scenario.

**Goal**: Add `EditorCommand::Enter` variant, map Enter key to it, and handle newline insertion at cursor using existing buffer utilities. Pressing Enter at end-of-line creates a new blank line; pressing Enter mid-line splits the line into two.

**Independent Test**: Run the editor, press Enter at end of a line — verify a blank line appears below with cursor positioned at column 0. Move cursor mid-line and press Enter — verify the line splits with text before cursor on the top line and text after cursor on the new bottom line.

### Tests for User Story 1 + User Story 2 (REQUIRED) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T005 [P] [US1] Integration test for Enter-at-end-of-line creating a new blank line in `tests/integration/enter_newline.rs`
- [X] T006 [P] [US2] Integration test for Enter-mid-line splitting the line correctly in `tests/integration/enter_newline.rs`
- [X] T007 [P] Edge-case integration tests (empty document, single empty line, read-only protection) in `tests/integration/enter_newline.rs`

### Implementation for User Stories 1 + 2

- [X] T008 Add `Enter` variant to `EditorCommand` enum in `src/editor/input.rs`
- [X] T009 Update `map_key_event` in `src/editor/input.rs`: change `(_, KeyCode::Enter) => Some(EditorCommand::Confirm)` to `(_, KeyCode::Enter) => Some(EditorCommand::Enter)`
- [X] T010 Add match arm for `EditorCommand::Enter` in `handle_editing_command` in `src/app.rs`: delegate to a private helper that inserts '\n' at `self.cursor.char_index` using `buffer::insert_text`, updates cursor position, calls `document.mark_dirty()`, and sets status message
- [X] T011 Ensure the new match arm for `Enter` does NOT conflict with existing `Confirm`/`Cancel`/choice-handling patterns in search mode — verify that `handle_search_command` still maps Enter to `Confirm` (no change needed, but audit confirms correctness)

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently and all tests pass

---

## Phase N: Polish & Cross-Cutting Concerns

**Purpose**: Final review and quality assurance

- [X] T012 [P] Review readability of `src/editor/input.rs` and `src/app.rs` changes against Constitution Section I (Readability First): names reveal intent, control flow straightforward, no surprising behavior
- [X] T013 [P] Review maintainability of touched files against Constitution Section II (Maintainable Design): clear boundaries between input handling, command dispatch, buffer operations — one reason to change per module
- [X] T014 Verify security: confirm that `Enter` in editing mode does NOT override the read-only guard in any code path, and in search/prompt modes Enter still maps to `Confirm` as before (no behavioral regression)
- [X] T015 Run all tests (`cargo test --lib && cargo test --test integration`) to confirm no regressions from existing functionality
- [X] T016 Verify success criteria from spec.md: multi-line documents work (SC-001), mid-line split shows expected content (SC-002), document remains valid in all edge cases (SC-003)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion
- **User Stories (Phase 3)**: Depend on Foundational phase completion
- **Polish (Phase N)**: Depends on User Stories being complete

### User Story Dependencies

- **US1 + US2 (P1, same file, single implementation)**: Can start after Foundational — no dependencies on other stories since this is the first feature addition

### Within Each User Story

- Automated tests MUST be written and FAIL before implementation (Constitution Section IV)
- Tests written first → Implementation → Tests pass
- Integration tests verify behavior through `cargo test --test integration`
- Readability/maintainability review after implementation

---

## Parallel Example: Testing

```bash
# Run all Enter-key integration tests together:
cargo test --test enter_newline

# Run full regression suite to confirm no regressions:
cargo test --lib && cargo test --test integration
```

---

## Implementation Strategy

### MVP First (Single User Story — US1 + US2)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: Implement Enter command + write failing tests → implement → all tests pass
4. **STOP and VALIDATE**: Run `cargo test --test integration` to independently verify Enter behavior
5. Complete Polish phase for final review

### Success Criteria

- `EditorCommand::Enter` exists in the enum
- Enter key maps to `EditorCommand::Enter` (not `Confirm`) via `map_key_event`
- Pressing Enter at end of line creates a new blank line
- Pressing Enter mid-line splits correctly with cursor positioned between halves
- Read-only protection works (no edits, status message shown)
- Search mode Enter key still maps to `Confirm` (no regression)
- All existing tests pass + new integration tests pass

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- US1 and US2 share a single implementation (same file edits, same code path)
- Verify tests fail before implementing (Constitution Section IV: Verification Before Merge)
- Commit after each task or logical group
- Stop at checkpoint to validate story independently
