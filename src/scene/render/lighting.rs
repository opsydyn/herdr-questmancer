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

pub fn apply_cool_ambient(target: &mut RgbBuffer, strength: u8) {
    let amount = u16::from(strength.min(100));
    for colour in target.pixels_mut() {
        *colour = blend(*colour, Rgb::new(18, 88, 105), amount);
    }
}

pub fn apply_cool_pool(target: &mut RgbBuffer, centre: PixelPoint, radius: u16, strength: u8) {
    apply_pool(target, centre, radius, strength, Rgb::new(34, 184, 177));
}

pub fn dim(target: &mut RgbBuffer, amount: u8) {
    for colour in target.pixels_mut() {
        *colour = blend(*colour, Rgb::BLACK, u16::from(amount.min(100)));
    }
}

pub fn apply_warm_pool(target: &mut RgbBuffer, centre: PixelPoint, radius: u16, strength: u8) {
    apply_pool(target, centre, radius, strength, Rgb::new(255, 175, 67));
}

fn apply_pool(target: &mut RgbBuffer, centre: PixelPoint, radius: u16, strength: u8, tint: Rgb) {
    let radius = u32::from(radius.max(1));
    for y in 0..target.size().height {
        for x in 0..target.size().width {
            let dx = i32::from(x).saturating_sub(centre.x).unsigned_abs();
            let dy = i32::from(y).saturating_sub(centre.y).unsigned_abs();
            let distance = dx.saturating_add(dy);
            if distance >= radius {
                continue;
            }
            let amount = u16::from(strength)
                .saturating_mul(u16::try_from(radius - distance).unwrap_or(u16::MAX))
                / u16::try_from(radius).unwrap_or(u16::MAX);
            if let Some(colour) = target.get(i32::from(x), i32::from(y)) {
                target.put(
                    i32::from(x),
                    i32::from(y),
                    blend(colour, tint, amount.min(100)),
                );
            }
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
