//! Isolated party-capacity studies using existing authored assets.
use ratatui::layout::Rect;

use super::fixtures::{FitParty, StoryContext, hall_fit_fixture};
use crate::{
    app::{Model, View},
    domain::{
        AdventurerClass, AgentKey, GuildAttention, GuildSummons, PaneId, Presence, Timestamp,
    },
    scene::{
        SceneActorRegion,
        assets::{
            adventurer::{adventurer_animation_frame, adventurer_roster_frame},
            roster,
        },
        fit::{CellPixels, FitFailure, FitScene, FitTarget, FitTransform},
        pixel::{PixelPoint, PixelRect, PixelSize, Rgb, RgbBuffer},
        presentation::ScenePresentation,
        render_scene_for_world,
        snapshot::SceneSnapshot,
        sprite::blit,
        stage::{ScenePose, WorldScene},
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PartySize {
    Seven,
    TwentyFour,
    ThirtyTwo,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StudyScale {
    Large,
    Small,
}

pub(crate) fn scale_for_story(story: &str) -> Option<StudyScale> {
    story.starts_with("world.capacity-").then(|| {
        if story.ends_with("-small") {
            StudyScale::Small
        } else {
            StudyScale::Large
        }
    })
}

/// Fixed local identities only; no Herdr commands or persistent work.
pub fn fixture(context: StoryContext, size: PartySize, view: View) -> Model {
    let count = match size {
        PartySize::Seven => 7,
        PartySize::TwentyFour => 24,
        PartySize::ThirtyTwo => 32,
    };
    let mut model = hall_fit_fixture(context, FitParty::Eleven);
    let seeds = model.domain().agents.values().cloned().collect::<Vec<_>>();
    let agents = seeds
        .iter()
        .cycle()
        .take(count)
        .enumerate()
        .map(|(index, seed)| {
            let mut agent = seed.clone();
            agent.key = AgentKey::new(format!("capacity-{index:02}"));
            agent.pane_id = PaneId::new(format!("storybook:capacity-{index:02}"));
            agent.name = format!("Adventurer {:02}", index + 1);
            agent.persona.class = AdventurerClass::ALL[index % AdventurerClass::ALL.len()];
            agent.presence = if size == PartySize::Seven {
                if index == 5 {
                    Presence::Unknown
                } else {
                    Presence::Idle
                }
            } else {
                [
                    Presence::Working,
                    Presence::Idle,
                    Presence::Blocked,
                    Presence::Unknown,
                ][index % 4]
            };
            agent.attention = if agent.presence == Presence::Blocked {
                GuildAttention::unread(
                    GuildSummons::CounselRequested,
                    Timestamp::from_millis(111_000),
                )
            } else {
                GuildAttention::Clear
            };
            agent.custom_status = None;
            (agent.key.clone(), agent)
        })
        .collect();
    model.domain_mut().agents = agents;
    let keys = model
        .domain()
        .agents
        .values()
        .map(|a| (a.key.clone(), a.workspace_id.clone()))
        .collect::<Vec<_>>();
    for campaign in model.domain_mut().campaigns.values_mut() {
        campaign.party = keys
            .iter()
            .filter(|(_, workspace)| workspace == &campaign.workspace_id)
            .map(|(key, _)| key.clone())
            .collect();
    }
    model.domain_mut().selected_agent = Some(AgentKey::new("capacity-00"));
    model.switch_to(view);
    model
}

/// Preview projection only; production does not import this module.
pub fn project(
    model: &Model,
    area: Rect,
    cell: CellPixels,
    scale: StudyScale,
) -> Result<FitScene, FitFailure> {
    let transform = FitTransform::new(area, cell).ok_or(FitFailure::Geometry)?;
    let mut backdrop = SceneSnapshot::from_model(model);
    let agents = std::mem::take(&mut backdrop.agents);
    let presentation = ScenePresentation::from_model(model);
    let mut rgb = RgbBuffer::filled(160, 90, Rgb::BLACK);
    let mut frame =
        render_scene_for_world(&backdrop, &presentation, PixelSize::new(160, 90), &mut rgb);
    frame.next_frame_in = None;
    let slots = seats(presentation.world, scale);
    let live = agents
        .iter()
        .filter(|a| a.presence != Presence::Exited)
        .collect::<Vec<_>>();
    if live.len() > slots.len() {
        return Err(FitFailure::Incomplete {
            visible: slots.len(),
            total: live.len(),
        });
    }
    for (agent, origin) in live.iter().zip(&slots) {
        let pose = pose(agent.presence);
        let sprite = match scale {
            StudyScale::Large => adventurer_animation_frame(&agent.persona, pose, 0),
            StudyScale::Small => adventurer_roster_frame(&agent.persona),
        };
        let bounds = PixelRect::new(
            origin.x,
            origin.y,
            sprite.size().width,
            sprite.size().height,
        );
        rgb.fill_rect(
            PixelRect::new(
                bounds.x,
                (bounds.y + i32::from(bounds.height)).min(89),
                bounds.width,
                1,
            ),
            Rgb::new(24, 24, 30),
        );
        blit(&sprite, *origin, &mut rgb);
        frame.actors.push(SceneActorRegion {
            agent: agent.key.clone(),
            bounds,
        });
    }
    for (agent, actor) in live.iter().zip(&frame.actors) {
        let bounds = actor.bounds;
        let marker = roster::state_marker(pose(agent.presence));
        let marker_bounds = PixelRect::new(
            bounds.x + i32::from(bounds.width / 2) - 2,
            bounds.y - 6,
            5,
            5,
        );
        if frame
            .actors
            .iter()
            .all(|other| !intersects(marker_bounds, other.bounds))
            && frame
                .interactables
                .iter()
                .all(|other| !intersects(marker_bounds, other.bounds))
        {
            blit(
                marker,
                PixelPoint::new(marker_bounds.x, marker_bounds.y),
                &mut rgb,
            );
        }
        if model.selected_agent_key() == Some(&agent.key) {
            rgb.fill_rect(
                PixelRect::new(
                    bounds.x,
                    (bounds.y + i32::from(bounds.height) + 1).min(89),
                    bounds.width,
                    1,
                ),
                Rgb::new(0, 230, 245),
            );
        }
    }
    let scene = FitScene {
        rgb,
        frame,
        transform,
    };
    for actor in &scene.frame.actors {
        if !scene
            .cells()
            .any(|(x, y)| scene.target_at(x, y) == Some(FitTarget::Adventurer(actor.agent.clone())))
        {
            return Err(FitFailure::HitGeometry);
        }
    }
    Ok(scene)
}

fn pose(presence: Presence) -> ScenePose {
    match presence {
        Presence::Working => ScenePose::Working,
        Presence::Blocked => ScenePose::SeekingCounsel,
        Presence::Done => ScenePose::Settled,
        Presence::Idle => ScenePose::Resting,
        Presence::Unknown | Presence::Exited => ScenePose::Unknown,
    }
}

fn seats(world: WorldScene, scale: StudyScale) -> Vec<PixelPoint> {
    use crate::scene::render::party::{self, PartyDensity};
    party::seats(
        world,
        match scale {
            StudyScale::Large => PartyDensity::Full,
            StudyScale::Small => PartyDensity::Small,
        },
    )
}

fn intersects(a: PixelRect, b: PixelRect) -> bool {
    a.x < b.x + i32::from(b.width)
        && a.x + i32::from(a.width) > b.x
        && a.y < b.y + i32::from(b.height)
        && a.y + i32::from(a.height) > b.y
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn thirty_two_small_adventurers_keep_the_complete_room_and_mouse_targets() {
        for view in [View::Guild, View::Delve] {
            let model = fixture(StoryContext::fixed(), PartySize::ThirtyTwo, view);
            for (width, height) in [(120, 34), (96, 28), (80, 24)] {
                let scene = project(
                    &model,
                    Rect::new(0, 1, width, height - 3),
                    CellPixels::new(8, 16).unwrap(),
                    StudyScale::Small,
                )
                .unwrap_or_else(|e| panic!("{view:?}: capacity study lost room fitting: {e}"));
                assert_eq!(scene.frame.actors.len(), 32);
                assert!(
                    scene
                        .frame
                        .actors
                        .iter()
                        .all(|a| a.bounds.width == 8 && a.bounds.height == 12)
                );
                for actor in &scene.frame.actors {
                    assert!(scene.cells().any(|(x, y)| scene.target_at(x, y)
                        == Some(crate::scene::fit::FitTarget::Adventurer(
                            actor.agent.clone()
                        ))));
                }
            }
        }
    }

    #[test]
    fn status_changes_keep_seats_room_landmarks_and_density_stable() {
        for view in [View::Guild, View::Delve] {
            for (size, scale) in [
                (PartySize::Seven, StudyScale::Large),
                (PartySize::Seven, StudyScale::Small),
                (PartySize::TwentyFour, StudyScale::Small),
                (PartySize::ThirtyTwo, StudyScale::Small),
            ] {
                let mut model = fixture(StoryContext::fixed(), size, view);
                let area = Rect::new(0, 1, 96, 25);
                let cell = CellPixels::new(8, 16).unwrap();
                let before = project(&model, area, cell, scale).unwrap();
                for presence in [
                    Presence::Idle,
                    Presence::Working,
                    Presence::Blocked,
                    Presence::Unknown,
                    Presence::Done,
                ] {
                    for agent in model.domain_mut().agents.values_mut() {
                        agent.presence = presence;
                    }
                    let after = project(&model, area, cell, scale).unwrap();
                    assert_eq!(after.transform, before.transform);
                    assert_eq!(
                        after.frame.actors, before.frame.actors,
                        "presence must not relocate the entire party"
                    );
                    assert_eq!(after.frame.next_frame_in, None);
                    for y in 0..8 {
                        for x in 0..160 {
                            assert_eq!(after.rgb.get(x, y), before.rgb.get(x, y));
                        }
                    }
                }
                for (index, actor) in before.frame.actors.iter().enumerate() {
                    assert!(actor.bounds.x >= 0 && actor.bounds.y >= 0);
                    assert!(actor.bounds.x + i32::from(actor.bounds.width) <= 160);
                    assert!(actor.bounds.y + i32::from(actor.bounds.height) <= 90);
                    assert!(
                        before
                            .frame
                            .actors
                            .iter()
                            .skip(index + 1)
                            .all(|other| !intersects(actor.bounds, other.bounds))
                    );
                    assert!(
                        before
                            .frame
                            .interactables
                            .iter()
                            .all(|other| !intersects(actor.bounds, other.bounds))
                    );
                }
            }
        }
    }

    #[test]
    fn small_state_cues_keep_their_authored_pixels_and_large_overflow_is_explicit() {
        for view in [View::Guild, View::Delve] {
            let model = fixture(StoryContext::fixed(), PartySize::ThirtyTwo, view);
            let area = Rect::new(0, 1, 120, 31);
            let cell = CellPixels::new(8, 16).unwrap();
            let scene = project(&model, area, cell, StudyScale::Small).unwrap();
            for actor in &scene.frame.actors {
                let marker =
                    roster::state_marker(pose(model.domain().agents[&actor.agent].presence));
                for y in 0_u16..5 {
                    for x in 0_u16..5 {
                        if let Some(colour) = marker.pixels()[usize::from(y * 5 + x)] {
                            assert_eq!(
                                scene.rgb.get(
                                    actor.bounds.x + 2 + i32::from(x),
                                    actor.bounds.y - 6 + i32::from(y)
                                ),
                                Some(colour)
                            );
                        }
                    }
                }
            }
            assert!(matches!(
                project(&model, area, cell, StudyScale::Large),
                Err(FitFailure::Incomplete {
                    visible: 11,
                    total: 32
                })
            ));
        }
    }
}
