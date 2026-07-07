# Contract: Help Dialog Key Binding Mapping

**Generated**: 2026-07-07

## Overview

This contract defines the exact key binding → label mapping displayed by the Help Dialog (Ctrl-H). The content is **static** — built once from `src/editor/input.rs` mappings — and never varies at runtime.

Only **Ctrl-key combinations** active during text editing are shown. Low-level keys (Tab, Enter, Backspace, Delete, Shift) are handled by the terminal/OS and excluded per FR-002.

### Binding List (flat, no categories)

| Key | EditorCommand | Help Label |
|-----|---------------|------------|
| Ctrl-F | Search | Find in document |
| Ctrl-G | FindNext | Find next match |
| Ctrl-S | Save | Save document |
| Ctrl-Z | Undo | Undo last edit |
| Ctrl-Y | Redo | Redo last undone edit |
| Ctrl-C | Copy | Copy selection to clipboard |
| Ctrl-X | Cut | Cut selection to clipboard |
| Ctrl-V | Paste | Paste from clipboard |
| Ctrl-Q | Quit | Quit (clean) / save prompt (dirty) |

## Contract Guarantees

1. **Completeness**: Every `Ctrl-<key>` binding in `src/editor/input.rs` that is active during editing MUST have an entry. Omission = bug.
2. **Immutability**: The list does not change based on session state, mode, or file type.
3. **One-to-one mapping**: Each command produces exactly one row; no duplicates.
4. **Ordering**: Display order follows the order listed above unless a constitution violation prevents it.

## Validation Criterion

Cross-reference `src/editor/input.rs` `map_key_event()` match arms for `KeyModifiers::CONTROL`: every such arm MUST have a corresponding help entry, and vice versa.
