# Data Model: Clipboard Cut, Copy & Paste

## Context

This document describes the data shapes involved in clipboard operations. The editor does **not**
own any clipboard state — all three commands are transient operations that read from or write to
the OS-provided system clipboard via `arboard`.

## Entities

### EditorCommand (clipboard-related variants)

Extension of the existing `EditorCommand` enum in `src/editor/input.rs`.

| Variant | Description |
|---|---|
| `Copy` | Copy selected text or single grapheme to OS clipboard |
| `Cut` | Copy selected text or single grapheme to OS clipboard, then delete from buffer |
| `Paste` | Read OS clipboard text and insert at cursor position (replaces selection if any) |

**Fields**: None. Variants carry no state; the clipboard content flows through local variables
in the command handler.

### Clipboard Data Shape

Internal transfer type between `clipboard.rs` functions and `app.rs` handlers.

| Field | Type | Description |
|---|---|---|
| `text` | `String` | UTF-8 text content from OS clipboard (only on success) |
| `len_bytes` | `usize` | Byte length, used for 1 MB limit check |

**Invariants**:
- `text` is always valid UTF-8 (enforced by `arboard::get_text()` returning `Option<String>`)
- `len_bytes ≤ 1_048_576` (1 MB hard cap enforced before any insertion)
- Empty `String` from clipboard → treated as no-op (FR-009)

### CutState (internal concept — not a struct)

A logical concept, **not** persisted as a struct. When Ctrl-X fires:

1. The source text is captured into a local variable (`removed_text`)
2. `arboard.set_text(removed_text.clone())` writes to OS clipboard
3. A single `EditStep::Delete` (or `EditStep::Replace` if selection) records the deletion
4. `removed_text` goes out of scope after record() returns

The `EditStep` alone carries the undo information. The OS clipboard copy is a side effect that
persists per FR-006.

### ClipboardState (external — not managed by editor)

| Attribute | Description |
|---|---|
| Owner | Operating system (singleton display clipboard) |
| Lifetime | Indefinite, controlled entirely by the OS and other processes |
| Content | UTF-8 text or binary; only `Some(String)` from `get_text()` is accepted |
| Editor relationship | Read-only access via ARBOARD at paste time; write access at Copy/Cut time |
| Consistency guarantee | None — any process can modify clipboard between commands. Paste always reads the **current** clipboard value (**FR-012**) |

## Relationships

```
app.rs (handler) ──── ClipboardData (local variable) ────→ arboard::DisplayClipboard (OS singleton)
     │                                                              │
     ├───────── EditStep::Delete/Replace (undo record)             │
     │                                                              │
buffer → rope text mutation (insert/delete/replace)      [no state stored]
```

No persistent data model changes. All clipboard data lives exclusively in:
- **Memory**: local variables during command dispatch
- **OS**: `arboard::DisplayClipboard` (managed by OS, not the editor)

## Validation Rules

| Rule | Source | Enforcement |
|---|---|---|
| Only UTF-8 text accepted from clipboard | FR-008, FR-009 | `get_text()` → `Option<String>`; `None` = no-op |
| Max 1 MB paste | FR-013, SC-05 | Checked on read (paste) and write (copy/cut) |
| Grapheme-cluster granularity for no-selection cut/copy | FR-001, FR-002, FR-003 | `unicode_segmentation::GraphemeClusterIter` |
