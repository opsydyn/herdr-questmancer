use crate::{
    app::{Modal, Model, View},
    domain::{AgentKey, Timestamp},
    scene::stage::WorldScene,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SceneOverlay {
    None,
    Counsel,
    Search,
    LibrarianLedger,
    Scrying,
    Chronicle,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScenePresentation {
    pub world: WorldScene,
    pub selected_agent: Option<AgentKey>,
    pub overlay: SceneOverlay,
    /// Socket-lifetime cutoff for one-shot theatre, without changing summons.
    pub transition_floor: Option<Timestamp>,
    /// Whether the goblin outbreak is currently running.
    ///
    /// This rides on the presentation rather than the scene snapshot on
    /// purpose: `snapshot_ignores_legacy_ui_persistence_and_goblin_state`
    /// requires that releasing goblins never alters Herdr-reported truth.
    pub goblin_outbreak: bool,
}

impl ScenePresentation {
    #[must_use]
    pub fn from_model(model: &Model) -> Self {
        Self {
            world: match model.view() {
                View::Guild => WorldScene::GuildHall,
                View::Delve => WorldScene::Delve,
            },
            selected_agent: model.selected_agent_key().cloned(),
            transition_floor: model.scene_transition_floor(),
            goblin_outbreak: model.goblins().is_visible(model.now()),
            overlay: match model.modal() {
                Modal::None => SceneOverlay::None,
                Modal::LibrarianLedger { .. } => SceneOverlay::LibrarianLedger,
                Modal::Counsel { .. } => SceneOverlay::Counsel,
                Modal::Search { .. } => SceneOverlay::Search,
                Modal::Scrying => SceneOverlay::Scrying,
                Modal::Chronicle => SceneOverlay::Chronicle,
            },
        }
    }
}
