# Research: Tab Support and Auto-Indent

## Context

The spec already resolved the user-facing clarifications: selection handling, column definition, and undo semantics. Phase 0 research therefore focuses on implementation shape, integration boundaries, and test strategy.

## Decisions

### Decision: Add a small pure helper module at `src/editor/indent.rs`
**Rationale**: The feature introduces three related but distinct indentation rules: tab insertion width, auto-indent after enter, and special backspace deletion width. Keeping those calculations in a pure helper module preserves the existing boundary between key interpretation (`input.rs`), session orchestration (`app.rs`), and rope mutation/history recording (`buffer.rs` / `history.rs`). This directly supports FR-016 and Constitution I/II.
**Alternatives considered**:
- Put all indentation math directly into `src/app.rs`: simpler at first, but it would further grow an already large orchestration file and make the rules harder to review.
- Scatter helpers across `buffer.rs` and `cursor.rs`: rejected because indentation is a feature-level behavior, not a generic rope primitive or cursor primitive.

### Decision: Introduce a dedicated `EditorCommand::Tab` for the physical Tab key
**Rationale**: The Tab key becomes a real editing action in editing mode while still acting as “next choice” in prompt mode. A dedicated command keeps the input mapping honest and avoids overloading `NextChoice` with two unrelated meanings. Prompt navigation stays explicit in `handle_prompt_command`, where `Tab` can be treated equivalently to `NextChoice`.
**Alternatives considered**:
- Reuse `EditorCommand::NextChoice` in editing mode for indentation: rejected because it hides that editing mode is performing a text mutation.
- Map Tab directly to `InsertChar(' ')`: rejected because the number of inserted spaces depends on the current column and selection state.

### Decision: Compute indentation columns from char counts, not visual width
**Rationale**: FR-002 defines the column as the zero-based number of characters left of the cursor in the current line, with each inserted or deleted character counting as 1. The feature must therefore use line-start char indices and raw char-distance calculations rather than `visual_column`, which is intentionally grapheme/display-width aware for cursor motion.
**Alternatives considered**:
- Reuse `cursor::visual_column()`: rejected because full-width or combining characters would make indentation behavior diverge from the clarified requirement.

### Decision: Enter inserts `"\n" + leading_spaces(previous_line)` as one atomic edit
**Rationale**: Auto-indent should preserve the content to the right of the cursor and remain one undo step. Building the full inserted text up front lets `app.rs` record exactly one `EditStep::Insert` or `EditStep::Replace`, depending on whether a selection is active.
**Alternatives considered**:
- Insert newline first, then insert spaces as a second edit: rejected because it violates FR-017’s single-undo-step requirement.
- Recompute indentation after mutating the rope: rejected because it complicates reasoning around selection replacement and split-line behavior.

### Decision: Special Backspace operates only on all-space prefixes and deletes to the previous even column in one atomic edit
**Rationale**: FR-008..FR-013 require the custom backspace behavior only when the current line segment from line start to cursor consists exclusively of spaces. The deletion width is deterministic from the count of those spaces: odd -> 1, even -> 2, clamped at line start. Recording the whole action as one delete/replace step preserves undo semantics.
**Alternatives considered**:
- Trigger special backspace whenever the immediate previous char is a space: rejected because mixed content before the cursor must keep normal backspace behavior.
- Implement repeated single-char deletions: rejected because it would create multiple history steps or require hidden batching.

### Decision: Handle selection replacement inside the indentation commands themselves
**Rationale**: FR-015 requires Tab, Enter, and Backspace to delete the active selection first and then apply their normal logic at the insertion point, all in one undo step. That means the feature cannot rely only on the existing generic selection replacement seam; each indentation command must build the final replacement text or replacement range before recording history.
**Alternatives considered**:
- Reuse `replace_selection()` first, then run the normal command: rejected because that would create two distinct edits.
- Ignore the special selection rule and fall back to current generic replacement: rejected because it does not satisfy the spec.

### Decision: Extend the existing standalone integration-test pattern with a dedicated feature test file
**Rationale**: Existing behavior changes are validated through `EditingSession::open()` plus `handle_command()`. A focused integration file keeps the feature easy to reason about and matches Constitution IV. Small pure calculations in `indent.rs` should additionally get unit tests for boundary cases.
**Alternatives considered**:
- Test only through unit tests: rejected because mode handling, selection replacement, and history behavior live in `EditingSession`.
- Fold tests into `enter_newline.rs`: rejected because this feature expands beyond Enter and would blur responsibility.

## Requirements Resolved

| Topic | Resolution |
|---|---|
| Tab meaning in editing mode vs prompts | Add dedicated `EditorCommand::Tab`; prompt handling still treats Tab as next-choice navigation |
| Column basis for indentation logic | Raw char count from line start to cursor, not visual width |
| Atomic Enter behavior | Insert newline plus copied leading spaces as one edit |
| Atomic selection replacement for Tab/Enter/Backspace | Each command computes its own final replace/delete result before history recording |
| Special Backspace trigger | Only when all chars from line start to cursor are spaces |

## Best-Practice Notes

- Prefer pure helper functions that return widths/ranges/inserted strings, leaving mutation and history recording in `app.rs`.
- Reuse existing line helpers in `buffer.rs` (`line_of_char`, `line_start_char`, `line_content`) instead of introducing alternate line parsing.
- Keep prompt behavior unchanged: Tab/Shift-Tab still move between prompt choices outside editing mode.
- Add explicit automated tests for selection cases and undo because those are the most regression-prone integration points.
