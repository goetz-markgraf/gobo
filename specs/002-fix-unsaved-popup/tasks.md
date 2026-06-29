# Tasks: Fix Unsaved Popup

**Input**: Design documents from `/specs/002-fix-unsaved-popup/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Automated test tasks are required by the constitution and by `specs/002-fix-unsaved-popup/spec.md`. Write the listed tests first, confirm they fail, then implement the feature.

**Organization**: Tasks are grouped by user story so each story can be implemented and tested independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g. US1, US2)
- Every task includes exact file paths

## Path Conventions

- Rust CLI/TUI application at repository root
- Production code in `src/`
- Integration tests in `tests/integration/`
- Feature docs in `specs/002-fix-unsaved-popup/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare the popup-focused render and test surfaces needed by the feature.

- [X] T001 Add popup-specific render/view placeholders and naming comments in `src/editor/render.rs` and `src/editor/status.rs`
- [X] T002 [P] Prepare shared prompt test setup helpers for dirty-session scenarios in `tests/integration/unsaved_guards.rs`, `tests/integration/search_and_resize.rs`, and `tests/integration/readonly_and_conflict.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Refactor the shared prompt and draw pipeline so popup overlays can be implemented safely.

**⚠️ CRITICAL**: Complete this phase before starting user story implementation.

- [X] T003 Refactor `RenderView` and prompt-formatting flow from bottom-line prompts to popup overlay data in `src/editor/render.rs` and `src/editor/status.rs`
- [X] T004 [P] Update prompt precedence, resize bookkeeping, and viewport reservation so overlay prompts do not consume bottom document rows in `src/app.rs` and `src/editor/render.rs`
- [X] T005 [P] Refactor the terminal draw path to support centered overlay blocks with background clearing in `src/main.rs`

**Checkpoint**: Popup scaffolding is ready and user stories can now be implemented as vertical slices.

---

## Phase 3: User Story 1 - See the unsaved-changes quit prompt (Priority: P1) 🎯 MVP

**Goal**: Show a clearly visible quit-confirmation popup for dirty documents, keep `Save` focused by default, allow `Esc` to cancel, and keep the editor open with an error message if save fails.

**Independent Test**: Edit a document, press `Ctrl-Q`, verify that a visible popup appears with save/discard/cancel actions, confirm `Save` is focused by default, press `Esc` to return to editing, and verify that a forced save failure closes the popup while preserving unsaved changes and showing the save error.

### Tests for User Story 1 (REQUIRED) ⚠️

- [X] T006 [P] [US1] Add failing integration coverage for visible quit-confirmation popup rendering, default `Save` focus, and `Esc` cancellation in `tests/integration/unsaved_guards.rs`
- [X] T007 [P] [US1] Add failing integration coverage for save failure after confirming `Save` from the quit popup in `tests/integration/readonly_and_conflict.rs`

### Implementation for User Story 1

- [X] T008 [US1] Implement full-size quit-popup view data, focused action labels, and popup copy in `src/editor/render.rs` and `src/editor/status.rs`
- [X] T009 [US1] Render the unsaved-changes popup above the editor body and status line in `src/main.rs`
- [X] T010 [US1] Update quit-confirmation state transitions so the popup takes precedence over other transient UI and `Esc` returns to editing in `src/app.rs`
- [X] T011 [US1] Handle save failure from quit confirmation without exiting by closing the popup and surfacing the existing error status in `src/app.rs`, `src/document.rs`, and `src/editor/status.rs`

**Checkpoint**: User Story 1 is independently functional and can be validated as the MVP fix.

---

## Phase 4: User Story 2 - Use the prompt in constrained terminal layouts (Priority: P2)

**Goal**: Keep the quit-confirmation popup visible and usable in narrow, short, resized, and long-path terminal layouts.

**Independent Test**: Trigger the quit popup in at least three terminal sizes, including a constrained layout and a long-path file, then resize while the popup is open and verify that the prompt stays visible and actionable.

### Tests for User Story 2 (REQUIRED) ⚠️

- [X] T012 [P] [US2] Add failing integration coverage for resize-while-prompted behavior and compact popup fallback in `tests/integration/search_and_resize.rs`
- [X] T013 [P] [US2] Add failing integration coverage for long file-path popup precedence in `tests/integration/unsaved_guards.rs`

### Implementation for User Story 2

- [X] T014 [US2] Implement compact popup titles, abbreviated action labels, and terminal-size-based variant selection in `src/editor/render.rs` and `src/editor/status.rs`
- [X] T015 [US2] Recompute and redraw popup layout on terminal resize without losing prompt state or focused action in `src/app.rs` and `src/main.rs`
- [X] T016 [US2] Isolate popup overlay rendering from long status and path output so competing text cannot obscure the prompt in `src/editor/render.rs`, `src/editor/status.rs`, and `src/main.rs`

**Checkpoint**: User Story 2 keeps the prompt visible and usable across constrained and changing terminal layouts.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Finish regression coverage, validation guidance, and final cleanup across the feature.

- [X] T017 [P] Add cross-story regression coverage for immediate clean quit and quit-popup precedence over active search in `tests/integration/unsaved_guards.rs` and `tests/integration/search_and_resize.rs`
- [X] T018 [P] Update popup validation steps and expected outcomes in `specs/002-fix-unsaved-popup/quickstart.md`
- [X] T019 Perform a final readability, maintainability, and security cleanup of popup-related logic in `src/app.rs`, `src/main.rs`, `src/editor/render.rs`, and `src/editor/status.rs`

---

## Dependencies & Execution Order

### Dependency Graph

```text
Phase 1 Setup
  ↓
Phase 2 Foundational
  ↓
Phase 3 User Story 1 (MVP)
  ↓
Phase 4 User Story 2
  ↓
Phase 5 Polish
```

### Phase Dependencies

- **Phase 1: Setup** — no dependencies
- **Phase 2: Foundational** — depends on Phase 1 and blocks all story work
- **Phase 3: US1** — depends on Phase 2
- **Phase 4: US2** — depends on Phase 2 and on the visible popup behavior delivered in US1
- **Phase 5: Polish** — depends on all desired user stories being complete

### User Story Dependencies

- **US1**: First delivery slice and MVP; no dependency on other stories after Foundation
- **US2**: Builds on the popup behavior from US1 but remains independently testable once implemented

### Within Each User Story

- Write the listed automated tests first and confirm they fail
- Implement render/model changes before wiring the final interaction flow
- Keep each story independently runnable and verifiable before moving on

---

## Parallel Opportunities

- **Setup**: `T002` can run in parallel with `T001`
- **Foundational**: `T004` and `T005` can run in parallel after `T003`
- **US1**: `T006` and `T007` can run in parallel because they touch different test files
- **US2**: `T012` and `T013` can run in parallel because they touch different test files
- **Polish**: `T017` and `T018` can run in parallel

### Parallel Example: User Story 1

```bash
# Write failing US1 tests in parallel
Task: "T006 [US1] Add failing integration coverage for visible quit-confirmation popup rendering, default Save focus, and Esc cancellation in tests/integration/unsaved_guards.rs"
Task: "T007 [US1] Add failing integration coverage for save failure after confirming Save from the quit popup in tests/integration/readonly_and_conflict.rs"
```

### Parallel Example: User Story 2

```bash
# Write failing constrained-layout tests in parallel
Task: "T012 [US2] Add failing integration coverage for resize-while-prompted behavior and compact popup fallback in tests/integration/search_and_resize.rs"
Task: "T013 [US2] Add failing integration coverage for long file-path popup precedence in tests/integration/unsaved_guards.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1
4. Validate the popup fix with the US1 automated tests and the matching quickstart scenarios
5. Demo or review the fix before extending to constrained-layout coverage

### Incremental Delivery

1. Deliver **US1** to restore a safe, visible quit-confirmation flow
2. Deliver **US2** to harden the popup across constrained and resized terminal layouts
3. Finish with Phase 5 polish and regression coverage

### Suggested Team Strategy

1. One developer completes Setup + Foundational
2. Then:
   - Developer A implements US1 popup rendering and quit/save behavior
   - Developer B prepares US2 constrained-layout tests and follow-up render changes
3. Merge only after each story passes its independent tests

---

## Notes

- All tasks follow the required checklist format: checkbox, task ID, optional `[P]`, required story label for story phases, and exact file paths
- The UI contract in `specs/002-fix-unsaved-popup/contracts/quit-confirmation-popup.md` is covered primarily by `T006`, `T008`, `T009`, `T014`, `T015`, and `T016`
- The prompt presentation entities in `specs/002-fix-unsaved-popup/data-model.md` map primarily to `T003`, `T008`, `T014`, and `T015`
- Use `specs/002-fix-unsaved-popup/quickstart.md` as the manual validation pass after the automated tests are green
