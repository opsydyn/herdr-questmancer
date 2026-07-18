use super::pixel::{PixelPoint, PixelSize, Rgb, RgbBuffer};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpriteFrame {
    size: PixelSize,
    pixels: Vec<Option<Rgb>>,
}

impl SpriteFrame {
    #[must_use]
    pub fn from_pixels(width: u16, height: u16, pixels: Vec<Option<Rgb>>) -> Self {
        let size = PixelSize::new(width, height);
        let expected_length = usize::from(width)
            .checked_mul(usize::from(height))
            .expect("sprite dimensions overflow pixel allocation");
        assert_eq!(
            pixels.len(),
            expected_length,
            "sprite pixels must match its dimensions"
        );
        Self { size, pixels }
    }

    #[must_use]
    pub const fn size(&self) -> PixelSize {
        self.size
    }

    #[must_use]
    pub fn pixels(&self) -> &[Option<Rgb>] {
        &self.pixels
    }
}

pub fn blit(frame: &SpriteFrame, origin: PixelPoint, target: &mut RgbBuffer) {
    blit_with_source_x(frame, origin, target, |x| x);
}

pub fn blit_mirrored(frame: &SpriteFrame, origin: PixelPoint, target: &mut RgbBuffer) {
    let width = frame.size.width;
    blit_with_source_x(frame, origin, target, |x| width - 1 - x);
}

fn blit_with_source_x(
    frame: &SpriteFrame,
    origin: PixelPoint,
    target: &mut RgbBuffer,
    source_x: impl Fn(u16) -> u16,
) {
    for y in 0..frame.size.height {
        for x in 0..frame.size.width {
            let pixel = frame.pixels
                [usize::from(y) * usize::from(frame.size.width) + usize::from(source_x(x))];
            if let Some(colour) = pixel {
                let Some(destination_x) = origin.x.checked_add(i32::from(x)) else {
                    continue;
                };
                let Some(destination_y) = origin.y.checked_add(i32::from(y)) else {
                    continue;
                };
                target.put(destination_x, destination_y, colour);
            }
        }
    }
}
