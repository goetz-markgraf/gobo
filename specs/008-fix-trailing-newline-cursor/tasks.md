# Tasks: Fix Trailing Newline Cursor Position

**Input**: Design documents from `/specs/008-fix-trailing-newline-cursor/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Automated test tasks are REQUIRED. Every user story and every in-scope feature must include tasks for the primary flow and relevant edge cases, with extra coverage for safeguards and regression-prone behavior.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g. US1, US2)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm existing project state; no new project scaffolding required (single-binary Rust editor already created, no new dependencies).

- [X] T001 Verify Rust toolchain and `cargo test` baseline is green on branch `008-fix-trailing-newline-cursor` before any change (run `cargo test` from repo root)
- [X] T002 [P] Read `src/editor/buffer.rs` `line_of_char`, `cursor.rs` `visual_column`/`move_right`/`move_left`, and `render.rs` `render_view` to confirm they match the contract in `specs/008-fix-trailing-newline-cursor/contracts/cursor-line-mapping.md` (no structural assumptions drift)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Reproduce the bug with a failing test that anchors the fix; no user story can be considered complete until this regression test exists.

- [X] T003 Add a failing unit test in `tests/unit/buffer.rs` that asserts `line_of_char("abc\n", 4) == 1`, `line_of_char("abc\n\n", 5) == 2`, `line_of_char("abc\n\n\n", 6) == 3`, and `line_of_char("\n", 1) == 1` (the `[BUG]` rows from `research.md`), guarded by a `// # Fix Trailing Newline Cursor Position` marker; confirm it FAILS (`cargo test --test unit buffer`) before any implementation

**Checkpoint**: Bug reproduced by a failing test - implementation may begin

---

## Phase 3: User Story 1 - Cursor korrekt hinter abschließendem Newline platzieren (Priority: P1) 🎯 MVP

**Goal**: Fix `buffer::line_of_char` so that a cursor sitting at `len_chars()` of a document ending in `\n` maps to the empty trailing line, making the displayed cursor line match the logical insert position (FR-001, FR-002, FR-006).

**Independent Test**: Open a document ending in `\n` (e.g. `"abc\n"`), move the cursor to the document end (behind the last `\n`), and verify the cursor is drawn at column 0 of the empty trailing line; the unit test `line_of_char(..., len_chars())` returns the trailing empty line for `"abc\n"`, `"abc\n\n"`, `"abc\n\n\n"`, and `"\n"`.

### Tests for User Story 1 (REQUIRED) ⚠️

> **NOTE**: Write these tests FIRST, ensure they FAIL before implementation

- [X] T004 [P] [US1] Add unit tests in `tests/unit/buffer.rs` covering `line_of_char` end-of-doc trailing-newline cases per the corrected mapping table in `specs/008-fix-trailing-newline-cursor/data-model.md`: `"abc\n"`→1 at idx 4, `"abc\n\n"`→2 at idx 5, `"abc\n\n\n"`→3 at idx 6, `"\n"`→1 at idx 1 (FR-006)
- [X] T005 [P] [US1] Add unit tests in `tests/unit/cursor.rs` asserting `visual_column` is `0` (start of the empty trailing line) when the cursor is at `len_chars()` after a trailing `\n` for `src/editor/cursor.rs`, and that `move_right` to end-of-doc then `visual_column`/line yields the empty trailing line (FR-002, FR-003)
- [X] T006 [P] [US1] Add unit/integration test in `tests/unit/buffer.rs` (or a render unit test) verifying the cursor `(x,y)` produced via `line_of_char` for `"abc\n"` at end-of-doc is the empty trailing line, so `render::render_view` (src/editor/render.rs) draws the cursor at the correct line (FR-001)

### Implementation for User Story 1

- [X] T007 [US1] In `src/editor/buffer.rs` `line_of_char`, add the end-of-document trailing-newline branch exactly per the contract `specs/008-fix-trailing-newline-cursor/contracts/cursor-line-mapping.md`: when `len_chars() != 0` and `idx == len_chars()` and `text.char(len_chars()-1) == '\n'`, return `text.char_to_line(len_chars()-1) + 1`; include a comment explaining the ropey `\n`/end-of-document quirk (depends on T004–T006 failing tests)
- [X] T008 [US1] Confirm the fix cascades without changes to `src/editor/cursor.rs` (`visual_column`) and `src/editor/render.rs` (`render_view`) by re-running `cargo test --test unit buffer cursor`; do not modify those consumers unless a test reveals otherwise
- [X] T009 [US1] Add a pure/total and monotonicity sanity assertion in `tests/unit/buffer.rs`: `line_of_char` is non-decreasing across `0..=len_chars()` for `"abc\n"` and never panics for any in-range index (FR-005 regression safety)
- [X] T010 [US1] Review `src/editor/buffer.rs` for readability and maintainability per constitution principle I/II: keep the branch minimal, named/justified in a comment, one reason to change in `line_of_char`

**Checkpoint**: Cursor at end-of-doc after a trailing `\n` is drawn at column 0 of the empty trailing line, matching the logical insert position (MVP delivered)

---

## Phase 4: User Story 2 - Eingabe am Ende der Zeile vor dem abschließenden Newline (Priority: P2)

**Goal**: Prove no regression for the already-correct case where the cursor sits at the end of the last non-empty line (just before the trailing `\n`), and that arrow navigation over the trailing `\n` is consistent (FR-004, FR-005).

**Independent Test**: Open `"abc\n"`, place the cursor at the end of the `abc` line (index 3, on/before the `\n`), type `x`, and verify the content becomes `"abcx\n"` with the cursor after `x`; pressing right moves the cursor to the empty trailing line.

### Tests for User Story 2 (REQUIRED) ⚠️

- [X] T011 [P] [US2] Add regression unit tests in `tests/unit/buffer.rs` for the already-correct cases that MUST NOT change under this fix: `"abc"` (no newline) idx 3 → line 0; `""` (empty) idx 0 → line 0; `"abc\n"` idx 3 (on `\n`, not end) → line 0 (FR-005)
- [X] T012 [P] [US2] Add unit tests in `tests/unit/cursor.rs` for `move_left` from end-of-doc (`len_chars()`) back over the trailing `\n` landing on line 0 at the end of `abc` for `"abc\n"` (FR-004), and `move_right` from index 3 to `len_chars()` moving the cursor to the empty trailing line (FR-004)
- [X] T013 [P] [US2] Add an integration-level regression guard (or extend an existing test in `tests/integration/`) verifying an insert at the end of the `abc` line for `"abc\n"` keeps the trailing `\n` intact and yields `"abcx\n"` (FR-003, FR-007 persistence untouched)

### Implementation for User Story 2

- [X] T014 [US2] Verify existing `src/editor/cursor.rs` motions (`move_right`/`move_left`) and the insert path require no change; only confirm the corrected `line_of_char` keeps these passing (depends on T011–T013 tests)
- [X] T015 [US2] If any test in T011–T013 fails, scope the smallest possible fix strictly within `src/editor/buffer.rs` (do not alter `char_index` semantics or persistence); document any deviation in this tasks.md before implementing

**Checkpoint**: Cursor behavior before the trailing `\n` is unchanged (no regression); navigation across the trailing `\n` is consistent with US1

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Final verification that the localized fix is complete, edge cases covered, and the constitution gates pass.

- [X] T016 Run the quickstart validation from `specs/008-fix-trailing-newline-cursor/quickstart.md`: `cargo test --test unit buffer cursor render`, and confirm all primary-fix and regression-guard outcomes pass
- [X] T017 [P] Run the full suite `cargo test` from repo root to confirm existing integration tests (open/save/search/enter/undo) remain green as persistence/edit regression guards (FR-007)
- [X] T018 Final readability, maintainability, and constitution review of `src/editor/buffer.rs` change: confirm single-responsibility boundary preserved, comment explains the ropey quirk, no new dependency, no persistence change (constitution I–V)
- [X] T019 [P] Update architecture docs (`architecture.md`) only if the one-line behavior note in `buffer::line_of_char` warrants a cross-reference; otherwise note explicitly "no doc change needed"

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup - BLOCKS all user story implementation
- **User Story 1 (Phase 3)**: Depends on Foundational (failing test T003 must exist); receives the actual fix
- **User Story 2 (Phase 4)**: Depends on US1 fix being applied (its regression tests assert behavior around the corrected mapping); independently testable
- **Polish (Phase 5)**: Depends on US1 and US2 completion

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational; provides the code fix in `src/editor/buffer.rs`
- **User Story 2 (P2)**: Starts after US1 fix; its tests are regression guards that must pass against the corrected `line_of_char`. Independently testable but logically sequenced after US1.

### Within Each User Story

- Automated tests for the story's primary flow and relevant edge cases MUST be written and FAIL before implementation
- Models/contracts before services before endpoints (here: mapping function `line_of_char` is the single source of truth; its consumers inherit the fix)
- Story complete before moving to next priority

### Parallel Opportunities

- Setup task T002 can run in parallel with T001 (verification vs. read)
- US1 tests T004, T005, T006 are in two different test files (`tests/unit/buffer.rs`, `tests/unit/cursor.rs`) and can be written in parallel before the implementation task T007
- US2 tests T011–T013 are in different files and can be written in parallel once US1 fix lands
- Polish tasks T017 and T019 can run in parallel with T018 review

---

## Parallel Example: User Story 1

```bash
# Write the failing tests across different test files in parallel:
Task: "Add line_of_char end-of-doc trailing-newline unit tests in tests/unit/buffer.rs"
Task: "Add visual_column / move_right end-of-doc unit tests in tests/unit/cursor.rs"
Task: "Add render_view cursor (x,y) end-of-doc unit test in tests/unit/buffer.rs"

# Then apply the single mapped fix:
Task: "Add the end-of-doc trailing-newline branch to line_of_char in src/editor/buffer.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (verify baseline)
2. Complete Phase 2: Foundational (failing regression test T003)
3. Complete Phase 3: User Story 1 (the actual `line_of_char` fix + its tests)
4. **STOP and VALIDATE**: Run `cargo test --test unit buffer cursor` - all US1 tests green
5. The editor now displays the cursor correctly behind a trailing `\n`

### Incremental Delivery

1. Complete Setup + Foundational → bug reproduced by failing test
2. Add User Story 1 fix → tests green → MVP (cursor display corrected)
3. Add User Story 2 regression guards → tests green → no regression confirmed
4. Polish → full `cargo test` green, constitution review complete

### Parallel Team Strategy

With multiple developers after Foundational:

1. Developer A: US1 implementation + buffer tests (`tests/unit/buffer.rs`, `src/editor/buffer.rs`)
2. Developer B: US1 cursor/render tests (`tests/unit/cursor.rs`)
3. Once US1 fix lands, regression tests in US2 run against it

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Fix is intentionally localized to `src/editor/buffer.rs` `line_of_char` (single source of truth per `research.md` R-1); consumers (`cursor.rs`, `render.rs`) inherit the correction - do NOT patch them
- Logical `char_index` is NEVER modified (FR-002); only the char→line *display* mapping changes
- Persistence is out-of-scope (FR-007): existing save tests remain as regression guards, no save-path code change or new save test
- Verify tests FAIL before implementing the fix, then verify they PASS after
- Commit after each task or logical group

## Phase 6: Convergence

- [ ] T020 Add an integration test in `tests/integration/trailing_newline_insert.rs` that inserts a character with the cursor at end-of-doc behind the trailing newline (`"abc\n"`, cursor at `len_chars()`) and asserts the content becomes `"abc\nx"`, the `char_index` advances behind the inserted character, and `line_of_char`/`visual_column` land on the empty trailing line per US1/AC2, SC-002, FR-003, FR-008 (partial)

