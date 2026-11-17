//! Fitted-room party size tiers; source facts remain in the semantic scene plan.
use std::time::Duration;

use super::{actor_frame, actor_next_frame_delay, delve, guild_hall, roster};
use crate::{
    domain::Presence,
    scene::{
        SceneActorRegion, SceneFrame,
        assets::{adventurer::adventurer_roster_frame, roster as roster_assets},
        pixel::{PixelPoint, PixelRect, PixelSize, Rgb, RgbBuffer},
        snapshot::SceneSnapshot,
        sprite::blit,
        stage::{COMPLETION_THEATRE_MS, SceneEffect, ScenePlan, WorldScene},
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PartyDensity {
    Full,
    Small,
    Dense,
}

impl PartyDensity {
    fn for_count(count: usize) -> Option<Self> {
        match count {
            0..=11 => Some(Self::Full),
            12..=32 => Some(Self::Small),
            33..=64 => Some(Self::Dense),
            _ => None,
        }
    }
}

pub(crate) fn paint(
    snapshot: &SceneSnapshot,
    plan: &ScenePlan,
    target: &mut RgbBuffer,
) -> Option<SceneFrame> {
    let tier = PartyDensity::for_count(plan.actors.len())?;
    let mut frame = match plan.world {
        WorldScene::GuildHall => guild_hall::paint_fitted_backdrop(snapshot, plan, target),
        WorldScene::Delve => delve::paint_fitted_backdrop(snapshot, plan, target),
    };
    // Presence/attention ordering must not move every seat as a state changes.
    let mut placements = plan.actors.iter().collect::<Vec<_>>();
    placements.sort_by(|a, b| a.agent.cmp(&b.agent));
    let mut deadline = frame.next_frame_in;
    for (placement, origin) in placements.iter().zip(seats(plan.world, tier)) {
        let Some(agent) = snapshot.agents.iter().find(|a| a.key == placement.agent) else {
            continue;
        };
        let sprite = if tier == PartyDensity::Full {
            let animation = actor_frame(snapshot, plan, agent, placement.pose);
            deadline = deadline
                .into_iter()
                .chain(actor_next_frame_delay(
                    &animation,
                    origin,
                    PixelSize::new(160, 90),
                ))
                .min();
            animation.sprite
        } else {
            adventurer_roster_frame(&agent.persona)
        };
        let bounds = PixelRect::new(
            origin.x,
            origin.y,
            sprite.size().width,
            sprite.size().height,
        );
        target.fill_rect(
            PixelRect::new(
                bounds.x,
                (bounds.y + i32::from(bounds.height)).min(89),
                bounds.width,
                1,
            ),
            Rgb::new(24, 24, 30),
        );
        if plan.world == WorldScene::Delve && agent.presence == Presence::Unknown {
            delve::blit_uncertain(&sprite, origin, target);
        } else {
            blit(&sprite, origin, target);
        }
        frame.actors.push(SceneActorRegion {
            agent: agent.key.clone(),
            bounds,
        });
    }
    if tier != PartyDensity::Dense && plan.world == WorldScene::GuildHall {
        frame.interactables = guild_hall::paint_fitted_librarian(target)
            .into_iter()
            .collect();
    }
    paint_cues(&frame, &placements, target);
    deadline = deadline
        .into_iter()
        .chain(roster::paint_effects(snapshot, plan, target, &frame.actors))
        .chain(paint_departures(snapshot, plan, target))
        .min();
    frame.next_frame_in = deadline;
    Some(frame)
}

fn paint_cues(
    frame: &SceneFrame,
    placements: &[&crate::scene::stage::ActorPlacement],
    target: &mut RgbBuffer,
) {
    for actor in &frame.actors {
        let Some(placement) = placements.iter().find(|p| p.agent == actor.agent) else {
            continue;
        };
        let bounds = actor.bounds;
        let marker = PixelRect::new(
            bounds.x + i32::from(bounds.width / 2) - 2,
            bounds.y - 6,
            5,
            5,
        );
        if frame
            .actors
            .iter()
            .all(|other| !intersects(marker, other.bounds))
            && frame
                .interactables
                .iter()
                .all(|other| !intersects(marker, other.bounds))
        {
            blit(
                roster_assets::state_marker(placement.pose),
                PixelPoint::new(marker.x, marker.y),
                target,
            );
        }
        if placement.selected {
            target.fill_rect(
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
}

fn paint_departures(
    snapshot: &SceneSnapshot,
    plan: &ScenePlan,
    target: &mut RgbBuffer,
) -> Option<Duration> {
    let mut deadline = None;
    for effect in &plan.effects {
        if let SceneEffect::RecentDeparture { since, .. } = effect {
            let remaining = Duration::from_millis(COMPLETION_THEATRE_MS.cast_unsigned())
                .checked_sub(since.elapsed_until(snapshot.now))
                .filter(|d| !d.is_zero());
            if let Some(remaining) = remaining {
                let (x, y) = if plan.world == WorldScene::GuildHall {
                    (10, 56)
                } else {
                    (7, 49)
                };
                target.put(x, y, Rgb::new(245, 219, 160));
                deadline =
                    Some(deadline.map_or(remaining, |current: Duration| current.min(remaining)));
            }
        }
    }
    deadline
}

pub(crate) fn intersects(a: PixelRect, b: PixelRect) -> bool {
    a.x < b.x + i32::from(b.width)
        && a.x + i32::from(a.width) > b.x
        && a.y < b.y + i32::from(b.height)
        && a.y + i32::from(a.height) > b.y
}
pub(crate) fn seats(world: WorldScene, scale: PartyDensity) -> Vec<PixelPoint> {
    match (world, scale) {
        (_, PartyDensity::Dense) => (0..64)
            .map(|i| PixelPoint::new(1 + (i % 16) * 10, 17 + (i / 16) * 20))
            .collect(),
        (WorldScene::GuildHall, PartyDensity::Full) => [
            (39, 50),
            (81, 50),
            (121, 40),
            (144, 40),
            (23, 66),
            (61, 26),
            (101, 26),
            (58, 50),
            (100, 50),
            (116, 66),
            (139, 66),
        ]
        .map(|(x, y)| PixelPoint::new(x, y))
        .to_vec(),
        (WorldScene::Delve, PartyDensity::Full) => [
            (14, 20),
            (52, 20),
            (92, 20),
            (130, 20),
            (14, 60),
            (52, 60),
            (92, 60),
            (130, 60),
            (32, 40),
            (72, 40),
            (112, 40),
        ]
        .map(|(x, y)| PixelPoint::new(x, y))
        .to_vec(),
        (_, PartyDensity::Small) => [0, 10, 5, 2, 8, 3, 7, 1, 9, 4, 6]
            .into_iter()
            .flat_map(|column| {
                [50, 30, 70].map(|row| PixelPoint::new(30 + column * 12, row + (column % 3) * 2))
            })
            .collect(),
    }
}
