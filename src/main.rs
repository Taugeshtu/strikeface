mod auth;
mod clock;
pub mod pam;
pub mod scrambler;

use std::{
    io::{self, stdout},
    time::Duration,
};

use chrono::{Local, Timelike};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Terminal,
};

const COLOR_BG: Color = Color::Rgb(0xf9, 0xf9, 0xf9);
const COLOR_FG: Color = Color::Rgb(0x00, 0x00, 0x00);
const COLOR_MUTED: Color = Color::Rgb(0x55, 0x55, 0x55);
const COLOR_RED: Color = Color::Rgb(0xff, 0x00, 0x4f);

#[derive(PartialEq, Eq)]
enum FocusedField {
    Username,
    Password,
}

enum AuthState {
    Idle,
    Success,
}

struct App {
    username: String,
    password: String,
    focused: FocusedField,
    state: AuthState,
    time_offset_minutes: i64,
    session: Option<String>,
}

impl App {
    fn new(user: String, session: Option<String>) -> Self {
        Self {
            username: user,
            password: String::new(),
            focused: FocusedField::Password,
            state: AuthState::Idle,
            time_offset_minutes: 0,
            session,
        }
    }
}

fn main() -> io::Result<()> {
    let mut session = None;
    let mut user = std::env::var("USER").unwrap_or_default();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--session" => {
                session = args.next();
            }
            "--user" | "-u" => {
                if let Some(u) = args.next() {
                    user = u;
                }
            }
            _ => {}
        }
    }

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        crossterm::style::Print("\x1b]11;#f9f9f9\x1b\\")
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(user, session);
    let res = run_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        crossterm::style::Print("\x1b]111\x1b\\"),
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }

    if let AuthState::Success = app.state {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}

fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        if let AuthState::Success = app.state {
            break Ok(());
        }

        terminal
            .draw(|f| ui(f, app))
            .map_err(|e| io::Error::other(e.to_string()))?;

        // Poll events (instant on keypress, wakes up every 100ms for clock)
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {

                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    break Ok(());
                }

                // Global time manipulation keys (when not typing in username)
                if app.focused != FocusedField::Username {
                    match key.code {
                        KeyCode::Char('+') | KeyCode::Char('=') => {
                            app.time_offset_minutes += 1;
                            continue;
                        }
                        KeyCode::Char('-') | KeyCode::Char('_') => {
                            app.time_offset_minutes -= 1;
                            continue;
                        }
                        KeyCode::Char(']') => {
                            app.time_offset_minutes += 10;
                            continue;
                        }
                        KeyCode::Char('[') => {
                            app.time_offset_minutes -= 10;
                            continue;
                        }
                        KeyCode::Char('r') if app.password.is_empty() => {
                            app.time_offset_minutes = 0;
                            continue;
                        }
                        KeyCode::Right => {
                            app.time_offset_minutes += 1;
                            continue;
                        }
                        KeyCode::Left => {
                            app.time_offset_minutes -= 1;
                            continue;
                        }
                        _ => {}
                    }
                }

                match key.code {
                    KeyCode::Up => {
                        app.focused = FocusedField::Username;
                    }
                    KeyCode::Down => {
                        app.focused = FocusedField::Password;
                    }
                    KeyCode::Enter => match app.focused {
                        FocusedField::Username => {
                            app.focused = FocusedField::Password;
                        }
                        FocusedField::Password => {
                            if app.password.is_empty() {
                                continue;
                            }
                            let service = if app.session.is_some() {
                                "login"
                            } else {
                                "strikeface"
                            }
                            .to_string();
                            let user = app.username.clone();
                            let pass = app.password.clone();

                            // 1. Spawn PAM check on worker thread
                            let (tx, rx) = std::sync::mpsc::channel();
                            std::thread::spawn(move || {
                                let _ = tx.send(auth::verify(&service, &user, &pass));
                            });

                            // 2. Wait 150ms
                            std::thread::sleep(Duration::from_millis(150));

                            // 3. If PAM already succeeded (<150ms) -> exit clean!
                            if let Ok(Ok(())) = rx.try_recv() {
                                app.state = AuthState::Success;
                                break Ok(());
                            }

                            // 4. Still waiting (PAM fail sleep active) -> GO RED!
                            execute!(
                                io::stdout(),
                                crossterm::style::Print("\x1b]11;#ff004f\x1b\\")
                            )?;
                            terminal
                                .draw(|f| {
                                    let size = f.area();
                                    f.render_widget(
                                        Block::default().style(Style::default().bg(COLOR_RED)),
                                        size,
                                    );
                                })
                                .map_err(|e| io::Error::other(e.to_string()))?;

                            // 5. Block on rx.recv() while PAM finishes sleeping out its penalty
                            let _ = rx.recv();

                            // Restore canvas background to off-white
                            execute!(
                                io::stdout(),
                                crossterm::style::Print("\x1b]11;#f9f9f9\x1b\\")
                            )?;

                            // 6. Reset password and focus
                            app.password.clear();
                            app.focused = FocusedField::Password;
                        }
                    },
                    KeyCode::Backspace => match app.focused {
                        FocusedField::Username => {
                            app.username.pop();
                        }
                        FocusedField::Password => {
                            app.password.pop();
                        }
                    },
                    KeyCode::Char(c) => match app.focused {
                        FocusedField::Username => {
                            app.username.push(c);
                        }
                        FocusedField::Password => {
                            app.password.push(c);
                        }
                    },
                    KeyCode::Esc => {
                        if !app.password.is_empty() {
                            app.password.clear();
                        } else {
                            break Ok(());
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &App) {
    let size = f.area();

    // 1. Normal state: light parchment background
    let bg_block = Block::default().style(Style::default().bg(COLOR_BG));
    f.render_widget(bg_block, size);

    // Center vertical layout
    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Ratio(1, 4), // Top empty space
            Constraint::Length(1),   // Date (YYYY.MM.DD - Weekday)
            Constraint::Length(1),   // Gap
            Constraint::Length(7),   // Slanted Clock (7 rows!)
            Constraint::Length(2),   // Spacer
            Constraint::Length(1),   // Username
            Constraint::Length(1),   // Extra spacer between username and password
            Constraint::Length(1),   // Password slug + cursor
            Constraint::Min(0),      // Bottom space
        ])
        .split(size);

    let base_now = Local::now();
    let display_time = base_now + chrono::Duration::minutes(app.time_offset_minutes);

    let date_str = display_time.format("%Y.%m.%d - %A").to_string();
    let hours = display_time.hour();
    let minutes = display_time.minute();

    // Date (above clock)
    let date_p = Paragraph::new(date_str)
        .alignment(Alignment::Center)
        .style(Style::default().fg(COLOR_MUTED).bg(COLOR_BG));
    f.render_widget(date_p, v_chunks[1]);

    // Slanted Block Clock (7 lines tall)
    let clock_lines = clock::render_slanted_clock(hours, minutes);
    let clock_text: Vec<Line> = clock_lines
        .into_iter()
        .map(|line| {
            Line::from(vec![Span::styled(
                line,
                Style::default()
                    .fg(COLOR_FG)
                    .bg(COLOR_BG)
                    .add_modifier(Modifier::BOLD),
            )])
        })
        .collect();

    let clock_p = Paragraph::new(clock_text).alignment(Alignment::Center);
    f.render_widget(clock_p, v_chunks[3]);

    // Username line (> user < when focused)
    let user_display = if app.focused == FocusedField::Username {
        format!("> {} <", app.username)
    } else {
        app.username.clone()
    };
    let user_style = if app.focused == FocusedField::Username {
        Style::default()
            .fg(COLOR_RED)
            .bg(COLOR_BG)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(COLOR_FG).bg(COLOR_BG)
    };
    let user_p = Paragraph::new(user_display)
        .alignment(Alignment::Center)
        .style(user_style);
    f.render_widget(user_p, v_chunks[5]);

    // Password slug line + electric crimson blinking cursor
    let mut pass_spans = Vec::new();

    if !app.password.is_empty() {
        pass_spans.push(Span::styled(
            "█".repeat(app.password.len()),
            Style::default().fg(COLOR_FG).bg(COLOR_BG),
        ));
    }

    if app.focused == FocusedField::Password {
        pass_spans.push(Span::styled(
            "█",
            Style::default().fg(COLOR_RED).bg(COLOR_BG),
        ));
    }

    let pass_p = Paragraph::new(Line::from(pass_spans)).alignment(Alignment::Center);
    f.render_widget(pass_p, v_chunks[7]);
}
