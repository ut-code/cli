use anyhow::Result;
use crossterm::{
    event::{Event, EventStream, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use reqwest::Client;
use serde::Deserialize;
use std::io;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::util::build_ws_url;

#[derive(Deserialize, Clone)]
struct QueueEntry {
    #[serde(rename = "roomId")]
    room_id: String,
    name: String,
}

enum AppState {
    SelectingProgrammer,
    AskingQuestion,
    Streaming { response: String },
    Done { response: String },
}

struct App {
    programmers: Vec<QueueEntry>,
    state: AppState,
    question: String,
    selected: usize,
}

impl App {
    fn new(programmers: Vec<QueueEntry>) -> Self {
        App {
            programmers,
            state: AppState::SelectingProgrammer,
            question: String::new(),
            selected: 0,
        }
    }

    fn move_down(&mut self) {
        if matches!(self.state, AppState::SelectingProgrammer) {
            // Use checked comparison to avoid usize underflow panic on empty list
            if self.selected + 1 < self.programmers.len() {
                self.selected += 1;
            }
        }
    }

    fn move_up(&mut self) {
        if matches!(self.state, AppState::SelectingProgrammer) {
            if self.selected > 0 {
                self.selected -= 1;
            }
        }
    }

    fn select(&mut self) {
        self.state = AppState::AskingQuestion;
    }
}

/// Drop guard that restores the terminal on drop (handles panics too).
struct TerminalGuard;
impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

pub async fn run() -> Result<()> {
    dotenvy::dotenv().ok();
    let server_url =
        std::env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:8787".to_string());

    // Fetch queue (before entering raw mode so println! works normally)
    let client = Client::new();
    let programmers = fetch_queue(&client, &server_url).await?;

    if programmers.is_empty() {
        println!("No programmers available right now.");
        return Ok(());
    }

    // Setup terminal
    enable_raw_mode()?;
    let _guard = TerminalGuard; // restores raw mode on drop/panic

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Create app
    let mut app = App::new(programmers);

    // Run app
    let res = run_app(&mut terminal, &mut app, &server_url, &client).await;

    // Restore terminal
    terminal.clear()?;
    terminal.show_cursor()?;
    // _guard drops here, calling disable_raw_mode()

    res
}

async fn fetch_queue(client: &Client, server_url: &str) -> Result<Vec<QueueEntry>> {
    let resp = client
        .get(format!("{}/queue", server_url))
        .send()
        .await?
        .error_for_status()?;
    Ok(resp.json().await?)
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    server_url: &str,
    client: &Client,
) -> Result<()> {
    let mut event_stream = EventStream::new();

    loop {
        terminal.draw(|f| ui(f, app))?;

        let Some(event_result) = event_stream.next().await else {
            break;
        };

        if let Event::Key(key) = event_result? {
            match app.state {
                AppState::SelectingProgrammer => match key.code {
                    KeyCode::Char('j') | KeyCode::Down => app.move_down(),
                    KeyCode::Char('k') | KeyCode::Up => app.move_up(),
                    KeyCode::Enter => app.select(),
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    _ => {}
                },
                AppState::AskingQuestion => match key.code {
                    KeyCode::Char(c) => app.question.push(c),
                    KeyCode::Backspace => {
                        app.question.pop();
                    }
                    KeyCode::Enter => {
                        if !app.question.is_empty() {
                            let selected_prog = app.programmers[app.selected].clone();
                            let question = app.question.clone();
                            stream_response(
                                terminal,
                                app,
                                server_url,
                                selected_prog,
                                question,
                                &mut event_stream,
                            )
                            .await?;
                        }
                    }
                    KeyCode::Esc => {
                        app.state = AppState::SelectingProgrammer;
                        app.selected = 0;
                        app.question.clear();
                        // Refresh the programmer list
                        match fetch_queue(client, server_url).await {
                            Ok(p) => app.programmers = p,
                            Err(_) => {} // keep stale list on network error
                        }
                    }
                    _ => {}
                },
                AppState::Streaming { .. } => {
                    // key events during streaming are handled inside stream_response
                }
                AppState::Done { .. } => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Enter => {
                        app.question.clear();
                        app.state = AppState::AskingQuestion;
                    }
                    KeyCode::Char('b') => {
                        app.question.clear();
                        app.selected = 0;
                        // Refresh the programmer list before going back
                        match fetch_queue(client, server_url).await {
                            Ok(p) => app.programmers = p,
                            Err(_) => {}
                        }
                        app.state = AppState::SelectingProgrammer;
                    }
                    _ => {}
                },
            }
        }
    }

    Ok(())
}

async fn stream_response(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    server_url: &str,
    programmer: QueueEntry,
    question: String,
    event_stream: &mut EventStream,
) -> Result<()> {
    let ws_url = build_ws_url(server_url, &format!("/rooms/{}/user", programmer.room_id));

    let (ws_stream, _) = connect_async(&ws_url).await?;
    let (mut write, mut read) = ws_stream.split();

    write.send(Message::text(question)).await?;

    let mut response = String::new();
    app.state = AppState::Streaming {
        response: response.clone(),
    };

    loop {
        tokio::select! {
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if text == "[DONE]" {
                            break;
                        }
                        response.push_str(&text);
                        app.state = AppState::Streaming { response: response.clone() };
                        terminal.draw(|f| ui(f, app))?;
                    }
                    Some(Ok(Message::Close(_))) | Some(Err(_)) | None => break,
                    _ => {}
                }
            }
            evt = event_stream.next() => {
                if let Some(Ok(Event::Key(key))) = evt {
                    if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                        break;
                    }
                }
            }
        }
    }

    app.state = AppState::Done { response };

    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.area());

    let title = Paragraph::new("Coding Human - Select Programmer").style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(title, chunks[0]);

    match &app.state {
        AppState::SelectingProgrammer => {
            let items: Vec<ListItem> = app
                .programmers
                .iter()
                .enumerate()
                .map(|(idx, prog)| {
                    let style = if idx == app.selected {
                        Style::default()
                            .bg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    ListItem::new(format!("  {}", prog.name)).style(style)
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().title("Programmers").borders(Borders::ALL))
                .style(Style::default().fg(Color::White));

            f.render_widget(list, chunks[1]);

            let help = Paragraph::new("j/k or ↑/↓ to navigate | Enter to select | q/Esc to quit");
            f.render_widget(help, chunks[2]);
        }
        AppState::AskingQuestion => {
            let question_para = Paragraph::new(app.question.as_str())
                .block(
                    Block::default()
                        .title("Your Question")
                        .borders(Borders::ALL),
                )
                .style(Style::default().fg(Color::Green));
            f.render_widget(question_para, chunks[1]);

            let help = Paragraph::new("Type your question | Enter to send | Esc to cancel");
            f.render_widget(help, chunks[2]);
        }
        AppState::Streaming { response } => {
            let response_para = Paragraph::new(response.as_str())
                .block(
                    Block::default()
                        .title("Response (streaming...)")
                        .borders(Borders::ALL),
                )
                .wrap(Wrap { trim: false });
            f.render_widget(response_para, chunks[1]);

            let help = Paragraph::new("q/Esc to cancel");
            f.render_widget(help, chunks[2]);
        }
        AppState::Done { response } => {
            let response_para = Paragraph::new(response.as_str())
                .block(Block::default().title("Response").borders(Borders::ALL))
                .wrap(Wrap { trim: false });
            f.render_widget(response_para, chunks[1]);

            let help =
                Paragraph::new("Enter to ask another question | b to go back | q/Esc to quit");
            f.render_widget(help, chunks[2]);
        }
    }
}
