use questmancer::{
    app::{
        ConnectionState, CounselPhase, DisplayPreferences, Model, Motion, Notice, OutputRequest,
        RuntimeSettings, View,
    },
    command::{AgentCommand, CommandResult},
    config::OutputPreviewLines,
    domain::{
        AdventurerPersona, AgentKey, DomainState, GuildAttention, PaneId, PersonaKey, Presence,
        Timestamp,
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
    terminal::RuntimeClock,
    ui::input::Action,
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

fn snapshot_request(commands: &[AgentCommand]) -> questmancer::snapshot_refresh::SnapshotRequest {
    commands
        .iter()
        .find_map(|command| match command {
            AgentCommand::RefreshSnapshot(request) => Some(*request),
            _ => None,
        })
        .expect("a fresh snapshot request should be scheduled")
}

fn apply_snapshot(model: &mut Model, snapshot: SessionSnapshot, at: Timestamp) -> RuntimeEffects {
    let request = snapshot_request(
        &questmancer::runtime_loop::request_snapshot_refresh(model).agent_commands,
    );
    apply_command_result(
        model,
        CommandResult::SnapshotLoaded {
            request,
            snapshot: Box::new(snapshot),
        },
        at,
    )
}

fn output_request(commands: &[AgentCommand]) -> OutputRequest {
    commands
        .iter()
        .find_map(|command| match command {
            AgentCommand::LoadOutput { request, .. } => Some(*request),
            _ => None,
        })
        .expect("selected output should be requested")
}

fn connect_for_output() -> (Model, OutputRequest) {
    let mut model = Model::new(View::Guild);
    let effects = apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(snapshot()),
        Timestamp::from_millis(1_000),
    );
    (model, output_request(&effects.agent_commands))
}

fn refresh_output(model: &mut Model) -> OutputRequest {
    output_request(&reduce_action(model, Action::Refresh).commands)
}

fn output_loaded(model: &mut Model, request: OutputRequest, revision: u64, text: &str) {
    apply_command_result(
        model,
        CommandResult::OutputLoaded {
            pane_id: PaneId::new("w1:p1"),
            request,
            revision,
            text: text.to_owned(),
            truncated: false,
        },
        Timestamp::from_millis(2_000),
    );
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
    model
        .domain_mut()
        .capture
        .start(questmancer::domain::CaptureRunId::new("runtime-test"));
    apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(snapshot()),
        Timestamp::from_millis(1_000),
    );
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
            "terminal_id":"terminal-1", "agent_session":{"source":"codex","agent":"codex","kind":"id","value":"session-123"},
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

    let effects = apply_snapshot(
        &mut model,
        changed_snapshot(questmancer::herdr::protocol::AgentStatus::Blocked, 8),
        Timestamp::from_millis(2_000),
    );

    assert_eq!(effects.persistence.last(), Some(&Command::PersistState));
    assert_eq!(
        effects
            .persistence
            .iter()
            .filter(|command| command.is_chronicle_append())
            .count(),
        1
    );
    assert_eq!(model.selected_agent().unwrap().persona.name, restored_name);
}

#[test]
fn marginalia_failure_is_an_integration_diagnostic_not_an_action_error() {
    let mut model = Model::new(View::Guild);

    apply_command_result(
        &mut model,
        CommandResult::MarginaliaFailed {
            message: "socket closed".to_owned(),
        },
        Timestamp::from_millis(2_000),
    );

    assert_eq!(
        model.integration_diagnostic(),
        Some("sidebar marginalia failed: socket closed")
    );
    assert_eq!(model.action_feedback(), None);
}

#[test]
fn snapshot_result_excludes_the_managed_webmaster_pane() {
    let mut model = Model::new(View::Guild);
    model.set_connection(ConnectionState::Connected);
    let managed = PaneId::new("w2:p3");
    model.set_managed_pane_id(Some(managed.clone()));
    let mut snapshot = snapshot();
    let mut managed_agent = snapshot.agents[0].clone();
    managed_agent.pane_id = managed.as_str().to_owned();
    managed_agent.workspace_id = "w2".to_owned();
    snapshot.agents.push(managed_agent);

    apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(snapshot.clone()),
        Timestamp::from_millis(1_000),
    );
    apply_snapshot(&mut model, snapshot, Timestamp::from_millis(2_000));

    assert!(model.domain().agent_key_for_pane(&managed).is_none());
}

#[test]
fn connection_bootstrap_updates_model_and_lazily_loads_selected_output() {
    let mut model = Model::new(View::Guild);
    model.set_settings(RuntimeSettings {
        sidebar_urgency_order: false,
        output_preview_lines: OutputPreviewLines::new(123).unwrap(),
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
        AgentCommand::LoadOutput { pane_id, lines: 123, .. }
            if pane_id.as_str() == "w1:p1"
    )));
    assert!(
        effects
            .agent_commands
            .contains(&AgentCommand::DiscoverReviewr {
                qualified_id: "acme.diff.inspect".to_owned(),
            })
    );
    assert!(effects.agent_commands.iter().any(|command| matches!(
        command,
        AgentCommand::PublishMarginalia(projection)
            if projection.agents.len() == 1 && projection.campaigns.len() == 1
    )));
}

#[test]
fn reconnect_does_not_replay_completion_but_a_new_done_event_can() {
    use questmancer::{
        herdr::protocol::AgentStatus,
        scene::{
            pixel::{PixelSize, Rgb, RgbBuffer},
            presentation::ScenePresentation,
            render_scene_for_world,
            snapshot::SceneSnapshot,
        },
    };
    let (mut model, _) = connect_for_output();
    model.switch_to(View::Delve);
    model.set_preferences(DisplayPreferences {
        motion: Motion::Full,
        ..DisplayPreferences::default()
    });
    let deadline = |model: &Model| {
        let mut pixels = RgbBuffer::filled(0, 0, Rgb::BLACK);
        render_scene_for_world(
            &SceneSnapshot::from_model(model),
            &ScenePresentation::from_model(model),
            PixelSize::new(30, 30),
            &mut pixels,
        )
        .next_frame_in
    };
    apply_connection_update(
        &mut model,
        status_update_with_revision("done", 8),
        Timestamp::from_millis(2_000),
    );
    model.set_now(Timestamp::from_millis(2_125));
    assert!(deadline(&model).is_some());
    let attention = model.selected_agent().unwrap().attention.clone();

    apply_connection_update(
        &mut model,
        ConnectionUpdate::Disconnected("test disconnect".to_owned()),
        Timestamp::from_millis(2_150),
    );
    assert_eq!(deadline(&model), None);
    let mut resumed = snapshot();
    resumed.panes[0].agent_status = AgentStatus::Done;
    resumed.panes[0].revision = 8;
    resumed.agents[0].agent_status = AgentStatus::Done;
    apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(resumed),
        Timestamp::from_millis(2_200),
    );
    model.set_now(Timestamp::from_millis(2_250));
    assert_eq!(
        model.selected_agent().unwrap().attention,
        attention,
        "connection changes cannot erase the summons"
    );
    assert_eq!(
        deadline(&model),
        None,
        "retained completion must stay settled after reconnect"
    );

    apply_connection_update(
        &mut model,
        status_update_with_revision("working", 9),
        Timestamp::from_millis(2_300),
    );
    apply_connection_update(
        &mut model,
        status_update_with_revision("done", 10),
        Timestamp::from_millis(2_400),
    );
    model.set_now(Timestamp::from_millis(2_525));
    assert!(
        deadline(&model).is_some(),
        "a new completion in this connection must still play"
    );
}

fn party_world_deadline(model: &Model) -> Option<std::time::Duration> {
    use questmancer::scene::{
        pixel::{PixelSize, Rgb, RgbBuffer},
        presentation::ScenePresentation,
        render_scene_for_world,
        snapshot::SceneSnapshot,
    };
    let mut pixels = RgbBuffer::filled(0, 0, Rgb::BLACK);
    render_scene_for_world(
        &SceneSnapshot::from_model(model),
        &ScenePresentation::from_model(model),
        PixelSize::new(160, 90),
        &mut pixels,
    )
    .next_frame_in
}

#[test]
fn party_counsel_gestures_follow_presence_episodes_across_reports_and_reconnects() {
    use questmancer::{domain::AdventurerClass, herdr::protocol::AgentStatus};
    use std::time::Duration;
    for class in [
        AdventurerClass::Wizard,
        AdventurerClass::Ranger,
        AdventurerClass::Barbarian,
    ] {
        let mut model = connected_model_with_presence(Presence::Working);
        model.set_connection_at(ConnectionState::Connected, Timestamp::from_millis(1_000));
        let mut saved = questmancer::persistence::PersistedStateV1::capture(&model);
        for persona in saved.personas.values_mut() {
            persona.class = class;
        }
        model
            .durable_intent_mut()
            .seed(&saved)
            .expect("valid pilot persona fixture");
        model.set_preferences(DisplayPreferences {
            motion: Motion::Full,
            ..DisplayPreferences::default()
        });
        apply_connection_update(
            &mut model,
            status_update_with_revision("blocked", 8),
            Timestamp::from_millis(2_000),
        );
        model.set_now(Timestamp::from_millis(2_000));
        assert_eq!(
            party_world_deadline(&model),
            Some(Duration::from_millis(600))
        );
        apply_connection_update(
            &mut model,
            status_update_with_revision("blocked", 9),
            Timestamp::from_millis(2_200),
        );
        model.set_now(Timestamp::from_millis(2_200));
        for view in [View::Delve, View::Guild] {
            model.switch_to(view);
            assert_eq!(
                party_world_deadline(&model),
                Some(Duration::from_millis(400)),
                "reports and room changes cannot restart the gesture"
            );
        }
        apply_connection_update(
            &mut model,
            ConnectionUpdate::Disconnected("gesture test".to_owned()),
            Timestamp::from_millis(2_250),
        );
        assert_eq!(party_world_deadline(&model), None);
        let mut resumed = snapshot();
        resumed.panes[0].agent_status = AgentStatus::Blocked;
        resumed.panes[0].revision = 9;
        resumed.agents[0].agent_status = AgentStatus::Blocked;
        apply_connection_update(
            &mut model,
            ConnectionUpdate::Connected(resumed),
            Timestamp::from_millis(2_300),
        );
        model.set_now(Timestamp::from_millis(2_350));
        assert_eq!(
            party_world_deadline(&model),
            None,
            "retained blocked facts must remain still"
        );
        apply_connection_update(
            &mut model,
            status_update_with_revision("working", 10),
            Timestamp::from_millis(2_400),
        );
        apply_connection_update(
            &mut model,
            status_update_with_revision("blocked", 11),
            Timestamp::from_millis(2_500),
        );
        model.set_now(Timestamp::from_millis(2_500));
        assert_eq!(
            party_world_deadline(&model),
            Some(Duration::from_millis(600)),
            "a new blocked episode has a new gesture"
        );
        model.set_now(Timestamp::from_millis(3_100));
        for view in [View::Delve, View::Guild] {
            model.switch_to(view);
            assert_eq!(party_world_deadline(&model), None);
        }
    }
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
            data: json!({"pane_id": "w1:p1", "workspace_id": "w1", "agent_status": "done", "revision":8, "terminal_id":"terminal-1", "agent_session":{"source":"codex","agent":"codex","kind":"id","value":"session-123"}}),
        }),
        Timestamp::from_millis(2_000),
    );

    assert!(effects.agent_commands.iter().any(|command| matches!(
        command,
        AgentCommand::LoadOutput { pane_id, .. } if pane_id.as_str() == "w1:p1"
    )));
    assert!(effects.agent_commands.iter().any(|command| matches!(
        command,
        AgentCommand::PublishMarginalia(projection)
            if projection.agents.len() == 1 && projection.campaigns.len() == 1
    )));
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
                "agent_status": "done",
                "revision":8,
                "terminal_id":"terminal-1", "agent_session":{"source":"codex","agent":"codex","kind":"id","value":"session-123"}
            }),
        }),
        Timestamp::from_millis(2_000),
    );

    assert_eq!(model.selected_agent_key(), Some(&selected));
    assert_eq!(model.domain().agents[&selected].presence, Presence::Done);
}

#[test]
fn older_output_result_cannot_overwrite_a_newer_selected_preview() {
    let (mut model, older) = connect_for_output();
    let newer = refresh_output(&mut model);
    output_loaded(&mut model, newer, 20, "newer output");
    output_loaded(&mut model, older, 10, "older output");

    let preview = model.output_preview().unwrap();
    assert_eq!(preview.revision, 20);
    assert_eq!(preview.text, "newer output");
}

#[test]
fn stale_output_failure_preserves_newer_text_and_feedback() {
    let (mut model, older) = connect_for_output();
    let newer = refresh_output(&mut model);
    output_loaded(&mut model, newer, 20, "current output");
    model.set_action_feedback("counsel issued".into());

    apply_command_result(
        &mut model,
        CommandResult::OutputFailed {
            pane_id: PaneId::new("w1:p1"),
            request: older,
            message: "old read failed".into(),
        },
        Timestamp::from_millis(3_000),
    );

    let preview = model.output_preview().unwrap();
    assert_eq!(preview.text, "current output");
    assert_eq!(preview.revision, 20);
    assert_eq!(preview.error, None);
    assert_eq!(model.action_feedback(), Some("counsel issued"));
}

#[test]
fn even_the_latest_output_request_cannot_regress_the_current_revision() {
    let (mut model, first) = connect_for_output();
    output_loaded(&mut model, first, 20, "current output");
    let newer = refresh_output(&mut model);

    apply_command_result(
        &mut model,
        CommandResult::OutputLoaded {
            pane_id: PaneId::new("w1:p1"),
            request: newer,
            revision: 10,
            text: "stale output".into(),
            truncated: true,
        },
        Timestamp::from_millis(3_000),
    );

    let preview = model.output_preview().unwrap();
    assert_eq!(preview.text, "current output");
    assert_eq!(preview.revision, 20);
    assert!(!preview.loading);
    assert_eq!(model.action_feedback(), None);

    let current = refresh_output(&mut model);
    output_loaded(&mut model, current, 20, "same revision refreshed");
    assert_eq!(
        model.output_preview().unwrap().text,
        "same revision refreshed"
    );
}

#[test]
fn superseded_output_does_not_settle_a_newer_pending_read_even_at_a_higher_revision() {
    let (mut model, older) = connect_for_output();
    let newer = refresh_output(&mut model);
    output_loaded(&mut model, older, 100, "superseded output");
    assert!(model.output_preview().unwrap().loading);
    assert!(model.output_preview().unwrap().text.is_empty());
    output_loaded(&mut model, newer, 20, "current request");
    assert_eq!(model.output_preview().unwrap().text, "current request");
}

#[test]
fn leaving_and_reselecting_an_adventurer_invalidates_the_old_output_request() {
    let mut model = model_with_two_distinct_personas();
    model.set_connection(ConnectionState::Connected);
    let old = refresh_output(&mut model);
    model.select_last_agent();
    assert!(model.output_preview().is_none());
    model.select_first_agent();
    output_loaded(&mut model, old, 100, "old selection lifetime");
    assert!(model.output_preview().is_none());

    let current = refresh_output(&mut model);
    output_loaded(&mut model, current, 20, "reselected output");
    assert_eq!(model.output_preview().unwrap().text, "reselected output");
}

#[test]
fn removal_and_recreation_of_a_pane_cannot_revive_its_old_output_request() {
    let (mut model, old) = connect_for_output();
    let mut empty = snapshot();
    empty.agents.clear();
    apply_snapshot(&mut model, empty, Timestamp::from_millis(2_000));
    assert!(model.selected_agent().is_none());
    output_loaded(&mut model, old, 100, "removed pane");
    assert!(model.output_preview().is_none());

    let restored = apply_snapshot(&mut model, snapshot(), Timestamp::from_millis(3_000));
    let current = output_request(&restored.agent_commands);
    output_loaded(&mut model, old, 100, "old pane incarnation");
    assert!(model.output_preview().unwrap().loading);
    output_loaded(&mut model, current, 1, "recreated pane");
    assert_eq!(model.output_preview().unwrap().text, "recreated pane");
}

#[test]
fn replacing_the_selected_agent_on_the_same_pane_invalidates_output() {
    let (mut model, old) = connect_for_output();
    let mut domain = model.domain().clone();
    let mut replacement = domain.agents.values().next().unwrap().clone();
    replacement.key = AgentKey::new("replacement-agent");
    domain.selected_agent = Some(replacement.key.clone());
    domain.agents.clear();
    domain.agents.insert(replacement.key.clone(), replacement);
    model.replace_domain(domain);

    output_loaded(&mut model, old, 100, "former adventurer");
    assert!(model.output_preview().is_none());
    let current = refresh_output(&mut model);
    output_loaded(&mut model, current, 1, "replacement adventurer");
    assert_eq!(
        model.output_preview().unwrap().text,
        "replacement adventurer"
    );
}

#[test]
fn every_connection_boundary_rejects_old_reads_and_reloads_unchanged_selection() {
    for boundary in [
        ConnectionUpdate::Disconnected("closed".into()),
        ConnectionUpdate::Resyncing,
        ConnectionUpdate::Reconnecting {
            attempt: 1,
            delay: std::time::Duration::from_millis(250),
        },
        ConnectionUpdate::Incompatible {
            expected: 22,
            actual: 19,
        },
        ConnectionUpdate::Connected(snapshot()),
    ] {
        let (mut model, first) = connect_for_output();
        output_loaded(&mut model, first, 20, "former connection");
        let pending = refresh_output(&mut model);
        apply_connection_update(&mut model, boundary, Timestamp::from_millis(3_000));

        let reconnected = apply_connection_update(
            &mut model,
            ConnectionUpdate::Connected(snapshot()),
            Timestamp::from_millis(4_000),
        );
        let current = output_request(&reconnected.agent_commands);
        assert_ne!(current, pending);
        output_loaded(&mut model, pending, 100, "late from former connection");
        apply_command_result(
            &mut model,
            CommandResult::OutputFailed {
                pane_id: PaneId::new("w1:p1"),
                request: pending,
                message: "former connection failed".into(),
            },
            Timestamp::from_millis(4_000),
        );
        assert!(model.output_preview().unwrap().loading);
        assert_eq!(model.output_preview().unwrap().error, None);
        output_loaded(&mut model, current, 1, "new connection");
        assert_eq!(model.output_preview().unwrap().text, "new connection");
        assert_eq!(model.output_preview().unwrap().revision, 1);
    }
}

#[test]
fn unsolicited_output_results_are_ignored_without_a_selected_request() {
    let mut model = Model::new(View::Guild);
    output_loaded(&mut model, OutputRequest(99), 100, "unowned output");
    apply_command_result(
        &mut model,
        CommandResult::OutputFailed {
            pane_id: PaneId::new("w1:p1"),
            request: OutputRequest(99),
            message: "unowned failure".into(),
        },
        Timestamp::from_millis(3_000),
    );
    assert!(model.output_preview().is_none());
}

#[test]
fn output_result_must_match_both_request_and_pane() {
    let (mut model, current) = connect_for_output();
    apply_command_result(
        &mut model,
        CommandResult::OutputLoaded {
            pane_id: PaneId::new("w1:p2"),
            request: current,
            revision: 100,
            text: "wrong pane".into(),
            truncated: true,
        },
        Timestamp::from_millis(3_000),
    );
    assert!(model.output_preview().unwrap().loading);
    assert_eq!(model.action_feedback(), None);
    output_loaded(&mut model, current, 20, "correct pane");
    assert_eq!(model.output_preview().unwrap().text, "correct pane");
}

#[test]
fn an_exit_event_invalidates_the_pending_read_without_loading_the_exited_pane() {
    let (mut model, request) = connect_for_output();
    let effects = apply_connection_update(
        &mut model,
        ConnectionUpdate::Event(WireEvent {
            event: "pane.exited".into(),
            data: json!({"pane_id": "w1:p1", "workspace_id": "w1", "revision": 8, "terminal_id":"terminal-1", "agent_session":{"source":"codex","agent":"codex","kind":"id","value":"session-123"}}),
        }),
        Timestamp::from_millis(3_000),
    );
    assert!(
        !effects
            .agent_commands
            .iter()
            .any(|command| matches!(command, AgentCommand::LoadOutput { .. }))
    );
    output_loaded(&mut model, request, 100, "departed output");
    assert!(model.output_preview().is_none());
}

#[test]
fn disconnected_refresh_does_not_read_a_retained_agent_snapshot() {
    let (mut model, request) = connect_for_output();
    apply_connection_update(
        &mut model,
        ConnectionUpdate::Disconnected("closed".into()),
        Timestamp::from_millis(3_000),
    );
    assert!(model.selected_agent().is_some());
    assert!(
        reduce_action(&mut model, Action::Refresh)
            .commands
            .is_empty()
    );
    output_loaded(&mut model, request, 100, "offline output");
    assert!(model.output_preview().is_none());
}

#[test]
fn a_pending_read_cannot_populate_the_managed_pane() {
    let (mut model, request) = connect_for_output();
    model.set_managed_pane_id(Some(PaneId::new("w1:p1")));
    output_loaded(&mut model, request, 20, "managed output");
    assert!(model.output_preview().is_none());
    assert!(
        reduce_action(&mut model, Action::Refresh)
            .commands
            .is_empty()
    );
}

#[test]
fn output_and_discovery_results_update_app_state() {
    let (mut model, request) = connect_for_output();

    apply_command_result(
        &mut model,
        CommandResult::OutputLoaded {
            pane_id: PaneId::new("w1:p1"),
            request,
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
    let mut model = connected_model_with_presence(Presence::Working);
    let _ = reduce_action(&mut model, Action::Counsel);
    let _ = reduce_action(&mut model, Action::TypeCharacter('x'));
    let sent = reduce_action(&mut model, Action::Submit);
    let request = match sent.commands.as_slice() {
        [AgentCommand::SendCounsel { request, .. }] => *request,
        commands => panic!("expected one counsel command, got {commands:?}"),
    };

    apply_command_result(
        &mut model,
        CommandResult::CounselSent {
            pane_id: PaneId::new("w1:p1"),
            request,
        },
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
fn failed_enter_becomes_a_submission_only_retry() {
    let mut model = connected_model_with_presence(Presence::Working);
    let _ = reduce_action(&mut model, Action::Counsel);
    for character in "use jsonb".chars() {
        let _ = reduce_action(&mut model, Action::TypeCharacter(character));
    }
    let sent = reduce_action(&mut model, Action::Submit);
    let (pane_id, request) = match sent.commands.as_slice() {
        [
            AgentCommand::SendCounsel {
                pane_id, request, ..
            },
        ] => (pane_id.clone(), *request),
        commands => panic!("expected one counsel command, got {commands:?}"),
    };

    apply_command_result(
        &mut model,
        CommandResult::CounselSubmissionFailed {
            pane_id: pane_id.clone(),
            request,
            message: "enter was not accepted".to_owned(),
        },
        Timestamp::from_millis(2_000),
    );

    assert!(matches!(
        model.counsel_phase(),
        Some(CounselPhase::SubmissionFailed { request: failed, .. }) if *failed == request
    ));
    assert_eq!(
        model.status_message(),
        Some("Counsel was written but not submitted: enter was not accepted")
    );
    let retry = reduce_action(&mut model, Action::Submit);
    assert_eq!(
        retry.commands,
        vec![AgentCommand::SubmitCounsel { pane_id, request }]
    );
}

#[test]
fn output_failure_is_scoped_to_the_selected_scrying_preview() {
    let (mut model, request) = connect_for_output();
    let before = model.domain().clone();

    apply_command_result(
        &mut model,
        CommandResult::OutputFailed {
            pane_id: PaneId::new("w1:p1"),
            request,
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

    assert_live_domain_eq(model.domain(), &connected_domain);
    assert_eq!(effects, RuntimeEffects::default());
}

#[tokio::test]
async fn runtime_shutdown_cancels_supervisor_and_command_tasks() {
    let directory = tempdir().unwrap();
    let socket_path = directory.path().join("herdr.sock");
    let _listener = UnixListener::bind(&socket_path).unwrap();
    let environment = HerdrEnvironment::new(&socket_path, "/usr/bin/herdr");
    let mut connection = RuntimeConnection::start(&environment);
    connection.schedule([AgentCommand::RefreshSnapshot(
        questmancer::snapshot_refresh::SnapshotRequest::default(),
    )]);
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
    connection.schedule([AgentCommand::FocusPane(PaneId::new("w1:p1"))]);

    timeout(std::time::Duration::from_secs(1), async {
        loop {
            if matches!(
                connection.next_event().await,
                RuntimeEvent::Command(CommandResult::Failed {
                    operation: "focus pane",
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

/// Reordering Herdr's own agent list is opt-in, because it changes shared
/// Herdr UI rather than anything inside Questmancer's pane.
#[test]
fn the_urgency_view_is_only_requested_when_the_user_asked_for_it() {
    for (asked_for, expected) in [(false, false), (true, true)] {
        let mut model = Model::new(View::Guild);
        model.set_settings(RuntimeSettings {
            sidebar_urgency_order: asked_for,
            ..RuntimeSettings::default()
        });
        let response: SuccessResponse<SessionSnapshotResult> =
            serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();

        let effects = apply_connection_update(
            &mut model,
            ConnectionUpdate::Connected(response.result.snapshot),
            Timestamp::from_millis(1_000),
        );

        assert_eq!(
            effects
                .agent_commands
                .contains(&AgentCommand::SetUrgencyView),
            expected,
            "sidebar_urgency_order = {asked_for} must {} the view",
            if expected { "request" } else { "leave alone" }
        );
    }
}

/// Herdr's agent view is transient, so a reconnect drops it. Asking again on
/// every fresh connection is what keeps the order alive across a server
/// restart.
#[test]
fn the_urgency_view_is_requested_again_after_reconnecting() {
    let mut model = Model::new(View::Guild);
    model.set_settings(RuntimeSettings {
        sidebar_urgency_order: true,
        ..RuntimeSettings::default()
    });
    let snapshot = || {
        let response: SuccessResponse<SessionSnapshotResult> =
            serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
        response.result.snapshot
    };

    for round in 1..=2 {
        let _ = apply_connection_update(
            &mut model,
            ConnectionUpdate::Disconnected("server restarted".to_owned()),
            Timestamp::from_millis(round * 1_000),
        );
        let effects = apply_connection_update(
            &mut model,
            ConnectionUpdate::Connected(snapshot()),
            Timestamp::from_millis(round * 1_000 + 1),
        );
        assert!(
            effects
                .agent_commands
                .contains(&AgentCommand::SetUrgencyView),
            "connection {round} must re-request the view"
        );
    }
}

/// Events that record an adventurer getting stuck, arriving or resting earn
/// nothing. Paying for a block would reward agents for blocking.
#[test]
fn only_finished_work_is_worth_standing() {
    use questmancer::domain::ChronicleEvent;

    assert_eq!(ChronicleEvent::SpoilsReturned.experience(), 10);
    assert_eq!(ChronicleEvent::CampaignClosed.experience(), 25);
    for quiet in [
        ChronicleEvent::AdventurerJoined,
        ChronicleEvent::DelveBegan,
        ChronicleEvent::CounselRequested,
        ChronicleEvent::AdventurerRested,
        ChronicleEvent::AdventurerDeparted,
    ] {
        assert_eq!(
            quiet.experience(),
            0,
            "{quiet:?} must not be worth standing"
        );
    }
}

#[test]
fn a_refresh_from_before_disconnect_cannot_replace_the_new_connection_baseline() {
    let (mut model, _) = connect_for_output();
    let old_snapshot = snapshot();
    let requested = apply_connection_update(
        &mut model,
        ConnectionUpdate::Event(WireEvent {
            event: "pane_created".into(),
            data: json!({"pane_id": "w1:p9"}),
        }),
        Timestamp::from_millis(1_500),
    );
    let request = snapshot_request(&requested.agent_commands);
    apply_connection_update(
        &mut model,
        ConnectionUpdate::Disconnected("test disconnect".into()),
        Timestamp::from_millis(2_000),
    );
    let mut fresh = snapshot();
    fresh.workspaces[0].label = "after reconnect".into();
    fresh.agents[0].agent_status = questmancer::herdr::protocol::AgentStatus::Working;
    fresh.agents[0].revision = 20;
    apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(fresh),
        Timestamp::from_millis(3_000),
    );
    let baseline = model.domain().clone();
    let experience = model.experience();
    let effects = apply_command_result(
        &mut model,
        CommandResult::SnapshotLoaded {
            request,
            snapshot: Box::new(old_snapshot),
        },
        Timestamp::from_millis(4_000),
    );
    assert_eq!(
        model.domain(),
        &baseline,
        "a prior-connection result rolled back live facts"
    );
    assert_eq!(model.experience(), experience);
    assert_eq!(effects, RuntimeEffects::default());
}

fn changed_snapshot(
    status: questmancer::herdr::protocol::AgentStatus,
    revision: u64,
) -> SessionSnapshot {
    let mut next = snapshot();
    for agent in &mut next.agents {
        agent.agent_status = status;
        agent.revision = revision;
    }
    for pane in &mut next.panes {
        pane.agent_status = status;
        pane.revision = revision;
    }
    next
}

fn begin_snapshot(model: &mut Model) -> questmancer::snapshot_refresh::SnapshotRequest {
    snapshot_request(&questmancer::runtime_loop::request_snapshot_refresh(model).agent_commands)
}

fn deliver_snapshot(
    model: &mut Model,
    request: questmancer::snapshot_refresh::SnapshotRequest,
    snapshot: SessionSnapshot,
) -> RuntimeEffects {
    apply_command_result(
        model,
        CommandResult::SnapshotLoaded {
            request,
            snapshot: Box::new(snapshot),
        },
        Timestamp::from_millis(5_000),
    )
}

#[test]
fn snapshot_refresh_bursts_coalesce_and_pending_work_uses_installed_facts() {
    use questmancer::herdr::protocol::AgentStatus;
    let (mut model, _) = connect_for_output();
    let first = begin_snapshot(&mut model);
    for _ in 0..100 {
        assert_eq!(
            questmancer::runtime_loop::request_snapshot_refresh(&mut model),
            RuntimeEffects::default()
        );
    }
    let applied = deliver_snapshot(&mut model, first, changed_snapshot(AgentStatus::Idle, 8));
    assert_eq!(applied.persistence, vec![Command::PersistState]);
    assert!(model.domain().chronicle.entries().is_empty());
    assert_eq!(model.experience(), 0);
    let pending = snapshot_request(&applied.agent_commands);
    assert_ne!(pending.id, first.id);
    assert_ne!(pending.generation, first.generation);
    let finished = deliver_snapshot(&mut model, pending, changed_snapshot(AgentStatus::Idle, 8));
    assert!(!finished.resubscribe);
    assert!(
        !finished
            .agent_commands
            .iter()
            .any(|command| matches!(command, AgentCommand::RefreshSnapshot(_)))
    );
}

#[test]
fn duplicate_and_late_results_cannot_complete_a_new_snapshot_request() {
    use questmancer::herdr::protocol::AgentStatus;
    let (mut model, _) = connect_for_output();
    let first = begin_snapshot(&mut model);
    deliver_snapshot(&mut model, first, changed_snapshot(AgentStatus::Idle, 8));
    let second = begin_snapshot(&mut model);
    let stable = model.domain().clone();
    assert_eq!(
        deliver_snapshot(&mut model, first, changed_snapshot(AgentStatus::Done, 90)),
        RuntimeEffects::default()
    );
    assert_eq!(model.domain(), &stable);
    let failure = apply_command_result(
        &mut model,
        CommandResult::SnapshotFailed {
            request: first,
            message: "obsolete".into(),
        },
        Timestamp::from_millis(6_000),
    );
    assert_eq!(failure, RuntimeEffects::default());
    assert_eq!(model.action_feedback(), None);
    deliver_snapshot(
        &mut model,
        second,
        changed_snapshot(AgentStatus::Working, 9),
    );
    assert_eq!(model.selected_agent().unwrap().presence, Presence::Working);
    assert!(model.domain().chronicle.entries().is_empty());
}

#[test]
fn two_overtaken_refreshes_resubscribe_without_rolling_back_or_backfilling() {
    use questmancer::herdr::protocol::AgentStatus;
    let (mut model, _) = connect_for_output();
    let first = begin_snapshot(&mut model);
    apply_connection_update(
        &mut model,
        status_update_with_revision("working", 8),
        Timestamp::from_millis(2_000),
    );
    let retry = deliver_snapshot(&mut model, first, snapshot());
    let second = snapshot_request(&retry.agent_commands);
    assert_eq!(model.selected_agent().unwrap().presence, Presence::Working);
    apply_connection_update(
        &mut model,
        status_update_with_revision("idle", 9),
        Timestamp::from_millis(3_000),
    );
    let history = model.domain().chronicle.clone();
    let recovered = deliver_snapshot(
        &mut model,
        second,
        changed_snapshot(AgentStatus::Working, 8),
    );
    assert!(recovered.resubscribe);
    assert!(recovered.agent_commands.is_empty());
    assert!(recovered.persistence.is_empty());
    assert_eq!(model.selected_agent().unwrap().presence, Presence::Idle);
    assert_eq!(model.connection(), &ConnectionState::Connecting);
    assert_eq!(
        questmancer::runtime_loop::request_snapshot_refresh(&mut model),
        RuntimeEffects::default()
    );
    let baseline = apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(changed_snapshot(AgentStatus::Done, 10)),
        Timestamp::from_millis(6_000),
    );
    assert!(
        !baseline
            .persistence
            .iter()
            .any(Command::is_chronicle_append)
    );
    assert_eq!(model.domain().chronicle, history);
    assert_eq!(model.experience(), 0);
    assert_eq!(
        deliver_snapshot(&mut model, second, snapshot()),
        RuntimeEffects::default()
    );
    assert_eq!(model.selected_agent().unwrap().presence, Presence::Done);
}

#[test]
fn local_interaction_and_marginalia_do_not_supersede_snapshot_work() {
    use questmancer::herdr::protocol::AgentStatus;
    let (mut model, _) = connect_for_output();
    let request = begin_snapshot(&mut model);
    let _ = reduce_action(&mut model, Action::Search);
    let _ = reduce_action(&mut model, Action::TypeCharacter('x'));
    model.set_now(Timestamp::from_millis(10_000));
    for name in ["pane.updated", "workspace.metadata_updated"] {
        apply_connection_update(
            &mut model,
            ConnectionUpdate::Event(WireEvent {
                event: name.into(),
                data: json!({}),
            }),
            Timestamp::from_millis(4_000),
        );
    }
    let applied = deliver_snapshot(&mut model, request, changed_snapshot(AgentStatus::Idle, 8));
    assert!(!applied.resubscribe);
    assert_eq!(applied.persistence, vec![Command::PersistState]);
    assert_eq!(model.selected_agent().unwrap().presence, Presence::Idle);
}

#[test]
fn resync_does_not_launch_a_competing_command_snapshot() {
    let (mut model, _) = connect_for_output();
    let request = begin_snapshot(&mut model);
    let resync = apply_connection_update(
        &mut model,
        ConnectionUpdate::Resyncing,
        Timestamp::from_millis(2_000),
    );
    assert_eq!(resync, RuntimeEffects::default());
    assert_eq!(model.connection(), &ConnectionState::Connecting);
    assert_eq!(
        deliver_snapshot(&mut model, request, snapshot()),
        RuntimeEffects::default()
    );
}

#[test]
fn conflicting_equal_revision_snapshot_requires_reconciliation() {
    use questmancer::herdr::protocol::AgentStatus;
    let (mut model, _) = connect_for_output();
    let request = begin_snapshot(&mut model);
    let before = model.domain().clone();
    let effects = deliver_snapshot(
        &mut model,
        request,
        changed_snapshot(AgentStatus::Working, 7),
    );
    assert_eq!(model.domain(), &before);
    assert!(effects.persistence.is_empty());
    assert_ne!(snapshot_request(&effects.agent_commands).id, request.id);
}

#[test]
fn conflicting_equal_revision_status_requests_snapshot_without_history() {
    let (mut model, _) = connect_for_output();
    let before = model.domain().clone();
    let effects = apply_connection_update(
        &mut model,
        status_update_with_revision("working", 7),
        Timestamp::from_millis(2_000),
    );
    assert_eq!(model.domain(), &before);
    assert!(effects.persistence.is_empty());
    let _ = snapshot_request(&effects.agent_commands);
}

#[test]
fn topology_hints_supersede_refreshes_before_the_new_party_is_known() {
    let (mut model, _) = connect_for_output();
    let first = begin_snapshot(&mut model);
    let before = model.domain().clone();
    let queued = apply_connection_update(
        &mut model,
        ConnectionUpdate::Event(WireEvent {
            event: "pane_created".into(),
            data: json!({"pane_id":"w1:p9"}),
        }),
        Timestamp::from_millis(2_000),
    );
    assert!(
        queued.agent_commands.is_empty(),
        "the in-flight read should coalesce the topology refresh"
    );
    let retry = deliver_snapshot(&mut model, first, snapshot());
    assert_eq!(model.domain(), &before);
    let second = snapshot_request(&retry.agent_commands);
    assert_ne!(second.id, first.id);
    assert_ne!(second.generation, first.generation);
}

#[test]
fn offline_and_unsolicited_snapshot_results_cannot_establish_a_baseline() {
    let mut model = Model::new(View::Guild);
    let before = model.clone();
    assert_eq!(
        questmancer::runtime_loop::request_snapshot_refresh(&mut model),
        RuntimeEffects::default()
    );
    assert_eq!(
        deliver_snapshot(
            &mut model,
            questmancer::snapshot_refresh::SnapshotRequest::default(),
            snapshot()
        ),
        RuntimeEffects::default()
    );
    assert_eq!(model, before);
}

#[test]
fn matching_snapshot_failure_releases_only_one_coalesced_pending_request() {
    let (mut model, _) = connect_for_output();
    let first = begin_snapshot(&mut model);
    for _ in 0..10 {
        questmancer::runtime_loop::request_snapshot_refresh(&mut model);
    }
    let failed = apply_command_result(
        &mut model,
        CommandResult::SnapshotFailed {
            request: first,
            message: "read failed".into(),
        },
        Timestamp::from_millis(2_000),
    );
    let next = snapshot_request(&failed.agent_commands);
    assert_ne!(next.id, first.id);
    assert_eq!(failed.agent_commands.len(), 1);
    assert_eq!(
        model.action_feedback(),
        Some("refresh snapshot failed: read failed")
    );
    let repeated = apply_command_result(
        &mut model,
        CommandResult::SnapshotFailed {
            request: first,
            message: "obsolete".into(),
        },
        Timestamp::from_millis(3_000),
    );
    assert_eq!(repeated, RuntimeEffects::default());
    assert_eq!(
        model.action_feedback(),
        Some("refresh snapshot failed: read failed")
    );
    assert_eq!(
        deliver_snapshot(&mut model, next, snapshot()).persistence,
        vec![Command::PersistState]
    );
}

#[test]
fn unsupported_and_ambiguous_snapshots_never_replace_the_qualified_party() {
    for ambiguous in [false, true] {
        let (mut model, _) = connect_for_output();
        let before = model.domain().clone();
        let first = begin_snapshot(&mut model);
        let mut invalid = snapshot();
        if ambiguous {
            invalid.agents.push(invalid.agents[0].clone());
        } else {
            invalid.protocol = 23;
        }
        let retry = deliver_snapshot(&mut model, first, invalid.clone());
        let second = snapshot_request(&retry.agent_commands);
        let resync = deliver_snapshot(&mut model, second, invalid);
        assert!(resync.resubscribe);
        assert_live_domain_eq(model.domain(), &before);
        assert!(model.domain().chronicle.entries().is_empty());
    }
}

#[test]
fn refresh_with_unsubscribed_panes_requires_a_quiet_new_baseline() {
    let (mut model, _) = connect_for_output();
    let request = begin_snapshot(&mut model);
    let before = model.domain().clone();
    let mut changed = snapshot();
    changed.agents[0].pane_id = "w1:new".into();
    let effects = deliver_snapshot(&mut model, request, changed.clone());
    assert!(
        effects.resubscribe,
        "new pane facts require a subscription baseline"
    );
    assert_live_domain_eq(model.domain(), &before);
    assert!(effects.persistence.is_empty());
    assert!(effects.agent_commands.is_empty());
    apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(changed),
        Timestamp::from_millis(6_000),
    );
    assert_eq!(
        model
            .domain()
            .agents
            .values()
            .next()
            .unwrap()
            .pane_id
            .as_str(),
        "w1:new"
    );
    assert!(model.domain().chronicle.entries().is_empty());
    assert_eq!(model.experience(), 0);
    let active = begin_snapshot(&mut model);
    let late = deliver_snapshot(&mut model, request, snapshot());
    assert_eq!(late, RuntimeEffects::default());
    assert_ne!(active.epoch, request.epoch);
}

fn assert_live_domain_eq(actual: &DomainState, expected: &DomainState) {
    assert_eq!(actual.agents, expected.agents);
    assert_eq!(actual.campaigns, expected.campaigns);
    assert_eq!(actual.selected_agent, expected.selected_agent);
    assert_eq!(actual.chronicle, expected.chronicle);
}
