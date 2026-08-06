use crate::scene::{
    assets::guild_hall::{GuildHallAsset, frame},
    pixel::{PixelPoint, PixelRect, PixelSize, Rgb, RgbBuffer},
    presentation::ScenePresentation,
    sprite::blit,
    stage::{ScenePlan, ScenePose},
};

use crate::scene::assets::palette::SELECTION_RUNE;

pub(crate) fn apply_selection(plan: &mut ScenePlan, presentation: &ScenePresentation) {
    for actor in &mut plan.actors {
        actor.selected = presentation
            .selected_agent
            .as_ref()
            .is_some_and(|selected| selected == &actor.agent);
    }
}

/// Marks the selected adventurer with a rune ring on the floor beneath it.
///
/// The art direction asks for "a modest rune, lamp or floor ring under the
/// selected adventurer" and explicitly rules out four detached corner pixels:
/// at terminal scale those read as stray sparkle rather than as focus. The
/// ring scales with the master so it works at world and roster sizes alike,
/// and it slides up rather than clipping when the adventurer stands against
/// the bottom of the viewport.
pub(crate) fn paint_selection_marker(target: &mut RgbBuffer, origin: PixelPoint, size: PixelSize) {
    let last_x = i32::from(target.size().width).saturating_sub(1);
    let last_y = i32::from(target.size().height).saturating_sub(1);
    if last_x < 0 || last_y < 0 || size.width == 0 {
        return;
    }
    let width = i32::from(size.width);
    let inset = (width / 4).max(1);
    let top = origin
        .y
        .saturating_add(i32::from(size.height))
        .min(last_y - 2)
        .max(0);

    for x in (origin.x + inset)..(origin.x + width - inset) {
        let x = x.clamp(0, last_x);
        target.put(x, top, SELECTION_RUNE);
        target.put(x, top + 2, SELECTION_RUNE);
    }
    // The widest points sit one pixel outside the arcs so the ring closes;
    // any further out and it reads as two detached bars.
    for x in [origin.x + inset - 1, origin.x + width - inset] {
        target.put(x.clamp(0, last_x), top + 1, SELECTION_RUNE);
    }
}

/// Grounds an actor: a contact shadow that separates it from the surface it
/// stands on. Painted before the sprite so the adventurer stands on it.
pub(crate) fn paint_actor_grounding(
    target: &mut RgbBuffer,
    bounds: PixelRect,
    _pose: ScenePose,
    shadow: Rgb,
) {
    let shadow_width = bounds.width.saturating_sub(4);
    let shadow_x = bounds.x + 2;
    let shadow_y = bounds.y + i32::from(bounds.height).saturating_sub(2);
    target.fill_rect(PixelRect::new(shadow_x, shadow_y, shadow_width, 2), shadow);
}

/// Marks an actor's state *after* its sprite is drawn.
///
/// The counsel marker has to sit on top. An adventurer flush against the top
/// of a compact pane has its marker clamped into its own footprint, and when
/// the marker was painted first the sprite simply covered it — the highest
/// priority state in the room silently disappeared.
pub(crate) fn paint_actor_state_marker(target: &mut RgbBuffer, bounds: PixelRect, pose: ScenePose) {
    if pose == ScenePose::SeekingCounsel {
        paint_counsel_marker(target, bounds);
    }
}

/// Places the authored counsel marker above an adventurer's head, nudged into
/// the viewport when the actor stands at an edge so the highest-priority state
/// in the room is never the one that gets clipped.
pub(crate) fn paint_counsel_marker(target: &mut RgbBuffer, bounds: PixelRect) {
    let marker = frame(GuildHallAsset::CounselMarker);
    let width = i32::from(marker.size().width);
    let height = i32::from(marker.size().height);
    let last_x = i32::from(target.size().width).saturating_sub(width);
    let last_y = i32::from(target.size().height).saturating_sub(height);
    if last_x < 0 || last_y < 0 {
        return;
    }
    // Sits directly on the adventurer's head. A marker floating a few pixels
    // clear reads as room decoration rather than as this adventurer's state.
    let x = (bounds.x + i32::from(bounds.width) / 2 - width / 2).clamp(0, last_x);
    let y = (bounds.y - height + 1).clamp(0, last_y);
    blit(marker, PixelPoint::new(x, y), target);
}
