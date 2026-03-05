use cfonts::{render, Colors, Fonts, Options};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Terminal,
};
use std::io;

const MENU_ITEMS: &[&str] = &["Projects", "Members"];

fn build_logo() -> Vec<Line<'static>> {
    let parts: &[(&str, Option<Color>)] =
        &[("ut.", None), ("code", Some(Color::Green)), ("();", None)];
    let row_groups: Vec<(Vec<String>, Option<Color>)> = parts
        .iter()
        .map(|(text, color)| {
            let rendered = render(Options {
                text: String::from(*text),
                font: Fonts::FontBlock,
                colors: vec![Colors::System],
                spaceless: true,
                ..Options::default()
            });
            let lines: Vec<String> = rendered.text.lines().map(String::from).collect();
            (lines, *color)
        })
        .collect();
    let num_rows = row_groups.iter().map(|(g, _)| g.len()).max().unwrap_or(0);
    (0..num_rows)
        .map(|i| {
            let spans: Vec<Span<'static>> = row_groups
                .iter()
                .map(|(g, color)| {
                    let text: String = g.get(i).cloned().unwrap_or_default();
                    match color {
                        Some(c) => Span::styled(text, Style::default().fg(*c)),
                        None => Span::raw(text),
                    }
                })
                .collect();
            Line::from(spans)
        })
        .collect()
}

fn page_content(index: usize) -> Vec<Line<'static>> {
    match index {
        0 => vec![
            Line::from(" ut.code(); CLI"),
            Line::from(" coursemate"),
            Line::from(" ut.code(); Learn"),
        ],
        1 => vec![
            Line::from(" aster-void"),
            Line::from(" nakaterm"),
            Line::from(" tknkaa"),
        ],
        _ => vec![],
    }
}

pub fn run() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let logo_lines = build_logo();
    let logo_height = logo_lines.len() as u16 + 2; // +2 for border

    let mut list_state = ListState::default();
    list_state.select(Some(0));

    loop {
        let selected = list_state.selected().unwrap_or(0);

        terminal.draw(|f| {
            let vertical = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(logo_height), Constraint::Min(0)])
                .split(f.area());

            // Logo at top
            let logo =
                Paragraph::new(logo_lines.clone()).block(Block::default().borders(Borders::NONE));
            f.render_widget(logo, vertical[0]);

            // Bottom: left menu + right content
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(20), Constraint::Min(0)])
                .split(vertical[1]);

            // Left menu
            let items: Vec<ListItem> = MENU_ITEMS.iter().map(|s| ListItem::new(*s)).collect();
            let menu = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(" Menu "))
                .highlight_style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(" ");
            f.render_stateful_widget(menu, chunks[0], &mut list_state);

            // Right content
            let content = Paragraph::new(page_content(selected))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!(" {} ", MENU_ITEMS[selected])),
                )
                .wrap(Wrap { trim: false });
            f.render_widget(content, chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Down | KeyCode::Char('j') => {
                    let next = (selected + 1) % MENU_ITEMS.len();
                    list_state.select(Some(next));
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let prev = if selected == 0 {
                        MENU_ITEMS.len() - 1
                    } else {
                        selected - 1
                    };
                    list_state.select(Some(prev));
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
