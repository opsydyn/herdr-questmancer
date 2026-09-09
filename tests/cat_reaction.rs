use questmancer::{
    app::{Model, View},
    command::CommandResult,
    domain::Timestamp,
    herdr::{
        protocol::{SessionSnapshot, SessionSnapshotResult, SuccessResponse},
        supervisor::ConnectionUpdate,
    },
    runtime_loop::{apply_command_result, apply_connection_update},
    scene::{
        SceneFrame,
        pixel::{PixelSize, Rgb, RgbBuffer},
        presentation::ScenePresentation,
        render_scene_for_world,
        snapshot::SceneSnapshot,
    },
};

fn snapshot(status: &str, revision: u64) -> SessionSnapshot {
    let mut value: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
    for field in ["agents", "panes"] {
        let record = &mut value["result"]["snapshot"][field][0];
        record["agent_status"] = status.into();
        record["revision"] = revision.into();
    }
    serde_json::from_value::<SuccessResponse<SessionSnapshotResult>>(value)
        .unwrap()
        .result
        .snapshot
}

fn connected(status: &str) -> Model {
    let mut model = Model::new(View::Guild);
    apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(snapshot(status, 7)),
        Timestamp::from_millis(1_000),
    );
    model.set_now(Timestamp::from_millis(1_000));
    model
}

fn apply_snapshot(model: &mut Model, snapshot: SessionSnapshot, at: Timestamp) {
    let effects = questmancer::runtime_loop::request_snapshot_refresh(model);
    let [questmancer::command::AgentCommand::RefreshSnapshot(request)] =
        effects.agent_commands.as_slice()
    else {
        panic!("expected a snapshot request");
    };
    apply_command_result(
        model,
        CommandResult::SnapshotLoaded {
            request: *request,
            snapshot: Box::new(snapshot),
        },
        at,
    );
}

fn refresh(model: &mut Model, status: &str, revision: u64, now: i64) {
    model.set_now(Timestamp::from_millis(now));
    apply_snapshot(
        model,
        snapshot(status, revision),
        Timestamp::from_millis(now),
    );
}

fn render(model: &Model) -> (RgbBuffer, SceneFrame) {
    let mut pixels = RgbBuffer::filled(160, 90, Rgb::BLACK);
    let frame = render_scene_for_world(
        &SceneSnapshot::from_model(model),
        &ScenePresentation::from_model(model),
        PixelSize::new(160, 90),
        &mut pixels,
    );
    (pixels, frame)
}

fn cat(pixels: &RgbBuffer) -> Vec<Rgb> {
    (19..24)
        .flat_map(|y| (88..98).map(move |x| pixels.get(x, y).unwrap()))
        .collect()
}

#[test]
fn the_last_resting_transition_gets_one_bounded_cat_reaction() {
    let mut model = connected("working");
    refresh(&mut model, "idle", 8, 2_000);
    let (initial, frame) = render(&model);
    let mut baseline = connected("idle");
    baseline.set_now(Timestamp::from_millis(2_000));
    let (asleep, static_frame) = render(&baseline);
    assert_ne!(
        cat(&initial),
        cat(&asleep),
        "the cat should notice the party becoming entirely resting"
    );
    assert_eq!(frame.actors, static_frame.actors);
    assert_eq!(frame.interactables, static_frame.interactables);
    assert_eq!(
        frame.next_frame_in,
        Some(std::time::Duration::from_millis(400))
    );
    model.set_now(Timestamp::from_millis(2_400));
    let (stretch, frame) = render(&model);
    assert_ne!(cat(&initial), cat(&stretch));
    assert_ne!(cat(&stretch), cat(&asleep));
    assert_eq!(
        frame.next_frame_in,
        Some(std::time::Duration::from_millis(400))
    );
    model.set_now(Timestamp::from_millis(2_800));
    let (settled, frame) = render(&model);
    assert_eq!(cat(&settled), cat(&asleep));
    assert_eq!(frame.next_frame_in, None);
    refresh(&mut model, "idle", 9, 4_000);
    let (repeated, frame) = render(&model);
    assert_eq!(cat(&repeated), cat(&asleep));
    assert_eq!(frame.next_frame_in, None);
}

#[test]
fn unknown_empty_changed_parties_and_connection_baselines_do_not_trigger() {
    for initial in ["idle", "unknown"] {
        let mut model = connected(initial);
        refresh(&mut model, "idle", 8, 2_000);
        assert_eq!(model.party_rest_since(), None, "{initial}");
    }
    let mut model = connected("working");
    apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(snapshot("idle", 8)),
        Timestamp::from_millis(2_000),
    );
    assert_eq!(model.party_rest_since(), None, "reconnect is a baseline");

    for empty in [false, true] {
        let mut model = connected("working");
        let mut changed = snapshot("idle", 8);
        if empty {
            changed.agents.clear();
            changed.panes.clear();
        } else {
            changed.agents[0].pane_id = "w1:p2".into();
            changed.panes[0].pane_id = "w1:p2".into();
        }
        apply_snapshot(&mut model, changed, Timestamp::from_millis(2_000));
        assert_eq!(model.party_rest_since(), None, "membership changed");
    }
}

#[test]
fn reaction_cancels_when_work_resumes_or_connection_is_lost() {
    for disconnected in [false, true] {
        let mut model = connected("working");
        refresh(&mut model, "idle", 8, 2_000);
        assert!(model.party_rest_since().is_some());
        if disconnected {
            apply_connection_update(
                &mut model,
                ConnectionUpdate::Disconnected("test".into()),
                Timestamp::from_millis(2_100),
            );
        } else {
            refresh(&mut model, "working", 9, 2_100);
        }
        assert_eq!(model.party_rest_since(), None);
    }
}

#[test]
fn reduced_still_delve_and_small_halls_do_not_schedule_cat_wakes() {
    use questmancer::app::Motion;
    for motion in [Motion::Reduced, Motion::None] {
        let mut model = connected("working");
        let mut preferences = *model.preferences();
        preferences.motion = motion;
        model.set_preferences(preferences);
        refresh(&mut model, "idle", 8, 2_000);
        assert_eq!(model.party_rest_since(), None);
        assert_eq!(render(&model).1.next_frame_in, None);
    }
    let mut model = connected("working");
    refresh(&mut model, "idle", 8, 2_000);
    for size in [
        PixelSize::new(100, 60),
        PixelSize::new(64, 40),
        PixelSize::new(30, 30),
        PixelSize::new(24, 26),
        PixelSize::new(16, 18),
    ] {
        let mut pixels = RgbBuffer::filled(0, 0, Rgb::BLACK);
        let frame = render_scene_for_world(
            &SceneSnapshot::from_model(&model),
            &ScenePresentation::from_model(&model),
            size,
            &mut pixels,
        );
        assert_eq!(frame.next_frame_in, None, "{size:?}");
    }
    model.switch_to(View::Delve);
    assert_eq!(render(&model).1.next_frame_in, None);
}

#[test]
fn stale_status_events_cannot_trigger_or_restart_the_reaction() {
    use questmancer::herdr::protocol::WireEvent;
    let mut model = connected("working");
    for (revision, now, expected) in [
        (6, 1_500, None),
        (8, 2_000, Some(2_000)),
        (8, 2_100, Some(2_000)),
    ] {
        apply_connection_update(
            &mut model,
            ConnectionUpdate::Event(WireEvent {
                event: "pane.agent_status_changed".into(),
                data: serde_json::json!({"pane_id":"w1:p1","workspace_id":"w1","agent_status":"idle","revision":revision,"terminal_id":"terminal-1", "agent_session":{"source":"codex","agent":"codex","kind":"id","value":"session-123"}}),
            }),
            Timestamp::from_millis(now),
        );
        assert_eq!(
            model.party_rest_since(),
            expected.map(Timestamp::from_millis)
        );
    }
}

#[test]
fn reaction_does_not_change_domain_projection_persistence_or_pixels_outside_the_cat() {
    use questmancer::{app::ConnectionState, persistence::PersistedStateV1};
    let mut model = connected("working");
    refresh(&mut model, "idle", 8, 2_000);
    let mut baseline = model.clone();
    baseline.set_connection_at(ConnectionState::Connected, Timestamp::from_millis(2_000));
    assert_eq!(
        SceneSnapshot::from_model(&model),
        SceneSnapshot::from_model(&baseline)
    );
    assert_eq!(
        PersistedStateV1::capture(&model),
        PersistedStateV1::capture(&baseline)
    );
    let (pixels, _) = render(&model);
    let (asleep, _) = render(&baseline);
    for y in 0..90 {
        for x in 0..160 {
            if !(88..98).contains(&x) || !(19..24).contains(&y) {
                assert_eq!(
                    pixels.get(x, y),
                    asleep.get(x, y),
                    "cat escaped reservation at {x},{y}"
                );
            }
        }
    }
}

#[test]
fn the_whole_party_must_rest_and_departure_cancels_a_running_reaction() {
    fn party(status: &str, revision: u64) -> SessionSnapshot {
        let mut snapshot = snapshot(status, revision);
        let mut second = snapshot.agents[0].clone();
        second.pane_id = "w1:p2".into();
        second.terminal_id = "terminal-2".into();
        second.agent_session = None;
        second.agent_status = questmancer::herdr::protocol::AgentStatus::Idle;
        snapshot.agents.push(second);
        let mut pane = snapshot.panes[0].clone();
        pane.pane_id = "w1:p2".into();
        pane.terminal_id = "terminal-2".into();
        pane.agent_status = questmancer::herdr::protocol::AgentStatus::Idle;
        snapshot.panes.push(pane);
        snapshot
    }
    let mut model = Model::new(View::Guild);
    apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(party("working", 7)),
        Timestamp::from_millis(1_000),
    );
    assert_eq!(model.domain().agents.len(), 2);
    apply_snapshot(
        &mut model,
        party("working", 8),
        Timestamp::from_millis(1_500),
    );
    assert_eq!(model.party_rest_since(), None);
    apply_snapshot(&mut model, party("idle", 9), Timestamp::from_millis(2_000));
    assert_eq!(
        model.party_rest_since(),
        Some(Timestamp::from_millis(2_000))
    );
    refresh(&mut model, "idle", 10, 2_100);
    assert_eq!(
        model.party_rest_since(),
        None,
        "a changed party cancels rather than replays"
    );
}

#[test]
fn authored_cat_frames_end_at_the_exact_deadlines_and_keep_native_bounds() {
    use questmancer::scene::assets::cat;
    use std::time::Duration;
    for (elapsed, remaining) in [(0, 400), (399, 1), (400, 400), (799, 1)] {
        let (frame, deadline) = cat::reaction(Duration::from_millis(elapsed)).unwrap();
        assert_eq!(deadline, Duration::from_millis(remaining));
        assert_eq!(frame.size(), PixelSize::new(10, 5));
        assert!(frame.pixels().iter().filter(|p| p.is_some()).count() >= 20);
    }
    assert!(cat::reaction(Duration::from_millis(800)).is_none());
    assert!(cat::reaction(Duration::from_secs(5)).is_none());
}
