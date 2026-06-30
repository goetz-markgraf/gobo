---

description: "Task list for Visible Search Popup & Ctrl+G Find-Next feature"

---

# Tasks: Visible Search Popup

**Input**: Design documents from `specs/004-visible-search-popup/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Automated test tasks are REQUIRED per Constitution IV (Verification Before Merge).

## Phase 1: Setup

**Purpose**: No setup changes needed — project already buildable and structured.

- [X] T001 Verify `cargo build` and `cargo test` pass in existing state
---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Prepare the codebase for US1, US2, and US3 implementation.

- [X] T002 Add unit tests for `SearchState::find_next()` edge cases in `tests/unit/search.rs`  - Empty query returns `None` without side effects
  - Single-character document with matching query at position 0
  - Query longer than document never matches
  - Wrap-around when cursor is past the last match
    - Multi-byte grapheme query (e.g. CJK, emoji) finds exact character matches
    - Invalid UTF-8 sequences in query do not crash find_next()
- [X] T003 [P] Verify existing unit tests (`cargo test --lib`) still pass after adding search edge-case tests
**Checkpoint**: Foundation ready — user story implementation can now proceed.

---

## Phase 3: User Story 1 - Search for text in document (Priority: P1) 🎯 MVP

**Goal**: Make the `Search: ` prompt visible on the last screen line with distinct yellow foreground when Ctrl+F is pressed; support typing, Enter to confirm/find, and Esc to cancel.

**Independent Test**: Start editor, press Ctrl+F, verify "Search: "... appears in yellow on the bottom line; type characters and watch them appear immediately; press Enter to jump to first match; press Esc to cancel without modifying cursor.

### Tests for User Story 1 (REQUIRED) ⚠️

- [X] T004 [P] [US1] Integration test for full search flow in `tests/integration/search_and_resize.rs`: Ctrl+F → typing query → Enter confirms first match → editor returns to editing mode — cursor jumps to match start- [X] T005 [P] [US1] Integration test for cancel flow in `tests/integration/search_and_resize.rs`: Ctrl+F → typing partial query → Esc → no cursor movement, mode returns to Editing
### Implementation for User Story 1

- [X] T006 [US1] Fix search prompt rendering in `src/editor/render.rs` — ensure `render_view()` correctly wires `status::search_prompt()` into `bottom_line: Option<String>` when popup is None and mode is SearchInput; verify `prompt_lines()` returns 2 for SearchInput
- [X] T007 [US1] Fix yellow foreground styling in `src/main.rs` — replace ambiguous `Style::default().fg(Color::Yellow)` with explicit styled block on the search prompt paragraph to ensure visible contrast against terminal background
- [X] T008 [US1] Verify status line mechanism in `src/editor/status.rs` — confirm popup_view returns None for SearchInput; update search_prompt() return type if needed per plan's analysis (visual fix may reside in main.rs or render.rs)

**Checkpoint**: User Story 1 is fully functional — pressing Ctrl+F shows visible "Search: " prompt on the last screen line.

---

## Phase 4: User Story 2 - Find next occurrence (Priority: P2)

**Goal**: Add `EditorCommand::FindNext` variant + Ctrl+G keybinding; during SearchInput mode, Ctrl+G finds the next match from cursor position with wrap-around to beginning of document.

**Independent Test**: After confirming a search match (Enter), press Ctrl+G repeatedly — each press jumps to the next occurrence forward; when reaching end-of-document, first match reappears (wrap-around); empty query + Ctrl+G shows "no match" without moving cursor.

### Tests for User Story 2 (REQUIRED) ⚠️

- [X] T009 [P] [US2] Unit test for `find_next()` wrap-around in `tests/unit/search.rs`: with a document containing 3 occurrences at positions P1 < P2 < P3, starting from cursor at P2 returns P3; starting past P3 wraps to P1
- [X] T010 [P] [US2] Integration test for find-next flow in `tests/integration/search_and_resize.rs`: confirm search → Ctrl+G jumps to next match → Ctrl+G wraps around to first occurrence — verify cursor position and status messages at each step- [X] T011 [P] [US2] Edge-case unit test: searching a query with no matches → `find_next()` returns `None` every time; pressing multiple times shows "No match" without moving cursor
### Implementation for User Story 2

- [X] T012 [P] [US2] Add `FindNext` variant to `EditorCommand` enum and add keybinding in `src/editor/input.rs`: `(KeyModifiers::CONTROL, KeyCode::Char('g')) → EditorCommand::FindNext`; keep it as a no-op in the match arm for other keys- [X] T013 [US2] Add `FindNext` branch to `handle_search_command()` in `src/app.rs`: call `search.find_next(&self.document.text, self.cursor.char_index)`, move cursor on match, display "Match at.." or "No match" status message, stay in SearchInput mode- [X] T014 [US2] Add `FindNext` handling to `handle_editing_command()` in `src/app.rs`: emit no-op (search not active outside SearchInput mode), add to the catch-all arm alongside existing variants
**Checkpoint**: User Stories 1 AND 2 are both functional — search is visible and Ctrl+G navigates matches correctly.

---

## Phase 5: User Story 3 - Case mode awareness (Priority: P3)

**Goal**: Verify and document that default case-insensitive matching behavior works correctly. **No code changes needed** — `SearchState` already uses `CaseMode::Insensitive` by default; `normalize()` lowercases both query and haystack.

**Independent Test**: Search for "HELLO" in a document containing only "hello" — verify Enter confirms the lowercase match, query stays as "HELLO" on screen (original casing preserved).

### Tests for User Story 3 (REQUIRED) ⚠️

- [X] T015 [P] [US3] Unit test for case-insensitive matching in `tests/unit/search.rs`: document contains "hello world" (lowercase only), query "HELLO" matches → cursor moves to position 0; query "WORLD" matches → cursor moves to position 6
### Implementation for User Story 3

- [X] T016 [US3] No code changes required — verify `normalize()` in `src/editor/search.rs` lowercases both query and haystack, confirming case-insensitive matching works per default `CaseMode::Insensitive`- [X] T016a [US3] Integration check: after confirming a search for uppercase "HELLO" that matches lowercase "hello", the prompt still displays original casing "Search: HELLO" on screen
**Checkpoint**: All user stories are independently functional.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Quality improvements, constitution checks, final validation.

- [X] T017 Verify all unit tests pass: `cargo test --lib`- [X] T018 Run integration tests: `cargo test --test '*'`
- [X] T019 [P] Add edge-case integration test to `tests/integration/search_and_resize.rs`: empty query + Enter exits silently (FR-004a); empty query + Ctrl+G shows "No match" without moving cursor (FR-007)
- [X] T020 Verify code readability against Constitution Principle I: check `src/app.rs`, `src/editor/input.rs`, `src/editor/search.rs`, `src/editor/status.rs`, `src/main.rs` for clear naming and simple control flow
- [X] T021 Maintainability check against Constitution Principle II: no new types/modules needed, existing boundaries preserved (app → state, search → text ops, status → output, main → rendering)
- [X] T022 Security/governance check against Constitution Principle III: search input is pure in-memory, no file I/O during search, empty query exits cleanly without panic

---

## Dependencies & Execution Order

### Phase Dependencies

| Phase | Depends On | Notes |
|-------|-----------|-------|
| 1. Setup | None | Verification only |
| 2. Foundational | Phase 1 | Tests must pass in base state before US work |
| 3. US1 (P1) | Phase 2 | MVP — fully independent deliverable |
| 4. US2 (P2) | Phase 2 | Depends on US1 for working search input |
| 5. US3 (P3) | Phase 2 | Verification-only, minimal risk |
| 6. Polish | All US | Final validation sweep |

### User Story Dependencies

- **US1 (P1)**: No dependencies — core rendering fix + basic search flow
- **US2 (P2)**: Depends on US1 working — needs visible search input to navigate matches
- **US3 (P3)**: Verified independently — no-code verification of existing behavior

---

## Parallel Opportunities

| Story | Parallel Tasks |
|-------|---------------|
| Setup | T004, T005, T006 (all unit + integration tests) |
| US1 | T009, T010, T011 — tests can be written in any order; render fix + keybinding + app wiring are independent file changes |
| US2 | T014, T013 — input.rs keybinding independent of app.rs logic; both must complete before US2 is done |
| Phase 6 | T017, T018, T020, T021, T022 — all verifications can be reviewed in parallel; actual runs are sequential |

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1 → verify `cargo test` passes
2. Complete Phase 2 → add edge-case search unit tests
3. Complete Phase 3 → US1 implemented: visible "Search: " prompt, Enter to find, Esc to cancel
4. **STOP and VALIDATE**: Run `cargo test` — all tests must pass
5. Deliver MVP: Ctrl+F shows working yellow search prompt

### Incremental Delivery

1. Setup + Foundational → Foundation ready for feature work
2. Add US1 → Test independently → Search works (MVP!)
3. Add US2 → Test independently → Find-next with wrap-around works
4. Add US3 → Verify behavior confirmed
5. Polish → Final constitution compliance sweep

---

## Task Summary

| Phase | Tasks | Focus |
|-------|-------|-------|
| Phase 1: Setup | T001 | Base build verification |
| Phase 2: Foundational | T002, T003 | Unit test groundwork |
| Phase 3: US1 (P1) | T004–T008 | Visible search prompt rendering + basic flow |
| Phase 4: US2 (P2) | T009–T014 | FindNext / Ctrl+G keybinding + wrap-around navigation |
| Phase 5: US3 (P3) | T015–T016 | Case-insensitive matching verification |
| Phase 6: Polish | T017–T022 | Full test suite run, constitution check |

**Total**: 22 tasks across 6 phases
**MVP Scope**: Phases 1 + 2 + 3 (8 tasks) — US1 delivers the visible search popup
**Estimated Parallel Opportunities**: ~4 groups of parallelizable tasks per user story phase
