pub mod assets;
pub mod pixel;
pub mod presentation;
pub mod render;
pub mod snapshot;
pub mod sprite;
pub mod stage;

use std::time::Duration;

use crate::domain::AgentKey;
use pixel::{PixelRect, PixelSize, RgbBuffer};
use presentation::ScenePresentation;
use snapshot::SceneSnapshot;
use stage::{ScenePlan, WorldScene};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SceneFrame {
    pub world: WorldScene,
    pub next_frame_in: Option<Duration>,
    pub actors: Vec<SceneActorRegion>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SceneActorRegion {
    pub agent: AgentKey,
    /// Bounds in the RGB scene's pixel coordinates.
    pub bounds: PixelRect,
}

impl SceneFrame {
    #[must_use]
    pub fn agent_at(&self, column: u16, row: u16) -> Option<&AgentKey> {
        let x = i32::from(column);
        let top = i32::from(row) * 2;
        let bottom = top + 1;
        self.actors
            .iter()
            .rev()
            .find(|region| contains(region.bounds, x, top) || contains(region.bounds, x, bottom))
            .map(|region| &region.agent)
    }
}

fn contains(rect: PixelRect, x: i32, y: i32) -> bool {
    x >= rect.x
        && x < rect.x + i32::from(rect.width)
        && y >= rect.y
        && y < rect.y + i32::from(rect.height)
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
