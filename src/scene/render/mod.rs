pub mod delve;
pub mod guild_hall;
pub(crate) mod interaction;
pub mod lighting;
pub(crate) mod roster;

use std::time::Duration;

use crate::app::Motion;
use crate::scene::{
    SceneFrame,
    assets::adventurer::{AdventurerFrame, adventurer_at},
    pixel::{PixelPoint, PixelRect, PixelSize, RgbBuffer},
    snapshot::{SceneAgent, SceneConnection, SceneSnapshot},
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

/// Select the same authored pose and age in every world. Rendering retained
/// blocked/done facts after a socket boundary cannot restart their gesture.
pub(crate) fn actor_frame(
    snapshot: &SceneSnapshot,
    plan: &ScenePlan,
    agent: &SceneAgent,
    pose: ScenePose,
) -> AdventurerFrame {
    let since = if pose == ScenePose::ReturningWithSpoils {
        agent
            .transition
            .map_or(agent.presence_since, |transition| transition.since)
    } else {
        agent.presence_since
    };
    let may_animate = snapshot.connection == SceneConnection::Connected
        && since <= snapshot.now
        && (pose == ScenePose::Working || plan.transition_floor.is_none_or(|floor| since > floor));
    adventurer_at(
        &agent.persona,
        pose,
        snapshot.motion,
        may_animate.then(|| since.elapsed_until(snapshot.now)),
    )
}

/// A cropped static head must not wake to animate hands outside the viewport.
pub(crate) fn actor_next_frame_delay(
    sample: &AdventurerFrame,
    origin: PixelPoint,
    viewport: PixelSize,
) -> Option<Duration> {
    let (delay, next) = sample.next.as_ref()?;
    let width = usize::from(sample.sprite.size().width);
    let visible_change = sample
        .sprite
        .pixels()
        .iter()
        .zip(next.pixels())
        .enumerate()
        .any(|(index, (current, next))| {
            let x = origin.x + i32::try_from(index % width).unwrap_or(i32::MAX);
            let y = origin.y + i32::try_from(index / width).unwrap_or(i32::MAX);
            current != next
                && x >= 0
                && y >= 0
                && x < i32::from(viewport.width)
                && y < i32::from(viewport.height)
        });
    visible_change.then_some(*delay)
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

#[cfg(test)]
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

/// Places authored masters on the scene's shared visual foot line. Scene
/// anchors describe a station, not a particular sprite's top-left corner.
pub(crate) fn actor_origin(
    world_origin: PixelPoint,
    anchor: PixelPoint,
    frame: &SpriteFrame,
) -> PixelPoint {
    let width_offset = i32::from(frame.size().width.saturating_sub(8)) / 2;
    let height_offset = i32::from(frame.size().height.saturating_sub(14));
    PixelPoint::new(
        world_origin
            .x
            .saturating_add(anchor.x)
            .saturating_sub(width_offset),
        world_origin
            .y
            .saturating_add(anchor.y)
            .saturating_sub(height_offset),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authored_masters_share_the_station_foot_line() {
        let station_reference = SpriteFrame::from_pixels(8, 14, vec![None; 8 * 14]);
        let master = SpriteFrame::from_pixels(16, 24, vec![None; 16 * 24]);
        let origin = PixelPoint::new(0, 0);
        let anchor = PixelPoint::new(112, 69);

        let reference_origin = actor_origin(origin, anchor, &station_reference);
        let master_origin = actor_origin(origin, anchor, &master);

        assert_eq!(
            reference_origin.y + i32::from(station_reference.size().height),
            83
        );
        assert_eq!(master_origin.y + i32::from(master.size().height), 83);
        assert_eq!(master_origin.x, 108);
    }
}
