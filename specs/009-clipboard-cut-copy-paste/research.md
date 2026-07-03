# Research: Clipboard Cut, Copy & Paste

## Decisions

### Decision: Use `arboard v3` for system clipboard access
**Rationale**: Cross-platform (macOS, Linux/X11/Wayland, Windows), well-maintained (latest 3.6.1),
provides a simple `DisplayClipboard::get_text()` / `set_text()` API. No manual FFI or per-platform
conditional compilation needed. Satisfies Constitution II and V.
**Alternatives considered**: 
- Direct platform-specific FFI (`NSPasteboard` on macOS, `xdg-clipboard`/Wayland protocols): would add 3x code paths and maintenance burden.
- Spawn `pbcopy`/`xclip` subprocesses: fragile, adds process overhead (~50-100ms per call), and breaks in sandboxed environments (e.g., noo sandbox).

### Decision: Clipboard data stored only in local variables during command dispatch
**Rationale**: FR-008 requires the editor to never manage its own clipboard copies. Cut/Copy commands
write to OS clipboard in a single `set_text()` call and discard the local variable immediately.
Paste reads fresh from OS clipboard on each invocation via `get_text()`. No caching or stale-data risk.

### Decision: Single grapheme cluster for cut/copy without selection
**Rationale**: User requested "das Zeichen unter dem Cursor" — the first complete Unicode grapheme cluster
after the cursor position. Uses `unicode-segmentation` (already a dependency) to get `GraphemeClusterIter`.
Consistent with ropey's char index model but uses grapheme semantics for user-facing behavior.

### Decision: 1 MB hard limit enforced at both read and write boundaries
**Rationale**: Spec clarification confirmed 1 MB as the upper bound. Enforced on paste (rejected if >1 MB)
and on copy (clipped to 1 MB with warning). Prevents memory pressure from malicious clipboard content.

### Decision: Paste of empty or textless clipboard is a no-op with no status change
**Rationale**: FR-009 specifies silent no-op for empty/non-text clipboard. No error message displayed since
it's an expected operational state (clipboard may legitimately be empty or contain only images).

## Requirements Resolved

| NEEDS CLARIFICATION | Resolution |
|---|---|
| Ctrl-C without selection behavior | Copies the first grapheme cluster after cursor (symmetric with Cut) |
| Maximum clipboard size | 1 MB hard limit, user informed via status message on overflow |
| Undo semantics for Cut | Single `EditStep::Delete` restores entire removed text; OS clipboard persists unchanged |
| Paste over selection | Atomic `EditStep::Replace` — selected text removed, clipboard content inserted, cursor repositioned |

## Technology Notes

### `arboard` v3 API surface used
- `arboard::DisplayClipboard::new()` → creates handle per-platform
- `clipboard.set_text(text)` → writes UTF-8 string to OS clipboard
- `clipboard.get_text()` → returns `Option<String>`; `None` if clipboard lacks text content
- Single lifecycle: create once in main, use per command — no persistent connection needed
