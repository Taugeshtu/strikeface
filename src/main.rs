mod clock;

use std::{
    io::{self, stdout},
    time::{Duration, Instant},
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
    ErrorFlash(Instant),
    Success,
}

struct App {
    username: String,
    password: String,
    focused: FocusedField,
    state: AuthState,
    time_offset_minutes: i64,
}

impl App {
    fn new() -> Self {
        Self {
            username: "tau".to_string(),
            password: String::new(),
            focused: FocusedField::Password,
            state: AuthState::Idle,
            time_offset_minutes: 0,
        }
    }
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = run_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {err}");
    }
    Ok(())
}

fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        // Handle error flash timeout
        if let AuthState::ErrorFlash(start) = app.state {
            if start.elapsed() >= Duration::from_secs(2) {
                app.state = AuthState::Idle;
                app.password.clear();
                app.focused = FocusedField::Password;
            }
        }

        if let AuthState::Success = app.state {
            break Ok(());
        }

        terminal
            .draw(|f| ui(f, app))
            .map_err(|e| io::Error::other(e.to_string()))?;

        // Poll events
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                // Discard inputs during full-screen error lockout
                if let AuthState::ErrorFlash(_) = app.state {
                    continue;
                }

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
                            // Mock auth: password "tau" succeeds, everything else flashes red
                            if app.password == "tau" {
                                app.state = AuthState::Success;
                            } else {
                                app.state = AuthState::ErrorFlash(Instant::now());
                            }
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

    // 1. Error state: full screen red
    if let AuthState::ErrorFlash(_) = app.state {
        let red_block = Block::default().style(Style::default().bg(COLOR_RED));
        f.render_widget(red_block, size);
        return;
    }

    // 2. Normal state: light parchment background
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
