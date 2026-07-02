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

**Testing**: Unit tests in `tests/unit/render.rs` for `format_footer_line` (clean/dirty, relative & absolute paths, truncation, narrow terminals, message dropped, empty path, non-`Ready` messages). Integration tests in `tests/integration/{open_and_save,unsaved_guards,search_and_resize}.rs` assert the footer via `render_view()` and via the headless `paint` tests in `main.rs`.

**Target Platform**: macOS / Linux / BSD terminal emulators

**Performance Goals**: Footer string formatting is O(path-length), typically < 200 chars; negligible impact on render loop (< 1ms). Left-truncation with `...` prefix runs in linear grapheme-scan.

**Constraints**: Touch only `render.rs`, `status.rs`, `main.rs`, `app.rs`, and two integration test files. No new dependencies. No changes to `cli.rs`, `document.rs`, or `editor/input.rs` (per spec assumption). The footer row is always visible — no mode hides it.

## Constitution Check

- **Readability Gate**: The change replaces the prior two-row footer/status design with a single `footer_line: String` field on `RenderView` and one helper `format_footer_line` (plus `truncate_left`). `format_status_line` is removed entirely — there is no status-line row any more. `main.rs::paint` renders only body, the optional search prompt, and the footer. A reviewer reading `render.rs` / `main.rs` in isolation sees the single-row design with no leftover two-row references.
- **Maintainability Gate**: Clear boundary — `render.rs` projects session state into `RenderView`; `status.rs` owns the status *message* (`current_message`); `main.rs` owns the draw/layout layer. The footer is derived solely from `document.path` + `document.dirty` + the status message, all already present. No cross-cutting concerns introduced.
- **Security Gate**: No file I/O or security-sensitive changes. Footer merely displays the path string passed at open time.
- **Verification Gate**: Existing integration tests updated from `status_line` to `footer_line`; new unit tests cover `format_footer_line` edge cases (truncation, narrow terminals, message-drop, empty path, absolute paths, non-`Ready` messages); `main.rs` headless `paint` tests assert the bottom row carries filename left + message right in both normal and SearchInput modes. Full compile + test gate via `cargo test`.
- **Scope Gate**: Purely cosmetic UI change within the single-file editor scope. Simpler than the prior two-row design. No constitutional exception required.

## Project Structure

### Source Code (repository root)

```text
src/
├── main.rs                # paint(): dynamic constraints — body | [search-prompt] | footer; footer styled reversed-colors (Black on White)
├── app.rs                 # prompt_lines(): 1 (Editing) / 2 (SearchInput) — footer always counts
├── lib.rs                 # No changes
├── cli.rs                 # No changes (per spec)
├── document.rs            # No changes (per spec); document.path stays non-canonicalized
└── editor/
     ├── mod.rs             # No changes
     ├── render.rs          # RenderView { body_lines, footer_line, bottom_line, popup, cursor_x/y } (status_line removed); format_footer_line() + truncate_left() helpers
     └── status.rs          # status_line removed; current_message() returns the status text or "Ready"

tests/
├── unit/render.rs        # format_footer_line edge cases (incl. absolute path + non-Ready message)
└── integration/
     ├── open_and_save.rs    # footer dirty marker appears/disappears; relative path verbatim
     ├── unsaved_guards.rs   # footer dirty marker + left-truncation while prompted
     └── search_and_resize.rs# footer carries non-"Ready" message after search; SearchInput layout body|prompt|footer
```

## Detailed Design

The sections below describe the **implemented single-row design** (per the 2026-07-02
revision). The original two-row design is no longer pursued.

### 1. Viewport Budget Adjustment (`app.rs`)

`prompt_lines()` returns the number of non-body lines subtracted from total height by
`ViewportState::update_for_terminal`. The footer counts as one always-present row; the
search prompt adds one more row only while in `SearchInput`. There is no status-line row.

```rust
pub fn prompt_lines(&self) -> u16 {
    match self.mode == SessionMode::SearchInput {
        true => 2,  // search-prompt + footer
        false => 1, // footer only
    }
}
```

### 2. Footer field on `RenderView` (`editor/render.rs`)

`RenderView` carries `footer_line: String` instead of the former `status_line`. It is
produced by `format_footer_line`, which lays out the filename (+ optional ` (*)`) on the
LEFT and the status message on the RIGHT, padded to `terminal_width`:

```rust
pub struct RenderView {
    pub body_lines: Vec<String>,
    pub footer_line: String,         // filename (left) + status message (right), padded to width
    pub bottom_line: Option<String>, // search prompt, only in SearchInput
    pub popup: Option<PopupView>,
    pub cursor_x: u16,
    pub cursor_y: u16,
}

/// One footer row: filename (+ optional ` (*)`) on the LEFT, status `message` on the
/// RIGHT, padded to `terminal_width`. Long names truncate from the left with `...`;
/// the message is dropped first when space is tight. Spec 005 FR-001 / FR-003.
pub fn format_footer_line(path: &str, dirty: bool, message: &str, terminal_width: u16) -> String {
    let name = if dirty { format!("{} (*)", path) } else { path.to_string() };
    let width = terminal_width as usize;
    let name_width = UnicodeWidthStr::width(name.as_str());
    if name_width >= width {
        return truncate_name(&name, terminal_width, dirty);
    }
    let msg_width = UnicodeWidthStr::width(message);
    let gap_width = width - name_width;
    if msg_width < gap_width {
        let pad = gap_width - msg_width;
        return format!("{}{}{}", name, " ".repeat(pad), message);
    }
    // Message won't fit beside the name: keep the name, pad the rest with spaces.
    if name_width <= width {
        return format!("{}{}", name, " ".repeat(gap_width));
    }
    truncate_name(&name, terminal_width, dirty)
}
```

`truncate_name` peels off the ` (*)` suffix when present so the dirty marker survives
truncation; `truncate_left` walks graphemes from the right and prepends `...`.

In `render_view()`, the footer is sourced from the already-stored, non-canonicalized CLI
path plus the current status message:

```rust
let message = status::current_message(session);
let footer_line = format_footer_line(
    session.document.path.display().to_string().as_str(),
    session.document.dirty,
    &message,
    session.terminal_size.width,
);
```

### 3. Status message helper (`editor/status.rs`)

`format_status_line` is removed. Its only remaining responsibility is the status message
itself, surfaced via `current_message`, which returns `session.status` text or `"Ready"`:

```rust
pub fn current_message(session: &EditingSession) -> String {
    session.status.as_ref().map(|s| s.text.clone()).unwrap_or_else(|| "Ready".to_string())
}
```

Access mode and editing mode are **not** shown anywhere (per spec), and dirtiness is
expressed exclusively by the footer `(*)`. There is no separate status row.

### 4. Layout (`main.rs::paint`)

`paint` builds a dynamic constraint vector — body first, then the search prompt only when
`bottom_line` is `Some`, then the always-present footer as the final chunk:

```rust
let mut constraints = vec![Constraint::Min(1)];            // body
if view.bottom_line.is_some() {
    constraints.push(Constraint::Length(1));               // search prompt (SearchInput only)
}
constraints.push(Constraint::Length(1));                    // footer (always)
let footer_idx = constraints.len() - 1;
```

The footer is rendered as a single-line `Paragraph` with reversed colors,
`Style::default().fg(Color::Black).bg(Color::White)`. A `Borders::TOP` block is **not**
used because a 1-line bordered block leaves no room for the text; the reversed-color row
provides the visual separation instead. Because `format_footer_line` already pads the
string to the full width, no ratatui `Alignment` is needed.

This dynamic vector requests exactly the rows that render — no `Min(0)`/leftover chunks —
so stray empty / `-` filler rows cannot appear (SC-003).

### 5. Test coverage

- `tests/unit/render.rs` — `format_footer_line`: clean/dirty left marker, relative and
  absolute paths verbatim, left-truncation with `...` (dirty marker preserved), width
  never exceeds terminal, narrow terminal width-7 fits exactly, overlong message is
  dropped leaving the padded name, empty path renders message right, non-`Ready` message
  surfaces on the right.
- `tests/integration/open_and_save.rs` — dirty marker appears on edit and disappears after
  save; relative path shown verbatim.
- `tests/integration/unsaved_guards.rs` — footer dirty marker and left-truncation still
  render while a quit popup is pending.
- `tests/integration/search_and_resize.rs` — after a search producing a non-`Ready`
  message (e.g. "No match for …"), the footer carries that message on the right alongside
  the filename on the left.
- `main.rs` (inline `#[cfg(test)]`) — headless `paint` via `TestBackend` asserts the
  bottom row shows filename left + message right in both normal and SearchInput modes,
  and that no extra status row is emitted.

## Migration Checklist (executed)

| Step | File | Change | Done |
|------|------|--------|------|
| 1 | `src/app.rs` | `prompt_lines()` → 1 (Editing) / 2 (SearchInput); footer always counts, no status row | ✅ |
| 2 | `src/editor/render.rs` | `RenderView.status_line` → `footer_line`; `format_footer_line()` + `truncate_left()` populate it in `render_view()` | ✅ |
| 3 | `src/editor/status.rs` | `format_status_line` removed; `current_message()` returns status text or `"Ready"` | ✅ |
| 4 | `src/main.rs` | `paint()` uses a dynamic constraint vector — body \| [search-prompt] \| footer; footer styled reversed-colors (no `Borders::TOP`) | ✅ |
| 5 | `tests/integration/open_and_save.rs` | Dirty marker + relative-path footer assertions | ✅ |
| 6 | `tests/integration/unsaved_guards.rs` | Footer dirty marker / truncation while prompted | ✅ |
| 7 | `tests/integration/search_and_resize.rs` | Footer carries non-`Ready` message after search; SearchInput layout | ✅ |
| 8 | `tests/unit/render.rs` | `format_footer_line` edge cases incl. absolute path + non-`Ready` message | ✅ |
| 9 | `src/main.rs` (inline) | Headless `paint` tests assert bottom row in both modes | ✅ |
| 10 | Compile & test | `cargo build && cargo test` — green, no regressions | ✅ |

## Extension Hooks

The `after_plan` hook is registered in `.specify/extensions.yml`:

- **agent-context** (optional): `speckit.agent-context.update` — refreshes the AGENTS.md file to point to this plan. Run after all planning artifacts are generated.
