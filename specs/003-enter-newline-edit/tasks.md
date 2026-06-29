---

description: "Task list for Enter Key Newline Editing feature"
---

# Tasks: Enter Key Newline Editing

**Input**: Design documents from `/specs/003-enter-newline-edit/`

**Prerequisites**: plan.md, spec.md, data-model.md, contracts/editor-command-enter.md

**Tests**: Automated test tasks are REQUIRED. Every user story and every in-scope feature must include tasks for the primary flow and relevant edge cases.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

<!--
    ============================================================================
  Tasks generated from specs/003-enter-newline-edit design documents.

  Both user stories share the same core behavior: inserting '\n' at cursor position.
  The Enter key is mapped to EditorCommand::Enter (currently maps to Confirm),
  and the handler delegates to existing insert_text method which has read-only
  guard, cursor update, dirty flag, and status message setup.

  Scope: 2 files modified (input.rs, app.rs) + 1 new test file (enter_newline.rs)
    ============================================================================
-->

## Phase 3: User Story 1 - Press Enter to Create / Split Lines (Priority: P1)

**Goal**: Pressing Enter with cursor at the end of a line creates a new blank line below. Pressing Enter mid-line splits the current line — text before cursor stays on top, text after moves to the new bottom line. Cursor ends up at position 0 of the new second line.

**Independent Test**: Open a document, type "Hello World", move cursor between "Hello" and "World", press Enter. Verify: line 1 = "Hello ", line 2 = "World", cursor positioned at start of line 2. Then position cursor at end of any line and press Enter — a new blank line appears below. Both lines remain intact.

### Tests for User Story 1 (REQUIRED)

> **NOTE: Write these tests first to verify they FAIL before implementation**

- [ ] T004 [P] [US1] Integration test: end-of-line Enter creates new blank line in tests/integration/enter_newline.rs
- [ ] T005 [P] [US1] Integration test: mid-line Enter splits line into two with correct content in tests/integration/enter_newline.rs
- [ ] T006 [P] [US1] Integration test: Edge case — Enter on empty document creates first line in tests/integration/enter_newline.rs
- [ ] T007 [P] [US1] Integration test: Edge case — Enter on empty line adds second blank line in tests/integration/enter_newline.rs
- [ ] T008 [P] [US1] Integration test: Edge case — Enter on read-only file blocks edit in tests/integration/enter_newline.rs

### Implementation for User Story 1

- [ ] T009 [US1] Add `EditorCommand::Enter` variant to `EditorCommand` enum in src/editor/input.rs
- [ ] T010 [US1] Change key mapping from `Confirm` to `Enter` for `KeyCode::Enter` in `map_key_event` in src/editor/input.rs
- [ ] T011 [US1] Add match arm for `EditorCommand::Enter` delegating to `self.insert_text("\n")` in `handle_editing_command` in src/app.rs

**Checkpoint**: User Story 1 is fully functional — Enter creates newlines at end of line, splits lines mid-line, preserves other content, and places cursor correctly.

---

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: Validation against design documents and existing regression safety

- [ ] T012 Run `cargo test --lib` and `cargo test --test integration` to confirm all existing tests still pass
- [ ] T013 Validate quickstart.md manual scenarios (scenario 1: end-of-line, scenario 2: mid-line, scenario 4: empty document) against the built editor
- [ ] T014 Review readability of src/editor/input.rs and src/app.rs — confirm naming consistency with existing conventions

---

## Dependencies & Execution Order

### Phase Dependencies

- **User Story 1 (Phase 3)**: No phase dependencies — this is a focused single-editor feature on an existing, building Rust project.
   - T004-T008 (tests) can run in parallel
   - Tests must FAIL before implementation tasks (T009-T011) are completed
   - T011 depends on T009 and T010 (code must compile for tests to work)

### Within Each User Story

- Integration tests for the story's primary flow and edge cases must be written first and FAIL before implementation
- Add variant (T009) before mapping change (T010), then handler (T011)
- Read-only guard delegates automatically through `self.insert_text("\n")` — no additional checks needed

### Parallel Opportunities

- All test tasks (T004-T008) can be written and run in parallel
- Variant enum addition (T009), key mapping change (T010), and handler (T011) are sequentially dependent but small enough to complete together
- T006 (empty document) is independent of T007 (empty line)

---

## Implementation Strategy

### MVP First

1. Add `EditorCommand::Enter` variant and key mapping in `src/editor/input.rs` (T009-T010)
2. Add Enter handler delegating to `self.insert_text("\n")` in `handle_editing_command` (T011)
3. Manual test: build and run editor, press Enter to verify newline behavior
4. Write integration tests covering all acceptance scenarios (T004-T008)

### Incremental Delivery

1. T009+T010+T011: Core Enter key behavior works — manually test with built editor
2. T004-T008: Automated tests verify all acceptance scenarios and edge cases
3. T012: Regression check — confirm all existing tests still pass

### Notes

- Both US1 P1 stories share the same implementation since inserting `'\n'` at cursor handles both end-of-line newline creation AND mid-line splitting automatically
- No new modules, no changes to save/draw/render paths
- Handled via existing `buffer::insert_text` — no new buffer methods needed
- Read-only guard is handled automatically by the shared `insert_text` method
