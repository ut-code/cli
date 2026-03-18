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
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

// ─── Terminal lifecycle ────────────────────────────────────────────────────────

pub type Term = Terminal<CrosstermBackend<Stdout>>;

pub fn enter() -> Result<Term> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

pub fn leave(term: &mut Term) -> Result<()> {
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
pub const WARN: Color = Color::Rgb(220, 130, 40);

// ─── Banner ───────────────────────────────────────────────────────────────────

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

// ─── Shared types ─────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
pub enum ChatRole {
    You,
    Peer,
    System,
}

/// Messages sent from the async task → TUI draw loop.
#[derive(Clone)]
pub enum ChatMsg {
    Msg { role: ChatRole, text: String },
    SetWaiting(bool),
    OpenEditor(String),
}

impl ChatMsg {
    pub fn sys(text: impl Into<String>) -> Self {
        ChatMsg::Msg {
            role: ChatRole::System,
            text: text.into(),
        }
    }
    pub fn you(text: impl Into<String>) -> Self {
        ChatMsg::Msg {
            role: ChatRole::You,
            text: text.into(),
        }
    }
    pub fn peer(text: impl Into<String>) -> Self {
        ChatMsg::Msg {
            role: ChatRole::Peer,
            text: text.into(),
        }
    }
}

/// Events sent from the TUI → async task.
pub enum ChatEvent {
    Line(String),
    Eof,
    Quit,
    EditorClosed,
}

// ─── Tab-completion state ──────────────────────────────────────────────────────

/// Tracks an in-progress @path tab-completion cycle.
struct TabState {
    /// Everything in the input up to and including the `@` character.
    prefix: String,
    /// The directory component of the path being completed (e.g. `"src/"`).
    dir_display: String,
    /// Sorted list of candidate filenames (directories have a trailing `/`).
    completions: Vec<String>,
    /// Index of the currently shown completion.
    index: usize,
}

/// Split a partial path (the text after `@`) into the directory to read, the
/// display prefix for that directory, and the filename prefix to filter by.
///
/// Examples:
///   `"src/ma"` → (`"src"`, `"src/"`, `"ma"`)
///   `"ma"`     → (`"."`,   `""`,    `"ma"`)
///   `"/usr/lo"` → (`"/usr"`, `"/usr/"`, `"lo"`)
///   `"/lo"`    → (`"/"`,   `"/"`,   `"lo"`)
fn split_path(partial: &str) -> (String, String, String) {
    if let Some(slash_pos) = partial.rfind('/') {
        let dir_str = &partial[..slash_pos]; // e.g. "src" or "" for root
        let dir_display = format!("{}/", dir_str); // e.g. "src/" or "/"
        let dir_to_read = if slash_pos == 0 {
            "/".to_string()
        } else {
            dir_str.to_string()
        };
        let file_prefix = partial[slash_pos + 1..].to_string();
        (dir_to_read, dir_display, file_prefix)
    } else {
        (".".to_string(), String::new(), partial.to_string())
    }
}

/// Return a sorted list of filesystem entries inside `dir` whose names start
/// with `prefix`.  Directories are returned with a trailing `/`.
///
/// Hidden entries (names starting with `.`) are only included when `prefix`
/// itself starts with `.`, matching typical shell completion behaviour.
async fn get_path_completions(dir: &str, prefix: &str) -> Vec<String> {
    let mut entries = Vec::new();
    if let Ok(mut rd) = tokio::fs::read_dir(dir).await {
        while let Ok(Some(entry)) = rd.next_entry().await {
            let name = entry.file_name().to_string_lossy().into_owned();
            // Skip hidden files unless the user explicitly typed a leading dot.
            if name.starts_with('.') && !prefix.starts_with('.') {
                continue;
            }
            if name.starts_with(prefix) {
                let is_dir = entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false);
                let display = if is_dir { format!("{}/", name) } else { name };
                entries.push(display);
            }
        }
    }
    entries.sort();
    entries
}

// ─── Coder picker ─────────────────────────────────────────────────────────────

pub fn pick_from_list(title: &str, entries: &[String]) -> Result<Option<usize>> {
    if entries.is_empty() {
        return Ok(None);
    }
    let mut term = enter()?;
    let mut state = ListState::default();
    state.select(Some(0));

    loop {
        term.draw(|f| draw_picker(f, title, entries, &mut state))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match (key.code, key.modifiers) {
                    (KeyCode::Char('c'), KeyModifiers::CONTROL)
                    | (KeyCode::Char('q'), _)
                    | (KeyCode::Esc, _) => {
                        leave(&mut term)?;
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
                        leave(&mut term)?;
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

    let help = Paragraph::new("↑/↓ or j/k to navigate  •  Enter to select  •  q / Esc to quit")
        .alignment(Alignment::Center)
        .style(Style::default().fg(DIM));
    f.render_widget(help, chunks[2]);
}

// ─── Chat session TUI ─────────────────────────────────────────────────────────

pub async fn run_chat(
    title: String,
    prompt_label: String,
    mut log_rx: mpsc::UnboundedReceiver<ChatMsg>,
    event_tx: mpsc::UnboundedSender<ChatEvent>,
) -> Result<()> {
    let mut term = enter()?;
    let mut log: Vec<(ChatRole, String)> = Vec::new();
    let mut input = String::new();

    // Tab-completion state ──────────────────────────────────────────────────────
    let mut tab_state: Option<TabState> = None;

    // Scroll state ──────────────────────────────────────────────────────────────
    // `scroll_offset` is always in rendered-line units (not message-entry units).
    // `at_bottom = true` means we track the tail automatically; in that mode
    // `scroll_offset` is ignored — the effective offset is computed from
    // total_lines / inner_height each frame.
    let mut scroll_offset: usize = 0;
    let mut at_bottom = true;

    // Input lock + temporary warning ───────────────────────────────────────────
    let mut waiting = false;
    let mut warn_until: Option<Instant> = None;

    loop {
        // ── Drain incoming control / log messages ──────────────────────────────
        while let Ok(msg) = log_rx.try_recv() {
            match msg {
                ChatMsg::Msg { role, text } => {
                    log.push((role, text));
                    // Don't touch scroll_offset here — it will be computed correctly
                    // per-frame based on at_bottom + rendered line count.
                }
                ChatMsg::SetWaiting(w) => {
                    waiting = w;
                    if !w {
                        warn_until = None; // clear any leftover warning when unblocked
                    }
                }
                ChatMsg::OpenEditor(path) => {
                    // Suspend the TUI so the editor gets a clean terminal.
                    leave(&mut term)?;
                    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string());
                    let _ = std::process::Command::new(&editor).arg(&path).status();
                    // Restore the TUI.
                    enable_raw_mode()?;
                    execute!(term.backend_mut(), EnterAlternateScreen, EnableMouseCapture)?;
                    term.clear()?;
                    let _ = event_tx.send(ChatEvent::EditorClosed);
                }
            }
        }

        // ── Pre-compute rendered lines and layout metrics ──────────────────────
        //
        // Layout (no margin):
        //   chunks[0] = Length(3)  →  header
        //   chunks[1] = Min(1)     →  log  (height = term_h − 7)
        //   chunks[2] = Length(3)  →  input
        //   chunks[3] = Length(1)  →  help
        //
        // log inner height = (term_h − 7) − 2 borders = term_h − 9
        //
        // We pre-render lines here so both the scroll-key handler and draw_chat
        // share the same line count, preventing the message-index vs line-index
        // mismatch that broke scrolling.

        let rendered: Vec<Line<'static>> = log.iter().flat_map(|(r, t)| render_msg(r, t)).collect();
        let total_lines = rendered.len();

        let term_h = term.size().map(|s| s.height as usize).unwrap_or(24);
        let inner_h = term_h.saturating_sub(9).max(1);
        let max_offset = total_lines.saturating_sub(inner_h);

        // Effective scroll position (clamped, at_bottom always shows the tail)
        let effective = if at_bottom {
            max_offset
        } else {
            scroll_offset.min(max_offset)
        };

        // ── Warning text (shown 2 s after a blocked keypress) ──────────────────
        let now = Instant::now();
        let warning: Option<&str> = warn_until
            .filter(|&t| t > now)
            .map(|_| "⚠  Cannot send while waiting — please wait for a response.");

        // ── Draw ───────────────────────────────────────────────────────────────
        term.draw(|f| {
            draw_chat(
                f,
                &title,
                &prompt_label,
                &rendered,
                total_lines,
                inner_h,
                effective,
                &input,
                waiting,
                warning,
            );
        })?;

        // ── Key events ─────────────────────────────────────────────────────────
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match (key.code, key.modifiers) {
                    // Always-on: quit
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        let _ = event_tx.send(ChatEvent::Quit);
                        break;
                    }
                    // Always-on: Ctrl+D (Eof / finish answer — needed by coder)
                    (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                        let _ = event_tx.send(ChatEvent::Eof);
                        input.clear();
                    }
                    // ── Tab: @path completion ──────────────────────────────────
                    (KeyCode::Tab, _) if waiting => {
                        warn_until = Some(now + Duration::from_secs(2));
                    }
                    (KeyCode::Tab, _) => {
                        if let Some(state) = tab_state.as_mut() {
                            // `state.completions` is guaranteed non-empty: TabState is only
                            // created when the completions vec has at least one element.
                            state.index = (state.index + 1) % state.completions.len();
                            input = format!(
                                "{}{}{}",
                                state.prefix, state.dir_display, state.completions[state.index]
                            );
                        } else if let Some(at_pos) = input.rfind('@') {
                            let partial_path = input[at_pos + 1..].to_string();
                            let prefix_part = input[..=at_pos].to_string();
                            let (dir_to_read, dir_display, file_prefix) = split_path(&partial_path);
                            let completions =
                                get_path_completions(&dir_to_read, &file_prefix).await;
                            if !completions.is_empty() {
                                input = format!("{}{}{}", prefix_part, dir_display, completions[0]);
                                tab_state = Some(TabState {
                                    prefix: prefix_part,
                                    dir_display,
                                    completions,
                                    index: 0,
                                });
                            }
                        }
                    }
                    // ── Blocked input: show 2-second warning ────────────────────
                    (KeyCode::Enter | KeyCode::Char(_) | KeyCode::Backspace, _) if waiting => {
                        warn_until = Some(now + Duration::from_secs(2));
                    }
                    // ── Normal typing ──────────────────────────────────────────
                    (KeyCode::Enter, _) => {
                        tab_state = None;
                        let line = input.trim().to_string();
                        input.clear();
                        if !line.is_empty() {
                            let _ = event_tx.send(ChatEvent::Line(line));
                        }
                    }
                    (KeyCode::Char(c), _) => {
                        tab_state = None;
                        input.push(c);
                    }
                    (KeyCode::Backspace, _) => {
                        tab_state = None;
                        input.pop();
                    }
                    // ── Scrolling ──────────────────────────────────────────────
                    // Always uses `effective` as the base so the first Up/PageUp
                    // from at_bottom mode starts from the true last line, not a
                    // stale scroll_offset value.
                    (KeyCode::Up, _) => {
                        at_bottom = false;
                        scroll_offset = effective.saturating_sub(1);
                    }
                    (KeyCode::Down, _) => {
                        let new = effective + 1;
                        if new >= max_offset {
                            at_bottom = true;
                        } else {
                            at_bottom = false;
                            scroll_offset = new;
                        }
                    }
                    (KeyCode::PageUp, _) => {
                        at_bottom = false;
                        scroll_offset = effective.saturating_sub(inner_h.max(1) / 2);
                    }
                    (KeyCode::PageDown, _) => {
                        let new = effective + inner_h.max(1) / 2;
                        if new >= max_offset {
                            at_bottom = true;
                        } else {
                            at_bottom = false;
                            scroll_offset = new;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Expire the warning once its time is up
        if warn_until.map(|t| now >= t).unwrap_or(false) {
            warn_until = None;
        }
    }

    leave(&mut term)?;
    Ok(())
}

fn draw_chat(
    f: &mut Frame,
    title: &str,
    prompt_label: &str,
    lines: &[Line<'static>],
    total_lines: usize,
    _inner_h: usize,
    scroll_offset: usize, // already computed, clamped effective offset
    input: &str,
    waiting: bool,
    warning: Option<&str>, // non-None = show 2-second blocked warning
) {
    let area = f.area();
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
    f.render_widget(
        Paragraph::new(title)
            .alignment(Alignment::Center)
            .style(Style::default().fg(GREEN).add_modifier(Modifier::BOLD))
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(DIM)),
            ),
        chunks[0],
    );

    // ── Message log ───────────────────────────────────────────────────────────
    let msg_area = chunks[1];

    // Use the actual widget-rendered inner height for slicing, so the visible
    // slice matches exactly what ratatui will render inside the borders.
    let rendered_inner_h = msg_area.height.saturating_sub(2) as usize;
    let safe_offset = scroll_offset.min(total_lines.saturating_sub(rendered_inner_h));

    let visible: Vec<Line> = lines
        .iter()
        .cloned()
        .skip(safe_offset)
        .take(rendered_inner_h)
        .collect();

    f.render_widget(
        Paragraph::new(visible)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(DIM))
                    .bg(PANEL_BG),
            )
            .wrap(Wrap { trim: false }),
        msg_area,
    );

    // Scrollbar — position uses the same safe_offset so thumb tracks correctly
    if total_lines > rendered_inner_h {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        let mut sb_state =
            ScrollbarState::new(total_lines.saturating_sub(rendered_inner_h)).position(safe_offset);
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
    // Priority: warning (orange) > waiting (dim) > normal (green)
    let (border_color, box_title, box_content): (Color, String, Line) = if let Some(msg) = warning {
        (
            WARN,
            format!(" ⚠  {} ", prompt_label),
            Line::from(Span::styled(
                msg,
                Style::default().fg(WARN).add_modifier(Modifier::BOLD),
            )),
        )
    } else if waiting {
        (
            DIM,
            " Waiting… ".to_string(),
            Line::from(Span::styled(
                "  …",
                Style::default().fg(DIM).add_modifier(Modifier::DIM),
            )),
        )
    } else {
        (
            GREEN,
            format!(" {} ", prompt_label),
            Line::from(Span::styled(
                format!("{}_", input),
                Style::default().fg(Color::White),
            )),
        )
    };

    f.render_widget(
        Paragraph::new(box_content).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color))
                .bg(INPUT_BG)
                .title(Span::styled(
                    box_title,
                    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                )),
        ),
        chunks[2],
    );

    // ── Help bar ──────────────────────────────────────────────────────────────
    let help_text = if waiting || warning.is_some() {
        "↑/↓ PgUp/PgDn → scroll  •  Ctrl+C → quit"
    } else {
        "Enter → send  •  @path Tab → complete  •  Ctrl+D → done/finish  •  Ctrl+C → quit  •  ↑/↓ PgUp/PgDn → scroll"
    };
    f.render_widget(
        Paragraph::new(help_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(DIM)),
        chunks[3],
    );
}

// ─── Rendering helpers ────────────────────────────────────────────────────────

fn render_msg(role: &ChatRole, text: &str) -> Vec<Line<'static>> {
    match role {
        ChatRole::You => {
            let mut lines = vec![Line::from(vec![Span::styled(
                "▶ You",
                Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
            )])];
            for l in text.lines() {
                lines.push(Line::from(vec![
                    Span::raw("  "),
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
            for l in text.lines() {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(l.to_owned(), Style::default().fg(Color::White)),
                ]));
            }
            lines.push(Line::from(""));
            lines
        }
        ChatRole::System => {
            let mut lines = vec![];
            for l in text.lines() {
                lines.push(Line::from(Span::styled(
                    format!("  ⓘ  {}", l),
                    Style::default().fg(DIM).add_modifier(Modifier::ITALIC),
                )));
            }
            lines.push(Line::from(""));
            lines
        }
    }
}
