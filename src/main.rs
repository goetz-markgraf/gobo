use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size, EnterAlternateScreen, LeaveAlternateScreen};
use gobo::app::EditingSession;
use gobo::cli::Cli;
use gobo::editor::input::{map_key_event, EditorCommand};
use gobo::editor::render::TerminalSize;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;
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
                    session.handle_command(EditorCommand::Resize(TerminalSize::new(width, height)))?;
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
    terminal
        .draw(|frame| {
            let prompt_height = if view.prompt_line.is_some() { 1 } else { 0 };
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),
                    Constraint::Length(1),
                    Constraint::Length(prompt_height),
                ])
                .split(frame.area());

            let body = Paragraph::new(view.body_lines.join("\n")).block(Block::default().borders(Borders::NONE));
            frame.render_widget(body, chunks[0]);

            let status = Paragraph::new(view.status_line)
                .style(Style::default().fg(Color::Black).bg(Color::White))
                .block(Block::default().borders(Borders::TOP));
            frame.render_widget(status, chunks[1]);

            if let Some(prompt_line) = view.prompt_line {
                let prompt = Paragraph::new(prompt_line)
                    .style(Style::default().fg(Color::Yellow))
                    .block(Block::default().borders(Borders::TOP));
                frame.render_widget(prompt, chunks[2]);
            }

            frame.set_cursor_position((view.cursor_x, view.cursor_y));
        })
        .map(|_| ())
}
