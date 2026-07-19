pub mod assets;
pub mod pixel;
pub mod presentation;
pub mod render;
pub mod snapshot;
pub mod sprite;
pub mod stage;

use std::time::Duration;

use pixel::{PixelSize, RgbBuffer};
use presentation::ScenePresentation;
use snapshot::SceneSnapshot;
use stage::{ScenePlan, WorldScene};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneFrame {
    pub world: WorldScene,
    pub next_frame_in: Option<Duration>,
}

pub fn render_scene(
    snapshot: &SceneSnapshot,
    viewport: PixelSize,
    target: &mut RgbBuffer,
) -> SceneFrame {
    let plan = ScenePlan::project(snapshot, viewport);
    render::paint(snapshot, &plan, viewport, target)
}

pub fn render_scene_for_world(
    snapshot: &SceneSnapshot,
    presentation: &ScenePresentation,
    viewport: PixelSize,
    target: &mut RgbBuffer,
) -> SceneFrame {
    let mut plan = stage::project_for_world(snapshot, viewport, presentation.world);
    render::interaction::apply_selection(&mut plan, presentation);
    render::paint(snapshot, &plan, viewport, target)
}

#[cfg(feature = "storybook")]
pub fn render_scene_for_story(
    snapshot: &SceneSnapshot,
    world_override: Option<WorldScene>,
    viewport: PixelSize,
    target: &mut RgbBuffer,
) -> SceneFrame {
    let plan = world_override.map_or_else(
        || ScenePlan::project(snapshot, viewport),
        |world| stage::project_for_world(snapshot, viewport, world),
    );
    render::paint(snapshot, &plan, viewport, target)
}
