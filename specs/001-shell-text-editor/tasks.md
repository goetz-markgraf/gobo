# Tasks: Shell Text Editor

**Input**: Design documents from `/specs/001-shell-text-editor/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Automated verification is required by the constitution for editing, persistence, permissions, conflict handling, encoding validation, and other safety-critical flows. Write the listed tests first and confirm they fail before implementation.

**Organization**: Tasks are grouped by user story so each story can be implemented and validated independently once its dependencies are complete.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g. US1, US2, US3)
- Every task includes exact file paths

## Path Conventions

- Rust binary crate at repository root
- Production code in `src/`
- Automated tests in `tests/integration/` and `tests/unit/`
- Feature docs in `specs/001-shell-text-editor/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Initialize the Rust binary project and create the file layout required by the implementation plan.

- [X] T001 Initialize the Rust binary crate and declare `clap`, `ratatui`, `crossterm`, `ropey`, `unicode-segmentation`, and `unicode-width` in `Cargo.toml`
- [X] T002 Create the application module skeleton in `src/main.rs`, `src/cli.rs`, `src/app.rs`, `src/document.rs`, `src/editor/mod.rs`, `src/editor/buffer.rs`, `src/editor/cursor.rs`, `src/editor/input.rs`, `src/editor/render.rs`, `src/editor/search.rs`, and `src/editor/status.rs`
- [X] T003 [P] Create the planned automated test files in `tests/integration/open_and_save.rs`, `tests/integration/unsaved_guards.rs`, `tests/integration/readonly_and_conflict.rs`, `tests/integration/search_and_resize.rs`, `tests/unit/buffer.rs`, `tests/unit/cursor.rs`, and `tests/unit/search.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Build the shared editor core, document model, and rendering/session scaffolding required by all user stories.

**⚠️ CRITICAL**: Complete this phase before starting user story implementation.

- [X] T004 Implement the one-path CLI contract, startup validation, and exit-code error mapping in `src/cli.rs` and `src/main.rs`
- [X] T005 [P] Implement `DocumentBuffer`, `AccessMode`, `DiskSnapshot`, UTF-8 decoding checks, and filesystem open/save primitives in `src/document.rs`
- [X] T006 [P] Implement `EditingSession`, `SessionMode`, prompt state, and high-level app state transitions in `src/app.rs` and `src/editor/mod.rs`
- [X] T007 [P] Implement rope-backed text operations and line/character helpers for the in-memory buffer in `src/editor/buffer.rs`
- [X] T008 [P] Implement cursor state, preferred-column movement support, viewport bookkeeping, and terminal-size models in `src/editor/cursor.rs` and `src/editor/render.rs`
- [X] T009 [P] Implement shared status-message and prompt-display scaffolding for the TUI in `src/editor/status.rs` and `src/editor/render.rs`
- [X] T010 Add foundational unit coverage for buffer, document, and cursor invariants in `tests/unit/buffer.rs` and `tests/unit/cursor.rs`

**Checkpoint**: Foundation complete — the project can now deliver user stories in vertical slices.

---

## Phase 3: User Story 1 - Create and edit plain text files (Priority: P1) 🎯 MVP

**Goal**: Let a user open an existing UTF-8 text file or a new path, edit the content in the terminal, and save the result back to disk.

**Independent Test**: Open a sample file or missing path with `gobo <path>`, insert and delete text, save with `Ctrl-S`, exit, and verify the file on disk matches the edited content.

### Tests for User Story 1

- [X] T011 [P] [US1] Add failing integration coverage for opening an existing UTF-8 file, editing it, and saving it in `tests/integration/open_and_save.rs`
- [X] T012 [P] [US1] Add failing integration coverage for starting from a missing file path and creating the file on first save in `tests/integration/open_and_save.rs`

### Implementation for User Story 1

- [X] T013 [P] [US1] Implement insert, delete, and replace editing commands over the rope buffer in `src/editor/buffer.rs` and `src/editor/input.rs`
- [X] T014 [US1] Implement dirty tracking, save execution, and save success/failure feedback in `src/app.rs`, `src/document.rs`, and `src/editor/status.rs`
- [X] T015 [P] [US1] Render document text, cursor placement, file path, and dirty-state indicators in `src/editor/render.rs` and `src/editor/status.rs`
- [X] T016 [US1] Wire the interactive event loop so `gobo <path>` opens exactly one editable document session in `src/main.rs`, `src/cli.rs`, and `src/app.rs`

**Checkpoint**: User Story 1 is a complete MVP slice and should be independently demoable.

---

## Phase 4: User Story 2 - Avoid accidental data loss (Priority: P2)

**Goal**: Prevent silent data loss through unsaved-change guards, read-only handling, and explicit conflict prompts when disk state changes.

**Independent Test**: Edit a file, attempt to quit without saving, cancel the prompt, then test read-only open behavior and an external-change save conflict that offers reload, overwrite, or cancel.

### Tests for User Story 2

- [X] T017 [P] [US2] Add failing integration coverage for quit attempts with unsaved changes in `tests/integration/unsaved_guards.rs`
- [X] T018 [P] [US2] Add failing integration coverage for read-only files and external-change save conflicts in `tests/integration/readonly_and_conflict.rs`

### Implementation for User Story 2

- [X] T019 [US2] Implement unsaved-change prompts with save, discard, and cancel outcomes for quit flows in `src/app.rs` and `src/editor/status.rs`
- [X] T020 [P] [US2] Enforce read-only mode for non-writable files and show blocked-edit or blocked-save feedback in `src/document.rs`, `src/editor/input.rs`, and `src/editor/status.rs`
- [X] T021 [US2] Implement pre-save disk snapshot checks and reload/overwrite/cancel conflict resolution in `src/document.rs`, `src/app.rs`, and `src/editor/status.rs`

**Checkpoint**: User Story 2 safely protects the editing workflow from the main accidental-loss paths.

---

## Phase 5: User Story 3 - Work efficiently in a terminal-only environment (Priority: P3)

**Goal**: Provide keyboard-first navigation, case-insensitive search, resize resilience, and clear status feedback for terminal-only editing.

**Independent Test**: Complete a short edit using only the keyboard: move through the file with arrow keys, run a case-insensitive search, observe no-match feedback, resize the terminal, and keep editing without restarting.

### Tests for User Story 3

- [X] T022 [P] [US3] Add failing unit coverage for cursor movement, preferred-column behavior, and viewport clamping in `tests/unit/cursor.rs`
- [X] T023 [P] [US3] Add failing unit and integration coverage for case-insensitive search, no-match feedback, and resize handling in `tests/unit/search.rs` and `tests/integration/search_and_resize.rs`

### Implementation for User Story 3

- [X] T024 [US3] Implement keyboard navigation and vertical/horizontal cursor motion in `src/editor/input.rs` and `src/editor/cursor.rs`
- [X] T025 [P] [US3] Implement case-insensitive search state, match jumping, and no-match status feedback in `src/editor/search.rs` and `src/editor/status.rs`
- [X] T026 [US3] Implement terminal resize handling, viewport reflow, and redraw behavior in `src/app.rs` and `src/editor/render.rs`
- [X] T027 [P] [US3] Add visible status affordances for current file, mode, save state, and available next actions in `src/editor/status.rs` and `src/editor/render.rs`

**Checkpoint**: User Story 3 completes the keyboard-first terminal experience.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Finish documentation, performance, regression coverage, and final constitutional review across the whole feature.

- [X] T028 [P] Document CLI usage, keybindings, read-only behavior, conflict prompts, and no-crash-recovery scope in `README.md`
- [X] T029 Optimize render and save hot paths for typical UTF-8 files up to 1 MB in `src/editor/buffer.rs`, `src/editor/render.rs`, and `src/document.rs`
- [X] T030 [P] Add regression coverage for startup failure paths and UTF-8 grapheme-width handling in `tests/integration/open_and_save.rs` and `tests/unit/cursor.rs`
- [X] T031 [P] Align the manual validation scenarios and expected outcomes with the implemented behavior in `specs/001-shell-text-editor/quickstart.md`
- [X] T032 Perform a final readability, maintainability, and security review of module boundaries and comments in `src/app.rs`, `src/document.rs`, and `src/editor/mod.rs`

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
Phase 4 User Story 2 ─┐
                      ├─→ Phase 6 Polish
Phase 5 User Story 3 ─┘
```

### Phase Dependencies

- **Phase 1: Setup** — no dependencies
- **Phase 2: Foundational** — depends on Phase 1; blocks all story work
- **Phase 3: US1** — depends on Phase 2
- **Phase 4: US2** — depends on Phase 2 and the editable/save loop delivered in US1
- **Phase 5: US3** — depends on Phase 2 and the interactive editing loop delivered in US1
- **Phase 6: Polish** — depends on all desired user stories being complete

### User Story Dependencies

- **US1**: First deliverable and MVP; no dependency on other stories after Foundation
- **US2**: Builds on US1 dirty-state and save flows but remains independently testable once implemented
- **US3**: Builds on US1 session/render loop but remains independently testable once implemented

### Within Each User Story

- Write the listed tests first and confirm they fail
- Complete core implementation before integration wiring
- Keep each story independently runnable and verifiable before moving on

---

## Parallel Opportunities

- **Setup**: `T003` can run while `T001` and `T002` are in progress
- **Foundational**: `T005`, `T006`, `T007`, `T008`, and `T009` can proceed in parallel once `T004` defines the startup contract
- **US1**: `T011` and `T012` can run together; `T013` and `T015` can run in parallel after the tests exist
- **US2**: `T017` and `T018` can run together; `T020` can proceed in parallel with `T019` once the prompt flow shape is agreed
- **US3**: `T022` and `T023` can run together; `T025` and `T027` can run in parallel after navigation primitives are stable
- **Polish**: `T028`, `T030`, and `T031` can run in parallel

### Parallel Example: User Story 1

```bash
# Write failing US1 verification in parallel
Task: "T011 [US1] Add failing integration coverage for opening an existing UTF-8 file, editing it, and saving it in tests/integration/open_and_save.rs"
Task: "T012 [US1] Add failing integration coverage for starting from a missing file path and creating the file on first save in tests/integration/open_and_save.rs"

# Split implementation after tests exist
Task: "T013 [US1] Implement insert, delete, and replace editing commands in src/editor/buffer.rs and src/editor/input.rs"
Task: "T015 [US1] Render document text, cursor placement, file path, and dirty-state indicators in src/editor/render.rs and src/editor/status.rs"
```

### Parallel Example: User Story 2

```bash
# Write failing safety-flow verification in parallel
Task: "T017 [US2] Add failing integration coverage for quit attempts with unsaved changes in tests/integration/unsaved_guards.rs"
Task: "T018 [US2] Add failing integration coverage for read-only files and external-change save conflicts in tests/integration/readonly_and_conflict.rs"

# Split implementation once prompt structure exists
Task: "T019 [US2] Implement unsaved-change prompts in src/app.rs and src/editor/status.rs"
Task: "T020 [US2] Enforce read-only mode and blocked-edit feedback in src/document.rs, src/editor/input.rs, and src/editor/status.rs"
```

### Parallel Example: User Story 3

```bash
# Write failing keyboard/search verification in parallel
Task: "T022 [US3] Add failing unit coverage for cursor movement, preferred-column behavior, and viewport clamping in tests/unit/cursor.rs"
Task: "T023 [US3] Add failing unit and integration coverage for case-insensitive search, no-match feedback, and resize handling in tests/unit/search.rs and tests/integration/search_and_resize.rs"

# Split implementation after navigation primitives are stable
Task: "T025 [US3] Implement case-insensitive search state, match jumping, and no-match status feedback in src/editor/search.rs and src/editor/status.rs"
Task: "T027 [US3] Add visible status affordances in src/editor/status.rs and src/editor/render.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1
4. Run US1 integration tests and the matching quickstart scenario
5. Demo or review the single-document edit/save workflow before expanding scope

### Incremental Delivery

1. Deliver **US1** for core open/edit/save behavior
2. Add **US2** for destructive-action safeguards and conflict protection
3. Add **US3** for keyboard efficiency, search, and resize resilience
4. Finish with Phase 6 polish and regression coverage

### Suggested Team Strategy

1. One developer completes Setup + Foundational
2. Then:
   - Developer A focuses on US1 completion
   - Developer B prepares US2 tests and safety-flow scaffolding
   - Developer C prepares US3 tests and navigation/search scaffolding
3. Merge story slices only after each one passes its independent tests

---

## Notes

- All tasks use the required checklist format: checkbox, task ID, optional `[P]`, required story label for user-story phases, and exact file paths
- The CLI contract in `specs/001-shell-text-editor/contracts/cli-contract.md` is covered by `T004`, `T011`, `T012`, `T020`, and `T030`
- The data-model entities in `specs/001-shell-text-editor/data-model.md` map primarily to `T005`, `T006`, `T008`, `T019`, `T021`, and `T025`
- The validation scenarios in `specs/001-shell-text-editor/quickstart.md` should be used as the manual acceptance pass after the automated tests are green
