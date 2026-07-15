use futures_util::FutureExt;
use herdr_webmaster::{
    app::{ConnectionState, DisplayPreferences, Model, Motion, RuntimeSettings, View},
    command::{CommandResult, DeskCommand},
    domain::{
        Agent, AgentKey, AgentPersona, Attention, AttentionReason, DomainState, PaneId, PersonaKey,
        Presence, Timestamp,
    },
    herdr::{
        environment::HerdrEnvironment,
        protocol::{SessionSnapshot, SessionSnapshotResult, SuccessResponse, WireEvent},
        supervisor::ConnectionUpdate,
    },
    runtime_loop::{
        RuntimeConnection, RuntimeEffects, RuntimeEvent, apply_command_result,
        apply_connection_update, bootstrap_model,
    },
    terminal::{AnimationScheduler, RuntimeClock},
    ui::theatre::{TheatrePose, frame_for},
    update::Command,
};
use serde_json::json;
use std::future::Future;
use tempfile::tempdir;
use tokio::{net::UnixListener, time::timeout};

fn snapshot() -> SessionSnapshot {
    let response: SuccessResponse<SessionSnapshotResult> =
        serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
    response.result.snapshot
}

fn animated_agent(key: &str, presence: Presence, since: i64) -> Agent {
    let mut agent = DomainState::from_snapshot(&snapshot(), Timestamp::from_millis(0))
        .agents
        .into_values()
        .next()
        .unwrap();
    agent.key = AgentKey::new(key);
    agent.presence = presence;
    agent.presence_since = Timestamp::from_millis(since);
    agent.attention = Attention::Clear;
    agent
}

fn animated_model(agents: impl IntoIterator<Item = Agent>, now: i64, motion: Motion) -> Model {
    let mut domain = DomainState::default();
    for agent in agents {
        domain.agents.insert(agent.key.clone(), agent);
    }
    let mut model = Model::new(View::Cafe);
    model.replace_domain(domain);
    model.set_now(Timestamp::from_millis(now));
    model.set_preferences(DisplayPreferences {
        motion,
        ..DisplayPreferences::default()
    });
    model
}

fn model_with_two_distinct_personas() -> Model {
    let mut domain = DomainState::from_snapshot(&snapshot(), Timestamp::from_millis(1_000));
    let mut second = domain.agents.values().next().unwrap().clone();
    second.key = AgentKey::new("agent-z");
    second.pane_id = PaneId::new("w1:p2");
    let persona_key = PersonaKey::new("persona-z");
    second.persona = AgentPersona {
        appearance: AgentPersona::appearance_for_key(&persona_key),
        key: persona_key,
        handle: "second_persona".to_owned(),
    };
    domain.agents.insert(second.key.clone(), second);
    let mut model = Model::new(View::Desk);
    model.replace_domain(domain);
    model
}

fn connected_model_with_presence(presence: Presence) -> Model {
    let mut model = Model::new(View::Desk);
    model.replace_domain(DomainState::from_snapshot(
        &snapshot(),
        Timestamp::from_millis(1_000),
    ));
    let agent = model.domain_mut().agents.values_mut().next().unwrap();
    agent.presence = presence;
    agent.attention = Attention::Clear;
    model
}

fn status_update_with_revision(status: &str, revision: u64) -> ConnectionUpdate {
    ConnectionUpdate::Event(WireEvent {
        event: "pane.agent_status_changed".into(),
        data: json!({
            "pane_id": "w1:p1",
            "workspace_id": "w1",
            "agent_status": status,
            "revision": revision,
        }),
    })
}

#[test]
fn blocked_transition_routes_history_and_state_to_persistence() {
    let mut model = connected_model_with_presence(Presence::Working);

    let effects = apply_connection_update(
        &mut model,
        status_update_with_revision("blocked", 8),
        Timestamp::from_millis(2_000),
    );

    assert_eq!(
        effects
            .persistence
            .iter()
            .filter(|effect| effect.is_guestbook_append())
            .count(),
        1
    );
    assert_eq!(
        effects
            .persistence
            .iter()
            .filter(|effect| **effect == Command::PersistState)
            .count(),
        1
    );
}

#[test]
fn explicit_duplicate_and_stale_status_updates_have_no_runtime_effects() {
    let mut model = connected_model_with_presence(Presence::Blocked);

    for revision in [7, 6] {
        let effects = apply_connection_update(
            &mut model,
            status_update_with_revision("blocked", revision),
            Timestamp::from_millis(2_000),
        );

        assert!(effects.desk.is_empty(), "revision {revision}");
        assert!(effects.persistence.is_empty(), "revision {revision}");
    }
}

#[test]
fn snapshot_result_preserves_persistence_effect_after_durable_overlay() {
    let mut model = connected_model_with_presence(Presence::Working);
    let restored_handle = model.selected_agent().unwrap().persona.handle.clone();

    let effects = apply_command_result(
        &mut model,
        CommandResult::SnapshotLoaded(Box::new(snapshot())),
        Timestamp::from_millis(2_000),
    );

    assert_eq!(effects.persistence, vec![Command::PersistState]);
    assert_eq!(
        model.selected_agent().unwrap().persona.handle,
        restored_handle
    );
}

#[test]
fn connection_bootstrap_updates_model_and_lazily_loads_selected_output() {
    let mut model = Model::new(View::Desk);
    model.set_settings(RuntimeSettings {
        output_preview_lines: 123,
        reviewr_action: "acme.diff.inspect".to_owned(),
        show_elapsed_time: true,
    });

    let effects = apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(snapshot()),
        Timestamp::from_millis(1_000),
    );

    assert_eq!(model.connection(), &ConnectionState::Connected);
    assert_eq!(model.domain().agents.len(), 1);
    assert!(effects.desk.iter().any(|command| matches!(
        command,
        DeskCommand::LoadOutput { pane_id, lines: 123 }
            if pane_id.as_str() == "w1:p1"
    )));
    assert!(effects.desk.contains(&DeskCommand::DiscoverReviewr {
        qualified_id: "acme.diff.inspect".to_owned(),
    }));
}

#[test]
fn selected_status_change_refreshes_only_that_output() {
    let mut model = Model::new(View::Desk);
    apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(snapshot()),
        Timestamp::from_millis(1_000),
    );

    let effects = apply_connection_update(
        &mut model,
        ConnectionUpdate::Event(WireEvent {
            event: "pane.agent_status_changed".into(),
            data: json!({"pane_id": "w1:p1", "workspace_id": "w1", "agent_status": "done"}),
        }),
        Timestamp::from_millis(2_000),
    );

    assert_eq!(effects.desk.len(), 1);
    assert!(matches!(
        &effects.desk[0],
        DeskCommand::LoadOutput { pane_id, .. } if pane_id.as_str() == "w1:p1"
    ));
}

#[test]
fn runtime_domain_update_keeps_the_newly_selected_distinct_persona_selected() {
    let mut model = model_with_two_distinct_personas();
    model.select_last_agent();
    let selected = model.selected_agent_key().unwrap().clone();

    apply_connection_update(
        &mut model,
        ConnectionUpdate::Event(WireEvent {
            event: "pane.agent_status_changed".into(),
            data: json!({
                "pane_id": "w1:p2",
                "workspace_id": "w1",
                "agent_status": "done"
            }),
        }),
        Timestamp::from_millis(2_000),
    );

    assert_eq!(model.selected_agent_key(), Some(&selected));
    assert_eq!(model.domain().agents[&selected].presence, Presence::Done);
}

#[test]
fn output_and_discovery_results_update_app_state() {
    let mut model = Model::new(View::Desk);

    apply_command_result(
        &mut model,
        CommandResult::OutputLoaded {
            pane_id: PaneId::new("w1:p1"),
            revision: 12,
            text: "published".into(),
            truncated: false,
        },
        Timestamp::from_millis(2_000),
    );
    apply_command_result(
        &mut model,
        CommandResult::ReviewrAvailable(true),
        Timestamp::from_millis(2_000),
    );

    assert_eq!(model.output_preview().unwrap().text, "published");
    assert!(model.reviewr_available());
}

#[test]
fn command_failure_is_visible_without_replacing_domain_state() {
    let mut model = Model::new(View::Desk);
    let before = model.domain().clone();

    apply_command_result(
        &mut model,
        CommandResult::Failed {
            operation: "load output",
            message: "pane vanished".into(),
        },
        Timestamp::from_millis(2_000),
    );

    assert_eq!(model.domain(), &before);
    assert_eq!(
        model.status_message(),
        Some("load output failed: pane vanished")
    );
}

#[test]
fn startup_without_plugin_environment_is_usefully_offline() {
    let mut restored = Model::new(View::Cafe);
    restored.set_preferences(DisplayPreferences {
        motion: Motion::None,
        ..DisplayPreferences::default()
    });

    let model = bootstrap_model(restored, None);

    assert_eq!(model.connection(), &ConnectionState::Offline);
    assert_eq!(model.view(), View::Cafe);
    assert_eq!(model.preferences().motion, Motion::None);
    assert_eq!(
        model.status_message(),
        Some("offline: launch from Herdr to connect to the live session")
    );
}

#[test]
fn startup_with_plugin_environment_begins_connecting() {
    let environment = HerdrEnvironment::new("/tmp/herdr.sock", "/usr/bin/herdr");
    let restored = Model::new(View::Cafe);

    let model = bootstrap_model(restored, Some(&environment));

    assert_eq!(model.connection(), &ConnectionState::Connecting);
    assert_eq!(model.view(), View::Cafe);
    assert_eq!(model.status_message(), Some("connecting to Herdr"));
}

#[test]
fn disconnect_preserves_the_last_connected_snapshot() {
    let mut model = Model::new(View::Desk);
    apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(snapshot()),
        Timestamp::from_millis(1_000),
    );
    let connected_domain = model.domain().clone();

    let effects = apply_connection_update(
        &mut model,
        ConnectionUpdate::Disconnected("socket closed".into()),
        Timestamp::from_millis(2_000),
    );

    assert_eq!(model.domain(), &connected_domain);
    assert_eq!(effects, RuntimeEffects::default());
}

#[tokio::test]
async fn runtime_shutdown_cancels_supervisor_and_command_tasks() {
    let directory = tempdir().unwrap();
    let socket_path = directory.path().join("herdr.sock");
    let _listener = UnixListener::bind(&socket_path).unwrap();
    let environment = HerdrEnvironment::new(&socket_path, "/usr/bin/herdr");
    let mut connection = RuntimeConnection::start(&environment);
    connection.schedule([DeskCommand::RefreshSnapshot]);
    tokio::task::yield_now().await;

    timeout(std::time::Duration::from_secs(1), connection.shutdown())
        .await
        .expect("runtime tasks did not stop before terminal restoration")
        .unwrap();
}

#[tokio::test]
async fn runtime_connection_exposes_owned_work_as_typed_events() {
    let directory = tempdir().unwrap();
    let environment =
        HerdrEnvironment::new(directory.path().join("missing.sock"), "/usr/bin/herdr");
    let mut connection = RuntimeConnection::start(&environment);
    connection.schedule([DeskCommand::RefreshSnapshot]);

    timeout(std::time::Duration::from_secs(1), async {
        loop {
            if matches!(
                connection.next_event().await,
                RuntimeEvent::Command(CommandResult::Failed {
                    operation: "refresh snapshot",
                    ..
                })
            ) {
                break;
            }
        }
    })
    .await
    .expect("command completion was not exposed through the runtime event boundary");

    connection.shutdown().await.unwrap();
}

#[test]
fn terminal_runtime_is_async() {
    fn assert_future(_: impl Future<Output = anyhow::Result<()>>) {}

    assert_future(herdr_webmaster::terminal::run(None));
}

#[tokio::test(start_paused = true)]
async fn runtime_clock_advances_from_one_epoch_sample_using_tokio_time() {
    let clock = RuntimeClock::new(Timestamp::from_millis(50_000));

    assert_eq!(clock.now(), Timestamp::from_millis(50_000));
    tokio::time::advance(std::time::Duration::from_millis(1_234)).await;
    assert_eq!(clock.now(), Timestamp::from_millis(51_234));
    tokio::time::advance(std::time::Duration::from_millis(66)).await;
    assert_eq!(clock.now(), Timestamp::from_millis(51_300));
}

#[tokio::test(start_paused = true)]
async fn stale_999ms_model_resets_to_the_absolute_done_boundary_after_slow_render() {
    let clock = RuntimeClock::new(Timestamp::from_millis(0));
    let mut done = animated_agent("done", Presence::Done, 0);
    done.attention = Attention::unseen(AttentionReason::WorkCompleted, Timestamp::from_millis(0));
    let mut model = animated_model([done], 0, Motion::Full);
    let mut scheduler = AnimationScheduler::new();

    tokio::time::advance(std::time::Duration::from_millis(999)).await;
    model.set_now(clock.now());
    assert_eq!(model.now(), Timestamp::from_millis(999));

    tokio::time::advance(std::time::Duration::from_millis(20)).await;
    scheduler.reset_for(&model, &clock);
    assert!(scheduler.wait().now_or_never().is_some());

    model.set_now(clock.now());
    assert_eq!(model.now(), Timestamp::from_millis(1_019));
    let done = model.domain().agents.values().next().unwrap();
    let frame = frame_for(done, model.now(), model.preferences());
    assert_eq!(frame.pose, TheatrePose::DoneUnseen);
    assert_eq!(frame.animation_frame, 0);

    scheduler.reset_for(&model, &clock);
    tokio::time::advance(std::time::Duration::from_secs(24 * 60 * 60)).await;
    assert!(scheduler.wait().now_or_never().is_none());
}

#[tokio::test(start_paused = true)]
async fn prolonged_six_fps_animation_tracks_phase_without_drift_or_skips() {
    let clock = RuntimeClock::new(Timestamp::from_millis(0));
    let working = animated_agent("working", Presence::Working, 0);
    let mut model = animated_model([working], 0, Motion::Full);
    let mut scheduler = AnimationScheduler::new();
    let boundaries = [167, 334, 500, 667, 834, 1_000, 1_167, 1_334, 1_500];
    let expected_frames = [1, 2, 3, 0, 1, 2, 3, 0, 1];
    let mut previous = 0;

    for (boundary, expected_frame) in boundaries.into_iter().zip(expected_frames) {
        scheduler.reset_for(&model, &clock);
        tokio::time::advance(std::time::Duration::from_millis(
            u64::try_from(boundary - previous).unwrap(),
        ))
        .await;
        assert!(scheduler.wait().now_or_never().is_some());
        model.set_now(clock.now());
        assert_eq!(model.now(), Timestamp::from_millis(boundary));
        let agent = model.domain().agents.values().next().unwrap();
        assert_eq!(
            frame_for(agent, model.now(), model.preferences()).animation_frame,
            expected_frame
        );
        previous = boundary;
    }
}

#[tokio::test(start_paused = true)]
async fn mixed_six_and_eight_fps_agents_choose_each_earliest_boundary() {
    let clock = RuntimeClock::new(Timestamp::from_millis(0));
    let working = animated_agent("working", Presence::Working, 0);
    let mut done = animated_agent("done", Presence::Done, 0);
    done.attention = Attention::unseen(AttentionReason::WorkCompleted, Timestamp::from_millis(0));
    let mut model = animated_model([working, done], 0, Motion::Full);
    let mut scheduler = AnimationScheduler::new();
    let boundaries = [125, 167, 250, 334, 375, 500, 625, 667, 750, 834, 875, 1_000];
    let mut previous = 0;

    for boundary in boundaries {
        scheduler.reset_for(&model, &clock);
        tokio::time::advance(std::time::Duration::from_millis(
            u64::try_from(boundary - previous).unwrap(),
        ))
        .await;
        assert!(
            scheduler.wait().now_or_never().is_some(),
            "missed mixed-agent boundary at {boundary}ms"
        );
        model.set_now(clock.now());
        assert_eq!(model.now(), Timestamp::from_millis(boundary));
        previous = boundary;
    }

    scheduler.reset_for(&model, &clock);
    tokio::time::advance(std::time::Duration::from_millis(166)).await;
    assert!(scheduler.wait().now_or_never().is_none());
    tokio::time::advance(std::time::Duration::from_millis(1)).await;
    assert!(scheduler.wait().now_or_never().is_some());
}

#[tokio::test(start_paused = true)]
async fn event_driven_animation_scheduler_never_wakes_on_time_alone() {
    let clock = RuntimeClock::new(Timestamp::from_millis(0));
    let working = animated_agent("working", Presence::Working, 0);
    let model = animated_model([working], 0, Motion::None);
    let mut scheduler = AnimationScheduler::new();
    scheduler.reset_for(&model, &clock);

    tokio::time::advance(std::time::Duration::from_secs(86_400)).await;
    assert!(scheduler.wait().now_or_never().is_none());
}
