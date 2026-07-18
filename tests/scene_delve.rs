#![cfg(feature = "storybook")]

use std::{
    collections::{HashSet, VecDeque},
    time::Duration,
};

use questmancer::{
    app::Motion,
    domain::{
        AccentTone, AdventurerPersona, AgentKey, PersonaKey, Presence, Timestamp, WorkspaceId,
    },
    scene::{
        assets::{
            delve::{DelveAsset, frame},
            palette::{VOID, adventurer_palette},
        },
        pixel::{PixelRect, PixelSize, Rgb, RgbBuffer},
        render::delve::{
            ARCHITECTURE_REGIONS, DOORWAYS, HEIGHT, WIDTH, is_walkable, station_region,
        },
        render_scene_for_story,
        snapshot::{SceneAgent, SceneCampaign, SceneConnection, SceneSnapshot},
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
    let mut target = RgbBuffer::filled(0, 0, Rgb::BLACK);
    let frame = render_scene_for_story(snapshot, Some(world), viewport, &mut target);
    assert_eq!(frame.world, world);
    target
}

fn colours_in(buffer: &RgbBuffer, rect: PixelRect) -> HashSet<Rgb> {
    (rect.y..rect.y + i32::from(rect.height))
        .flat_map(|y| {
            (rect.x..rect.x + i32::from(rect.width)).filter_map(move |x| buffer.get(x, y))
        })
        .collect()
}

fn contains_colour(buffer: &RgbBuffer, rect: PixelRect, colour: Rgb) -> bool {
    (rect.y..rect.y + i32::from(rect.height))
        .any(|y| (rect.x..rect.x + i32::from(rect.width)).any(|x| buffer.get(x, y) == Some(colour)))
}

fn reachable_floor() -> HashSet<(i32, i32)> {
    let start = (DOORWAYS[0].point.x, DOORWAYS[0].point.y);
    assert!(is_walkable(start.0, start.1));
    let mut queue = VecDeque::from([start]);
    let mut visited = HashSet::from([start]);
    while let Some((x, y)) = queue.pop_front() {
        for (next_x, next_y) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
            if is_walkable(next_x, next_y) && visited.insert((next_x, next_y)) {
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
    let walkable = (0..HEIGHT)
        .flat_map(|y| (0..WIDTH).filter_map(move |x| is_walkable(x, y).then_some((x, y))))
        .collect::<HashSet<_>>();
    let reachable = reachable_floor();
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
    }
}

#[test]
fn truthful_station_regions_contain_their_actor_colours_and_exited_is_absent() {
    let snapshot = mixed_snapshot();
    let buffer = render(&snapshot, WorldScene::Delve, VIEWPORT);
    for (presence, accent, pose) in [
        (Presence::Working, AccentTone::Cyan, ScenePose::Working),
        (
            Presence::Blocked,
            AccentTone::Magenta,
            ScenePose::SeekingCounsel,
        ),
        (Presence::Done, AccentTone::Amber, ScenePose::Settled),
        (Presence::Idle, AccentTone::Lime, ScenePose::Resting),
        (Presence::Unknown, AccentTone::Violet, ScenePose::Unknown),
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
        let mut colour = adventurer_palette(
            agent.persona.appearance.skin_tone,
            agent.persona.appearance.hair_tone,
            agent.persona.appearance.garb,
            agent.persona.class,
            accent,
        )
        .accent;
        if pose == ScenePose::Unknown {
            colour = Rgb::new(colour.r / 2, colour.g / 2, colour.b / 2);
        }
        assert!(
            contains_colour(&buffer, station_region(&station, pose), colour),
            "{presence:?} actor is outside its truthful region"
        );
    }

    let exited = snapshot
        .agents
        .iter()
        .find(|agent| agent.presence == Presence::Exited)
        .expect("fixed exited agent exists");
    let exited_colour = adventurer_palette(
        exited.persona.appearance.skin_tone,
        exited.persona.appearance.hair_tone,
        exited.persona.appearance.garb,
        exited.persona.class,
        AccentTone::Red,
    )
    .accent;
    assert!(!buffer.pixels().contains(&exited_colour));
}

#[test]
fn unknown_dimming_preserves_transparent_sprite_pixels() {
    let snapshot = mixed_snapshot();
    let with_unknown = render(&snapshot, WorldScene::Delve, VIEWPORT);
    let mut without_unknown_snapshot = snapshot;
    without_unknown_snapshot
        .agents
        .retain(|agent| agent.presence != Presence::Unknown);
    let without_unknown = render(&without_unknown_snapshot, WorldScene::Delve, VIEWPORT);

    assert_eq!(
        with_unknown.get(24, 15),
        without_unknown.get(24, 15),
        "transparent corners of the Unknown sprite must leave dungeon pixels intact"
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
        "78958dc4adb346a389365d1009b6be288e46abf309ad3fd465fd1e842af1c5cf"
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
fn static_delve_has_no_cadence_and_active_animation_never_exceeds_eight_fps() {
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
    let interval = active_frame
        .next_frame_in
        .expect("working adventurers animate under full motion");
    assert!(interval >= Duration::from_millis(125));
}

fn is_crop_of(crop: &RgbBuffer, full: &RgbBuffer) -> bool {
    let max_x = i32::from(full.size().width.saturating_sub(crop.size().width));
    let max_y = i32::from(full.size().height.saturating_sub(crop.size().height));
    (0..=max_y).any(|offset_y| {
        (0..=max_x).any(|offset_x| {
            (0..i32::from(crop.size().height)).all(|y| {
                (0..i32::from(crop.size().width))
                    .all(|x| crop.get(x, y) == full.get(x + offset_x, y + offset_y))
            })
        })
    })
}

fn rgb_hash(buffer: &RgbBuffer) -> blake3::Hash {
    let bytes = buffer
        .pixels()
        .iter()
        .flat_map(|pixel| [pixel.r, pixel.g, pixel.b])
        .collect::<Vec<_>>();
    blake3::hash(&bytes)
}
