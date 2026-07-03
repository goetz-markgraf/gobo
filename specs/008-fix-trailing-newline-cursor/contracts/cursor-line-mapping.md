# Contract: `buffer::line_of_char` char→line mapping

**Owner**: `src/editor/buffer.rs`
**Consumers**: `cursor::visual_column`, `cursor::char_index_for_visual_column`
(via `line_start_char`), `render::render_view` (cursor_y), viewport clamping.

## Signature (unchanged)

```rust
pub fn line_of_char(text: &Rope, char_index: usize) -> usize
```

## Behavior contract (corrected)

Given `n = text.len_chars()`, `idx = clamp_char_index(text, char_index)`:

1. If `n == 0` → return `0` (empty document is a single virtual line 0).
2. If `idx == n` AND the final character of the document is `\n`
   (`text.char(n - 1) == '\n'`) → return `text.char_to_line(n - 1) + 1`
   (the empty trailing line created by the final newline).
3. Otherwise → return `text.char_to_line(probe)`, where
   `probe = if idx == n { n - 1 } else { idx }` (existing pre-fix behavior).

## Required invariants

- Pure: no mutation of `text`.
- Total over `0..=len_chars()`; never panics for any in-range `char_index`.
- Monotonically non-decreasing in `char_index` within a document: moving the
  cursor right never decreases the reported line.
- Identity on already-correct cases: indices that are **not** `len_chars()`
  are unchanged from pre-fix behavior (regression safety, FR-005).
- End-of-doc trailing-newline case must return the line after the final `\n`
  (FR-001, FR-002, FR-006).

## Out of scope

- Does not touch `char_index` values stored in `CursorState` (logical
  insert position is already correct; only the display line is fixed).
- No persistence / save behavior change (FR-007 guarded by existing tests).
