use crate::ui::app::{App, Mode};
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;
use std::sync::Arc;
use std::time::Duration;

pub fn run_ui(db: Arc<crate::db::Db>) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(db);
    app.refresh_keys();

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(f.size());

            // Header
            let _uptime = std::time::Instant::now();
            let header_text = format!(
                "🦀 Mini Redis Dashboard | Keys: {} | Uptime: 0s | Tab: {}",
                app.keys.len(),
                app.tab + 1
            );
            let header = Paragraph::new(header_text)
                .style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(header, chunks[0]);

            // Tabs
            let tabs = Paragraph::new("1: Keys | 2: History | 3: Help")
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title("Tabs"));
            f.render_widget(tabs, chunks[1]);

            // Content based on selected tab
            match app.tab {
                0 => {
                    // Keys tab
                    let items: Vec<ListItem> = app
                        .keys
                        .iter()
                        .enumerate()
                        .map(|(i, key)| {
                            let value = app.get_value(key).unwrap_or_else(|| "(nil)".to_string());
                            let ttl = app.get_ttl(key);
                            let display = if let Some(ms) = ttl {
                                format!("{} → {} [TTL: {}ms]", key, value, ms)
                            } else {
                                format!("{} → {}", key, value)
                            };

                            let style = if i == app.selected {
                                Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::White)
                            };

                            ListItem::new(Span::styled(display, style))
                        })
                        .collect();

                    let list = List::new(items).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Key-Value Store"),
                    );
                    f.render_widget(list, chunks[2]);
                }
                1 => {
                    // History tab
                    let items: Vec<ListItem> = app
                        .history
                        .iter()
                        .map(|line| {
                            let style = if line.starts_with('>') {
                                Style::default().fg(Color::Cyan)
                            } else if line.contains('✗') {
                                Style::default().fg(Color::Red)
                            } else {
                                Style::default().fg(Color::Green)
                            };
                            ListItem::new(Span::styled(line, style))
                        })
                        .collect();

                    let list = List::new(items).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Command History"),
                    );
                    f.render_widget(list, chunks[2]);
                }
                2 => {
                    // Help tab
                    let help_text = vec![
                        Line::from(Span::styled(
                            "╔═════════════════════════════════════════════════════════╗",
                            Style::default().fg(Color::Cyan),
                        )),
                        Line::from(Span::styled(
                            "║              MINI REDIS DASHBOARD HELP                   ║",
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        )),
                        Line::from(Span::styled(
                            "╚═════════════════════════════════════════════════════════╝",
                            Style::default().fg(Color::Cyan),
                        )),
                        Line::from(Span::raw("")),
                        Line::from(Span::styled(
                            "⌨️  KEYBOARD SHORTCUTS",
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        )),
                        Line::from(Span::styled(
                            "  q          Quit dashboard",
                            Style::default().fg(Color::White),
                        )),
                        Line::from(Span::styled(
                            "  i          Enter command mode",
                            Style::default().fg(Color::White),
                        )),
                        Line::from(Span::styled(
                            "  Esc        Exit command mode",
                            Style::default().fg(Color::White),
                        )),
                        Line::from(Span::styled(
                            "  j/k        Navigate keys up/down",
                            Style::default().fg(Color::White),
                        )),
                        Line::from(Span::styled(
                            "  ↑/↓        Navigate keys",
                            Style::default().fg(Color::White),
                        )),
                        Line::from(Span::styled(
                            "  Tab        Next tab",
                            Style::default().fg(Color::White),
                        )),
                        Line::from(Span::styled(
                            "  Shift+Tab  Previous tab",
                            Style::default().fg(Color::White),
                        )),
                        Line::from(Span::raw("")),
                        Line::from(Span::styled(
                            "📝 BASIC COMMANDS",
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        )),
                        Line::from(Span::styled(
                            "  SET <key> <value> [EXP <sec>]",
                            Style::default().fg(Color::White),
                        )),
                        Line::from(Span::styled(
                            "  GET <key>",
                            Style::default().fg(Color::White),
                        )),
                        Line::from(Span::styled(
                            "  DEL <key>",
                            Style::default().fg(Color::White),
                        )),
                        Line::from(Span::styled(
                            "  EXISTS <key> [<key> ...]",
                            Style::default().fg(Color::White),
                        )),
                        Line::from(Span::styled(
                            "  TTL <key>",
                            Style::default().fg(Color::White),
                        )),
                        Line::from(Span::raw("")),
                        Line::from(Span::styled(
                            "🔢 COUNTER COMMANDS",
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        )),
                        Line::from(Span::styled(
                            "  INCR <key>    Increment by 1",
                            Style::default().fg(Color::White),
                        )),
                        Line::from(Span::styled(
                            "  DECR <key>    Decrement by 1",
                            Style::default().fg(Color::White),
                        )),
                        Line::from(Span::raw("")),
                        Line::from(Span::styled(
                            "💾 PERSISTENCE COMMANDS",
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        )),
                        Line::from(Span::styled(
                            "  SAVE <file.json>    Save to ./data/<file.json>",
                            Style::default().fg(Color::White),
                        )),
                        Line::from(Span::styled(
                            "  LOAD <file.json>    Load from ./data/<file.json>",
                            Style::default().fg(Color::White),
                        )),
                        Line::from(Span::styled(
                            "  DROP                Clear all keys",
                            Style::default().fg(Color::White),
                        )),
                        Line::from(Span::raw("")),
                        Line::from(Span::styled(
                            "📡 PUB/SUB COMMANDS",
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        )),
                        Line::from(Span::styled(
                            "  SUB <channel>       Subscribe to channel",
                            Style::default().fg(Color::White),
                        )),
                        Line::from(Span::styled(
                            "  PUB <channel> <msg> Publish message",
                            Style::default().fg(Color::White),
                        )),
                        Line::from(Span::styled(
                            "  UNSUB <channel>     Unsubscribe",
                            Style::default().fg(Color::White),
                        )),
                        Line::from(Span::raw("")),
                        Line::from(Span::styled(
                            "💡 EXAMPLES",
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        )),
                        Line::from(Span::styled(
                            "  SET user:1 \"Alice\" EXP 60    Set with 60s TTL",
                            Style::default().fg(Color::White),
                        )),
                        Line::from(Span::styled(
                            "  GET user:1                    Get value",
                            Style::default().fg(Color::White),
                        )),
                        Line::from(Span::styled(
                            "  TTL user:1                    Get remaining TTL",
                            Style::default().fg(Color::White),
                        )),
                        Line::from(Span::styled(
                            "  INCR counter                  Increment counter",
                            Style::default().fg(Color::White),
                        )),
                        Line::from(Span::styled(
                            "  EXISTS key1 key2 key3        Check multiple keys",
                            Style::default().fg(Color::White),
                        )),
                        Line::from(Span::raw("")),
                        Line::from(Span::styled(
                            "ℹ️  TIPS",
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        )),
                        Line::from(Span::styled(
                            "  • TTL shows milliseconds remaining",
                            Style::default().fg(Color::White),
                        )),
                        Line::from(Span::styled(
                            "  • Keys with TTL show countdown",
                            Style::default().fg(Color::White),
                        )),
                        Line::from(Span::styled(
                            "  • Use Tab to switch between views",
                            Style::default().fg(Color::White),
                        )),
                        Line::from(Span::styled(
                            "  • Run 'cargo run' for TCP-only mode",
                            Style::default().fg(Color::White),
                        )),
                    ];

                    let paragraph = Paragraph::new(help_text)
                        .block(Block::default().borders(Borders::ALL).title("Help"));
                    f.render_widget(paragraph, chunks[2]);
                }
                _ => {}
            }

            // Footer
            let footer_text = if app.mode == Mode::Insert {
                format!(
                    "Command: {} (Enter to execute, Esc to cancel)",
                    app.command_input
                )
            } else {
                "Press 'q' to quit | 'i' to enter command | arrows to navigate | Tab to switch"
                    .to_string()
            };

            let footer_style = if app.mode == Mode::Insert {
                Style::default().fg(Color::Cyan).bg(Color::Black)
            } else {
                Style::default().fg(Color::Gray).bg(Color::Black)
            };

            let footer = Paragraph::new(Span::styled(footer_text, footer_style))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[3]);
        })?;

        if event::poll(Duration::from_millis(250))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.mode {
                        Mode::Normal => match key.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Char('i') => app.mode = Mode::Insert,
                            KeyCode::Down => app.down(),
                            KeyCode::Up => app.up(),
                            KeyCode::Tab => app.next_tab(),
                            KeyCode::BackTab => app.prev_tab(),
                            _ => {}
                        },
                        Mode::Insert => match key.code {
                            KeyCode::Esc => {
                                app.command_input.clear();
                                app.mode = Mode::Normal;
                            }
                            KeyCode::Enter => {
                                app.execute();
                                app.mode = Mode::Normal;
                            }
                            KeyCode::Char(c) => app.command_input.push(c),
                            KeyCode::Backspace => {
                                app.command_input.pop();
                            }
                            _ => {}
                        },
                    }
                }
            }
        }

        app.refresh_keys();
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
