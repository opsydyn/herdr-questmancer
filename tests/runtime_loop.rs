use herdr_webmaster::{
    app::{ConnectionState, Model, View},
    command::{CommandResult, DeskCommand},
    domain::{PaneId, Timestamp},
    herdr::{
        protocol::{SessionSnapshot, SessionSnapshotResult, SuccessResponse, WireEvent},
        supervisor::ConnectionUpdate,
    },
    runtime_loop::{apply_command_result, apply_connection_update},
};
use serde_json::json;

fn snapshot() -> SessionSnapshot {
    let response: SuccessResponse<SessionSnapshotResult> =
        serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
    response.result.snapshot
}

#[test]
fn connection_bootstrap_updates_model_and_lazily_loads_selected_output() {
    let mut model = Model::new(View::Desk);

    let commands = apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(snapshot()),
        Timestamp::from_millis(1_000),
    );

    assert_eq!(model.connection(), &ConnectionState::Connected);
    assert_eq!(model.domain().agents.len(), 1);
    assert!(commands.iter().any(|command| matches!(
        command,
        DeskCommand::LoadOutput { pane_id, lines: 80 }
            if pane_id.as_str() == "w1:p1"
    )));
    assert!(commands.contains(&DeskCommand::DiscoverReviewr));
}

#[test]
fn selected_status_change_refreshes_only_that_output() {
    let mut model = Model::new(View::Desk);
    apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(snapshot()),
        Timestamp::from_millis(1_000),
    );

    let commands = apply_connection_update(
        &mut model,
        ConnectionUpdate::Event(WireEvent {
            event: "pane.agent_status_changed".into(),
            data: json!({"pane_id": "w1:p1", "workspace_id": "w1", "agent_status": "done"}),
        }),
        Timestamp::from_millis(2_000),
    );

    assert_eq!(commands.len(), 1);
    assert!(matches!(
        &commands[0],
        DeskCommand::LoadOutput { pane_id, .. } if pane_id.as_str() == "w1:p1"
    ));
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
