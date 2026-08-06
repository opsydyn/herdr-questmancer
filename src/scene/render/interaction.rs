use crate::scene::{
    pixel::{PixelPoint, PixelSize, RgbBuffer},
    presentation::ScenePresentation,
    stage::ScenePlan,
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
