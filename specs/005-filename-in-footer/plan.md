# Implementation Plan: Filename in Footer Row

**Branch**: `[005-filename-in-footer]` | **Date**: 2026-07-01 | **Spec**: [spec.md](./spec.md)

## Revision 2026-07-02 (design simplification — authoritative)

The original plan below added a footer row **in addition** to the existing status line (body | status-line | footer). During implementation this turned out to be problematic in practice: on wide terminals the right-aligned filename in the lower row was hard to perceive, and the mode/access information was never requested by the user.

The design is collapsed to a **single bottom row** that combines both concerns:

- **LEFT**: the filename (CLI path as-is) with an optional ` (*)` dirty marker.
- **RIGHT**: the current status message (e.g. `Ready`, `Match found`, `No match`, `Search cancelled`).
- There is **no separate status-line row** and **no mode/access display** any more.
- Layout is now: **body | footer** in normal mode, **body | search-prompt | footer** during `SearchInput`. The search prompt is the only additional row.

Consequences for the sections below (which otherwise describe the two-row design and are kept for history):

- `status::format_status_line()` is removed entirely; the message is produced from `session.status` and merged into the single footer string.
- `RenderView.status_line` is removed and `RenderView.footer_line` now carries `"{filename}  {message}"`-style content (filename left, message right-padded to terminal width). The `truncate_left` helper still applies to the filename part when the path is too long, but the message is dropped first if space is tight.
- `prompt_lines()` returns **2** in `SearchInput` mode (search prompt + footer) and **1** otherwise (footer). Earlier `+1` values were based on the now-removed status line and are reduced.
- `main.rs::draw` renders only `body`, the optional search prompt, and the footer. The status chunk is gone.
- Tests asserting on `render_view().status_line` are updated to assert on `footer_line` instead.

## Summary

Add a dedicated footer row at the bottom of the screen displaying the current filename (as given on the CLI), right-aligned, with an optional `(*)` marker when the file is dirty. Eliminate stray empty/hyphen filler rows so the visible layout renders exactly three areas: **body | status-line | footer** (expanding to four during `SearchInput` mode). The implementation touches five files: `render.rs` (new `footer_line` field + truncation helper), `status.rs` (remove path from status line → move dirtiness indicator to footer), `main.rs` (layout constraint expansion for the footer row), and two integration test files that reference `RenderView.status_line` contents.

## Technical Context

**Language/Version**: Rust 2024 edition (Rust 1.85+)

**Primary Dependencies**:
- `ratatui` 0.29 — layout constraints, `Paragraph` + `Alignment::Right` for footer rendering
- `crossterm` 0.28 — terminal size events (unchanged)
- `unicode-width` 0.2 — grapheme width for left-truncation when path exceeds terminal width

**Storage**: None (pure in-memory projection from existing `document.path`)

**Testing**: Integration tests in `tests/integration/` driven by `cargo test`. Two files need minor assertion updates (`unsaved_guards.rs`, `search_and_resize.rs`).

**Target Platform**: macOS / Linux / BSD terminal emulators

**Performance Goals**: Footer string formatting is O(path-length), typically < 200 chars; negligible impact on render loop (< 1ms). Left-truncation with `...` prefix runs in linear grapheme-scan.

**Constraints**: Touch only `render.rs`, `status.rs`, `main.rs`, `app.rs`, and two integration test files. No new dependencies. No changes to `cli.rs`, `document.rs`, or `editor/input.rs` (per spec assumption). The footer row is always visible — no mode hides it.

## Constitution Check

- **Readability Gate**: The change adds one new field (`footer_line: String`) to the existing `RenderView` struct and one helper function (`format_footer_line`). Rendering logic in `main.rs::draw()` gains one additional layout chunk + one `Paragraph` widget. All additions are localized and follow existing patterns.
- **Maintainability Gate**: Clear boundary — `render.rs` owns projection of session data into `RenderView`; `status.rs` owns status-line formatting; `main.rs` owns the draw/layout layer. The footer line is derived from `document.path` + `document.dirty`, both already present on the session. No cross-cutting concerns introduced.
- **Security Gate**: No file I/O or security-sensitive changes. Footer merely displays the path string passed at open time.
- **Verification Gate**: All existing integration tests updated to match new `RenderView` shape and content. New assertions verify footer presence, dirty marker appearance/disappearance, and correct layout heights under SearchInput mode. Full compile + test gate via `cargo test`.
- **Scope Gate**: Purely cosmetic UI addition within the single-file editor scope. No constitutional exception required.

## Project Structure

### Source Code (repository root)

```text
src/
├── main.rs                # draw(): add footer chunk + Paragraph widget; layout constraint update
├── app.rs                 # prompt_lines(): +1 for footer row in viewport calc
├── lib.rs                 # No changes
├── cli.rs                 # No changes (per spec)
├── document.rs            # No changes (per spec)
└── editor/
     ├── mod.rs             # No changes
     └── render.rs          # Add footer_line field to RenderView + format_footer_line() helper
     └── status.rs          # Remove path/dirty from status_line; add footer dirty marker logic

tests/integration/
├── unsaved_guards.rs      # Update assertion: .status_line no longer contains filename
└── search_and_resize.rs   # Layout height checks still valid (prompt_lines shifted by +1)
```

## Detailed Design

### 1. Viewport Budget Adjustment (`app.rs`)

**Current**: `prompt_lines()` returns 2 during `SearchInput` mode, 1 otherwise. The value is consumed by `ViewportState::update_for_terminal(size, prompt_lines)` as the number of non-body lines to subtract from total height.

**Change**: Bump return values by +1 to account for the always-visible footer row:

```rust
pub fn prompt_lines(&self) -> u16 {
    // 1 (status) + N (search prompt if active) + 1 (footer)
    match self.mode == SessionMode::SearchInput {
        true => 3,   // status + search-prompt + footer
        false => 2,  // status + footer
    }
}
```

This ensures `viewport.visible_height` correctly reduces body area by the extra row.

### 2. New Footer Field on `RenderView` (`editor/render.rs`)

**Change**: Add `footer_line: String` to `RenderView` and populate it via a new helper function.

```rust
pub struct RenderView {
    pub body_lines: Vec<String>,
    pub status_line: String,
    pub footer_line: String,        // NEW: always-present filename footer
    pub bottom_line: Option<String>, // unchanged: search prompt only in SearchInput
    pub popup: Option<PopupView>,
    pub cursor_x: u16,
    pub cursor_y: u16,
}
```

New helper:
```rust
/// Format the footer line: right-aligned filename with optional (*) dirty marker.
/// If the path exceeds terminal width, truncate from the left with "..." prefix.
pub fn format_footer_line(path: &str, dirty: bool, terminal_width: u16) -> String {
    let label = if dirty {
        format!("{} (*)", path)
    } else {
        path.to_string()
    };

    let label_width = unicode_width::UnicodeWidthStr::width(&label);
    let text = if label_width as u16 > terminal_width {
        // Truncate from left with "..." prefix, keep dirty marker intact
        let available = terminal_width.saturating_sub(3).saturating_sub(4); // "..." + " (*)"
        let trimmed_label = if dirty {
            // Strip " (*)" suffix for truncation math, re-add after
            let path_part = label.trim_end_matches(" (*)");
            truncate_left(path_part, available as usize) + " (*)"
        } else {
            truncate_left(&label, available as usize)
        };
        trimmed_label
    } else {
        label
    };

    // Right-align: pad left with spaces
    let text_width = unicode_width::UnicodeWidthStr::width(&text);
    let padding = terminal_width.saturating_sub(text_width as u16) as usize;
    format!("{}{}", " ".repeat(padding), text)
}

fn truncate_left(s: &str, max_grapheme_width: usize) -> String {
    let mut result = String::new();
    let mut current_width = 0;
    for grapheme in s.graphemes(true) {
        let gw = unicode_width::UnicodeWidthStr::width(grapheme);
        if current_width + gw > max_grapheme_width {
            break;
        }
        result.push_str(grapheme);
        current_width += gw;
    }
    format!("...{}", result)
}
```

In `render_view()`, add after existing fields:
```rust
let footer_line = format_footer_line(
    session.document.path.display().to_string().as_str(),
    session.document.dirty,
    session.terminal_size.width,
);
```

Include `footer_line` in the returned `RenderView`.

### 3. Status Line Cleanup (`editor/status.rs`)

**Change**: Remove path and dirty state from the status line (they move to the footer). Keep access mode, editing mode, and current status message.

**Before**:
```rust
// "path | EDIT | DIRTY | editing | Ready"
format!("{} | {} | {} | {} | {}", path, access, dirty, mode, message)
```

**After**:
```rust
// "EDIT | editing | Ready"  (access | mode | message — no path, no dirty)
format!("{} | {} | {}", access, mode, message)
```

This eliminates duplication between status line and footer. Access mode remains relevant (read-only is a file property shown alongside the filename context). Dirtiness moves exclusively to the footer `(*)`.

### 4. Draw Layout Expansion (`main.rs`)

**Current layout constraints**:
```rust
let prompt_height = if view.bottom_line.is_some() { 2 } else { 0 };
let chunks = Layout::default().direction(Direction::Vertical)
    .constraints([Min(1), Length(1), Length(prompt_height)])
    .split(frame.area());
// chunks[0] = body, [1] = status line, [2] = bottom_line (optional search prompt)
```

**After**:
```rust
let mut constraints: Vec<Constraint> = vec![
    Constraint::Min(1),   // body
    Constraint::Length(1), // status line
];
if view.bottom_line.is_some() {
    constraints.push(Constraint::Length(1)); // search prompt
}
constraints.push(Constraint::Length(1)); // footer (always)

let chunks = Layout::default().direction(Direction::Vertical).constraints(constraints).split(frame.area());

// Render sequence: body (0), status (1), [search prompt (2)], footer (last index)
let footer_index = chunks.len() - 1;
// ... render body at chunks[0], status at chunks[1], search-prompt at chunks[2] if present
// Footer is the last chunk — always rendered:
let footer = Paragraph::new(view.footer_line.clone())
    .style(Style::default().fg(Color::White).bg(Color::Black))
    .alignment(Alignment::Right)
    .block(Block::default().borders(Borders::TOP));
frame.render_widget(footer, chunks[footer_index]);
```

The footer renders with a `Borders::TOP` separator (matching the status line's existing style) and uses ratatui's `Alignment::Right` for right-alignment fallback. The helper `format_footer_line()` already produces right-padded text, so both approaches compose correctly.

**Elimation of stray rows**: The old code could produce empty filler areas when `prompt_height` mismatched actual constraints. With the new dynamic constraint vector, exactly the needed lines are requested — no leftover hyphens or empty rows.

### 5. Integration Test Updates

Two integration test files reference `RenderView.status_line` and `bottom_line`:

**`tests/integration/unsaved_guards.rs`**:
- Line ~110: `assert!(view.status_line.contains("example.txt"));` → Change to check `footer_line` instead, since path no longer appears on status line. Update to: `assert!(view.footer_line.contains("example.txt"));`
- Two assertions on `bottom_line == None` remain valid (popup still takes precedence over search prompt).

**`tests/integration/search_and_resize.rs`**:
- Line ~61, ~232: `bottom_line.contains("Search:")` — remains valid, no change needed.
- Layout height / resize assertions may need update since `prompt_lines()` values shifted by +1. Verify viewport math post-merge.

## Migration Checklist

| Step | File | Change |
|------|------|--------|
| 1 | `src/app.rs` | Update `prompt_lines()` return values (+1 for footer row) |
| 2 | `src/editor/render.rs` | Add `footer_line: String` to `RenderView`; add `format_footer_line()` + `truncate_left()` helpers; populate in `render_view()` |
| 3 | `src/editor/status.rs` | Remove path and dirty from `format_status_line()` output |
| 4 | `src/main.rs` | Expand layout constraints dynamically; render footer as separate chunk with Borders::TOP and right-aligned style |
| 5 | `tests/integration/unsaved_guards.rs` | Update path assertion to check `footer_line` instead of `status_line` |
| 6 | `tests/integration/search_and_resize.rs` | Verify resize assertions against new viewport heights |
| 7 | Compile & test | `cargo build && cargo test` — all existing tests pass, no regressions |

## Extension Hooks

The `after_plan` hook is registered in `.specify/extensions.yml`:

- **agent-context** (optional): `speckit.agent-context.update` — refreshes the AGENTS.md file to point to this plan. Run after all planning artifacts are generated.
