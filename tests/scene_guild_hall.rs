#![cfg(feature = "storybook")]

use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use questmancer::{
    app::Motion,
    domain::{
        AccentTone, AdventurerClass, AdventurerPersona, AgentKey, GuildSummons, PersonaKey,
        Presence, Timestamp, WorkspaceId,
    },
    scene::{
        SceneFrame, SceneInteractable,
        assets::palette::{
            EMBER, FLAME, INK_BLUE, OAK, PARCHMENT_DARK, PARCHMENT_LIGHT, RUG, RUG_GOLD, SHADOW,
            VOID,
        },
        pixel::{PixelRect, PixelSize, Rgb, RgbBuffer},
        render_scene_for_story,
        snapshot::{SceneAgent, SceneCampaign, SceneConnection, SceneSnapshot, SceneTransition},
        stage::WorldScene,
    },
};

const VIEWPORT: PixelSize = PixelSize::new(160, 90);
const PARCHMENT: Rgb = Rgb::new(230, 207, 154);

fn campaign(id: &str, seed: u64) -> SceneCampaign {
    SceneCampaign {
        workspace_id: WorkspaceId::new(id),
        label: id.replace('-', " "),
        variant_seed: seed,
    }
}

fn agent(key: &str, workspace: &str, presence: Presence, accent: AccentTone) -> SceneAgent {
    let mut persona = AdventurerPersona::for_key(PersonaKey::new(format!("guild-hall-{key}")));
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
        campaigns: vec![campaign("amber-library", 7), campaign("moss-vault", 29)],
        agents: vec![
            agent(
                "first-working",
                "amber-library",
                Presence::Working,
                AccentTone::Cyan,
            ),
            agent(
                "second-working",
                "amber-library",
                Presence::Working,
                AccentTone::Lime,
            ),
            agent(
                "counsel-seeker",
                "moss-vault",
                Presence::Blocked,
                AccentTone::Magenta,
            ),
            agent(
                "spoils-returnee",
                "moss-vault",
                Presence::Done,
                AccentTone::Blue,
            ),
            agent(
                "hearth-resting",
                "amber-library",
                Presence::Idle,
                AccentTone::Teal,
            ),
        ],
        motion: Motion::None,
        now: Timestamp::from_millis(10_000),
    }
}

fn render(snapshot: &SceneSnapshot, viewport: PixelSize) -> RgbBuffer {
    render_with_frame(snapshot, viewport).0
}

fn render_with_frame(snapshot: &SceneSnapshot, viewport: PixelSize) -> (RgbBuffer, SceneFrame) {
    let mut target = RgbBuffer::filled(0, 0, Rgb::BLACK);
    let frame =
        render_scene_for_story(snapshot, Some(WorldScene::GuildHall), viewport, &mut target);
    assert_eq!(frame.world, WorldScene::GuildHall);
    (target, frame)
}

fn rect_contains(buffer: &RgbBuffer, rect: PixelRect, colour: Rgb) -> bool {
    (rect.y..rect.y + i32::from(rect.height))
        .any(|y| (rect.x..rect.x + i32::from(rect.width)).any(|x| buffer.get(x, y) == Some(colour)))
}

fn hash_parity(workspace: &str) -> usize {
    usize::from(blake3::hash(workspace.as_bytes()).as_bytes()[0] % 2)
}

#[test]
fn canonical_room_contains_every_owned_landmark_signature() {
    let mut snapshot = mixed_snapshot();
    snapshot.agents.clear();
    let buffer = render(&snapshot, VIEWPORT);
    for (name, rect, colour) in [
        ("guild door", PixelRect::new(5, 14, 25, 46), OAK),
        ("quest wall", PixelRect::new(34, 11, 43, 27), PARCHMENT),
        ("left campaign table", PixelRect::new(35, 47, 38, 26), OAK),
        ("right campaign table", PixelRect::new(77, 47, 38, 26), OAK),
        ("counsel bell", PixelRect::new(116, 31, 18, 31), RUG_GOLD),
        ("hearth", PixelRect::new(132, 9, 27, 48), EMBER),
        ("spoils bench", PixelRect::new(112, 64, 47, 25), RUG),
    ] {
        assert!(
            rect_contains(&buffer, rect, colour),
            "{name} lacks its authored colour signature"
        );
    }
}

#[test]
fn actors_occupy_truthful_station_bounds_with_unique_final_anchors() {
    let (buffer, frame) = render_with_frame(&mixed_snapshot(), VIEWPORT);
    let expected = [
        (
            "first-working",
            PixelRect::new(35, 47, 80, 27),
            "first campaign token",
        ),
        (
            "second-working",
            PixelRect::new(35, 47, 80, 27),
            "second campaign token",
        ),
        (
            "counsel-seeker",
            PixelRect::new(112, 31, 24, 38),
            "counsel seeker",
        ),
        (
            "spoils-returnee",
            PixelRect::new(108, 61, 51, 29),
            "spoils returnee",
        ),
        (
            "hearth-resting",
            PixelRect::new(128, 42, 31, 39),
            "hearth resting",
        ),
    ];
    let mut anchors = HashSet::new();
    for (agent, station, label) in expected {
        let region = frame
            .actors
            .iter()
            .find(|region| region.agent == AgentKey::new(agent))
            .unwrap_or_else(|| panic!("{label} has no rendered actor region"));
        let anchor = (
            region.bounds.x + i32::from(region.bounds.width / 2),
            region.bounds.y + i32::from(region.bounds.height) - 1,
        );
        assert!(
            anchor.0 >= station.x
                && anchor.0 < station.x + i32::from(station.width)
                && anchor.1 >= station.y
                && anchor.1 < station.y + i32::from(station.height),
            "{label} anchor {anchor:?} is outside {station:?}"
        );
        assert!(anchors.insert(anchor), "two actors share anchor {anchor:?}");
        assert!(
            (region.bounds.y..region.bounds.y + i32::from(region.bounds.height)).any(|y| {
                (region.bounds.x..region.bounds.x + i32::from(region.bounds.width))
                    .any(|x| buffer.get(x, y).is_some_and(|pixel| pixel != VOID))
            }),
            "{label} region is visually empty"
        );
    }
}

#[test]
fn canonical_hall_never_stacks_complete_adventurer_footprints() {
    let mut snapshot = mixed_snapshot();
    snapshot.agents.extend([
        agent(
            "second-counsel-seeker",
            "amber-library",
            Presence::Blocked,
            AccentTone::Red,
        ),
        agent(
            "second-spoils-returnee",
            "amber-library",
            Presence::Done,
            AccentTone::Amber,
        ),
        agent(
            "second-hearth-resting",
            "moss-vault",
            Presence::Idle,
            AccentTone::Violet,
        ),
    ]);

    let (_, frame) = render_with_frame(&snapshot, VIEWPORT);
    for (index, left) in frame.actors.iter().enumerate() {
        for right in frame.actors.iter().skip(index + 1) {
            assert!(
                !overlaps(left.bounds, right.bounds),
                "{} overlaps {}: {:?} vs {:?}",
                left.agent,
                right.agent,
                left.bounds,
                right.bounds
            );
        }
    }
}

#[test]
fn a_full_canonical_hall_recomposes_before_hiding_adventurers() {
    let mut snapshot = mixed_snapshot();
    snapshot.agents = (0_u8..12)
        .map(|index| {
            agent(
                &format!("party-{index}"),
                if index.is_multiple_of(2) {
                    "amber-library"
                } else {
                    "moss-vault"
                },
                Presence::Working,
                AccentTone::Cyan,
            )
        })
        .collect();

    let (_, frame) = render_with_frame(&snapshot, VIEWPORT);
    assert_eq!(
        frame.actors.len(),
        snapshot.agents.len(),
        "the Hall must use its whole-party compact composition before omitting party members"
    );
}

#[test]
fn campaign_tables_show_full_adventurer_identity_instead_of_colour_tokens() {
    let snapshot = mixed_snapshot();
    let (_, frame) = render_with_frame(&snapshot, VIEWPORT);

    for key in ["first-working", "second-working"] {
        let region = frame
            .actors
            .iter()
            .find(|region| region.agent == AgentKey::new(key))
            .unwrap_or_else(|| panic!("{key} has no campaign-table actor"));
        assert_eq!(
            (region.bounds.width, region.bounds.height),
            (16, 24),
            "{key} was reduced to an unrecognisable campaign token"
        );
    }
}

#[test]
fn compact_guild_hall_keeps_the_whole_party_visible_and_clickable() {
    let viewport = PixelSize::new(80, 48);
    let (_, frame) = render_with_frame(&mixed_snapshot(), viewport);

    assert_eq!(frame.actors.len(), 5, "the compact hall retains the party");
    for region in &frame.actors {
        assert!(
            region.bounds.x >= 0
                && region.bounds.y >= 0
                && region.bounds.x + i32::from(region.bounds.width) <= i32::from(viewport.width)
                && region.bounds.y + i32::from(region.bounds.height) <= i32::from(viewport.height),
            "{} is cropped outside the compact hall: {:?}",
            region.agent,
            region.bounds
        );
        assert_eq!(
            (region.bounds.width, region.bounds.height),
            (16, 24),
            "{} must retain its authored world-master scale",
            region.agent
        );
    }
}

#[test]
fn canonical_hearth_keeps_three_resting_adventurers_inside_the_room() {
    let mut snapshot = mixed_snapshot();
    snapshot.agents = ["first", "second", "third"]
        .into_iter()
        .map(|key| agent(key, "amber-library", Presence::Idle, AccentTone::Teal))
        .collect();

    let (_, frame) = render_with_frame(&snapshot, VIEWPORT);

    assert_eq!(frame.actors.len(), 3);
    for actor in frame.actors {
        assert!(
            actor.bounds.x >= 0
                && actor.bounds.y >= 0
                && actor.bounds.x + i32::from(actor.bounds.width) <= i32::from(VIEWPORT.width)
                && actor.bounds.y + i32::from(actor.bounds.height) <= i32::from(VIEWPORT.height),
            "{} is cropped outside the canonical Hall: {:?}",
            actor.agent,
            actor.bounds
        );
    }
}

#[test]
fn librarian_has_one_complete_non_agent_station_in_canonical_and_compact_halls() {
    for viewport in [PixelSize::new(160, 90), PixelSize::new(80, 48)] {
        let (_, frame) = render_with_frame(&mixed_snapshot(), viewport);
        let librarians = frame
            .interactables
            .iter()
            .filter(|region| region.kind == SceneInteractable::Librarian)
            .collect::<Vec<_>>();

        assert_eq!(librarians.len(), 1, "missing Librarian at {viewport:?}");
        let bounds = librarians[0].bounds;
        assert!(bounds.x >= 0 && bounds.y >= 0);
        assert!(bounds.x + i32::from(bounds.width) <= i32::from(viewport.width));
        assert!(bounds.y + i32::from(bounds.height) <= i32::from(viewport.height));
        assert_eq!((bounds.width, bounds.height), (16, 24));
        assert!(
            frame
                .actors
                .iter()
                .all(|actor| !overlaps(actor.bounds, bounds))
        );
    }
}

#[test]
fn librarian_yields_vignette_and_status_only_halls_to_live_status() {
    for viewport in [PixelSize::new(40, 36), PixelSize::new(12, 12)] {
        let (_, frame) = render_with_frame(&mixed_snapshot(), viewport);
        assert!(
            frame.interactables.is_empty(),
            "unexpected NPC at {viewport:?}"
        );
    }
}

#[test]
fn compact_and_vignette_halls_keep_a_seamless_quiet_stage_behind_the_party() {
    let snapshot = SceneSnapshot {
        connection: SceneConnection::Connected,
        campaigns: vec![campaign("amber-library", 7)],
        agents: vec![agent(
            "solo-working",
            "amber-library",
            Presence::Working,
            AccentTone::Amber,
        )],
        motion: Motion::None,
        now: Timestamp::from_millis(2_000),
    };

    // A compact and a vignette viewport. The sampled corner sits beside the
    // party on the floor, where the plank pattern used to run seams straight
    // through the actor lane.
    for viewport in [PixelSize::new(100, 48), PixelSize::new(40, 30)] {
        let buffer = render(&snapshot, viewport);
        let bottom = i32::from(viewport.height);
        let mut colours = HashSet::new();
        for x in 1..=6 {
            for y in (bottom - 8)..(bottom - 1) {
                colours.insert(buffer.get(x, y).expect("sampled pixel is inside the stage"));
            }
        }
        assert_eq!(
            colours.len(),
            1,
            "{viewport:?}: the stage behind the party must be a single flat value, found {colours:?}"
        );
    }
}

fn roster_snapshot(size: usize) -> SceneSnapshot {
    SceneSnapshot {
        connection: SceneConnection::Connected,
        campaigns: vec![campaign("amber-library", 7)],
        agents: (0..size)
            .map(|index| {
                let presence = if index == 1 {
                    Presence::Blocked
                } else {
                    Presence::Working
                };
                agent(
                    &format!("roster-{index}"),
                    "amber-library",
                    presence,
                    AccentTone::Amber,
                )
            })
            .collect(),
        motion: Motion::None,
        now: Timestamp::from_millis(2_000),
    }
}

#[test]
fn a_narrow_pane_keeps_the_whole_party_visible_instead_of_one_adventurer() {
    // Too small for six 16x24 masters, comfortably large enough for the
    // authored roster tier. Before the roster tier this fell to a vignette
    // and hid five of the six adventurers.
    let snapshot = roster_snapshot(6);
    let viewport = PixelSize::new(60, 40);

    let (buffer, frame) = render_with_frame(&snapshot, viewport);

    assert_eq!(
        frame.actors.len(),
        6,
        "every adventurer must keep a hit region in a narrow pane"
    );
    for actor in &frame.actors {
        assert_eq!(
            (actor.bounds.width, actor.bounds.height),
            (8, 12),
            "roster actors use the authored 8x12 master"
        );
        assert!(
            actor.bounds.x >= 0
                && actor.bounds.y >= 0
                && actor.bounds.x + i32::from(actor.bounds.width) <= i32::from(viewport.width)
                && actor.bounds.y + i32::from(actor.bounds.height) <= i32::from(viewport.height),
            "{:?} escapes the viewport",
            actor.bounds
        );
        let painted = (actor.bounds.y..actor.bounds.y + i32::from(actor.bounds.height)).any(|y| {
            (actor.bounds.x..actor.bounds.x + i32::from(actor.bounds.width))
                .any(|x| buffer.get(x, y).is_some())
        });
        assert!(painted, "a roster actor must actually be painted");
    }
}

#[test]
fn roster_actors_never_share_a_silhouette_edge() {
    let snapshot = roster_snapshot(6);

    let (_, frame) = render_with_frame(&snapshot, PixelSize::new(60, 40));

    for (index, actor) in frame.actors.iter().enumerate() {
        for other in frame.actors.iter().skip(index + 1) {
            assert!(
                !overlaps(actor.bounds, other.bounds),
                "{:?} overlaps {:?}",
                actor.bounds,
                other.bounds
            );
        }
    }
}

#[test]
fn a_blocked_adventurer_carries_the_authored_counsel_marker_at_every_scale() {
    let snapshot = roster_snapshot(6);

    // Roster, compact and canonical all recompose the same party; a blocked
    // adventurer must be the fastest answer in every one of them.
    for viewport in [
        PixelSize::new(60, 40),
        PixelSize::new(100, 48),
        PixelSize::new(160, 90),
    ] {
        let (buffer, frame) = render_with_frame(&snapshot, viewport);
        let blocked = frame
            .actors
            .iter()
            .find(|actor| actor.agent == AgentKey::new("roster-1"))
            .expect("the blocked adventurer is visible");
        // The marker sits in the reserved lane directly above the actor.
        let lane = PixelRect::new(
            blocked.bounds.x - 4,
            (blocked.bounds.y - 8).max(0),
            u16::try_from(i32::from(blocked.bounds.width) + 8).unwrap(),
            8,
        );
        assert!(
            rect_contains(&buffer, lane, FLAME),
            "{viewport:?}: no authored counsel marker above the blocked adventurer"
        );
    }
}

#[test]
fn a_party_too_large_for_the_roster_still_falls_back_to_the_priority_vignette() {
    let snapshot = roster_snapshot(40);

    let (_, frame) = render_with_frame(&snapshot, PixelSize::new(20, 26));

    assert_eq!(
        frame.actors.len(),
        1,
        "the vignette remains the truthful last resort"
    );
    assert_eq!(
        (frame.actors[0].bounds.width, frame.actors[0].bounds.height),
        (16, 24),
        "the vignette keeps the full world master"
    );
}

fn overlaps(left: PixelRect, right: PixelRect) -> bool {
    left.x < right.x + i32::from(right.width)
        && left.x + i32::from(left.width) > right.x
        && left.y < right.y + i32::from(right.height)
        && left.y + i32::from(left.height) > right.y
}

#[test]
fn minimum_guild_hall_shows_one_complete_priority_adventurer() {
    let viewport = PixelSize::new(40, 36);
    let (_, frame) = render_with_frame(&mixed_snapshot(), viewport);

    assert_eq!(
        frame.actors.len(),
        1,
        "the minimum hall must become a coherent adventurer vignette"
    );
    let region = &frame.actors[0];
    assert_eq!(
        region.agent,
        AgentKey::new("counsel-seeker"),
        "the most urgent adventurer must survive minimum-size reduction"
    );
    assert_eq!(
        (region.bounds.width, region.bounds.height),
        (16, 24),
        "the priority adventurer must retain authored scale"
    );
    assert!(
        region.bounds.x >= 0
            && region.bounds.y >= 0
            && region.bounds.x + i32::from(region.bounds.width) <= i32::from(viewport.width)
            && region.bounds.y + i32::from(region.bounds.height) <= i32::from(viewport.height),
        "the priority adventurer is cropped outside the minimum hall: {:?}",
        region.bounds
    );
}

#[test]
fn compact_guild_hall_degrades_to_the_roster_before_hiding_adventurers() {
    let viewport = PixelSize::new(64, 40);
    let snapshot = mixed_snapshot();
    let expected = snapshot
        .agents
        .iter()
        .filter(|agent| agent.presence != Presence::Exited)
        .count();
    let (_, frame) = render_with_frame(&snapshot, viewport);

    assert_eq!(
        frame.actors.len(),
        expected,
        "a party too large for compact must recompose at roster scale, not vanish"
    );
    for actor in &frame.actors {
        assert_eq!(
            (actor.bounds.width, actor.bounds.height),
            (8, 12),
            "compact must not retain actors it cannot render completely"
        );
    }
    assert!(
        frame
            .actors
            .iter()
            .any(|actor| actor.agent == AgentKey::new("counsel-seeker"))
    );
    assert!(
        frame.actors[0].bounds.y + i32::from(frame.actors[0].bounds.height)
            <= i32::from(viewport.height)
    );
}

#[test]
fn focused_campaign_crop_uses_the_same_physical_table_as_its_actor() {
    let workspace = (0..100)
        .map(|index| format!("focus-parity-mismatch-{index}"))
        .find(|workspace| hash_parity(workspace) != 0)
        .expect("the deterministic search finds an odd workspace hash");
    let mut focused = agent(
        "focused-working",
        &workspace,
        Presence::Working,
        AccentTone::Cyan,
    );
    focused.focused = true;
    let snapshot = SceneSnapshot {
        connection: SceneConnection::Connected,
        campaigns: vec![campaign(&workspace, 41)],
        agents: vec![focused],
        motion: Motion::None,
        now: Timestamp::from_millis(10_000),
    };

    assert_eq!(hash_parity(&workspace), 1);
    let (buffer, frame) = render_with_frame(&snapshot, PixelSize::new(80, 48));
    let region = frame
        .actors
        .iter()
        .find(|region| region.agent == AgentKey::new("focused-working"))
        .expect("the focused crop retains its working adventurer region");
    assert!(
        (region.bounds.y..region.bounds.y + i32::from(region.bounds.height)).any(|y| {
            (region.bounds.x..region.bounds.x + i32::from(region.bounds.width))
                .any(|x| buffer.get(x, y).is_some_and(|pixel| pixel != VOID))
        }),
        "the focused 80px crop must retain its working actor"
    );
}

#[test]
fn campaign_adventurers_have_unique_anchors_per_physical_table_across_three_campaigns() {
    let snapshot = SceneSnapshot {
        connection: SceneConnection::Connected,
        campaigns: vec![
            campaign("left-first", 11),
            campaign("right-only", 17),
            campaign("left-second", 23),
        ],
        agents: vec![
            agent(
                "left-first-agent",
                "left-first",
                Presence::Working,
                AccentTone::Cyan,
            ),
            agent(
                "right-agent",
                "right-only",
                Presence::Working,
                AccentTone::Lime,
            ),
            agent(
                "left-second-agent",
                "left-second",
                Presence::Working,
                AccentTone::Magenta,
            ),
        ],
        motion: Motion::None,
        now: Timestamp::from_millis(10_000),
    };

    let (_, frame) = render_with_frame(&snapshot, VIEWPORT);
    let expected = [
        (
            "left-first-agent",
            PixelRect::new(35, 47, 38, 27),
            "first left-table adventurer",
        ),
        (
            "right-agent",
            PixelRect::new(77, 47, 38, 27),
            "right-table adventurer",
        ),
        (
            "left-second-agent",
            PixelRect::new(35, 47, 38, 27),
            "second left-table adventurer",
        ),
    ];
    let mut anchors = HashSet::new();
    for (key, bounds, label) in expected {
        let region = frame
            .actors
            .iter()
            .find(|region| region.agent == AgentKey::new(key))
            .unwrap_or_else(|| panic!("{label} has no rendered region"));
        let anchor = (
            region.bounds.x + i32::from(region.bounds.width / 2),
            region.bounds.y + i32::from(region.bounds.height) - 1,
        );
        assert!(
            anchor.0 >= bounds.x
                && anchor.0 < bounds.x + i32::from(bounds.width)
                && anchor.1 >= bounds.y
                && anchor.1 < bounds.y + i32::from(bounds.height),
            "{label} anchor {anchor:?} is outside {bounds:?}"
        );
        assert!(anchors.insert(anchor), "two actors share anchor {anchor:?}");
    }
}

#[test]
fn exited_agents_have_no_actor_pixels() {
    let mut without = mixed_snapshot();
    without.agents.clear();
    let mut with_exited = without.clone();
    with_exited.agents.push(agent(
        "departed",
        "amber-library",
        Presence::Exited,
        AccentTone::Red,
    ));
    assert_eq!(
        render(&without, VIEWPORT).pixels(),
        render(&with_exited, VIEWPORT).pixels()
    );
}

#[test]
fn connection_states_change_the_door_without_replacing_the_room() {
    let mut snapshot = mixed_snapshot();
    let mut buffers = HashMap::new();
    for (name, connection) in [
        ("connected", SceneConnection::Connected),
        ("connecting", SceneConnection::Connecting),
        ("reconnecting", SceneConnection::Reconnecting { attempt: 3 }),
        ("offline", SceneConnection::Offline),
        (
            "incompatible",
            SceneConnection::Incompatible {
                expected: 16,
                actual: 15,
            },
        ),
    ] {
        snapshot.connection = connection;
        let buffer = render(&snapshot, VIEWPORT);
        assert!(rect_contains(
            &buffer,
            PixelRect::new(34, 11, 43, 27),
            PARCHMENT
        ));
        buffers.insert(name, buffer);
    }
    let connected = &buffers["connected"];
    for state in ["connecting", "reconnecting", "offline", "incompatible"] {
        assert_ne!(
            connected.pixels(),
            buffers[state].pixels(),
            "{state} must be visible through the room"
        );
    }
    assert_ne!(
        buffers["connecting"].pixels(),
        buffers["offline"].pixels(),
        "lit opening and closed dark door must remain distinct"
    );
}

#[test]
fn minimum_door_crop_keeps_incompatible_versions_visible_over_the_room() {
    let snapshot = SceneSnapshot {
        connection: SceneConnection::Incompatible {
            expected: 16,
            actual: 15,
        },
        campaigns: Vec::new(),
        agents: Vec::new(),
        motion: Motion::None,
        now: Timestamp::from_millis(10_000),
    };

    let buffer = render(&snapshot, PixelSize::new(40, 36));
    assert!(
        buffer.pixels().contains(&PARCHMENT_LIGHT)
            && buffer.pixels().contains(&PARCHMENT_DARK)
            && buffer.pixels().contains(&INK_BLUE),
        "the minimum door crop must retain both protocol-version rows"
    );
    let room_colours = (0..36)
        .flat_map(|y| (0..19).map(move |x| (x, y)))
        .filter_map(|(x, y)| buffer.get(x, y))
        .collect::<HashSet<_>>();
    assert!(
        room_colours.len() >= 4,
        "the diagnostic must remain an overlay over the authored room; found {} colours",
        room_colours.len()
    );
}

#[test]
fn fresh_spoils_effect_ends_exactly_once_at_three_seconds() {
    let mut fresh = mixed_snapshot();
    fresh
        .agents
        .retain(|agent| agent.presence == Presence::Done);
    fresh.agents[0].transition = Some(SceneTransition {
        summons: GuildSummons::SpoilsReturned,
        since: Timestamp::from_millis(8_000),
    });
    let fresh_pixels = render(&fresh, VIEWPORT);

    let mut settled = fresh.clone();
    settled.now = Timestamp::from_millis(11_000);
    let settled_pixels = render(&settled, VIEWPORT);
    let mut no_transition = settled.clone();
    no_transition.agents[0].transition = None;
    let no_transition_pixels = render(&no_transition, VIEWPORT);

    assert_ne!(fresh_pixels.pixels(), settled_pixels.pixels());
    assert_eq!(settled_pixels.pixels(), no_transition_pixels.pixels());
}

#[test]
fn canonical_room_meets_density_colour_and_byte_determinism_floors() {
    let snapshot = mixed_snapshot();
    let first = render(&snapshot, VIEWPORT);
    let second = render(&snapshot, VIEWPORT);
    let clear = VOID;
    let painted = first
        .pixels()
        .iter()
        .filter(|pixel| **pixel != clear)
        .count();
    assert!(painted * 100 >= first.pixels().len() * 85);
    assert!(first.pixels().iter().copied().collect::<HashSet<_>>().len() >= 24);

    let bytes = |buffer: &RgbBuffer| {
        buffer
            .pixels()
            .iter()
            .flat_map(|pixel| [pixel.r, pixel.g, pixel.b])
            .collect::<Vec<_>>()
    };
    assert_eq!(blake3::hash(&bytes(&first)), blake3::hash(&bytes(&second)));
    assert!(first.pixels().contains(&SHADOW));
}

#[test]
fn static_guild_hall_is_event_driven_and_fresh_spoils_is_capped_at_eight_fps() {
    let mut snapshot = mixed_snapshot();
    snapshot.motion = Motion::Full;
    snapshot
        .agents
        .retain(|agent| agent.presence == Presence::Done);
    let mut target = RgbBuffer::filled(0, 0, Rgb::BLACK);
    let static_frame = render_scene_for_story(
        &snapshot,
        Some(WorldScene::GuildHall),
        VIEWPORT,
        &mut target,
    );
    assert_eq!(static_frame.next_frame_in, None);

    snapshot.agents[0].transition = Some(SceneTransition {
        summons: GuildSummons::SpoilsReturned,
        since: Timestamp::from_millis(8_000),
    });
    let animated_frame = render_scene_for_story(
        &snapshot,
        Some(WorldScene::GuildHall),
        VIEWPORT,
        &mut target,
    );
    assert_eq!(
        animated_frame.next_frame_in,
        Some(Duration::from_millis(125))
    );

    snapshot.now = Timestamp::from_millis(10_999);
    let deadline_frame = render_scene_for_story(
        &snapshot,
        Some(WorldScene::GuildHall),
        VIEWPORT,
        &mut target,
    );
    assert_eq!(deadline_frame.next_frame_in, Some(Duration::from_millis(1)));

    snapshot.now = Timestamp::from_millis(11_000);
    let settled_frame = render_scene_for_story(
        &snapshot,
        Some(WorldScene::GuildHall),
        VIEWPORT,
        &mut target,
    );
    assert_eq!(settled_frame.next_frame_in, None);
}

#[test]
fn campaign_table_motion_is_authored_and_does_not_wake_static_classes() {
    let mut barbarian = agent(
        "animated-barbarian",
        "amber-library",
        Presence::Working,
        AccentTone::Red,
    );
    barbarian.persona.class = AdventurerClass::Barbarian;
    let mut snapshot = SceneSnapshot {
        connection: SceneConnection::Connected,
        campaigns: vec![campaign("amber-library", 7)],
        agents: vec![barbarian],
        motion: Motion::Full,
        now: Timestamp::from_millis(1_000),
    };

    let (first, first_frame) = render_with_frame(&snapshot, VIEWPORT);
    assert_eq!(
        first_frame.next_frame_in,
        Some(Duration::from_millis(167)),
        "the authored working Barbarian pose should schedule its next frame"
    );

    snapshot.now = Timestamp::from_millis(1_167);
    let (second, second_frame) = render_with_frame(&snapshot, VIEWPORT);
    assert_ne!(
        first.pixels(),
        second.pixels(),
        "the scheduled frame must change visible authored pixels"
    );
    assert!(second_frame.next_frame_in.is_some());

    snapshot.agents[0].persona.class = AdventurerClass::Rogue;
    let (_, static_frame) = render_with_frame(&snapshot, VIEWPORT);
    assert_eq!(
        static_frame.next_frame_in, None,
        "a static class master must not keep the Guild Hall render loop awake"
    );
}
