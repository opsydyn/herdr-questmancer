use futures_util::FutureExt;
use herdr_webmaster::{
    app::{ConnectionState, Model, View},
    command::{CommandResult, DeskCommand},
    domain::{PaneId, Timestamp},
    herdr::{
        environment::HerdrEnvironment,
        protocol::{SessionSnapshot, SessionSnapshotResult, SuccessResponse, WireEvent},
        supervisor::ConnectionUpdate,
    },
    runtime_loop::{
        RuntimeConnection, RuntimeEvent, apply_command_result, apply_connection_update,
        bootstrap_model,
    },
    terminal::AnimationScheduler,
    ui::theatre::RenderCadence,
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

#[test]
fn startup_without_plugin_environment_is_usefully_offline() {
    let model = bootstrap_model(View::Desk, None);

    assert_eq!(model.connection(), &ConnectionState::Offline);
    assert_eq!(
        model.status_message(),
        Some("offline: launch from Herdr to connect to the live session")
    );
}

#[test]
fn startup_with_plugin_environment_begins_connecting() {
    let environment = HerdrEnvironment::new("/tmp/herdr.sock", "/usr/bin/herdr");

    let model = bootstrap_model(View::Desk, Some(&environment));

    assert_eq!(model.connection(), &ConnectionState::Connecting);
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

    let commands = apply_connection_update(
        &mut model,
        ConnectionUpdate::Disconnected("socket closed".into()),
        Timestamp::from_millis(2_000),
    );

    assert_eq!(model.domain(), &connected_domain);
    assert!(commands.is_empty());
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

    assert_future(herdr_webmaster::terminal::run(View::Desk));
}

#[tokio::test(start_paused = true)]
async fn animation_scheduler_wakes_once_at_the_next_visible_frame() {
    let cases = [
        (RenderCadence::Fps(8), 125),
        (RenderCadence::Fps(6), 167),
        (RenderCadence::Fps(2), 500),
        (RenderCadence::Fps(1), 1_000),
    ];

    for (cadence, milliseconds) in cases {
        let mut scheduler = AnimationScheduler::new();
        scheduler.reset(cadence);

        tokio::time::advance(std::time::Duration::from_millis(milliseconds - 1)).await;
        assert!(
            scheduler.wait().now_or_never().is_none(),
            "{cadence:?} woke before its next visible frame"
        );

        tokio::time::advance(std::time::Duration::from_millis(1)).await;
        assert!(
            scheduler.wait().now_or_never().is_some(),
            "{cadence:?} did not wake at its next visible frame"
        );
    }
}

#[tokio::test(start_paused = true)]
async fn event_driven_animation_scheduler_never_wakes_on_time_alone() {
    let mut scheduler = AnimationScheduler::new();
    scheduler.reset(RenderCadence::EventDriven);

    tokio::time::advance(std::time::Duration::from_secs(86_400)).await;

    assert!(scheduler.wait().now_or_never().is_none());
}

#[tokio::test(start_paused = true)]
async fn resetting_animation_scheduler_replaces_the_previous_deadline() {
    let mut scheduler = AnimationScheduler::new();
    scheduler.reset(RenderCadence::Fps(8));
    tokio::time::advance(std::time::Duration::from_millis(100)).await;

    scheduler.reset(RenderCadence::Fps(2));
    tokio::time::advance(std::time::Duration::from_millis(400)).await;
    assert!(scheduler.wait().now_or_never().is_none());

    tokio::time::advance(std::time::Duration::from_millis(100)).await;
    assert!(scheduler.wait().now_or_never().is_some());

    scheduler.reset(RenderCadence::EventDriven);
    tokio::time::advance(std::time::Duration::from_secs(1)).await;
    assert!(scheduler.wait().now_or_never().is_none());
}
