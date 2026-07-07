# Research: Help Dialog

**Generated**: 2026-07-07

## Clarifications Resolved

All technical unknowns were already resolved by the feature spec:

| Unknown | Resolution | Source |
|---------|------------|--------|
| Popup mechanism to reuse | `PopupView` + `pending_prompt` from ConfirmQuit/SaveConflictPrompt pattern | architecture.md § Popup Precedence |
| Scrolling style | Line-by-line arrow keys only | Spec § Clarifications |
| Shortcuts scope | Ctrl-bindings active during text editing only | Spec § Clarifications |
| Internal search/filter | Not required (~10 entries; scroll sufficient) | Spec § Clarifications |

## Technology Decisions

### Decision: Reuse `PopupView` / `pending_prompt`
**Rationale**: The spec requires exactly this mechanism (FR-007). ConfirmQuit and SaveConflictPrompt already demonstrate the pattern. Adding a new widget type would violate Scope and Maintainability gates.
**Alternatives considered**: Custom modal struct, dedicated HelpView module — both rejected as unnecessary complexity.

### Decision: Scroll using ratatui `ScrollState` (if available) or custom index tracking
**Rationale**: ratatui 0.29 may provide a `ListState` with scroll support; if not, cursor-offset on the entry list is simple enough to implement manually. No external crate needed.

### Decision: Static data construction in `editor/status.rs` (new helper)
**Rationale**: Help entries never change at runtime. Defining them as a static or once-initialized structure in an existing editor module avoids new module boundaries.
