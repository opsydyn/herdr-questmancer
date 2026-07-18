use proptest::prelude::*;
use questmancer::scene::{
    pixel::{PixelPoint, PixelRect, Rgb, RgbBuffer},
    sprite::{SpriteFrame, blit, blit_mirrored},
};

fn rgb_strategy() -> impl Strategy<Value = Rgb> {
    (any::<u8>(), any::<u8>(), any::<u8>()).prop_map(|(r, g, b)| Rgb::new(r, g, b))
}

fn sprite_pixel_strategy() -> impl Strategy<Value = Option<Rgb>> {
    (any::<bool>(), rgb_strategy()).prop_map(|(opaque, colour)| opaque.then_some(colour))
}

proptest! {
    #[test]
    fn blits_never_resize_or_change_uncovered_destination_pixels(
        target_width in 0_u16..128,
        target_height in 0_u16..128,
        frame_width in 0_u16..40,
        frame_height in 0_u16..40,
        origin_x in -80_i32..160,
        origin_y in -80_i32..160,
        mirrored in any::<bool>(),
        background in rgb_strategy(),
        source_pixels in proptest::collection::vec(sprite_pixel_strategy(), 0..1600),
    ) {
        let frame_length = usize::from(frame_width) * usize::from(frame_height);
        let pixels = source_pixels
            .into_iter()
            .chain(std::iter::repeat(None))
            .take(frame_length)
            .collect::<Vec<_>>();
        let frame = SpriteFrame::from_pixels(frame_width, frame_height, pixels.clone());
        let mut target = RgbBuffer::filled(target_width, target_height, background);
        let size_before = target.size();
        let capacity_before = target.capacity();
        let mut expected = target.pixels().to_vec();

        for source_y in 0..frame_height {
            for destination_x in 0..frame_width {
                let source_x = if mirrored {
                    frame_width.saturating_sub(1).saturating_sub(destination_x)
                } else {
                    destination_x
                };
                let source_index = usize::from(source_y) * usize::from(frame_width) + usize::from(source_x);
                let destination_x = origin_x + i32::from(destination_x);
                let destination_y = origin_y + i32::from(source_y);
                if let Some(colour) = pixels[source_index]
                    && destination_x >= 0
                    && destination_y >= 0
                    && destination_x < i32::from(target_width)
                    && destination_y < i32::from(target_height)
                {
                    let destination_index = usize::try_from(destination_y).unwrap() * usize::from(target_width)
                        + usize::try_from(destination_x).unwrap();
                    expected[destination_index] = colour;
                }
            }
        }

        if mirrored {
            blit_mirrored(&frame, PixelPoint::new(origin_x, origin_y), &mut target);
        } else {
            blit(&frame, PixelPoint::new(origin_x, origin_y), &mut target);
        }

        prop_assert_eq!(target.size(), size_before);
        prop_assert_eq!(target.capacity(), capacity_before);
        prop_assert_eq!(target.pixels(), expected.as_slice());
    }

    #[test]
    fn clipped_fills_never_resize_or_change_pixels_outside_the_rectangle(
        target_width in 0_u16..128,
        target_height in 0_u16..128,
        rect_width in 0_u16..40,
        rect_height in 0_u16..40,
        rect_x in -80_i32..160,
        rect_y in -80_i32..160,
        background in rgb_strategy(),
        fill in rgb_strategy(),
    ) {
        let mut target = RgbBuffer::filled(target_width, target_height, background);
        let size_before = target.size();
        let capacity_before = target.capacity();
        let mut expected = target.pixels().to_vec();

        for y in 0..target_height {
            for x in 0..target_width {
                let x = i32::from(x);
                let y = i32::from(y);
                if x >= rect_x
                    && x < rect_x.saturating_add(i32::from(rect_width))
                    && y >= rect_y
                    && y < rect_y.saturating_add(i32::from(rect_height))
                {
                    expected[usize::try_from(y).unwrap() * usize::from(target_width)
                        + usize::try_from(x).unwrap()] = fill;
                }
            }
        }

        target.fill_rect(PixelRect::new(rect_x, rect_y, rect_width, rect_height), fill);

        prop_assert_eq!(target.size(), size_before);
        prop_assert_eq!(target.capacity(), capacity_before);
        prop_assert_eq!(target.pixels(), expected.as_slice());
    }
}
