use anyhow::Result;
use cfonts::{render, Colors, Options, Rgb};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    prelude::Stylize,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Wrap,
    },
    Frame, Terminal,
};
use std::io::{self, Stdout};
use tokio::sync::mpsc;

// ─── Terminal lifecycle ────────────────────────────────────────────────────────

pub type Term = Terminal<CrosstermBackend<Stdout>>;

/// Set up the alternate screen, raw mode, and return a Terminal.
pub fn enter() -> Result<Term> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

/// Restore the terminal to its original state.
pub fn leave(mut term: Term) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;
    Ok(())
}

// ─── Palette ──────────────────────────────────────────────────────────────────

pub const GREEN: Color = Color::Rgb(72, 214, 108);
pub const DIM: Color = Color::Rgb(100, 115, 100);
pub const DARK_BG: Color = Color::Rgb(15, 20, 15);
pub const PANEL_BG: Color = Color::Rgb(20, 28, 20);
pub const INPUT_BG: Color = Color::Rgb(25, 35, 25);
pub const ACCENT: Color = Color::Rgb(130, 255, 160);

// ─── Banner (cfonts, printed before entering raw mode) ────────────────────────

/// Display a banner with the given text using cfonts (plain stdout, before TUI mode).
pub fn print_banner(text: &str) {
    let mut options = Options::default();
    options.colors = vec![
        Colors::Rgb(Rgb::Val(72, 214, 108)),
        Colors::Rgb(Rgb::Val(72, 214, 108)),
    ];
    options.text = text.to_string();
    options.letter_spacing = 0;
    let output = render(options);
    println!("{}", output.text);
}

// ─── Coder-picker (client side) ───────────────────────────────────────────────

/// Interactively pick one item from `entries` (label strings).
/// Returns the selected index, or `None` if the user pressed Escape / q.
pub fn pick_from_list(title: &str, entries: &[String]) -> Result<Option<usize>> {
    if entries.is_empty() {
        return Ok(None);
    }
    let mut term = enter()?;
    let mut state = ListState::default();
    state.select(Some(0));

    loop {
        term.draw(|f| draw_picker(f, title, entries, &mut state))?;

        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match (key.code, key.modifiers) {
                    (KeyCode::Char('c'), KeyModifiers::CONTROL)
                    | (KeyCode::Char('q'), _)
                    | (KeyCode::Esc, _) => {
                        leave(term)?;
                        return Ok(None);
                    }
                    (KeyCode::Down | KeyCode::Char('j'), _) => {
                        let i = state.selected().unwrap_or(0);
                        state.select(Some((i + 1).min(entries.len() - 1)));
                    }
                    (KeyCode::Up | KeyCode::Char('k'), _) => {
                        let i = state.selected().unwrap_or(0);
                        state.select(Some(i.saturating_sub(1)));
                    }
                    (KeyCode::Enter, _) => {
                        let selected = state.selected();
                        leave(term)?;
                        return Ok(selected);
                    }
                    _ => {}
                }
            }
        }
    }
}

fn draw_picker(f: &mut Frame, title: &str, entries: &[String], state: &mut ListState) {
    let area = f.area();
    f.render_widget(Block::default().style(Style::default().bg(DARK_BG)), area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .margin(2)
        .split(area);

    // Title bar
    let title_block = Paragraph::new(title)
        .alignment(Alignment::Center)
        .style(Style::default().fg(GREEN).add_modifier(Modifier::BOLD))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(DIM))
                .border_type(BorderType::Plain),
        );
    f.render_widget(title_block, chunks[0]);

    // Coder list
    let items: Vec<ListItem> = entries
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let num = Span::styled(format!(" {:>2}.  ", i + 1), Style::default().fg(DIM));
            let text = Span::styled(label.clone(), Style::default().fg(Color::White));
            ListItem::new(Line::from(vec![num, text]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(DIM))
                .bg(PANEL_BG)
                .title(Span::styled(
                    " Available Coders ",
                    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                )),
        )
        .highlight_style(
            Style::default()
                .bg(GREEN)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    f.render_stateful_widget(list, chunks[1], state);

    // Help line
    let help = Paragraph::new("↑/↓ or j/k to navigate  •  Enter to select  •  q / Esc to quit")
        .alignment(Alignment::Center)
        .style(Style::default().fg(DIM));
    f.render_widget(help, chunks[2]);
}

// ─── Chat session TUI ─────────────────────────────────────────────────────────

/// A message in the chat widget.
#[derive(Clone)]
pub struct ChatMsg {
    pub role: ChatRole,
    pub text: String,
}

#[derive(Clone, PartialEq)]
pub enum ChatRole {
    /// Local user input (question / answer)
    You,
    /// Remote peer (coder's answer / client's question)
    Peer,
    /// System info (connection events, command results …)
    System,
}

/// Events published by the chat TUI back to the caller.
pub enum ChatEvent {
    /// The user submitted a line of text (pressed Enter).
    Line(String),
    /// The user pressed Ctrl+D (signals EOF / done).
    Eof,
    /// The user pressed Ctrl+C or q (wants to quit the session).
    Quit,
}

/// Push a message into the shared log and wake the draw loop.
#[allow(dead_code)]
pub fn append_msg(log: &mut Vec<ChatMsg>, role: ChatRole, text: impl Into<String>) {
    log.push(ChatMsg {
        role,
        text: text.into(),
    });
}

/// Run the full interactive chat TUI.
///
/// * `title`        — window title shown in the border
/// * `prompt_label` — label shown in the input box (e.g. "Question" / "Answer")
/// * `log_rx`       — channel that delivers `ChatMsg`s produced by the async task
/// * `event_tx`     — channel on which we send `ChatEvent`s back to the async task
pub async fn run_chat(
    title: String,
    prompt_label: String,
    mut log_rx: mpsc::UnboundedReceiver<ChatMsg>,
    event_tx: mpsc::UnboundedSender<ChatEvent>,
) -> Result<()> {
    let mut term = enter()?;
    let mut log: Vec<ChatMsg> = Vec::new();
    let mut input = String::new();
    let mut scroll_offset: usize = 0;
    let mut auto_scroll = true;

    loop {
        // Drain all pending log messages
        while let Ok(msg) = log_rx.try_recv() {
            log.push(msg);
            if auto_scroll {
                scroll_offset = log.len().saturating_sub(1);
            }
        }

        term.draw(|f| {
            draw_chat(f, &title, &prompt_label, &log, &input, scroll_offset);
        })?;

        // Poll for keyboard events (non-blocking, 50 ms timeout)
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match (key.code, key.modifiers) {
                    // Quit
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        let _ = event_tx.send(ChatEvent::Quit);
                        break;
                    }
                    // EOF / done
                    (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                        let _ = event_tx.send(ChatEvent::Eof);
                        input.clear();
                    }
                    // Submit line
                    (KeyCode::Enter, _) => {
                        let line = input.trim().to_string();
                        input.clear();
                        if !line.is_empty() {
                            let _ = event_tx.send(ChatEvent::Line(line));
                        }
                    }
                    // Typing
                    (KeyCode::Char(c), _) => {
                        input.push(c);
                    }
                    (KeyCode::Backspace, _) => {
                        input.pop();
                    }
                    // Manual scroll
                    (KeyCode::PageUp, _) => {
                        auto_scroll = false;
                        scroll_offset = scroll_offset.saturating_sub(10);
                    }
                    (KeyCode::PageDown, _) => {
                        if scroll_offset + 10 >= log.len().saturating_sub(1) {
                            auto_scroll = true;
                            scroll_offset = log.len().saturating_sub(1);
                        } else {
                            scroll_offset += 10;
                        }
                    }
                    (KeyCode::Up, _) => {
                        auto_scroll = false;
                        scroll_offset = scroll_offset.saturating_sub(1);
                    }
                    (KeyCode::Down, _) => {
                        if scroll_offset + 1 >= log.len().saturating_sub(1) {
                            auto_scroll = true;
                            scroll_offset = log.len().saturating_sub(1);
                        } else {
                            scroll_offset += 1;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    leave(term)?;
    Ok(())
}

fn draw_chat(
    f: &mut Frame,
    title: &str,
    prompt_label: &str,
    log: &[ChatMsg],
    input: &str,
    scroll_offset: usize,
) {
    let area = f.area();

    // Background
    f.render_widget(Block::default().style(Style::default().bg(DARK_BG)), area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Min(1),    // message log
            Constraint::Length(3), // input box
            Constraint::Length(1), // help
        ])
        .split(area);

    // ── Header ────────────────────────────────────────────────────────────────
    let header = Paragraph::new(title)
        .alignment(Alignment::Center)
        .style(Style::default().fg(GREEN).add_modifier(Modifier::BOLD))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(DIM)),
        );
    f.render_widget(header, chunks[0]);

    // ── Message log ───────────────────────────────────────────────────────────
    let msg_area = chunks[1];
    let inner_height = msg_area.height.saturating_sub(2) as usize; // inside borders

    let lines: Vec<Line> = log.iter().flat_map(|m| render_chat_msg(m)).collect();

    let total_lines = lines.len();
    let safe_offset = scroll_offset.min(total_lines.saturating_sub(inner_height));

    let visible: Vec<Line> = lines
        .into_iter()
        .skip(safe_offset)
        .take(inner_height)
        .collect();

    let log_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DIM))
        .bg(PANEL_BG);

    let log_widget = Paragraph::new(visible)
        .block(log_block)
        .wrap(Wrap { trim: false });
    f.render_widget(log_widget, msg_area);

    // Scrollbar
    if total_lines > inner_height {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        let mut sb_state =
            ScrollbarState::new(total_lines.saturating_sub(inner_height)).position(safe_offset);
        f.render_stateful_widget(
            scrollbar,
            msg_area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut sb_state,
        );
    }

    // ── Input box ─────────────────────────────────────────────────────────────
    let display_input = format!("{}_", input); // blinking-cursor simulation
    let input_paragraph = Paragraph::new(display_input)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(GREEN))
                .bg(INPUT_BG)
                .title(Span::styled(
                    format!(" {} ", prompt_label),
                    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                )),
        );
    f.render_widget(input_paragraph, chunks[2]);

    // ── Help bar ──────────────────────────────────────────────────────────────
    let help = Paragraph::new(
        "Enter → send  •  Ctrl+D → done/finish  •  Ctrl+C → quit  •  ↑/↓ PgUp/PgDn → scroll",
    )
    .alignment(Alignment::Center)
    .style(Style::default().fg(DIM));
    f.render_widget(help, chunks[3]);
}

fn render_chat_msg(msg: &ChatMsg) -> Vec<Line<'static>> {
    match msg.role {
        ChatRole::You => {
            let mut lines = vec![Line::from(vec![Span::styled(
                "▶ You",
                Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
            )])];
            for l in msg.text.lines() {
                lines.push(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(l.to_owned(), Style::default().fg(Color::White)),
                ]));
            }
            lines.push(Line::from(""));
            lines
        }
        ChatRole::Peer => {
            let mut lines = vec![Line::from(vec![Span::styled(
                "◀ Peer",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            )])];
            for l in msg.text.lines() {
                lines.push(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(l.to_owned(), Style::default().fg(Color::White)),
                ]));
            }
            lines.push(Line::from(""));
            lines
        }
        ChatRole::System => {
            let mut lines = vec![];
            for l in msg.text.lines() {
                lines.push(Line::from(vec![Span::styled(
                    format!("  ⓘ  {}", l),
                    Style::default().fg(DIM).add_modifier(Modifier::ITALIC),
                )]));
            }
            lines.push(Line::from(""));
            lines
        }
    }
}
