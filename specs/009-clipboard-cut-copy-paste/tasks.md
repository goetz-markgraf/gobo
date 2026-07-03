# Tasks: Clipboard Cut, Copy & Paste

**Input**: Design documents from `/specs/009-clipboard-cut-copy-paste/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Automated test tasks are REQUIRED. Every user story and every in-scope feature must include tests for the primary flow and relevant edge cases.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add dependency and create the clipboard module scaffold.

- [ ] T001 Add `arboard = "3"` to `[dependencies]` in Cargo.toml
- [ ] T002 Create `src/editor/clipboard.rs` with module declaration in `src/editor/mod.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Implement the clipboard I/O layer and size-limit logic — required before ANY user story can be implemented.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T003 [P] Implement `clipboard::read_from_system_clipboard() -> Option<String>` in `src/editor/clipboard.rs`
- [ ] T004 [P] Implement `clipboard::write_to_system_clipboard(text: &str) -> Result<(), String>` in `src/editor/clipboard.rs`
- [ ] T005 [P] Implement `is_text_only(content: &[u8]) -> bool` text-filter helper in `src/editor/clipboard.rs`
- [ ] T006 Add 1 MB size-limit enforcement to both read and write boundaries with status message for rejections

**Checkpoint**: Foundation ready — user story implementation can now begin.

---

## Phase 3: User Story 1 — Copy (Ctrl-C) (Priority: P1) 🎯 MVP

**Goal**: Copy selected text or single grapheme to OS clipboard without modifying the editor buffer.

**Independent Test**: Select "Welt" in "Hallo Welt", press Ctrl-C; verify system clipboard contains "Welt" via `pbpaste`; confirm editor text is unchanged and selection remains visible.

### Implementation for User Story 1

- [ ] T007 [P] [US1] Derive two new `EditorCommand` variants (`Copy`, `Cut`, `Paste`) in `src/editor/input.rs`
- [ ] T008 [P] [US1] Add `(Ctrl, Char('c')) => Some(EditorCommand::Copy)` binding in `src/editor/input.rs::map_key_event`
- [ ] T009 [P] [US1] Implement copy handler: if selection exists → copy selected text; else → copy first grapheme cluster after cursor (uses `unicode-segmentation`) in `src/app.rs::handle_editing_command`
- [ ] T010 [US1] Add status message `"Copied {N} chars"` on success via `StatusMessage::info` in the same handler
- [ ] T011 [US1] Wire clipboard write call, pass source text, handle error with warning `"Failed to copy: {msg}"`

**Checkpoint**: User Story 1 (Copy) should be fully functional and testable independently.

---

## Phase 4: User Story 2 — Cut without Selection (Priority: P1)

**Goal**: Cut the single grapheme cluster at/after the cursor position to OS clipboard, then delete it in one undo step.

**Independent Test**: Place cursor between "o" and "W" in "HalloWelt", press Ctrl-X; confirm text becomes "Halloelt" and clipboard has "W"; press Ctrl-Z; confirm full restoration to "HalloWelt". Clipboard still contains "W".

### Implementation for User Story 2

- [ ] T012 [P] [US2] Add `(Ctrl, Char('x')) => Some(EditorCommand::Cut)` binding in `src/editor/input.rs::map_key_event`
- [ ] T013 [US2] Implement cut-no-selection: get first grapheme at cursor via `unicode-segmentation`, write to clipboard (same as US1), delete from buffer, record single `EditStep::Delete`, show `"Cut {N} chars"` in `src/app.rs::handle_editing_command`
- [ ] T014 [US2] Handle end-of-document edge: when cursor is at last char or past it, delete the trailing character (including `\n`)
- [ ] T015 [US2] Handle "nothing to cut" case → no grapheme after cursor → show `"Nothing to cut"` status

**Checkpoint**: User Stories 1 AND 2 should both work independently.

---

## Phase 5: User Story 3 — Cut with Selection (Priority: P1)

**Goal**: Cut selected text (potentially multi-line) to OS clipboard and delete from buffer in one undo step.

**Independent Test**: Select "Zeile2\n" across lines in the editor, press Ctrl-X; confirm editor now has "Zeile1\nZeile3\n"; press Ctrl-Z; confirm full line restoration. Clipboard still has "Zeile2\n".

### Implementation for User Story 3

- [ ] T016 [US3] Implement cut-with-selection: source = selected range text, write to clipboard via arboard, record single `EditStep::Replace { removed, inserted: "" }`, clear selection → same handler extension as US2 in `src/app.rs::handle_editing_command`
- [ ] T017 [US3] Ensure all newlines and special characters within selected range are preserved in the edit step (constitution III — data integrity)

**Checkpoint**: All three user stories (Copy, Cut with/without selection) should now be independently functional.

---

## Phase 6: User Story 4 — Paste without Selection (Priority: P1)

**Goal**: Insert OS clipboard content at cursor position in one undo step; silently no-op on empty/non-text clipboard.

**Independent Test**: Copy "Test", place cursor in middle of "HalloWelt", press Ctrl-V → "HalloTestWelt"; press Ctrl-Z → back to "HalloWelt"; clipboard still contains "Test".

### Implementation for User Story 4

- [ ] T018 [P] [US4] Add `(Ctrl, Char('v')) => Some(EditorCommand::Paste)` binding in `src/editor/input.rs::map_key_event`
- [ ] T019 [US4] Implement paste-no-selection: call `clipboard::read_from_system_clipboard()`, if `None` or empty → silent no-op (no status, no undo); if >1 MB → `"Clipboard content too large (>1 MB)"`; else → insert at cursor via existing buffer/insert machinery
- [ ] T020 [US4] Move cursor to end of inserted text; clear any selection; show `"Pasted {N} chars"` on success
- [ ] T021 [US4] Handle binary clipboard content → `None` from arboard → same as empty clipboard (silent no-op)

**Checkpoint**: User Stories 1–4 should now all be independently functional.

---

## Phase 7: User Story 5 — Paste with Selection (Priority: P2)

**Goal**: Replace the selected text range with clipboard content in one undo step; leave selection cleared.

**Independent Test**: Select "Alte" in "Hallo AlteWelt", clipboard has "Neue", press Ctrl-V → "Hallo NeueWelt"; press Ctrl-Z → back to "Hallo AlteWelt".

### Implementation for User Story 5

- [ ] T022 [US5] Extend paste handler: if non-empty selection exists → use `EditStep::Replace { removed: selected, inserted: clipboard_text }` instead of `Insert`, clear selection, land cursor after inserted text — extension to `src/app.rs::handle_editing_command`

**Checkpoint**: All five user stories should now be independently functional.

---

## Phase 8: Integration Tests

**Purpose**: Automated tests covering all primary flows and edge cases per constitution IV.

### Test file creation

- [ ] T023 [P] Create `tests/integration/clipboard_features.rs` skeleton with `[[test]]` entry in Cargo.toml
- [ ] T024 [US1] Write integration test: copy with selection — select "Welt", Ctrl-C → verify clipboard content via arboard or mock; text unchanged
- [ ] T025 [US1] Write integration test: copy without selection (single char) — cursor between chars, Ctrl-C → verify single grapheme in clipboard
- [ ] T026 [US2] Write integration test: cut without selection — cursor between "o" and "W", Ctrl-X → verify deletion + clipboard content
- [ ] T027 [US3] Write integration test: cut with multi-line selection — select across lines, Ctrl-X → verify text removal preserves newlines; undo restores
- [ ] T028 [US3] Write integration test: cut then undo → verify clipboard persists after undo (FR-006)
- [ ] T029 [US4] Write integration test: paste without selection — clipboard has "Test", cursor mid-text, Ctrl-V → verify insertion + cursor position
- [ ] T030 [US5] Write integration test: paste over selection — select text, clipboard has replacement, Ctrl-V → verify replacement + undo restores
- [ ] T031 Empty clipboard paste → silent no-op (no status, no undo entry)
- [ ] T032 Large clipboard (>1 MB rejection) — synthetic large text, paste → verify warning message and no buffer change
- [ ] T033 Binary clipboard content → `None` from arboard → silent no-op
- [ ] T034 Multi-line cut/restore — multi-line selection, Ctrl-X, undo → verify full restoration of all lines

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Final verification against constitution and quickstart validation guide.

- [ ] T035 [P] Verify read-write symmetry: copy content exactly matches text copied; cut removed text equals clipboard written
- [ ] T036 [P] Ensure no grapheme cluster boundaries are violated when selecting/cutting single characters (use `unicode-segmentation::GraphemeClusterIter`)
- [ ] T037 Run quickstart.md validation scenarios (Scenarios 1–9) against implementation; fix any mismatches
- [ ] T038 Review readability and maintainability of all touched files: verify clear boundaries per constitution II (`clipboard.rs` — I/O only, `input.rs` — bindings only, `app.rs` — dispatch only); simplify names where needed
- [ ] T039 Final constitution compliance review: readability (one new module), maintainability (bounded changes), security (no internal clipboard cache), verification (tests cover all feature paths), scope (single dependency, focused feature)

---

## Dependencies & Execution Order

### Phase Dependencies

1. **Setup (Phase 1)** → no dependencies
2. **Foundational (Phase 2)** → depends on Setup; **BLOCKS all user stories**
3. **User Stories (Phase 3–7)** → depend on Foundational completion; can proceed sequentially (P1→P2) or in parallel by a team
4. **Tests (Phase 8)** → depend on implementation; can start after each US phase independently
5. **Polish (Phase 9)** → depends on all desired user stories + tests being complete

### User Story Dependencies

- **US1 (Copy)**: After Foundational — MVP scope
- **US2 (Cut no-selection)**: After Foundational — independent from US1
- **US3 (Cut with-selection)**: After Foundational — shares cut logic with US2
- **US4 (Paste no-selection)**: After Foundational — independent, uses read side of clipboard
- **US5 (Paste with-selection)**: Depends on pasting foundation + selection replace

### Within Each User Story

1. Key binding in `input.rs` can be done early (parallel with handler)
2. Command handler extension in `app.rs` is the core work
3. Status message wiring last

---

## Parallel Example: Foundational Phase

```bash
# T003, T004, T005 — clipboard I/O functions can be developed independently
# (different function bodies in same file, no cross-dependency)

# After setup tasks complete:
cargo build  # verify arboard added and module compiles
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL — blocks all stories)
3. Complete Phase 3: User Story 1 (Copy)
4. **STOP and VALIDATE**: Verify Ctrl-C copies selection; `pbpaste` confirms content; editor text unchanged

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. US1 (Copy) → Test → MVP! ✅
3. US2 (Cut no-selection) → Test
4. US3 (Cut with-selection) → Test
5. US4 (Paste) → Test
6. US5 (Paste over selection) → Test
7. Tests (Phase 8) + Polish (Phase 9)

---

## Task Count Summary

| Phase | Description | Tasks |
|-------|-------------|-------|
| Phase 1 | Setup | 2 |
| Phase 2 | Foundational | 4 |
| Phase 3 | US1 — Copy | 5 |
| Phase 4 | US2 — Cut (no selection) | 4 |
| Phase 5 | US3 — Cut (with selection) | 2 |
| Phase 6 | US4 — Paste (no selection) | 4 |
| Phase 7 | US5 — Paste (with selection) | 1 |
| Phase 8 | Integration Tests | 12 |
| Phase 9 | Polish & Cross-Cutting | 5 |
| **Total** | | **39** |

### Per User Story

- **US1 (Copy)**: T007–T011 = **5 tasks**
- **US2 (Cut no-selection)**: T012–T015 = **4 tasks**
- **US3 (Cut with-selection)**: T016–T017 = **2 tasks**
- **US4 (Paste no-selection)**: T018–T021 = **4 tasks**
- **US5 (Paste selection)**: T022 = **1 task**

### Parallel Opportunities

- Phase 2 tasks T003, T004, T005 can execute in parallel
- Tests for different USs can run concurrently
- All copy/cut/paste key bindings (T007, T012, T018) are independent additions to the same file

### Independent Test Criteria Per Story

| User Story | Test Criteria |
|------------|---------------|
| US1 (Copy) | Selection remains in editor; clipboard has correct text via `pbpaste` |
| US2 (Cut no-selection) | Single char deleted; undo restores; clipboard retains content |
| US3 (Cut with-selection) | Multi-line text removed correctly; multi-line cut restored by undo; clipboard persists |
| US4 (Paste no-selection) | Text inserted at cursor; undo restores original; empty/binary clipboard = no-op |
| US5 (Paste with-selection) | Selected range replaced; single undo restores original; clipboard unchanged |

### Suggested MVP Scope

**User Story 1 only** (Copy with/without selection): After completing Phases 1–3, the editor can copy text to clipboard — a usable, independently testable feature delivering real user value.
