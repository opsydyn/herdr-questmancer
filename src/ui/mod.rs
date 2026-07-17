mod theme;
mod views;

pub mod copy;
pub mod delve_projection;
pub mod delve_scene;
pub mod goblins;
pub mod guild_room_projection;
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
use std::collections::BTreeSet;

use crate::{
    app::{CharacterSet, Modal, Model, Region, View},
    domain::{AgentKey, WorkspaceId},
};

use self::{
    delve_scene::DelveVariant,
    theatre::{TheatrePose, frame_for},
    widgets::chamber_presentation,
};

pub use views::great_room::{GuildRoomRenderPath, great_room_render_plan};
pub use widgets::ChamberPresentation;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PersonaRenderMode {
    None,
    Silhouette,
    Full,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GuildRegion {
    QuestBoard,
    Party,
    Summons,
    Chronicle,
    AdventurerProfile,
    Scrying,
    Spoils,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum GuildPresentation {
    #[default]
    Tiny,
    Empty,
    Wide,
    Medium,
    Focused,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedAgent {
    pub key: AgentKey,
    pub pose: TheatrePose,
    pub chamber: Option<ChamberPresentation>,
    pub chamber_area: Option<Rect>,
    pub persona: PersonaRenderMode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedDelveRegion {
    pub workspace_id: WorkspaceId,
    pub area: Rect,
    pub variant: DelveVariant,
    pub active: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RenderProjection {
    guild_goblin_sprite_visible: bool,
    guild_goblin_marginalia_visible: bool,
    pub guild_regions: BTreeSet<GuildRegion>,
    pub guild_profile_agent: Option<AgentKey>,
    pub visible_agents: Vec<ProjectedAgent>,
    pub delve_variants: BTreeSet<DelveVariant>,
    pub delve_regions: Vec<ProjectedDelveRegion>,
    pub delve_connected_scene_visible: bool,
    pub guild_room: Option<guild_room_projection::GuildRoomProjection>,
    pub(crate) guild_presentation: GuildPresentation,
}

impl RenderProjection {
    #[must_use]
    pub const fn guild_goblin_effect_visible(&self) -> bool {
        self.guild_goblin_sprite_visible || self.guild_goblin_marginalia_visible
    }

    pub(crate) const fn guild_goblin_motion(
        &self,
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

#[must_use]
pub fn render_projection_for(model: &Model, area: Rect) -> RenderProjection {
    let mut projection = RenderProjection::default();
    match model.view() {
        View::Guild => project_guild(model, area, &mut projection),
        View::Delve => {
            let delve = delve_projection::render_projection(model, area);
            project_delve(model, &delve, &mut projection);
        }
    }
    projection
}

fn project_delve(
    model: &Model,
    delve: &delve_projection::DelveRenderProjection,
    projection: &mut RenderProjection,
) {
    projection.delve_connected_scene_visible = matches!(
        delve.content,
        delve_projection::DelveContentProjection::Connected { .. }
    );
    projection.delve_regions = delve
        .delves()
        .iter()
        .map(|region| ProjectedDelveRegion {
            workspace_id: region.workspace_id.clone(),
            area: region.area,
            variant: region.variant,
            active: region.active,
        })
        .collect();
    projection.delve_variants = projection
        .delve_regions
        .iter()
        .map(|region| region.variant)
        .collect();
    projection.visible_agents = delve
        .chambers()
        .into_iter()
        .filter_map(|projected| {
            let agent = model.domain().agents.get(&projected.key)?;
            let theatre = frame_for(agent, model.now(), model.preferences());
            let chamber = chamber_presentation(projected.area);
            Some(ProjectedAgent {
                key: projected.key.clone(),
                pose: theatre.pose,
                chamber: Some(chamber),
                chamber_area: Some(projected.area),
                persona: persona_render_mode_for_chamber(
                    chamber,
                    theatre.pose,
                    model.preferences().character_set,
                ),
            })
        })
        .collect();
}

fn project_guild(model: &Model, area: Rect, projection: &mut RenderProjection) {
    projection.guild_room = Some(guild_room_projection::project(model, area));
    if area.width < 4 || area.height < 3 {
        return;
    }
    if model.domain().agents.is_empty() {
        projection.guild_presentation = GuildPresentation::Empty;
        return;
    }
    projection.guild_presentation = if area.width >= 120 {
        projection.guild_regions.extend([
            GuildRegion::QuestBoard,
            GuildRegion::Party,
            GuildRegion::Summons,
            GuildRegion::Chronicle,
            GuildRegion::AdventurerProfile,
            GuildRegion::Scrying,
            GuildRegion::Spoils,
        ]);
        GuildPresentation::Wide
    } else if area.width >= 80 {
        projection.guild_regions.extend([
            GuildRegion::QuestBoard,
            GuildRegion::Party,
            GuildRegion::AdventurerProfile,
            GuildRegion::Scrying,
        ]);
        GuildPresentation::Medium
    } else {
        match model.region() {
            Region::QuestBoard => {
                projection.guild_regions.insert(GuildRegion::QuestBoard);
            }
            Region::Party => {
                projection.guild_regions.insert(GuildRegion::Party);
            }
            Region::Summons => {
                projection.guild_regions.insert(GuildRegion::Summons);
            }
            Region::Chronicle => {
                projection.guild_regions.insert(GuildRegion::Chronicle);
            }
            Region::Adventurer => {
                projection
                    .guild_regions
                    .extend([GuildRegion::AdventurerProfile, GuildRegion::Scrying]);
            }
        }
        GuildPresentation::Focused
    };
    if projection
        .guild_regions
        .contains(&GuildRegion::AdventurerProfile)
    {
        projection.guild_profile_agent = model.selected_agent_key().cloned();
    }
}

pub const fn persona_render_mode_for_chamber(
    chamber: ChamberPresentation,
    pose: TheatrePose,
    character_set: CharacterSet,
) -> PersonaRenderMode {
    if matches!(pose, TheatrePose::Departed) {
        return PersonaRenderMode::None;
    }
    match chamber {
        ChamberPresentation::Full | ChamberPresentation::CompactScene => match character_set {
            CharacterSet::Unicode => PersonaRenderMode::Full,
            CharacterSet::Ascii => PersonaRenderMode::Silhouette,
        },
        ChamberPresentation::Hidden | ChamberPresentation::Text => PersonaRenderMode::None,
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
    fn project_after_overlays(
        &self,
        buffer: &Buffer,
        mut projection: RenderProjection,
    ) -> RenderProjection {
        projection.guild_goblin_sprite_visible = self.sprites.survives_in(buffer);
        projection.guild_goblin_marginalia_visible = self.marginalia.survives_in(buffer);
        projection
    }
}

pub fn render(frame: &mut Frame<'_>, model: &Model) {
    render_with_projection(frame, model);
}

pub fn render_with_projection(frame: &mut Frame<'_>, model: &Model) -> RenderProjection {
    let (projection, goblin_evidence) = match model.view() {
        View::Guild => {
            let projection = render_projection_for(model, frame.area());
            let evidence = views::guild_hall::render(frame, model, &projection);
            (projection, evidence)
        }
        View::Delve => {
            let delve = delve_projection::render_projection(model, frame.area());
            let mut projection = RenderProjection::default();
            project_delve(model, &delve, &mut projection);
            views::delve::render(frame, model, &delve);
            (projection, GuildGoblinEvidence::default())
        }
    };
    if matches!(model.modal(), Modal::Counsel { .. } | Modal::Search { .. }) {
        views::reply::render(frame, model);
    } else if model.modal() == &Modal::Help {
        views::help::render(frame, model);
    }
    goblin_evidence.project_after_overlays(frame.buffer_mut(), projection)
}
