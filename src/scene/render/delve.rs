use std::{
    collections::{HashMap, VecDeque},
    sync::OnceLock,
    time::Duration,
};

use crate::{
    app::Motion,
    domain::Presence,
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
pub struct ArchitectureForeground {
    pub asset: DelveAsset,
    pub anchor_x: i32,
    pub anchor_y: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CompositionError {
    OverlappingBackground {
        point: PixelPoint,
        existing: ArchitectureBackground,
        attempted: ArchitectureBackground,
    },
    OverlappingForeground {
        point: PixelPoint,
        existing: ArchitectureForeground,
        attempted: ArchitectureForeground,
    },
    MissingBackground(PixelPoint),
    DoorwayBlocked(&'static str),
    DisconnectedFloor(PixelPoint),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelveArchitectureMask {
    background: Vec<Option<ArchitectureBackground>>,
    foreground: Vec<Option<ArchitectureForeground>>,
    walkable: Vec<bool>,
}

impl DelveArchitectureMask {
    #[must_use]
    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        architecture_index(x, y).is_some_and(|index| self.walkable[index])
    }

    #[must_use]
    pub fn background_at(&self, x: i32, y: i32) -> Option<ArchitectureBackground> {
        architecture_index(x, y).and_then(|index| self.background[index])
    }

    #[must_use]
    pub fn foreground_at(&self, x: i32, y: i32) -> Option<ArchitectureForeground> {
        architecture_index(x, y).and_then(|index| self.foreground[index])
    }

    pub fn background_owners(&self) -> impl Iterator<Item = ArchitectureBackground> + '_ {
        self.background.iter().filter_map(|owner| *owner)
    }
}

struct CompositionRecorder {
    background: Vec<Option<ArchitectureBackground>>,
    foreground: Vec<Option<ArchitectureForeground>>,
    walkable: Vec<bool>,
}

impl CompositionRecorder {
    fn new() -> Self {
        let pixel_count = usize::try_from(WIDTH * HEIGHT).expect("Delve area fits usize");
        Self {
            background: vec![None; pixel_count],
            foreground: vec![None; pixel_count],
            walkable: vec![false; pixel_count],
        }
    }

    fn record_background(
        &mut self,
        point: PixelPoint,
        owner: ArchitectureBackground,
        walkable: bool,
    ) -> Result<(), CompositionError> {
        let Some(index) = architecture_index(point.x, point.y) else {
            return Ok(());
        };
        if let Some(existing) = self.background[index] {
            return Err(CompositionError::OverlappingBackground {
                point,
                existing,
                attempted: owner,
            });
        }
        self.background[index] = Some(owner);
        self.walkable[index] = walkable;
        Ok(())
    }

    fn record_foreground(
        &mut self,
        point: PixelPoint,
        owner: ArchitectureForeground,
        blocks_walkable: bool,
    ) -> Result<(), CompositionError> {
        let Some(index) = architecture_index(point.x, point.y) else {
            return Ok(());
        };
        if let Some(existing) = self.foreground[index] {
            return Err(CompositionError::OverlappingForeground {
                point,
                existing,
                attempted: owner,
            });
        }
        self.foreground[index] = Some(owner);
        if blocks_walkable {
            self.walkable[index] = false;
        }
        Ok(())
    }

    fn finish(self) -> Result<DelveArchitectureMask, CompositionError> {
        let mask = DelveArchitectureMask {
            background: self.background,
            foreground: self.foreground,
            walkable: self.walkable,
        };
        validate_composition(&mask)?;
        Ok(mask)
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
        .get_or_init(|| {
            compose_architecture(0x00d3_116e_u64)
                .expect("built-in Delve composition must remain valid")
        })
        .is_walkable(x, y)
}

#[must_use]
pub fn architecture_mask(snapshot: &SceneSnapshot) -> DelveArchitectureMask {
    compose_architecture(dungeon_seed(snapshot))
        .expect("built-in Delve composition must remain valid")
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
    DelvePainter::new(snapshot, plan, viewport, target).paint()
}

struct DelvePainter<'a> {
    snapshot: &'a SceneSnapshot,
    plan: &'a ScenePlan,
    target: &'a mut RgbBuffer,
    origin: PixelPoint,
    seed: u64,
    architecture: DelveArchitectureMask,
}

impl<'a> DelvePainter<'a> {
    fn new(
        snapshot: &'a SceneSnapshot,
        plan: &'a ScenePlan,
        viewport: PixelSize,
        target: &'a mut RgbBuffer,
    ) -> Self {
        target.ensure_size(viewport.width, viewport.height, DEEP_BLUE_BLACK);
        let origin = dungeon_origin(snapshot, plan, viewport);
        let seed = dungeon_seed(snapshot);
        let architecture = compose_architecture(seed)
            .expect("built-in Delve composition must remain valid before painting");
        Self {
            snapshot,
            plan,
            target,
            origin,
            seed,
            architecture,
        }
    }

    fn paint(self) -> SceneFrame {
        paint_materials(
            self.target,
            self.origin,
            self.seed,
            Some(&self.architecture),
            None,
        )
        .expect("unrecorded material painting cannot conflict");
        paint_background_architecture(self.target, self.origin, self.seed);
        apply_lighting(self.snapshot, self.target, self.origin);
        let actor_deadline = paint_depth_sorted(self.snapshot, self.plan, self.target, self.origin);
        let effect_deadline = paint_effects(self.snapshot, self.plan, self.target, self.origin);
        paint_connection_fact(self.snapshot, self.target, self.origin);

        SceneFrame {
            world: self.plan.world,
            next_frame_in: actor_deadline.into_iter().chain(effect_deadline).min(),
        }
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
    architecture: Option<&DelveArchitectureMask>,
    mut recorder: Option<&mut CompositionRecorder>,
) -> Result<(), CompositionError> {
    for y in 0..target.size().height {
        for x in 0..target.size().width {
            let world_x = i32::from(x).saturating_sub(origin.x);
            let world_y = i32::from(y).saturating_sub(origin.y);
            let variant = material_variant(world_x, world_y, seed);
            let walkable = architecture.map_or_else(
                || base_walkable(world_x, world_y),
                |architecture| architecture.is_walkable(world_x, world_y),
            );
            let colour = if walkable {
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
            if let Some(recorder) = recorder.as_deref_mut() {
                recorder.record_background(
                    PixelPoint::new(world_x, world_y),
                    ArchitectureBackground::ConnectedDungeon,
                    walkable,
                )?;
            }
        }
    }
    Ok(())
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
    placed(DelveAsset::Rubble, 59, 53, false),
];

fn paint_background_architecture(target: &mut RgbBuffer, origin: PixelPoint, seed: u64) {
    paint_architecture_assets(target, origin, &background_assets(seed), None)
        .expect("unrecorded architecture painting cannot conflict");
}

fn paint_architecture_assets(
    target: &mut RgbBuffer,
    origin: PixelPoint,
    assets: &[PlacedAsset],
    mut recorder: Option<&mut CompositionRecorder>,
) -> Result<(), CompositionError> {
    for placed in assets {
        for_each_opaque_asset_pixel(*placed, |point, colour| {
            put(target, origin, point.x, point.y, colour);
            if let Some(recorder) = recorder.as_deref_mut() {
                recorder.record_foreground(
                    point,
                    ArchitectureForeground {
                        asset: placed.asset,
                        anchor_x: placed.anchor.x,
                        anchor_y: placed.anchor.y,
                    },
                    placed.blocks_walkable,
                )?;
            }
            Ok(())
        })?;
    }
    Ok(())
}

fn for_each_opaque_asset_pixel(
    placed: PlacedAsset,
    mut paint: impl FnMut(PixelPoint, Rgb) -> Result<(), CompositionError>,
) -> Result<(), CompositionError> {
    let sprite = frame(placed.asset);
    for (index, pixel) in sprite.pixels().iter().enumerate() {
        let Some(colour) = pixel else {
            continue;
        };
        let local_x = i32::try_from(index % usize::from(sprite.size().width))
            .expect("Delve asset x fits i32");
        let local_y = i32::try_from(index / usize::from(sprite.size().width))
            .expect("Delve asset y fits i32");
        paint(
            PixelPoint::new(placed.anchor.x + local_x, placed.anchor.y + local_y),
            *colour,
        )?;
    }
    Ok(())
}

fn compose_architecture(seed: u64) -> Result<DelveArchitectureMask, CompositionError> {
    compose_architecture_from(seed, &background_assets(seed), FOREGROUND_ASSETS)
}

fn compose_architecture_from(
    seed: u64,
    background: &[PlacedAsset],
    foreground: &[PlacedAsset],
) -> Result<DelveArchitectureMask, CompositionError> {
    let width = u16::try_from(WIDTH).expect("Delve width fits u16");
    let height = u16::try_from(HEIGHT).expect("Delve height fits u16");
    let origin = PixelPoint::new(0, 0);
    let mut target = RgbBuffer::filled(width, height, DEEP_BLUE_BLACK);
    let mut recorder = CompositionRecorder::new();
    paint_materials(&mut target, origin, seed, None, Some(&mut recorder))?;
    paint_architecture_assets(&mut target, origin, background, Some(&mut recorder))?;
    paint_architecture_assets(&mut target, origin, foreground, Some(&mut recorder))?;
    recorder.finish()
}

fn validate_composition(mask: &DelveArchitectureMask) -> Result<(), CompositionError> {
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if mask.background_at(x, y).is_none() {
                return Err(CompositionError::MissingBackground(PixelPoint::new(x, y)));
            }
        }
    }
    for doorway in DOORWAYS {
        if !mask.is_walkable(doorway.point.x, doorway.point.y) {
            return Err(CompositionError::DoorwayBlocked(doorway.name));
        }
    }

    let start = DOORWAYS[0].point;
    let mut visited = vec![false; mask.walkable.len()];
    let start_index = architecture_index(start.x, start.y)
        .expect("the named dungeon entrance is inside the canonical scene");
    visited[start_index] = true;
    let mut queue = VecDeque::from([start]);
    while let Some(point) = queue.pop_front() {
        for next in [
            PixelPoint::new(point.x - 1, point.y),
            PixelPoint::new(point.x + 1, point.y),
            PixelPoint::new(point.x, point.y - 1),
            PixelPoint::new(point.x, point.y + 1),
        ] {
            let Some(index) = architecture_index(next.x, next.y) else {
                continue;
            };
            if mask.walkable[index] && !visited[index] {
                visited[index] = true;
                queue.push_back(next);
            }
        }
    }
    if let Some(index) = mask
        .walkable
        .iter()
        .zip(&visited)
        .position(|(walkable, visited)| *walkable && !*visited)
    {
        let width = usize::try_from(WIDTH).expect("Delve width fits usize");
        return Err(CompositionError::DisconnectedFloor(PixelPoint::new(
            i32::try_from(index % width).expect("Delve x fits i32"),
            i32::try_from(index / width).expect("Delve y fits i32"),
        )));
    }
    Ok(())
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
) -> Option<Duration> {
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

    let mut next_frame_in = None;
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
                let elapsed = agent.presence_since.elapsed_until(snapshot.now);
                let animation = actor_animation_phase(snapshot.motion, placement.pose, elapsed);
                let sprite =
                    compact_adventurer_animation_frame(&agent.persona, placement.pose, animation);
                let actor_origin =
                    translate(origin, anchor.x, anchor.y - i32::from(animation == 1));
                if painted_sprite_is_visible(&sprite, actor_origin, target.size())
                    && let Some(delay) =
                        actor_next_frame_delay(snapshot.motion, placement.pose, elapsed)
                {
                    next_frame_in = Some(earliest_deadline(next_frame_in, delay));
                }
                if placement.pose == ScenePose::Unknown {
                    blit_dimmed(&sprite, actor_origin, target);
                } else {
                    blit(&sprite, actor_origin, target);
                }
                if placement.selected {
                    paint_selection_marker(target, actor_origin, sprite.size());
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
    next_frame_in
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
    PixelPoint::new(143, 56),
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
) -> Option<Duration> {
    let mut next_frame_in = None;
    for effect in &plan.effects {
        match effect {
            SceneEffect::FreshSpoils { agent, since } => {
                let elapsed = since.elapsed_until(snapshot.now);
                let phase = effect_animation_phase(snapshot.motion, elapsed);
                let seed =
                    stable_hash(agent.as_str().as_bytes()) ^ u64::try_from(phase).unwrap_or(0);
                let mut effect_visible = false;
                for index in 0..10_u64 {
                    let value = seed.rotate_left(u32::try_from(index * 5).unwrap_or(0));
                    let x = 119 + i32::try_from(value % 28).unwrap_or(0);
                    let y = 57 + i32::try_from(value.rotate_left(9) % 25).unwrap_or(0);
                    effect_visible |= is_visible(origin, PixelRect::new(x, y, 1, 1), target.size());
                    put(
                        target,
                        origin,
                        x,
                        y,
                        if index.is_multiple_of(2) {
                            TEAL_LIGHT
                        } else {
                            CHEST_GOLD
                        },
                    );
                }
                if snapshot.motion == Motion::Full && effect_visible {
                    next_frame_in = Some(earliest_deadline(
                        next_frame_in,
                        next_frame_delay(elapsed, 8),
                    ));
                }
            }
            SceneEffect::RecentDeparture { workspace_id, .. } => {
                let offset =
                    i32::try_from(stable_hash(workspace_id.as_str().as_bytes()) % 5).unwrap_or(0);
                put(target, origin, 5 + offset, 49, TEAL_GLOW);
            }
        }
    }
    next_frame_in
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
    paint_architecture_assets(target, origin, &[placed(asset, x, y, false)], None)
        .expect("unrecorded architecture painting cannot conflict");
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
        domain::{
            AdventurerClass, AdventurerPersona, AgentKey, Ancestry, PersonaKey, Timestamp,
            WorkspaceId,
        },
        scene::{
            snapshot::{SceneConnection, SceneSnapshot},
            stage::{ActorPlacement, SceneCadence, SceneCamera, ScenePlan, WorldScene},
        },
    };

    use super::*;

    fn rectangles_intersect(left: PixelRect, right: PixelRect) -> bool {
        left.x < right.x + i32::from(right.width)
            && right.x < left.x + i32::from(left.width)
            && left.y < right.y + i32::from(right.height)
            && right.y < left.y + i32::from(left.height)
    }

    #[test]
    fn exit_slots_keep_human_and_orc_edge_details_inside_the_landing() {
        for ancestry in [Ancestry::Human, Ancestry::Orc] {
            let mut persona =
                AdventurerPersona::for_key(PersonaKey::new(format!("exit-edge-{ancestry:?}")));
            persona.ancestry = ancestry;
            for animation_frame in 0..3 {
                let sprite = compact_adventurer_animation_frame(
                    &persona,
                    ScenePose::Working,
                    animation_frame,
                );
                for anchor in EXIT_SLOTS.iter().take(VISIBLE_ACTORS_PER_STATION) {
                    for (index, pixel) in sprite.pixels().iter().enumerate() {
                        if pixel.is_none() {
                            continue;
                        }
                        let x = anchor.x
                            + i32::try_from(index % usize::from(sprite.size().width))
                                .expect("sprite x fits i32");
                        let y = anchor.y
                            + i32::try_from(index / usize::from(sprite.size().width))
                                .expect("sprite y fits i32");
                        assert!(
                            contains(EXIT_LANDING, x, y),
                            "{ancestry:?} frame {animation_frame} at {anchor:?} paints ({x}, {y}) outside {EXIT_LANDING:?}"
                        );
                    }
                }
            }
        }
    }

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
                    selected: false,
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
        for (index, (_, anchor)) in anchors.iter().enumerate() {
            let bounds = PixelRect::new(anchor.x, anchor.y, 8, 14);
            for (_, other) in anchors.iter().skip(index + 1) {
                let other_bounds = PixelRect::new(other.x, other.y, 8, 14);
                assert!(
                    !rectangles_intersect(bounds, other_bounds),
                    "actor sprite rectangles collide: {bounds:?} and {other_bounds:?}"
                );
            }
        }

        let base_persona = AdventurerPersona::for_key(PersonaKey::new("slot-test"));
        for (placement, anchor) in anchors {
            let region = station_region(&placement.station, placement.pose);
            assert!(contains(region, anchor.x, anchor.y));
            assert!(contains(region, anchor.x + 7, anchor.y + 13));
            for ancestry in Ancestry::ALL {
                for class in AdventurerClass::ALL {
                    let mut persona = base_persona.clone();
                    persona.ancestry = *ancestry;
                    persona.class = *class;
                    for animation_frame in 0..3 {
                        let sprite = compact_adventurer_animation_frame(
                            &persona,
                            placement.pose,
                            animation_frame,
                        );
                        for (index, pixel) in sprite.pixels().iter().enumerate() {
                            if pixel.is_none() {
                                continue;
                            }
                            let x = anchor.x
                                + i32::try_from(index % usize::from(sprite.size().width))
                                    .expect("sprite x fits i32");
                            let y = anchor.y
                                + i32::try_from(index / usize::from(sprite.size().width))
                                    .expect("sprite y fits i32");
                            assert!(
                                contains(region, x, y),
                                "{} {ancestry:?} {class:?} frame {animation_frame} at ({x}, {y}) escapes {region:?}",
                                placement.agent
                            );
                        }
                    }
                }
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
                selected: false,
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
        paint_materials(
            &mut ambient,
            origin,
            dungeon_seed(&snapshot),
            Some(&architecture),
            None,
        )
        .expect("unrecorded material painting cannot conflict");
        paint_background_architecture(&mut ambient, origin, dungeon_seed(&snapshot));
        lighting::apply_cool_ambient(&mut ambient, 20);
        let mut composed = RgbBuffer::filled(width, height, DEEP_BLUE_BLACK);
        paint_materials(
            &mut composed,
            origin,
            dungeon_seed(&snapshot),
            Some(&architecture),
            None,
        )
        .expect("unrecorded material painting cannot conflict");
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
        paint_materials(&mut floor, origin, seed, Some(&architecture), None)
            .expect("unrecorded material painting cannot conflict");
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

    #[test]
    fn transparent_arch_pixels_do_not_block_the_floor_beneath_them() {
        let snapshot = SceneSnapshot {
            connection: SceneConnection::Connected,
            campaigns: Vec::new(),
            agents: Vec::new(),
            motion: Motion::None,
            now: Timestamp::from_millis(0),
        };
        let architecture = architecture_mask(&snapshot);

        assert!(base_walkable(50, 29), "the arch corner sits over floor");
        assert!(
            frame(DelveAsset::Arch).pixels()[0].is_none(),
            "the authored arch corner is transparent"
        );
        assert!(
            architecture.is_walkable(50, 29),
            "transparent arch paint must leave the floor traversable"
        );
    }

    #[test]
    fn duplicate_opaque_foreground_owner_writes_are_rejected() {
        let duplicate = placed(DelveAsset::Arch, 50, 29, true);
        let error = compose_architecture_from(0x00d3_116e_u64, &[], &[duplicate, duplicate])
            .expect_err("the second opaque owner write must be rejected");

        assert!(matches!(
            error,
            CompositionError::OverlappingForeground { .. }
        ));
    }

    #[test]
    fn duplicate_background_owner_writes_are_rejected() {
        let point = PixelPoint::new(12, 12);
        let mut recorder = CompositionRecorder::new();
        recorder
            .record_background(point, ArchitectureBackground::ConnectedDungeon, true)
            .expect("first owner write is valid");
        let error = recorder
            .record_background(point, ArchitectureBackground::ConnectedDungeon, true)
            .expect_err("the second background owner write must be rejected");

        assert!(matches!(
            error,
            CompositionError::OverlappingBackground { .. }
        ));
    }

    #[test]
    fn both_deterministic_rubble_variants_have_valid_final_compositions() {
        compose_architecture(0).expect("even-seed composition is valid");
        compose_architecture(1).expect("odd-seed composition is valid");
    }

    #[test]
    fn actual_opaque_foreground_paint_blocking_a_doorway_fails_validation() {
        let doorway = DOORWAYS[0];
        let blocking_door = placed(DelveAsset::Door, doorway.point.x, doorway.point.y, true);
        let error = compose_architecture_from(0x00d3_116e_u64, &[], &[blocking_door])
            .expect_err("an opaque foreground pixel must not close a named doorway");

        assert_eq!(error, CompositionError::DoorwayBlocked(doorway.name));
    }
}
