pub mod delve;
pub mod guild_hall;
pub mod lighting;

use std::time::Duration;

use crate::app::Motion;
use crate::scene::{
    SceneFrame,
    pixel::{PixelPoint, PixelRect, PixelSize, RgbBuffer},
    snapshot::SceneSnapshot,
    sprite::SpriteFrame,
    stage::{ScenePlan, ScenePose},
};

pub fn paint(
    snapshot: &SceneSnapshot,
    plan: &ScenePlan,
    viewport: PixelSize,
    target: &mut RgbBuffer,
) -> SceneFrame {
    match plan.world {
        crate::scene::stage::WorldScene::GuildHall => {
            guild_hall::paint(snapshot, plan, viewport, target)
        }
        crate::scene::stage::WorldScene::Delve => delve::paint(snapshot, plan, viewport, target),
    }
}

pub(crate) fn next_frame_delay(elapsed: Duration, fps: u8) -> Duration {
    let fps = u128::from(fps.max(1));
    let elapsed_millis = elapsed.as_millis();
    let completed_steps = elapsed_millis.saturating_mul(fps) / 1_000;
    let next_boundary = completed_steps
        .saturating_add(1)
        .saturating_mul(1_000)
        .div_ceil(fps);
    let delay = next_boundary.saturating_sub(elapsed_millis).max(1);
    Duration::from_millis(u64::try_from(delay).unwrap_or(u64::MAX))
}

pub(crate) fn earliest_deadline(current: Option<Duration>, candidate: Duration) -> Duration {
    current.map_or(candidate, |current| current.min(candidate))
}

pub(crate) const fn actor_animation_fps(motion: Motion, pose: ScenePose) -> Option<u8> {
    match (motion, pose) {
        (Motion::Full, ScenePose::ReturningWithSpoils) => Some(8),
        (Motion::Full, ScenePose::Working) => Some(6),
        (Motion::Full, ScenePose::SeekingCounsel) => Some(2),
        (Motion::Full | Motion::Reduced, ScenePose::Resting) => Some(1),
        _ => None,
    }
}

pub(crate) fn actor_animation_phase(motion: Motion, pose: ScenePose, elapsed: Duration) -> u8 {
    let Some(fps) = actor_animation_fps(motion, pose) else {
        return 0;
    };
    let steps = elapsed.as_millis().saturating_mul(u128::from(fps)) / 1_000;
    u8::try_from(steps % 3).unwrap_or(0)
}

pub(crate) fn effect_animation_phase(motion: Motion, elapsed: Duration) -> u128 {
    if motion == Motion::Full {
        elapsed.as_millis() / 125
    } else {
        0
    }
}

pub(crate) fn is_visible(origin: PixelPoint, world_bounds: PixelRect, viewport: PixelSize) -> bool {
    let left = origin.x.saturating_add(world_bounds.x);
    let top = origin.y.saturating_add(world_bounds.y);
    let right = left.saturating_add(i32::from(world_bounds.width));
    let bottom = top.saturating_add(i32::from(world_bounds.height));
    left < i32::from(viewport.width) && right > 0 && top < i32::from(viewport.height) && bottom > 0
}

pub(crate) fn painted_sprite_is_visible(
    frame: &SpriteFrame,
    origin: PixelPoint,
    viewport: PixelSize,
) -> bool {
    let width = usize::from(frame.size().width);
    frame.pixels().iter().enumerate().any(|(index, pixel)| {
        if pixel.is_none() {
            return false;
        }
        let x = i32::try_from(index % width).unwrap_or(i32::MAX);
        let y = i32::try_from(index / width).unwrap_or(i32::MAX);
        let x = origin.x.saturating_add(x);
        let y = origin.y.saturating_add(y);
        x >= 0 && y >= 0 && x < i32::from(viewport.width) && y < i32::from(viewport.height)
    })
}
