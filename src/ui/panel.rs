use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders},
};

#[derive(Clone, Copy, PartialEq)]
pub enum Panel {
    Category,
    Content,
    Details,
}

impl Panel {
    pub fn next(self) -> Self {
        match self {
            Panel::Category => Panel::Content,
            Panel::Content => Panel::Details,
            Panel::Details => Panel::Category,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Panel::Category => Panel::Details,
            Panel::Content => Panel::Category,
            Panel::Details => Panel::Content,
        }
    }
}

pub fn focused_block(title: &str, focused: bool) -> Block<'_> {
    let border_style = if focused {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    };
    Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(format!(" {} ", title))
}
