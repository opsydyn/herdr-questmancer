use herdr_webmaster::ui::pixel::{Canvas, ColorRole, Palette, pack};
use ratatui::style::Color;

#[test]
fn new_canvas_is_transparent() {
    let canvas = Canvas::new(3, 2);

    assert_eq!(canvas.pixels(), &[None; 6]);
}

#[test]
fn set_writes_in_bounds_pixels_in_row_major_order() {
    let mut canvas = Canvas::new(3, 2);

    canvas.set(1, 0, ColorRole::Accent);
    canvas.set(2, 1, ColorRole::Shadow);

    assert_eq!(
        canvas.pixels(),
        &[
            None,
            Some(ColorRole::Accent),
            None,
            None,
            None,
            Some(ColorRole::Shadow),
        ]
    );
}

#[test]
fn set_ignores_out_of_bounds_pixels() {
    let mut canvas = Canvas::new(2, 2);

    canvas.set(2, 0, ColorRole::Accent);
    canvas.set(0, 2, ColorRole::Accent);
    canvas.set(u16::MAX, u16::MAX, ColorRole::Accent);

    assert_eq!(canvas.pixels(), &[None; 4]);
}

#[test]
fn fill_rect_clips_to_the_canvas() {
    let mut canvas = Canvas::new(4, 3);

    canvas.fill_rect(2, 1, 4, 3, ColorRole::Highlight);

    assert_eq!(
        canvas.pixels(),
        &[
            None,
            None,
            None,
            None,
            None,
            None,
            Some(ColorRole::Highlight),
            Some(ColorRole::Highlight),
            None,
            None,
            Some(ColorRole::Highlight),
            Some(ColorRole::Highlight),
        ]
    );
}

#[test]
fn pack_uses_a_background_coloured_space_for_two_empty_pixels() {
    let canvas = Canvas::new(1, 2);

    let text = pack(&canvas, &Palette::Xterm256, ColorRole::PanelBackground);
    let span = &text.lines[0].spans[0];

    assert_eq!(span.content, " ");
    assert_eq!(span.style.fg, None);
    assert_eq!(span.style.bg, Some(Color::Indexed(234)));
}

#[test]
fn pack_uses_an_upper_half_block_for_a_top_pixel() {
    let mut canvas = Canvas::new(1, 2);
    canvas.set(0, 0, ColorRole::Accent);

    let text = pack(&canvas, &Palette::Xterm256, ColorRole::PanelBackground);
    let span = &text.lines[0].spans[0];

    assert_eq!(span.content, "▀");
    assert_eq!(span.style.fg, Some(Color::Indexed(48)));
    assert_eq!(span.style.bg, Some(Color::Indexed(234)));
}

#[test]
fn pack_uses_a_lower_half_block_for_a_bottom_pixel() {
    let mut canvas = Canvas::new(1, 2);
    canvas.set(0, 1, ColorRole::Accent);

    let text = pack(&canvas, &Palette::Ansi16, ColorRole::PanelBackground);
    let span = &text.lines[0].spans[0];

    assert_eq!(span.content, "▄");
    assert_eq!(span.style.fg, Some(Color::LightCyan));
    assert_eq!(span.style.bg, Some(Color::Black));
}

#[test]
fn pack_uses_a_full_block_for_matching_pixels() {
    let mut canvas = Canvas::new(1, 2);
    canvas.fill_rect(0, 0, 1, 2, ColorRole::Accent);

    let text = pack(&canvas, &Palette::Xterm256, ColorRole::PanelBackground);
    let span = &text.lines[0].spans[0];

    assert_eq!(span.content, "█");
    assert_eq!(span.style.fg, Some(Color::Indexed(48)));
    assert_eq!(span.style.bg, Some(Color::Indexed(234)));
}

#[test]
fn pack_uses_foreground_and_background_for_distinct_pixels() {
    let mut canvas = Canvas::new(1, 2);
    canvas.set(0, 0, ColorRole::Accent);
    canvas.set(0, 1, ColorRole::Shadow);

    let text = pack(&canvas, &Palette::Xterm256, ColorRole::PanelBackground);
    let span = &text.lines[0].spans[0];

    assert_eq!(span.content, "▀");
    assert_eq!(span.style.fg, Some(Color::Indexed(48)));
    assert_eq!(span.style.bg, Some(Color::Indexed(236)));
}
