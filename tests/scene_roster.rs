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
        presentation::{SceneOverlay, ScenePresentation},
        render_scene_for_world,
        snapshot::{SceneAgent, SceneConnection, SceneSnapshot, SceneTransition},
        stage::WorldScene,
    },
};

const WORLDS: [WorldScene; 2] = [WorldScene::GuildHall, WorldScene::Delve];

fn snapshot(presence: Presence, motion: Motion, now: i64) -> SceneSnapshot {
    let mut persona = AdventurerPersona::for_key(PersonaKey::new("roster-state"));
    persona.class = AdventurerClass::Wizard;
    SceneSnapshot {
        connection: SceneConnection::Connected,
        campaigns: Vec::new(),
        agents: vec![SceneAgent {
            key: AgentKey::new("same-adventurer"),
            workspace_id: WorkspaceId::new("same-campaign"),
            name: "Willow".to_owned(),
            custom_status: None,
            presence,
            presence_since: Timestamp::from_millis(1_000),
            transition: (presence == Presence::Done).then_some(SceneTransition {
                summons: GuildSummons::SpoilsReturned,
                since: Timestamp::from_millis(1_000),
            }),
            focused: false,
            persona,
        }],
        motion,
        now: Timestamp::from_millis(now),
    }
}

fn render(snapshot: &SceneSnapshot, world: WorldScene) -> (RgbBuffer, SceneFrame) {
    render_at_size(snapshot, world, PixelSize::new(30, 30))
}

fn render_at_size(
    snapshot: &SceneSnapshot,
    world: WorldScene,
    size: PixelSize,
) -> (RgbBuffer, SceneFrame) {
    let mut pixels = RgbBuffer::filled(0, 0, Rgb::BLACK);
    let presentation = ScenePresentation {
        world,
        selected_agent: Some(snapshot.agents[0].key.clone()),
        transition_floor: None,
        overlay: SceneOverlay::None,
        goblin_outbreak: false,
    };
    let frame = render_scene_for_world(snapshot, &presentation, size, &mut pixels);
    (pixels, frame)
}

#[test]
fn label_free_rosters_distinguish_every_live_presence() {
    let states = [
        Presence::Working,
        Presence::Idle,
        Presence::Unknown,
        Presence::Blocked,
        Presence::Done,
    ];
    for world in WORLDS {
        let renders = states.map(|presence| {
            let (pixels, frame) = render(&snapshot(presence, Motion::None, 4_000), world);
            assert_eq!(frame.actors.len(), 1);
            assert_eq!(frame.actors[0].bounds.width, 8);
            assert_eq!(frame.actors[0].bounds.height, 12);
            assert_eq!(frame.next_frame_in, None);
            pixels
        });
        for left in 0..states.len() {
            for right in left + 1..states.len() {
                assert!(
                    renders[left] != renders[right],
                    "{world:?}: {:?} aliases {:?} without labels",
                    states[left],
                    states[right]
                );
            }
        }
    }
}

#[test]
fn roster_completion_moves_then_stops_at_its_three_second_deadline() {
    for world in WORLDS {
        let mut scene = snapshot(Presence::Done, Motion::Full, 1_125);
        let (first, frame) = render(&scene, world);
        assert_eq!(
            frame.next_frame_in,
            Some(Duration::from_millis(125)),
            "{world:?}"
        );
        scene.now = Timestamp::from_millis(1_250);
        assert_ne!(
            first,
            render(&scene, world).0,
            "{world:?}: theatre never moves"
        );
        scene.now = Timestamp::from_millis(3_999);
        assert_eq!(
            render(&scene, world).1.next_frame_in,
            Some(Duration::from_millis(1))
        );
        scene.now = Timestamp::from_millis(4_000);
        let (settled, frame) = render(&scene, world);
        assert_eq!(frame.next_frame_in, None);
        scene.now = Timestamp::from_millis(30_000);
        assert_eq!(settled, render(&scene, world).0);
    }
}

#[test]
fn reduced_and_still_rosters_keep_one_stable_completion_cue() {
    for world in WORLDS {
        for motion in [Motion::Reduced, Motion::None] {
            let (fresh, frame) = render(&snapshot(Presence::Done, motion, 1_125), world);
            assert_eq!(frame.next_frame_in, None);
            let (settled, frame) = render(&snapshot(Presence::Done, motion, 4_000), world);
            assert_eq!(frame.next_frame_in, None);
            assert_eq!(
                fresh, settled,
                "{world:?}: {motion:?} needs a cleanup redraw"
            );
        }
    }
}

#[test]
fn newer_presence_cancels_stale_completion_theatre() {
    for world in WORLDS {
        for presence in [
            Presence::Working,
            Presence::Blocked,
            Presence::Idle,
            Presence::Unknown,
            Presence::Exited,
        ] {
            let mut scene = snapshot(Presence::Done, Motion::Full, 1_125);
            scene.agents[0].presence = presence;
            scene.agents[0].presence_since = Timestamp::from_millis(1_100);
            let (stale, frame) = render(&scene, world);
            assert_eq!(frame.next_frame_in, None, "{world:?}: {presence:?}");
            scene.agents[0].transition = None;
            assert_eq!(stale, render(&scene, world).0);
        }
    }
}

#[test]
fn a_disconnected_roster_cannot_perform_completion_theatre() {
    for world in WORLDS {
        for connection in [
            SceneConnection::Offline,
            SceneConnection::Connecting,
            SceneConnection::Reconnecting { attempt: 1 },
            SceneConnection::Incompatible {
                expected: 16,
                actual: 15,
            },
        ] {
            let mut scene = snapshot(Presence::Done, Motion::Full, 1_125);
            scene.connection = connection;
            let (first, frame) = render(&scene, world);
            assert_eq!(
                frame.next_frame_in, None,
                "{world:?}: {:?}",
                scene.connection
            );
            scene.now = Timestamp::from_millis(1_250);
            assert!(first == render(&scene, world).0);
        }
    }
}

#[test]
fn full_roster_rows_keep_cues_selection_and_complete_hits_clear_of_each_other() {
    use questmancer::scene::{
        assets::{palette::SELECTION_RUNE, roster},
        stage::ScenePose,
    };
    for world in WORLDS {
        for connection in [
            SceneConnection::Connected,
            SceneConnection::Reconnecting { attempt: 6 },
            SceneConnection::Incompatible {
                expected: 16,
                actual: 15,
            },
        ] {
            let mut scene = snapshot(Presence::Blocked, Motion::None, 4_000);
            scene.connection = connection;
            let template = scene.agents[0].clone();
            scene.agents = (0..6)
                .map(|index| {
                    let mut agent = template.clone();
                    agent.key = AgentKey::new(format!("adventurer-{index}"));
                    agent
                })
                .collect();
            let (pixels, frame) = render_at_size(&scene, world, PixelSize::new(30, 47));
            assert_eq!(frame.actors.len(), 6);
            let marker = roster::state_marker(ScenePose::SeekingCounsel);
            for region in &frame.actors {
                let body = region.bounds;
                assert_eq!((body.width, body.height), (8, 12));
                assert!(body.x >= 0 && body.y >= 0 && body.x + 8 <= 30 && body.y + 12 <= 47);
                assert_eq!(
                    frame.agent_at(
                        u16::try_from(body.x + 4).unwrap(),
                        u16::try_from((body.y + 6).div_euclid(2)).unwrap()
                    ),
                    Some(&region.agent)
                );
                for (index, colour) in marker.pixels().iter().enumerate() {
                    if let Some(colour) = colour {
                        let x = body.x + 2 + i32::try_from(index % 5).unwrap();
                        let y = body.y - 5 + i32::try_from(index / 5).unwrap();
                        assert_eq!(
                            pixels.get(x, y),
                            Some(*colour),
                            "{world:?}/{:?}: clipped or covered state cue",
                            scene.connection
                        );
                    }
                }
            }
            let selected = frame.actors[0].bounds;
            for x in selected.x + 2..selected.x + 6 {
                assert_eq!(
                    pixels.get(x, selected.y + 14),
                    Some(SELECTION_RUNE),
                    "next row must not overwrite the selection ring"
                );
            }
        }
    }
}

#[test]
fn roster_state_shapes_remain_distinct_after_ansi16_half_block_conversion() {
    use questmancer::{app::ColorMode, ui::scene_adapter::flush_rgb};
    use ratatui::{buffer::Buffer, layout::Rect};
    for world in WORLDS {
        let cells = [
            Presence::Working,
            Presence::Idle,
            Presence::Unknown,
            Presence::Blocked,
            Presence::Done,
        ]
        .map(|presence| {
            let (pixels, _) = render(&snapshot(presence, Motion::None, 4_000), world);
            let area = Rect::new(0, 0, 30, 15);
            let mut cells = Buffer::empty(area);
            flush_rgb(&mut cells, area, &pixels, Rgb::BLACK, ColorMode::Ansi16);
            cells
        });
        for left in 0..cells.len() {
            for right in left + 1..cells.len() {
                assert!(
                    cells[left] != cells[right],
                    "{world:?}: states collapse in ANSI 16"
                );
            }
        }
    }
}

#[tokio::test(start_paused = true)]
async fn final_roster_frame_disarms_the_real_animation_scheduler() {
    use questmancer::terminal::{AnimationScheduler, RuntimeClock};
    for world in WORLDS {
        let clock = RuntimeClock::new(Timestamp::from_millis(3_999));
        let mut scene = snapshot(Presence::Done, Motion::Full, 3_999);
        let mut scheduler = AnimationScheduler::new();
        scheduler.reset_after(scene.now, render(&scene, world).1.next_frame_in, &clock);
        tokio::time::timeout(Duration::from_millis(2), scheduler.wait())
            .await
            .unwrap();
        scene.now = clock.now();
        assert_eq!(scene.now, Timestamp::from_millis(4_000));
        let (_, settled) = render(&scene, world);
        assert_eq!(settled.next_frame_in, None);
        scheduler.reset_after(scene.now, settled.next_frame_in, &clock);
        assert!(
            tokio::time::timeout(Duration::from_secs(60), scheduler.wait())
                .await
                .is_err()
        );
    }
}
