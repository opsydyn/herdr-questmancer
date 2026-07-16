use questmancer::ui::pixel::{Canvas, ColorRole, Palette, pack};
use ratatui::style::Color;

#[test]
fn new_canvas_is_transparent() {
    let canvas = Canvas::new(3, 2);

    assert_eq!(canvas.pixels(), &[None; 6]);
}

#[test]
fn set_writes_in_bounds_pixels_in_row_major_order() {
    let mut canvas = Canvas::new(3, 2);

    canvas.set(1, 0, ColorRole::RuneGlow);
    canvas.set(2, 1, ColorRole::Ink);

    assert_eq!(
        canvas.pixels(),
        &[
            None,
            Some(ColorRole::RuneGlow),
            None,
            None,
            None,
            Some(ColorRole::Ink),
        ]
    );
}

#[test]
fn set_ignores_out_of_bounds_pixels() {
    let mut canvas = Canvas::new(2, 2);

    canvas.set(2, 0, ColorRole::RuneGlow);
    canvas.set(0, 2, ColorRole::RuneGlow);
    canvas.set(u16::MAX, u16::MAX, ColorRole::RuneGlow);

    assert_eq!(canvas.pixels(), &[None; 4]);
}

#[test]
fn fill_rect_clips_to_the_canvas() {
    let mut canvas = Canvas::new(4, 3);

    canvas.fill_rect(2, 1, 4, 3, ColorRole::Parchment);

    assert_eq!(
        canvas.pixels(),
        &[
            None,
            None,
            None,
            None,
            None,
            None,
            Some(ColorRole::Parchment),
            Some(ColorRole::Parchment),
            None,
            None,
            Some(ColorRole::Parchment),
            Some(ColorRole::Parchment),
        ]
    );
}

#[test]
fn pack_uses_a_background_coloured_space_for_two_empty_pixels() {
    let canvas = Canvas::new(1, 2);

    let text = pack(&canvas, &Palette::Xterm256, ColorRole::DarkStone);
    let span = &text.lines[0].spans[0];

    assert_eq!(span.content, " ");
    assert_eq!(span.style.fg, None);
    assert_eq!(span.style.bg, Some(Color::Indexed(234)));
}

#[test]
fn pack_uses_an_upper_half_block_for_a_top_pixel() {
    let mut canvas = Canvas::new(1, 2);
    canvas.set(0, 0, ColorRole::RuneGlow);

    let text = pack(&canvas, &Palette::Xterm256, ColorRole::DarkStone);
    let span = &text.lines[0].spans[0];

    assert_eq!(span.content, "▀");
    assert_eq!(span.style.fg, Some(Color::Indexed(81)));
    assert_eq!(span.style.bg, Some(Color::Indexed(234)));
}

#[test]
fn pack_uses_a_lower_half_block_for_a_bottom_pixel() {
    let mut canvas = Canvas::new(1, 2);
    canvas.set(0, 1, ColorRole::RuneGlow);

    let text = pack(&canvas, &Palette::Ansi16, ColorRole::DarkStone);
    let span = &text.lines[0].spans[0];

    assert_eq!(span.content, "▄");
    assert_eq!(span.style.fg, Some(Color::LightCyan));
    assert_eq!(span.style.bg, Some(Color::Black));
}

#[test]
fn pack_uses_a_full_block_for_matching_pixels() {
    let mut canvas = Canvas::new(1, 2);
    canvas.fill_rect(0, 0, 1, 2, ColorRole::RuneGlow);

    let text = pack(&canvas, &Palette::Xterm256, ColorRole::DarkStone);
    let span = &text.lines[0].spans[0];

    assert_eq!(span.content, "█");
    assert_eq!(span.style.fg, Some(Color::Indexed(81)));
    assert_eq!(span.style.bg, None);
}

#[test]
fn pack_uses_a_full_block_when_distinct_roles_resolve_to_the_same_colour() {
    let mut canvas = Canvas::new(1, 2);
    canvas.set(0, 0, ColorRole::Stone);
    canvas.set(0, 1, ColorRole::Steel);

    let text = pack(&canvas, &Palette::Ansi16, ColorRole::DarkStone);
    let span = &text.lines[0].spans[0];

    assert_eq!(span.content, "█");
    assert_eq!(span.style.fg, Some(Color::Gray));
    assert_eq!(span.style.bg, None);
}

#[test]
fn pack_uses_foreground_and_background_for_distinct_pixels() {
    let mut canvas = Canvas::new(1, 2);
    canvas.set(0, 0, ColorRole::RuneGlow);
    canvas.set(0, 1, ColorRole::Ink);

    let text = pack(&canvas, &Palette::Xterm256, ColorRole::DarkStone);
    let span = &text.lines[0].spans[0];

    assert_eq!(span.content, "▀");
    assert_eq!(span.style.fg, Some(Color::Indexed(81)));
    assert_eq!(span.style.bg, Some(Color::Indexed(233)));
}

#[test]
fn ansi_palette_keeps_steel_and_ink_distinct() {
    let mut canvas = Canvas::new(1, 2);
    canvas.set(0, 0, ColorRole::Steel);
    canvas.set(0, 1, ColorRole::Ink);

    let text = pack(&canvas, &Palette::Ansi16, ColorRole::DarkStone);
    let span = &text.lines[0].spans[0];

    assert_eq!(span.content, "▀");
    assert_eq!(span.style.fg, Some(Color::Gray));
    assert_eq!(span.style.bg, Some(Color::DarkGray));
}
