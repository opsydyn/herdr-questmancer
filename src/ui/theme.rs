use ratatui::style::{Color, Style};

pub(crate) const INK: Style = Style::new().fg(Color::Rgb(226, 246, 211));
pub(crate) const ACCENT: Style = Style::new().fg(Color::Rgb(111, 255, 180));
pub(crate) const MUTED: Style = Style::new().fg(Color::Rgb(126, 153, 138));
pub(crate) const PARCHMENT: Style = Style::new()
    .fg(Color::Rgb(52, 35, 28))
    .bg(Color::Rgb(230, 207, 154));
pub(crate) const PARCHMENT_BORDER: Style = Style::new()
    .fg(Color::Rgb(248, 196, 92))
    .bg(Color::Rgb(76, 25, 35));
