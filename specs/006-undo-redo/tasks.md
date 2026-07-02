---

description: "Task list template for feature implementation"
---

# Tasks: Undo / Redo

**Input**: Design documents from `/specs/006-undo-redo/`

**Spec**: `specs/006-undo-redo/spec.md` | **Plan**: `specs/006-undo-redo/plan.md`
**Research**: `specs/006-undo-redo/research.md` | **Data Model**: `specs/006-undo-redo/data-model.md`
**Contracts**: `specs/006-undo-redo/contracts/api.md` | **Quickstart**: `specs/006-undo-redo/quickstart.md`

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story. The constitution (`specs/006-undo-redo/plan.md` → Constitution Check → Verification Gate) requires automated tests for every user-visible feature and edge case; test tasks are therefore REQUIRED for each user story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Single project: `src/` and `tests/` at repository root. Per `architecture.md`, unit tests live in `tests/unit/` and integration tests in `tests/integration/`; each integration test file is a standalone `[[test]]` target registered in `Cargo.toml`. All positions are character indices.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure. No new dependencies are required (per plan.md → Technical Context, the existing `ropey`/`crossterm`/`ratatui`/`clap` deps suffice).

- [ ] T001 Verify build baseline passes on the `006-undo-redo` branch: `cargo build` and `cargo test` both succeed before any undo/redo changes are made; record the green baseline
- [ ] T002 Add `editor/history.rs` empty module stub and register it in `src/editor/mod.rs` so the crate compiles with a placeholder `History` struct (no logic yet) in `src/editor/history.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The `History` / `EditStep` core data structures and pure invariants. MUST be complete before ANY user story, because every story records/restores steps through this module.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [ ] T003 [P] Implement `EditStep` enum and its pure derived helpers (`len_chars`, `end_index`, `before_cursor`, `after_cursor`) exactly per `specs/006-undo-redo/data-model.md` in `src/editor/history.rs`
- [ ] T004 [P] Implement `RecordOutcome { oldest_dropped: bool }` struct in `src/editor/history.rs`
- [ ] T005 Implement `History` struct (fields `undo: Vec<EditStep>`, `redo: Vec<EditStep>`, private `undo_capacity: usize`) with constructors `new()` (capacity `usize::MAX`) and `with_capacity(usize)` (test injection point) in `src/editor/history.rs`
- [ ] T006 Implement `History::record(&mut self, step: EditStep) -> RecordOutcome`: push to `undo`, clear `redo`, and when `undo.len() > undo_capacity` evict the oldest (`remove(0)`) and return `oldest_dropped: true` in `src/editor/history.rs`
- [ ] T007 Implement `History::undo(&mut self, text: &mut Rope) -> Option<usize>`: pop top undo step, apply its **reverse** diff (`Insert→remove`, `Delete→insert`) using `ropey::Rope::insert`/`remove` in char indices, push the step onto `redo`, return `Some(step.before_cursor())`; return `None` and mutate nothing when undo is empty, in `src/editor/history.rs`
- [ ] T008 Implement `History::redo(&mut self, text: &mut Rope) -> Option<usize>`: pop top redo step, apply its **forward** diff, push the step back onto `undo`, return `Some(step.after_cursor())`; `None` when redo is empty, in `src/editor/history.rs`
- [ ] T009 [P] Implement `History::can_undo`, `can_redo`, and `clear` helpers in `src/editor/history.rs`
- [ ] T010 [P] Register the new unit-test target `[[test]] name = "unit_history", path = "tests/unit/history.rs"` in `Cargo.toml`
- [ ] T011 Write failing unit tests for `History` pure invariants in `tests/unit/history.rs`: insert↔delete reverse symmetry (round-trip rope identity), `undo` then `redo` no-op on rope+cursor, `record` clears redo, `record` at capacity evicts the oldest step with `oldest_dropped == true`, empty-stack `undo`/`redo` return `None` and mutate nothing
- [ ] T012 Run `cargo test --test unit_history`: confirm all new unit tests pass after T003–T009 land
- [ ] T013 Add `history: History` field to `EditingSession` in `src/app.rs` and initialize it as `History::new()` in `EditingSession::new()`; confirm `cargo build` still compiles
- [ ] T014 Document `history.rs` module purpose and the History invariants (clear-redo-on-push, reverse-diff symmetry, session-bound lifetime) as module-level doc comments in `src/editor/history.rs`

**Checkpoint**: Foundation ready — `History` type complete and unit-tested; the session owns an empty history; user story implementation can now begin.

---

## Phase 3: User Story 1 — Mistakes rückgängig machen (Priority: P1) 🎯 MVP

**Goal**: Ctrl-Z performs one-step Undo of the last text change; repeated Ctrl-Z walks back through history to the origin; Undo at the empty stack is a no-op. Satisfies FR-001, FR-003, FR-004 (per-step), FR-010, FR-011, and US1 acceptance scenarios.

**Independent Test**: Open an empty/seed document, apply several edits, then press Ctrl-Z repeatedly and confirm the text and cursor match each prior state exactly, until the origin is reached; further Ctrl-Z is a no-op. (See `quickstart.md` → US1-A1/A2/A3 + stack-end edge.)

### Tests for User Story 1 (REQUIRED) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation.**

- [ ] T015 [P] [US1] Register the new integration-test target `[[test]] name = "integration_undo_redo", path = "tests/integration/undo_redo.rs"` in `Cargo.toml`
- [ ] T016 [P] [US1] Write failing integration test US1-A1 (empty doc, type a/b/c, Undo ×3 → empty) via `EditingSession::open()` + `handle_command()` in `tests/integration/undo_redo.rs`
- [ ] T017 [P] [US1] Write failing integration test US1-A2 (seed "Hallo", Backspace last char, Undo → "Hallo" restored incl. cursor) in `tests/integration/undo_redo.rs`
- [ ] T018 [P] [US1] Write failing integration test US1-A3 (k-fold Undo equals state after the (n−k)th edit) in `tests/integration/undo_redo.rs`
- [ ] T019 [P] [US1] Write failing integration test for Undo at empty stack as a complete no-op (text, cursor, dirty flag, status all unchanged) in `tests/integration/undo_redo.rs`

### Implementation for User Story 1

- [ ] T020 [US1] Add `EditorCommand::Undo` and `EditorCommand::Redo` variants to the enum in `src/editor/input.rs`
- [ ] T021 [US1] Add `Ctrl-Z → EditorCommand::Undo` (and placeholder `Ctrl-Y → EditorCommand::Redo`) arms to `map_key_event` in `src/editor/input.rs`, placed before the bare-printable `Char(c)` catch-all so the existing `!CONTROL` guard prevents aliasing to `InsertChar('z')`
- [ ] T022 [US1] Implement the record seam in `insert_text` (`src/app.rs`): before mutating the rope, capture the pre-insert cursor index; after `buffer::insert_text` succeeds, call `self.history.record(EditStep::Insert { index, text })` and on `oldest_dropped == true` set `StatusMessage::warning("History truncated to free memory")` instead of the default info
- [ ] T023 [US1] Implement the record seam in `backspace` (`src/app.rs`): when a char will be removed, read the removed char from `self.document.text` at `char_index - 1` *before* calling `buffer::remove_char_before`; after it succeeds, `record(EditStep::Delete { index: next_index, text: <removed char> })` with the OOM warning handling
- [ ] T024 [US1] Implement the record seam in `delete` (`src/app.rs`): read the char at `char_index` before calling `buffer::delete_char_at`; after it succeeds, `record(EditStep::Delete { index: char_index, text: <removed char> })` with the OOM warning handling
- [ ] T025 [US1] Implement `EditingSession::undo(&mut self)` helper in `src/app.rs`: call `self.history.undo(&mut self.document.text)`; on `Some(idx)` set cursor index + recompute `preferred_column` via `cursor::visual_column`, `mark_dirty`, set `StatusMessage::info("Undo")`, `sync_viewport`; on `None` do nothing
- [ ] T026 [US1] Dispatch `EditorCommand::Undo => self.undo()` in `handle_editing_command` in `src/app.rs` (editing mode only — the existing prompt/search precedence already gates it per FR-009)
- [ ] T027 [US1] Run `cargo test --test integration_undo_redo`: confirm US1 tests (T016–T019) pass
- [ ] T028 [US1] Review readability and maintainability of `src/editor/history.rs` and the touched seams in `src/app.rs`; simplify names, responsibilities, and interfaces where needed (Readability/Maintainability gates)
- [ ] T029 [US1] Add or verify safeguards for US1 failure paths: read-only mode interaction (stacks stay empty so Undo is inert), no corruption of rope on empty-stack Undo (Security gate)

**Checkpoint**: User Story 1 fully functional and independently testable — Ctrl-Z works end-to-end.

---

## Phase 4: User Story 2 — Änderungen wiederherstellen (Priority: P2)

**Goal**: Ctrl-Y performs one-step Redo of the last undone change; repeated Ctrl-Y walks forward to the latest state; Redo at the empty stack is a no-op. Satisfies FR-002, FR-005 (per-step), FR-010, FR-012, and US2 acceptance scenarios. Builds directly on the `History::redo` API from Phase 2.

**Independent Test**: Make several edits, undo some/all steps, then press Ctrl-Y repeatedly and confirm the text and cursor return step-by-step to the latest edited state; further Ctrl-Y is a no-op. (See `quickstart.md` → US2-A1/A2/A3.)

### Tests for User Story 2 (REQUIRED) ⚠️

- [ ] T030 [P] [US2] Write failing integration test US2-A1 (3 undos then 3 redos → final edited state) in `tests/integration/undo_redo.rs`
- [ ] T031 [P] [US2] Write failing integration test US2-A2 (undo to origin, Redo once → state after first edit) in `tests/integration/undo_redo.rs`
- [ ] T032 [P] [US2] Write failing integration test US2-A3 (Redo at empty redo stack is a no-op: text/cursor/dirty/status unchanged) in `tests/integration/undo_redo.rs`
- [ ] T033 [P] [US2] Write failing integration test for FR-012 determinism (repeated undo to None then repeated redo to None restores byte-identical content and final cursor) in `tests/integration/undo_redo.rs`

### Implementation for User Story 2

- [ ] T034 [US2] Implement `EditingSession::redo(&mut self)` helper in `src/app.rs`: call `self.history.redo(&mut self.document.text)`; on `Some(idx)` set cursor index + recompute `preferred_column`, `mark_dirty`, `StatusMessage::info("Redo")`, `sync_viewport`; on `None` do nothing
- [ ] T035 [US2] Dispatch `EditorCommand::Redo => self.redo()` in `handle_editing_command` in `src/app.rs` (the `Ctrl-Y` binding was added in T021)
- [ ] T036 [US2] Run `cargo test --test integration_undo_redo`: confirm US2 tests (T030–T033) pass
- [ ] T037 [US2] Verify no new files beyond the existing `history.rs` and `app.rs` seams were touched by Redo (Maintainability gate: Redo reuses the `History` API without duplication)

**Checkpoint**: User Stories 1 AND 2 both work independently — full Undo/Redo navigation works.

---

## Phase 5: User Story 3 — Redo wird bei neuer Änderung geleert (Priority: P2)

**Goal**: After any Ctrl-Z, a new text edit clears the redo stack so the previously-undone steps are unreachable via Ctrl-Y; only the new edit is undoable. Satisfies FR-007, FR-011 (new edit records one step), and US3 acceptance scenarios.

**Independent Test**: Type "a","b", Undo once (only "a"), type "x" (text "ax"), then Ctrl-Y — confirm Redo is a no-op (redo stack empty). Repeat with multiple undos before the new edit. (See `quickstart.md` → US3-A1/A2.)

> **Note**: The core `History::record` behaviour (clear redo on push) is already implemented in Phase 2 (T006); this phase adds the **tests proving the session-level contract** and any wiring needed to expose it.

### Tests for User Story 3 (REQUIRED) ⚠️

- [ ] T038 [P] [US3] Write failing integration test US3-A1 (type "a","b", Undo, InsertChar 'x' → "ax", Redo is a no-op, `history.redo` empty) in `tests/integration/undo_redo.rs`
- [ ] T039 [P] [US3] Write failing integration test US3-A2 (several undos, new edit, Redo no-op; only the new edit is undoable via Ctrl-Z) in `tests/integration/undo_redo.rs`

### Implementation for User Story 3

- [ ] T040 [US3] Verify the existing record seams from T022–T024 already clear `history.redo` via `History::record`; if any mutation path bypasses `record` (e.g. a future `replace_range` caller), route it through `record` too, in `src/app.rs`
- [ ] T041 [US3] Run `cargo test --test integration_undo_redo`: confirm US3 tests (T038–T039) pass
- [ ] T042 [US3] Add a direct unit assertion in `tests/unit/history.rs` that `record` leaves `history.redo` empty after a prior `undo` populated it, locking the invariant at the type level

**Checkpoint**: User Stories 1, 2, AND 3 all work independently — Redo-cleared-on-edit rule enforced.

---

## Phase 6: User Story 4 — Ein Leben pro Anwendungssitzung (Priority: P3)

**Goal**: Undo/redo history is bound to the session: closing the application discards both stacks; reopening starts empty, and Ctrl-Z/Ctrl-Y have no effect until new edits are made. Satisfies FR-008 and US4 acceptance scenarios.

**Independent Test**: Build a history in one session, drop the session, `open()` the same file as a fresh session, and confirm `history.undo` and `history.redo` are empty and Ctrl-Z/Ctrl-Y are no-ops. (See `quickstart.md` → US4-A1/A2.)

> **Note**: Session binding is automatic via `History::new()` in `EditingSession::new()` (T013); this phase adds session-level tests and confirms nothing persists history to disk.

### Tests for User Story 4 (REQUIRED) ⚠️

- [ ] T043 [P] [US4] Write failing integration test US4-A1 (build history in session A, drop it, `open()` same file fresh → empty undo+redo, Ctrl-Z/Ctrl-Y no-op) in `tests/integration/undo_redo.rs`
- [ ] T044 [P] [US4] Write failing integration test US4-A2 (save document, end session, reopen → only future edits are undoable) in `tests/integration/undo_redo.rs`

### Implementation for User Story 4

- [ ] T045 [US4] Confirm `EditingSession::new()` and `open()` initialize `history: History::new()` (empty) and that no code path persists history to disk; add a doc comment on the `history` field stating the session-bound, no-persistence invariant in `src/app.rs`
- [ ] T046 [US4] Run `cargo test --test integration_undo_redo`: confirm US4 tests (T043–T044) pass

**Checkpoint**: All four user stories independently functional.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Edge cases spanning multiple stories, save interaction, full regression, and final review against the constitution.

- [ ] T047 [P] Write integration test for Unicode + newline steps (multibyte char insert, Enter, Backspace, Undo/Redo → byte-identical restoration incl. cursor) in `tests/integration/undo_redo.rs`
- [ ] T048 [P] Write integration test for a large single insert step (one big string insert, Undo, Redo → single step, exact restore, no stutter) in `tests/integration/undo_redo.rs`
- [ ] T049 [P] Write integration test for FR-009 mode gating: enter `SearchInput`, send Ctrl-Z/Ctrl-Y → ignored (buffer, query, history unchanged); enter the ConfirmQuit / SaveConflictPrompt, send Ctrl-Z/Ctrl-Y → ignored, in `tests/integration/undo_redo.rs`
- [ ] T050 [P] Write integration test for FR-013: build history, Save, then Undo still restores pre-save edits (`history.undo` non-empty after save, save emitted no step) in `tests/integration/undo_redo.rs`
- [ ] T051 [P] Write integration test for FR-006/SC-007 memory pressure at the session level: inject a `History::with_capacity(2)` via a test-only constructor on `EditingSession` (or construct session then replace `history`), apply 3 edits, and confirm the 3rd edit's `record` evicted the oldest step and the status became the "History truncated to free memory" warning while the edit was still applied and the rope intact — in `tests/integration/undo_redo.rs`
- [ ] T052 Add a test-only constructor or accessor on `EditingSession` to inject a capped `History` for T051 (if not already present) in `src/app.rs`, keeping it clearly scoped as a test hook with a doc comment (not a product cap — aligns with FR-004)
- [ ] T053 Run the full `cargo test` suite and confirm all pre-existing tests (`integration_open_and_save`, `unsaved_guards`, `readonly_and_conflict`, `search_and_resize`, `enter_newline`, `unit_buffer`, `unit_cursor`, `unit_search`, `unit_render`) plus `unit_history` and `integration_undo_redo` are green
- [ ] T054 [P] Run `cargo clippy --all-targets -- -D warnings` and resolve any new lint findings; update `architecture.md` to mention the `editor/history.rs` module, the `history` field on `EditingSession`, and the Ctrl-Z/Ctrl-Y bindings, in `architecture.md`
- [ ] T055 Run the `quickstart.md` validation guide end-to-end (automated blocks + manual TUI checks 1–5, 7, 8) and confirm every checklist item passes
- [ ] T056 Run a final readability, maintainability, security, verification, and scope review against the constitution (`.specify/memory/constitution.md`) and the plan's Constitution Check; record any exception in the plan's Complexity Tracking table if a gate cannot pass

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately.
- **Foundational (Phase 2)**: Depends on Setup; **BLOCKS** all user stories (the `History` type and `history` field are prerequisites for every story's record/undo/redo seams).
- **User Stories (Phase 3–6)**: All depend on Foundational completion.
  - US1 (Phase 3) is the MVP and also wires the input bindings + record seams that US2/US3 reuse.
  - US2 (Phase 4) dispatches `Redo` — depends on the `Ctrl-Y` binding added in T021 (within US1) and the shared command enum, but its own logic is independent.
  - US3 (Phase 5) verifies the clear-redo-on-record invariant already built in T006.
  - US4 (Phase 6) verifies session-lifetime invariants already established in T013.
  - Stories may proceed sequentially in priority order (recommended for this feature given shared seams) or, with care, US2/US3/US4 tests can be drafted in parallel after Phase 3.
- **Polish (Phase 7)**: Depends on all four user stories being complete.

### User Story Dependencies

- **User Story 1 (P1)**: Starts after Foundational. No dependencies on other stories. Provides the input bindings + record seams used by the rest.
- **User Story 2 (P2)**: Depends on US1's `Redo` command variant + `Ctrl-Y` binding (T020/T021). Independently testable once those exist.
- **User Story 3 (P2)**: Depends on US1's record seams (T022–T024); tests prove the Phase-2 `record` clear-redo behaviour at session level. Independently testable.
- **User Story 4 (P3)**: Depends on the foundational `History::new()` initialization (T013). Independently testable.

### Within Each User Story

- Automated tests for the story's primary flow and relevant edge cases are written first and must FAIL before implementation lands (constitution Verification gate).
- Pure `History` logic precedes session wiring.
- Session record seams precede Undo/Redo dispatch.
- Story complete and tested before the next priority.

### Parallel Opportunities

- Phase 1 Setup tasks marked [P]: none distinct (T001 serial, T002 builds on it).
- Phase 2: T003, T004, T009, T010, T011 (different files / independent units) can be drafted in parallel; T005–T008 form the `History` logic chain and are sequential.
- Per story, all test tasks marked [P] target the same file (`integration_undo_redo.rs` or `unit/history.rs`) — `[P]` here indicates they are logically independent *test cases* and can be authored together, though they touch one file so commit serially.
- US1 implementation tasks T022–T024 (the three record seams) are in different locations within `src/app.rs`; draft together but apply as one coherent change to avoid conflicts.
- Polish edge-case tests T047–T051 are independent test cases and can be authored together.

---

## Parallel Example: Foundational Phase

```bash
# Launch independent foundational pieces together:
Task: "Implement EditStep enum + derived helpers in src/editor/history.rs"
Task: "Implement RecordOutcome struct in src/editor/history.rs"
Task: "Implement can_undo/can_redo/clear helpers in src/editor/history.rs"
Task: "Register unit_history test target in Cargo.toml"
Task: "Write failing History invariant unit tests in tests/unit/history.rs"
```

## Parallel Example: User Story 1

```bash
# Launch all US1 test cases together (author together, commit serially, same file):
Task: "US1-A1 empty doc undo-back-to-empty test in tests/integration/undo_redo.rs"
Task: "US1-A2 delete-last-char undo test in tests/integration/undo_redo.rs"
Task: "US1-A3 k-fold undo test in tests/integration/undo_redo.rs"
Task: "Undo at empty stack no-op test in tests/integration/undo_redo.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (green baseline + module stub).
2. Complete Phase 2: Foundational (`History` type + `unit_history` green; `history` field on session).
3. Complete Phase 3: User Story 1 (record seams + `Undo` dispatch + `Ctrl-Z`; `integration_undo_redo` US1 cases green).
4. **STOP and VALIDATE**: Test User Story 1 independently — Ctrl-Z walks the undo stack back to the origin and is a no-op at the bottom.
5. Demo/ship if ready: a single binary with usable Undo is already valuable.

### Incremental Delivery

1. Setup + Foundational → Foundation ready.
2. Add User Story 1 → test independently → **MVP** (Undo works).
3. Add User Story 2 → test independently → Redo works; full back/forward navigation.
4. Add User Story 3 → test independently → Redo cleared on new edit (consistency guarantee).
5. Add User Story 4 → test independently → session-lifetime behaviour confirmed.
6. Polish phase → cross-cutting edge cases (Unicode, large insert, mode gating, save, OOM) + final review.

### Parallel Team Strategy

This feature is small and shares a few seams (`input.rs`, `record seams in app.rs`); sequential delivery in priority order is recommended. If parallel capacity exists:
- One developer owns Phase 2 (`history.rs` + `unit_history`).
- After Foundational, a second developer can draft US2/US3/US4 test cases while the first wires US1 seams.
- Coordinating the shared `src/app.rs` record seams and the shared `tests/integration/undo_redo.rs` file remains serial to avoid merge conflicts.

---

## Notes

- [P] tasks = different files, no dependencies (when a [P] task touches the same file as another, the [P] marks logical independence of the *test case* or *unit*; apply serially).
- [Story] label maps task to a specific user story for traceability.
- Every user story is independently completable and testable through the `EditingSession` interface per `architecture.md`'s testing convention.
- Per the constitution Verification gate, tests MUST be written first and FAIL before implementation.
- Commit after each task or logical group; run `cargo test` at every checkpoint.
- The `undo_capacity` test hook (T005/T052) is a test-only injection point defaulting to `usize::MAX` and is NOT a product cap — this preserves FR-004/SC-006 (no artificial history limit).
