use questmancer::{
    app::CounselRequest,
    app::{ConnectionState, DisplayPreferences, Model, Motion, Notice, RuntimeSettings, View},
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
    assert!(effects.agent_commands.iter().any(|command| matches!(
        command,
        AgentCommand::PublishMarginalia(projection)
            if projection.agents.len() == 1 && projection.campaigns.len() == 1
    )));
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
        CommandResult::CounselSent {
            pane_id: PaneId::new("w1:p1"),
            request: CounselRequest(1),
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
