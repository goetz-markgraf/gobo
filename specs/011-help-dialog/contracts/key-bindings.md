# Contract: Help Dialog Key Binding Mapping

**Generated**: 2026-07-07

## Overview

This contract defines the exact key binding → label mapping displayed by the Help Dialog (Ctrl-H). The content is **static** — built once from `src/editor/input.rs` mappings — and never varies at runtime.

## Binding List

| Key | EditorCommand | Help Label | Category |
|-----|---------------|------------|----------|
| Ctrl-F | Search | Find in document | Navigation |
| Ctrl-G | FindNext | Find Next | Navigation |
| Tab | Tab | Insert tab (spaces) | Editing |
| Shift+Tab | PreviousChoice | Tab back | Editing |
| Enter | Enter | Newline + auto-indention | Editing |
| Backspace | Backspace | Delete previous char | Editing |
| Delete | Delete | Delete next char | Editing |
| Shift+↑ | MoveSelectUp | Select upwards | Selection |
| Shift+↓ | MoveSelectDown | Select downwards | Selection |
| Shift+← | MoveSelectLeft | Select left | Selection |
| Shift+→ | MoveSelectRight | Select right | Selection |
| Ctrl-S | Save | Save document | Document |
| Ctrl-Z | Undo | Undo last edit | Undo / Redo |
| Ctrl-Y | Redo | Redo last undone edit | Undo / Redo |
| Ctrl-C | Copy | Copy selection to clipboard | Clipboard |
| Ctrl-X | Cut | Cut selection to clipboard | Clipboard |
| Ctrl-V | Paste | Paste from clipboard | Clipboard |
| Ctrl-Q | Quit | Quit (if clean) or save prompt | Quit |

## Contract Guarantees

1. **Completeness**: Every `Ctrl-<key>` binding in `src/editor/input.rs` that is active during editing MUST have an entry. Omission = bug.
2. **Immutability**: The list does not change based on session state, mode, or file type.
3. **One-to-one mapping**: Each command produces exactly one row; no duplicates.
4. **Ordering**: Categories appear in the order listed above unless a constitutional scope violation prevents it.

## Validation Criterion

Cross-reference `src/editor/input.rs` `map_key_event()` match arms for `KeyModifiers::CONTROL`: every arm MUST have a corresponding help entry, and vice versa.
