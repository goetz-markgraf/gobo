# Data Model: Help Dialog

**Generated**: 2026-07-07 (updated)

## Entities

### HelpDialogRow

A single shortcut row displayed inside the help dialog.

| Field | Type | Description |
|-------|------|------------|
| `key` | `String` | The key binding as formatted text (e.g., `"Ctrl-S"`, `"Ctrl-Q"`) |
| `label` | `String` | Human-readable description of what the shortcut does |

**Validation rules**: `key` must match an active Ctrl-command mapping in `src/editor/input.rs`. No empty key or label.

### HelpDialog (aggregate)

| Field | Type | Description |
|-------|------|------------|
| `rows` | `Vec<HelpDialogRow>` | All shortcut rows in a flat list, displayed top-to-bottom |
| `title` | `&'static str` | Dialog title text — `"Keyboard Shortcuts"` |
| `scroll_offset` | `usize` | Viewport offset for vertical scrolling (0 when content fits) |

**Invariant**: `rows.len()` equals total active Ctrl-key bindings (currently 9). All rows are shown in the order listed in the contract.

## Flat Binding List (display order)

All entries are shown as a single column — key binding on the left, description label on the right. Consistent with the table style in `architecture.md`.

| # | Key | Label |
|---|-----|-------|
| 1 | Ctrl-F | Find in document |
| 2 | Ctrl-G | Find next match |
| 3 | Ctrl-S | Save document |
| 4 | Ctrl-Z | Undo last edit |
| 5 | Ctrl-Y | Redo last undone edit |
| 6 | Ctrl-C | Copy selection to clipboard |
| 7 | Ctrl-X | Cut selection to clipboard |
| 8 | Ctrl-V | Paste from clipboard |
| 9 | Ctrl-Q | Quit (clean) / save prompt (dirty) |

Total: **9** active Ctrl-key bindings.

## ScrollState

Tracks viewport position within the help dialog when content exceeds visible lines.

| Field | Type | Description |
|-------|------|------------|
| `offset` | `usize` | Index of the topmost visible row (0 = no scroll needed) |
| `visible_lines` | `usize` | How many rows fit in the dialog body after accounting for borders/title |

**Validation rules**: Content overhang triggers scroll; when lines ≥ total entries, `scroll_offset = 0`. One-line scroll per arrow key press.

## Rendered Layout (Full mode, ≥ 44×8 terminal)

```
┌──────────────────────────────┐
│         Keyboard             │
│         Shortcuts            │
├──────────────────────────────┤
│ Ctrl-F    Find in document   │
│ Ctrl-G    Find next match    │
│ Ctrl-S    Save document      │
│ Ctrl-Z    Undo last edit     │
│ Ctrl-Y    Redo last undone   │
│ Ctrl-C    Copy selection...  │
│ Ctrl-X    Cut selection...   │
│ Ctrl-V    Paste from clip.   │
│ Ctrl-Q    Quit / save prompt │
├──────────────────────────────┤
│  ↑/↓ Scroll · Enter/Esc Close│
└──────────────────────────────┘
```

## Compact Mode (width < 44 cols **or** height < 8 rows)

Title collapses to a single line ("Keyboard Shortcuts"). Footer line omitted. Rows may be truncated with ellipsis when dialog width is insufficient for even the shortest key binding.
