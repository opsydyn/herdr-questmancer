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
            let top = canvas.pixel(x, top_y).map(|role| palette.resolve(role));
            let bottom = canvas
                .pixel(x, top_y.saturating_add(1))
                .map(|role| palette.resolve(role));
            let (glyph, foreground, cell_background) = match (top, bottom) {
                (None, None) => (" ", None, Some(background)),
                (Some(colour), None) => ("▀", Some(colour), Some(background)),
                (None, Some(colour)) => ("▄", Some(colour), Some(background)),
                (Some(top), Some(bottom)) if top == bottom => ("█", Some(top), None),
                (Some(top), Some(bottom)) => ("▀", Some(top), Some(bottom)),
            };

            let mut style = Style::new();
            if let Some(foreground) = foreground {
                style = style.fg(foreground);
            }
            if let Some(background) = cell_background {
                style = style.bg(background);
            }
            spans.push(Span::styled(glyph, style));
        }

        lines.push(Line::from(spans));
    }

    Text::from(lines)
}
