# Data Model: Fix Trailing Newline Cursor Position

No new entities, fields, or persistence are introduced. This feature corrects
an existing *mapping* between two already-modelled concepts. Existing entities
unchanged; relationships below describe the corrected behavior.

## Entities

### Cursor (`CursorState`, unchanged)

| Field | Type | Notes |
|---|---|---|
| `char_index` | `usize` | Logical insert position in the Rope char stream. **Unchanged** by this fix. Range `0..=len_chars()`. |
| `preferred_column` | `usize` | Visual column remembered across up/down moves. **Unchanged**. |

Invariants preserved:
- `0 <= char_index <= text.len_chars()` (enforced by `clamp_char_index`).
- `char_index == text.len_chars()` denotes "end of document"; when the document
  ends with `\n` this is the explicit "cursor after the trailing newline"
  state at the center of this fix.

### Document (Rope, unchanged)

- Lines are ropey lines: `"abc\n"` has 2 lines (`"abc\n"`, `""`).
- A document ending in `\n` always has at least one trailing empty line.
- Content bytes are **not** modified by this feature (persistence out-of-scope).

## Relationship: char_index ↔ line (CORRECTED mapping)

`buffer::line_of_char(text, char_index)` is the single source of truth for the
char→line mapping and consumes by `cursor::visual_column` and
`render::render_view`. Corrected rules:

| Document | char_index | char at index-1 | line_of_char (corrected) | Before fix |
|---|---|---|---|---|
| `"abc\n"` | 4 (end) | `'\n'` | **1** | 0 ❌ |
| `"abc\n\n"` | 5 (end) | `'\n'` | **2** | 1 ❌ |
| `"abc\n\n\n"` | 6 (end) | `'\n'` | **3** | 2 ❌ |
| `"\n"` | 1 (end) | `'\n'` | **1** | 0 ❌ |
| `"abc"` | 3 (end) | `'c'` | 0 | 0 ✓ |
| `""` | 0 (empty) | — | 0 | 0 ✓ |
| `"abc\n"` | 3 (on `\n`, not end) | `'c'` | 0 | 0 ✓ |
| `"abc\n\n"` | 4 (on middle `\n`) | `'\n'` | 1 | 1 ✓ |

Rule (formal): if `text.len_chars() == 0` → `0`. Else if
`char_index == text.len_chars()` AND `text.char(len_chars()-1) == '\n'` →
`text.char_to_line(len_chars()-1) + 1`. Else existing behavior (probe and
`char_to_line`).

## State Transitions

No state machine changes. `SessionMode` and its transitions are unaffected.

## Validation Rules

- Logical insert position must equal displayed cursor position (FR-002):
  since only `line_of_char` changes and `char_index` is untouched, the insert
  position was already correct; the display now matches it.
- No regression for cursor *before* the trailing newline (FR-005): the new
  branch fires only when `char_index == len_chars()` and the final char is
  `\n`; any earlier index keeps the existing (correct) mapping.
- Persistence byte-identical (FR-007): no `Rope` mutation in the fix;
  existing save tests guard this.
