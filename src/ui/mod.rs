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

use ratatui::{
    Frame,
    buffer::{Buffer, Cell},
    layout::Rect,
};

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

#[derive(Debug)]
struct EffectCell {
    position: (u16, u16),
    rendered: Cell,
}

#[derive(Debug, Default)]
pub(crate) struct EffectCells(Vec<EffectCell>);

impl EffectCells {
    pub(crate) fn changed_between(before: &Buffer, after: &Buffer, area: Rect) -> Self {
        let mut changed = Vec::new();
        for y in area.y..area.bottom() {
            for x in area.x..area.right() {
                let (Some(before_cell), Some(after_cell)) =
                    (before.cell((x, y)), after.cell((x, y)))
                else {
                    continue;
                };
                if before_cell != after_cell {
                    changed.push(EffectCell {
                        position: (x, y),
                        rendered: after_cell.clone(),
                    });
                }
            }
        }
        Self(changed)
    }

    fn survives_in(&self, buffer: &Buffer) -> bool {
        self.0
            .iter()
            .any(|evidence| buffer.cell(evidence.position) == Some(&evidence.rendered))
    }
}

#[derive(Debug, Default)]
pub(crate) struct GuildGoblinEvidence {
    pub(crate) sprites: EffectCells,
    pub(crate) marginalia: EffectCells,
}

impl GuildGoblinEvidence {
    fn project_after_overlays(&self, buffer: &Buffer) -> RenderProjection {
        RenderProjection {
            guild_goblin_sprite_visible: self.sprites.survives_in(buffer),
            guild_goblin_marginalia_visible: self.marginalia.survives_in(buffer),
        }
    }
}

pub fn render(frame: &mut Frame<'_>, model: &Model) {
    render_with_projection(frame, model);
}

pub fn render_with_projection(frame: &mut Frame<'_>, model: &Model) -> RenderProjection {
    let goblin_evidence = match model.view() {
        View::Guild => views::guild_hall::render(frame, model),
        View::Delve => {
            views::delve::render(frame, model);
            GuildGoblinEvidence::default()
        }
    };
    if matches!(model.modal(), Modal::Counsel { .. } | Modal::Search { .. }) {
        views::reply::render(frame, model);
    } else if model.modal() == &Modal::Help {
        views::help::render(frame, model);
    }
    goblin_evidence.project_after_overlays(frame.buffer_mut())
}
