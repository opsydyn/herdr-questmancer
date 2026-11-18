use std::time::Duration;

use crate::{
    app::Motion,
    scene::{
        pixel::{PixelPoint, PixelRect, Rgb, RgbBuffer},
        render::delve::DelveArchitectureMask,
    },
};

pub(crate) const WALK_STEP: Duration = Duration::from_millis(125);
pub(crate) const WALK_DURATION: Duration = Duration::from_millis(1_500);
#[cfg(any(feature = "storybook", test))]
const FEET: [(i32, i32); 13] = [
    (38, 73),
    (42, 73),
    (46, 73),
    (50, 73),
    (54, 73),
    (58, 73),
    (62, 73),
    (66, 73),
    (70, 73),
    (74, 73),
    (78, 71),
    (81, 66),
    (81, 62),
];

#[cfg(any(feature = "storybook", test))]
pub(in crate::scene) fn delve_walk(
    elapsed: Duration,
    motion: Motion,
) -> (PixelPoint, Option<usize>) {
    let ticks = usize::try_from(elapsed.as_millis() / WALK_STEP.as_millis()).unwrap_or(usize::MAX);
    let last = FEET.len() - 1;
    let frame = if motion == Motion::Full {
        ticks.min(last)
    } else {
        last
    };
    let (x, y) = FEET[frame];
    (
        PixelPoint::new(x - 8, y - 21),
        (frame < last).then_some(frame % 2),
    )
}

pub(in crate::scene) fn next_walk_frame(elapsed: Duration, motion: Motion) -> Option<Duration> {
    if motion != Motion::Full || elapsed >= WALK_DURATION {
        return None;
    }
    let tick =
        u32::try_from(elapsed.as_millis() / WALK_STEP.as_millis() + 1).expect("bounded walk tick");
    Some(WALK_STEP * tick - elapsed)
}

pub(super) fn work_approach(
    seat: PixelPoint,
    seats: &[PixelPoint],
    mask: &DelveArchitectureMask,
    elapsed: Duration,
) -> Option<(PixelPoint, usize, Duration)> {
    const TRAVEL: u16 = 11;
    let deadline = next_walk_frame(elapsed, Motion::Full)?;
    let (width, height) = (16, 24);
    let step_count = i32::try_from(WALK_DURATION.as_millis() / WALK_STEP.as_millis()).ok()?;
    let step = i32::try_from(elapsed.as_millis() / WALK_STEP.as_millis()).ok()?;
    let travel = i32::from(TRAVEL);
    for offset in [travel, -travel] {
        let start = seat.x + offset;
        let left = start.min(seat.x);
        let right = start.max(seat.x);
        if left < 0 || right + i32::from(width) > super::delve::WIDTH {
            continue;
        }
        let shared_lane = PixelRect::new(left, seat.y, width + TRAVEL, height);
        let clear = seats.iter().filter(|other| **other != seat).all(|other| {
            // Reserve both of each neighbour's possible approaches, including
            // their cue space, so simultaneous moves cannot move the lane.
            let occupied = PixelRect::new(
                other.x - travel,
                other.y - 6,
                width + 2 * TRAVEL,
                height + 8,
            );
            !super::party::intersects(shared_lane, occupied)
        });
        if clear && (left..=right).all(|x| mask.is_walkable(x + 8, seat.y + 21)) {
            let remaining = (travel * (step_count - step) + step_count - 1) / step_count;
            let origin = PixelPoint::new(seat.x + remaining * offset.signum(), seat.y);
            return Some((origin, usize::from(step % 2 != 0), deadline));
        }
    }
    None
}

pub(super) fn delve_stone(
    target: &mut RgbBuffer,
    origin: PixelPoint,
    mask: &DelveArchitectureMask,
) {
    for y in origin.y.max(0)..(origin.y + super::delve::HEIGHT).min(i32::from(target.size().height))
    {
        for x in
            origin.x.max(0)..(origin.x + super::delve::WIDTH).min(i32::from(target.size().width))
        {
            let wx = x - origin.x;
            let wy = y - origin.y;
            if !mask.is_walkable(wx, wy) {
                continue;
            }
            let course = wy.div_euclid(7);
            let stagger = wx + 5 * course.rem_euclid(2);
            let tx = stagger.rem_euclid(11);
            let ty = wy.rem_euclid(7);
            let variation = (course * 7 + stagger.div_euclid(11) * 5).rem_euclid(4);
            let edge = !mask.is_walkable(wx - 1, wy)
                || !mask.is_walkable(wx, wy - 1)
                || !mask.is_walkable(wx, wy + 1);
            let colour = if edge {
                Rgb::new(30, 50, 43)
            } else if tx == 0 || ty == 0 {
                Rgb::new(24, 41, 44)
            } else if ty == 1 && (2..=8).contains(&tx) && variation != 2 {
                Rgb::new(53, 72, 72)
            } else if variation == 2 && (7..=9).contains(&tx) && ty >= 4 {
                Rgb::new(36, 62, 49)
            } else {
                let shade = [0, 3, 7, 10][usize::try_from(variation).expect("four tile variants")];
                Rgb::new(39 + shade, 55 + shade, 57 + shade)
            };
            target.put(x, y, colour);
        }
    }
}

pub(super) fn contact_shadow(target: &mut RgbBuffer, bounds: PixelRect) {
    let y = (bounds.y + i32::from(bounds.height)).min(89);
    let span = bounds.width.saturating_sub(4);
    target.fill_rect(
        PixelRect::new(bounds.x + 2, y, span, 1),
        Rgb::new(24, 42, 43),
    );
    target.fill_rect(
        PixelRect::new(bounds.x + 4, y, span.saturating_sub(4), 1),
        Rgb::new(19, 30, 32),
    );
}

pub(super) fn hall_materials(target: &mut RgbBuffer, origin: PixelPoint) {
    use crate::scene::assets::palette::{OAK, OAK_LIGHT, RUG, STONE, STONE_LIGHT};
    for y in
        origin.y.max(0)..(origin.y + super::guild_hall::HEIGHT).min(i32::from(target.size().height))
    {
        for x in origin.x.max(0)
            ..(origin.x + super::guild_hall::WIDTH).min(i32::from(target.size().width))
        {
            let (wx, wy) = (x - origin.x, y - origin.y);
            let Some(base) = target.get(x, y) else {
                continue;
            };
            let plank = (wy - 50).div_euclid(7);
            let grain = (wx + 12 * plank.rem_euclid(2)).rem_euclid(24);
            let colour = match base {
                OAK | OAK_LIGHT
                    if wy >= 50 && (wy - 50).rem_euclid(7) == 3 && (3..=15).contains(&grain) =>
                {
                    Rgb::new(100, 57, 36)
                }
                RUG if (36..=115).contains(&wx)
                    && (72..=83).contains(&wy)
                    && (wx.div_euclid(7) + wy).rem_euclid(7) == 0 =>
                {
                    Rgb::new(123, 56, 48)
                }
                STONE | STONE_LIGHT
                    if (132..=156).contains(&wx)
                        && (10..=33).contains(&wy)
                        && (wx - 144).abs() <= (wy - 10) / 2 =>
                {
                    Rgb::new(63, 47, 43)
                }
                _ => continue,
            };
            target.put(x, y, colour);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feet_follow_walkable_floor_and_reduced_motion_settles() {
        for (index, &(x, y)) in FEET.iter().enumerate() {
            assert!(
                super::super::delve::is_walkable(x, y),
                "step {index} is outside the floor"
            );
        }
        let start = delve_walk(Duration::ZERO, Motion::Full);
        let mid = delve_walk(3 * WALK_STEP, Motion::Full);
        let finish = delve_walk(Duration::ZERO, Motion::Reduced);
        assert_ne!(start.0, mid.0);
        assert_ne!(start.1, mid.1);
        assert_eq!(finish, delve_walk(Duration::from_secs(3), Motion::Full));
        assert_eq!(finish, delve_walk(Duration::ZERO, Motion::None));
    }
}
