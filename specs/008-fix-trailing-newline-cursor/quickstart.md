# Quickstart: Fix Trailing Newline Cursor Position

Runnable validation that the trailing-newline cursor bug is fixed without
regressions. All checks are automated unit tests — no terminal/manual step
required.

## Prerequisites

- Rust toolchain (edition 2024), `cargo` available.
- Repo root: `/Users/dragon/Development/internal/gobo`.

## Run the validation

```bash
cargo test --test unit buffer cursor render
```

(Or simply `cargo test` to run the full suite, including the existing
integration tests that act as regression guards.)

## Expected outcomes

The unit tests under `tests/unit/buffer.rs` and `tests/unit/cursor.rs`
(os märtied `# Fix Trailing Newline Cursor Position`, spec 008) must pass.

### Primary fix (the bug) — must pass after the fix

- `line_of_char(trailing_newline_doc, len_chars) == trailing_empty_line`:
  - `"abc\n"`        → cursor at end → line **1**.
  - `"abc\n\n"`      → cursor at end → line **2**.
  - `"abc\n\n\n"`    → cursor at end → line **3**.
  - `"\n"`           → cursor at end → line **1**.
- `visual_column` at end-of-doc after a trailing `\n` is **0** (start of the
  empty line), not the width of the previous line.
- `move_right` to end-of-doc then `visual_column`/line == empty trailing line
  (FR-003): the visible cursor is where the next insert lands.

### Regression guards — must still pass (unchanged)

- `"abc"` (no trailing newline), cursor at end → line 0, column 3.
- `""` (empty document), cursor at 0 → line 0, column 0.
- `"abc\n"`, cursor at index 3 (on the `\n`, i.e. end of `abc` line, *before*
  moving past it) → line 0 (FR-005).
- `move_left` from end-of-doc back over the trailing `\n` lands on line 0 at
  the end of `abc` (FR-004).
- Existing `tests/integration/*` (open/save/search/enter/undo) all green —
  persistence and edit flows unchanged (FR-007, out-of-scope save path).

## References

- Mapping contract: `contracts/cursor-line-mapping.md`.
- Data model: `data-model.md` (char_index ↔ line corrected table).
- Root-cause + decision detail: `research.md`.
- Implementation tasks: `tasks.md` (produced by `/speckit.tasks`).
