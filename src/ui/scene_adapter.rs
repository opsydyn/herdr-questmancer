use ratatui::{buffer::Buffer, layout::Rect, style::Color};

use crate::{
    app::ColorMode,
    scene::pixel::{Rgb, RgbBuffer},
};

const HALF_BLOCK: &str = "▀";

pub fn flush_rgb(
    target: &mut Buffer,
    area: Rect,
    source: &RgbBuffer,
    odd_row_fill: Rgb,
    colour_mode: ColorMode,
) {
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
                    .set_fg(to_colour(top_colour, colour_mode))
                    .set_bg(to_colour(bottom_colour, colour_mode));
            }
        }
    }
}

/// The sixteen colours a base ANSI terminal can show, in ratatui's order.
///
/// Values are the widely-used xterm defaults. A terminal may theme them
/// differently, which is fine: the point is to emit an indexed colour the
/// terminal owns rather than a truecolour triple it cannot render.
const ANSI_16: [(Color, Rgb); 16] = [
    (Color::Black, Rgb::new(0, 0, 0)),
    (Color::Red, Rgb::new(170, 0, 0)),
    (Color::Green, Rgb::new(0, 170, 0)),
    (Color::Yellow, Rgb::new(170, 85, 0)),
    (Color::Blue, Rgb::new(0, 0, 170)),
    (Color::Magenta, Rgb::new(170, 0, 170)),
    (Color::Cyan, Rgb::new(0, 170, 170)),
    (Color::Gray, Rgb::new(170, 170, 170)),
    (Color::DarkGray, Rgb::new(85, 85, 85)),
    (Color::LightRed, Rgb::new(255, 85, 85)),
    (Color::LightGreen, Rgb::new(85, 255, 85)),
    (Color::LightYellow, Rgb::new(255, 255, 85)),
    (Color::LightBlue, Rgb::new(85, 85, 255)),
    (Color::LightMagenta, Rgb::new(255, 85, 255)),
    (Color::LightCyan, Rgb::new(85, 255, 255)),
    (Color::White, Rgb::new(255, 255, 255)),
];

/// Emits a scene pixel in whatever the terminal can actually show.
///
/// `ColorMode` was configurable, validated and persisted from the beginning
/// and read by nothing: every pixel went out as truecolour regardless, so the
/// `ansi16` setting was a preference the renderer never heard about. On a
/// sixteen-colour terminal that is the difference between a scene and a mess.
fn to_colour(rgb: Rgb, mode: ColorMode) -> Color {
    match mode {
        ColorMode::Xterm256 => Color::Rgb(rgb.r, rgb.g, rgb.b),
        ColorMode::Ansi16 => nearest_ansi(rgb),
    }
}

fn nearest_ansi(rgb: Rgb) -> Color {
    ANSI_16
        .iter()
        .min_by(|left, right| {
            colour_distance(rgb, left.1)
                .partial_cmp(&colour_distance(rgb, right.1))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map_or(Color::Black, |(colour, _)| *colour)
}

/// The same redmean approximation the art-direction contrast guard uses, so
/// "which colour is nearest" means the same thing everywhere in the codebase.
fn colour_distance(left: Rgb, right: Rgb) -> f64 {
    let mean_red = f64::midpoint(f64::from(left.r), f64::from(right.r));
    let dr = f64::from(left.r) - f64::from(right.r);
    let dg = f64::from(left.g) - f64::from(right.g);
    let db = f64::from(left.b) - f64::from(right.b);
    let weight_r = 2.0 + mean_red / 256.0;
    let weight_b = 2.0 + (255.0 - mean_red) / 256.0;
    (weight_r * dr * dr + 4.0 * dg * dg + weight_b * db * db).sqrt()
}
