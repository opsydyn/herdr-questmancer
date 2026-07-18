use questmancer::scene::{
    pixel::{PixelPoint, PixelRect, Rgb, RgbBuffer},
    sprite::{SpriteFrame, blit, blit_mirrored},
};

#[test]
fn buffer_clear_and_clipped_fill_are_opaque() {
    let black = Rgb::new(0, 0, 0);
    let amber = Rgb::new(214, 139, 53);
    let mut buffer = RgbBuffer::filled(3, 2, black);
    buffer.fill_rect(PixelRect::new(-1, 1, 3, 2), amber);

    assert_eq!(buffer.get(0, 0), Some(black));
    assert_eq!(buffer.get(0, 1), Some(amber));
    assert_eq!(buffer.get(1, 1), Some(amber));
    assert_eq!(buffer.get(2, 1), Some(black));
}

#[test]
fn ensure_size_reuses_capacity_at_the_same_size() {
    let mut buffer = RgbBuffer::filled(8, 6, Rgb::BLACK);
    let capacity = buffer.capacity();
    buffer.ensure_size(8, 6, Rgb::new(1, 2, 3));
    assert_eq!(buffer.capacity(), capacity);
    assert!(
        buffer
            .pixels()
            .iter()
            .all(|pixel| *pixel == Rgb::new(1, 2, 3))
    );
}

#[test]
fn blit_preserves_transparency_when_clipped_at_negative_and_positive_edges() {
    let black = Rgb::BLACK;
    let red = Rgb::new(255, 0, 0);
    let green = Rgb::new(0, 255, 0);
    let blue = Rgb::new(0, 0, 255);
    let frame = SpriteFrame::from_pixels(2, 2, vec![Some(red), None, Some(green), Some(blue)]);
    let mut target = RgbBuffer::filled(3, 2, black);

    blit(&frame, PixelPoint::new(-1, -1), &mut target);
    assert_eq!(target.get(0, 0), Some(blue));
    assert_eq!(target.get(1, 0), Some(black));

    target.clear(black);
    blit(&frame, PixelPoint::new(2, 0), &mut target);
    assert_eq!(target.get(2, 0), Some(red));
    assert_eq!(target.get(2, 1), Some(green));

    target.clear(black);
    blit(&frame, PixelPoint::new(0, 1), &mut target);
    assert_eq!(target.get(0, 1), Some(red));
    assert_eq!(target.get(1, 1), Some(black));

    target.clear(black);
    blit(&frame, PixelPoint::new(3, 2), &mut target);
    assert!(target.pixels().iter().all(|pixel| *pixel == black));
}

#[test]
fn mirrored_blit_reverses_source_columns_without_writing_transparent_pixels() {
    let black = Rgb::BLACK;
    let red = Rgb::new(255, 0, 0);
    let blue = Rgb::new(0, 0, 255);
    let frame = SpriteFrame::from_pixels(2, 1, vec![Some(red), Some(blue)]);
    let mut target = RgbBuffer::filled(2, 1, black);

    blit_mirrored(&frame, PixelPoint::new(0, 0), &mut target);

    assert_eq!(target.get(0, 0), Some(blue));
    assert_eq!(target.get(1, 0), Some(red));
}

#[test]
fn blit_skips_unrepresentable_extreme_destination_coordinates() {
    let black = Rgb::BLACK;
    let frame = SpriteFrame::from_pixels(2, 2, vec![Some(Rgb::new(255, 0, 0)); 4]);
    let mut target = RgbBuffer::filled(2, 2, black);

    blit(&frame, PixelPoint::new(i32::MAX, i32::MAX), &mut target);
    blit(&frame, PixelPoint::new(i32::MIN, i32::MIN), &mut target);

    assert!(target.pixels().iter().all(|pixel| *pixel == black));
}

#[test]
fn mirrored_blit_skips_unrepresentable_extreme_destination_coordinates() {
    let black = Rgb::BLACK;
    let frame = SpriteFrame::from_pixels(2, 2, vec![Some(Rgb::new(255, 0, 0)); 4]);
    let mut target = RgbBuffer::filled(2, 2, black);

    blit_mirrored(&frame, PixelPoint::new(i32::MAX, i32::MAX), &mut target);
    blit_mirrored(&frame, PixelPoint::new(i32::MIN, i32::MIN), &mut target);

    assert!(target.pixels().iter().all(|pixel| *pixel == black));
}
