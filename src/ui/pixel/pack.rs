use ratatui::{
    style::Style,
    text::{Line, Span, Text},
};

use super::{Canvas, ColorRole, Palette};

pub fn pack(canvas: &Canvas, palette: &Palette, background: ColorRole) -> Text<'static> {
    let background = palette.resolve(background);
    let mut lines = Vec::with_capacity(usize::from(canvas.height().div_ceil(2)));

    for top_y in (0..canvas.height()).step_by(2) {
        let mut spans = Vec::with_capacity(usize::from(canvas.width()));

        for x in 0..canvas.width() {
            let top = canvas.pixel(x, top_y);
            let bottom = canvas.pixel(x, top_y.saturating_add(1));
            let (glyph, foreground, cell_background) = match (top, bottom) {
                (None, None) => (" ", None, background),
                (Some(role), None) => ("▀", Some(palette.resolve(role)), background),
                (None, Some(role)) => ("▄", Some(palette.resolve(role)), background),
                (Some(top), Some(bottom)) if top == bottom => {
                    ("█", Some(palette.resolve(top)), background)
                }
                (Some(top), Some(bottom)) => {
                    ("▀", Some(palette.resolve(top)), palette.resolve(bottom))
                }
            };

            let mut style = Style::new().bg(cell_background);
            if let Some(foreground) = foreground {
                style = style.fg(foreground);
            }
            spans.push(Span::styled(glyph, style));
        }

        lines.push(Line::from(spans));
    }

    Text::from(lines)
}
