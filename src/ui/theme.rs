use ratatui::style::{Color, Style};

pub(crate) const PARCHMENT: Style = Style::new()
    .fg(Color::Rgb(52, 35, 28))
    .bg(Color::Rgb(230, 207, 154));
pub(crate) const PARCHMENT_BORDER: Style = Style::new()
    .fg(Color::Rgb(248, 196, 92))
    .bg(Color::Rgb(76, 25, 35));
