pub mod delve;
pub mod guild_hall;
pub mod lighting;

use std::time::Duration;

use crate::app::Motion;
use crate::scene::{
    SceneFrame,
    pixel::{PixelPoint, PixelRect, PixelSize, RgbBuffer},
    snapshot::SceneSnapshot,
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

pub(crate) fn fps_period(fps: u8) -> Duration {
    Duration::from_millis((1_000 / u64::from(fps.max(1))).max(125))
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
    let period_ms = fps_period(fps).as_millis();
    u8::try_from((elapsed.as_millis() / period_ms) % 3).unwrap_or(0)
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
