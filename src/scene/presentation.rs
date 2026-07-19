use crate::{
    app::{Modal, Model, View},
    domain::AgentKey,
    scene::stage::WorldScene,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SceneOverlay {
    None,
    Counsel,
    Search,
    Help,
    Scrying,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScenePresentation {
    pub world: WorldScene,
    pub selected_agent: Option<AgentKey>,
    pub overlay: SceneOverlay,
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
            overlay: match model.modal() {
                Modal::None => SceneOverlay::None,
                Modal::Help => SceneOverlay::Help,
                Modal::Counsel { .. } => SceneOverlay::Counsel,
                Modal::Search { .. } => SceneOverlay::Search,
            },
        }
    }
}
