use std::{collections::HashMap, time::Duration};

use crate::{
    app::Motion,
    domain::{AgentKey, WorkspaceId},
    scene::{
        SceneActorRegion, SceneFrame, SceneInteractable, SceneInteractableRegion,
        assets::{
            adventurer::{adventurer_animation_frame, adventurer_pose_is_animated},
            guild_hall::{GuildHallAsset, frame},
            librarian,
            palette::{
                AMBER_LIGHT, EMBER, FLAME, INK_BLUE, OAK, OAK_DARK, OAK_LIGHT, PARCHMENT,
                PARCHMENT_DARK, PARCHMENT_LIGHT, RUG, RUG_DARK, RUG_GOLD, SHADOW, STONE,
                STONE_DARK, STONE_LIGHT, VOID, WINE_DARK,
            },
        },
        pixel::{PixelPoint, PixelRect, PixelSize, Rgb, RgbBuffer},
        snapshot::{SceneConnection, SceneSnapshot},
        sprite::blit,
        stage::{
            ActorPlacement, CameraAnchor, SceneCamera, SceneEffect, ScenePlan, ScenePose,
            TruthfulStation,
        },
    },
};

use super::interaction::{self, paint_selection_marker};
use super::{
    actor_animation_phase, actor_next_frame_delay, actor_origin, earliest_deadline,
    effect_animation_phase, is_visible, lighting, next_frame_delay, roster,
};

pub const WIDTH: i32 = 160;
pub const HEIGHT: i32 = 90;
const CANONICAL_WIDTH: u16 = 160;
const CANONICAL_HEIGHT: u16 = 90;
const COMPACT_MIN_WIDTH: u16 = 64;
const COMPACT_MIN_HEIGHT: u16 = 40;
const ADVENTURER_WIDTH: u16 = 16;
const ADVENTURER_HEIGHT: u16 = 24;
const CANONICAL_PARTY_CAPACITY: usize = 11;
const LIBRARIAN_CANONICAL_ORIGIN: PixelPoint = PixelPoint::new(7, 64);

// Flat, slightly darkened oak behind the compact and vignette party rows.
// The quiet-lane rule: material seams and speckle never run through an
// actor's silhouette, so the stage carries no plank or stone pattern.
const QUIET_STAGE: Rgb = Rgb::new(88, 53, 31);

const DOOR: PixelRect = PixelRect::new(5, 14, 25, 46);
const QUEST_WALL: PixelRect = PixelRect::new(34, 11, 43, 27);
const LEFT_TABLE: PixelRect = PixelRect::new(35, 47, 38, 27);
const RIGHT_TABLE: PixelRect = PixelRect::new(77, 47, 38, 27);
const BELL: PixelRect = PixelRect::new(116, 31, 18, 31);
const HEARTH: PixelRect = PixelRect::new(132, 9, 28, 49);
const SPOILS: PixelRect = PixelRect::new(111, 64, 49, 26);

// These anchors reserve complete 16x24 masters. They are intentionally spaced
// by at least one full master width or height so a busy station never turns
// into a stack of overlapping sprites.
const LEFT_TABLE_SLOTS: [PixelPoint; 2] = [PixelPoint::new(43, 60), PixelPoint::new(62, 60)];
const RIGHT_TABLE_SLOTS: [PixelPoint; 2] = [PixelPoint::new(85, 60), PixelPoint::new(104, 60)];
const COUNSEL_SLOTS: [PixelPoint; 1] = [PixelPoint::new(125, 50)];
const HEARTH_SLOTS: [PixelPoint; 1] = [PixelPoint::new(148, 50)];
const SPOILS_SLOTS: [PixelPoint; 2] = [PixelPoint::new(120, 76), PixelPoint::new(143, 76)];
const OVERFLOW_ALCOVE_SLOTS: [PixelPoint; 3] = [
    PixelPoint::new(25, 76),
    PixelPoint::new(65, 36),
    PixelPoint::new(105, 36),
];

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum CampaignTable {
    Left,
    Right,
}

impl CampaignTable {
    const fn focus(self) -> PixelPoint {
        match self {
            Self::Left => PixelPoint::new(54, 60),
            Self::Right => PixelPoint::new(96, 60),
        }
    }
}

pub fn paint(
    snapshot: &SceneSnapshot,
    plan: &ScenePlan,
    viewport: PixelSize,
    target: &mut RgbBuffer,
) -> SceneFrame {
    match composition_for(viewport, plan.actors.len()) {
        GuildHallComposition::Canonical => {}
        GuildHallComposition::Compact => return paint_compact(snapshot, plan, viewport, target),
        GuildHallComposition::Roster => return paint_roster(snapshot, plan, viewport, target),
        GuildHallComposition::Vignette => {
            return paint_vignette(snapshot, plan, viewport, target);
        }
        GuildHallComposition::StatusOnly => {
            return paint_status_only(snapshot, plan, viewport, target);
        }
    }
    target.ensure_size(viewport.width, viewport.height, VOID);
    let origin = room_origin(snapshot, plan, viewport);
    let seed = room_seed(snapshot);

    paint_materials(target, origin, seed);
    paint_architecture(target, origin);
    paint_furnishings(snapshot, target, origin);
    apply_connection_light(snapshot, target, origin);
    restore_landmark_signatures(target, origin);
    let (actor_deadline, actors) = paint_actors(snapshot, plan, target, origin);
    let interactables = paint_librarian(
        target,
        translate(
            origin,
            LIBRARIAN_CANONICAL_ORIGIN.x,
            LIBRARIAN_CANONICAL_ORIGIN.y,
        ),
    )
    .into_iter()
    .collect();
    let effect_deadline = paint_effects(snapshot, plan, target, origin);
    paint_connection_fact(snapshot, target, origin);

    SceneFrame {
        world: plan.world,
        next_frame_in: actor_deadline.into_iter().chain(effect_deadline).min(),
        actors,
        interactables,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GuildHallComposition {
    Canonical,
    Compact,
    Roster,
    Vignette,
    StatusOnly,
}

fn composition_for(viewport: PixelSize, actor_count: usize) -> GuildHallComposition {
    if viewport.width >= CANONICAL_WIDTH
        && viewport.height >= CANONICAL_HEIGHT
        && actor_count <= CANONICAL_PARTY_CAPACITY
    {
        GuildHallComposition::Canonical
    } else if viewport.width >= COMPACT_MIN_WIDTH
        && viewport.height >= COMPACT_MIN_HEIGHT
        && compact_party_capacity(viewport) >= actor_count
    {
        GuildHallComposition::Compact
    } else if viewport.width >= roster::MIN_WIDTH
        && viewport.height >= roster::MIN_HEIGHT
        && roster::capacity(viewport) >= actor_count
    {
        // The party no longer fits at world scale, but the whole party still
        // fits at the authored roster scale. Losing everyone except one
        // adventurer is a worse answer than a smaller authored master.
        GuildHallComposition::Roster
    } else if viewport.width >= ADVENTURER_WIDTH && viewport.height >= ADVENTURER_HEIGHT {
        GuildHallComposition::Vignette
    } else {
        GuildHallComposition::StatusOnly
    }
}

fn compact_actor_capacity(viewport: PixelSize) -> usize {
    let columns = viewport.width / ADVENTURER_WIDTH;
    let rows = viewport.height / ADVENTURER_HEIGHT;
    usize::from(columns) * usize::from(rows)
}

fn compact_party_capacity(viewport: PixelSize) -> usize {
    compact_actor_capacity(viewport).saturating_sub(1)
}

fn paint_vignette(
    snapshot: &SceneSnapshot,
    plan: &ScenePlan,
    viewport: PixelSize,
    target: &mut RgbBuffer,
) -> SceneFrame {
    target.ensure_size(viewport.width, viewport.height, VOID);
    paint_compact_materials(target, room_seed(snapshot));
    paint_quiet_stage(
        target,
        i32::from(viewport.height)
            .saturating_sub(i32::from(ADVENTURER_HEIGHT))
            .saturating_sub(2),
    );
    paint_vignette_architecture(target);
    apply_compact_connection_light(snapshot, target);
    let (actor_deadline, actors) = paint_priority_actor(snapshot, plan, target);
    let effect_deadline = paint_compact_effects(snapshot, plan, target, &actors);
    paint_compact_connection_fact(snapshot, target);

    SceneFrame {
        world: plan.world,
        next_frame_in: actor_deadline.into_iter().chain(effect_deadline).min(),
        actors,
        interactables: Vec::new(),
    }
}

/// The whole party at authored roster scale. This tier exists so a narrow
/// Herdr pane still answers "who needs counsel, who is working, how many
/// adventurers do I have" instead of showing one adventurer and hiding the
/// rest.
fn paint_roster(
    snapshot: &SceneSnapshot,
    plan: &ScenePlan,
    viewport: PixelSize,
    target: &mut RgbBuffer,
) -> SceneFrame {
    target.ensure_size(viewport.width, viewport.height, VOID);
    paint_compact_materials(target, room_seed(snapshot));
    let block_top = roster::block_top(viewport, plan.actors.len());
    // The stage reaches above the party far enough that a counsel marker
    // lands on the floor rather than up in the stonework.
    paint_quiet_stage(target, block_top.saturating_sub(6));
    apply_compact_connection_light(snapshot, target);

    let regions = roster::paint_party(snapshot, plan, target, block_top, SHADOW);
    // Completion theatre is a truthful one-shot signal, not decoration: a
    // narrow pane must still show that an adventurer returned with spoils.
    let effect_deadline = paint_compact_effects(snapshot, plan, target, &regions);
    paint_compact_connection_fact(snapshot, target);

    SceneFrame {
        world: plan.world,
        next_frame_in: effect_deadline,
        actors: regions,
        interactables: Vec::new(),
    }
}

fn paint_status_only(
    snapshot: &SceneSnapshot,
    plan: &ScenePlan,
    viewport: PixelSize,
    target: &mut RgbBuffer,
) -> SceneFrame {
    target.ensure_size(viewport.width, viewport.height, VOID);
    paint_compact_materials(target, room_seed(snapshot));
    paint_compact_connection_fact(snapshot, target);
    SceneFrame {
        world: plan.world,
        next_frame_in: None,
        actors: Vec::new(),
        interactables: Vec::new(),
    }
}

fn paint_vignette_architecture(target: &mut RgbBuffer) {
    let width = i32::from(target.size().width);
    let height = i32::from(target.size().height);
    let floor_line = height / 2;
    target.fill_rect(PixelRect::new(0, 0, target.size().width, 3), OAK_DARK);
    target.fill_rect(
        PixelRect::new(0, floor_line, target.size().width, 2),
        OAK_DARK,
    );
    let bell = frame(GuildHallAsset::CounselBell);
    blit(
        bell,
        PixelPoint::new(width - i32::from(bell.size().width), floor_line - 8),
        target,
    );
}

fn paint_priority_actor(
    snapshot: &SceneSnapshot,
    plan: &ScenePlan,
    target: &mut RgbBuffer,
) -> (Option<Duration>, Vec<SceneActorRegion>) {
    let Some(placement) = plan.actors.iter().min_by_key(|placement| {
        (
            !placement.selected,
            minimum_pose_priority(placement.pose),
            placement.agent.as_str(),
        )
    }) else {
        return (None, Vec::new());
    };
    let Some(agent) = snapshot
        .agents
        .iter()
        .find(|agent| agent.key == placement.agent)
    else {
        return (None, Vec::new());
    };
    let elapsed = agent.presence_since.elapsed_until(snapshot.now);
    let animation = actor_animation_phase(snapshot.motion, placement.pose, elapsed);
    let sprite = adventurer_animation_frame(&agent.persona, placement.pose, animation);
    let origin = PixelPoint::new(
        (i32::from(target.size().width) - i32::from(sprite.size().width)) / 2,
        i32::from(target.size().height) - i32::from(sprite.size().height),
    );
    let bounds = PixelRect::new(
        origin.x,
        origin.y,
        sprite.size().width,
        sprite.size().height,
    );
    paint_actor_grounding(target, bounds, placement.pose);
    blit(&sprite, origin, target);
    interaction::paint_actor_state_marker(target, bounds, placement.pose);
    if placement.selected {
        paint_selection_marker(target, origin, sprite.size());
    }
    let next_frame_in = if adventurer_pose_is_animated(&agent.persona, placement.pose) {
        actor_next_frame_delay(snapshot.motion, placement.pose, elapsed)
    } else {
        None
    };
    (
        next_frame_in,
        vec![SceneActorRegion {
            agent: placement.agent.clone(),
            bounds,
        }],
    )
}

const fn minimum_pose_priority(pose: crate::scene::stage::ScenePose) -> u8 {
    use crate::scene::stage::ScenePose;
    match pose {
        ScenePose::SeekingCounsel => 0,
        ScenePose::ReturningWithSpoils => 1,
        ScenePose::Working => 2,
        ScenePose::Settled => 3,
        ScenePose::Resting => 4,
        ScenePose::Unknown => 5,
    }
}

fn paint_compact(
    snapshot: &SceneSnapshot,
    plan: &ScenePlan,
    viewport: PixelSize,
    target: &mut RgbBuffer,
) -> SceneFrame {
    target.ensure_size(viewport.width, viewport.height, VOID);
    let seed = room_seed(snapshot);
    paint_compact_materials(target, seed);
    paint_compact_architecture(target);
    paint_quiet_stage(
        target,
        compact_stage_top(target.size(), plan.actors.len().saturating_add(1)),
    );
    paint_compact_furnishings(snapshot, target);
    apply_compact_connection_light(snapshot, target);
    let (actor_deadline, actors) = paint_compact_actors(snapshot, plan, target);
    let librarian_origin = compact_actor_origin(
        target.size(),
        0,
        plan.actors.len().saturating_add(1),
        librarian::world().size(),
    );
    let interactables = paint_librarian(target, librarian_origin)
        .into_iter()
        .collect();
    let effect_deadline = paint_compact_effects(snapshot, plan, target, &actors);
    paint_compact_connection_fact(snapshot, target);

    SceneFrame {
        world: plan.world,
        next_frame_in: actor_deadline.into_iter().chain(effect_deadline).min(),
        actors,
        interactables,
    }
}

fn paint_compact_materials(target: &mut RgbBuffer, seed: u64) {
    let floor_line = i32::from(target.size().height) / 2;
    for y in 0..target.size().height {
        for x in 0..target.size().width {
            let x = i32::from(x);
            let y = i32::from(y);
            let colour = if y < floor_line {
                let course = y.div_euclid(5);
                let offset = if course.rem_euclid(2) == 0 { 0 } else { 6 };
                if y.rem_euclid(5) == 0 || (x + offset).rem_euclid(12) == 0 {
                    STONE_DARK
                } else if material_variant(x.div_euclid(12), course, seed).is_multiple_of(4) {
                    STONE_LIGHT
                } else {
                    STONE
                }
            } else {
                let plank_y = y.saturating_sub(floor_line);
                let course = plank_y.div_euclid(6);
                let offset = if course.rem_euclid(2) == 0 { 0 } else { 9 };
                if plank_y.rem_euclid(6) == 0 || (x + offset).rem_euclid(18) == 0 {
                    OAK_DARK
                } else if material_variant(x.div_euclid(18), course, seed).is_multiple_of(4) {
                    OAK_LIGHT
                } else {
                    OAK
                }
            };
            target.put(x, y, colour);
        }
    }
}

fn paint_compact_architecture(target: &mut RgbBuffer) {
    let width = i32::from(target.size().width);
    let height = i32::from(target.size().height);
    let floor_line = height / 2;
    target.fill_rect(PixelRect::new(0, 0, target.size().width, 4), OAK_DARK);
    target.fill_rect(PixelRect::new(0, 4, target.size().width, 2), OAK);
    target.fill_rect(
        PixelRect::new(0, floor_line, target.size().width, 3),
        OAK_DARK,
    );
    target.fill_rect(
        PixelRect::new(0, floor_line + 2, target.size().width, 1),
        OAK_LIGHT,
    );
    for x in [0, width / 2, width.saturating_sub(3)] {
        target.fill_rect(
            PixelRect::new(x, 0, 3, u16::try_from(floor_line + 3).unwrap_or(0)),
            OAK_DARK,
        );
    }

    let map = frame(GuildHallAsset::QuestMapWall);
    let map_x = (width - i32::from(map.size().width)) / 2;
    blit(map, PixelPoint::new(map_x, 6), target);

    let door = frame(GuildHallAsset::GuildDoor);
    let door_y = (floor_line - i32::from(door.size().height) + 11).max(6);
    blit(door, PixelPoint::new(2, door_y), target);

    let hearth = frame(GuildHallAsset::Hearth);
    let hearth_x = width - i32::from(hearth.size().width) - 2;
    blit(hearth, PixelPoint::new(hearth_x, 7), target);
}

fn paint_compact_furnishings(snapshot: &SceneSnapshot, target: &mut RgbBuffer) {
    let width = i32::from(target.size().width);
    let height = i32::from(target.size().height);
    let rug_width = target.size().width.saturating_sub(20);
    target.fill_rect(PixelRect::new(10, height - 9, rug_width, 8), RUG_DARK);
    target.fill_rect(
        PixelRect::new(12, height - 8, rug_width.saturating_sub(4), 6),
        RUG_GOLD,
    );
    target.fill_rect(
        PixelRect::new(14, height - 7, rug_width.saturating_sub(8), 4),
        RUG,
    );

    let table = frame(GuildHallAsset::CampaignTable);
    let table_x = (width - i32::from(table.size().width)) / 2;
    let table_y = height - i32::from(table.size().height);
    blit(table, PixelPoint::new(table_x, table_y), target);

    let bell = frame(GuildHallAsset::CounselBell);
    blit(
        bell,
        PixelPoint::new(width / 4 - i32::from(bell.size().width) / 2, height - 15),
        target,
    );
    if let Some(campaign) = snapshot.campaigns.first() {
        let detail = i32::try_from(campaign.variant_seed % 5).unwrap_or(0);
        blit_asset(
            GuildHallAsset::WaxSeal,
            target,
            PixelPoint::new(0, 0),
            table_x + 8 + detail,
            table_y + 2,
        );
    }
}

fn apply_compact_connection_light(snapshot: &SceneSnapshot, target: &mut RgbBuffer) {
    let width = i32::from(target.size().width);
    lighting::apply_warm_pool(target, PixelPoint::new(width - 12, 16), 26, 15);
    match snapshot.connection {
        SceneConnection::Connected => {}
        SceneConnection::Connecting => {
            lighting::dim(target, 18);
            lighting::apply_warm_pool(target, PixelPoint::new(10, 20), 28, 28);
        }
        SceneConnection::Reconnecting { .. } => {
            lighting::dim(target, 25);
            lighting::apply_warm_pool(target, PixelPoint::new(10, 20), 28, 34);
        }
        SceneConnection::Offline => lighting::dim(target, 40),
        SceneConnection::Incompatible { .. } => lighting::dim(target, 8),
    }
}

fn paint_compact_actors(
    snapshot: &SceneSnapshot,
    plan: &ScenePlan,
    target: &mut RgbBuffer,
) -> (Option<Duration>, Vec<SceneActorRegion>) {
    let mut next_frame_in = None;
    let mut regions = Vec::with_capacity(plan.actors.len());
    let count = plan.actors.len();
    for (index, placement) in plan.actors.iter().enumerate() {
        let Some(agent) = snapshot
            .agents
            .iter()
            .find(|agent| agent.key == placement.agent)
        else {
            continue;
        };
        let elapsed = agent.presence_since.elapsed_until(snapshot.now);
        let animation = actor_animation_phase(snapshot.motion, placement.pose, elapsed);
        let sprite = adventurer_animation_frame(&agent.persona, placement.pose, animation);
        let actor_origin = compact_actor_origin(
            target.size(),
            index.saturating_add(1),
            count.saturating_add(1),
            sprite.size(),
        );
        let bounds = PixelRect::new(
            actor_origin.x,
            actor_origin.y,
            sprite.size().width,
            sprite.size().height,
        );
        if adventurer_pose_is_animated(&agent.persona, placement.pose)
            && is_visible(PixelPoint::new(0, 0), bounds, target.size())
            && let Some(delay) = actor_next_frame_delay(snapshot.motion, placement.pose, elapsed)
        {
            next_frame_in = Some(earliest_deadline(next_frame_in, delay));
        }
        paint_actor_grounding(target, bounds, placement.pose);
        blit(&sprite, actor_origin, target);
        interaction::paint_actor_state_marker(target, bounds, placement.pose);
        if placement.selected {
            paint_selection_marker(target, actor_origin, sprite.size());
        }
        regions.push(SceneActorRegion {
            agent: placement.agent.clone(),
            bounds,
        });
    }
    (next_frame_in, regions)
}

fn paint_librarian(target: &mut RgbBuffer, origin: PixelPoint) -> Option<SceneInteractableRegion> {
    let sprite = librarian::world();
    let bounds = PixelRect::new(
        origin.x,
        origin.y,
        sprite.size().width,
        sprite.size().height,
    );
    if bounds.x < 0
        || bounds.y < 0
        || bounds.x + i32::from(bounds.width) > i32::from(target.size().width)
        || bounds.y + i32::from(bounds.height) > i32::from(target.size().height)
    {
        return None;
    }
    blit(sprite, origin, target);
    Some(SceneInteractableRegion {
        kind: SceneInteractable::Librarian,
        bounds,
    })
}

/// Paints the flat stage from `top` to the bottom of the viewport, with a
/// one-pixel lip that grounds it against the patterned material above.
fn paint_quiet_stage(target: &mut RgbBuffer, top: i32) {
    let top = top.max(0);
    let Ok(height) = u16::try_from(i32::from(target.size().height).saturating_sub(top)) else {
        return;
    };
    target.fill_rect(
        PixelRect::new(0, top, target.size().width, height),
        QUIET_STAGE,
    );
    target.fill_rect(PixelRect::new(0, top, target.size().width, 1), OAK_DARK);
}

/// The topmost pixel row the compact party grid (including the Librarian's
/// reservation) can occupy, for the stage that flattens texture behind it.
fn compact_stage_top(viewport: PixelSize, total: usize) -> i32 {
    let adventurer = PixelSize::new(ADVENTURER_WIDTH, ADVENTURER_HEIGHT);
    compact_actor_origin(viewport, 0, total, adventurer)
        .y
        .min(compact_actor_origin(viewport, 0, total, librarian::world().size()).y)
        .saturating_sub(2)
}

fn compact_actor_origin(
    viewport: PixelSize,
    index: usize,
    count: usize,
    sprite: PixelSize,
) -> PixelPoint {
    let sprite_width = usize::from(sprite.width.max(1));
    let columns = (usize::from(viewport.width) / sprite_width).max(1);
    let rows = count.max(1).div_ceil(columns);
    let row = index / columns;
    let column = index % columns;
    let actors_in_row = count.saturating_sub(row * columns).min(columns).max(1);
    let row_width = actors_in_row.saturating_mul(sprite_width);
    let start_x = usize::from(viewport.width).saturating_sub(row_width) / 2;
    let required_height = rows.saturating_mul(usize::from(sprite.height));
    let start_y = usize::from(viewport.height).saturating_sub(required_height);
    PixelPoint::new(
        i32::try_from(start_x + column * sprite_width).unwrap_or(i32::MAX),
        i32::try_from(start_y + row * usize::from(sprite.height)).unwrap_or(i32::MAX),
    )
}

fn paint_compact_effects(
    snapshot: &SceneSnapshot,
    plan: &ScenePlan,
    target: &mut RgbBuffer,
    actors: &[SceneActorRegion],
) -> Option<Duration> {
    let mut next_frame_in = None;
    for effect in &plan.effects {
        match effect {
            SceneEffect::FreshSpoils { agent, since } => {
                let elapsed = since.elapsed_until(snapshot.now);
                let phase = effect_animation_phase(snapshot.motion, elapsed);
                let Some(region) = actors.iter().find(|region| region.agent == *agent) else {
                    continue;
                };
                let seed = stable_hash(agent.as_str().as_bytes())
                    ^ u64::try_from(phase).unwrap_or(u64::MAX);
                for index in 0..6_u64 {
                    let value = seed.rotate_left(u32::try_from(index * 7).unwrap_or(0));
                    let x = region.bounds.x
                        + i32::try_from(value % u64::from(region.bounds.width)).unwrap_or(0);
                    let y = (region.bounds.y - 1
                        + i32::try_from(value.rotate_left(11) % 8).unwrap_or(0))
                    .max(0);
                    target.put(
                        x,
                        y,
                        if index.is_multiple_of(2) {
                            AMBER_LIGHT
                        } else {
                            FLAME
                        },
                    );
                }
                if snapshot.motion == Motion::Full {
                    next_frame_in = Some(earliest_deadline(
                        next_frame_in,
                        next_frame_delay(elapsed, 8),
                    ));
                }
            }
            SceneEffect::RecentDeparture { workspace_id, .. } => {
                let x = i32::try_from(
                    stable_hash(workspace_id.as_str().as_bytes())
                        % u64::from(target.size().width.max(1)),
                )
                .unwrap_or(0);
                target.put(
                    x,
                    i32::from(target.size().height).saturating_sub(2),
                    PARCHMENT_LIGHT,
                );
            }
        }
    }
    next_frame_in
}

fn paint_compact_connection_fact(snapshot: &SceneSnapshot, target: &mut RgbBuffer) {
    match snapshot.connection {
        SceneConnection::Connecting => target.put(2, 2, AMBER_LIGHT),
        SceneConnection::Reconnecting { attempt } => {
            for index in 0..attempt.clamp(1, 6) {
                target.put(2 + i32::try_from(index * 2).unwrap_or(0), 2, AMBER_LIGHT);
            }
        }
        SceneConnection::Offline => target.put(2, 2, WINE_DARK),
        SceneConnection::Incompatible { .. } => {
            paint_connection_fact(snapshot, target, PixelPoint::new(0, 0));
        }
        SceneConnection::Connected => {}
    }
}

fn room_origin(snapshot: &SceneSnapshot, plan: &ScenePlan, viewport: PixelSize) -> PixelPoint {
    let width = i32::from(viewport.width);
    let height = i32::from(viewport.height);
    let focus = match &plan.camera {
        SceneCamera::WholeRoom => PixelPoint::new(WIDTH / 2, HEIGHT / 2),
        SceneCamera::Focused { anchor } => focus_point(snapshot, anchor),
    };
    let x = if width >= WIDTH {
        (width - WIDTH) / 2
    } else {
        -(focus.x - width / 2).clamp(0, WIDTH - width)
    };
    let y = if height >= HEIGHT {
        (height - HEIGHT) / 2
    } else {
        -(focus.y - height / 2).max(0)
    };
    PixelPoint::new(x, y)
}

fn focus_point(snapshot: &SceneSnapshot, anchor: &CameraAnchor) -> PixelPoint {
    match anchor {
        CameraAnchor::Door => PixelPoint::new(18, 38),
        CameraAnchor::CampaignTable(workspace) => campaign_table(snapshot, workspace).focus(),
        CameraAnchor::CounselBell => PixelPoint::new(124, 48),
        CameraAnchor::Hearth => PixelPoint::new(145, 38),
        CameraAnchor::Spoils => PixelPoint::new(135, 75),
        CameraAnchor::DelveParty(_) => PixelPoint::new(WIDTH / 2, HEIGHT / 2),
    }
}

fn paint_materials(target: &mut RgbBuffer, origin: PixelPoint, seed: u64) {
    for y in 0..target.size().height {
        for x in 0..target.size().width {
            let world_x = i32::from(x).saturating_sub(origin.x);
            let world_y = i32::from(y).saturating_sub(origin.y);
            let colour = if world_y < 50 {
                let course = world_y.div_euclid(6);
                let offset = if course.rem_euclid(2) == 0 { 0 } else { 7 };
                if world_y.rem_euclid(6) == 0 || (world_x + offset).rem_euclid(14) == 0 {
                    STONE_DARK
                } else if material_variant(world_x.div_euclid(14), course, seed).is_multiple_of(3) {
                    STONE_LIGHT
                } else if material_variant(world_x, world_y, seed).is_multiple_of(67) {
                    Rgb::new(132, 120, 111)
                } else {
                    STONE
                }
            } else {
                let plank_y = world_y.saturating_sub(50);
                let course = plank_y.div_euclid(7);
                let offset = if course.rem_euclid(2) == 0 { 0 } else { 12 };
                if plank_y.rem_euclid(7) == 0 || (world_x + offset).rem_euclid(24) == 0 {
                    OAK_DARK
                } else if material_variant(world_x.div_euclid(24), course, seed).is_multiple_of(3) {
                    OAK_LIGHT
                } else if material_variant(world_x, world_y, seed).is_multiple_of(79) {
                    Rgb::new(126, 75, 42)
                } else {
                    OAK
                }
            };
            target.put(i32::from(x), i32::from(y), colour);
        }
    }
}

fn material_variant(x: i32, y: i32, seed: u64) -> u64 {
    let x = u64::from(x.unsigned_abs());
    let y = u64::from(y.unsigned_abs());
    seed.rotate_left(11)
        ^ x.wrapping_mul(0x9e37_79b9)
        ^ y.wrapping_mul(0x85eb_ca6b)
        ^ (x / 7).wrapping_add(y / 5)
}

fn paint_architecture(target: &mut RgbBuffer, origin: PixelPoint) {
    fill(target, origin, PixelRect::new(0, 0, 160, 5), OAK_DARK);
    fill(target, origin, PixelRect::new(0, 5, 160, 3), OAK);
    fill(target, origin, PixelRect::new(0, 47, 160, 4), OAK_DARK);
    fill(target, origin, PixelRect::new(0, 50, 160, 2), OAK_LIGHT);
    for x in [0, 30, 79, 128, 157] {
        fill(target, origin, PixelRect::new(x, 0, 3, 52), OAK_DARK);
        fill(target, origin, PixelRect::new(x + 1, 3, 1, 44), OAK_LIGHT);
    }
    for x in (3..157).step_by(12) {
        blit_asset(GuildHallAsset::TimberBeam, target, origin, x, 3);
    }

    fill(target, origin, PixelRect::new(3, 12, 29, 49), SHADOW);
    blit_asset(GuildHallAsset::GuildDoor, target, origin, 8, 16);
    fill(target, origin, PixelRect::new(5, 59, 25, 3), STONE_LIGHT);
    fill(target, origin, PixelRect::new(7, 60, 21, 2), STONE);

    fill(target, origin, PixelRect::new(32, 9, 47, 31), OAK_DARK);
    blit_asset(GuildHallAsset::QuestMapWall, target, origin, 39, 13);
    put(target, origin, 37, 12, PARCHMENT_LIGHT);
    put(target, origin, 76, 36, PARCHMENT_DARK);

    blit_asset(GuildHallAsset::Shelf, target, origin, 83, 11);
    blit_asset(GuildHallAsset::Shelf, target, origin, 83, 25);
    blit_asset(GuildHallAsset::Banner, target, origin, 106, 9);

    fill(target, origin, PixelRect::new(130, 7, 30, 52), STONE_DARK);
    fill(target, origin, PixelRect::new(134, 12, 24, 44), STONE_LIGHT);
    fill(target, origin, PixelRect::new(136, 15, 20, 39), STONE_DARK);
    fill(target, origin, PixelRect::new(139, 19, 14, 32), SHADOW);
    blit_asset(GuildHallAsset::Hearth, target, origin, 134, 10);
    fill(target, origin, PixelRect::new(141, 46, 10, 3), OAK_DARK);
    for (x, y, colour) in [
        (144, 45, EMBER),
        (148, 45, EMBER),
        (145, 44, FLAME),
        (147, 44, FLAME),
        (146, 42, FLAME),
        (146, 43, AMBER_LIGHT),
    ] {
        put(target, origin, x, y, colour);
    }
    fill(target, origin, PixelRect::new(132, 7, 28, 4), STONE_LIGHT);
    fill(target, origin, PixelRect::new(130, 54, 30, 5), STONE);
}

fn paint_furnishings(snapshot: &SceneSnapshot, target: &mut RgbBuffer, origin: PixelPoint) {
    paint_rug(target, origin);
    blit_asset(GuildHallAsset::CampaignTable, target, origin, 40, 50);
    blit_asset(GuildHallAsset::CampaignTable, target, origin, 82, 50);
    for (x, y) in [(42, 45), (62, 45), (84, 45), (104, 45), (46, 68), (88, 68)] {
        blit_asset(GuildHallAsset::Chair, target, origin, x, y);
    }

    fill(target, origin, PixelRect::new(116, 53, 18, 7), OAK_DARK);
    blit_asset(GuildHallAsset::CounselBell, target, origin, 120, 40);
    blit_asset(GuildHallAsset::SpoilsBench, target, origin, 124, 69);
    blit_asset(GuildHallAsset::Shelf, target, origin, 3, 67);
    blit_asset(GuildHallAsset::Banner, target, origin, 18, 65);

    for (x, y) in [(33, 42), (76, 42), (109, 42), (128, 27)] {
        blit_asset(GuildHallAsset::Candle, target, origin, x, y);
    }
    for (index, campaign) in snapshot.campaigns.iter().enumerate() {
        let (table_x, table_y) = if index.is_multiple_of(2) {
            (40, 50)
        } else {
            (82, 50)
        };
        let detail = campaign.variant_seed;
        let mug_x = table_x + 3 + i32::try_from(detail % 9).unwrap_or(0);
        let dice_x = table_x + 16 + i32::try_from(detail.rotate_left(9) % 6).unwrap_or(0);
        blit_asset(GuildHallAsset::Mug, target, origin, mug_x, table_y + 3);
        blit_asset(GuildHallAsset::Dice, target, origin, dice_x, table_y + 4);
        blit_asset(
            GuildHallAsset::WaxSeal,
            target,
            origin,
            table_x + 11 + i32::try_from(detail % 4).unwrap_or(0),
            table_y + 2,
        );
    }
    blit_asset(GuildHallAsset::Scroll, target, origin, 69, 15);
    blit_asset(GuildHallAsset::Clutter, target, origin, 101, 31);
}

fn paint_rug(target: &mut RgbBuffer, origin: PixelPoint) {
    fill(target, origin, PixelRect::new(29, 66, 94, 23), RUG_DARK);
    fill(target, origin, PixelRect::new(31, 68, 90, 19), RUG_GOLD);
    fill(target, origin, PixelRect::new(33, 70, 86, 15), RUG);
    for x in (37..117).step_by(10) {
        put(target, origin, x, 73, RUG_GOLD);
        put(target, origin, x - 1, 74, RUG_GOLD);
        put(target, origin, x, 75, RUG_GOLD);
        put(target, origin, x + 1, 74, RUG_GOLD);
        put(target, origin, x, 81, RUG_DARK);
    }
}

fn apply_connection_light(snapshot: &SceneSnapshot, target: &mut RgbBuffer, origin: PixelPoint) {
    lighting::apply_warm_pool(target, translate(origin, 146, 38), 35, 18);
    lighting::apply_warm_pool(target, translate(origin, 78, 47), 28, 10);
    match snapshot.connection {
        SceneConnection::Connected => {}
        SceneConnection::Connecting => {
            lighting::dim(target, 18);
            lighting::apply_warm_pool(target, translate(origin, 18, 36), 42, 46);
        }
        SceneConnection::Reconnecting { .. } => {
            lighting::dim(target, 25);
            lighting::apply_warm_pool(target, translate(origin, 18, 36), 42, 55);
        }
        SceneConnection::Offline => {
            lighting::dim(target, 40);
            fill(target, origin, PixelRect::new(8, 17, 18, 39), SHADOW);
            fill(target, origin, PixelRect::new(9, 18, 16, 37), OAK_DARK);
            fill(target, origin, PixelRect::new(11, 20, 12, 35), WINE_DARK);
        }
        SceneConnection::Incompatible { .. } => lighting::dim(target, 8),
    }
}

fn restore_landmark_signatures(target: &mut RgbBuffer, origin: PixelPoint) {
    put(target, origin, DOOR.x + 2, DOOR.y + 2, OAK);
    put(
        target,
        origin,
        QUEST_WALL.x + 4,
        QUEST_WALL.y + 3,
        PARCHMENT,
    );
    put(target, origin, LEFT_TABLE.x + 7, LEFT_TABLE.y + 4, OAK);
    put(target, origin, RIGHT_TABLE.x + 7, RIGHT_TABLE.y + 4, OAK);
    put(target, origin, BELL.x + 7, BELL.y + 12, RUG_GOLD);
    put(target, origin, HEARTH.x + 14, HEARTH.y + 29, EMBER);
    put(target, origin, SPOILS.x + 17, SPOILS.y + 10, RUG);
}

fn paint_actors(
    snapshot: &SceneSnapshot,
    plan: &ScenePlan,
    target: &mut RgbBuffer,
    origin: PixelPoint,
) -> (Option<Duration>, Vec<SceneActorRegion>) {
    let mut next_frame_in = None;
    let mut regions = Vec::new();
    for (placement, anchor) in actor_anchors(snapshot, plan) {
        let Some(agent) = snapshot
            .agents
            .iter()
            .find(|agent| agent.key == placement.agent)
        else {
            continue;
        };
        let elapsed = agent.presence_since.elapsed_until(snapshot.now);
        let animation = actor_animation_phase(snapshot.motion, placement.pose, elapsed);
        let sprite = adventurer_animation_frame(&agent.persona, placement.pose, animation);
        let actor_origin = actor_origin(origin, anchor, &sprite);
        let bounds = PixelRect::new(
            actor_origin.x,
            actor_origin.y,
            sprite.size().width,
            sprite.size().height,
        );
        if adventurer_pose_is_animated(&agent.persona, placement.pose)
            && is_visible(PixelPoint::new(0, 0), bounds, target.size())
            && let Some(delay) = actor_next_frame_delay(snapshot.motion, placement.pose, elapsed)
        {
            next_frame_in = Some(earliest_deadline(next_frame_in, delay));
        }
        paint_actor_grounding(target, bounds, placement.pose);
        blit(&sprite, actor_origin, target);
        interaction::paint_actor_state_marker(target, bounds, placement.pose);
        if placement.selected {
            paint_selection_marker(target, actor_origin, sprite.size());
        }
        regions.push(SceneActorRegion {
            agent: placement.agent.clone(),
            bounds,
        });
    }
    (next_frame_in, regions)
}

fn actor_anchors<'a>(
    snapshot: &SceneSnapshot,
    plan: &'a ScenePlan,
) -> Vec<(&'a ActorPlacement, PixelPoint)> {
    let mut station_counts = HashMap::<String, usize>::new();
    let mut overflow_count = 0;
    let mut result = Vec::with_capacity(plan.actors.len());
    for placement in &plan.actors {
        let key = station_key(snapshot, &placement.station);
        let slot = *station_counts
            .entry(key)
            .and_modify(|n| *n += 1)
            .or_insert(0);
        let primary_slot = match &placement.station {
            TruthfulStation::CampaignToken(workspace) => {
                campaign_table_slots(campaign_table(snapshot, workspace)).get(slot)
            }
            TruthfulStation::CounselBell => COUNSEL_SLOTS.get(slot),
            TruthfulStation::SpoilsBench => SPOILS_SLOTS.get(slot),
            TruthfulStation::Hearth => HEARTH_SLOTS.get(slot),
            TruthfulStation::DelveActive(_)
            | TruthfulStation::DelveGate(_)
            | TruthfulStation::DelveExit(_)
            | TruthfulStation::DelveCamp(_) => continue,
        };
        let anchor = primary_slot.copied().or_else(|| {
            let overflow = OVERFLOW_ALCOVE_SLOTS.get(overflow_count).copied();
            overflow_count += usize::from(overflow.is_some());
            overflow
        });
        if let Some(anchor) = anchor {
            result.push((placement, anchor));
        }
    }
    result
}

const fn campaign_table_slots(table: CampaignTable) -> &'static [PixelPoint] {
    match table {
        CampaignTable::Left => &LEFT_TABLE_SLOTS,
        CampaignTable::Right => &RIGHT_TABLE_SLOTS,
    }
}

/// The Hall is candlelit, so its actors ground with the warm room shadow.
fn paint_actor_grounding(target: &mut RgbBuffer, bounds: PixelRect, pose: ScenePose) {
    interaction::paint_actor_grounding(target, bounds, pose, SHADOW);
}

fn campaign_table(snapshot: &SceneSnapshot, workspace: &WorkspaceId) -> CampaignTable {
    let index = snapshot
        .campaigns
        .iter()
        .position(|campaign| campaign.workspace_id == *workspace);
    if index.map_or_else(
        || stable_hash(workspace.as_str().as_bytes()).is_multiple_of(2),
        |index| index.is_multiple_of(2),
    ) {
        CampaignTable::Left
    } else {
        CampaignTable::Right
    }
}

fn station_key(snapshot: &SceneSnapshot, station: &TruthfulStation) -> String {
    match station {
        TruthfulStation::CampaignToken(workspace) => {
            format!("campaign:{:?}", campaign_table(snapshot, workspace))
        }
        TruthfulStation::CounselBell => "bell".to_owned(),
        TruthfulStation::SpoilsBench => "spoils".to_owned(),
        TruthfulStation::Hearth => "hearth".to_owned(),
        TruthfulStation::DelveActive(workspace) => format!("active:{workspace}"),
        TruthfulStation::DelveGate(workspace) => format!("gate:{workspace}"),
        TruthfulStation::DelveExit(workspace) => format!("exit:{workspace}"),
        TruthfulStation::DelveCamp(workspace) => format!("camp:{workspace}"),
    }
}

fn paint_effects(
    snapshot: &SceneSnapshot,
    plan: &ScenePlan,
    target: &mut RgbBuffer,
    origin: PixelPoint,
) -> Option<Duration> {
    let mut next_frame_in = None;
    for effect in &plan.effects {
        match effect {
            SceneEffect::FreshSpoils { agent, since } => {
                let elapsed = since.elapsed_until(snapshot.now);
                let phase = effect_animation_phase(snapshot.motion, elapsed);
                let visible = paint_spoils_flash(target, origin, agent, phase);
                if snapshot.motion == Motion::Full && visible {
                    next_frame_in = Some(earliest_deadline(
                        next_frame_in,
                        next_frame_delay(elapsed, 8),
                    ));
                }
            }
            SceneEffect::RecentDeparture { workspace_id, .. } => {
                let offset =
                    i32::try_from(stable_hash(workspace_id.as_str().as_bytes()) % 5).unwrap_or(0);
                put(target, origin, 12 + offset, 59, PARCHMENT_LIGHT);
            }
        }
    }
    next_frame_in
}

fn paint_spoils_flash(
    target: &mut RgbBuffer,
    origin: PixelPoint,
    agent: &AgentKey,
    phase: u128,
) -> bool {
    let seed = stable_hash(agent.as_str().as_bytes()) ^ u64::try_from(phase).unwrap_or(u64::MAX);
    let mut visible = false;
    for index in 0..12_u64 {
        let value = seed.rotate_left(u32::try_from(index * 5).unwrap_or(0));
        let x = 113 + i32::try_from(value % 44).unwrap_or(0);
        let y = 59 + i32::try_from(value.rotate_left(13) % 25).unwrap_or(0);
        visible |= is_visible(origin, PixelRect::new(x, y, 1, 1), target.size());
        put(
            target,
            origin,
            x,
            y,
            if index.is_multiple_of(2) {
                AMBER_LIGHT
            } else {
                FLAME
            },
        );
    }
    visible
}

fn paint_connection_fact(snapshot: &SceneSnapshot, target: &mut RgbBuffer, origin: PixelPoint) {
    match snapshot.connection {
        SceneConnection::Connecting => {
            put(target, origin, 17, 58, AMBER_LIGHT);
        }
        SceneConnection::Reconnecting { attempt } => {
            let pips = attempt.clamp(1, 6);
            for index in 0..pips {
                put(
                    target,
                    origin,
                    14 + i32::try_from(index).unwrap_or(0) * 2,
                    58,
                    AMBER_LIGHT,
                );
            }
        }
        SceneConnection::Incompatible { expected, actual } => {
            let left = i32::from(target.size().width).saturating_sub(21).max(0);
            let top = 2;
            target.fill_rect(PixelRect::new(left, top, 19, 6), SHADOW);
            target.fill_rect(PixelRect::new(left + 1, top + 1, 17, 4), PARCHMENT_DARK);
            for index in 0..expected.min(7) {
                target.put(
                    left + 2 + i32::try_from(index * 2).unwrap_or(0),
                    top + 2,
                    PARCHMENT_LIGHT,
                );
            }
            for index in 0..actual.min(7) {
                target.put(
                    left + 2 + i32::try_from(index * 2).unwrap_or(0),
                    top + 4,
                    INK_BLUE,
                );
            }
        }
        SceneConnection::Offline | SceneConnection::Connected => {}
    }
}

fn room_seed(snapshot: &SceneSnapshot) -> u64 {
    snapshot
        .campaigns
        .iter()
        .fold(0x4f1b_a11d_u64, |seed, campaign| {
            seed.rotate_left(7) ^ campaign.variant_seed
        })
}

fn stable_hash(bytes: &[u8]) -> u64 {
    let digest = blake3::hash(bytes);
    u64::from_le_bytes(
        digest.as_bytes()[..8]
            .try_into()
            .expect("digest has eight bytes"),
    )
}

fn blit_asset(asset: GuildHallAsset, target: &mut RgbBuffer, origin: PixelPoint, x: i32, y: i32) {
    blit(frame(asset), translate(origin, x, y), target);
}

fn fill(target: &mut RgbBuffer, origin: PixelPoint, rect: PixelRect, colour: Rgb) {
    let translated = PixelRect::new(
        origin.x.saturating_add(rect.x),
        origin.y.saturating_add(rect.y),
        rect.width,
        rect.height,
    );
    target.fill_rect(translated, colour);
}

fn put(target: &mut RgbBuffer, origin: PixelPoint, x: i32, y: i32, colour: Rgb) {
    let point = translate(origin, x, y);
    target.put(point.x, point.y, colour);
}

const fn translate(origin: PixelPoint, x: i32, y: i32) -> PixelPoint {
    PixelPoint::new(origin.x.saturating_add(x), origin.y.saturating_add(y))
}
