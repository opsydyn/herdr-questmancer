//! The shared roster tier: the whole party at authored 8x12 scale.
//!
//! Both worlds recompose rather than crop when a party stops fitting at world
//! scale. The layout, grounding and hit regions live here so a roster
//! adventurer behaves identically in the Guild Hall and the Delve; only the
//! surface it stands on differs.

use crate::scene::{
    SceneActorRegion,
    assets::{adventurer::adventurer_roster_frame, roster},
    pixel::{PixelPoint, PixelRect, PixelSize, Rgb, RgbBuffer},
    snapshot::SceneSnapshot,
    sprite::blit,
    stage::ScenePlan,
};

use super::interaction::{paint_actor_grounding, paint_actor_state_marker, paint_selection_marker};
use super::is_visible;

/// Each master keeps a one-pixel gutter beside it so neighbouring adventurers
/// never share a silhouette edge.
pub(crate) const STRIDE_X: u16 = roster::WIDTH + 2;
/// The vertical lane carries the grounding shadow plus a full counsel marker
/// for the row below, so a blocked adventurer's marker never lands on the feet
/// of the adventurer above it.
pub(crate) const STRIDE_Y: u16 = roster::HEIGHT + 8;
pub(crate) const TOP_MARGIN: u16 = 7;
pub(crate) const MIN_WIDTH: u16 = STRIDE_X * 2;
pub(crate) const MIN_HEIGHT: u16 = TOP_MARGIN + STRIDE_Y;

pub(crate) fn columns(viewport: PixelSize) -> usize {
    usize::from(viewport.width / STRIDE_X).max(1)
}

pub(crate) fn capacity(viewport: PixelSize) -> usize {
    let rows = viewport
        .height
        .saturating_sub(TOP_MARGIN)
        .saturating_div(STRIDE_Y);
    columns(viewport) * usize::from(rows)
}

/// Centres the occupied roster rows so the party stands in the room rather
/// than clinging to the top of the pane.
pub(crate) fn block_top(viewport: PixelSize, actor_count: usize) -> i32 {
    let rows = actor_count.max(1).div_ceil(columns(viewport));
    let block_height = i32::try_from(rows).unwrap_or(i32::MAX) * i32::from(STRIDE_Y);
    let centred = (i32::from(viewport.height) - block_height) / 2;
    centred.max(i32::from(TOP_MARGIN))
}

/// Lays the roster out left-to-right, top-to-bottom in reading order so an
/// adventurer keeps its place as the party changes around it.
pub(crate) fn origin(viewport: PixelSize, index: usize, block_top: i32) -> PixelPoint {
    let columns = columns(viewport);
    let column = index % columns;
    let row = index / columns;
    let used = columns.saturating_mul(usize::from(STRIDE_X));
    let left = usize::from(viewport.width).saturating_sub(used) / 2;
    PixelPoint::new(
        i32::try_from(left + column * usize::from(STRIDE_X)).unwrap_or(i32::MAX) + 1,
        block_top.saturating_add(i32::try_from(row).unwrap_or(i32::MAX) * i32::from(STRIDE_Y)),
    )
}

/// Paints every placed adventurer at roster scale and returns their hit
/// regions. `shadow` is the surface the party grounds against.
pub(crate) fn paint_party(
    snapshot: &SceneSnapshot,
    plan: &ScenePlan,
    target: &mut RgbBuffer,
    block_top: i32,
    shadow: Rgb,
) -> Vec<SceneActorRegion> {
    let mut regions = Vec::with_capacity(plan.actors.len());
    for (index, placement) in plan.actors.iter().enumerate() {
        let Some(agent) = snapshot
            .agents
            .iter()
            .find(|agent| agent.key == placement.agent)
        else {
            continue;
        };
        let sprite = adventurer_roster_frame(&agent.persona);
        let actor_origin = origin(target.size(), index, block_top);
        let bounds = PixelRect::new(
            actor_origin.x,
            actor_origin.y,
            sprite.size().width,
            sprite.size().height,
        );
        if !is_visible(PixelPoint::new(0, 0), bounds, target.size()) {
            continue;
        }
        paint_actor_grounding(target, bounds, placement.pose, shadow);
        blit(&sprite, actor_origin, target);
        paint_actor_state_marker(target, bounds, placement.pose);
        if placement.selected {
            paint_selection_marker(target, actor_origin, sprite.size());
        }
        regions.push(SceneActorRegion {
            agent: placement.agent.clone(),
            bounds,
        });
    }
    regions
}
