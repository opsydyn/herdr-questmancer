use ratatui::{buffer::Buffer, layout::Rect, style::Color};

use crate::scene::pixel::{Rgb, RgbBuffer};

const HALF_BLOCK: &str = "▀";

pub fn flush_rgb(target: &mut Buffer, area: Rect, source: &RgbBuffer, odd_row_fill: Rgb) {
    let left = area.x.max(target.area.x);
    let right = area.right().min(target.area.right());
    let top = area.y.max(target.area.y);
    let bottom = area.bottom().min(target.area.bottom());

    for y in top..bottom {
        let source_y = i32::from(y - area.y) * 2;
        for x in left..right {
            let source_x = i32::from(x - area.x);
            let top_colour = source.get(source_x, source_y).unwrap_or(odd_row_fill);
            let bottom_colour = source.get(source_x, source_y + 1).unwrap_or(odd_row_fill);
            if let Some(cell) = target.cell_mut((x, y)) {
                cell.set_symbol(HALF_BLOCK)
                    .set_fg(to_colour(top_colour))
                    .set_bg(to_colour(bottom_colour));
            }
        }
    }
}

const fn to_colour(rgb: Rgb) -> Color {
    Color::Rgb(rgb.r, rgb.g, rgb.b)
}
