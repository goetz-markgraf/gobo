# Implementation Review Report — Stories 001–004

**Reviewer**: automated code review pass
**Date**: 2026-06-30
**Scope**: shipped implementation of the first four features, evaluated in this order: correctness → readability → maintainability → test coverage & test logic → security.

**Head commit reviewed**: `6812bbc 004-fix`
**Build status**: `cargo build` clean; `cargo clippy` → 1 warning.
**Test status**: all 51 tests pass (16 unit-search, 3 unit-cursor, 2 unit-buffer, 28 integration across 4 files). *Passing tests do not imply correctness* — see correctness section for a panic bug that no existing test exercises.

---

## 1. Correctness

### 1.1 🔴 HIGH — `SearchState::find_next` panics on UTF-8 whose lowercase form changes byte length

`src/editor/search.rs` converts the rope to a `String`, lowercases the **whole haystack**, runs byte-space `str::find`, and then converts byte positions back to char indices on the **original (un-lowercased)** string:

```rust
let haystack = text.to_string();
let haystack_cmp = normalize(&haystack, &self.case_mode);   // may have different length
...
let byte_pos = *all_matches.get(next_idx).unwrap();
let byte_end = byte_pos + needle_cmp.len();
let cs = haystack[..byte_pos].chars().count();   // <-- slices ORIGINAL string
let ce = haystack[..byte_end].chars().count();
```

Because `to_lowercase()` can change byte length (e.g. `İ` U+0130 → `i\u{307}`, 2 bytes → 3 bytes; `ß` uppercase rules; Greek “Σ” → “σ”/“ς”), byte offsets found in `haystack_cmp` do **not** align to char boundaries in `haystack`. The slice `haystack[..byte_end]` then panics with `byte index ... is not a char boundary`.

**Reproduced** with a one-line test (appended temporarily to `tests/unit/search.rs`):

```
thread 'bug_lowercase_byte_mismatch' panicked at src/editor/search.rs:95:29:
end byte index 1 is not a char boundary; it is inside 'İ' (bytes 0..2 of string)
```

**Spec impact**: breaks FR-010 (UTF-8 / multi-byte grapheme support) of story 004 and FR-010a (UTF-8 plain-text files) of story 001. Pure-ASCII queries on pure-ASCII documents (everything the test-suite covers) never trigger it, so the test suite is green but the feature is unsafe for the UTF-8 content the specs explicitly target.

**Fix direction**: perform the search in **char space**, not byte space, and never slice a non-lowercased string with offsets derived from a lowercased one. Compare character-by-character by lowercasing each side into an iterator and matching on the lowered char streams, recording match positions as char indices in the original rope. (Or use `ropey`’s char iteration directly to avoid ever materializing a `String`.)

### 1.2 🟠 MEDIUM — `EditorCommand::FindNext` from editing mode uses stale `last_match_char_range`

`find_next` computes its starting point as
`base = last_end_byte.max(start_byte)`, where `last_end_byte` is the **end of the previous Ctrl‑G / Enter result**. In editing mode the user can move the cursor freely after a confirmed search. If the user moves the cursor **backward** below the stored last-match end, the next Ctrl‑G still advances from the old `last_end_byte`, skipping matches that lie between the cursor and that end.

Example: document `alpha … alpha … alpha`, confirmed search lands cursor at match #2 (`last_match = (11..16)`); user moves cursor to char 2 and presses Ctrl‑G. Expected per FR-005 (“find the next occurrence … from the cursor position forward”): jump to match #2 (the next one at/after 2 in char 11). Actual: jumps to match #3 (23), because `base = max(byte(16), byte(2)) = byte(16)`.

Spec 004 FR-005/FR-006 do not fix this behavior precisely, but the natural reading of “from the cursor position forward” is violated. Existing tests never move the cursor backward before Ctrl‑G, so this is uncovered.

### 1.3 🟠 MEDIUM — Render layout off-by-one in `SearchInput` (FR-009)

Two independent computations disagree about how many rows the bottom occupies during search:

- `app::prompt_lines()` returns **2** for `SearchInput`, and `ViewportState::update_for_terminal` sets `visible_height = size.height - 2` (so the body is treated as `height - 2` rows tall).
- `main.rs::draw` builds the layout as `[Min(1), Length(1) status, Length(prompt_height)=2 search]`, reserving **1 (status) + 2 (search) = 3** bottom rows.

`prompt_lines()` semantically counts *status + search combined* (2), but `draw` adds the status row again *and* reserves `Length(prompt_height)=2` for the search block. Net result: the body area shown by ratatui is `height - 3`, while `render_view()` produces `height - 2` body lines. The extra body line is clipped, so one line of document content is hidden during search.

Spec 004 FR-009 explicitly says: “The status area above the search prompt MUST remain at 1 line; the search prompt occupies 1 additional line only when in SearchInput mode (total of 2 bottom areas during search).” The current layout reserves 3. Fix: in `draw`, use `Constraint::Length(1)` for the search block (not `Length(prompt_height)`), keeping `prompt_lines()` as the *total* bottom budget that the viewport already subtracts.

### 1.4 🟢 LOW — status line persisted with `search.query` left over after cancel/clean-quit

- After pressing `Esc` in search, mode returns to `Editing` but `self.search` (with its query) is **not** cleared. A subsequent `Ctrl‑F` re-enters search with the *old* query in the prompt. The spec assumption only requires the query to be cleared on file switch, so this is technically allowed, but it is surprising UX.
- `request_quit` / `dismiss_prompt` likewise leave `search` populated. Harmless today, but a hidden stale query can interact with 1.2 above.

### 1.5 🟢 LOW — `Enter` search confirm message text differs from spec example

FR-003 suggests the message form “Match at (10..15)”; the implementation prints `Match found at 6..11` for Enter and `Match at 0..5` for Ctrl‑G (two different formats for the same concept). Both are human-readable, so this is cosmetic, but the inconsistency is a small correctness/readability smell.

### 1.6 🟢 LOW — `handle_command` overwrite of status during prompts/search

`EditorCommand::Resize` is intercepted *before* the prompt check and unconditionally sets `status = "Resized to …"`. When a resize arrives while an unsaved-changes warning status is showing, the helpful warning message is overwritten by the resize info. Also, resize during `SearchInput` works (intercepted early), which is correct, but the status reset still clobbers the in-flight search status message.

### 1.7 🟢 LOW — dead code in `handle_editing_command` match arms

The arms `| EditorCommand::NextChoice | EditorCommand::PreviousChoice | EditorCommand::Resize(_) => {}` and the earlier `| Cancel | FindNext => {}` are unreachable in practice (`Resize` is intercepted in `handle_command`; `NextChoice/PreviousChoice` are only sent inside prompts). They are harmless but muddy the dispatch story and mislead future maintainers.

---

## 2. Readability

### 2.1 🟠 MEDIUM — `handle_editing_command` handles `FindNext` twice

The match contains `| EditorCommand::Cancel | EditorCommand::FindNext => {}` (no-op) and then, *after* the match, a second `if let EditorCommand::FindNext = command { … }` block implements the real logic:

```rust
match command {
    ...
    | EditorCommand::Cancel | EditorCommand::FindNext => {},
    ...
}
// Handle FindNext from editing mode (cursor jumps to next search match)
if let EditorCommand::FindNext = command { ... }
```

The first arm is dead; the real handler is a dangling `if let` after the match with inconsistent indentation. This is the single most confusing spot in `app.rs`. Fix: lift `FindNext` into a normal match arm with the real body (assigned to `self.status` / cursor updates) and drop the trailing `if let`.

### 2.2 🟠 MEDIUM — `src/editor/search.rs` is poorly formatted

The file has wildly varying indentation (3/4/6-space blocks mid-expression), trailing whitespace, and a `return Some((cs, ce));` that triggers the lone clippy warning. The whole `find_next` body looks like it was edited in many passes without reformatting. A `cargo fmt` pass would materially improve readability.

### 2.3 🟢 LOW — `EditorCommand::PreviousChoice` is mis-aligned in `input.rs`

```rust
    NextChoice,
PreviousChoice,
    FindNext,
```

One enum variant is flush-left while the others are indented. Cosmetic only, but it stands out.

### 2.4 🟢 LOW — magic status strings duplicated across `app.rs`

The same status strings (“No match”, “Match at …”, “Search cancelled”) are repeated as string literals in `handle_search_command`, `handle_editing_command`’s `FindNext` branch, etc. There is no single source of truth for the search-result text format, which is why 1.5 above diverged. Extracting one helper would improve both readability and consistency.

---

## 3. Maintainability

### 3.1 🟠 MEDIUM — search logic mixes normalization, byte-space find, char-space return, and state mutation in one ~70-line method

`find_next` does (a) empty-query guard, (b) rope→String, (c) lowercase both sides, (d) collect all byte matches, (e) compute `base` from a mix of the previous match’s *char range* and the requested *char index*, (f) byte→char back-mapping, (g) mutate `self.last_match_char_range` / `self.last_result`, all in one body. This is hard to reason about (and is exactly what hides the panic in 1.1). Splitting into “find all matches (char indices)”, “choose next index given last state”, and “update state” would localize each concern and make the UTF-8 fix straightforward.

### 3.2 🟠 MEDIUM — duplicate search-result handling between `handle_search_command` and `handle_editing_command`

Both functions contain near-identical `find_next → Some/None → set cursor + status` blocks. This duplication is how the “Match found” vs “Match at” text divergence (1.5) was introduced and is the reason the FindNext-from-editing path is bolted on as an `if let`. A single private method like `fn advance_to_next_match(&mut self) -> Result<(), DocumentError>` would serve both call sites and remove the duplicated logic.

### 3.3 🟢 LOW — `make_session` in `tests/integration/enter_newline.rs` opens a hardcoded shared `/tmp/...md` path

```rust
DocumentBuffer::open("/tmp/enter-test.md").unwrap()
```

The helper works only because `/tmp/enter-test.md` does not exist (the `open` falls back to an empty buffer on `NotFound`). If any developer (or a parallel test run) ever creates that file, several tests silently start testing against *stale content* rather than failing cleanly. This is an environmental coupling, not a test-logic bug, but it makes the suite fragile. Use a `tempdir()` like the other integration fixtures do.

### 3.4 🟢 LOW — `prompt_lines()` / viewport / `draw` layout coupling is implicit

The “bottom area = status + optional search” budget is encoded in three places (`prompt_lines()`, `ViewportState::update_for_terminal`, and `main.rs::draw`). The off-by-one in 1.3 is a direct symptom of this implicit contract. Consider a single `fn bottom_height(&self) -> u16` that both the viewport and the render loop consult.

---

## 4. Test coverage and test logic

### 4.1 🔴 HIGH — panic-correctness bug (1.1) is completely uncovered

The unit tests cover: case-insensitive ASCII, no-match, empty-query, single-char, wrap-around, multi-byte CJK. None of them use a character whose `to_lowercase` changes byte length (`İ`, `ß`→`ss` in uppercase queries, Greek sigma). The plan’s Verification Gate (“T002 tests CJK/emoji grapheme matching and invalid UTF-8 resilience”) overstates actual coverage. Add a test like `search_with_İ_lowercase_expansion_does_not_panic` to lock the fix.

### 4.2 🟠 MEDIUM — misleading/incomplete “FindNext” unit tests

`find_next_after_cursor_at_first_match_finds_second` (named to assert “finds second”) actually only asserts `result1 == Some((0,5))` and *prints* `result2` via `println!`. The inline comment even says “This is what currently happens: still returns Some((0,5)) … But for Ctrl-G to work, it should return Some((12, 17)) or similar.” The test documents a known bug it does not assert against — it is a debug probe left in the suite. Same for `debug_find_next_behavior`, which is a `println!`-only test. Both should either be turned into real assertions or removed.

### 4.3 🟠 MEDIUM — `find_next_jump_to_next_match_via_command` (T010) is a no-op assertion test

The test enters search, types “alpha”, presses Enter, stores `_initial_cursor`, and then ends — it never calls `EditorCommand::FindNext`. Its name and the spec’s T010 reference imply it verifies Ctrl‑G navigation, but it actually only re-verifies the Enter-confirm path already covered by `search_full_flow_confirms_first_match_and_exits`. Effective coverage for “from-search-mode Ctrl‑G jumps to next match” (only present in `ctrlg_with_empty_query`, which asserts it stays in mode + no cursor move) is thin.

### 4.4 🟢 LOW — missing negative / edge tests for story 003 acceptance

Spec 003 Edge Cases list: pressing Enter on a whitespace-only line, and at the very start of a document (no existing lines). `enter_at_start_of_text` covers insertion at index 0 of a non-empty doc, and `enter_empty_doc_creates_newline` covers the empty doc. But there is no explicit test for a *whitespace-only* line split (the spec answer says no trimming should occur). Worth adding for SC-003 / “no data loss or corruption”.

### 4.5 🟢 LOW — no test for the search-during-unsaved-popup precedence from a fresh search

`quit_popup_takes_precedence_over_active_search` covers the case where search *was already started* before quit (mode was `SearchInput`). There is no test for `Ctrl-Q` pressed while `mode == Searching` but `pending_prompt` is fresh — trivially handled by the same code, but spec 002 FR-009 (“prompt takes precedence over transient UI”) would benefit from an explicit assertion that the search bottom-line is suppressed (it is, via `render_view`).

### 4.6 🟢 LOW — `prompt_lines()` / viewport height contract has no test

FR-009 mandates 1 status + 1 search line = 2 total bottom during search, but no test asserts `visible_height == size.height - 2` specifically in `SessionMode::SearchInput`. Adding one would have surfaced the off-by-one in 1.3 immediately.

---

## 5. Security

### 5.1 🟠 MEDIUM — non-atomic file save can corrupt the target on interruption

`DocumentBuffer::write_to_disk` writes directly via `fs::write(&self.path, output)`. If the process is killed, the disk fills, or an I/O error occurs mid-write, the target file may be left partially overwritten (existing content destroyed, new content truncated). FR-006/SC-004 require that failed saves leave “the previously stored file content intact.” On a crash mid-`fs::write`, this guarantee is **not** upheld.

**Fix**: write to a temporary file in the same directory and `fs::rename` it over the target (atomic on POSIX local filesystems). On failure the original is untouched. This also satisfies FR-004 (save must not silently corrupt).

### 5.2 🟢 LOW — `DiskSnapshot` reads and re-hashes the whole file on every save

`has_external_change` calls `snapshot_for_path`, which `fs::read`s the full file and hashes all bytes. This is O(n) per save and doubles I/O for large (1 MB) files. Not a security hole, but the hashing also silently re-reads the file (TOCTOU window between `has_external_change` and `write_to_disk`). A size/mtime-based check, or hashing only when mtime/size changed, would close the window and reduce I/O. Out of scope for a v1 hardening pass but worth flagging.

### 5.3 🟢 LOW — path handling: symlinked/directory edge cases acceptable

`DocumentBuffer::open` rejects directories early and surfaces `InvalidUtf8` clearly (FR-010a). `fs::write` does not follow symlinks specially; on Unix this is standard. No path-injection or traversal risk exists because the path comes straight from argv and is never used to construct commands. No security issue; recorded for completeness.

### 5.4 🟢 LOW — no input-length bounds on search query / status line

`search.query` can grow without bound, and `format_status_line` concatenates `path | access | dirty | mode | message` without truncation. A very long query or path will simply be clipped by ratatui at draw time (no overflow), so there is no memory-safety risk, just visual noise. No action required for this release.

---

## Summary by severity

| Severity | Count | Items |
|---|---|---|
| 🔴 High | 2 | 1.1 search panic on Unicode lowercase expansion; 4.1 that bug is uncovered by tests |
| 🟠 Medium | 8 | 1.2 stale-match Ctrl‑G; 1.3 layout off-by-one; 2.1/2.2 readability; 3.1/3.2 maintainability; 4.2/4.3 misleading tests |
| 🟢 Low | 9 | 1.4/1.5/1.6/1.7; 2.3/2.4; 3.3/3.4; 4.4/4.5/4.6; 5.2/5.3/5.4 |
| 🔴/🟠 Security | 1 medium | 5.1 non-atomic save vs SC-004 |

## Top recommended fixes (in order)

1. **Rewrite `find_next` to operate in char-space** so it cannot panic on multi-byte lowercase expansion (fixes 1.1 + 4.1 + much of 3.1). Add a regression test with `İ`.
2. **Use atomic save (`write-to-temp + rename`)** so a failed/interrupted save leaves the previous content intact (5.1; closes SC-004).
3. **Fix the search-mode layout off-by-one**: reserve `Length(1)` for the search block so the bottom totals 2, matching `prompt_lines() == 2` and FR-009 (1.3). Add a `prompt_lines`/`visible_height` test.
4. **Deduplicate the FindNext handling** into one private method on `EditingSession`, remove the trailing `if let` and dead match arms (1.7, 2.1, 3.2, 1.5).
5. **Change Ctrl‑G base to ignore stale `last_match_char_range` when the cursor is before it** (or store cursor-at-confirm instead of last match end) to honor FR-005 literally (1.2).
6. **Clean up the search suite**: convert the two `println!` “debug” tests into real assertions or delete them; make `find_next_jump_to_next_match_via_command` actually press Ctrl‑G (4.2, 4.3).
7. Run `cargo fmt` and clear the clippy warning (2.2, 2.3).
8. Replace the `/tmp/*.md` fixture in the enter-newline integration tests with a `tempdir()` (3.3).

Resolving items 1–3 closes the spec-level correctness regressions (FR-010/010a, FR-009, SC-004); the remaining items are quality hardening.
