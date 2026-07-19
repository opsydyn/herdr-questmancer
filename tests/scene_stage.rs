use std::time::Duration;

use questmancer::{
    app::Motion,
    domain::{
        AdventurerPersona, AgentKey, GuildSummons, PersonaKey, Presence, Timestamp, WorkspaceId,
    },
    scene::{
        pixel::{PixelSize, Rgb, RgbBuffer},
        render_scene,
        snapshot::{SceneAgent, SceneConnection, SceneSnapshot, SceneTransition},
        stage::{
            ActorPlacement, COMPLETION_THEATRE_MS, CameraAnchor, SceneCadence, SceneCamera,
            SceneEffect, ScenePlan, ScenePose, TruthfulStation, WorldScene,
        },
    },
};

fn render(
    snapshot: &SceneSnapshot,
    viewport: PixelSize,
) -> (questmancer::scene::SceneFrame, RgbBuffer) {
    let mut target = RgbBuffer::filled(1, 1, Rgb::new(255, 0, 255));
    let frame = render_scene(snapshot, viewport, &mut target);
    (frame, target)
}

fn contains_environment_palette_pixel(world: WorldScene, target: &RgbBuffer) -> bool {
    let palette: &[Rgb] = match world {
        WorldScene::GuildHall => &[
            Rgb::new(48, 45, 54),
            Rgb::new(75, 70, 74),
            Rgb::new(105, 96, 92),
            Rgb::new(59, 36, 29),
            Rgb::new(104, 63, 37),
            Rgb::new(151, 93, 48),
            Rgb::new(76, 25, 35),
            Rgb::new(130, 43, 48),
            Rgb::new(196, 126, 48),
            Rgb::new(67, 22, 31),
        ],
        WorldScene::Delve => &[
            Rgb::new(22, 43, 49),
            Rgb::new(37, 65, 68),
            Rgb::new(68, 96, 94),
            Rgb::new(23, 50, 54),
            Rgb::new(35, 75, 72),
            Rgb::new(31, 67, 45),
            Rgb::new(68, 104, 62),
            Rgb::new(21, 52, 60),
            Rgb::new(33, 69, 75),
            Rgb::new(58, 94, 96),
            Rgb::new(22, 57, 64),
            Rgb::new(31, 77, 78),
            Rgb::new(28, 71, 57),
            Rgb::new(58, 100, 70),
            Rgb::new(31, 94, 93),
        ],
    };
    target.pixels().iter().any(|pixel| palette.contains(pixel))
}

fn assert_authored_camera_crop(
    world: WorldScene,
    snapshot: &SceneSnapshot,
    expected_world_origin: (i32, i32),
) {
    let viewport = PixelSize::new(80, 48);
    let (_, crop) = render(snapshot, viewport);
    let (_, reference) = render(snapshot, PixelSize::new(160, 90));
    let reference_origin = expected_world_origin;

    assert!(reference_origin.0 >= 0 && reference_origin.1 >= 0);
    assert!(reference_origin.0 + 80 <= 160 && reference_origin.1 + 48 <= 90);
    for y in 0..48 {
        for x in 0..80 {
            assert_eq!(
                crop.get(x, y),
                reference.get(x + reference_origin.0, y + reference_origin.1),
                "{world:?} camera differed at crop pixel ({x}, {y})"
            );
        }
    }
    assert!(contains_environment_palette_pixel(world, &crop));
}

fn render_at(snapshot: &SceneSnapshot, now: i64) -> (Vec<Rgb>, Option<Duration>) {
    let mut snapshot = snapshot.clone();
    snapshot.now = Timestamp::from_millis(now);
    let (frame, target) = render(&snapshot, PixelSize::new(160, 90));
    (target.pixels().to_vec(), frame.next_frame_in)
}

fn agent(key: &str, presence: Presence) -> SceneAgent {
    SceneAgent {
        key: AgentKey::new(key),
        workspace_id: WorkspaceId::new(format!("workspace-{key}")),
        name: format!("Agent {key}"),
        custom_status: None,
        presence,
        presence_since: Timestamp::from_millis(1_000),
        transition: None,
        focused: false,
        persona: AdventurerPersona::for_key(PersonaKey::new(format!("persona-{key}"))),
    }
}

fn snapshot(agents: Vec<SceneAgent>) -> SceneSnapshot {
    SceneSnapshot {
        connection: SceneConnection::Connected,
        campaigns: Vec::new(),
        agents,
        motion: Motion::Full,
        now: Timestamp::from_millis(5_000),
    }
}

fn project(snapshot: &SceneSnapshot) -> ScenePlan {
    ScenePlan::project(snapshot, PixelSize::new(160, 90))
}

#[test]
fn automatic_world_choice_uses_the_locked_priority() {
    let mut working = agent("working", Presence::Working);
    working.focused = true;

    let mut non_connected = snapshot(vec![working.clone()]);
    non_connected.connection = SceneConnection::Reconnecting { attempt: 3 };

    let blocked = snapshot(vec![agent("blocked", Presence::Blocked), working.clone()]);

    let mut fresh = agent("done", Presence::Done);
    fresh.transition = Some(SceneTransition {
        summons: GuildSummons::SpoilsReturned,
        since: Timestamp::from_millis(2_001),
    });

    let cases = [
        (non_connected, WorldScene::GuildHall),
        (blocked, WorldScene::GuildHall),
        (
            snapshot(vec![fresh, working.clone()]),
            WorldScene::GuildHall,
        ),
        (snapshot(vec![working]), WorldScene::Delve),
        (
            snapshot(vec![agent("working", Presence::Working)]),
            WorldScene::Delve,
        ),
        (
            snapshot(vec![agent("idle", Presence::Idle)]),
            WorldScene::GuildHall,
        ),
    ];

    for (snapshot, expected) in cases {
        assert_eq!(project(&snapshot).world, expected);
    }
}

#[test]
fn completion_theatre_is_fresh_before_but_not_at_three_seconds() {
    assert_eq!(COMPLETION_THEATRE_MS, 3_000);

    for (since, expected_world, expected_effects) in [
        (2_001, WorldScene::GuildHall, 1),
        (2_000, WorldScene::Delve, 0),
        (5_001, WorldScene::Delve, 0),
    ] {
        let mut completed = agent("done", Presence::Done);
        completed.transition = Some(SceneTransition {
            summons: GuildSummons::SpoilsReturned,
            since: Timestamp::from_millis(since),
        });
        let snapshot = snapshot(vec![completed, agent("working", Presence::Working)]);
        let plan = project(&snapshot);
        assert_eq!(plan.world, expected_world, "since={since}");
        assert_eq!(plan.effects.len(), expected_effects, "since={since}");
    }
}

#[test]
fn guild_hall_places_each_truthful_actor_once_and_keeps_departures_as_effects() {
    let mut fresh_done = agent("d-fresh", Presence::Done);
    fresh_done.transition = Some(SceneTransition {
        summons: GuildSummons::SpoilsReturned,
        since: Timestamp::from_millis(2_001),
    });
    let mut exited = agent("g-exited", Presence::Exited);
    exited.transition = Some(SceneTransition {
        summons: GuildSummons::AdventurerDeparted,
        since: Timestamp::from_millis(2_500),
    });
    let snapshot = snapshot(vec![
        agent("a-working", Presence::Working),
        agent("b-unknown", Presence::Unknown),
        agent("c-blocked", Presence::Blocked),
        fresh_done,
        agent("e-settled", Presence::Done),
        agent("f-idle", Presence::Idle),
        exited,
    ]);

    let plan = project(&snapshot);

    assert_eq!(plan.world, WorldScene::GuildHall);
    assert_eq!(
        plan.actors,
        vec![
            ActorPlacement {
                agent: AgentKey::new("a-working"),
                station: TruthfulStation::CampaignToken(WorkspaceId::new("workspace-a-working")),
                pose: ScenePose::Working,
                focused: false,
            },
            ActorPlacement {
                agent: AgentKey::new("b-unknown"),
                station: TruthfulStation::CampaignToken(WorkspaceId::new("workspace-b-unknown")),
                pose: ScenePose::Unknown,
                focused: false,
            },
            ActorPlacement {
                agent: AgentKey::new("c-blocked"),
                station: TruthfulStation::CounselBell,
                pose: ScenePose::SeekingCounsel,
                focused: false,
            },
            ActorPlacement {
                agent: AgentKey::new("d-fresh"),
                station: TruthfulStation::SpoilsBench,
                pose: ScenePose::ReturningWithSpoils,
                focused: false,
            },
            ActorPlacement {
                agent: AgentKey::new("e-settled"),
                station: TruthfulStation::SpoilsBench,
                pose: ScenePose::Settled,
                focused: false,
            },
            ActorPlacement {
                agent: AgentKey::new("f-idle"),
                station: TruthfulStation::Hearth,
                pose: ScenePose::Resting,
                focused: false,
            },
        ]
    );
    assert_eq!(
        plan.effects,
        vec![
            SceneEffect::FreshSpoils {
                agent: AgentKey::new("d-fresh"),
                since: Timestamp::from_millis(2_001),
            },
            SceneEffect::RecentDeparture {
                workspace_id: WorkspaceId::new("workspace-g-exited"),
                since: Timestamp::from_millis(2_500),
            },
        ]
    );
}

#[test]
fn automatic_delve_places_supported_states_at_truthful_stations() {
    let snapshot = snapshot(vec![
        agent("a-working", Presence::Working),
        agent("b-done", Presence::Done),
        agent("c-idle", Presence::Idle),
        agent("d-unknown", Presence::Unknown),
        agent("e-exited", Presence::Exited),
    ]);

    let plan = project(&snapshot);

    assert_eq!(plan.world, WorldScene::Delve);
    assert_eq!(
        plan.actors,
        vec![
            ActorPlacement {
                agent: AgentKey::new("a-working"),
                station: TruthfulStation::DelveActive(WorkspaceId::new("workspace-a-working")),
                pose: ScenePose::Working,
                focused: false,
            },
            ActorPlacement {
                agent: AgentKey::new("b-done"),
                station: TruthfulStation::DelveExit(WorkspaceId::new("workspace-b-done")),
                pose: ScenePose::Settled,
                focused: false,
            },
            ActorPlacement {
                agent: AgentKey::new("c-idle"),
                station: TruthfulStation::DelveCamp(WorkspaceId::new("workspace-c-idle")),
                pose: ScenePose::Resting,
                focused: false,
            },
            ActorPlacement {
                agent: AgentKey::new("d-unknown"),
                station: TruthfulStation::DelveActive(WorkspaceId::new("workspace-d-unknown")),
                pose: ScenePose::Unknown,
                focused: false,
            },
        ]
    );
}

#[test]
fn camera_uses_room_threshold_and_small_view_priority() {
    assert_eq!(
        ScenePlan::project(
            &snapshot(vec![agent("blocked", Presence::Blocked)]),
            PixelSize::new(120, 72)
        )
        .camera,
        SceneCamera::WholeRoom
    );

    let mut fresh = agent("fresh", Presence::Done);
    fresh.transition = Some(SceneTransition {
        summons: GuildSummons::SpoilsReturned,
        since: Timestamp::from_millis(2_001),
    });
    let mut focused = agent("focused", Presence::Working);
    focused.focused = true;
    let cases = [
        (
            snapshot(vec![
                agent("blocked", Presence::Blocked),
                fresh.clone(),
                focused.clone(),
            ]),
            CameraAnchor::CounselBell,
        ),
        (snapshot(vec![fresh, focused.clone()]), CameraAnchor::Spoils),
        (
            {
                let mut value = snapshot(vec![focused]);
                value.connection = SceneConnection::Offline;
                value
            },
            CameraAnchor::CampaignTable(WorkspaceId::new("workspace-focused")),
        ),
        (
            {
                let mut idle = agent("idle", Presence::Idle);
                idle.focused = true;
                snapshot(vec![idle])
            },
            CameraAnchor::Hearth,
        ),
        (snapshot(Vec::new()), CameraAnchor::Door),
    ];

    for (snapshot, anchor) in cases {
        assert_eq!(
            ScenePlan::project(&snapshot, PixelSize::new(119, 71)).camera,
            SceneCamera::Focused { anchor }
        );
    }

    assert_eq!(
        ScenePlan::project(
            &snapshot(vec![agent("working", Presence::Working)]),
            PixelSize::new(119, 71)
        )
        .camera,
        SceneCamera::Focused {
            anchor: CameraAnchor::Door,
        }
    );
}

#[test]
fn focus_changes_only_emphasis_and_crop_not_station_or_identity() {
    let unfocused = snapshot(vec![agent("working", Presence::Working)]);
    let mut focused_agent = agent("working", Presence::Working);
    focused_agent.focused = true;
    let focused = snapshot(vec![focused_agent]);
    let viewport = PixelSize::new(80, 50);

    let unfocused_plan = ScenePlan::project(&unfocused, viewport);
    let focused_plan = ScenePlan::project(&focused, viewport);

    assert_eq!(unfocused_plan.actors[0].agent, focused_plan.actors[0].agent);
    assert_eq!(
        unfocused_plan.actors[0].station,
        focused_plan.actors[0].station
    );
    assert_eq!(unfocused_plan.actors[0].pose, focused_plan.actors[0].pose);
    assert!(!unfocused_plan.actors[0].focused);
    assert!(focused_plan.actors[0].focused);
    assert_eq!(
        focused_plan.camera,
        SceneCamera::Focused {
            anchor: CameraAnchor::DelveParty(AgentKey::new("working")),
        }
    );
}

#[test]
fn cadence_is_derived_only_from_motion_and_visible_needs() {
    let mut fresh = agent("fresh", Presence::Done);
    fresh.transition = Some(SceneTransition {
        summons: GuildSummons::SpoilsReturned,
        since: Timestamp::from_millis(2_001),
    });

    let full_cases = [
        (
            vec![agent("unknown", Presence::Unknown)],
            SceneCadence::EventDriven,
        ),
        (
            vec![agent("settled", Presence::Done)],
            SceneCadence::EventDriven,
        ),
        (vec![agent("idle", Presence::Idle)], SceneCadence::Fps(1)),
        (
            vec![agent("blocked", Presence::Blocked)],
            SceneCadence::Fps(2),
        ),
        (
            vec![agent("working", Presence::Working)],
            SceneCadence::Fps(6),
        ),
        (
            vec![agent("working", Presence::Working), fresh.clone()],
            SceneCadence::Fps(8),
        ),
    ];
    for (agents, expected) in full_cases {
        assert_eq!(project(&snapshot(agents)).cadence, expected);
    }

    let mut reduced_idle = snapshot(vec![
        agent("working", Presence::Working),
        agent("idle", Presence::Idle),
    ]);
    reduced_idle.motion = Motion::Reduced;
    assert_eq!(project(&reduced_idle).cadence, SceneCadence::Fps(1));

    let mut reduced_working = snapshot(vec![agent("working", Presence::Working)]);
    reduced_working.motion = Motion::Reduced;
    assert_eq!(project(&reduced_working).cadence, SceneCadence::EventDriven);

    let mut no_motion = snapshot(vec![agent("idle", Presence::Idle), fresh]);
    no_motion.motion = Motion::None;
    assert_eq!(project(&no_motion).cadence, SceneCadence::EventDriven);
}

#[test]
fn viewport_matrix_preserves_exact_targets_and_authored_camera_crops() {
    let guild = snapshot(Vec::new());
    let mut delve_agent = agent("working", Presence::Working);
    delve_agent.focused = true;
    let delve = snapshot(vec![delve_agent]);

    for viewport in [
        PixelSize::new(0, 0),
        PixelSize::new(1, 1),
        PixelSize::new(80, 48),
        PixelSize::new(120, 72),
        PixelSize::new(160, 90),
        PixelSize::new(240, 120),
    ] {
        for (expected_world, value) in
            [(WorldScene::GuildHall, &guild), (WorldScene::Delve, &delve)]
        {
            let (frame, target) = render(value, viewport);
            assert_eq!(frame.world, expected_world);
            assert_eq!(target.size(), viewport);
            if viewport.width > 0 && viewport.height > 0 {
                assert!(!target.pixels().is_empty());
                assert!(
                    target
                        .pixels()
                        .iter()
                        .all(|pixel| *pixel != Rgb::new(255, 0, 255)),
                    "{expected_world:?} left an unpainted pixel at {viewport:?}"
                );
                assert!(
                    contains_environment_palette_pixel(expected_world, &target),
                    "{expected_world:?} omitted every known environment/material colour at {viewport:?}"
                );
            }
        }
    }

    let mut blocked = agent("blocked", Presence::Blocked);
    blocked.focused = true;
    let guild_focus = snapshot(vec![blocked]);
    assert_authored_camera_crop(WorldScene::GuildHall, &guild_focus, (80, 24));
    assert_authored_camera_crop(WorldScene::Delve, &delve, (41, 23));

    for value in [&guild, &delve] {
        let (_, canonical) = render(value, PixelSize::new(160, 90));
        let (_, surplus) = render(value, PixelSize::new(240, 120));
        for y in 0..90 {
            for x in 0..160 {
                assert_eq!(canonical.get(x, y), surplus.get(x + 40, y + 15));
            }
        }
    }
}

#[test]
fn exact_motion_phases_match_their_visible_cadence() {
    let working = snapshot(vec![agent("working", Presence::Working)]);
    let (working_start, working_deadline) = render_at(&working, 1_000);
    let (working_after_five_steps, working_late_deadline) = render_at(&working, 1_996);
    let (working_after_six_steps, _) = render_at(&working, 2_000);
    assert_eq!(working_deadline, Some(Duration::from_millis(167)));
    assert_eq!(working_late_deadline, Some(Duration::from_millis(4)));
    assert_ne!(working_start, working_after_five_steps);
    assert_eq!(working_start, working_after_six_steps);

    let blocked = snapshot(vec![agent("blocked", Presence::Blocked)]);
    let (blocked_start, blocked_deadline) = render_at(&blocked, 1_000);
    let (blocked_before, _) = render_at(&blocked, 1_499);
    let (blocked_next, _) = render_at(&blocked, 1_500);
    assert_eq!(blocked_deadline, Some(Duration::from_millis(500)));
    assert_eq!(blocked_start, blocked_before);
    assert_ne!(blocked_before, blocked_next);

    let (blocked_phase_two, blocked_phase_two_deadline) = render_at(&blocked, 2_000);
    let (blocked_unchanged_boundary, _) = render_at(&blocked, 2_500);
    let (blocked_visible_change, _) = render_at(&blocked, 3_000);
    assert_eq!(
        blocked_phase_two_deadline,
        Some(Duration::from_millis(1_000))
    );
    assert_eq!(blocked_phase_two, blocked_unchanged_boundary);
    assert_ne!(blocked_unchanged_boundary, blocked_visible_change);

    let mut completed = agent("completed", Presence::Done);
    completed.transition = Some(SceneTransition {
        summons: GuildSummons::SpoilsReturned,
        since: Timestamp::from_millis(1_000),
    });
    let fresh = snapshot(vec![completed]);
    let (fresh_start, fresh_deadline) = render_at(&fresh, 1_000);
    let (fresh_before, _) = render_at(&fresh, 1_124);
    let (fresh_next, _) = render_at(&fresh, 1_125);
    assert_eq!(fresh_deadline, Some(Duration::from_millis(125)));
    assert_eq!(fresh_start, fresh_before);
    assert_ne!(fresh_before, fresh_next);

    let idle = snapshot(vec![agent("idle", Presence::Idle)]);
    let (idle_start, idle_deadline) = render_at(&idle, 1_000);
    let (idle_before, _) = render_at(&idle, 1_999);
    let (idle_next, _) = render_at(&idle, 2_000);
    assert_eq!(idle_deadline, Some(Duration::from_millis(1_000)));
    assert_eq!(idle_start, idle_before);
    assert_ne!(idle_before, idle_next);
}

#[test]
fn static_and_no_motion_frames_have_no_idle_rendering_cost() {
    let settled = snapshot(vec![agent("settled", Presence::Done)]);
    let (_, settled_deadline) = render_at(&settled, 1_000);
    assert_eq!(settled_deadline, None);

    let mut completed = agent("completed", Presence::Done);
    completed.transition = Some(SceneTransition {
        summons: GuildSummons::SpoilsReturned,
        since: Timestamp::from_millis(1_000),
    });
    let mut no_motion = snapshot(vec![completed]);
    no_motion.motion = Motion::None;
    let (first, first_deadline) = render_at(&no_motion, 1_000);
    let (second, second_deadline) = render_at(&no_motion, 1_125);
    let (third, third_deadline) = render_at(&no_motion, 3_999);
    assert_eq!(first, second);
    assert_eq!(second, third);
    assert_eq!([first_deadline, second_deadline, third_deadline], [None; 3]);
}

#[test]
fn one_target_buffer_is_reused_for_one_thousand_fixed_frames() {
    let value = snapshot(Vec::new());
    let viewport = PixelSize::new(240, 120);
    let mut target = RgbBuffer::filled(viewport.width, viewport.height, Rgb::BLACK);
    let capacity = target.capacity();

    for _ in 0..1_000 {
        let frame = render_scene(&value, viewport, &mut target);
        assert_eq!(frame.next_frame_in, None);
        assert_eq!(target.capacity(), capacity);
    }
}

#[test]
fn renderer_deadlines_only_track_animation_inside_the_camera() {
    let blocked = agent("a-blocked", Presence::Blocked);
    let working = agent("z-working", Presence::Working);
    let value = snapshot(vec![blocked, working]);
    let viewport = PixelSize::new(40, 36);

    let mut start = value.clone();
    start.now = Timestamp::from_millis(1_000);
    let (start_frame, start_pixels) = render(&start, viewport);
    let mut before = value.clone();
    before.now = Timestamp::from_millis(1_166);
    let (_, before_pixels) = render(&before, viewport);
    let mut next = value.clone();
    next.now = Timestamp::from_millis(1_500);
    let (_, next_pixels) = render(&next, viewport);

    assert_eq!(start_frame.next_frame_in, Some(Duration::from_millis(500)));
    assert_eq!(start_pixels.pixels(), before_pixels.pixels());
    assert_ne!(before_pixels.pixels(), next_pixels.pixels());

    let (zero_frame, _) = render(
        &snapshot(vec![agent("working", Presence::Working)]),
        PixelSize::new(0, 0),
    );
    assert_eq!(zero_frame.next_frame_in, None);
}

#[test]
fn fresh_spoils_deadline_requires_a_visible_animated_pixel() {
    let mut completed = agent("completed", Presence::Done);
    completed.transition = Some(SceneTransition {
        summons: GuildSummons::SpoilsReturned,
        since: Timestamp::from_millis(1_000),
    });
    let mut fresh = snapshot(vec![completed]);
    fresh.now = Timestamp::from_millis(1_000);

    let (empty_effect_pixel, _) = render(&fresh, PixelSize::new(1, 1));
    assert_eq!(empty_effect_pixel.next_frame_in, None);

    let (visible_effect_pixels, _) = render(&fresh, PixelSize::new(20, 20));
    assert_eq!(
        visible_effect_pixels.next_frame_in,
        Some(Duration::from_millis(125))
    );
}

#[test]
fn actor_deadline_requires_a_visible_painted_pixel() {
    let blocked = snapshot(vec![agent("blocked", Presence::Blocked)]);

    let (transparent_actor_edge, _) = render(&blocked, PixelSize::new(9, 1));
    assert_eq!(transparent_actor_edge.next_frame_in, None);

    let (visible_actor, _) = render(&blocked, PixelSize::new(20, 20));
    assert_eq!(
        visible_actor.next_frame_in,
        Some(Duration::from_millis(1_000))
    );
}
