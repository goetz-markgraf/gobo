use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode, size,
};
use gobo::app::EditingSession;
use gobo::cli::Cli;
use gobo::editor::input::{EditorCommand, map_key_event};
use gobo::editor::render::{RenderView, TerminalSize};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use std::io::{self, stdout};
use std::process::ExitCode;
use std::time::Duration;

fn main() -> ExitCode {
    let cli = match gobo::cli::parse_args(std::env::args_os()) {
        Ok(cli) => cli,
        Err(error) => {
            let _ = error.print();
            return ExitCode::from(2);
        }
    };

    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("gobo: {error}");
            ExitCode::from(1)
        }
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let (width, height) = size().unwrap_or((80, 24));
    let mut session = EditingSession::open(cli.path, TerminalSize::new(width, height))?;

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, &mut session);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    session: &mut EditingSession,
) -> Result<(), Box<dyn std::error::Error>> {
    while !session.is_exiting() {
        draw(terminal, session)?;

        if event::poll(Duration::from_millis(250))? {
            match event::read()? {
                Event::Key(key) => {
                    if let Some(command) = map_key_event(key) {
                        session.handle_command(command)?;
                    }
                }
                Event::Resize(width, height) => {
                    session
                        .handle_command(EditorCommand::Resize(TerminalSize::new(width, height)))?;
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn draw(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    session: &EditingSession,
) -> io::Result<()> {
    let view = session.render_view();
    terminal.draw(|frame| paint(frame, &view)).map(|_| ())
}

/// Build the styled body text from `BodyLine`s, applying `Modifier::REVERSED`
/// to each `HighlightSpan` (spec 007, FR-010). Styling lives in this layer,
/// not in `render.rs` (constitution II boundary).
#[allow(mismatched_lifetime_syntaxes)]
fn format_body_lines(lines: &[gobo::editor::render::BodyLine]) -> ratatui::text::Text {
    let reversed = Style::default().add_modifier(Modifier::REVERSED);
    let normal = Style::default();
    let out: Vec<Line> = lines
        .iter()
        .map(|line| {
            if line.highlights.is_empty() {
                return Line::from(Span::styled(line.text.clone(), normal));
            }
            // Render the line as spans: split at each highlight's column boundaries.
            let spans = highlight_spans_to_spans(line, normal, reversed);
            Line::from(spans)
        })
        .collect();
    ratatui::text::Text::from(out)
}

/// Convert a `BodyLine`'s char string + visual-column highlight spans into a
/// list of owned `Span`s, applying `reversed` style to the highlighted visual
/// columns. Owning the span strings avoids tying their lifetime to the line.
fn highlight_spans_to_spans(
    line: &gobo::editor::render::BodyLine,
    normal: Style,
    reversed: Style,
) -> Vec<Span<'static>> {
    use unicode_segmentation::UnicodeSegmentation;
    use unicode_width::UnicodeWidthStr;

    // Collect each highlight as a (start, end) visual-column range.
    let ranges: Vec<(usize, usize)> = line
        .highlights
        .iter()
        .map(|h| (h.start_col, h.end_col))
        .collect();

    let graphemes: Vec<&str> = UnicodeSegmentation::graphemes(line.text.as_str(), true).collect();
    let mut spans = Vec::new();
    let mut col = 0usize;
    let mut buf = String::new();
    let mut buf_reversed = false; // style of the buffer currently being accumulated

    let flush = |spans: &mut Vec<Span>, buf: &mut String, use_reversed: bool| {
        if !buf.is_empty() {
            let style = if use_reversed { reversed } else { normal };
            spans.push(Span::styled(std::mem::take(buf), style));
        }
    };

    for g in graphemes {
        let gw = UnicodeWidthStr::width(g);
        let in_highlight = ranges
            .iter()
            .any(|(s, e)| col + gw > *s && col < *e);
        if in_highlight != buf_reversed {
            flush(&mut spans, &mut buf, buf_reversed);
            buf_reversed = in_highlight;
        }
        buf.push_str(g);
        col += gw;
    }
    flush(&mut spans, &mut buf, buf_reversed);
    spans
}

/// Pure widget layout + render for one frame. Separated from `draw` so the
/// assembled frame can be tested headlessly with ratatui's `TestBackend`.
fn paint(frame: &mut ratatui::Frame, view: &RenderView) {
    // Single footer row is always present; the search prompt adds one row only
    // while in SearchInput mode. There is no separate status line.
    let mut constraints = vec![Constraint::Min(1)]; // body
    if view.bottom_line.is_some() {
        constraints.push(Constraint::Length(1)); // search prompt
    }
    constraints.push(Constraint::Length(1)); // footer
    let footer_idx = constraints.len() - 1;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(frame.area());
    let body_idx = 0usize;
    let search_idx = 1usize; // valid only when bottom_line is Some

    let body =
        Paragraph::new(format_body_lines(&view.body_lines))
            .block(Block::default().borders(Borders::NONE));
    frame.render_widget(body, chunks[body_idx]);

    if let Some(prompt_line) = &view.bottom_line {
        let prompt = Paragraph::new(prompt_line.as_str()).style(
            Style::default()
                .fg(Color::Yellow)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );
        frame.render_widget(prompt, chunks[search_idx]);
    }

    // Footer carries filename (left) + status message (right) in one row, already
    // padded to the full width by `format_footer_line`, so no alignment/Border is
    // needed (a 1-line `Borders::TOP` block would otherwise hide the text).
    let footer = Paragraph::new(view.footer_line.clone())
        .style(Style::default().fg(Color::Black).bg(Color::White));
    frame.render_widget(footer, chunks[footer_idx]);

    if let Some(popup) = &view.popup {
        let rect = ratatui::layout::Rect::new(
            popup.rect.x,
            popup.rect.y,
            popup.rect.width,
            popup.rect.height,
        );
        frame.render_widget(Clear, rect);
        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Yellow).bg(Color::Black));
        let inner = block.inner(rect);
        frame.render_widget(block, rect);

        let mut lines = vec![popup.title.clone()];
        if let Some(message) = &popup.message {
            lines.push(String::new());
            lines.push(message.clone());
        }
        // HelpDialog: show keybinding rows instead of action buttons.
        if !popup.popup_rows.is_empty() {
            lines.push(String::new());
            for row in &popup.popup_rows {
                lines.push(row.clone());
            }
            lines.push(String::new());
        } else {
            lines.push(String::new());
            lines.push(
                popup
                    .actions
                    .iter()
                    .map(|action| action.label.as_str())
                    .collect::<Vec<_>>()
                    .join("    "),
            );
        }
        lines.push(popup.help_text.clone());

        let popup_paragraph = Paragraph::new(lines.join("\n")).style(
            Style::default()
                .fg(Color::White)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );
        frame.render_widget(popup_paragraph, inner);
    }

    frame.set_cursor_position((view.cursor_x, view.cursor_y));
}

#[cfg(test)]
mod tests {
    use super::paint;
    use gobo::app::{EditingSession, SessionMode};
    use gobo::editor::input::EditorCommand;
    use gobo::editor::render::{RenderView, TerminalSize};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    /// Render `view` once into a fresh TestBackend buffer and return the visible
    /// text, one string per row.
    fn render_rows(view: &RenderView, width: u16, height: u16) -> Vec<String> {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| paint(frame, view)).unwrap();
        let buffer = terminal.backend().buffer();
        let mut rows = Vec::new();
        for y in 0..buffer.area().height {
            let mut row = String::new();
            for x in 0..buffer.area().width {
                row.push_str(buffer[(x, y)].symbol());
            }
            rows.push(row);
        }
        rows
    }

    /// Spec 005 FR-001 + FR-003: the bottom row shows the filename on the left
    /// and the status message on the right, in one combined footer row.
    #[test]
    fn footer_filename_and_message_visible_in_one_row() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("visible.txt");
        std::fs::write(&path, "seed\n").unwrap();
        let session = EditingSession::open(&path, TerminalSize::new(160, 8)).unwrap();
        let view = session.render_view();
        let rows = render_rows(&view, 160, 8);
        let last = rows.last().unwrap();
        assert!(
            last.contains("visible.txt"),
            "filename missing from bottom row: {last:?}"
        );
        assert!(
            last.contains("Ready"),
            "status message missing from bottom row: {last:?}"
        );
        // There must be NO second status row above the footer.
        assert_eq!(rows.len(), 8, "expected exactly body(7) + footer(1)");
    }

    /// Spec 005 FR-005: in SearchInput mode the layout is body | search-prompt |
    /// footer, with the search prompt visible and the footer still present.
    #[test]
    fn search_prompt_and_footer_both_visible() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("search.txt");
        std::fs::write(&path, "hello world\n").unwrap();
        let mut session = EditingSession::open(&path, TerminalSize::new(40, 8)).unwrap();

        session.handle_command(EditorCommand::Search).unwrap();
        session
            .handle_command(EditorCommand::InsertChar('x'))
            .unwrap();
        assert_eq!(session.mode, SessionMode::SearchInput);

        let view = session.render_view();
        assert!(view.bottom_line.as_deref().unwrap().contains("Search: x"));

        let rows = render_rows(&view, 40, 8);
        let footer_row = rows.last().unwrap();
        let search_row = &rows[rows.len() - 2];
        assert!(
            footer_row.contains("search.txt"),
            "footer text missing, row: {footer_row:?}"
        );
        assert!(
            search_row.contains("Search: x"),
            "search prompt text missing, row: {search_row:?}"
        );
    }

    #[test]
    fn help_header_and_footer_always_visible_when_scrolling() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hf.txt");
        std::fs::write(&path, "seed\n").unwrap();

        // FR-004: on a small terminal header (title) + footer (Enter/Esc hint)
        // must stay visible at every scroll position; only the shortcut rows
        // scroll, and all 9 shortcuts must become reachable.
        let h = 10u16;
        let mut session =
            EditingSession::open(&path, TerminalSize::new(80, h)).unwrap();
        session.handle_command(EditorCommand::ShowHelp).unwrap();

        let mut seen_keys: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        for step in 0..12 {
            let view = session.render_view();
            let rows = render_rows(&view, 80, h);
            let joined = rows.join("\n");
            assert!(
                joined.contains("Keyboard Shortcuts"),
                "header missing at scroll step {step}: {joined:?}"
            );
            assert!(
                joined.contains("Enter/Esc"),
                "footer missing at scroll step {step}: {joined:?}"
            );
            for line in &rows {
                let t = line.trim();
                if let Some(key) = t
                    .strip_prefix("│")
                    .and_then(|s| s.trim_start().split_whitespace().next())
                    .filter(|k| k.starts_with("Ctrl-"))
                {
                    seen_keys.insert(key.to_string());
                }
            }
            session.handle_command(EditorCommand::MoveDown).unwrap();
        }
        assert_eq!(
            seen_keys.len(),
            9,
            "not all 9 shortcuts reachable by scrolling: {seen_keys:?}"
        );
    }
}
