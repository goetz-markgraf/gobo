# Review: Filename in Footer Row (Spec 005)

**Reviewer**: pi coding agent
**Date**: 2026-07-02
**Spec**: `specs/005-filename-in-footer/spec.md`
**Plan**: `specs/005-filename-in-footer/plan.md` (authoritative revision 2026-07-02)
**Constitution**: `.specify/memory/constitution.md` v1.1.0

## Follow-up applied (2026-07-02)

After the initial review, the two non-blocking findings were addressed:

- **F1 resolved** — `plan.md` rewritten: the Detailed Design, Project Structure,
  Constitution Check, Testing summary, and Migration Checklist now describe the
  implemented single-row design (`prompt_lines` 1/2, `footer_line` field, no status
  row, dynamic constraints, reversed-color footer, no `Borders::TOP`). The stale
  two-row sections and the `+1`/`format_status_line` instructions are gone.
- **F2 resolved** — two edge-case tests added:
  - `tests/unit/render.rs::absolute_path_shown_verbatim_left` (US1-3)
  - `tests/unit/render.rs::non_ready_message_appears_on_right` (US1-6 secondary states)
  - `tests/integration/search_and_resize.rs::footer_carries_non_ready_message_after_failed_search`
    drives a real failed search and asserts the footer carries "No match" on the right.
- `cargo test` stays green (37 tests total, 0 failures).

## Summary

The implementation **conforms to the authoritative 2026-07-02 revision** of the spec,
matches the simplified single-row design, and passes the constitution check. All tests
compile and pass (`cargo test`: 35 tests across unit + integration targets, 0 failures).

The only caveat is a **drift between the lived plan body and the implemented design**:
the plan's "Detailed Design" sections describe the *original* two-row design
(body | status-line | footer), while the 2026-07-02 revision note at the top overrides
them. The implementation correctly follows the revision, not the stale section bodies
below. This makes the plan harder to read than it should be (Constitution Principle I).

## Spec → Implementation Traceability

| Spec item | Where implemented | Status |
|---|---|---|
| FR-001 filename left + ` (*)` dirty marker | `render.rs::format_footer_line` + `render_view()` populates from `document.path.display()` + `document.dirty` | ✅ |
| FR-002 always-visible 1-line footer; viewport budget reduced | `app.rs::prompt_lines` returns 1 (Editing) / 2 (SearchInput); `ViewportState::update_for_terminal` subtracts them | ✅ |
| FR-003 status message on the RIGHT of the same row; no separate status line, no mode/access | `status.rs::current_message` (returns `Ready` default); merged into `footer_line` by `format_footer_line`; no `format_status_line` exists | ✅ |
| FR-004 exactly 2 visible areas (body \| footer), no stray rows | `main.rs::paint` builds a dynamic `constraints` vec with no filler lines; old `status_line` chunk removed | ✅ |
| FR-005 SearchInput → body \| search-prompt \| footer | `view.bottom_line` adds a `Length(1)` constraint between body and footer; footer always last | ✅ |
| SC-001 / SC-002 path left + message right in all modes | Confirmed by `footer_filename_and_message_visible_in_one_row` and `search_prompt_and_footer_both_visible` (main.rs inline tests) | ✅ |
| SC-003 no stray empty / `-` rows | Dynamic constraint vector; no `Min(0)` / leftover chunks; `cargo test` passes | ✅ |
| SC-004 no overlap with body / search prompt / popups | Footer is a dedicated final chunk; popup renders via `Clear`+absolute `Rect` and `status.rs` keeps `bottom_line = None` while popup active | ✅ |
| SC-005 existing integration tests compile/pass | `cargo test` green | ✅ |
| Assumption: `document.path` holds original CLI arg, no canonicalization | `document.rs::open` stores `path.into()` without canonicalize; `render_view` uses `.display()` | ✅ |
| Assumption: truncate from left with `...` prefix when too wide | `truncate_left` walks graphemes from the right, prepends `...` | ✅ |

### User Story acceptance scenarios

| Scenario | Automated coverage | Status |
|---|---|---|
| US1-1 `gobo something.txt` → bottom row left shows `something.txt` | `footer_filename_and_message_visible_in_one_row` (main.rs), `clean_name_left_message_right` (unit) | ✅ |
| US1-2 `gobo somedir/something.txt` shown verbatim | `footer_shows_relative_path_verbatim` (integration), `relative_path_shown_verbatim_left` (unit) | ✅ |
| US1-3 absolute path `/Users/foo/bar.txt` shown verbatim | `absolute_path_shown_verbatim_left` (unit); also covered indirectly by the non-canonicalizing `open()` + `.display()` path | ✅
| US1-4 edit → `something.txt (*)` on left | `footer_dirty_marker_appears_on_edit_and_disappears_after_save` (integration), `dirty_path_appends_dirty_marker_on_left` (unit) | ✅ |
| US1-5 save → marker removed | same integration test asserts absence after `Save` | ✅ |
| US1-6 status message right in bottom row | `footer_filename_and_message_visible_in_one_row` asserts `Ready`; `non_ready_message_appears_on_right` (unit) + `footer_carries_non_ready_message_after_failed_search` (integration) cover non-`Ready` messages ("No match for …") | ✅ |

## Constitution Check (v1.1.0)

- **I. Readability First** — ✅ (after follow-up). `format_footer_line` is short and
  its branches are commented; helpers are purpose-named. `plan.md` now describes the
  implemented single-row design throughout — a reviewer reading top-to-bottom no longer
  hits contradictory two-row sections.
- **II. Maintainable Design** — ✅. Boundaries preserved: `render.rs` projects state,
  `status.rs` owns the message, `main.rs` owns layout. `footer_line` is derived only
  from already-present `document.path` + `document.dirty` + `status`. No new dependency.
- **III. Secure by Default** — ✅. No new I/O; footer only displays the path passed at
  open time. No content logged/transmitted externally.
- **IV. Verification Before Merge** — ✅. Coverage spans main flow + edge cases
  (truncation left, narrow terminal, overlong message dropped, empty path, dirty marker
  on/off, SearchInput layout) and now also covers the previously-untested absolute-path
  display and non-`Ready` status messages (US1-3, US1-6 secondary states).
- **V. Scope and Simplicity Control** — ✅. Single-row design is the simpler option and
  the revision explicitly chose it over the two-row design. No new moving parts.

## Findings

### F1 — Plan body drifts from implemented design (readability, Constitution I)
**Status**: ✅ Resolved (2026-07-02). `plan.md` Detailed Design / Project Structure /
Constitution Check / Testing / Migration Checklist rewritten under the authoritative
2026-07-02 revision to match the implemented single-row design.

### F2 — Missing edge-case tests (verification, Constitution IV)
**Status**: ✅ Resolved (2026-07-02). Added `absolute_path_shown_verbatim_left`,
`non_ready_message_appears_on_right` (unit), and `footer_carries_non_ready_message_after_failed_search`
(integration).

### F3 — Minor: `truncate_left` symmetric edge behavior
**Severity**: Informational. `truncate_left` prepends `...` even when the string already
fits (`if width <= max_width { return format!("...{}", s) }`). The caller guards against
this in `format_footer_line` only on the overflow path, so the early-return is unreachable
with current call sites but harmless. No action required unless the helper is reused elsewhere.

### F4 — Footer styling choice
**Severity**: Informational. `main.rs::paint` renders the footer with reversed colors
(`fg=Black, bg=White`) and no `Borders::TOP`, departing from the plan's stated
"Borders::TOP separator". The inline comment explains a 1-line bordered block would hide
the text. This is a reasonable deviation; document it in the plan if you keep it.

## Test Results

```
cargo test — all targets green, 0 failures.
New/affected targets after follow-up:
  unit_render            11 passed (was 9 — added 2)
  integration_search_and_resize  13 passed (was 12 — added 1)
Total across all targets: 37 passed, 0 failed
```

## Verdict

**Approved.** Follow-ups F1 and F2 are complete: the plan now documents the
implemented single-row design, and edge-case tests cover absolute-path display
and non-`Ready` footer messages. The implementation satisfies all functional
requirements and success criteria of the authoritative spec revision, respects
the constitution's readability/maintainability/security/scope/verification
principles, and `cargo test` is green (37 passed, 0 failed).

No outstanding non-blocking items beyond the informational F3/F4, which need no
action.
