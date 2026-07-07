# Tasks: Help Dialog

**Input**: Design documents from `/specs/011-help-dialog/`
**Spec**: [/specs/011-help-dialog/spec.md](/specs/011-help-dialog/spec.md)
**Plan**: [/specs/011-help-dialog/plan.md](/specs/011-help-dialog/plan.md)
**Prerequisites**: plan.md, spec.md, data-model.md, contracts/key-bindings.md, quickstart.md, research.md

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)

## Phase 1: Setup — Crate structure already exists

**Purpose**: No new crate or dependency needed; gobo is a single binary crate with existing ratatui + crossterm deps in Cargo.toml.

- [ ] T001 Verify Cargo.toml lists crossterm ≥0.28 and ratatui ≥0.29 as dependencies

---

## Phase 2: Foundational — Core data structures (blocks all user stories)

**Purpose**: Define the static help data types and session state fields that every story consumes.

- [ ] T002 Define `HelpDialogRow` struct (`key: String`, `label: String`) in src/editor/status.rs
- [ ] T003 [P] (removed — flat list model only, no category chunks needed)
- [ ] T004 [P] Implement `HelpDialog::build_list()` that returns all 9 Ctrl-key bindings (flat list) from contracts/key-bindings.md, in src/editor/status.rs
- [ ] T005 Add `HelpContent` variant to session state (or reuse PendingPrompt) in src/app.rs — add field for open/close flag and scroll offset
- [ ] T006 Track scrolling bounds (offset usize, max_offset usize) as inline fields on the HelpDialog session data in src/app.rs
- [ ] T007 Build `help_view(content, rect)` in src/editor/status.rs that takes static content + TerminalRect → ratatui `Text`/`Paragraph` ready for overlay rendering

**Checkpoint**: All types and constructors are defined; no user story work can begin without these being complete.

---

## Phase 3: US1 — View All Keyboard Shortcuts (Priority: P1) 🎯 MVP

**Goal**: Pressing Ctrl-H opens a centered popup showing a flat table of all 9 active Ctrl-key shortcuts with descriptions.

**Independent Test**: Open Gobo, press Ctrl-H, verify the Help Dialog appears centered with all shortcuts listed clearly; press Enter or Escape to close and confirm control returns to editing mode without side effects. See quickstart.md scenarios S1‑S4.

### Tests for US1 (REQUIRED) ⚠️

- [ ] T008 [US1] Unit test `HelpDialog::build_list()` returns exactly 9 entries (contract-defined order) in tests/unit/help_dialog.rs
- [ ] T009 [P] [US1] Integration test Ctrl-H opens HelpDialog in tests/integration/help_dialog.rs — verify dialog appears with correct title and content

### Implementation for US1

- [ ] T010 Add `ShowHelp` variant to `EditorCommand` enum in src/editor/input.rs (or appropriate command type file)
- [ ] T011 [P] Map `Ctrl-H` → `ShowHelp` in `map_key_event()` in src/editor/input.rs — add one match arm before the printable catch-all, guarded by CONTROL modifier check
- [ ] T012 Add `Prompt::HelpDialog` variant (or `PendingPrompt::HelpDialog`) to the prompt enum in src/app.rs or appropriate state type file
- [ ] T013 Write handler for `ShowHelp` in handle_command/switch — stores dialog content, sets pending_prompt, pushes mode on a stack in src/app.rs
- [ ] T014 Wire `handle_editing_command(ShowHelp)` path in `app.rs` to open the HelpDialog prompt (follows pattern used by ConfirmQuit)
- [ ] T015 Render help dialog overlay in `main.rs::draw()` — match on `pending_prompt`, use full or compact variant based on terminal rect ≥ 44×8, render with ratatui List/Paragraph widget following existing popup styling
- [ ] T016 Handle Enter and Escape to close the dialog in src/app.rs (same mechanism as Cancel prompt: pop mode, clear pending_prompt)

**Checkpoint**: US1 is fully functional — user can open, view all shortcuts, and close with no side effects.

---

## Phase 4: US2 — Scrollable Shortcut List (Priority: P1)

**Goal**: When the list of shortcuts overflows the visible area, up/down arrow keys scroll the list one line per press, bounded at top and bottom.

**Independent Test**: Open Help Dialog in a small terminal so entries overflow vertically; use ↓ to scroll through all entries, then ↑ to navigate back; verify boundary behavior (no scroll past top/bottom). See quickstart.md scenario S2 and spec FR-004/FR-008.

### Tests for US2 (REQUIRED) ⚠️

- [ ] T017 [US2] Unit test `ScrollState` clamping — verify offset stays at 0 when already at top, and max_offset when at bottom in tests/unit/help_dialog.rs
- [ ] T018 [P] [US2] Integration test arrow-key scroll bounds in tests/integration/help_dialog.rs — verify down-arrow scrolls correctly and stops at boundary, up-arrow scrolls back

### Implementation for US2

- [ ] T019 Add `scroll_offset` field to HelpDialog session state in src/app.rs (if not already added in Phase 2)
- [ ] T020 In help_view in src/editor/status.rs: apply scroll_offset when building the ratatui Text/Paragraph so visible lines correspond to `offset..offset+visible_lines` clipped to content bounds
- [ ] T021 Handle `MoveUp` and `MoveDown` keys while HelpDialog is open as scroll actions (not editor motions) in handle_command or key dispatch in src/app.rs — adjust scroll_offset within bounds
- [ ] T022 Ensure other keys during help open are silently ignored (no-op for all non-Enter/non-Escape/non-scroll keys) — add explicit ignore logic in the same handler in src/app.rs

**Checkpoint**: US1 + US2 fully functional — user can discover all shortcuts via scrollable dialog.

---

## Phase 5: US3 — No Mode Interference (Priority: P2)

**Goal**: Opening Help Dialog preserves all underlying mode state (search query, cursor position, pending prompts). While help is open, typed characters have no effect on the document. Closing restores the previous mode exactly as it was.

**Independent Test**: Open a file, enter search mode and type a query, press Ctrl-H to open help, verify search input remains intact; close help with Escape and confirm search state restored; while help is open, verify printable keystrokes do not modify the document. See quickstart.md scenarios S5, S6, S8.

### Tests for US3 (REQUIRED) ⚠️

- [ ] T023 [US3] Integration test search state preserved across Help Dialog in tests/integration/help_dialog.rs — enter search mode with query → open help → close → verify query and cursor restored
- [ ] T024 [P] [US3] Integration test printable keystrokes ignored during help dialog in tests/integration/help_dialog.rs — while help is open, type chars; verify document content unchanged after closing

### Tests for S8 Layered Popup (REQUIRED) ⚠️ C10 from analysis

- [ ] T029 [S8] Integration test help-over-existing-prompt in tests/integration/help_dialog.rs — trigger quit/save-conflict prompt → press Ctrl-H to open help on top → verify stacked layout renders correctly → close help with Escape → verify underlying prompt state is fully preserved

### Implementation for US3

- [ ] T025 Add `previous_mode` field (or mode stack) to PendingPrompt/EditingSession to track the state before HelpDialog opens, stored by ShowHelp handler in src/app.rs
- [ ] T026 When HelpDialog closes: ensure restoration of previous_mode and all session fields (search state, cursor position, pending prompts from earlier layers such as ConfirmQuit) in src/app.rs
- [ ] T027 Refine the ignore-all-keys logic while HelpDialog is open to explicitly reject any key that is not Enter, Escape, MoveUp, or MoveDown — update match arm in handle_command to return an explicit `EditorCommand::NoOp` for all unmapped keys during help (see src/app.rs)
- [ ] T028 Verify no document mutations occur during HelpDialog lifecycle — add assertion in integration test that buffer content before open equals content after close

**Checkpoint**: All 3 user stories independently functional.

---

## Phase N: Polish & Cross-Cutting Concerns

**Purpose**: Readability, maintainability, constitution compliance, manual verification.

- [ ] T029 Review readability of src/editor/status.rs, src/app.rs (ShowHelp path), src/main.rs (draw overlay) — simplify names, reduce nesting where possible
- [ ] T030 Update AGENTS.md SPECKIT reference to point to tasks.md in this spec directory
- [ ] T031 Run quickstart.md manual validation scenarios S1–S8 against the built binary (`cargo run -- /tmp/gobo-test.txt`)
- [ ] T032 Run `cargo test --test unit_help_dialog && cargo test --test integration_help_dialog` — confirm all tests green

---

## Dependencies & Execution Order

### Phase Dependencies
- **Phase 1 (Setup)**: No dependencies — can start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 — **BLOCKS** all user stories
- **Phase 3 (US1)**: Depends on Phase 2 — MVP delivery
- **Phase 4 (US2)**: Depends on Phase 2 — parallel with or after US1
- **Phase 5 (US3)**: Depends on Phase 2 — may integrate with US1/US2 state but independently testable
- **Polish**: Depends on desired story phases being complete

### User Story Dependencies
- **US1 (P1)**: Can start after Foundational — **MVP scope** (can deploy/demo alone)
- **US2 (P1)**: Can start after Foundational — may integrate with US1 scroll + render but independently testable with US1's dialog open
- **US3 (P2)**: Depends on US1 state machine being wired — testing requires US1 to be complete

### Within Each User Story
- Tests MUST be written and FAIL before implementation
- Data structures before handler code
- Handler before rendering
- Story complete before next priority

### Parallel Opportunities
- T003 + T004 (Foundational data types) can run in parallel (same file but independent definitions)
- Once Phase 2 completes, US1 and US2 test tasks can start in parallel
- Different stories can be implemented by different developers once Phases 1 & 2 are done

### Within Each User Story — Parallel Opportunities
- `[P]` labeled tasks for a story can run in parallel (unit tests can run alongside integration tests if using Rust's independent test targets)

---

## Implementation Strategy

### MVP First (US1 Only)
1. Complete Phase 1: Setup verification (T001)
2. Complete Phase 2: Foundational types + state (T002–T007)
3. Complete Phase 3: US1 implementation + tests (T008–T016)
4. **STOP and VALIDATE**: `cargo run -- /tmp/test.txt`, press Ctrl-H, verify dialog, Enter/Escape close
5. Deploy/demo if ready

### Incremental Delivery
1. Setup + Foundational → foundation ready
2. US1 → test independently → MVP delivery (user can see shortcuts)
3. US2 → test independently → scrollable dialog
4. US3 → test independently → no side effects, mode preservation
5. Polish → constitution review, quickstart validation

### Parallel Team Strategy
- With multiple developers: Phase 1 + 2 done together
- Once Foundational complete: one dev on each user story in priority order

---

## Notes
- All tasks follow the checklist format with checkbox, sequential ID (T001+), [Story] label for US phases, and exact file paths
- Tests are REQUIRED per constitution Section IV — every feature behavior must have automated coverage
- Commit after each task or logical group
- Stop at any checkpoint to validate the story independently
- Total task count: **32** (T001–T032)
  - Setup: 1
  - Foundational: 6
  - US1: 7
  - US2: 6
  - US3: 6
  - Polish: 4

## Suggested MVP Scope
**User Story 1 only** (Phase 3): Ctrl-H opens centered Help Dialog with all shortcuts, Enter/Escape closes. This is independently testable and delivers value — users get the complete shortcut reference immediately.
