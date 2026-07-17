use futures_util::FutureExt;
use questmancer::{
    app::{
        ConnectionState, DisplayPreferences, Model, Motion, Notice, Region, RuntimeSettings, View,
    },
    command::{AgentCommand, CommandResult},
    domain::{
        AdventurerPersona, Agent, AgentKey, DomainState, GuildAttention, GuildSummons, PaneId,
        PersonaKey, Presence, Timestamp,
    },
    herdr::{
        environment::HerdrEnvironment,
        protocol::{SessionSnapshot, SessionSnapshotResult, SuccessResponse, WireEvent},
        supervisor::ConnectionUpdate,
    },
    interaction::reduce_action,
    runtime_loop::{
        RuntimeConnection, RuntimeEffects, RuntimeEvent, apply_command_result,
        apply_connection_update, bootstrap_model,
    },
    terminal::{AnimationScheduler, RuntimeClock},
    ui::input::Action,
    ui::theatre::{TheatrePose, frame_for},
    update::Command,
};
use ratatui::layout::Rect;
use ratatui::{Terminal, backend::TestBackend};
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
    agent.attention = GuildAttention::Clear;
    agent
}

fn animated_model(agents: impl IntoIterator<Item = Agent>, now: i64, motion: Motion) -> Model {
    let mut domain = DomainState::default();
    for agent in agents {
        domain.agents.insert(agent.key.clone(), agent);
    }
    let mut model = Model::new(View::Delve);
    model.replace_domain(domain);
    model.set_now(Timestamp::from_millis(now));
    model.set_preferences(DisplayPreferences {
        motion,
        ..DisplayPreferences::default()
    });
    model
}

fn render_area() -> Rect {
    Rect::new(0, 0, 120, 24)
}

fn render_projection(model: &Model, area: Rect) -> questmancer::ui::RenderProjection {
    let backend = TestBackend::new(area.width, area.height);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut projection = questmancer::ui::RenderProjection::default();
    terminal
        .draw(|frame| projection = questmancer::ui::render_with_projection(frame, model))
        .unwrap();
    projection
}

fn model_with_two_distinct_personas() -> Model {
    let mut domain = DomainState::from_snapshot(&snapshot(), Timestamp::from_millis(1_000));
    let mut second = domain.agents.values().next().unwrap().clone();
    second.key = AgentKey::new("agent-z");
    second.pane_id = PaneId::new("w1:p2");
    let persona_key = PersonaKey::new("persona-z");
    second.persona = AdventurerPersona::for_key(persona_key);
    "second persona".clone_into(&mut second.persona.name);
    domain.agents.insert(second.key.clone(), second);
    let mut model = Model::new(View::Guild);
    model.replace_domain(domain);
    model
}

fn connected_model_with_presence(presence: Presence) -> Model {
    let mut model = Model::new(View::Guild);
    model.replace_domain(DomainState::from_snapshot(
        &snapshot(),
        Timestamp::from_millis(1_000),
    ));
    let agent = model.domain_mut().agents.values_mut().next().unwrap();
    agent.presence = presence;
    agent.attention = GuildAttention::Clear;
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
            .filter(|effect| effect.is_chronicle_append())
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

        assert!(effects.agent_commands.is_empty(), "revision {revision}");
        assert!(effects.persistence.is_empty(), "revision {revision}");
    }
}

#[test]
fn snapshot_result_preserves_persistence_effect_after_durable_overlay() {
    let mut model = connected_model_with_presence(Presence::Working);
    let restored_name = model.selected_agent().unwrap().persona.name.clone();

    let effects = apply_command_result(
        &mut model,
        CommandResult::SnapshotLoaded(Box::new(snapshot())),
        Timestamp::from_millis(2_000),
    );

    assert_eq!(effects.persistence, vec![Command::PersistState]);
    assert_eq!(model.selected_agent().unwrap().persona.name, restored_name);
}

#[test]
fn snapshot_result_excludes_the_managed_webmaster_pane() {
    let mut model = Model::new(View::Guild);
    let managed = PaneId::new("w2:p3");
    model.set_managed_pane_id(Some(managed.clone()));
    let mut snapshot = snapshot();
    let mut managed_agent = snapshot.agents[0].clone();
    managed_agent.pane_id = managed.as_str().to_owned();
    managed_agent.workspace_id = "w2".to_owned();
    snapshot.agents.push(managed_agent);

    apply_command_result(
        &mut model,
        CommandResult::SnapshotLoaded(Box::new(snapshot)),
        Timestamp::from_millis(2_000),
    );

    assert!(model.domain().agent_key_for_pane(&managed).is_none());
}

#[test]
fn connection_bootstrap_updates_model_and_lazily_loads_selected_output() {
    let mut model = Model::new(View::Guild);
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
    assert!(effects.agent_commands.iter().any(|command| matches!(
        command,
        AgentCommand::LoadOutput { pane_id, lines: 123 }
            if pane_id.as_str() == "w1:p1"
    )));
    assert!(
        effects
            .agent_commands
            .contains(&AgentCommand::DiscoverReviewr {
                qualified_id: "acme.diff.inspect".to_owned(),
            })
    );
}

#[test]
fn connection_snapshot_excludes_the_managed_webmaster_pane() {
    let mut model = Model::new(View::Guild);
    let managed = PaneId::new("w2:p3");
    model.set_managed_pane_id(Some(managed.clone()));
    let mut snapshot = snapshot();
    let mut managed_agent = snapshot.agents[0].clone();
    managed_agent.pane_id = managed.as_str().to_owned();
    managed_agent.workspace_id = "w2".to_owned();
    snapshot.agents.push(managed_agent);

    apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(snapshot),
        Timestamp::from_millis(1_000),
    );

    assert!(model.domain().agent_key_for_pane(&managed).is_none());
}

#[test]
fn selected_status_change_refreshes_only_that_output() {
    let mut model = Model::new(View::Guild);
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

    assert_eq!(effects.agent_commands.len(), 1);
    assert!(matches!(
        &effects.agent_commands[0],
        AgentCommand::LoadOutput { pane_id, .. } if pane_id.as_str() == "w1:p1"
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
    let mut model = Model::new(View::Guild);

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
fn available_reviewr_discovery_clears_the_prior_typed_integration_notice() {
    let mut model = Model::new(View::Guild);
    model.set_reviewr_availability_diagnostic(
        "The spoils cannot be inspected here: Reviewr is unavailable.".to_owned(),
    );

    apply_command_result(
        &mut model,
        CommandResult::ReviewrAvailable(true),
        Timestamp::from_millis(2_000),
    );

    assert!(model.reviewr_available());
    assert_eq!(model.notice(), None);
}

#[test]
fn available_reviewr_discovery_preserves_a_newer_adapter_diagnostic() {
    let mut model = Model::new(View::Guild);
    model.replace_domain(DomainState::from_snapshot(
        &snapshot(),
        Timestamp::from_millis(1_000),
    ));

    let _ = reduce_action(&mut model, Action::InspectSpoils);
    model.set_integration_diagnostic(
        "Herdr adapter could not decode the refreshed sidebar row.".to_owned(),
    );

    apply_command_result(
        &mut model,
        CommandResult::ReviewrAvailable(true),
        Timestamp::from_millis(2_000),
    );

    assert_eq!(
        model.notice(),
        Some(&Notice::IntegrationDiagnostic(
            "Herdr adapter could not decode the refreshed sidebar row.".to_owned(),
        ))
    );
}

#[test]
fn operational_results_use_approved_guild_copy() {
    let mut model = Model::new(View::Guild);

    apply_command_result(
        &mut model,
        CommandResult::CounselSent(PaneId::new("w1:p1")),
        Timestamp::from_millis(2_000),
    );
    assert_eq!(model.status_message(), Some("Counsel issued."));

    apply_command_result(
        &mut model,
        CommandResult::SpoilsOpened,
        Timestamp::from_millis(2_000),
    );
    assert_eq!(model.status_message(), Some("Spoils inspected."));
}

#[test]
fn output_failure_is_scoped_to_the_selected_scrying_preview() {
    let mut model = Model::new(View::Guild);
    let before = model.domain().clone();

    apply_command_result(
        &mut model,
        CommandResult::OutputFailed {
            pane_id: PaneId::new("w1:p1"),
            message: "pane vanished".into(),
        },
        Timestamp::from_millis(2_000),
    );

    assert_eq!(model.domain(), &before);
    assert_eq!(model.status_message(), None);
    assert_eq!(
        model.output_preview().unwrap().error.as_deref(),
        Some("load output failed: pane vanished")
    );
}

#[test]
fn startup_without_plugin_environment_is_usefully_offline() {
    let mut restored = Model::new(View::Delve);
    restored.set_preferences(DisplayPreferences {
        motion: Motion::None,
        ..DisplayPreferences::default()
    });

    let model = bootstrap_model(restored, None);

    assert_eq!(model.connection(), &ConnectionState::Offline);
    assert_eq!(model.view(), View::Delve);
    assert_eq!(model.preferences().motion, Motion::None);
    assert_eq!(
        model.status_message(),
        Some("offline: launch from Herdr to connect to the live session")
    );
}

#[test]
fn startup_with_plugin_environment_begins_connecting() {
    let environment = HerdrEnvironment::new("/tmp/herdr.sock", "/usr/bin/herdr");
    let restored = Model::new(View::Delve);

    let model = bootstrap_model(restored, Some(&environment));

    assert_eq!(model.connection(), &ConnectionState::Connecting);
    assert_eq!(model.view(), View::Delve);
    assert_eq!(model.status_message(), Some("connecting to Herdr"));
}

#[test]
fn connected_clears_only_connection_notice() {
    let environment = HerdrEnvironment::new("/tmp/herdr.sock", "/usr/bin/herdr");
    let mut model = bootstrap_model(Model::new(View::Guild), Some(&environment));

    assert_eq!(
        model.notice(),
        Some(&Notice::ConnectionDiagnostic(
            "connecting to Herdr".to_owned()
        ))
    );

    apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(snapshot()),
        Timestamp::from_millis(1_000),
    );

    assert_eq!(model.connection(), &ConnectionState::Connected);
    assert_eq!(model.notice(), None);

    let mut action = Model::new(View::Guild);
    action.set_action_feedback("counsel issued".to_owned());
    apply_connection_update(
        &mut action,
        ConnectionUpdate::Connected(snapshot()),
        Timestamp::from_millis(1_000),
    );
    assert_eq!(
        action.notice(),
        Some(&Notice::ActionFeedback("counsel issued".to_owned()))
    );

    let mut persistence = Model::new(View::Guild);
    persistence.set_persistence_diagnostic("state file is unreadable".to_owned());
    apply_connection_update(
        &mut persistence,
        ConnectionUpdate::Connected(snapshot()),
        Timestamp::from_millis(1_000),
    );
    assert_eq!(
        persistence.notice(),
        Some(&Notice::PersistenceDiagnostic(
            "state file is unreadable".to_owned()
        ))
    );

    let mut integration = Model::new(View::Guild);
    integration.set_integration_diagnostic("Reviewr is unavailable".to_owned());
    apply_connection_update(
        &mut integration,
        ConnectionUpdate::Connected(snapshot()),
        Timestamp::from_millis(1_000),
    );
    assert_eq!(
        integration.notice(),
        Some(&Notice::IntegrationDiagnostic(
            "Reviewr is unavailable".to_owned()
        ))
    );
}

#[test]
fn disconnect_preserves_the_last_connected_snapshot() {
    let mut model = Model::new(View::Guild);
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
    connection.schedule([AgentCommand::RefreshSnapshot]);
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
    connection.schedule([AgentCommand::RefreshSnapshot]);

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

    assert_future(questmancer::terminal::run(None));
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
    done.attention =
        GuildAttention::unread(GuildSummons::SpoilsReturned, Timestamp::from_millis(0));
    let mut model = animated_model([done], 0, Motion::Full);
    let mut scheduler = AnimationScheduler::new();

    tokio::time::advance(std::time::Duration::from_millis(999)).await;
    model.set_now(clock.now());
    assert_eq!(model.now(), Timestamp::from_millis(999));

    tokio::time::advance(std::time::Duration::from_millis(20)).await;
    scheduler.reset_for(
        &model,
        render_area(),
        render_projection(&model, render_area()),
        &clock,
    );
    assert!(scheduler.wait().now_or_never().is_some());

    model.set_now(clock.now());
    assert_eq!(model.now(), Timestamp::from_millis(1_019));
    let done = model.domain().agents.values().next().unwrap();
    let frame = frame_for(done, model.now(), model.preferences());
    assert_eq!(frame.pose, TheatrePose::SpoilsUnopened);
    assert_eq!(frame.animation_frame, 0);

    scheduler.reset_for(
        &model,
        render_area(),
        render_projection(&model, render_area()),
        &clock,
    );
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
        scheduler.reset_for(
            &model,
            render_area(),
            render_projection(&model, render_area()),
            &clock,
        );
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
    done.attention =
        GuildAttention::unread(GuildSummons::SpoilsReturned, Timestamp::from_millis(0));
    let mut model = animated_model([working, done], 0, Motion::Full);
    let mut scheduler = AnimationScheduler::new();
    let boundaries = [125, 167, 250, 334, 375, 500, 625, 667, 750, 834, 875, 1_000];
    let mut previous = 0;

    for boundary in boundaries {
        scheduler.reset_for(
            &model,
            render_area(),
            render_projection(&model, render_area()),
            &clock,
        );
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

    scheduler.reset_for(
        &model,
        render_area(),
        render_projection(&model, render_area()),
        &clock,
    );
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
    scheduler.reset_for(
        &model,
        render_area(),
        render_projection(&model, render_area()),
        &clock,
    );

    tokio::time::advance(std::time::Duration::from_secs(86_400)).await;
    assert!(scheduler.wait().now_or_never().is_none());
}

#[tokio::test(start_paused = true)]
async fn guild_outbreak_wakes_at_four_fps_then_returns_to_event_driven_rendering() {
    let clock = RuntimeClock::new(Timestamp::from_millis(0));
    let mut model = Model::new(View::Guild);
    model.goblins_mut().release(Timestamp::from_millis(0));
    let mut scheduler = AnimationScheduler::new();

    scheduler.reset_for(
        &model,
        render_area(),
        render_projection(&model, render_area()),
        &clock,
    );
    tokio::time::advance(std::time::Duration::from_millis(249)).await;
    assert!(scheduler.wait().now_or_never().is_none());
    tokio::time::advance(std::time::Duration::from_millis(1)).await;
    assert!(scheduler.wait().now_or_never().is_some());

    tokio::time::advance(std::time::Duration::from_millis(2_749)).await;
    model.set_now(clock.now());
    assert_eq!(model.now(), Timestamp::from_millis(2_999));
    scheduler.reset_for(
        &model,
        render_area(),
        render_projection(&model, render_area()),
        &clock,
    );
    assert!(scheduler.wait().now_or_never().is_none());
    tokio::time::advance(std::time::Duration::from_millis(1)).await;
    assert!(scheduler.wait().now_or_never().is_some());

    model.set_now(clock.now());
    assert_eq!(model.now(), Timestamp::from_millis(3_000));
    scheduler.reset_for(
        &model,
        render_area(),
        render_projection(&model, render_area()),
        &clock,
    );
    tokio::time::advance(std::time::Duration::from_secs(86_400)).await;
    assert!(scheduler.wait().now_or_never().is_none());
}

#[tokio::test(start_paused = true)]
async fn static_motion_schedules_only_rendered_outbreak_terminal_boundaries() {
    for (motion, visible) in [(Motion::Reduced, true), (Motion::None, false)] {
        let clock = RuntimeClock::new(Timestamp::from_millis(0));
        let mut model = Model::new(View::Guild);
        model.set_preferences(DisplayPreferences {
            motion,
            ..DisplayPreferences::default()
        });
        model.set_settings(RuntimeSettings {
            show_elapsed_time: false,
            ..RuntimeSettings::default()
        });
        model.goblins_mut().release(Timestamp::from_millis(0));
        let mut scheduler = AnimationScheduler::new();

        scheduler.reset_for(
            &model,
            render_area(),
            render_projection(&model, render_area()),
            &clock,
        );
        tokio::time::advance(std::time::Duration::from_millis(2_999)).await;
        assert!(
            scheduler.wait().now_or_never().is_none(),
            "motion {motion:?}"
        );
        tokio::time::advance(std::time::Duration::from_millis(1)).await;
        assert_eq!(
            scheduler.wait().now_or_never().is_some(),
            visible,
            "motion {motion:?}"
        );
    }
}

#[tokio::test(start_paused = true)]
async fn guild_outbreak_scheduling_requires_an_exact_rendered_effect() {
    let invisible = [
        (Rect::new(0, 0, 3, 2), Motion::Full, Region::QuestBoard),
        (Rect::new(0, 0, 100, 24), Motion::None, Region::QuestBoard),
        (Rect::new(0, 0, 100, 3), Motion::Full, Region::QuestBoard),
        (Rect::new(0, 0, 79, 24), Motion::None, Region::Party),
    ];
    for (area, motion, region) in invisible {
        let clock = RuntimeClock::new(Timestamp::from_millis(0));
        let mut model = connected_model_with_presence(Presence::Working);
        model.set_preferences(DisplayPreferences {
            motion,
            ..DisplayPreferences::default()
        });
        model.set_settings(RuntimeSettings {
            show_elapsed_time: false,
            ..RuntimeSettings::default()
        });
        model.set_region(region);
        model.goblins_mut().release(Timestamp::from_millis(0));
        let projection = render_projection(&model, area);
        assert!(
            !projection.guild_goblin_effect_visible(),
            "{area:?} {motion:?} {region:?}"
        );

        let mut scheduler = AnimationScheduler::new();
        scheduler.reset_for(&model, area, projection, &clock);
        tokio::time::advance(std::time::Duration::from_secs(86_400)).await;
        assert!(
            scheduler.wait().now_or_never().is_none(),
            "{area:?} {motion:?} {region:?}"
        );
    }
}

#[tokio::test(start_paused = true)]
async fn visible_guild_sprite_and_marginalia_arm_their_exact_boundaries() {
    let area = Rect::new(0, 0, 130, 32);

    let sprite_clock = RuntimeClock::new(Timestamp::from_millis(0));
    let mut sprite_model = Model::new(View::Guild);
    sprite_model
        .goblins_mut()
        .release(Timestamp::from_millis(0));
    let sprite_projection = render_projection(&sprite_model, area);
    assert!(sprite_projection.guild_goblin_effect_visible());
    let mut sprite_scheduler = AnimationScheduler::new();
    sprite_scheduler.reset_for(&sprite_model, area, sprite_projection, &sprite_clock);
    tokio::time::advance(std::time::Duration::from_millis(250)).await;
    assert!(sprite_scheduler.wait().now_or_never().is_some());

    let notice_clock = RuntimeClock::new(Timestamp::from_millis(0));
    let mut notice_model = connected_model_with_presence(Presence::Working);
    notice_model.set_preferences(DisplayPreferences {
        motion: Motion::None,
        ..DisplayPreferences::default()
    });
    notice_model.set_settings(RuntimeSettings {
        show_elapsed_time: false,
        ..RuntimeSettings::default()
    });
    notice_model
        .goblins_mut()
        .release(Timestamp::from_millis(0));
    let notice_projection = render_projection(&notice_model, area);
    assert!(notice_projection.guild_goblin_effect_visible());
    let mut notice_scheduler = AnimationScheduler::new();
    notice_scheduler.reset_for(&notice_model, area, notice_projection, &notice_clock);
    tokio::time::advance(std::time::Duration::from_millis(2_999)).await;
    assert!(notice_scheduler.wait().now_or_never().is_none());
    tokio::time::advance(std::time::Duration::from_millis(1)).await;
    assert!(notice_scheduler.wait().now_or_never().is_some());
}

#[tokio::test(start_paused = true)]
async fn help_overlay_removes_hidden_goblin_effects_from_the_scheduler_projection() {
    let area = Rect::new(0, 0, 40, 10);
    let clock = RuntimeClock::new(Timestamp::from_millis(0));
    let mut model = connected_model_with_presence(Presence::Working);
    model.set_region(Region::Chronicle);
    model.set_preferences(DisplayPreferences {
        motion: Motion::None,
        ..DisplayPreferences::default()
    });
    model.set_settings(RuntimeSettings {
        show_elapsed_time: false,
        ..RuntimeSettings::default()
    });
    model.goblins_mut().release(Timestamp::from_millis(0));

    let visible = render_projection(&model, area);
    assert!(visible.guild_goblin_effect_visible());

    model.toggle_help();
    let covered = render_projection(&model, area);
    assert!(!covered.guild_goblin_effect_visible());

    let mut scheduler = AnimationScheduler::new();
    scheduler.reset_for(&model, area, covered, &clock);
    tokio::time::advance(std::time::Duration::from_secs(86_400)).await;
    assert!(scheduler.wait().now_or_never().is_none());
}
