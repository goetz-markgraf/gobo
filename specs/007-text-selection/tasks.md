# Tasks: Text Selection

**Input**: Design documents from `/specs/007-text-selection/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Automated test tasks are REQUIRED. Every user story and every in-scope feature must include tasks for the primary flow and relevant edge cases, with extra coverage for safeguards and regression-prone behavior. The feature spec (FR-014 / SC-005) and the Gobo constitution (IV, Development Workflow) mandate automated coverage for the main flows and edge cases.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story. The feature touches only existing Rust modules (`cursor.rs`, `history.rs`, `app.rs`, `input.rs`, `render.rs`, `main.rs`) plus test targets under `tests/unit/` and `tests/integration/`, per plan.md. No new crates, no new modules.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: Which user story this task belongs to (e.g. US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- Single Rust project: `src/` and `tests/` at repository root (per plan.md → Project Structure).
- Editor modules live under `src/editor/`; the session lives in `src/app.rs`; ratatui assembly in `src/main.rs`.
- Unit tests live under `tests/unit/` (standalone `[[test]]` targets), integration tests under `tests/integration/` (standalone `[[test]]` targets), each declared in `Cargo.toml`.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Wire up the new test target and confirm the build is green before any feature work.

- [X] T001 Add new `[[test]]` target `integration_text_selection` with path `tests/integration/text_selection.rs` in Cargo.toml
- [X] T002 Create empty placeholder file `tests/integration/text_selection.rs` with a trivial `#[test]` that does nothing yet, so `cargo test --no-run` compiles all targets (including the new one)
- [X] T003 Create feature branch `007-text-selection` and confirm `cargo build` and `cargo test` are green on the untouched baseline (document any pre-existing failures in a note, do not fix unrelated work here)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The shared data entities and pure geometry that EVERY user story depends on. These MUST be complete before any user story phase begins.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T004 [P] Define `Selection { anchor: usize; head: usize }` struct with `Clone, Copy, Debug, PartialEq, Eq, Default` derives plus `range()`, `is_empty()`, `is_forward()` methods in `src/editor/cursor.rs` per data-model.md Entity 1 and contracts/api.md §1
- [X] T005 [P] Define `EditStep::Replace { index: usize, removed: String, inserted: String }` variant in `src/editor/history.rs` and extend the existing `EditStep` methods (`apply_forward`, `apply_reverse`, cursor helpers, `len_chars`/`end_index` accessors) for the new variant per data-model.md Entity 2 and contracts/api.md §2
- [X] T006 [P] Add `pub selection: Option<Selection>` field to `EditingSession` in `src/app.rs`, initialized to `None` in `new()`/`open()`, and re-export `Selection` through `src/editor/mod.rs` per data-model.md Entity 3 and contracts/api.md §3
- [X] T007 [P] Add unit tests for `Selection::range()` / `is_empty()` / `is_forward()` geometry (forward/backward, empty, cross-anchor) in `tests/unit/cursor.rs`
- [X] T008 [P] Add unit tests for `EditStep::Replace` apply_forward/apply_reverse symmetry, `before_cursor`/`after_cursor` (empty `inserted` → cursor at `index`; non-empty → `index + inserted.chars().count()`), and clear-redo-on-record in `tests/unit/history.rs`
- [X] T009 [P] Add the four `MoveSelectLeft`, `MoveSelectRight`, `MoveSelectUp`, `MoveSelectDown` `EditorCommand` variants in `src/editor/input.rs` per contracts/api.md §4 (dispatch wiring comes per-story)
- [X] T010 Document, in a short comment block at the top of `src/editor/cursor.rs` and `src/editor/history.rs`, the module-boundary contract for selection state (cursor owns `Selection`/`MoveSelect*`; history owns `EditStep::Replace`) and the FR-013 separation of responsibilities, anchoring to constitution I/II (foundational readability/boundary note)

**Checkpoint**: Foundation ready — `Selection` type, `EditStep::Replace`, `EditingSession.selection`, the new command variants, and their unit tests exist and compile. User story implementation can now begin in priority order.

---

## Phase 3: User Story 1 — Text markieren (Priority: P1) 🎯 MVP

**Goal**: A user holds Shift and moves the cursor with arrows; the range between a fixed anchor and the moving head is highlighted and grows/shrinks, including backward over the anchor. (FR-001, FR-002, FR-003, FR-010)

**Independent Test**: Open a document with known text, send `Shift+Arrow` sequences through `EditingSession::open()` + `handle_command()`, and assert the resulting `selection.range()` (and rendered highlight spans) equal `[anchor, head)` after each move, including direction flip and document-boundary clamp.

### Tests for User Story 1 (REQUIRED) ⚠️

> **NOTE**: Write these tests FIRST, ensure they FAIL before implementation.

- [X] T011 [P] [US1] Unit tests for the four `move_select_*` functions in `src/editor/cursor.rs`: seeding anchor on first move, head moves via reused motion, anchor stays fixed, direction flip when head crosses anchor, empty selection at start — in `tests/unit/cursor.rs`
- [X] T012 [P] [US1] Unit tests for `move_select_*` document-boundary clamping (Shift+Left at index 0 stays 0; Shift+Right at end clamps; Shift+Up at top line stays; Shift+Down at last line clamps) honoring `preferred_column` — in `tests/unit/cursor.rs`
- [X] T013 [P] [US1] Unit test for `render_view` selection highlight projection: setup a selection spanning the visible text, assert each `BodyLine.highlights` carries the correct `HighlightSpan` column ranges (clipped to viewport), and empty when no intersection — in `tests/unit/render.rs`
- [X] T014 [P] [US1] Integration test for build/grow/shrink/reverse selection via `MoveSelect{Left,Right,Up,Down}` through `EditingSession` against text "Hallo" and a two-line document — in `tests/integration/text_selection.rs`
- [X] T015 [P] [US1] Integration edge-case test for document-boundary clamp (FR-003) across doc start/end via Shift+arrows — in `tests/integration/text_selection.rs`

### Implementation for User Story 1

- [X] T016 [US1] Implement `move_select_left`, `move_select_right`, `move_select_up`, `move_select_down` in `src/editor/cursor.rs`: seed `anchor = cursor.char_index` when `selection.is_none()`, move head by reusing the existing `move_left/right/up/down`, write the new head back to `cursor` and `selection.head`, recompute `preferred_column` — per contracts/api.md §1 and research.md R5
- [X] T017 [US1] Extend `RenderView` and `render_view` in `src/editor/render.rs`: change `body_lines: Vec<String>` to `Vec<BodyLine>` with `BodyLine { text, highlights: Vec<HighlightSpan> }`, compute per-line intersection of `selection.range()` with the line's char range, map to visual columns via existing grapheme-aware math, clip to the visible viewport; keep all other `RenderView` fields unchanged in formula — per contracts/api.md §5 and research.md R7
- [X] T018 [US1] Wire `MoveSelect{Left,Right,Up,Down}` arms into `map_key_event` in `src/editor/input.rs` for `(KeyModifiers::SHIFT, KeyCode::Left/Right/Up/Down)`, ordered BEFORE the existing wildcard-modifier arrow arms so Shift+arrows are not swallowed as plain moves — per contracts/api.md §4 and research.md R4
- [X] T019 [US1] Dispatch the four `MoveSelect*` commands from `handle_editing_command` in `src/app.rs`: call the corresponding `move_select_*` with `&mut self.selection`, `&mut self.cursor`, `&self.document.text`; no history recording — per contracts/api.md §3
- [X] T020 [US1] Apply `Style::default().reversed()` to each `HighlightSpan` in `src/main.rs::draw()` for the body lines; no styling logic leaves `render.rs` (boundary) — per contracts/api.md §5 and research.md R7
- [X] T021 [US1] Update the existing `tests/unit/render.rs` assertions for the `body_lines` type change from `Vec<String>` to `Vec<BodyLine>` so the baseline render test stays green (highlights empty when no selection) — per quickstart.md §3 "no regressions"
- [X] T022 [US1] Review readability of touched files (`cursor.rs`, `render.rs`, `input.rs`, `app.rs`, `main.rs`): confirm names reveal intent, control flow is straightforward, no hidden state (constitution I)
- [X] T023 [US1] Verify safeguards for US1: selection indices clamped to document bounds (FR-003), no grapheme-split (FR-012), read-only mode still allows selection building/collapsing without mutation

**Checkpoint**: User Story 1 is fully functional — Shift+arrows build/shrink/reverse a highlighted selection within document bounds. Core `cargo test` for the MVP is green.

---

## Phase 4: User Story 2 — Selektion zurücknehmen (Priority: P2)

**Goal**: A plain (non-Shift) arrow move collapses an existing selection immediately; the cursor lands at the motion's natural position and nothing stays highlighted. (FR-004) Also covers FR-011 (non-editing commands preserve selection), since US2 is the natural place to assert the collapse/preserve boundary.

**Independent Test**: Build a selection via Shift+arrows, send a plain arrow, and assert `selection == None` and the cursor at the motion-defined position; then assert Search/FindNext/Save leave `selection` unchanged.

### Tests for User Story 2 (REQUIRED) ⚠️

- [X] T024 [P] [US2] Integration tests for collapse-on-plain-move via `Move{Left,Right,Up,Down}` after a built selection in `tests/integration/text_selection.rs`: assert `selection == None` and cursor at the expected post-move position, including Up/Down preserving `preferred_column`
- [X] T025 [P] [US2] Integration test for FR-011 selection preservation: drive `Search`/`FindNext`/`Save`/`Resize` with an active selection and assert `selection` is unchanged afterward — in `tests/integration/text_selection.rs`

### Implementation for User Story 2

- [X] T026 [US2] In `handle_editing_command` (`src/app.rs`), clear `selection = None` at the head of the existing `Move{Left,Right,Up,Down}` arms BEFORE performing the motion, so the plain move still uses the existing code path — per contracts/api.md §3 and research.md R6
- [X] T027 [US2] In `handle_editing_command` (`src/app.rs`), ensure `Search`, `FindNext`, `Save`, `Resize`, and prompt-mode entries do NOT touch `self.selection` (preserve it) — per contracts/api.md §3 and FR-011; verify Undo/Redo arms clear `selection` per FR-015 (see US3/US4)
- [X] T028 [US2] Verify safeguard for US2: collapse happens before mutation so no edit path can observe a stale selection; read-only mode still collapses cleanly

**Checkpoint**: User Stories 1 AND 2 work independently — selections can be built and collapsed; non-editing commands preserve them.

---

## Phase 5: User Story 3 — Selektion durch Eingabe ersetzen (Priority: P3)

**Goal**: Typing one or more chars (or Enter) over a non-empty selection removes the selected text and inserts the typed text as one atomic undo step; one Ctrl-Z restores the original. (FR-005, FR-007, FR-009)

**Independent Test**: Build a selection over known text, send `InsertChar`/`Enter`, assert the rope equals the replaced text, `selection == None`, cursor after inserted text, exactly one `Replace` step recorded, `redo` cleared; then Ctrl-Z restores the original rope in one step.

### Tests for User Story 3 (REQUIRED) ⚠️

- [X] T029 [P] [US3] Integration test for replace-by-typing a single char over "llo" in "Hallo" → "Hax", selection cleared, cursor after inserted char, in `tests/integration/text_selection.rs`
- [X] T030 [P] [US3] Integration test for multi-char replace over a multi-line selection → single atomic `Replace` step, Ctrl-Z restores original in ONE step (FR-007/FR-009), Ctrl-Y re-applies — in `tests/integration/text_selection.rs`
- [X] T031 [P] [US3] Integration test for Undo/Redo round-trip around a replace: build selection, replace, Ctrl-Z restores rope+cursor and clears selection (FR-015), Ctrl-Y re-applies and clears selection — in `tests/integration/text_selection.rs`
- [X] T032 [P] [US3] Edge-case test: with `selection == None` (and `Some` with `is_empty()`), `InsertChar`/`Enter` record the normal `Insert` step and do not record a `Replace` (FR-008) — in `tests/integration/text_selection.rs`
- [X] T033 [P] [US3] Edge-case test: read-only document blocks replace-by-typing with a "Read-only: edits are blocked" status, leaving rope/cursor/selection/history unchanged (constitution III) — in `tests/integration/text_selection.rs`

### Implementation for User Story 3

- [X] T034 [US3] Implement the atomic replace path for `InsertChar` and `Enter` in `handle_editing_command` (`src/app.rs`): when `selection.is_some()` and non-empty and not read-only, compute `start = anchor.min(head)`, `end = anchor.max(head)`, `removed = text.slice(start..end).to_string()`, `inserted = c.to_string()` (or `"\n"` for `Enter`), perform a single `EditStep::Replace { index: start, removed, inserted }` via `history.record`, mutate the rope with one `replace_range`/remove-then-insert, set `cursor.char_index = start + inserted.chars().count()`, set `selection = None`, mark `document.dirty = true` — per contracts/api.md §3 atomic-edit postcondition and data-model.md Entity 2
- [X] T035 [US3] Add the read-only guard for the `InsertChar`/`Enter` selection branch: when `document.is_read_only()`, early-return with `status = Some(StatusMessage::warning("Read-only: edits are blocked"))` and leave rope/selection/cursor/history unchanged — per contracts/api.md §3 and research.md R10 (constitution III)
- [X] T036 [US3] Verify safeguard for US3: the Replace step is recorded exactly once (FR-007 atomicity), `redo` is cleared by `record`, and `removed` is non-empty so FR-008 fallthrough (no selection → existing `Insert`) is preserved
- [X] T037 [US3] Review readability/maintainability of the new atomic branch in `app.rs`: mirror the existing `insert_text`/`backspace`/`delete` seam style; one readable place for selection-aware edits (constitution I/II)

**Checkpoint**: User Story 3 delivered — replace-by-typing is atomic and undoable in one step, including multi-line and read-only cases.

---

## Phase 6: User Story 4 — Selektion löschen (Priority: P4)

**Goal**: DEL or BACKSPACE over a non-empty selection removes the whole selected range (including intervening newlines) as one atomic undo step and lands the cursor at the selection start; DEL and BACKSPACE behave identically. (FR-006, FR-007, FR-008, FR-009)

**Independent Test**: Build a selection, send `Delete`/`Backspace`, assert the selected range is gone, cursor at the selection start, `selection == None`, one `Replace` step with empty `inserted`; Ctrl-Z restores in one step; no selection → existing single-char delete (FR-008).

### Tests for User Story 4 (REQUIRED) ⚠️

- [X] T038 [P] [US4] Integration test for `Delete` over "llo" in "Hallo" → "Ha", cursor after "a", selection cleared, in `tests/integration/text_selection.rs`
- [X] T039 [P] [US4] Integration test for `Backspace` over the same selection → identical result to `Delete` (FR-006 same effect), cursor at selection start — in `tests/integration/text_selection.rs`
- [X] T040 [P] [US4] Integration test for multi-line selection `Delete`/`Backspace` removing all affected lines plus intervening newlines (FR-009), one atomic Ctrl-Z restore — in `tests/integration/text_selection.rs`
- [X] T041 [P] [US4] Edge-case test: with `selection == None` (and `Some` with `is_empty()`), `Delete`/`Backspace` record the normal single-char `Delete` step and delete the right/left char respectively (FR-008) — in `tests/integration/text_selection.rs`
- [X] T042 [P] [US4] Edge-case test: read-only document blocks delete-selection with "Read-only: edits are blocked", leaving all state unchanged (constitution III) — in `tests/integration/text_selection.rs`
- [X] T043 [P] [US4] Edge-case test: selection consisting only of newlines / a whole empty line is removed correctly and restored verbatim on Ctrl-Z (FR-012) — in `tests/integration/text_selection.rs`

### Implementation for User Story 4

- [X] T044 [US4] Implement the atomic delete-selection path for `Backspace` and `Delete` in `handle_editing_command` (`src/app.rs`): when a non-empty selection exists and not read-only, compute `start = anchor.min(head)`, `end = anchor.max(head)`, perform a single `EditStep::Replace { index: start, removed: <selection text>, inserted: String::new() }` via `history.record`, remove `[start, end)` from the rope, set `cursor.char_index = start`, set `selection = None`, mark dirty; both `Backspace` and `Delete` route to this identical path — per contracts/api.md §3 and FR-006
- [X] T045 [US4] Add the read-only guard for the `Backspace`/`Delete` selection branch, mirroring the US3 guard (constitution III)
- [X] T046 [US4] Verify safeguard for US4: exactly one `Replace` step per delete (FR-007), `removed` non-empty ensuring FR-008 fallthrough preserves existing single-char `Delete` behavior; multi-line/CRLF/newline-only deletion restored byte-for-byte (FR-009/FR-012)

**Checkpoint**: All four user stories are independently functional. `cargo test` is green end-to-end.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Cross-story edge coverage, Unicode/CRLF correctness, regression safety, and documentation alignment.

- [X] T047 [P] Cross-story edge-case test: selection over a CRLF line ending (`\r\n`) is removed and restored as a char pair, no halved cluster (FR-012) — in `tests/integration/text_selection.rs`
- [X] T048 [P] Cross-story edge-case test: selection over a multi-grapheme cluster (e.g. base char + combining mark) is removed/restored as whole chars, no partial cluster (FR-012) — in `tests/integration/text_selection.rs`
- [X] T049 [P] Cross-story edge-case test: selection at exact document start and exact document end boundaries clamp and edit safely with no out-of-range removes (FR-003/FR-012) — in `tests/integration/text_selection.rs`
- [X] T050 Run `cargo test --no-run` to confirm every `[[test]]` target (including the new `integration_text_selection`) compiles, then run full `cargo test` to confirm no regressions vs the baseline documented in T003
- [X] T051 Run the manual terminal UX validation steps from `specs/007-text-selection/quickstart.md` (visual inverse-color highlight, shrink, collapse, replace, delete, multi-line delete, search-preserves-selection) and record results; this supplements but does not replace the automated suite (constitution IV)
- [X] T052 [P] Update `architecture.md` to note the new `Selection` type in `cursor.rs`, the `EditStep::Replace` variant in `history.rs`, the `selection` field on `EditingSession`, and the `RenderView.body_lines`/`BodyLine`/`HighlightSpan` shape, keeping the module map current (operational constraint: architecture documented near entry points)
- [X] T053 Run a final readability, maintainability, security, and scope review against the Gobo constitution (I–V): confirm clear boundaries (selection state in `cursor`/`app`, mutation in `buffer`, history in `history`, render pure projection in `render`, input mapping isolated in `input`, styling in `main`), no new dependencies, no network/telemetry, destructive deletion guarded by atomic undo (constitution III)
- [X] T054 Confirm `SaveConflictPrompt → Reload` clears `selection` as part of the cursor reset (FR-011 enumerated trigger), and `Overwrite`/`Cancel` leave `selection` as-is per contracts/api.md §3 (cross-cutting session-state correctness)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately. T001→T002 (file referenced by Cargo must exist), T003 independent.
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories. All T004–T010 touch disjoint files and are [P] parallelizable; T010 may follow the type definitions.
- **User Stories (Phase 3–6)**: All depend on Foundational completion.
  - US1 (Phase 3) is the MVP and the prerequisite for the visible/acting selection that US2–US4 build on conceptually (collapse, replace, delete).
  - US2 (Phase 4) depends on US1 (collapse acts on a selection built by US1).
  - US3 (Phase 5) depends on US1 (replace acts on a selection) and the `EditStep::Replace` from Foundational.
  - US4 (Phase 6) depends on US1 and `EditStep::Replace` from Foundational; US3 and US4 are independent of each other and could run in parallel once US1 + Foundational are done.
- **Polish (Phase 7)**: Depends on all user stories being complete (T050–T054).

### Within Each User Story

- Automated tests for the story's primary flow and relevant edge cases are written FIRST and fail before implementation (constitution IV).
- Pure/unit seams before dispatch wiring (e.g. `move_select_*` before `app.rs` dispatch).
- `EditStep::Replace` (Foundational) before the atomic edit branches (US3/US4).
- Each story is independently testable before moving to the next priority.

### Parallel Opportunities

- Phase 2 (Foundational): T004–T010 all touch different files (cursor.rs, history.rs, app.rs, input.rs + four test files) → fully parallel.
- US1 tests (T011–T015) all in unit/integration test files → parallel where they don't edit the same file region; in practice batch per test file.
- US3 and US4 are independent of each other (different `EditorCommand` arms) → can be developed in parallel by two developers once US1 + Foundational are done.
- Cross-story Polish edge cases (T047–T049) touch one test file but assert disjoint scenarios → can be authored together.

### Parallel Example: User Story 1

```bash
# Unit tests (different files within tests/unit/) and integration tests:
Task: "Unit tests for move_select_* in tests/unit/cursor.rs"          # T011, T012
Task: "Unit test for render highlight projection in tests/unit/render.rs"  # T013
Task: "Integration build/shrink/reverse + boundary clamp in tests/integration/text_selection.rs"  # T014, T015

# Pure implementation seams (disjoint files) once tests fail:
Task: "Implement move_select_* in src/editor/cursor.rs"               # T016
Task: "Extend RenderView/highlights in src/editor/render.rs"          # T017
Task: "Add MoveSelect* map_key_event arms in src/editor/input.rs"    # T018
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T003).
2. Complete Phase 2: Foundational (T004–T010) — CRITICAL, blocks all stories.
3. Complete Phase 3: User Story 1 (T011–T023).
4. **STOP and VALIDATE**: run `cargo test --test unit_cursor`, `cargo test --test unit_render`, `cargo test --test integration_text_selection` — all green; run the quickstart.md visual steps 1–4.
5. Demo the MVP: visually highlight and grow/shrink/reverse a selection.

### Incremental Delivery

1. Setup + Foundational → foundation ready.
2. US1 → test independently → demo visible selection (MVP).
3. US2 → test independently → selection collapses on plain move.
4. US3 → test independently → replace-by-typing is atomic + undoable.
5. US4 → test independently → delete-selection is atomic + undoable.
6. Polish → run full `cargo test`, manual UX, architecture doc update, constitution review.

### Parallel Team Strategy

With multiple developers after Foundational + US1:
- One developer: US2 (collapse, FR-011 preservation).
- Two developers in parallel: US3 (replace-by-typing) and US4 (delete-selection) — different command arms, both reuse `EditStep::Replace`.
- One developer: Polish edge cases.

---

## Notes

- [P] tasks = different files, no dependencies on incomplete tasks.
- [Story] label maps every user-story task to US1–US4 for traceability against spec.md.
- Tests are REQUIRED (not optional) per the feature spec FR-014/SC-005 and the Gobo constitution IV + Development Workflow; the TDD ordering (tests first, fail before implementation) is mandated by constitution IV.
- Each user story is independently completable and testable; commit after each task or logical group.
- Stop at any checkpoint to validate a story independently before moving to the next priority.
- All indices are character indices into the `Rope`; selection never splits grapheme clusters (FR-012) by reusing existing motion + grapheme-aware column math.
- `EditStep::Replace` is only recorded when `removed` is non-empty (FR-008 fallthrough preserves existing behavior for empty selections).
- Manual terminal UX validation (quickstart.md) supplements but never replaces the automated suite (constitution IV).
