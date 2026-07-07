# Data Model: Help Dialog

**Generated**: 2026-07-07

## Entities

### HelpDialogEntry

A single shortcut row displayed inside the help dialog.

| Field | Type | Description |
|-------|------|-------------|
| `key_description` | `String` | The key binding as formatted text (e.g., `"Ctrl-S"`, `"Shift+↑"`) |
| `label` | `String` | Human-readable description of what the shortcut does |

**Validation rules**: `key_description` must match an active Ctrl-command mapping in `src/editor/input.rs`. No empty key or label.

### HelpCategoryChunk

A section header followed by its associated entries.

| Field | Type | Description |
|-------|------|-------------|
| `header` | `&'static str` | Section title (e.g., "Navigation", "Editing") |
| `entries` | `Vec<HelpDialogEntry>` | Shortcut rows belonging to this category |

**Invariant**: Every entry in the help dialog is grouped into exactly one category. Categories appear in a defined display order. No empty categories.

### ScrollState

Tracks viewport position within the help dialog (in-memory, session-bound).

| Field | Type | Description |
|-------|------|-------------|
| `offset` | `usize` | Index of the topmost visible entry |
| `max_offset` | `usize` | Maximum valid offset (total_entries - visible_lines) |
| `visible_lines` | `usize` | How many lines fit in the dialog body after accounting for borders/title |

**Validation rules**: `offset` clamped to `[0, max_offset]`. `visible_lines` ≥ 1. Scrolling by one line per arrow key press advances or decrements `offset` within bounds.

### HelpDialog (aggregate)

| Field | Type | Description |
|-------|------|-------------|
| `category_chunks` | `Vec<HelpCategoryChunk>` | All categories + entries in display order |
| `scroll` | `ScrollState` | Current viewport position |
| `title` | `&'static str` | Dialog title text (e.g., "Keyboard Shortcuts") |

**Invariant**: `category_chunks.len()` equals total visible sections; sum of all entry lists equals total shortcuts listed; scroll respects visible_lines computed from terminal dimensions at render time.

## Category Breakdown (all active bindings)

| Category | Bindings |
|----------|----------|
| **Navigation** | Ctrl-F, Ctrl-G |
| **Editing** | Tab, Shift+Tab, Enter, Backspace, Delete |
| **Selection** | Shift+←, Shift+→, Shift+↑, Shift+↓ |
| **Document** | Ctrl-S |
| **Undo / Redo** | Ctrl-Z, Ctrl-Y |
| **Clipboard** | Ctrl-C, Ctrl-X, Ctrl-V |
| **Quit** | Ctrl-Q |

Total: 17 bindable commands across 7 categories.

## Rendered Layout (Full mode)

```
┌──────────────────────────────┐
│         Keyboard             │
│         Shortcuts            │
├──────────────────────────────┤
│ Navigation                   │
│   Ctrl-F      Find           │
│   Ctrl-G      Find Next      │
│ Editing                      │
│   Tab         Insert Tab     │
│   Shift+Tab   Tab Back       │
│   Enter       Newline + Indent│
│   Backspace   Delete Back    │
│   Delete      Delete Fwd     │
│ Selection                    │
│   Shift+↑/↓/←/→  Extend      │
│ Document                     │
│   Ctrl-S      Save           │
│ Undo / Redo                  │
│   Ctrl-Z      Undo           │
│   Ctrl-Y      Redo           │
│ Clipboard                    │
│   Ctrl-C      Copy           │
│   Ctrl-X      Cut            │
│   Ctrl-V      Paste          │
│ Quit                         │
│   Ctrl-Q      Quit           │
├──────────────────────────────┤
│  ↑/↓ Scroll · Enter/Esc Close│
└──────────────────────────────┘
```

## Compact Mode (< 44×8 terminal)

Title collapses to a single line; no footer shown. Same categories/entries, just more space for content.
