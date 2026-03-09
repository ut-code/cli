use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Terminal,
};
use std::io;

use super::data::{content_entries, CATEGORIES};
use super::logo::build_logo;
use super::panel::{focused_block, Panel};

pub fn run() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let logo_lines = build_logo();
    let logo_height = logo_lines.len() as u16 + 2;

    let mut category_state = ListState::default();
    category_state.select(Some(0));

    let mut content_state = ListState::default();
    content_state.select(Some(0));

    let mut focus = Panel::Category;

    loop {
        let cat_idx = category_state.selected().unwrap_or(0);
        let entries = content_entries(cat_idx);
        let con_idx = content_state
            .selected()
            .unwrap_or(0)
            .min(entries.len().saturating_sub(1));

        let detail_text: &str = entries.get(con_idx).map(|e| e.detail).unwrap_or("");

        terminal.draw(|f| {
            // Vertical split: logo | body
            let vertical = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(logo_height), Constraint::Min(0)])
                .split(f.area());

            let logo =
                Paragraph::new(logo_lines.clone()).block(Block::default().borders(Borders::NONE));
            f.render_widget(logo, vertical[0]);

            // Horizontal split: category | content | details
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(16),
                    Constraint::Length(22),
                    Constraint::Min(0),
                ])
                .split(vertical[1]);

            // Category panel
            let cat_items: Vec<ListItem> = CATEGORIES.iter().map(|s| ListItem::new(*s)).collect();
            let cat_highlight = if focus == Panel::Category {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().add_modifier(Modifier::DIM)
            };
            let cat_list = List::new(cat_items)
                .block(focused_block("Category", focus == Panel::Category))
                .highlight_style(cat_highlight)
                .highlight_symbol(" ");
            f.render_stateful_widget(cat_list, chunks[0], &mut category_state);

            // Content panel
            let con_items: Vec<ListItem> = entries
                .iter()
                .map(|e| ListItem::new(format!(" {}", e.name)))
                .collect();
            let con_highlight = if focus == Panel::Content {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().add_modifier(Modifier::DIM)
            };
            let con_list = List::new(con_items)
                .block(focused_block(CATEGORIES[cat_idx], focus == Panel::Content))
                .highlight_style(con_highlight)
                .highlight_symbol(" ");
            f.render_stateful_widget(con_list, chunks[1], &mut content_state);

            // Details panel
            let detail_title = entries.get(con_idx).map(|e| e.name).unwrap_or("Details");
            let detail_para = Paragraph::new(detail_text)
                .block(focused_block(detail_title, focus == Panel::Details))
                .wrap(Wrap { trim: false });
            f.render_widget(detail_para, chunks[2]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,

                KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => {
                    focus = focus.next();
                }
                KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => {
                    focus = focus.prev();
                }

                KeyCode::Down | KeyCode::Char('j') => match focus {
                    Panel::Category => {
                        let next = (cat_idx + 1) % CATEGORIES.len();
                        category_state.select(Some(next));
                        content_state.select(Some(0));
                    }
                    Panel::Content => {
                        let len = entries.len();
                        if len > 0 {
                            content_state.select(Some((con_idx + 1) % len));
                        }
                    }
                    Panel::Details => {}
                },
                KeyCode::Up | KeyCode::Char('k') => match focus {
                    Panel::Category => {
                        let prev = if cat_idx == 0 {
                            CATEGORIES.len() - 1
                        } else {
                            cat_idx - 1
                        };
                        category_state.select(Some(prev));
                        content_state.select(Some(0));
                    }
                    Panel::Content => {
                        let len = entries.len();
                        if len > 0 {
                            let prev = if con_idx == 0 { len - 1 } else { con_idx - 1 };
                            content_state.select(Some(prev));
                        }
                    }
                    Panel::Details => {}
                },

                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
