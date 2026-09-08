#![cfg(feature = "storybook")]

use std::time::Duration;

use questmancer::{
    app::Motion,
    domain::{
        AdventurerClass, AdventurerPersona, AgentKey, GuildSummons, PersonaKey, Presence,
        Timestamp, WorkspaceId,
    },
    scene::{
        SceneFrame,
        pixel::{PixelSize, Rgb, RgbBuffer},
        render_scene_for_story,
        snapshot::{SceneAgent, SceneCampaign, SceneConnection, SceneSnapshot, SceneTransition},
        stage::WorldScene,
    },
};

const CLASSES: [AdventurerClass; 3] = [
    AdventurerClass::Wizard,
    AdventurerClass::Ranger,
    AdventurerClass::Barbarian,
];
const WORLDS: [WorldScene; 2] = [WorldScene::GuildHall, WorldScene::Delve];

fn snapshot(
    class: AdventurerClass,
    presence: Presence,
    motion: Motion,
    elapsed: i64,
) -> SceneSnapshot {
    let mut persona = AdventurerPersona::for_key(PersonaKey::new("party-pilot"));
    persona.class = class;
    SceneSnapshot {
        connection: SceneConnection::Connected,
        campaigns: vec![SceneCampaign {
            workspace_id: WorkspaceId::new("party-pilot"),
            label: "The pilot".to_owned(),
            variant_seed: 7,
        }],
        agents: vec![SceneAgent {
            key: AgentKey::new("pilot"),
            workspace_id: WorkspaceId::new("party-pilot"),
            name: "Pilot".to_owned(),
            custom_status: None,
            presence,
            presence_since: Timestamp::from_millis(1_000),
            transition: match presence {
                Presence::Done => Some(SceneTransition {
                    summons: GuildSummons::SpoilsReturned,
                    since: Timestamp::from_millis(1_000),
                }),
                Presence::Blocked => Some(SceneTransition {
                    summons: GuildSummons::CounselRequested,
                    since: Timestamp::from_millis(1_000),
                }),
                _ => None,
            },
            focused: false,
            persona,
        }],
        motion,
        now: Timestamp::from_millis(1_000 + elapsed),
    }
}

fn render(
    class: AdventurerClass,
    presence: Presence,
    motion: Motion,
    elapsed: i64,
    world: WorldScene,
) -> (RgbBuffer, SceneFrame) {
    let mut pixels = RgbBuffer::filled(0, 0, Rgb::BLACK);
    let frame = render_scene_for_story(
        &snapshot(class, presence, motion, elapsed),
        Some(world),
        PixelSize::new(160, 90),
        &mut pixels,
    );
    (pixels, frame)
}

#[test]
fn both_rooms_play_two_working_frames_at_two_fps_for_each_pilot_class() {
    for world in WORLDS {
        for class in CLASSES {
            let (first, frame) = render(class, Presence::Working, Motion::Full, 0, world);
            assert_eq!(frame.actors.len(), 1);
            assert_eq!(
                frame.next_frame_in,
                Some(Duration::from_millis(500)),
                "{world:?} {class:?}"
            );
            let (between, _) = render(class, Presence::Working, Motion::Full, 250, world);
            let (second, _) = render(class, Presence::Working, Motion::Full, 500, world);
            let (looped, _) = render(class, Presence::Working, Motion::Full, 1_000, world);
            assert_eq!(
                first, between,
                "no frame changes between authored boundaries"
            );
            assert_ne!(
                first, second,
                "{world:?} {class:?}: the ritual must reach the room"
            );
            assert_eq!(first, looped, "two frames make one one-second loop");
        }
    }
}

#[test]
fn a_counsel_gesture_settles_once_and_leaves_the_blocked_adventurer_still() {
    for world in WORLDS {
        for class in CLASSES {
            let (raised, first) = render(class, Presence::Blocked, Motion::Full, 0, world);
            assert_eq!(
                first.next_frame_in,
                Some(Duration::from_millis(600)),
                "{world:?} {class:?}"
            );
            let (_, last) = render(class, Presence::Blocked, Motion::Full, 599, world);
            assert_eq!(last.next_frame_in, Some(Duration::from_millis(1)));
            let (waiting, settled) = render(class, Presence::Blocked, Motion::Full, 600, world);
            let (later, quiet) = render(class, Presence::Blocked, Motion::Full, 60_000, world);
            assert_ne!(raised, waiting, "the raised hand must settle");
            assert_eq!(waiting, later);
            assert_eq!(settled.next_frame_in, None);
            assert_eq!(quiet.next_frame_in, None);
        }
    }
}

#[test]
fn reduced_and_still_party_states_do_not_change_pixels_at_cleanup_deadlines() {
    for world in WORLDS {
        for class in CLASSES {
            for motion in [Motion::Reduced, Motion::None] {
                for presence in [
                    Presence::Working,
                    Presence::Blocked,
                    Presence::Idle,
                    Presence::Unknown,
                    Presence::Done,
                ] {
                    let (first, frame) = render(class, presence, motion, 0, world);
                    let (later, quiet) = render(class, presence, motion, 60_000, world);
                    assert_eq!(first, later, "{world:?} {class:?} {motion:?} {presence:?}");
                    assert_eq!(frame.next_frame_in, None);
                    assert_eq!(quiet.next_frame_in, None);
                }
            }
        }
    }
}

#[test]
fn each_return_places_its_spoils_then_settles_at_the_exact_three_second_deadline() {
    for world in WORLDS {
        for class in CLASSES {
            let (arrival, first) = render(class, Presence::Done, Motion::Full, 0, world);
            let (placed, _) = render(class, Presence::Done, Motion::Full, 1_000, world);
            assert_ne!(
                arrival, placed,
                "{world:?} {class:?}: the parcel should be placed"
            );
            assert_eq!(first.next_frame_in, Some(Duration::from_millis(125)));
            let (_, last) = render(class, Presence::Done, Motion::Full, 2_999, world);
            assert_eq!(last.next_frame_in, Some(Duration::from_millis(1)));
            let (settled, frame) = render(class, Presence::Done, Motion::Full, 3_000, world);
            let (later, quiet) = render(class, Presence::Done, Motion::Full, 60_000, world);
            assert_eq!(settled, later);
            assert_eq!(frame.next_frame_in, None);
            assert_eq!(quiet.next_frame_in, None);
            let (working, _) = render(class, Presence::Working, Motion::None, 0, world);
            let (resting, _) = render(class, Presence::Idle, Motion::None, 0, world);
            assert_ne!(settled, working, "completed must not read as working");
            assert_ne!(settled, resting, "completed must not read as resting");
        }
    }
}

#[test]
fn every_pilot_pose_keeps_its_foot_anchor_and_native_dimensions() {
    use questmancer::scene::{
        assets::adventurer::{adventurer_animation_frame, adventurer_roster_frame},
        stage::ScenePose,
    };
    for class in CLASSES {
        let persona = snapshot(class, Presence::Working, Motion::None, 0)
            .agents
            .remove(0)
            .persona;
        let mut anchor = None;
        for pose in [
            ScenePose::Working,
            ScenePose::SeekingCounsel,
            ScenePose::ReturningWithSpoils,
            ScenePose::Settled,
            ScenePose::Resting,
            ScenePose::Unknown,
        ] {
            for index in 0..2 {
                let frame = adventurer_animation_frame(&persona, pose, index);
                assert_eq!(frame.size(), PixelSize::new(16, 24));
                let last = frame.pixels().iter().rposition(Option::is_some).unwrap();
                assert_eq!(last / 16, 21, "{class:?} {pose:?} {index}");
                let row = &frame.pixels()[21 * 16..22 * 16];
                let centre = row.iter().position(Option::is_some).unwrap()
                    + row.iter().rposition(Option::is_some).unwrap();
                if let Some(anchor) = anchor {
                    assert_eq!(centre, anchor, "pose must not shift the feet");
                }
                anchor = Some(centre);
            }
        }
        let roster = adventurer_roster_frame(&persona);
        assert_eq!(roster.size(), PixelSize::new(8, 12));
        assert_eq!(
            roster.pixels().iter().rposition(Option::is_some).unwrap() / 8,
            10
        );
    }
}
