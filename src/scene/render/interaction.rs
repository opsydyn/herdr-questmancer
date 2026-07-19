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

pub(crate) fn paint_selection_marker(target: &mut RgbBuffer, origin: PixelPoint, size: PixelSize) {
    let right = origin.x.saturating_add(i32::from(size.width));
    let bottom = origin.y.saturating_add(i32::from(size.height));
    for (x, y) in [
        (origin.x - 1, origin.y - 1),
        (right, origin.y - 1),
        (origin.x - 1, bottom),
        (right, bottom),
    ] {
        target.put(x, y, SELECTION_RUNE);
    }
}
