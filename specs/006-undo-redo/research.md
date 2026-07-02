# Research: Undo / Redo

**Branch**: `006-undo-redo` | **Date**: 2026-07-02

Resolves the design decisions left open by the spec and the plan's Technical Context.

---

## Decision 1: Step representation — diff, not full-state snapshot

**Decision**: Each `EditStep` stores a *diff* (the inserted text + position, or the deleted text + position), not a full `Rope` snapshot of the whole document.

**Rationale**:
- The spec explicitly leaves this open to planning but requires deterministic, exact Undo/Redo including cursor position across Unicode and line boundaries (FR-010).
- A diff is `O(step size)` in memory per step; a full-state snapshot is `O(document size)` per step. With an unbounded stack (FR-004) and per-keystroke granularity (FR-011), full-state snapshots would exhaust memory orders of magnitude sooner. FR-006's "drop oldest undo step on memory pressure" is far more meaningful when each step is small.
- `ropey::Rope` mutations (`insert`, `remove`) are O(log n); applying or reversing a small diff is cheap and keeps the 60fps event loop unaffected in practice (the editor is already user-paced per keystroke).
- Reversing a diff is trivial and symmetric: the reverse of `Insert { index, text }` is `Delete { index, text }` and vice versa, which makes the Undo/Redo determinism invariant (FR-012) easy to reason about and to test at the unit level.

**Alternatives considered**:
- *Full `Rope` snapshot per step*: rejected — memory blow-up makes the "unbounded" guarantee (FR-004/SC-006) meaningless in practice and forces frequent oldest-step eviction under normal use, defeating the purpose of long histories.
- *Full `String` snapshot per step*: same problem, plus `String` loses `ropey`'s structural sharing.
- *Operation log with no payload (just an enum tag)*: rejected — reversing a delete requires knowing *what* was deleted, so the deleted text must be captured. The minimal payload is exactly the diff payload chosen.

---

## Decision 2: Granularity — one keyboard event = one step (confirmed, not designed)

**Decision**: One `EditorCommand` that mutates text produces exactly one `EditStep`. No grouping/coalescing.

**Rationale**: FR-011 and the spec's Assumptions section make this binding and explicit; it is not a planning decision. Recorded here for completeness. Concretely: each `InsertChar`, `Enter` (newline), `Backspace`, and `Delete` in editing mode yields one step. Multi-character paste is not present in the current input map (no paste command exists), so the only multi-char insert path today is `InsertChar` of a single char; the design still records one step per call to the `insert_text` seam so a future paste path remains correct without restructuring.

---

## Decision 3: Stack semantics — undo grows on edit, redo grows on undo, edits clear redo

**Decision**:
- A new text edit pushes one `EditStep` (the *forward* diff) onto the **undo** stack and *clears* the **redo** stack (FR-007).
- `Undo` pops the top of the undo stack, applies its *reverse* diff to the `Rope`, and pushes the *forward* diff (the popped step itself) onto the redo stack.
- `Redo` pops the top of the redo stack, applies the *forward* diff, and pushes the *reverse* interpretation back onto the undo stack.
- Both `Undo` and `Redo` are no-ops when their respective stack is empty (stack-end edge cases).

**Rationale**: This is the standard, well-understood two-stack model. It directly satisfies FR-001/FR-002/FR-007/FR-012 and the "Redo wird bei neuer Änderung geleert" story. It keeps `History` as a self-contained struct operating purely on `&mut Rope`, matching the project's "stateless functions where possible / stateful structs only for compound concerns" pattern.

**Alternatives considered**:
- *Single combined step list with a current pointer*: equivalent in behavior but harder to read because the "clear redo on edit" rule becomes a slice truncation rather than an obvious `redo.clear()`. Two named stacks read more clearly and pass the Readability gate more easily.
- *Storing both forward and reverse diff on every step*: rejected — redundant; the reverse is deterministically derivable from the forward diff's kind, so storing one is enough and avoids the two copies drifting out of sync.

---

## Decision 4: Memory-exhaustion handling — drop oldest undo step, always apply edit, inform user

**Decision**: Pushing a new step onto the undo stack allocates the step payload (a small owned `String`). Allocation for a small `String` effectively never fails on normal OSes (Rust's default allocator aborts the process on OOM rather than returning `Err`), but the spec (FR-006/SC-007) requires a defined, testable behavior. Therefore:
- `History::record` attempts to push the step. We model a configurable *capacity budget* (in number of steps, default `usize::MAX` for the unbounded case, but the `History` API exposes a `record_with_budget` style hook so the OOM path is *deterministically testable* without needing to actually exhaust the host).
- When the budget says "no room", the **oldest** (bottom) undo step is removed (`Vec::remove(0)`) and the new step is pushed on top. The edit itself has *already* been applied to the `Rope` by `app.rs` before `record` is called, so the edit is never lost; only the ability to undo past the dropped step is lost.
- `record` returns a `RecordOutcome { oldest_dropped: bool }` so `app.rs` can set a `StatusMessage::warning("History truncated to free memory")` to inform the user (FR-006 last clause).
- `Undo` and `Redo` themselves do not grow memory unboundedly in a way distinct from record; they move an existing step between stacks, so no separate OOM handling is needed there.

**Rationale**:
- FR-006 mandates: edit is always applied (do not block input), existing history and document text are not corrupted, oldest undo step is the victim, and the user is informed if eviction occurred or recording failed. The two-stack model + "apply edit before record" + "drop oldest on budget" satisfies every clause.
- Using a step-count budget for the testable hook (rather than probing real allocator OOM) is the only way to make SC-007 automated and repeatable, which is itself a constitution requirement (Verification gate). The real, unbounded runtime behavior is preserved by defaulting the budget to `usize::MAX`; the budget is an injection point for tests, not a product-facing limit, so FR-004 ("no artificial cap") is respected.

**Alternatives considered**:
- *Actually allocate until the host OOMs*: rejected — non-deterministic, untestable, and may abort the process (default Rust allocator) rather than exercise the recovery path. Violates the Verification gate.
- *Hard cap on stack size at, e.g., 10 000 steps*: rejected — that is an artificial cap, violating FR-004/SC-006. The budget is test-only and defaults to unbounded.
- *Drop newest instead of oldest on pressure*: rejected — FR-006 explicitly names the oldest step as the victim.

---

## Decision 5: Gating — editing mode only

**Decision**: `EditorCommand::Undo` and `EditorCommand::Redo` are dispatched *only* in `SessionMode::Editing`. In `SearchInput`, `ConfirmQuit`, `SaveConflictPrompt`, and whenever `pending_prompt.is_some()`, they are ignored (the existing `handle_command` precedence — prompts first, then mode dispatch — already enforces this; they are simply not matched in the prompt/search handlers).

**Rationale**: Directly satisfies FR-009 and the "Undo/Redo während einer Eingabeaufforderung" edge case. No new dispatch logic is needed beyond adding the two arms to `handle_editing_command`.

---

## Decision 6: Read-only interaction

**Decision**: Undo/Redo are *not* blocked in read-only mode. However, because all text-mutation commands are already blocked in read-only mode (and therefore never record steps), the undo and redo stacks remain empty for a read-only document, so `Undo`/`Redo` are effectively no-ops there. No special-case code is required.

**Rationale**: Undo/Redo restore in-memory recorded states; they never touch disk. Blocking them would be unnecessary, and allowing them costs nothing because there is nothing to pop. This is noted in the plan's Security gate.

---

## Decision 7: Cursor restoration on Undo/Redo

**Decision**: Each `EditStep` carries the **cursor char index** that existed *after* the original edit (i.e., the result index the mutation returned). Undo restores the cursor to the index that existed *before* the edit (the start index, derivable from the diff); Redo restores it to the after-index. The `preferred_column` is recomputed via the existing `cursor::visual_column` helper after restoring the index, matching the pattern already used everywhere else in `app.rs`.

**Rationale**: FR-010 demands Undo/Redo restore position as well as content. Storing the after-index on the step and deriving the before-index from the diff (delete → start = index; insert → start = index) is the minimal payload that satisfies position restoration without storing a separate cursor snapshot. Recomputing `preferred_column` keeps behavior consistent with every other cursor move in the codebase.

---

## Consolidated summary

| # | Topic | Decision |
|---|-------|----------|
| 1 | Step representation | Diff (`Insert`/`Delete` with text + index), not full snapshot |
| 2 | Granularity | One mutation command = one step (spec-mandated) |
| 3 | Stack semantics | Two stacks; edits push undo + clear redo; undo moves to redo; redo moves back to undo |
| 4 | Memory exhaustion | Apply edit before record; on budget pressure drop oldest undo step; inform user; budget is a test injection point defaulting to unbounded |
| 5 | Mode gating | Undo/Redo active only in `Editing`; ignored in prompts/search |
| 6 | Read-only | No special handling; stacks stay empty so undo/redo are inert |
| 7 | Cursor restoration | Step carries after-index; before-index derived from diff; `preferred_column` recomputed |

All NEEDS CLARIFICATION items from the Technical Context are resolved. No external research dependencies were required — the decisions follow from the existing `ropey`-based architecture documented in `architecture.md` and the FR/SC/edge-case list in `spec.md`.
