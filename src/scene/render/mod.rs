pub mod delve;
pub mod guild_hall;
pub mod lighting;

use crate::scene::{
    SceneFrame,
    pixel::{PixelSize, RgbBuffer},
    snapshot::SceneSnapshot,
    stage::ScenePlan,
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
