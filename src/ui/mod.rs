mod theme;
mod views;

pub mod copy;
pub mod delve_projection;
pub mod delve_scene;
pub mod goblins;
pub mod input;
pub mod persona;
pub mod pixel;
pub mod theatre;
pub mod widgets;

use ratatui::Frame;

use crate::app::{Modal, Model, View};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RenderProjection {
    guild_goblin_sprite_visible: bool,
    guild_goblin_marginalia_visible: bool,
}

impl RenderProjection {
    #[must_use]
    pub const fn guild_goblin_effect_visible(self) -> bool {
        self.guild_goblin_sprite_visible || self.guild_goblin_marginalia_visible
    }

    pub(crate) const fn guild_goblin_motion(
        self,
        motion: crate::app::Motion,
    ) -> Option<crate::app::Motion> {
        if self.guild_goblin_sprite_visible {
            Some(motion)
        } else if self.guild_goblin_marginalia_visible {
            Some(crate::app::Motion::None)
        } else {
            None
        }
    }
}

pub fn render(frame: &mut Frame<'_>, model: &Model) {
    render_with_projection(frame, model);
}

pub fn render_with_projection(frame: &mut Frame<'_>, model: &Model) -> RenderProjection {
    let (guild_goblin_marginalia_visible, guild_goblin_sprite_visible) = match model.view() {
        View::Guild => views::guild_hall::render(frame, model),
        View::Delve => {
            views::delve::render(frame, model);
            (false, false)
        }
    };
    if matches!(model.modal(), Modal::Counsel { .. } | Modal::Search { .. }) {
        views::reply::render(frame, model);
    } else if model.modal() == &Modal::Help {
        views::help::render(frame, model);
    }
    RenderProjection {
        guild_goblin_sprite_visible,
        guild_goblin_marginalia_visible,
    }
}
