pub mod assets;
pub mod fit;
pub mod heraldry;
pub mod pixel;
pub mod presentation;
pub mod render;
pub mod snapshot;
pub mod sprite;
pub mod stage;
pub mod war_table;

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
    pub interactables: Vec<SceneInteractableRegion>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SceneActorRegion {
    pub agent: AgentKey,
    /// Bounds in the RGB scene's pixel coordinates.
    pub bounds: PixelRect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SceneInteractable {
    Librarian,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SceneInteractableRegion {
    pub kind: SceneInteractable,
    /// Bounds in the RGB scene's pixel coordinates.
    pub bounds: PixelRect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SceneTarget<'a> {
    Agent(&'a AgentKey),
    Interactable(SceneInteractable),
}

impl SceneFrame {
    #[must_use]
    pub fn agent_at(&self, column: u16, row: u16) -> Option<&AgentKey> {
        match self.target_at(column, row) {
            Some(SceneTarget::Agent(agent)) => Some(agent),
            Some(SceneTarget::Interactable(_)) | None => None,
        }
    }

    #[must_use]
    pub fn interactable_at(&self, column: u16, row: u16) -> Option<SceneInteractable> {
        match self.target_at(column, row) {
            Some(SceneTarget::Interactable(kind)) => Some(kind),
            Some(SceneTarget::Agent(_)) | None => None,
        }
    }

    #[must_use]
    pub fn target_at(&self, column: u16, row: u16) -> Option<SceneTarget<'_>> {
        let x = i32::from(column);
        let top = i32::from(row) * 2;
        let bottom = top + 1;
        if let Some(agent) = self
            .actors
            .iter()
            .rev()
            .find(|region| contains(region.bounds, x, top) || contains(region.bounds, x, bottom))
            .map(|region| &region.agent)
        {
            return Some(SceneTarget::Agent(agent));
        }
        self.interactables
            .iter()
            .rev()
            .find(|region| contains(region.bounds, x, top) || contains(region.bounds, x, bottom))
            .map(|region| SceneTarget::Interactable(region.kind))
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
    let plan = presented_plan(snapshot, presentation, viewport);
    let mut frame = render::paint(snapshot, &plan, viewport, target);
    if presentation.overlay == presentation::SceneOverlay::WarTable {
        let table = war_table::TablePlan::project(presentation.war_table.as_ref(), viewport);
        render::war_table::paint(presentation.war_table.as_ref(), &table, target);
        frame.actors.clear();
        frame.interactables.clear();
        frame.next_frame_in = None;
    }
    frame
}

fn presented_plan(
    snapshot: &SceneSnapshot,
    presentation: &ScenePresentation,
    viewport: PixelSize,
) -> ScenePlan {
    let mut plan = stage::project_for_world(snapshot, viewport, presentation.world);
    plan.suppress_transitions_through(snapshot, presentation.transition_floor);
    render::interaction::apply_selection(&mut plan, presentation);
    plan.goblin_outbreak = presentation.goblin_outbreak;
    plan.party_rest_since = presentation.party_rest_since;
    plan
}

pub(crate) fn render_fitted_scene_for_world(
    snapshot: &SceneSnapshot,
    presentation: &ScenePresentation,
    target: &mut RgbBuffer,
) -> SceneFrame {
    let viewport = PixelSize::new(160, 90);
    let plan = presented_plan(snapshot, presentation, viewport);
    render::party::paint(snapshot, &plan, target)
        .unwrap_or_else(|| render::paint(snapshot, &plan, viewport, target))
}

/// Storybook options use the same scene plan and painters as the live rooms.
#[cfg(feature = "storybook")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RoomStudy {
    DelveWalk(std::time::Duration),
    OriginalMaterials,
}

#[cfg(feature = "storybook")]
pub(crate) fn render_study_frame(
    snapshot: &SceneSnapshot,
    presentation: &ScenePresentation,
    study: RoomStudy,
    viewport: PixelSize,
    target: &mut RgbBuffer,
) -> SceneFrame {
    let mut plan = presented_plan(snapshot, presentation, viewport);
    plan.study = Some(study);
    if viewport == PixelSize::new(160, 90)
        && let Some(frame) = render::party::paint(snapshot, &plan, target)
    {
        return frame;
    }
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
