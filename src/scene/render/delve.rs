use std::{collections::HashMap, time::Duration};

use crate::{
    app::Motion,
    domain::{Presence, Timestamp},
    scene::{
        SceneFrame,
        assets::{
            adventurer::compact_adventurer_animation_frame,
            delve::{
                CHEST_GOLD, DEEP_BLUE_BLACK, DelveAsset, FLOOR_DARK, FLOOR_MID, MINERAL_VIOLET,
                MOSS_DARK, MOSS_LIGHT, STONE_DARK, STONE_LIGHT, STONE_MID, TEAL_GLOW, TEAL_LIGHT,
                TORCH_AMBER, frame,
            },
        },
        pixel::{PixelPoint, PixelRect, PixelSize, Rgb, RgbBuffer},
        snapshot::{SceneAgent, SceneConnection, SceneSnapshot},
        sprite::{SpriteFrame, blit},
        stage::{
            ActorPlacement, CameraAnchor, SceneCadence, SceneCamera, SceneEffect, ScenePlan,
            ScenePose, TruthfulStation,
        },
    },
};

use super::lighting;

pub const WIDTH: i32 = 160;
pub const HEIGHT: i32 = 90;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NamedRegion {
    pub name: &'static str,
    pub bounds: PixelRect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NamedDoorway {
    pub name: &'static str,
    pub point: PixelPoint,
}

const ENTRANCE: PixelRect = PixelRect::new(4, 31, 46, 25);
const ENTRANCE_PASSAGE: PixelRect = PixelRect::new(44, 38, 18, 10);
const WEST_CHAMBER: PixelRect = PixelRect::new(10, 7, 47, 28);
const WEST_PASSAGE: PixelRect = PixelRect::new(49, 25, 15, 10);
const CENTRAL_JUNCTION: PixelRect = PixelRect::new(58, 24, 48, 42);
const EAST_CHAMBER: PixelRect = PixelRect::new(105, 8, 45, 31);
const DESCENDING_CORRIDOR: PixelRect = PixelRect::new(76, 60, 18, 27);
const CAMP: PixelRect = PixelRect::new(10, 57, 52, 29);
const CAMP_PASSAGE: PixelRect = PixelRect::new(55, 66, 25, 12);
const EXIT_LANDING: PixelRect = PixelRect::new(106, 55, 45, 31);
const EXIT_PASSAGE: PixelRect = PixelRect::new(90, 67, 22, 11);

const FLOOR_REGIONS: &[PixelRect] = &[
    ENTRANCE,
    ENTRANCE_PASSAGE,
    WEST_CHAMBER,
    WEST_PASSAGE,
    CENTRAL_JUNCTION,
    EAST_CHAMBER,
    DESCENDING_CORRIDOR,
    CAMP,
    CAMP_PASSAGE,
    EXIT_LANDING,
    EXIT_PASSAGE,
];

pub const ARCHITECTURE_REGIONS: &[NamedRegion] = &[
    NamedRegion {
        name: "entrance",
        bounds: ENTRANCE,
    },
    NamedRegion {
        name: "west chamber",
        bounds: WEST_CHAMBER,
    },
    NamedRegion {
        name: "central junction",
        bounds: CENTRAL_JUNCTION,
    },
    NamedRegion {
        name: "east chamber",
        bounds: EAST_CHAMBER,
    },
    NamedRegion {
        name: "descending corridor",
        bounds: DESCENDING_CORRIDOR,
    },
    NamedRegion {
        name: "camp",
        bounds: CAMP,
    },
    NamedRegion {
        name: "exit landing",
        bounds: EXIT_LANDING,
    },
];

pub const DOORWAYS: &[NamedDoorway] = &[
    NamedDoorway {
        name: "dungeon entrance",
        point: PixelPoint::new(6, 43),
    },
    NamedDoorway {
        name: "entrance to junction",
        point: PixelPoint::new(59, 42),
    },
    NamedDoorway {
        name: "west chamber arch",
        point: PixelPoint::new(55, 30),
    },
    NamedDoorway {
        name: "east chamber arch",
        point: PixelPoint::new(105, 32),
    },
    NamedDoorway {
        name: "descending threshold",
        point: PixelPoint::new(83, 62),
    },
    NamedDoorway {
        name: "camp passage",
        point: PixelPoint::new(59, 71),
    },
    NamedDoorway {
        name: "exit passage",
        point: PixelPoint::new(108, 71),
    },
];

#[must_use]
pub fn is_walkable(x: i32, y: i32) -> bool {
    FLOOR_REGIONS.iter().any(|region| contains(*region, x, y))
}

#[must_use]
pub fn station_region(station: &TruthfulStation, pose: ScenePose) -> PixelRect {
    match station {
        TruthfulStation::DelveActive(_) if pose == ScenePose::Unknown => {
            PixelRect::new(18, 10, 31, 23)
        }
        TruthfulStation::DelveActive(_) => PixelRect::new(62, 33, 38, 28),
        TruthfulStation::DelveGate(_) => PixelRect::new(108, 31, 36, 31),
        TruthfulStation::DelveExit(_) => PixelRect::new(117, 59, 31, 27),
        TruthfulStation::DelveCamp(_) => PixelRect::new(19, 62, 38, 24),
        TruthfulStation::CampaignToken(_)
        | TruthfulStation::CounselBell
        | TruthfulStation::SpoilsBench
        | TruthfulStation::Hearth => PixelRect::new(58, 24, 48, 42),
    }
}

pub fn paint(
    snapshot: &SceneSnapshot,
    plan: &ScenePlan,
    viewport: PixelSize,
    target: &mut RgbBuffer,
) -> SceneFrame {
    target.ensure_size(viewport.width, viewport.height, DEEP_BLUE_BLACK);
    let origin = dungeon_origin(snapshot, plan, viewport);
    let seed = dungeon_seed(snapshot);

    paint_materials(target, origin, seed);
    paint_background_architecture(target, origin, seed);
    apply_lighting(snapshot, target, origin);
    paint_depth_sorted(snapshot, plan, target, origin);
    paint_effects(snapshot, plan, target, origin);
    paint_connection_fact(snapshot, target, origin);

    SceneFrame {
        world: plan.world,
        next_frame_in: next_frame(plan),
    }
}

fn dungeon_origin(snapshot: &SceneSnapshot, plan: &ScenePlan, viewport: PixelSize) -> PixelPoint {
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
        -(focus.y - height / 2).clamp(0, HEIGHT - height)
    };
    PixelPoint::new(x, y)
}

fn focus_point(snapshot: &SceneSnapshot, anchor: &CameraAnchor) -> PixelPoint {
    match anchor {
        CameraAnchor::DelveParty(workspace) => snapshot
            .agents
            .iter()
            .find(|agent| agent.workspace_id == *workspace && agent.presence != Presence::Exited)
            .map_or(PixelPoint::new(24, 43), presence_focus),
        CameraAnchor::Door => PixelPoint::new(24, 43),
        CameraAnchor::CampaignTable(_)
        | CameraAnchor::CounselBell
        | CameraAnchor::Hearth
        | CameraAnchor::Spoils => PixelPoint::new(82, 45),
    }
}

const fn presence_focus(agent: &SceneAgent) -> PixelPoint {
    match agent.presence {
        Presence::Working => PixelPoint::new(81, 47),
        Presence::Blocked => PixelPoint::new(126, 48),
        Presence::Done => PixelPoint::new(132, 72),
        Presence::Idle => PixelPoint::new(37, 73),
        Presence::Unknown => PixelPoint::new(33, 21),
        Presence::Exited => PixelPoint::new(24, 43),
    }
}

fn paint_materials(target: &mut RgbBuffer, origin: PixelPoint, seed: u64) {
    for y in 0..target.size().height {
        for x in 0..target.size().width {
            let world_x = i32::from(x).saturating_sub(origin.x);
            let world_y = i32::from(y).saturating_sub(origin.y);
            let variant = material_variant(world_x, world_y, seed);
            let colour = if is_walkable(world_x, world_y) {
                if variant.is_multiple_of(61) {
                    MINERAL_VIOLET
                } else if variant.is_multiple_of(29) {
                    MOSS_LIGHT
                } else if variant.is_multiple_of(11) {
                    MOSS_DARK
                } else if (world_x + world_y).rem_euclid(9) == 0 {
                    FLOOR_DARK
                } else {
                    FLOOR_MID
                }
            } else {
                let course = world_y.div_euclid(6);
                let stagger = if course.rem_euclid(2) == 0 { 0 } else { 7 };
                if world_y.rem_euclid(6) == 0 || (world_x + stagger).rem_euclid(14) == 0 {
                    STONE_DARK
                } else if variant.is_multiple_of(43) {
                    MOSS_DARK
                } else if variant.is_multiple_of(7) {
                    STONE_LIGHT
                } else {
                    STONE_MID
                }
            };
            target.put(i32::from(x), i32::from(y), colour);
        }
    }
}

fn paint_background_architecture(target: &mut RgbBuffer, origin: PixelPoint, seed: u64) {
    for (x, y) in [(3, 1), (31, 1), (61, 1), (91, 1), (121, 1), (145, 1)] {
        blit_asset(DelveAsset::DressedStoneWall, target, origin, x, y);
    }
    blit_asset(DelveAsset::Door, target, origin, 1, 38);
    blit_asset(DelveAsset::ActivePassage, target, origin, 68, 31);
    blit_asset(DelveAsset::SealedGate, target, origin, 127, 28);
    blit_asset(DelveAsset::DescendingStair, target, origin, 77, 76);
    blit_asset(DelveAsset::ExitLanding, target, origin, 128, 76);
    blit_asset(DelveAsset::Camp, target, origin, 25, 70);
    blit_asset(DelveAsset::RuneStones, target, origin, 24, 14);
    blit_asset(DelveAsset::Roots, target, origin, 12, 8);
    blit_asset(DelveAsset::Roots, target, origin, 143, 12);
    blit_asset(DelveAsset::Puddles, target, origin, 111, 17);
    blit_asset(DelveAsset::Bones, target, origin, 16, 49);
    blit_asset(DelveAsset::Chests, target, origin, 134, 61);
    blit_asset(DelveAsset::DungeonClutter, target, origin, 13, 76);
    blit_asset(DelveAsset::Torch, target, origin, 52, 18);
    blit_asset(DelveAsset::Torch, target, origin, 105, 16);
    if seed.is_multiple_of(2) {
        blit_asset(DelveAsset::Rubble, target, origin, 44, 52);
    } else {
        blit_asset(DelveAsset::Rubble, target, origin, 96, 53);
    }
}

fn apply_lighting(snapshot: &SceneSnapshot, target: &mut RgbBuffer, origin: PixelPoint) {
    lighting::apply_cool_ambient(target, 20);
    lighting::apply_cool_pool(target, translate(origin, 31, 19), 35, 25);
    lighting::apply_cool_pool(target, translate(origin, 82, 44), 52, 18);
    lighting::apply_cool_pool(target, translate(origin, 135, 72), 30, 16);
    lighting::apply_warm_pool(target, translate(origin, 52, 22), 18, 15);
    lighting::apply_warm_pool(target, translate(origin, 108, 20), 18, 15);
    lighting::apply_warm_pool(target, translate(origin, 32, 73), 20, 18);
    match snapshot.connection {
        SceneConnection::Connected => {}
        SceneConnection::Connecting => lighting::dim(target, 20),
        SceneConnection::Reconnecting { .. } => lighting::dim(target, 28),
        SceneConnection::Offline => lighting::dim(target, 48),
        SceneConnection::Incompatible { .. } => lighting::dim(target, 12),
    }
}

#[derive(Clone, Copy)]
enum DepthItem<'a> {
    Actor {
        placement: &'a ActorPlacement,
        anchor: PixelPoint,
    },
    Asset {
        asset: DelveAsset,
        anchor: PixelPoint,
    },
}

impl DepthItem<'_> {
    fn foot_row(self) -> i32 {
        match self {
            Self::Actor { anchor, .. } => anchor.y + 13,
            Self::Asset { asset, anchor } => anchor.y + i32::from(frame(asset).size().height) - 1,
        }
    }
}

fn paint_depth_sorted(
    snapshot: &SceneSnapshot,
    plan: &ScenePlan,
    target: &mut RgbBuffer,
    origin: PixelPoint,
) {
    let mut items = actor_anchors(plan)
        .into_iter()
        .map(|(placement, anchor)| DepthItem::Actor { placement, anchor })
        .collect::<Vec<_>>();
    items.extend([
        DepthItem::Asset {
            asset: DelveAsset::Arch,
            anchor: PixelPoint::new(50, 29),
        },
        DepthItem::Asset {
            asset: DelveAsset::Arch,
            anchor: PixelPoint::new(100, 30),
        },
        DepthItem::Asset {
            asset: DelveAsset::Columns,
            anchor: PixelPoint::new(58, 42),
        },
        DepthItem::Asset {
            asset: DelveAsset::Columns,
            anchor: PixelPoint::new(98, 42),
        },
        DepthItem::Asset {
            asset: DelveAsset::Brazier,
            anchor: PixelPoint::new(70, 54),
        },
        DepthItem::Asset {
            asset: DelveAsset::Rubble,
            anchor: PixelPoint::new(65, 52),
        },
    ]);
    items.sort_by_key(|item| item.foot_row());

    for item in items {
        match item {
            DepthItem::Actor { placement, anchor } => {
                let Some(agent) = snapshot
                    .agents
                    .iter()
                    .find(|agent| agent.key == placement.agent)
                else {
                    continue;
                };
                let animation =
                    actor_animation_frame(snapshot.motion, snapshot.now, agent, placement);
                let sprite =
                    compact_adventurer_animation_frame(&agent.persona, placement.pose, animation);
                if placement.pose == ScenePose::Unknown {
                    blit_dimmed(&sprite, translate(origin, anchor.x, anchor.y), target);
                } else {
                    blit(&sprite, translate(origin, anchor.x, anchor.y), target);
                }
            }
            DepthItem::Asset { asset, anchor } => {
                blit_asset(asset, target, origin, anchor.x, anchor.y);
            }
        }
    }
}

fn actor_anchors(plan: &ScenePlan) -> Vec<(&ActorPlacement, PixelPoint)> {
    let mut counts = HashMap::<String, usize>::new();
    plan.actors
        .iter()
        .filter_map(|placement| {
            let (key, base) = match &placement.station {
                TruthfulStation::DelveActive(workspace) if placement.pose == ScenePose::Unknown => {
                    (format!("unknown:{workspace}"), PixelPoint::new(24, 15))
                }
                TruthfulStation::DelveActive(workspace) => {
                    (format!("active:{workspace}"), PixelPoint::new(66, 39))
                }
                TruthfulStation::DelveGate(workspace) => {
                    (format!("gate:{workspace}"), PixelPoint::new(112, 38))
                }
                TruthfulStation::DelveExit(workspace) => {
                    (format!("exit:{workspace}"), PixelPoint::new(122, 62))
                }
                TruthfulStation::DelveCamp(workspace) => {
                    (format!("camp:{workspace}"), PixelPoint::new(21, 65))
                }
                TruthfulStation::CampaignToken(_)
                | TruthfulStation::CounselBell
                | TruthfulStation::SpoilsBench
                | TruthfulStation::Hearth => return None,
            };
            let slot = *counts
                .entry(key)
                .and_modify(|value| *value += 1)
                .or_insert(0);
            let column = i32::try_from(slot % 4).unwrap_or(0);
            let row = i32::try_from(slot / 4).unwrap_or(0);
            Some((
                placement,
                PixelPoint::new(base.x + column * 9, base.y + row * 3),
            ))
        })
        .collect()
}

fn actor_animation_frame(
    motion: Motion,
    now: Timestamp,
    agent: &SceneAgent,
    placement: &ActorPlacement,
) -> u8 {
    let elapsed = agent.presence_since.elapsed_until(now).as_millis();
    let frame = match motion {
        Motion::Reduced if placement.pose == ScenePose::Resting => elapsed / 1_000,
        Motion::None | Motion::Reduced => 0,
        Motion::Full => elapsed / 125,
    };
    u8::try_from(frame % 3).unwrap_or(0)
}

fn blit_dimmed(frame: &SpriteFrame, origin: PixelPoint, target: &mut RgbBuffer) {
    let pixels = frame
        .pixels()
        .iter()
        .map(|pixel| pixel.map(|colour| Rgb::new(colour.r / 2, colour.g / 2, colour.b / 2)))
        .collect();
    let dimmed = SpriteFrame::from_pixels(frame.size().width, frame.size().height, pixels);
    blit(&dimmed, origin, target);
}

fn paint_effects(
    snapshot: &SceneSnapshot,
    plan: &ScenePlan,
    target: &mut RgbBuffer,
    origin: PixelPoint,
) {
    for effect in &plan.effects {
        match effect {
            SceneEffect::FreshSpoils { agent, since } => {
                let phase = since.elapsed_until(snapshot.now).as_millis() / 125;
                let seed =
                    stable_hash(agent.as_str().as_bytes()) ^ u64::try_from(phase).unwrap_or(0);
                for index in 0..10_u64 {
                    let value = seed.rotate_left(u32::try_from(index * 5).unwrap_or(0));
                    put(
                        target,
                        origin,
                        119 + i32::try_from(value % 28).unwrap_or(0),
                        57 + i32::try_from(value.rotate_left(9) % 25).unwrap_or(0),
                        if index.is_multiple_of(2) {
                            TEAL_LIGHT
                        } else {
                            CHEST_GOLD
                        },
                    );
                }
            }
            SceneEffect::RecentDeparture { workspace_id, .. } => {
                let offset =
                    i32::try_from(stable_hash(workspace_id.as_str().as_bytes()) % 5).unwrap_or(0);
                put(target, origin, 5 + offset, 49, TEAL_GLOW);
            }
        }
    }
}

fn paint_connection_fact(snapshot: &SceneSnapshot, target: &mut RgbBuffer, origin: PixelPoint) {
    match snapshot.connection {
        SceneConnection::Connecting => put(target, origin, 7, 47, TORCH_AMBER),
        SceneConnection::Reconnecting { attempt } => {
            for index in 0..attempt.clamp(1, 6) {
                put(
                    target,
                    origin,
                    4 + i32::try_from(index).unwrap_or(0) * 2,
                    50,
                    TEAL_LIGHT,
                );
            }
        }
        SceneConnection::Incompatible { expected, actual } => {
            for index in 0..expected.min(7) {
                put(
                    target,
                    origin,
                    5 + i32::try_from(index * 2).unwrap_or(0),
                    34,
                    MINERAL_VIOLET,
                );
            }
            for index in 0..actual.min(7) {
                put(
                    target,
                    origin,
                    5 + i32::try_from(index * 2).unwrap_or(0),
                    36,
                    TEAL_GLOW,
                );
            }
        }
        SceneConnection::Offline | SceneConnection::Connected => {}
    }
}

fn next_frame(plan: &ScenePlan) -> Option<Duration> {
    match plan.cadence {
        SceneCadence::EventDriven => None,
        SceneCadence::Fps(fps) => Some(Duration::from_millis(1_000 / u64::from(fps.max(1)))),
    }
}

fn material_variant(x: i32, y: i32, seed: u64) -> u64 {
    seed.rotate_left(13)
        ^ u64::from(x.unsigned_abs()).wrapping_mul(0x9e37_79b9)
        ^ u64::from(y.unsigned_abs()).wrapping_mul(0x85eb_ca6b)
}

fn dungeon_seed(snapshot: &SceneSnapshot) -> u64 {
    snapshot
        .campaigns
        .iter()
        .fold(0x00d3_116e_u64, |seed, campaign| {
            seed.rotate_left(7) ^ campaign.variant_seed
        })
}

fn stable_hash(bytes: &[u8]) -> u64 {
    u64::from_le_bytes(
        blake3::hash(bytes).as_bytes()[..8]
            .try_into()
            .expect("digest has eight bytes"),
    )
}

fn blit_asset(asset: DelveAsset, target: &mut RgbBuffer, origin: PixelPoint, x: i32, y: i32) {
    blit(frame(asset), translate(origin, x, y), target);
}

fn put(target: &mut RgbBuffer, origin: PixelPoint, x: i32, y: i32, colour: Rgb) {
    let point = translate(origin, x, y);
    target.put(point.x, point.y, colour);
}

const fn contains(rect: PixelRect, x: i32, y: i32) -> bool {
    x >= rect.x && y >= rect.y && x < rect.x + rect.width as i32 && y < rect.y + rect.height as i32
}

const fn translate(origin: PixelPoint, x: i32, y: i32) -> PixelPoint {
    PixelPoint::new(origin.x.saturating_add(x), origin.y.saturating_add(y))
}
