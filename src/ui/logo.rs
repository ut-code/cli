use cfonts::{render, Colors, Fonts, Options};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

pub fn build_logo() -> Vec<Line<'static>> {
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
