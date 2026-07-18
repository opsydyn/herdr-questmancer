use std::{collections::HashMap, sync::OnceLock, time::Duration};

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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ArchitectureBackground {
    ConnectedDungeon,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WallOwner {
    SharedDungeonShell,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WallEdge {
    North,
    East,
    South,
    West,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct WallSegment {
    pub x: i32,
    pub y: i32,
    pub edge: WallEdge,
    pub owner: WallOwner,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelveArchitectureMask {
    walkable: Vec<bool>,
    wall_segments: Vec<WallSegment>,
}

impl DelveArchitectureMask {
    #[must_use]
    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        architecture_index(x, y).is_some_and(|index| self.walkable[index])
    }

    #[must_use]
    pub const fn background_at(&self, x: i32, y: i32) -> Option<ArchitectureBackground> {
        if x >= 0 && y >= 0 && x < WIDTH && y < HEIGHT {
            Some(ArchitectureBackground::ConnectedDungeon)
        } else {
            None
        }
    }

    #[must_use]
    pub fn wall_segments(&self) -> &[WallSegment] {
        &self.wall_segments
    }
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
        point: PixelPoint::new(13, 43),
    },
    NamedDoorway {
        name: "entrance to junction",
        point: PixelPoint::new(57, 42),
    },
    NamedDoorway {
        name: "west chamber arch",
        point: PixelPoint::new(55, 34),
    },
    NamedDoorway {
        name: "east chamber arch",
        point: PixelPoint::new(105, 35),
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
    static DEFAULT_ARCHITECTURE: OnceLock<DelveArchitectureMask> = OnceLock::new();
    DEFAULT_ARCHITECTURE
        .get_or_init(|| compose_architecture(0x00d3_116e_u64))
        .is_walkable(x, y)
}

#[must_use]
pub fn architecture_mask(snapshot: &SceneSnapshot) -> DelveArchitectureMask {
    compose_architecture(dungeon_seed(snapshot))
}

#[must_use]
pub fn station_region(station: &TruthfulStation, pose: ScenePose) -> PixelRect {
    match station {
        TruthfulStation::DelveActive(_) if pose == ScenePose::Unknown => WEST_CHAMBER,
        TruthfulStation::DelveActive(_) => CENTRAL_JUNCTION,
        TruthfulStation::DelveGate(_) => PixelRect::new(106, 24, 45, 29),
        TruthfulStation::DelveExit(_) => EXIT_LANDING,
        TruthfulStation::DelveCamp(_) => CAMP,
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
    let architecture = architecture_mask(snapshot);

    paint_materials(target, origin, seed, &architecture);
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
        CameraAnchor::DelveParty(agent_key) => snapshot
            .agents
            .iter()
            .find(|agent| agent.key == *agent_key && agent.presence != Presence::Exited)
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

fn paint_materials(
    target: &mut RgbBuffer,
    origin: PixelPoint,
    seed: u64,
    architecture: &DelveArchitectureMask,
) {
    for y in 0..target.size().height {
        for x in 0..target.size().width {
            let world_x = i32::from(x).saturating_sub(origin.x);
            let world_y = i32::from(y).saturating_sub(origin.y);
            let variant = material_variant(world_x, world_y, seed);
            let colour = if architecture.is_walkable(world_x, world_y) {
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

#[derive(Clone, Copy)]
struct PlacedAsset {
    asset: DelveAsset,
    anchor: PixelPoint,
    blocks_walkable: bool,
}

const fn placed(asset: DelveAsset, x: i32, y: i32, blocks_walkable: bool) -> PlacedAsset {
    PlacedAsset {
        asset,
        anchor: PixelPoint::new(x, y),
        blocks_walkable,
    }
}

fn background_assets(seed: u64) -> Vec<PlacedAsset> {
    let mut assets = vec![
        placed(DelveAsset::DressedStoneWall, 3, 1, true),
        placed(DelveAsset::DressedStoneWall, 31, 1, true),
        placed(DelveAsset::DressedStoneWall, 61, 1, true),
        placed(DelveAsset::DressedStoneWall, 91, 1, true),
        placed(DelveAsset::DressedStoneWall, 121, 1, true),
        placed(DelveAsset::DressedStoneWall, 145, 1, true),
        placed(DelveAsset::Door, 1, 38, true),
        placed(DelveAsset::ActivePassage, 68, 31, false),
        placed(DelveAsset::SealedGate, 127, 28, true),
        placed(DelveAsset::DescendingStair, 77, 76, false),
        placed(DelveAsset::ExitLanding, 128, 76, false),
        placed(DelveAsset::Camp, 25, 70, false),
        placed(DelveAsset::RuneStones, 24, 14, false),
        placed(DelveAsset::Roots, 12, 8, false),
        placed(DelveAsset::Roots, 143, 12, false),
        placed(DelveAsset::Puddles, 111, 17, false),
        placed(DelveAsset::Bones, 16, 49, false),
        placed(DelveAsset::Chests, 134, 61, false),
        placed(DelveAsset::DungeonClutter, 13, 76, false),
        placed(DelveAsset::Torch, 63, 18, false),
        placed(DelveAsset::Torch, 105, 16, false),
    ];
    assets.push(if seed.is_multiple_of(2) {
        placed(DelveAsset::Rubble, 44, 52, false)
    } else {
        placed(DelveAsset::Rubble, 96, 53, false)
    });
    assets
}

const FOREGROUND_ASSETS: &[PlacedAsset] = &[
    placed(DelveAsset::Arch, 50, 29, true),
    placed(DelveAsset::Arch, 100, 30, true),
    placed(DelveAsset::Columns, 58, 42, false),
    placed(DelveAsset::Columns, 98, 42, false),
    placed(DelveAsset::Brazier, 70, 54, false),
    placed(DelveAsset::Rubble, 65, 52, false),
];

fn paint_background_architecture(target: &mut RgbBuffer, origin: PixelPoint, seed: u64) {
    for placed in background_assets(seed) {
        blit_asset(
            placed.asset,
            target,
            origin,
            placed.anchor.x,
            placed.anchor.y,
        );
    }
}

fn compose_architecture(seed: u64) -> DelveArchitectureMask {
    let mut walkable = (0..HEIGHT)
        .flat_map(|y| (0..WIDTH).map(move |x| base_walkable(x, y)))
        .collect::<Vec<_>>();
    for placed in background_assets(seed)
        .into_iter()
        .chain(FOREGROUND_ASSETS.iter().copied())
        .filter(|placed| placed.blocks_walkable)
    {
        let sprite = frame(placed.asset);
        for (index, pixel) in sprite.pixels().iter().enumerate() {
            let local_x = i32::try_from(index % usize::from(sprite.size().width)).unwrap_or(0);
            let local_y = i32::try_from(index / usize::from(sprite.size().width)).unwrap_or(0);
            let blocks = if placed.asset == DelveAsset::Arch {
                local_y < 3 || !(3..=10).contains(&local_x)
            } else {
                pixel.is_some()
            };
            if !blocks {
                continue;
            }
            let x = placed.anchor.x + local_x;
            let y = placed.anchor.y + local_y;
            if let Some(index) = architecture_index(x, y) {
                walkable[index] = false;
            }
        }
    }

    let mut wall_segments = Vec::new();
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let Some(index) = architecture_index(x, y) else {
                continue;
            };
            if !walkable[index] {
                continue;
            }
            for (next_x, next_y, edge) in [
                (x, y - 1, WallEdge::North),
                (x + 1, y, WallEdge::East),
                (x, y + 1, WallEdge::South),
                (x - 1, y, WallEdge::West),
            ] {
                if architecture_index(next_x, next_y).is_none_or(|next| !walkable[next]) {
                    wall_segments.push(WallSegment {
                        x,
                        y,
                        edge,
                        owner: WallOwner::SharedDungeonShell,
                    });
                }
            }
        }
    }
    DelveArchitectureMask {
        walkable,
        wall_segments,
    }
}

fn architecture_index(x: i32, y: i32) -> Option<usize> {
    if x < 0 || y < 0 || x >= WIDTH || y >= HEIGHT {
        return None;
    }
    Some(
        usize::try_from(y).expect("non-negative y fits usize")
            * usize::try_from(WIDTH).expect("positive width fits usize")
            + usize::try_from(x).expect("non-negative x fits usize"),
    )
}

fn base_walkable(x: i32, y: i32) -> bool {
    FLOOR_REGIONS.iter().any(|region| contains(*region, x, y))
}

fn apply_lighting(snapshot: &SceneSnapshot, target: &mut RgbBuffer, origin: PixelPoint) {
    lighting::apply_cool_ambient(target, 20);
    lighting::apply_cool_pool(target, translate(origin, 82, 44), 36, 18);
    lighting::apply_cool_pool(target, translate(origin, 135, 72), 30, 16);
    lighting::apply_warm_pool(target, translate(origin, 66, 22), 10, 15);
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
    Overflow {
        count: usize,
        anchor: PixelPoint,
    },
}

impl DepthItem<'_> {
    fn foot_row(self) -> i32 {
        match self {
            Self::Actor { anchor, .. } | Self::Overflow { anchor, .. } => anchor.y + 13,
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
    items.extend(FOREGROUND_ASSETS.iter().map(|placed| DepthItem::Asset {
        asset: placed.asset,
        anchor: placed.anchor,
    }));
    items.extend(
        overflow_markers(plan)
            .into_iter()
            .map(|(count, anchor)| DepthItem::Overflow { count, anchor }),
    );
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
            DepthItem::Overflow { count, anchor } => {
                paint_overflow_marker(target, origin, anchor, count);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum StationKind {
    Active,
    Gate,
    Exit,
    Camp,
    Unknown,
}

const VISIBLE_ACTORS_PER_STATION: usize = 8;
const ACTIVE_SLOTS: &[PixelPoint] = &[
    PixelPoint::new(60, 26),
    PixelPoint::new(69, 26),
    PixelPoint::new(78, 26),
    PixelPoint::new(87, 26),
    PixelPoint::new(96, 26),
    PixelPoint::new(60, 42),
    PixelPoint::new(69, 42),
    PixelPoint::new(78, 42),
    PixelPoint::new(87, 42),
];
const GATE_SLOTS: &[PixelPoint] = &[
    PixelPoint::new(107, 24),
    PixelPoint::new(116, 24),
    PixelPoint::new(125, 24),
    PixelPoint::new(134, 24),
    PixelPoint::new(143, 24),
    PixelPoint::new(107, 39),
    PixelPoint::new(116, 39),
    PixelPoint::new(125, 39),
    PixelPoint::new(134, 39),
];
const EXIT_SLOTS: &[PixelPoint] = &[
    PixelPoint::new(108, 56),
    PixelPoint::new(117, 56),
    PixelPoint::new(126, 56),
    PixelPoint::new(135, 56),
    PixelPoint::new(144, 56),
    PixelPoint::new(108, 71),
    PixelPoint::new(117, 71),
    PixelPoint::new(126, 71),
    PixelPoint::new(135, 71),
];
const CAMP_SLOTS: &[PixelPoint] = &[
    PixelPoint::new(11, 58),
    PixelPoint::new(20, 58),
    PixelPoint::new(29, 58),
    PixelPoint::new(38, 58),
    PixelPoint::new(47, 58),
    PixelPoint::new(11, 72),
    PixelPoint::new(20, 72),
    PixelPoint::new(29, 72),
    PixelPoint::new(38, 72),
];
const UNKNOWN_SLOTS: &[PixelPoint] = &[
    PixelPoint::new(11, 7),
    PixelPoint::new(20, 7),
    PixelPoint::new(29, 7),
    PixelPoint::new(38, 7),
    PixelPoint::new(47, 7),
    PixelPoint::new(11, 21),
    PixelPoint::new(20, 21),
    PixelPoint::new(29, 21),
    PixelPoint::new(38, 21),
];

fn actor_anchors(plan: &ScenePlan) -> Vec<(&ActorPlacement, PixelPoint)> {
    let mut counts = HashMap::<StationKind, usize>::new();
    plan.actors
        .iter()
        .filter_map(|placement| {
            let kind = match &placement.station {
                TruthfulStation::DelveActive(workspace) if placement.pose == ScenePose::Unknown => {
                    let _ = workspace;
                    StationKind::Unknown
                }
                TruthfulStation::DelveActive(_) => StationKind::Active,
                TruthfulStation::DelveGate(_) => StationKind::Gate,
                TruthfulStation::DelveExit(_) => StationKind::Exit,
                TruthfulStation::DelveCamp(_) => StationKind::Camp,
                TruthfulStation::CampaignToken(_)
                | TruthfulStation::CounselBell
                | TruthfulStation::SpoilsBench
                | TruthfulStation::Hearth => return None,
            };
            let slot = *counts
                .entry(kind)
                .and_modify(|value| *value += 1)
                .or_insert(0);
            station_slots(kind)
                .get(slot)
                .copied()
                .filter(|_| slot < VISIBLE_ACTORS_PER_STATION)
                .map(|anchor| (placement, anchor))
        })
        .collect()
}

fn overflow_markers(plan: &ScenePlan) -> Vec<(usize, PixelPoint)> {
    let mut totals = HashMap::<StationKind, usize>::new();
    for placement in &plan.actors {
        if let Some(kind) = station_kind(placement) {
            *totals.entry(kind).or_default() += 1;
        }
    }
    let mut markers = totals
        .into_iter()
        .filter(|(_, count)| *count > VISIBLE_ACTORS_PER_STATION)
        .map(|(kind, count)| {
            (
                count - VISIBLE_ACTORS_PER_STATION,
                station_slots(kind)[VISIBLE_ACTORS_PER_STATION],
            )
        })
        .collect::<Vec<_>>();
    markers.sort_by_key(|(_, anchor)| (anchor.y, anchor.x));
    markers
}

fn station_kind(placement: &ActorPlacement) -> Option<StationKind> {
    match &placement.station {
        TruthfulStation::DelveActive(_) if placement.pose == ScenePose::Unknown => {
            Some(StationKind::Unknown)
        }
        TruthfulStation::DelveActive(_) => Some(StationKind::Active),
        TruthfulStation::DelveGate(_) => Some(StationKind::Gate),
        TruthfulStation::DelveExit(_) => Some(StationKind::Exit),
        TruthfulStation::DelveCamp(_) => Some(StationKind::Camp),
        TruthfulStation::CampaignToken(_)
        | TruthfulStation::CounselBell
        | TruthfulStation::SpoilsBench
        | TruthfulStation::Hearth => None,
    }
}

const fn station_slots(kind: StationKind) -> &'static [PixelPoint] {
    match kind {
        StationKind::Active => ACTIVE_SLOTS,
        StationKind::Gate => GATE_SLOTS,
        StationKind::Exit => EXIT_SLOTS,
        StationKind::Camp => CAMP_SLOTS,
        StationKind::Unknown => UNKNOWN_SLOTS,
    }
}

fn paint_overflow_marker(
    target: &mut RgbBuffer,
    origin: PixelPoint,
    anchor: PixelPoint,
    count: usize,
) {
    for y in 0..14 {
        for x in 0..8 {
            if x == 0 || x == 7 || y == 0 || y == 13 {
                put(target, origin, anchor.x + x, anchor.y + y, STONE_LIGHT);
            }
        }
    }
    for index in 0..count.min(18) {
        let x = 2 + i32::try_from(index % 4).unwrap_or(0);
        let y = 3 + i32::try_from(index / 4).unwrap_or(0) * 2;
        put(target, origin, anchor.x + x, anchor.y + y, TEAL_LIGHT);
    }
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{
        app::Motion,
        domain::{AdventurerPersona, AgentKey, PersonaKey, WorkspaceId},
        scene::{
            snapshot::{SceneConnection, SceneSnapshot},
            stage::{ActorPlacement, SceneCadence, SceneCamera, ScenePlan, WorldScene},
        },
    };

    use super::*;

    #[test]
    fn maximum_station_parties_have_unique_bounded_sprite_anchors() {
        let stations = [
            ("active", ScenePose::Working),
            ("gate", ScenePose::SeekingCounsel),
            ("exit", ScenePose::Settled),
            ("camp", ScenePose::Resting),
            ("unknown", ScenePose::Unknown),
        ];
        let actors = stations
            .into_iter()
            .flat_map(|(station_name, pose)| {
                (0..8).map(move |index| ActorPlacement {
                    agent: AgentKey::new(format!("{station_name}-{index}")),
                    station: match station_name {
                        "active" | "unknown" => {
                            TruthfulStation::DelveActive(WorkspaceId::new("shared"))
                        }
                        "gate" => TruthfulStation::DelveGate(WorkspaceId::new("shared")),
                        "exit" => TruthfulStation::DelveExit(WorkspaceId::new("shared")),
                        "camp" => TruthfulStation::DelveCamp(WorkspaceId::new("shared")),
                        _ => unreachable!(),
                    },
                    pose,
                    focused: false,
                })
            })
            .collect::<Vec<_>>();
        let plan = ScenePlan {
            world: WorldScene::Delve,
            camera: SceneCamera::WholeRoom,
            actors,
            effects: Vec::new(),
            cadence: SceneCadence::EventDriven,
        };
        let anchors = actor_anchors(&plan);
        let unique = anchors
            .iter()
            .map(|(_, anchor)| (anchor.x, anchor.y))
            .collect::<HashSet<_>>();

        assert_eq!(anchors.len(), 40);
        assert_eq!(
            unique.len(),
            anchors.len(),
            "actor anchors must not overlap"
        );
        let persona = AdventurerPersona::for_key(PersonaKey::new("slot-test"));
        for (placement, anchor) in anchors {
            let region = station_region(&placement.station, placement.pose);
            let sprite = compact_adventurer_animation_frame(&persona, placement.pose, 0);
            for (index, pixel) in sprite.pixels().iter().enumerate() {
                if pixel.is_none() {
                    continue;
                }
                let x = anchor.x + i32::try_from(index % 8).expect("sprite x fits i32");
                let y = anchor.y + i32::try_from(index / 8).expect("sprite y fits i32");
                assert!(
                    contains(region, x, y),
                    "{} at ({x}, {y}) escapes {region:?}",
                    placement.agent
                );
            }
        }
    }

    #[test]
    fn station_population_above_visible_capacity_uses_an_in_region_overflow_marker() {
        let actors = (0..9)
            .map(|index| ActorPlacement {
                agent: AgentKey::new(format!("active-{index}")),
                station: TruthfulStation::DelveActive(WorkspaceId::new("shared")),
                pose: ScenePose::Working,
                focused: false,
            })
            .collect();
        let plan = ScenePlan {
            world: WorldScene::Delve,
            camera: SceneCamera::WholeRoom,
            actors,
            effects: Vec::new(),
            cadence: SceneCadence::EventDriven,
        };

        assert_eq!(actor_anchors(&plan).len(), VISIBLE_ACTORS_PER_STATION);
        assert_eq!(overflow_markers(&plan), vec![(1, ACTIVE_SLOTS[8])]);
        let marker = ACTIVE_SLOTS[8];
        let region = station_region(
            &TruthfulStation::DelveActive(WorkspaceId::new("shared")),
            ScenePose::Working,
        );
        assert!(contains(region, marker.x, marker.y));
        assert!(contains(region, marker.x + 7, marker.y + 13));
    }

    #[test]
    fn unknown_station_is_ambient_only_while_active_station_has_a_local_pool() {
        let snapshot = SceneSnapshot {
            connection: SceneConnection::Connected,
            campaigns: Vec::new(),
            agents: Vec::new(),
            motion: Motion::None,
            now: Timestamp::from_millis(0),
        };
        let origin = PixelPoint::new(0, 0);
        let architecture = architecture_mask(&snapshot);
        let width = u16::try_from(WIDTH).expect("Delve width fits u16");
        let height = u16::try_from(HEIGHT).expect("Delve height fits u16");
        let mut ambient = RgbBuffer::filled(width, height, DEEP_BLUE_BLACK);
        paint_materials(&mut ambient, origin, dungeon_seed(&snapshot), &architecture);
        paint_background_architecture(&mut ambient, origin, dungeon_seed(&snapshot));
        lighting::apply_cool_ambient(&mut ambient, 20);
        let mut composed = RgbBuffer::filled(width, height, DEEP_BLUE_BLACK);
        paint_materials(
            &mut composed,
            origin,
            dungeon_seed(&snapshot),
            &architecture,
        );
        paint_background_architecture(&mut composed, origin, dungeon_seed(&snapshot));
        apply_lighting(&snapshot, &mut composed, origin);

        for anchor in UNKNOWN_SLOTS.iter().take(VISIBLE_ACTORS_PER_STATION) {
            for y in anchor.y..anchor.y + 14 {
                for x in anchor.x..anchor.x + 8 {
                    assert_eq!(
                        composed.get(x, y),
                        ambient.get(x, y),
                        "Unknown station pixel ({x}, {y}) received an explicit local pool"
                    );
                }
            }
        }
        assert!(
            ACTIVE_SLOTS
                .iter()
                .take(VISIBLE_ACTORS_PER_STATION)
                .any(|anchor| {
                    (anchor.y..anchor.y + 14).any(|y| {
                        (anchor.x..anchor.x + 8).any(|x| composed.get(x, y) != ambient.get(x, y))
                    })
                })
        );
    }

    #[test]
    fn final_architecture_painter_leaves_every_named_doorway_visually_open() {
        let snapshot = SceneSnapshot {
            connection: SceneConnection::Connected,
            campaigns: Vec::new(),
            agents: Vec::new(),
            motion: Motion::None,
            now: Timestamp::from_millis(0),
        };
        let origin = PixelPoint::new(0, 0);
        let seed = dungeon_seed(&snapshot);
        let architecture = architecture_mask(&snapshot);
        let width = u16::try_from(WIDTH).expect("Delve width fits u16");
        let height = u16::try_from(HEIGHT).expect("Delve height fits u16");
        let mut floor = RgbBuffer::filled(width, height, DEEP_BLUE_BLACK);
        paint_materials(&mut floor, origin, seed, &architecture);
        let mut composed = floor.clone();
        paint_background_architecture(&mut composed, origin, seed);
        let empty_plan = ScenePlan {
            world: WorldScene::Delve,
            camera: SceneCamera::WholeRoom,
            actors: Vec::new(),
            effects: Vec::new(),
            cadence: SceneCadence::EventDriven,
        };
        paint_depth_sorted(&snapshot, &empty_plan, &mut composed, origin);

        for doorway in DOORWAYS {
            assert_eq!(
                composed.get(doorway.point.x, doorway.point.y),
                floor.get(doorway.point.x, doorway.point.y),
                "{} was painted shut by final architecture",
                doorway.name
            );
        }
        assert!(
            !is_walkable(50, 31),
            "the final walkability evidence must include the opaque west arch wall"
        );
    }
}
