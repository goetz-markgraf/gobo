# Phase 0 Research: Fix Trailing Newline Cursor Position

## R-1: Where does the wrong visual cursor line come from?

**Decision**: The bug is in `buffer::line_of_char`.

**Rationale**: `line_of_char` handles the end-of-document cursor specially:
when `char_index == text.len_chars()` it probes `char_index - 1` and asks
ropey `char_to_line(probe)`. For a document ending in `\n`, the probed
character *is* the trailing `\n`, and ropey's `char_to_line('\n')` returns the
line that **contains** the newline (e.g. line 0 for `"abc\n"`), not the
following empty line (line 1). Consequently `visual_column` / `render_view`
place the cursor one line too high and at the end of the previous (non-empty)
line — exactly the reported symptom.

Verified empirically with a ropey probe (see notes below):
- `"abc\n"`, cursor i=4 → `line_of_char` returns **0** (should be 1).
- `"abc\n\n"`, cursor i=5 → returns **1** (should be 2).
- `"\n"`, cursor i=1 → returns **0** (should be 1).
- `"abc"` (no newline), cursor i=3 → returns **0** (correct).
- `""`, cursor i=0 → returns **0** (correct).

Only the "cursor sits exactly after a trailing `\n` at end-of-doc" case is
wrong; everything else already maps correctly because the probed character is
a non-newline (or the doc is empty).

**Alternatives considered**:
- Patch `cursor::visual_column` or `render_view` to nudge the line.
  Rejected: both consume `line_of_char`; the single source of truth is the
  mapping function. Fixing it there cascades to all consumers and keeps one
  reason to change.
- Change the stored `char_index` so the cursor never sits "after" the trailing
  newline. Rejected: that would alter the logical insert position and risk
  the very insertion regression the spec forbids (FR-002/FR-003). The logical
  position is already correct; only the *display* line is wrong.

## R-2: ropey end-of-line / end-of-doc semantics

**Decision**: Treat `char_index == len_chars()` after a trailing `\n` as
belonging to the line **after** the newline.

**Rationale**: ropey models `"abc\n"` as 2 lines: line 0 = `"abc\n"`, line 1 =
`""` (the empty trailing line). The cursor position *after* the `\n` (the start
of the trailing empty line) logically lives on line 1. ropey's `char_to_line`
cannot represent this for an index equal to `len_chars()` because that index is
past the last character, so the existing code probes the preceding char — which
collapses onto line 0 when that char is the `\n`. The fix: if the probed line's
character is `\n` AND the cursor is at `len_chars()` (end of doc), step forward
one line.

**Alternatives considered**:
- Use `text.line_to_char(probe_line + 1)` equality with `len_chars()` to detect
  "cursor is at the start of a trailing empty line". Works but is more complex
  than checking the final character; same observable result.

## R-3: Impact on navigation and selection

**Decision**: No change needed in `cursor.rs` motions or `Selection`.

**Rationale**: `move_right` clamps to `len_chars()` and recomputes
`preferred_column` via `visual_column`, which itself calls the (fixed)
`line_of_char`. `move_left` from end-of-doc decrements `char_index` onto the
`\n` (still line 0 → correct, cursor at end of `abc`). So FR-004 (arrow
navigation back over the newline) and FR-005 (no regression before the
newline) hold automatically once `line_of_char` is correct. Selection motions
reuse the plain motions, so they inherit the fix.

**Alternatives considered**: None — the consume-the-mapping design already
guarantees cascade.

## R-4: Persistence / save safety

**Decision**: Out of scope; no save-path change or test required.

**Rationale**: Per clarification (2026-07-03), persistence stays unchanged.
The fix only adjusts char→line *display* mapping; it never mutates the `Rope`
content or the persisted bytes, so a trailing `\n` is saved exactly as before.
Existing save tests remain as regression protection (FR-007).

## R-5: Multi-newline and single-newline edge cases

**Decision**: The same end-of-doc branch handles all counts uniformly.

**Rationale**: The faulty mapping occurs only when the cursor is at
`len_chars()` and the final char is `\n`. That condition is independent of how
many trailing newlines precede it; stepping `probe_line + 1` always yields the
correct empty trailing line for `"abc\n"`, `"abc\n\n"`, `"abc\n\n\n"`, and
`"\n"`. For a cursor *between* two trailing newlines (not at len_chars), the
probed char is the preceding `\n` and ropey already returns the right line
(empirically: `"abc\n\n"` i=4 → line 1, correct).

## Empirical ropey probe notes

Full `char_to_line` table for representative inputs (excerpt):
```
"abc\n"   i=4 (len) probe=3('\n') -> line 0   [BUG: should be 1]
"abc\n\n" i=5 (len) probe=4('\n') -> line 1   [BUG: should be 2]
"\n"      i=1 (len) probe=0('\n') -> line 0   [BUG: should be 1]
"abc"     i=3 (len) probe=2('c')  -> line 0   [correct]
""        i=0 (len, empty)        -> 0         [correct]
```
The fix branch fires exactly on the four `[BUG]` rows and is a no-op for the
rest.
