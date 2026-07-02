# Quickstart: Undo / Redo

**Branch**: `006-undo-redo` | **Date**: 2026-07-02 | **Spec**: [spec.md](./spec.md)

Validation guide proving the Undo/Redo feature works end-to-end. See [contracts/api.md](./contracts/api.md) for the programmatic interface and [data-model.md](./data-model.md) for type details. Implementation lives in the implementation phase (`tasks.md`).

---

## Prerequisites

- Rust toolchain (stable, edition 2024). Verify: `cargo --version`.
- A working checkout of the `006-undo-redo` branch.

## Build

```bash
cargo build            # must compile with zero errors
cargo clippy --all-targets -- -D warnings   # project lints pass
```

## Automated validation

### Unit tests — `History` invariants

```bash
cargo test --test unit_history
```

Expected: pure tests on `History` + `Rope` pass. Coverage target:
- `Insert` reverse = `Delete` and vice versa (round-trip identity on the rope).
- `undo` then `redo` is a no-op on rope content and cursor.
- `record` clears the redo stack.
- `record` at capacity evicts the **oldest** undo step and reports `oldest_dropped == true`.
- `undo`/`redo` return `None` and mutate nothing when their stack is empty.

### Integration tests — `EditingSession` end-to-end

```bash
cargo test --test integration_undo_redo
```

Expected: every acceptance scenario below passes through `EditingSession::open()` + `handle_command()` with no terminal rendering, per the project's integration-test convention.

| Scenario | Commands | Expected result |
|----------|----------|-----------------|
| US1-A1: empty doc, undo back to empty | open seed (`""`) → InsertChar a,b,c → Undo ×3 | `document.text == ""`, `history.undo` empty |
| US1-A2: delete last char then undo | open seed (`"Hallo"`) → move to end → Backspace → Undo | `document.text == "Hallo"`, cursor restored |
| US1-A3: k-fold undo = state after (n−k)th edit | n edits → k undos | text equals the state after the (n−k)th edit |
| US2-A1: 3 undos then 3 redos | build 3 edits → Undo ×3 → Redo ×3 | text == final edited state |
| US2-A2: undo to origin then redo once | undo to empty → Redo once | text == state after first edit |
| US2-A3: redo at empty redo stack | redo all → Redo again | no-op, text/status/dirty unchanged |
| US3-A1: redo cleared on new edit | "a","b" → Undo → InsertChar 'x' → Redo | text == "ax", Redo is no-op, `history.redo` empty |
| US3-A2: new edit after multiple undos, redo no-op | several undos → new edit → Redo | no restoration; only the new edit is undoable |
| US4-A1/A2: session lifetime | build history, drop session, `open()` same file afresh | new session has empty undo **and** redo; Undo/Redo are no-ops until new edits |
| Edge: undo at empty stack | Undo on fresh session | complete no-op (text, cursor, dirty, status unchanged) |
| Edge: redo at empty stack | Redo on fresh session | complete no-op |
| Edge: Unicode + newlines | InsertChar with multibyte char, Enter, then Undo/Redo | byte-identical restoration incl. cursor index |
| Edge: large single insert | insert big string in one step, Undo, Redo | single step, no stutter, exact restore |
| Edge: Undo/Redo in SearchInput / prompts | enter SearchInput → Ctrl-Z/Ctrl-Y | ignored, no effect on buffer or history |
| FR-006 / SC-007: memory pressure | `History::with_capacity(2)` via test hook; record 3 steps | 3rd `record` reports `oldest_dropped == true`; rope/edit intact; status message is the "History truncated to free memory" warning |
| FR-013: save preserves history | build history → Save → Undo still works | Undo restores pre-save edits; `history.undo` non-empty after save |
| FR-009: read-only doc | open read-only file, Undo/Redo | no panic; stacks empty so no-ops |

Run the **whole** suite together to catch regressions in pre-existing behavior:

```bash
cargo test
```

Expected: all existing tests (`integration_open_and_save`, `unsaved_guards`, `readonly_and_conflict`, `search_and_resize`, `enter_newline`, `unit_buffer`, `unit_cursor`, `unit_search`, `unit_render`) **and** the two new test targets pass.

## Manual terminal UX check (not automatable)

These exercise the live TUI render path and cannot be repeated deterministically in automation; they supplement, not replace, the automated coverage.

1. `cargo run -- tmp.txt` (create an empty file).
2. Type `a`, `b`, `c` — confirm the buffer shows `abc`.
3. Press `Ctrl-Z` three times — confirm the buffer returns to empty, then `abc`, `ab`, `a`, `""` step by step (visible content + footer status "Undo").
4. Press `Ctrl-Y` three times — confirm `abc` is rebuilt ("Redo" status each step).
5. Press `Ctrl-Z` once, type `x`, then press `Ctrl-Y` — confirm `Ctrl-Y` is a no-op (Redo stack cleared) and the footer does not show restoration.
6. Trigger the memory-pressure path (testable path only): not reachable from the live UI in production (capacity defaults to `usize::MAX`); verify via the automated FR-006/SC-007 test instead.
7. `Ctrl-F` to enter search, then `Ctrl-Z` / `Ctrl-Y` — confirm nothing happens (mode-gated), search query unaffected.
8. Quit and reopen `tmp.txt` — confirm `Ctrl-Z` / `Ctrl-Y` have no effect until new edits are made.

## Done-when checklist

- [X] `cargo build` succeeds.
- [X] `cargo test` is fully green, including `unit_history` and `integration_undo_redo`.
- [X] Every acceptance scenario row above has a passing automated test.
- [ ] Manual TUI checks 1–5, 7, 8 behave as described (requires live terminal; not run in automation).
