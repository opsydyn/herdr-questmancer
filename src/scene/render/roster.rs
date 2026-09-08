//! The shared roster tier: the whole party at authored 8x12 scale.
//!
//! Both worlds recompose rather than crop when a party stops fitting at world
//! scale. The layout, grounding and hit regions live here so a roster
//! adventurer behaves identically in the Guild Hall and the Delve; only the
//! surface it stands on differs.

use std::time::Duration;

use crate::{
    app::Motion,
    scene::{
        SceneActorRegion,
        assets::{
            adventurer::adventurer_roster_frame,
            palette::{AMBER_LIGHT, INK_BLUE, PARCHMENT_DARK, PARCHMENT_LIGHT, WINE_LIGHT},
            roster,
        },
        pixel::{PixelPoint, PixelRect, PixelSize, Rgb, RgbBuffer},
        snapshot::{SceneConnection, SceneSnapshot},
        sprite::blit,
        stage::{COMPLETION_THEATRE_MS, SceneEffect, ScenePlan},
    },
};

use super::interaction::{paint_actor_grounding, paint_selection_marker};
use super::{earliest_deadline, next_frame_delay};

/// Each master keeps a one-pixel gutter beside it so neighbouring adventurers
/// never share a silhouette edge.
pub(crate) const STRIDE_X: u16 = roster::WIDTH + 2;
/// The vertical lane carries the grounding shadow plus a full state marker
/// for the row below, so an adventurer's marker never lands on the feet
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
        if bounds.x < 0
            || bounds.y < 0
            || bounds.x + i32::from(bounds.width) > i32::from(target.size().width)
            || bounds.y + i32::from(bounds.height) > i32::from(target.size().height)
        {
            continue;
        }
        paint_actor_grounding(target, bounds, placement.pose, shadow);
        blit(&sprite, actor_origin, target);
        let marker = state_marker_bounds(bounds);
        blit(
            roster::state_marker(placement.pose),
            PixelPoint::new(marker.x, marker.y),
            target,
        );
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

/// A separate lane above the native roster master; labels must preserve it.
pub(crate) fn state_marker_bounds(actor: PixelRect) -> PixelRect {
    let size = roster::STATE_MARKER_SIZE;
    PixelRect::new(
        actor.x + i32::from(actor.width / 2) - i32::from(size / 2),
        actor.y - i32::from(size),
        size,
        size,
    )
}

/// The two reserved top pixels carry connection facts without covering a
/// party cue. World-coordinate diagnostics do not fit this recomposed tier.
pub(crate) fn paint_connection_fact(snapshot: &SceneSnapshot, target: &mut RgbBuffer) {
    match snapshot.connection {
        SceneConnection::Connected => {}
        SceneConnection::Offline => target.put(1, 0, WINE_LIGHT),
        SceneConnection::Connecting => target.put(1, 0, AMBER_LIGHT),
        SceneConnection::Reconnecting { attempt } => {
            for index in 0..attempt.clamp(1, 6) {
                target.put(1 + i32::try_from(index * 2).unwrap_or(0), 0, AMBER_LIGHT);
            }
        }
        SceneConnection::Incompatible { expected, actual } => {
            target.fill_rect(PixelRect::new(0, 0, 15, 2), PARCHMENT_DARK);
            for (count, row, colour) in [(expected, 0, PARCHMENT_LIGHT), (actual, 1, INK_BLUE)] {
                for index in 0..count.min(7) {
                    target.put(1 + i32::try_from(index * 2).unwrap_or(0), row, colour);
                }
            }
        }
    }
}

/// Fresh spoils shimmer in the side gutters without covering identity or the
/// persistent completion cue. Reduced/still scenes use that cue alone.
pub(crate) fn paint_effects(
    snapshot: &SceneSnapshot,
    plan: &ScenePlan,
    target: &mut RgbBuffer,
    actors: &[SceneActorRegion],
) -> Option<Duration> {
    if snapshot.motion != Motion::Full {
        return None;
    }
    let mut next_frame_in = None;
    let duration = Duration::from_millis(COMPLETION_THEATRE_MS.cast_unsigned());
    for effect in &plan.effects {
        let SceneEffect::FreshSpoils { agent, since } = effect else {
            continue;
        };
        let Some(region) = actors.iter().find(|region| region.agent == *agent) else {
            continue;
        };
        let elapsed = since.elapsed_until(snapshot.now);
        let Some(remaining) = duration.checked_sub(elapsed).filter(|left| !left.is_zero()) else {
            continue;
        };
        let phase = i32::try_from((elapsed.as_millis() / 125) % 4).unwrap_or(0);
        for (x, offset) in [
            (region.bounds.x - 1, phase),
            (region.bounds.x + i32::from(region.bounds.width), 3 - phase),
        ] {
            target.put(x, region.bounds.y + 1 + offset, AMBER_LIGHT);
            target.put(x, region.bounds.y + 6 + offset, PARCHMENT_LIGHT);
        }
        next_frame_in = Some(earliest_deadline(
            next_frame_in,
            next_frame_delay(elapsed, 8).min(remaining),
        ));
    }
    next_frame_in
}
