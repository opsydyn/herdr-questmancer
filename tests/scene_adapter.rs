use questmancer::{
    scene::pixel::{Rgb, RgbBuffer},
    ui::scene_adapter::flush_rgb,
};
use ratatui::{buffer::Buffer, layout::Rect, style::Color};

#[test]
fn flushes_a_two_pixel_column_as_one_half_block_cell() {
    let red = Rgb::new(255, 0, 0);
    let blue = Rgb::new(0, 0, 255);
    let source = RgbBuffer::filled(1, 2, red);
    let mut source = source;
    source.put(0, 1, blue);
    let mut target = Buffer::empty(Rect::new(0, 0, 1, 1));

    flush_rgb(&mut target, Rect::new(0, 0, 1, 1), &source, Rgb::BLACK);

    let cell = target.cell((0, 0)).unwrap();
    assert_eq!(cell.symbol(), "▀");
    assert_eq!(cell.fg, Color::Rgb(255, 0, 0));
    assert_eq!(cell.bg, Color::Rgb(0, 0, 255));
}

#[test]
fn flushes_relative_to_a_nonzero_destination_area_and_clips_to_target() {
    let red = Rgb::new(255, 0, 0);
    let blue = Rgb::new(0, 0, 255);
    let source = RgbBuffer::filled(4, 4, red);
    let mut source = source;
    source.put(1, 3, blue);
    let mut target = Buffer::empty(Rect::new(10, 20, 2, 1));

    flush_rgb(&mut target, Rect::new(9, 19, 3, 2), &source, Rgb::BLACK);

    let cell = target.cell((10, 20)).unwrap();
    assert_eq!(cell.symbol(), "▀");
    assert_eq!(cell.fg, Color::Rgb(255, 0, 0));
    assert_eq!(cell.bg, Color::Rgb(0, 0, 255));
    assert_eq!(target.cell((11, 20)).unwrap().symbol(), "▀");
}

#[test]
fn flushes_an_odd_final_pixel_row_with_the_explicit_fallback_colour() {
    let amber = Rgb::new(214, 139, 53);
    let fallback = Rgb::new(11, 22, 33);
    let source = RgbBuffer::filled(1, 3, amber);
    let mut target = Buffer::empty(Rect::new(0, 0, 1, 2));

    flush_rgb(&mut target, Rect::new(0, 0, 1, 2), &source, fallback);

    let final_cell = target.cell((0, 1)).unwrap();
    assert_eq!(final_cell.symbol(), "▀");
    assert_eq!(final_cell.fg, Color::Rgb(214, 139, 53));
    assert_eq!(final_cell.bg, Color::Rgb(11, 22, 33));
}

#[test]
fn flush_ignores_zero_sized_areas_and_source_pixels_outside_the_requested_width() {
    let red = Rgb::new(255, 0, 0);
    let blue = Rgb::new(0, 0, 255);
    let source = RgbBuffer::filled(3, 2, red);
    let mut source = source;
    source.put(2, 0, blue);
    let mut target = Buffer::empty(Rect::new(0, 0, 1, 1));

    flush_rgb(&mut target, Rect::new(0, 0, 0, 1), &source, Rgb::BLACK);
    assert_eq!(target.cell((0, 0)).unwrap().symbol(), " ");

    flush_rgb(&mut target, Rect::new(0, 0, 1, 1), &source, Rgb::BLACK);
    let cell = target.cell((0, 0)).unwrap();
    assert_eq!(cell.symbol(), "▀");
    assert_eq!(cell.fg, Color::Rgb(255, 0, 0));
    assert_eq!(cell.bg, Color::Rgb(255, 0, 0));
}
