# Quickstart: Validate Visible Search Popup & Ctrl+G Find-Next

This guide walks through manual validation scenarios that prove the visible search popup feature works end-to-end. Run these inside a terminal with gobo open.

## Prerequisites

- Build the project: `cd /Users/dragon/Development/internal/gobo && cargo build`
- Have a test file with repeated text, e.g.:
  ```bash
  echo -e "Hello World\nSecond line\nhello again\nThird\nHELLO once more" > /tmp/test_search.txt
  ```

---

## Scenario 1 — Search prompt is visible (FR-001, FR-002)

**Steps:**

1. Open gobo with the test file: `cargo run -- -f /tmp/test_search.txt`
2. Press **Ctrl+F**.
3. **Expected**: The text `Search: ` appears on the last screen line, in yellow foreground color. A blinking cursor is visible after the colon. Text is NOT transparent or blended into the background.
4. Type `hello`. Each character (`h`, `e`, `l`, `l`, `o`) appears appended to `Search: hello` immediately.

**Validation**: The search prompt text is clearly readable at all times during typing.

---

## Scenario 2 — Enter confirms search and finds match (FR-003, SC-001)

**Steps:**

1. After entering `hello` in the search field (from Scenario 1), press **Enter**.
2. **Expected**: Cursor jumps to the first lowercase "hello" in the document. A status message like `Match found at 0..5` appears briefly, then clears. The editor returns to editing mode. No persistent highlights remain.

**Validation**: Search result is immediate (within one frame render cycle). Prompt exits cleanly.

---

## Scenario 3 — Enter with empty query exits silently (FR-004a)

**Steps:**

1. Press **Ctrl+F**. Do NOT type anything.
2. Press **Enter**.
3. **Expected**: Search mode exits silently. No status message displayed. Cursor stays at original position. Editor returns to editing mode.

**Validation**: Zero-message exit on empty query confirmed.

---

## Scenario 4 — No-match feedback (FR-007)

**Steps:**

1. Press **Ctrl+F**, type `zzzznotexist`.
2. Press **Enter**.
3. **Expected**: Status message shows `No match for zzzznotexist`. Cursor does NOT move. Editor returns to editing mode.

---

## Scenario 5 — Cancel search (FR-004)

**Steps:**

1. Press **Ctrl+F**, type some text.
2. Press **Esc**.
3. **Expected**: Search cursor exits. No status message shown. Original cursor position preserved. Editor returns to editing mode.

---

## Scenario 6 — Find next with Ctrl+G (FR-005, FR-006)

These steps assume you are still in search mode with query `hello` typed (from Scenario 1 or repeat it).

**Steps:**

1. Press **Ctrl+F**, type `hello`, press **Enter** once to confirm first match.
2. While still in search mode, press **Ctrl+G**.
3. **Expected**: Cursor jumps to the next occurrence of "hello" (the second `hello` in the document). Status message says `Match at <start>..<end>`. Editor remains in search mode.
4. Press **Ctrl+G** again.
5. **Expected**: Cursor wraps to the third match ("HELLO" from `"HELLO once more"` — case-insensitive). Status shows the new position.
6. Press **Ctrl+G** repeatedly until wrapping back to the first match.
7. **Expected**: After the last physical occurrence, wrap-around brings cursor back to the first match at the top of the document.

**Validation**: Wrap-around behavior is correct (FR-006). Over 95% match success rate across test documents (SC-004).

---

## Scenario 7 — Case-insensitive search (User Story 3)

**Steps:**

1. Press **Ctrl+F**, type `HELLO` (all caps).
2. Press **Enter**.
3. **Expected**: The first lowercase "hello" is found at position 0 in the document. Status shows it was found. Query is displayed as `HELLO` (capital letters are preserved on screen even though match was case-insensitive internally).

**Validation**: Case-insensitive matching confirmed; query display preserves original casing (SC-003).

---

## Scenario 8 — No-match for Ctrl+G when no instances exist

**Steps:**

1. Press **Ctrl+F**, type a query that matches zero times, press **Enter**.
2. While in search mode, press **Ctrl+G**.
3. **Expected**: Status message shows "No match". Cursor does NOT move. Editor stays in search mode.

---

## Automated test coverage (SC-005)

Run the tests to verify automated validation:

```bash
cd /Users/dragon/Development/internal/gobo

# Unit tests for search state & find_next logic
cargo test --lib search::

# Run all tests including integration scenarios
cargo test
```

Expected pass results cover:
- Case-insensitive matching in `SearchState::find_next()`
- Wrap-around behavior from end-of-document to beginning
- Empty query returns no match / idle state
- No-match state persistence across multiple calls
