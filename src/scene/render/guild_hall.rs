use std::{collections::HashMap, time::Duration};

use crate::{
    app::Motion,
    domain::{AgentKey, WorkspaceId},
    scene::{
        SceneFrame,
        assets::{
            adventurer::compact_adventurer_animation_frame,
            guild_hall::{GuildHallAsset, frame},
            palette::{
                AMBER_LIGHT, BRASS_DARK, EMBER, FLAME, INK_BLUE, OAK, OAK_DARK, OAK_LIGHT,
                PARCHMENT, PARCHMENT_DARK, PARCHMENT_LIGHT, RUG, RUG_DARK, RUG_GOLD, SHADOW, STONE,
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

use super::interaction::paint_selection_marker;
use super::{
    actor_animation_phase, actor_next_frame_delay, earliest_deadline, effect_animation_phase,
    is_visible, lighting, next_frame_delay, painted_sprite_is_visible,
};

pub const WIDTH: i32 = 160;
pub const HEIGHT: i32 = 90;

const DOOR: PixelRect = PixelRect::new(5, 14, 25, 46);
const QUEST_WALL: PixelRect = PixelRect::new(34, 11, 43, 27);
const LEFT_TABLE: PixelRect = PixelRect::new(35, 47, 38, 27);
const RIGHT_TABLE: PixelRect = PixelRect::new(77, 47, 38, 27);
const BELL: PixelRect = PixelRect::new(116, 31, 18, 31);
const HEARTH: PixelRect = PixelRect::new(132, 9, 28, 49);
const SPOILS: PixelRect = PixelRect::new(111, 64, 49, 26);

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

    const fn actor_base_x(self) -> i32 {
        match self {
            Self::Left => 40,
            Self::Right => 82,
        }
    }
}

pub fn paint(
    snapshot: &SceneSnapshot,
    plan: &ScenePlan,
    viewport: PixelSize,
    target: &mut RgbBuffer,
) -> SceneFrame {
    target.ensure_size(viewport.width, viewport.height, VOID);
    let origin = room_origin(snapshot, plan, viewport);
    let seed = room_seed(snapshot);

    paint_materials(target, origin, seed);
    paint_architecture(target, origin);
    paint_furnishings(snapshot, target, origin);
    apply_connection_light(snapshot, target, origin);
    restore_landmark_signatures(target, origin);
    let actor_deadline = paint_actors(snapshot, plan, target, origin);
    let effect_deadline = paint_effects(snapshot, plan, target, origin);
    paint_connection_fact(snapshot, target, origin);

    SceneFrame {
        world: plan.world,
        next_frame_in: actor_deadline.into_iter().chain(effect_deadline).min(),
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
) -> Option<Duration> {
    let mut next_frame_in = None;
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
        if let TruthfulStation::CampaignToken(_) = placement.station {
            let token_origin = translate(origin, anchor.x, anchor.y - i32::from(animation == 1));
            if token_is_visible(token_origin, target.size())
                && let Some(delay) =
                    actor_next_frame_delay(snapshot.motion, placement.pose, elapsed)
            {
                next_frame_in = Some(earliest_deadline(next_frame_in, delay));
            }
            paint_token(
                target,
                translate(origin, anchor.x, anchor.y),
                crate::scene::assets::palette::adventurer_palette(
                    agent.persona.appearance.skin_tone,
                    agent.persona.appearance.hair_tone,
                    agent.persona.appearance.garb,
                    agent.persona.class,
                    agent.persona.appearance.accent,
                )
                .accent,
                placement.pose == ScenePose::Unknown,
                animation,
            );
            if placement.selected {
                paint_selection_marker(target, token_origin, PixelSize::new(5, 5));
            }
        } else {
            let sprite =
                compact_adventurer_animation_frame(&agent.persona, placement.pose, animation);
            let actor_origin = translate(origin, anchor.x, anchor.y - i32::from(animation == 1));
            if painted_sprite_is_visible(&sprite, actor_origin, target.size())
                && let Some(delay) =
                    actor_next_frame_delay(snapshot.motion, placement.pose, elapsed)
            {
                next_frame_in = Some(earliest_deadline(next_frame_in, delay));
            }
            blit(&sprite, actor_origin, target);
            if placement.selected {
                paint_selection_marker(target, actor_origin, sprite.size());
            }
        }
    }
    next_frame_in
}

fn token_is_visible(origin: PixelPoint, viewport: PixelSize) -> bool {
    is_visible(
        PixelPoint::new(0, 0),
        PixelRect::new(origin.x, origin.y + 1, 5, 3),
        viewport,
    ) || [(1, 0), (2, 0), (3, 0), (1, 4), (3, 4)]
        .into_iter()
        .any(|(x, y)| {
            is_visible(
                PixelPoint::new(0, 0),
                PixelRect::new(origin.x + x, origin.y + y, 1, 1),
                viewport,
            )
        })
}

fn paint_token(
    target: &mut RgbBuffer,
    anchor: PixelPoint,
    accent: Rgb,
    unknown: bool,
    animation: u8,
) {
    let anchor = PixelPoint::new(anchor.x, anchor.y - i32::from(animation == 1));
    target.fill_rect(PixelRect::new(anchor.x, anchor.y + 1, 5, 3), accent);
    target.put(anchor.x + 1, anchor.y, PARCHMENT_LIGHT);
    target.put(
        anchor.x + 2,
        anchor.y,
        if unknown { SHADOW } else { accent },
    );
    target.put(anchor.x + 3, anchor.y, PARCHMENT_LIGHT);
    target.put(anchor.x + 1, anchor.y + 4, BRASS_DARK);
    target.put(anchor.x + 3, anchor.y + 4, BRASS_DARK);
}

fn actor_anchors<'a>(
    snapshot: &SceneSnapshot,
    plan: &'a ScenePlan,
) -> Vec<(&'a ActorPlacement, PixelPoint)> {
    let mut station_counts = HashMap::<String, usize>::new();
    let mut result = Vec::with_capacity(plan.actors.len());
    for placement in &plan.actors {
        let key = station_key(snapshot, &placement.station);
        let slot = *station_counts
            .entry(key)
            .and_modify(|n| *n += 1)
            .or_insert(0);
        let anchor = match &placement.station {
            TruthfulStation::CampaignToken(workspace) => {
                let base_x = campaign_table(snapshot, workspace).actor_base_x();
                PixelPoint::new(
                    base_x + 4 + i32::try_from(slot % 4).unwrap_or(0) * 6,
                    56 + i32::try_from(slot / 4).unwrap_or(0) * 6,
                )
            }
            TruthfulStation::CounselBell => PixelPoint::new(
                113 + i32::try_from(slot % 2).unwrap_or(0) * 9,
                43 - i32::try_from(slot / 2).unwrap_or(0) * 3,
            ),
            TruthfulStation::SpoilsBench => PixelPoint::new(
                112 + i32::try_from(slot % 5).unwrap_or(0) * 9,
                69 - i32::try_from(slot / 5).unwrap_or(0) * 3,
            ),
            TruthfulStation::Hearth => PixelPoint::new(
                132 + i32::try_from(slot % 3).unwrap_or(0) * 9,
                47 - i32::try_from(slot / 3).unwrap_or(0) * 3,
            ),
            TruthfulStation::DelveActive(_)
            | TruthfulStation::DelveGate(_)
            | TruthfulStation::DelveExit(_)
            | TruthfulStation::DelveCamp(_) => continue,
        };
        result.push((placement, anchor));
    }
    result
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
