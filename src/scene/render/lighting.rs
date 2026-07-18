use crate::scene::pixel::{PixelPoint, Rgb, RgbBuffer};

pub fn apply_candle_light(target: &mut RgbBuffer, candle: PixelPoint) {
    let width = target.size().width;
    let height = target.size().height;
    for y in 0..height {
        for x in 0..width {
            let dx = i32::from(x).saturating_sub(candle.x).unsigned_abs();
            let dy = i32::from(y).saturating_sub(candle.y).unsigned_abs();
            let distance = dx.saturating_add(dy);
            let Some(colour) = target.get(i32::from(x), i32::from(y)) else {
                continue;
            };
            let lit = if distance < 12 {
                blend(colour, Rgb::new(255, 170, 70), 18)
            } else if distance > 70 {
                blend(colour, Rgb::BLACK, 16)
            } else {
                colour
            };
            target.put(i32::from(x), i32::from(y), lit);
        }
    }
}

fn blend(base: Rgb, tint: Rgb, amount: u16) -> Rgb {
    let keep = 100 - amount;
    Rgb::new(
        u8::try_from((u16::from(base.r) * keep + u16::from(tint.r) * amount) / 100)
            .expect("weighted red channel remains within u8"),
        u8::try_from((u16::from(base.g) * keep + u16::from(tint.g) * amount) / 100)
            .expect("weighted green channel remains within u8"),
        u8::try_from((u16::from(base.b) * keep + u16::from(tint.b) * amount) / 100)
            .expect("weighted blue channel remains within u8"),
    )
}
