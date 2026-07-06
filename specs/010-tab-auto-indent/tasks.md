# Tasks: Tab Support and Auto-Indent

**Input**: Design documents from `/specs/010-tab-auto-indent/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/tab-auto-indent.md, quickstart.md

**Tests**: Automated test tasks are REQUIRED for this feature because `spec.md` defines FR-018 and the constitution requires verification for every user-visible behavior change.

**Organization**: Tasks are grouped by user story so each story can be implemented and tested independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel when the referenced files do not overlap and prerequisites are complete
- **[Story]**: Which user story this task belongs to (`[US1]`, `[US2]`, `[US3]`)
- Every task includes exact file path(s)

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare the repo for the new indentation helper and standalone tests.

- [ ] T001 Register `integration_tab_auto_indent` and `unit_indent` test targets in Cargo.toml
- [ ] T002 [P] Export the new indentation helper module in src/editor/mod.rs
- [ ] T003 [P] Introduce `EditorCommand::Tab` and map `KeyCode::Tab` without breaking `KeyCode::BackTab` in src/editor/input.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Create the shared command-routing and edit-planning seams that all three user stories rely on.

**⚠️ CRITICAL**: No user story work should start before this phase is complete.

- [ ] T004 Create the pure indentation helper skeleton and action-planning types in src/editor/indent.rs
- [ ] T005 Preserve prompt navigation by routing `EditorCommand::Tab` to prompt choice movement in src/app.rs
- [ ] T006 Add a shared atomic selection-aware edit path for Tab, Enter, and Backspace in src/app.rs

**Checkpoint**: Editing-mode Tab is distinct from prompt navigation, and `src/editor/indent.rs` exists as the shared pure-logic home for the feature.

---

## Phase 3: User Story 1 - Mit Tab zur nächsten Einrückungsstufe springen (Priority: P1) 🎯 MVP

**Goal**: Pressing Tab in editing mode inserts 1 or 2 spaces so the cursor lands on the next even column, while prompt-mode Tab behavior stays unchanged.

**Independent Test**: Place the cursor at even and odd columns in editing mode, press Tab, and verify that only spaces are inserted, the cursor lands on the next even column, selection replacement happens atomically, read-only mode blocks mutation, and prompt dialogs still use Tab/Shift-Tab for navigation.

### Tests for User Story 1 (write first, ensure they fail before implementation)

- [ ] T007 [P] [US1] Add tab-width and cursor-column parity unit tests in tests/unit/indent.rs
- [ ] T008 [US1] Add editing-mode Tab integration tests for even/odd columns, mid-line insertion, selection replacement, prompt navigation, and read-only blocking in tests/integration/tab_auto_indent.rs

### Implementation for User Story 1

- [ ] T009 [US1] Implement Tab column and inserted-space planning helpers in src/editor/indent.rs
- [ ] T010 [US1] Implement editing-mode Tab atomic insertion, cursor placement, selection clearing, and one-step undo handling in src/app.rs

**Checkpoint**: User Story 1 works independently and can serve as the MVP slice.

---

## Phase 4: User Story 2 - Neue Zeile mit gleicher Einrückung beginnen (Priority: P1)

**Goal**: Pressing Enter inserts a newline plus the current line’s leading spaces, preserving text to the right of the cursor and treating selection replacement as one undo step.

**Independent Test**: Press Enter at the end, middle, and start of indented and non-indented lines, then verify the new line gets exactly the original leading spaces, right-side text stays intact, selection replacement is atomic, and undo restores the pre-Enter state in one step.

### Tests for User Story 2 (write first, ensure they fail before implementation)

- [ ] T011 [P] [US2] Add leading-space detection and newline payload unit tests in tests/unit/indent.rs
- [ ] T012 [US2] Update Enter auto-indent integration coverage for indented lines, split lines, no-indent lines, all-space lines, and selection replacement in tests/integration/enter_newline.rs and tests/integration/tab_auto_indent.rs

### Implementation for User Story 2

- [ ] T013 [US2] Implement leading-indentation detection and newline payload builders in src/editor/indent.rs
- [ ] T014 [US2] Replace plain Enter insertion with auto-indent, split-line preservation, selection replacement, and one-step undo behavior in src/app.rs

**Checkpoint**: User Story 2 works independently without requiring User Story 1 or User Story 3 behavior to validate it.

---

## Phase 5: User Story 3 - Mit Backspace sauber ausrücken (Priority: P1)

**Goal**: Pressing Backspace inside an all-space prefix removes 1 or 2 spaces to reach the previous even column, while mixed-content prefixes keep normal Backspace behavior.

**Independent Test**: Place the cursor after even and odd counts of leading spaces and verify Backspace removes the correct number of spaces, does nothing at column 0, falls back to normal Backspace after mixed content, applies selection replacement first, and records the whole action as one undo step.

### Tests for User Story 3 (write first, ensure they fail before implementation)

- [ ] T015 [P] [US3] Add all-space-prefix detection and smart-outdent width unit tests in tests/unit/indent.rs
- [ ] T016 [US3] Add smart Backspace integration and undo coverage for even/odd prefixes, mixed prefixes, column-0 behavior, and selection replacement in tests/integration/tab_auto_indent.rs and tests/integration/undo_redo.rs

### Implementation for User Story 3

- [ ] T017 [US3] Implement smart Backspace planning helpers and mixed-prefix fallback detection in src/editor/indent.rs
- [ ] T018 [US3] Implement editing-mode smart Backspace, normal fallback, selection replacement, and one-step undo behavior in src/app.rs

**Checkpoint**: All three user stories are independently functional and testable.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Finish cross-story regressions, docs, and boundary cleanup.

- [ ] T019 [P] Add cross-story regression coverage for Tab, Enter, and Backspace selection semantics in tests/integration/text_selection.rs
- [ ] T020 [P] Update feature validation commands and manual verification steps in specs/010-tab-auto-indent/quickstart.md
- [ ] T021 Run final boundary cleanup for command mapping, pure indentation logic, and stateful edit orchestration in src/editor/input.rs, src/editor/indent.rs, and src/app.rs

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1: Setup** → can start immediately
- **Phase 2: Foundational** → depends on Phase 1 and blocks all user stories
- **Phase 3: US1** → depends on Phase 2 only
- **Phase 4: US2** → depends on Phase 2 only
- **Phase 5: US3** → depends on Phase 2 only
- **Phase 6: Polish** → depends on all desired user stories being complete

### User Story Dependency Graph

```text
Setup -> Foundational -> {US1, US2, US3} -> Polish
```

### Suggested Completion Order

```text
US1 (MVP) -> US2 -> US3
```

This is a delivery recommendation, not a functional dependency. After Phase 2, the three stories are designed to be independently testable.

### Within Each User Story

- Write the story’s automated tests first and confirm they fail
- Implement pure helper logic in `src/editor/indent.rs`
- Wire the editing-session behavior in `src/app.rs`
- Re-run the story-specific tests before moving on

---

## Parallel Opportunities

- `T002` and `T003` can run in parallel after `T001`
- For **US1**, `T007` can run in parallel with `T008`
- For **US2**, `T011` can run in parallel with `T012`
- For **US3**, `T015` can run in parallel with `T016`
- `T019` and `T020` can run in parallel during Polish
- After Phase 2, different team members can own `US1`, `US2`, and `US3` concurrently, but they should coordinate changes in `src/app.rs` and `src/editor/indent.rs`

---

## Parallel Example: User Story 1

```bash
Task: "Add tab-width and cursor-column parity unit tests in tests/unit/indent.rs"
Task: "Add editing-mode Tab integration tests in tests/integration/tab_auto_indent.rs"
```

## Parallel Example: User Story 2

```bash
Task: "Add leading-space detection and newline payload unit tests in tests/unit/indent.rs"
Task: "Update Enter auto-indent integration coverage in tests/integration/enter_newline.rs and tests/integration/tab_auto_indent.rs"
```

## Parallel Example: User Story 3

```bash
Task: "Add all-space-prefix detection and smart-outdent width unit tests in tests/unit/indent.rs"
Task: "Add smart Backspace integration and undo coverage in tests/integration/tab_auto_indent.rs and tests/integration/undo_redo.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1
4. Validate `cargo test --test integration_tab_auto_indent --test unit_indent`
5. Stop for review before expanding to Enter and Backspace

### Incremental Delivery

1. Finish Setup + Foundational once
2. Deliver **US1** for Tab indentation
3. Deliver **US2** for auto-indent on Enter
4. Deliver **US3** for smart Backspace outdent
5. Finish with Polish regression coverage and quickstart updates

### Team Strategy

1. One developer completes Setup + Foundational
2. Then split by story:
   - Developer A: `US1`
   - Developer B: `US2`
   - Developer C: `US3`
3. Reconcile shared-file changes in `src/app.rs` and `src/editor/indent.rs`
4. Finish with shared regression and documentation work

---

## Notes

- All tasks use the required checklist format
- `[P]` markers only appear on tasks that can proceed without overlapping file edits in the same step
- Tests are explicit because the constitution and `spec.md` require automated verification for normal flows and edge cases
- Story independence is preserved by keeping pure calculations in `src/editor/indent.rs` and mode-aware orchestration in `src/app.rs`
