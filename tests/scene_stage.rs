use questmancer::{
    app::Motion,
    domain::{
        AdventurerPersona, AgentKey, GuildSummons, PersonaKey, Presence, Timestamp, WorkspaceId,
    },
    scene::{
        pixel::PixelSize,
        snapshot::{SceneAgent, SceneConnection, SceneSnapshot, SceneTransition},
        stage::{
            ActorPlacement, COMPLETION_THEATRE_MS, CameraAnchor, SceneCadence, SceneCamera,
            SceneEffect, ScenePlan, ScenePose, TruthfulStation, WorldScene,
        },
    },
};

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
            anchor: CameraAnchor::DelveParty(WorkspaceId::new("workspace-working")),
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
