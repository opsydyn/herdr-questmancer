#![cfg(feature = "storybook")]

use std::collections::{HashSet, VecDeque};

use questmancer::{
    app::Motion,
    domain::{
        AccentTone, AdventurerPersona, AgentKey, GuildSummons, PersonaKey, Presence, Timestamp,
        WorkspaceId,
    },
    scene::{
        SceneFrame,
        assets::{
            adventurer::adventurer_animation_frame,
            delve::{DelveAsset, frame},
            palette::VOID,
        },
        pixel::{PixelRect, PixelSize, Rgb, RgbBuffer},
        render::delve::{
            ARCHITECTURE_REGIONS, ArchitectureBackground, ArchitectureForeground, DOORWAYS,
            DelveArchitectureMask, HEIGHT, WIDTH, architecture_mask, station_region,
        },
        render_scene_for_story,
        snapshot::{SceneAgent, SceneCampaign, SceneConnection, SceneSnapshot, SceneTransition},
        stage::{ScenePose, TruthfulStation, WorldScene},
    },
};

const VIEWPORT: PixelSize = PixelSize::new(160, 90);

fn campaign(id: &str, seed: u64) -> SceneCampaign {
    SceneCampaign {
        workspace_id: WorkspaceId::new(id),
        label: id.replace('-', " "),
        variant_seed: seed,
    }
}

fn agent(key: &str, workspace: &str, presence: Presence, accent: AccentTone) -> SceneAgent {
    let mut persona = AdventurerPersona::for_key(PersonaKey::new(format!("delve-{key}")));
    persona.appearance.accent = accent;
    SceneAgent {
        key: AgentKey::new(key),
        workspace_id: WorkspaceId::new(workspace),
        name: key.replace('-', " "),
        custom_status: None,
        presence,
        presence_since: Timestamp::from_millis(1_000),
        transition: None,
        focused: false,
        persona,
    }
}

fn mixed_snapshot() -> SceneSnapshot {
    SceneSnapshot {
        connection: SceneConnection::Connected,
        campaigns: vec![
            campaign("moss-vault", 0x47a1),
            campaign("rune-road", 0xb20f),
        ],
        agents: vec![
            agent("working", "moss-vault", Presence::Working, AccentTone::Cyan),
            agent(
                "blocked",
                "moss-vault",
                Presence::Blocked,
                AccentTone::Magenta,
            ),
            agent("done", "rune-road", Presence::Done, AccentTone::Amber),
            agent("idle", "rune-road", Presence::Idle, AccentTone::Lime),
            agent(
                "unknown",
                "moss-vault",
                Presence::Unknown,
                AccentTone::Violet,
            ),
            agent("exited", "rune-road", Presence::Exited, AccentTone::Red),
        ],
        motion: Motion::None,
        now: Timestamp::from_millis(10_000),
    }
}

fn render(snapshot: &SceneSnapshot, world: WorldScene, viewport: PixelSize) -> RgbBuffer {
    render_with_frame(snapshot, world, viewport).0
}

fn render_with_frame(
    snapshot: &SceneSnapshot,
    world: WorldScene,
    viewport: PixelSize,
) -> (RgbBuffer, SceneFrame) {
    let mut target = RgbBuffer::filled(0, 0, Rgb::BLACK);
    let frame = render_scene_for_story(snapshot, Some(world), viewport, &mut target);
    assert_eq!(frame.world, world);
    (target, frame)
}

fn colours_in(buffer: &RgbBuffer, rect: PixelRect) -> HashSet<Rgb> {
    (rect.y..rect.y + i32::from(rect.height))
        .flat_map(|y| {
            (rect.x..rect.x + i32::from(rect.width)).filter_map(move |x| buffer.get(x, y))
        })
        .collect()
}

fn reachable_floor(mask: &DelveArchitectureMask) -> HashSet<(i32, i32)> {
    let start = (DOORWAYS[0].point.x, DOORWAYS[0].point.y);
    assert!(mask.is_walkable(start.0, start.1));
    let mut queue = VecDeque::from([start]);
    let mut visited = HashSet::from([start]);
    while let Some((x, y)) = queue.pop_front() {
        for (next_x, next_y) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
            if mask.is_walkable(next_x, next_y) && visited.insert((next_x, next_y)) {
                queue.push_back((next_x, next_y));
            }
        }
    }
    visited
}

fn mean_coolness(buffer: &RgbBuffer) -> i64 {
    let total = buffer.pixels().iter().fold(0_i64, |sum, pixel| {
        sum + i64::from(pixel.g) + i64::from(pixel.b) - 2 * i64::from(pixel.r)
    });
    total / i64::try_from(buffer.pixels().len()).expect("buffer length fits i64")
}

#[test]
fn delve_has_the_nineteen_original_indexed_environment_families() {
    assert_eq!(DelveAsset::ALL.len(), 19);
    assert_eq!(
        DelveAsset::ALL,
        &[
            DelveAsset::DressedStoneWall,
            DelveAsset::CrackedMossyFloor,
            DelveAsset::Arch,
            DelveAsset::Door,
            DelveAsset::DescendingStair,
            DelveAsset::ActivePassage,
            DelveAsset::SealedGate,
            DelveAsset::ExitLanding,
            DelveAsset::Camp,
            DelveAsset::Torch,
            DelveAsset::Brazier,
            DelveAsset::RuneStones,
            DelveAsset::Roots,
            DelveAsset::Columns,
            DelveAsset::Rubble,
            DelveAsset::Puddles,
            DelveAsset::Bones,
            DelveAsset::Chests,
            DelveAsset::DungeonClutter,
        ]
    );
    for asset in DelveAsset::ALL {
        let sprite = frame(*asset);
        assert!(sprite.size().width > 0, "{asset:?} width");
        assert!(sprite.size().height > 0, "{asset:?} height");
        assert!(
            sprite.pixels().iter().filter_map(|pixel| *pixel).count() >= 4,
            "{asset:?} must be visibly authored"
        );
    }
}

#[test]
fn every_authored_floor_pixel_doorway_and_named_region_is_connected() {
    assert_eq!((WIDTH, HEIGHT), (160, 90));
    let mask = architecture_mask(&mixed_snapshot());
    let mask_ref = &mask;
    let walkable = (0..HEIGHT)
        .flat_map(|y| (0..WIDTH).filter_map(move |x| mask_ref.is_walkable(x, y).then_some((x, y))))
        .collect::<HashSet<_>>();
    let reachable = reachable_floor(&mask);
    assert_eq!(
        reachable, walkable,
        "the authored dungeon floor must be one graph"
    );

    let mut doorway_names = HashSet::new();
    for doorway in DOORWAYS {
        assert!(doorway_names.insert(doorway.name), "duplicate doorway name");
        assert!(
            reachable.contains(&(doorway.point.x, doorway.point.y)),
            "{} is disconnected",
            doorway.name
        );
    }

    let mut region_names = HashSet::new();
    for region in ARCHITECTURE_REGIONS {
        assert!(
            region_names.insert(region.name),
            "duplicate architecture name"
        );
        assert!(
            (region.bounds.y..region.bounds.y + i32::from(region.bounds.height)).any(|y| {
                (region.bounds.x..region.bounds.x + i32::from(region.bounds.width))
                    .any(|x| reachable.contains(&(x, y)))
            }),
            "{} has no connected floor",
            region.name
        );
        assert!(
            (region.bounds.y..region.bounds.y + i32::from(region.bounds.height)).all(|y| {
                (region.bounds.x..region.bounds.x + i32::from(region.bounds.width)).all(|x| {
                    mask.background_at(x, y) == Some(ArchitectureBackground::ConnectedDungeon)
                })
            }),
            "{} acquired an independent chamber background",
            region.name
        );
    }

    let background_owners = mask.background_owners().collect::<HashSet<_>>();
    assert_eq!(
        background_owners,
        HashSet::from([ArchitectureBackground::ConnectedDungeon]),
        "the canonical scene must have one dungeon background owner and no chamber backplates"
    );
    assert_eq!(
        mask.background_owners().count(),
        usize::try_from(WIDTH * HEIGHT).expect("Delve area fits usize"),
        "the one dungeon background owner must come from every actual background paint"
    );
    assert_eq!(
        mask.foreground_at(50, 29),
        None,
        "transparent west-arch pixels must not acquire foreground ownership"
    );
    assert_eq!(
        mask.foreground_at(50, 31),
        Some(ArchitectureForeground {
            asset: DelveAsset::Arch,
            anchor_x: 50,
            anchor_y: 29,
        }),
        "opaque west-arch pixels must retain their actual paint owner"
    );
}

#[test]
fn truthful_station_regions_intersect_their_authored_actors_and_exited_is_absent() {
    let snapshot = mixed_snapshot();
    let (_, frame) = render_with_frame(&snapshot, WorldScene::Delve, VIEWPORT);
    for (presence, pose) in [
        (Presence::Working, ScenePose::Working),
        (Presence::Blocked, ScenePose::SeekingCounsel),
        (Presence::Done, ScenePose::Settled),
        (Presence::Idle, ScenePose::Resting),
        (Presence::Unknown, ScenePose::Unknown),
    ] {
        let agent = snapshot
            .agents
            .iter()
            .find(|agent| agent.presence == presence)
            .expect("fixed presence exists");
        let station = match presence {
            Presence::Working | Presence::Unknown => {
                TruthfulStation::DelveActive(agent.workspace_id.clone())
            }
            Presence::Blocked => TruthfulStation::DelveGate(agent.workspace_id.clone()),
            Presence::Done => TruthfulStation::DelveExit(agent.workspace_id.clone()),
            Presence::Idle => TruthfulStation::DelveCamp(agent.workspace_id.clone()),
            Presence::Exited => unreachable!(),
        };
        let actor = frame
            .actors
            .iter()
            .find(|actor| actor.agent == agent.key)
            .expect("visible agent has an actor region");
        assert!(
            rects_intersect(actor.bounds, station_region(&station, pose)),
            "{presence:?} actor is outside its truthful region"
        );
    }

    let exited = snapshot
        .agents
        .iter()
        .find(|agent| agent.presence == Presence::Exited)
        .expect("fixed exited agent exists");
    assert!(frame.actors.iter().all(|actor| actor.agent != exited.key));
}

#[test]
fn unknown_dimming_preserves_transparent_sprite_pixels() {
    let snapshot = mixed_snapshot();
    let unknown = snapshot
        .agents
        .iter()
        .find(|agent| agent.presence == Presence::Unknown)
        .expect("fixed unknown agent exists");
    let sprite = adventurer_animation_frame(&unknown.persona, ScenePose::Unknown, 0);
    let (with_unknown, frame) = render_with_frame(&snapshot, WorldScene::Delve, VIEWPORT);
    let region = frame
        .actors
        .iter()
        .find(|actor| actor.agent == unknown.key)
        .expect("unknown agent has an actor region");
    let mut without_unknown_snapshot = snapshot;
    without_unknown_snapshot
        .agents
        .retain(|agent| agent.presence != Presence::Unknown);
    let without_unknown = render(&without_unknown_snapshot, WorldScene::Delve, VIEWPORT);

    let mut transparent_pixels = 0;
    for (index, pixel) in sprite.pixels().iter().enumerate() {
        if pixel.is_some() {
            continue;
        }
        transparent_pixels += 1;
        let x = region.bounds.x
            + i32::try_from(index % usize::from(sprite.size().width)).expect("sprite x fits i32");
        let y = region.bounds.y
            + i32::try_from(index / usize::from(sprite.size().width)).expect("sprite y fits i32");
        assert_eq!(
            with_unknown.get(x, y),
            without_unknown.get(x, y),
            "transparent Unknown pixel at ({x}, {y}) changed the dungeon"
        );
    }
    assert!(
        transparent_pixels > 0,
        "authored Unknown sprite must contain transparency"
    );
}

#[test]
fn canonical_delve_is_dense_colourful_deterministic_and_cooler_than_the_hall() {
    let snapshot = mixed_snapshot();
    let first = render(&snapshot, WorldScene::Delve, VIEWPORT);
    let second = render(&snapshot, WorldScene::Delve, VIEWPORT);
    assert_eq!(first.pixels(), second.pixels());
    assert_eq!(rgb_hash(&first), rgb_hash(&second));
    assert_eq!(
        rgb_hash(&first).to_hex().as_str(),
        "a1d847670d4f890414dc224ad649c0cd64ce18ebbeea78fbd6dfaac16c832479"
    );

    let non_clear = first
        .pixels()
        .iter()
        .filter(|pixel| **pixel != VOID)
        .count();
    assert!(non_clear * 100 >= first.pixels().len() * 85);
    assert!(first.pixels().iter().copied().collect::<HashSet<_>>().len() >= 24);
    for region in ARCHITECTURE_REGIONS {
        assert!(
            colours_in(&first, region.bounds).len() >= 3,
            "{} is visually empty",
            region.name
        );
    }

    let guild = render(&snapshot, WorldScene::GuildHall, VIEWPORT);
    assert!(
        mean_coolness(&first) >= mean_coolness(&guild) + 20,
        "Delve {} vs Guild Hall {}",
        mean_coolness(&first),
        mean_coolness(&guild)
    );
}

#[test]
fn minimum_viewport_is_a_camera_crop_of_the_same_canonical_dungeon() {
    let mut snapshot = mixed_snapshot();
    snapshot
        .agents
        .retain(|agent| agent.presence == Presence::Working);
    snapshot.agents[0].focused = true;
    let full = render(&snapshot, WorldScene::Delve, VIEWPORT);
    let crop = render(&snapshot, WorldScene::Delve, PixelSize::new(80, 48));

    assert!(is_crop_of(&crop, &full));
    assert!(crop.pixels().iter().all(|pixel| *pixel != VOID));
}

#[test]
fn narrow_camera_keeps_the_priority_blocked_actor_and_sealed_gate_visible() {
    let mut unknown = agent(
        "a-unknown",
        "shared-vault",
        Presence::Unknown,
        AccentTone::Violet,
    );
    unknown.focused = true;
    let blocked = agent(
        "z-blocked",
        "shared-vault",
        Presence::Blocked,
        AccentTone::Magenta,
    );
    let snapshot = SceneSnapshot {
        connection: SceneConnection::Connected,
        campaigns: vec![campaign("shared-vault", 0x47a1)],
        agents: vec![unknown, blocked.clone()],
        motion: Motion::None,
        now: Timestamp::from_millis(10_000),
    };
    let full = render(&snapshot, WorldScene::Delve, VIEWPORT);
    let (crop, crop_frame) =
        render_with_frame(&snapshot, WorldScene::Delve, PixelSize::new(80, 48));
    let (offset_x, offset_y) = crop_offset(&crop, &full).expect("narrow Delve is a world crop");
    let visible_world = PixelRect::new(offset_x, offset_y, 80, 48);
    let gate_asset = PixelRect::new(127, 28, 16, 10);

    assert!(rects_intersect(visible_world, gate_asset));
    assert!(rects_intersect(
        visible_world,
        station_region(
            &TruthfulStation::DelveGate(blocked.workspace_id.clone()),
            ScenePose::SeekingCounsel,
        )
    ));
    let blocked_actor = crop_frame
        .actors
        .iter()
        .find(|actor| actor.agent == blocked.key)
        .expect("priority blocked actor has a rendered region");
    assert!(rects_intersect(
        blocked_actor.bounds,
        PixelRect::new(0, 0, 80, 48),
    ));
}

#[test]
fn reconnecting_preserves_the_authored_dungeon_under_connection_light() {
    let connected_snapshot = mixed_snapshot();
    let connected = render(&connected_snapshot, WorldScene::Delve, VIEWPORT);
    let mut reconnecting_snapshot = connected_snapshot;
    reconnecting_snapshot.connection = SceneConnection::Reconnecting { attempt: 3 };
    let reconnecting = render(&reconnecting_snapshot, WorldScene::Delve, VIEWPORT);

    assert_ne!(reconnecting.pixels(), connected.pixels());
    assert!(reconnecting.pixels().iter().all(|pixel| *pixel != VOID));
    for region in ARCHITECTURE_REGIONS {
        assert!(
            colours_in(&reconnecting, region.bounds).len() >= 3,
            "{} disappeared while reconnecting",
            region.name
        );
    }
}

#[test]
fn static_authored_delve_actors_have_no_invisible_cadence() {
    let static_snapshot = mixed_snapshot();
    let mut target = RgbBuffer::filled(0, 0, Rgb::BLACK);
    let static_frame = render_scene_for_story(
        &static_snapshot,
        Some(WorldScene::Delve),
        VIEWPORT,
        &mut target,
    );
    assert_eq!(static_frame.next_frame_in, None);

    let mut active_snapshot = static_snapshot;
    active_snapshot
        .agents
        .retain(|agent| agent.presence == Presence::Working);
    active_snapshot.motion = Motion::Full;
    let active_frame = render_scene_for_story(
        &active_snapshot,
        Some(WorldScene::Delve),
        VIEWPORT,
        &mut target,
    );
    assert_eq!(active_frame.next_frame_in, None);
}

#[test]
fn no_motion_delve_is_byte_identical_across_fresh_completion_timestamps() {
    let mut completed = agent("completed", "rune-road", Presence::Done, AccentTone::Amber);
    completed.transition = Some(SceneTransition {
        summons: GuildSummons::SpoilsReturned,
        since: Timestamp::from_millis(10_000),
    });
    let mut snapshot = SceneSnapshot {
        connection: SceneConnection::Connected,
        campaigns: vec![campaign("rune-road", 0xb20f)],
        agents: vec![completed],
        motion: Motion::None,
        now: Timestamp::from_millis(10_000),
    };

    let first = render(&snapshot, WorldScene::Delve, VIEWPORT);
    snapshot.now = Timestamp::from_millis(10_125);
    let second = render(&snapshot, WorldScene::Delve, VIEWPORT);

    assert_eq!(first.pixels(), second.pixels());
}

fn is_crop_of(crop: &RgbBuffer, full: &RgbBuffer) -> bool {
    crop_offset(crop, full).is_some()
}

fn crop_offset(crop: &RgbBuffer, full: &RgbBuffer) -> Option<(i32, i32)> {
    let max_x = i32::from(full.size().width.saturating_sub(crop.size().width));
    let max_y = i32::from(full.size().height.saturating_sub(crop.size().height));
    (0..=max_y).find_map(|offset_y| {
        (0..=max_x).find_map(|offset_x| {
            (0..i32::from(crop.size().height))
                .all(|y| {
                    (0..i32::from(crop.size().width))
                        .all(|x| crop.get(x, y) == full.get(x + offset_x, y + offset_y))
                })
                .then_some((offset_x, offset_y))
        })
    })
}

fn rects_intersect(left: PixelRect, right: PixelRect) -> bool {
    left.x < right.x + i32::from(right.width)
        && right.x < left.x + i32::from(left.width)
        && left.y < right.y + i32::from(right.height)
        && right.y < left.y + i32::from(left.height)
}

fn rgb_hash(buffer: &RgbBuffer) -> blake3::Hash {
    let bytes = buffer
        .pixels()
        .iter()
        .flat_map(|pixel| [pixel.r, pixel.g, pixel.b])
        .collect::<Vec<_>>();
    blake3::hash(&bytes)
}
