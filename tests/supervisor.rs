use std::{path::PathBuf, time::Duration};

use questmancer::herdr::{
    client::HerdrClient,
    supervisor::{Backoff, ConnectionSupervisor, ConnectionUpdate},
};
use serde_json::{Value, json};
use tempfile::TempDir;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
    sync::{mpsc, oneshot, watch},
    time::timeout,
};

fn listener() -> (TempDir, PathBuf, UnixListener) {
    let directory = tempfile::tempdir().expect("temporary socket directory");
    let path = directory.path().join("herdr.sock");
    let listener = UnixListener::bind(&path).expect("bind fake Herdr socket");
    (directory, path, listener)
}

fn fixture(name: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/herdr")
        .join(name);
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

async fn accept_request(listener: &UnixListener, expected_method: &str, mut response: Value) {
    let (mut stream, _) = listener.accept().await.unwrap();
    let request = read_request(&mut stream).await;
    assert_eq!(request["method"], expected_method);
    response["id"] = request["id"].clone();
    write_lines(&mut stream, &[response]).await;
}

async fn accept_subscription(listener: &UnixListener) -> UnixStream {
    let (mut stream, _) = listener.accept().await.unwrap();
    let request = read_request(&mut stream).await;
    assert_eq!(request["method"], "events.subscribe");
    write_lines(
        &mut stream,
        &[json!({"id": request["id"], "result": {"type": "subscription_started"}})],
    )
    .await;
    stream
}

async fn accept_baseline_subscription(listener: &UnixListener, snapshot: Value) -> UnixStream {
    let subscription = accept_subscription(listener).await;
    accept_request(listener, "session.snapshot", snapshot).await;
    subscription
}

async fn read_request(stream: &mut UnixStream) -> Value {
    let mut line = String::new();
    BufReader::new(stream)
        .read_line(&mut line)
        .await
        .expect("request line");
    serde_json::from_str(&line).expect("request JSON")
}

async fn write_lines(stream: &mut UnixStream, lines: &[Value]) {
    for line in lines {
        stream
            .write_all(serde_json::to_string(line).unwrap().as_bytes())
            .await
            .unwrap();
        stream.write_all(b"\n").await.unwrap();
    }
    stream.flush().await.unwrap();
}

async fn next_update(rx: &mut mpsc::Receiver<ConnectionUpdate>) -> ConnectionUpdate {
    timeout(Duration::from_secs(1), rx.recv())
        .await
        .expect("supervisor update timeout")
        .expect("supervisor channel closed")
}

fn test_supervisor(path: PathBuf) -> ConnectionSupervisor {
    ConnectionSupervisor::new(
        HerdrClient::new(path),
        Backoff::new(Duration::from_millis(1), Duration::from_millis(4)),
    )
}

#[test]
fn backoff_doubles_and_caps() {
    let backoff = Backoff::new(Duration::from_millis(1), Duration::from_millis(4));

    assert_eq!(backoff.delay(1), Duration::from_millis(1));
    assert_eq!(backoff.delay(2), Duration::from_millis(2));
    assert_eq!(backoff.delay(3), Duration::from_millis(4));
    assert_eq!(backoff.delay(20), Duration::from_millis(4));
}

#[tokio::test]
async fn bootstraps_in_order_and_forwards_events() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(async move {
        accept_request(&listener, "ping", fixture("pong.json")).await;
        accept_request(
            &listener,
            "session.snapshot",
            fixture("session_snapshot.json"),
        )
        .await;
        let mut subscription =
            accept_baseline_subscription(&listener, fixture("session_snapshot.json")).await;
        write_lines(
            &mut subscription,
            &[json!({"event": "workspace_focused", "data": {"type": "workspace_focused", "workspace_id": "w1"}})],
        )
        .await;
        tokio::time::sleep(Duration::from_secs(1)).await;
    });
    let (update_tx, mut update_rx) = mpsc::channel(16);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let supervisor = tokio::spawn(test_supervisor(path).run(update_tx, shutdown_rx));

    assert!(matches!(
        next_update(&mut update_rx).await,
        ConnectionUpdate::Connected(snapshot) if snapshot.protocol == 22
    ));
    assert!(matches!(
        next_update(&mut update_rx).await,
        ConnectionUpdate::Event(event) if event.event == "workspace_focused"
    ));

    shutdown_tx.send(true).unwrap();
    supervisor.await.unwrap();
    server.abort();
}

#[tokio::test]
async fn rejects_the_previous_herdr_082_capture() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(async move {
        accept_request(&listener, "ping", fixture("0.8.2/pong.json")).await;
    });
    let (tx, mut rx) = mpsc::channel(4);
    let (_stop, shutdown) = watch::channel(false);
    test_supervisor(path).run(tx, shutdown).await;
    assert_eq!(
        next_update(&mut rx).await,
        ConnectionUpdate::Incompatible {
            expected: 22,
            actual: 20
        }
    );
    assert!(rx.recv().await.is_none());
    server.await.unwrap();
}

#[tokio::test]
async fn bootstraps_herdr_090_capture_with_protocol_22() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(async move {
        accept_request(&listener, "ping", fixture("0.9.0/pong.json")).await;
        accept_request(
            &listener,
            "session.snapshot",
            fixture("0.9.0/session_snapshot.json"),
        )
        .await;
        let _subscription = accept_subscription(&listener).await;
        accept_request(
            &listener,
            "session.snapshot",
            fixture("0.9.0/session_snapshot.json"),
        )
        .await;
        tokio::time::sleep(Duration::from_secs(1)).await;
    });
    let (update_tx, mut update_rx) = mpsc::channel(4);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let supervisor = tokio::spawn(test_supervisor(path).run(update_tx, shutdown_rx));
    let update = next_update(&mut update_rx).await;
    assert!(
        matches!(&update, ConnectionUpdate::Connected(snapshot)
        if snapshot.protocol == 22 && snapshot.version == "0.9.0" && snapshot.agents.len() == 1),
        "the current stable Herdr capture must connect: {update:?}"
    );
    shutdown_tx.send(true).unwrap();
    supervisor.await.unwrap();
    server.abort();
}

fn capture_with_status(status: &str, revision: u64) -> Value {
    let mut response = fixture("0.9.0/session_snapshot.json");
    for collection in ["panes", "agents"] {
        response["result"]["snapshot"][collection][0]["agent_status"] = json!(status);
        response["result"]["snapshot"][collection][0]["revision"] = json!(revision);
    }
    response
}

async fn discover_and_subscribe(listener: &UnixListener, snapshot: Value) -> UnixStream {
    accept_request(listener, "ping", fixture("0.9.0/pong.json")).await;
    accept_request(listener, "session.snapshot", snapshot).await;
    accept_subscription(listener).await
}

async fn accept_pane_get(listener: &UnixListener, pane: Value) {
    let (mut stream, _) = listener.accept().await.unwrap();
    let request = read_request(&mut stream).await;
    assert_eq!(request["method"], "pane.get");
    assert_eq!(request["params"], json!({"pane_id": "w1:p1"}));
    write_lines(
        &mut stream,
        &[json!({"id": request["id"], "result": {"type": "pane_info", "pane": pane}})],
    )
    .await;
}

#[tokio::test]
async fn bootstrap_uses_the_snapshot_taken_after_subscription_acknowledgement() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(async move {
        let _subscription =
            discover_and_subscribe(&listener, capture_with_status("working", 9)).await;
        // A change before the subscription was accepted is no longer replayed.
        accept_request(
            &listener,
            "session.snapshot",
            capture_with_status("blocked", 10),
        )
        .await;
        tokio::time::sleep(Duration::from_secs(1)).await;
    });
    let (tx, mut rx) = mpsc::channel(4);
    let (stop, shutdown) = watch::channel(false);
    let supervisor = tokio::spawn(test_supervisor(path).run(tx, shutdown));
    let update = next_update(&mut rx).await;
    assert!(
        matches!(&update, ConnectionUpdate::Connected(snapshot)
        if snapshot.panes[0].agent_status == questmancer::herdr::protocol::AgentStatus::Blocked
            && snapshot.panes[0].revision == 10),
        "discovery is not the connected baseline: {update:?}"
    );
    stop.send(true).unwrap();
    supervisor.await.unwrap();
    server.abort();
}

#[tokio::test]
async fn bootstrap_rebuilds_subscriptions_if_the_party_changed_before_the_baseline() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(async move {
        let _old_subscription =
            discover_and_subscribe(&listener, capture_with_status("working", 9)).await;
        let mut changed = capture_with_status("working", 10);
        let mut added = changed["result"]["snapshot"]["panes"][0].clone();
        added["pane_id"] = json!("w1:p2");
        added["terminal_id"] = json!("second-terminal");
        changed["result"]["snapshot"]["panes"]
            .as_array_mut()
            .unwrap()
            .push(added);
        accept_request(&listener, "session.snapshot", changed.clone()).await;
        accept_request(&listener, "ping", fixture("0.9.0/pong.json")).await;
        accept_request(&listener, "session.snapshot", changed.clone()).await;
        let (mut subscription, _) = listener.accept().await.unwrap();
        let request = read_request(&mut subscription).await;
        assert_eq!(request["method"], "events.subscribe");
        assert!(
            request["params"]["subscriptions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|spec| spec["type"] == "pane.agent_status_changed"
                    && spec["pane_id"] == "w1:p2")
        );
        write_lines(
            &mut subscription,
            &[json!({"id": request["id"], "result": {"type": "subscription_started"}})],
        )
        .await;
        accept_request(&listener, "session.snapshot", changed).await;
        tokio::time::sleep(Duration::from_secs(1)).await;
    });
    let (tx, mut rx) = mpsc::channel(4);
    let (stop, shutdown) = watch::channel(false);
    let supervisor = tokio::spawn(test_supervisor(path).run(tx, shutdown));
    assert_eq!(next_update(&mut rx).await, ConnectionUpdate::Resyncing);
    assert!(
        matches!(next_update(&mut rx).await, ConnectionUpdate::Connected(snapshot) if snapshot.panes.len() == 2)
    );
    stop.send(true).unwrap();
    supervisor.await.unwrap();
    server.abort();
}

#[tokio::test]
async fn bootstrap_rejects_an_unsupported_post_subscription_snapshot() {
    for protocol in [20, 21, 23] {
        let (_directory, path, listener) = listener();
        let server = tokio::spawn(async move {
            let _subscription =
                discover_and_subscribe(&listener, capture_with_status("working", 9)).await;
            let mut unsupported = capture_with_status("working", 10);
            unsupported["result"]["snapshot"]["protocol"] = json!(protocol);
            accept_request(&listener, "session.snapshot", unsupported).await;
        });
        let (tx, mut rx) = mpsc::channel(4);
        let (_stop, shutdown) = watch::channel(false);
        let supervisor = tokio::spawn(test_supervisor(path).run(tx, shutdown));
        assert_eq!(
            next_update(&mut rx).await,
            ConnectionUpdate::Incompatible {
                expected: 22,
                actual: protocol
            }
        );
        supervisor.await.unwrap();
        server.await.unwrap();
    }
}

#[tokio::test]
async fn queued_unversioned_completion_is_reconciled_before_reaching_the_model() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(async move {
        let mut subscription =
            discover_and_subscribe(&listener, capture_with_status("working", 10)).await;
        write_lines(&mut subscription, &[json!({"event": "pane.agent_status_changed", "data": {"pane_id": "w1:p1", "agent_status": "done"}})]).await;
        let current = capture_with_status("working", 10);
        accept_request(&listener, "session.snapshot", current.clone()).await;
        accept_pane_get(&listener, current["result"]["snapshot"]["panes"][0].clone()).await;
        // A subsequent real blocked state must still arrive with its current revision.
        write_lines(&mut subscription, &[json!({"event": "pane.agent_status_changed", "data": {"pane_id": "w1:p1", "agent_status": "blocked"}})]).await;
        accept_pane_get(
            &listener,
            capture_with_status("blocked", 11)["result"]["snapshot"]["panes"][0].clone(),
        )
        .await;
        tokio::time::sleep(Duration::from_secs(1)).await;
    });
    let (tx, mut rx) = mpsc::channel(8);
    let (stop, shutdown) = watch::channel(false);
    let supervisor = tokio::spawn(test_supervisor(path).run(tx, shutdown));
    let mut model = questmancer::app::Model::new(questmancer::app::View::Guild);
    let connected = next_update(&mut rx).await;
    assert!(matches!(connected, ConnectionUpdate::Connected(_)));
    let _ = questmancer::runtime_loop::apply_connection_update(
        &mut model,
        connected,
        questmancer::domain::Timestamp::from_millis(1000),
    );
    let event = next_update(&mut rx).await;
    assert!(
        matches!(&event, ConnectionUpdate::Event(event) if event.data["agent_status"] == "working" && event.data["revision"] == 10),
        "stale completion must not be projected: {event:?}"
    );
    let _ = questmancer::runtime_loop::apply_connection_update(
        &mut model,
        event,
        questmancer::domain::Timestamp::from_millis(2000),
    );
    assert_eq!(
        model.domain().agents.values().next().unwrap().presence,
        questmancer::domain::Presence::Working
    );
    let event = next_update(&mut rx).await;
    assert!(
        matches!(&event, ConnectionUpdate::Event(event) if event.data["agent_status"] == "blocked" && event.data["revision"] == 11)
    );
    let _ = questmancer::runtime_loop::apply_connection_update(
        &mut model,
        event,
        questmancer::domain::Timestamp::from_millis(3000),
    );
    assert_eq!(
        model.domain().agents.values().next().unwrap().presence,
        questmancer::domain::Presence::Blocked
    );
    stop.send(true).unwrap();
    supervisor.await.unwrap();
    server.abort();
}

#[tokio::test]
async fn a_failed_post_subscription_baseline_never_publishes_discovery() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(async move {
        let _subscription =
            discover_and_subscribe(&listener, capture_with_status("working", 10)).await;
        accept_request(&listener, "session.snapshot", fixture("error.json")).await;
    });
    let (tx, mut rx) = mpsc::channel(4);
    let (stop, shutdown) = watch::channel(false);
    let supervisor = tokio::spawn(test_supervisor(path).run(tx, shutdown));
    assert!(matches!(
        next_update(&mut rx).await,
        ConnectionUpdate::Disconnected(_)
    ));
    assert!(matches!(
        next_update(&mut rx).await,
        ConnectionUpdate::Reconnecting { attempt: 1, .. }
    ));
    stop.send(true).unwrap();
    supervisor.await.unwrap();
    server.await.unwrap();
}

#[tokio::test]
async fn failed_status_reconciliation_reconnects_without_forwarding_stale_text() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(async move {
        let mut subscription =
            discover_and_subscribe(&listener, capture_with_status("working", 10)).await;
        accept_request(
            &listener,
            "session.snapshot",
            capture_with_status("working", 10),
        )
        .await;
        write_lines(&mut subscription, &[json!({"event": "pane.agent_status_changed", "data": {"pane_id": "w1:p1", "agent_status": "done"}})]).await;
        accept_request(&listener, "pane.get", fixture("error.json")).await;
    });
    let (tx, mut rx) = mpsc::channel(4);
    let (stop, shutdown) = watch::channel(false);
    let supervisor = tokio::spawn(test_supervisor(path).run(tx, shutdown));
    assert!(matches!(
        next_update(&mut rx).await,
        ConnectionUpdate::Connected(_)
    ));
    assert!(matches!(
        next_update(&mut rx).await,
        ConnectionUpdate::Disconnected(_)
    ));
    assert!(matches!(
        next_update(&mut rx).await,
        ConnectionUpdate::Reconnecting { attempt: 1, .. }
    ));
    stop.send(true).unwrap();
    supervisor.await.unwrap();
    server.await.unwrap();
}

#[tokio::test]
async fn shutdown_cancels_a_pending_status_read() {
    let (_directory, path, listener) = listener();
    let (reading, read_started) = oneshot::channel();
    let server = tokio::spawn(async move {
        let mut subscription =
            discover_and_subscribe(&listener, capture_with_status("working", 10)).await;
        accept_request(
            &listener,
            "session.snapshot",
            capture_with_status("working", 10),
        )
        .await;
        write_lines(&mut subscription, &[json!({"event": "pane.agent_status_changed", "data": {"pane_id": "w1:p1", "agent_status": "blocked"}})]).await;
        let (mut request_stream, _) = listener.accept().await.unwrap();
        let request = read_request(&mut request_stream).await;
        assert_eq!(request["method"], "pane.get");
        reading.send(()).unwrap();
        let mut remaining = Vec::new();
        assert_eq!(
            timeout(
                Duration::from_secs(1),
                request_stream.read_to_end(&mut remaining)
            )
            .await
            .unwrap()
            .unwrap(),
            0
        );
    });
    let (tx, mut rx) = mpsc::channel(4);
    let (stop, shutdown) = watch::channel(false);
    let supervisor = tokio::spawn(test_supervisor(path).run(tx, shutdown));
    assert!(matches!(
        next_update(&mut rx).await,
        ConnectionUpdate::Connected(_)
    ));
    timeout(Duration::from_secs(1), read_started)
        .await
        .unwrap()
        .unwrap();
    stop.send(true).unwrap();
    timeout(Duration::from_secs(1), supervisor)
        .await
        .unwrap()
        .unwrap();
    assert!(rx.recv().await.is_none());
    server.await.unwrap();
}

#[tokio::test]
async fn rejects_an_incompatible_protocol_without_subscribing() {
    for actual in [20, 21, 23] {
        let (_directory, path, listener) = listener();
        let server = tokio::spawn(async move {
            let mut pong = fixture("pong.json");
            pong["result"]["protocol"] = json!(actual);
            accept_request(&listener, "ping", pong).await;
        });
        let (update_tx, mut update_rx) = mpsc::channel(4);
        let (_shutdown_tx, shutdown_rx) = watch::channel(false);

        test_supervisor(path).run(update_tx, shutdown_rx).await;

        assert_eq!(
            next_update(&mut update_rx).await,
            ConnectionUpdate::Incompatible {
                expected: 22,
                actual
            }
        );
        assert!(update_rx.recv().await.is_none());
        server.await.unwrap();
    }
}

#[tokio::test]
async fn rejects_a_changed_protocol_in_the_initial_snapshot_without_subscribing() {
    for actual in [20, 21, 23] {
        let (_directory, path, listener) = listener();
        let server = tokio::spawn(async move {
            accept_request(&listener, "ping", fixture("pong.json")).await;
            let mut snapshot = fixture("session_snapshot.json");
            snapshot["result"]["snapshot"]["protocol"] = json!(actual);
            accept_request(&listener, "session.snapshot", snapshot).await;
        });
        let (update_tx, mut update_rx) = mpsc::channel(4);
        let (_shutdown_tx, shutdown_rx) = watch::channel(false);

        test_supervisor(path).run(update_tx, shutdown_rx).await;

        assert_eq!(
            next_update(&mut update_rx).await,
            ConnectionUpdate::Incompatible {
                expected: 22,
                actual
            }
        );
        assert!(update_rx.recv().await.is_none());
        server.await.unwrap();
    }
}

#[tokio::test]
async fn a_topology_refresh_cannot_switch_to_an_unsupported_protocol() {
    for actual in [20, 21, 23] {
        let (_directory, path, listener) = listener();
        let server = tokio::spawn(async move {
            accept_request(&listener, "ping", fixture("pong.json")).await;
            accept_request(
                &listener,
                "session.snapshot",
                fixture("session_snapshot.json"),
            )
            .await;
            let mut subscription =
                accept_baseline_subscription(&listener, fixture("session_snapshot.json")).await;
            write_lines(
                &mut subscription,
                &[json!({"event": "pane_created", "data": {"pane_id": "w1:p2"}})],
            )
            .await;
            let mut snapshot = fixture("session_snapshot.json");
            snapshot["result"]["snapshot"]["protocol"] = json!(actual);
            accept_request(&listener, "session.snapshot", snapshot).await;
        });
        let (update_tx, mut update_rx) = mpsc::channel(4);
        let (_shutdown_tx, shutdown_rx) = watch::channel(false);

        test_supervisor(path).run(update_tx, shutdown_rx).await;

        assert!(matches!(
            next_update(&mut update_rx).await,
            ConnectionUpdate::Connected(_)
        ));
        assert!(matches!(
            next_update(&mut update_rx).await,
            ConnectionUpdate::Event(_)
        ));
        assert_eq!(
            next_update(&mut update_rx).await,
            ConnectionUpdate::Incompatible {
                expected: 22,
                actual
            }
        );
        assert!(update_rx.recv().await.is_none());
        server.await.unwrap();
    }
}

#[tokio::test]
async fn reconnects_with_a_fresh_snapshot_after_disconnect() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(async move {
        accept_request(&listener, "ping", fixture("pong.json")).await;
        accept_request(
            &listener,
            "session.snapshot",
            fixture("session_snapshot.json"),
        )
        .await;
        drop(accept_baseline_subscription(&listener, fixture("session_snapshot.json")).await);

        accept_request(&listener, "ping", fixture("pong.json")).await;
        let mut second_snapshot = fixture("session_snapshot.json");
        second_snapshot["result"]["snapshot"]["workspaces"][0]["label"] =
            json!("resnapshotted-site");
        accept_request(&listener, "session.snapshot", second_snapshot.clone()).await;
        let _subscription = accept_baseline_subscription(&listener, second_snapshot).await;
        tokio::time::sleep(Duration::from_secs(1)).await;
    });
    let (update_tx, mut update_rx) = mpsc::channel(16);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let supervisor = tokio::spawn(test_supervisor(path).run(update_tx, shutdown_rx));

    assert!(matches!(
        next_update(&mut update_rx).await,
        ConnectionUpdate::Connected(_)
    ));
    assert!(matches!(
        next_update(&mut update_rx).await,
        ConnectionUpdate::Disconnected(_)
    ));
    assert!(matches!(
        next_update(&mut update_rx).await,
        ConnectionUpdate::Reconnecting {
            attempt: 1,
            delay
        } if delay == Duration::from_millis(1)
    ));
    assert!(matches!(
        next_update(&mut update_rx).await,
        ConnectionUpdate::Connected(snapshot)
            if snapshot.workspaces[0].label == "resnapshotted-site"
    ));

    shutdown_tx.send(true).unwrap();
    supervisor.await.unwrap();
    server.abort();
}

#[tokio::test]
async fn topology_event_resnapshots_without_reconnect_backoff() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(async move {
        accept_request(&listener, "ping", fixture("pong.json")).await;
        accept_request(
            &listener,
            "session.snapshot",
            fixture("session_snapshot.json"),
        )
        .await;
        let mut first_subscription =
            accept_baseline_subscription(&listener, fixture("session_snapshot.json")).await;
        write_lines(
            &mut first_subscription,
            &[json!({"event": "pane_created", "data": {"type": "pane_created", "pane": {"pane_id": "w1:p2"}}})],
        )
        .await;

        let mut changed_snapshot = fixture("session_snapshot.json");
        let mut added_pane = changed_snapshot["result"]["snapshot"]["panes"][0].clone();
        added_pane["pane_id"] = json!("w1:p2");
        added_pane["terminal_id"] = json!("terminal-2");
        changed_snapshot["result"]["snapshot"]["panes"]
            .as_array_mut()
            .unwrap()
            .push(added_pane);
        accept_request(&listener, "session.snapshot", changed_snapshot.clone()).await;

        accept_request(&listener, "ping", fixture("pong.json")).await;
        let mut second_snapshot = changed_snapshot;
        second_snapshot["result"]["snapshot"]["panes"][0]["revision"] = json!(99);
        accept_request(&listener, "session.snapshot", second_snapshot.clone()).await;
        let _second_subscription = accept_baseline_subscription(&listener, second_snapshot).await;
        tokio::time::sleep(Duration::from_secs(1)).await;
    });
    let (update_tx, mut update_rx) = mpsc::channel(16);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let supervisor = tokio::spawn(test_supervisor(path).run(update_tx, shutdown_rx));

    assert!(matches!(
        next_update(&mut update_rx).await,
        ConnectionUpdate::Connected(_)
    ));
    assert!(matches!(
        next_update(&mut update_rx).await,
        ConnectionUpdate::Event(event) if event.event == "pane_created"
    ));
    assert!(matches!(
        next_update(&mut update_rx).await,
        ConnectionUpdate::Resyncing
    ));
    assert!(matches!(
        next_update(&mut update_rx).await,
        ConnectionUpdate::Connected(snapshot) if snapshot.panes[0].revision == 99
    ));

    shutdown_tx.send(true).unwrap();
    supervisor.await.unwrap();
    server.abort();
}

#[tokio::test]
async fn replayed_topology_event_with_unchanged_panes_keeps_the_subscription_open() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(async move {
        accept_request(&listener, "ping", fixture("pong.json")).await;
        accept_request(
            &listener,
            "session.snapshot",
            fixture("session_snapshot.json"),
        )
        .await;
        let mut subscription =
            accept_baseline_subscription(&listener, fixture("session_snapshot.json")).await;
        write_lines(
            &mut subscription,
            &[json!({"event": "pane_created", "data": {"type": "pane_created", "pane": {"pane_id": "w1:p1"}}})],
        )
        .await;

        accept_request(
            &listener,
            "session.snapshot",
            fixture("session_snapshot.json"),
        )
        .await;
        write_lines(
            &mut subscription,
            &[json!({"event": "workspace_focused", "data": {"type": "workspace_focused", "workspace_id": "w1"}})],
        )
        .await;

        assert!(
            timeout(Duration::from_millis(50), listener.accept())
                .await
                .is_err(),
            "unchanged pane subscriptions must not reconnect or resubscribe"
        );
        tokio::time::sleep(Duration::from_secs(1)).await;
    });
    let (update_tx, mut update_rx) = mpsc::channel(16);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let supervisor = tokio::spawn(test_supervisor(path).run(update_tx, shutdown_rx));

    assert!(matches!(
        next_update(&mut update_rx).await,
        ConnectionUpdate::Connected(_)
    ));
    assert!(matches!(
        next_update(&mut update_rx).await,
        ConnectionUpdate::Event(event) if event.event == "pane_created"
    ));
    assert!(matches!(
        next_update(&mut update_rx).await,
        ConnectionUpdate::Event(event) if event.event == "workspace_focused"
    ));

    shutdown_tx.send(true).unwrap();
    supervisor.await.unwrap();
    server.await.unwrap();
}

#[tokio::test]
async fn topology_snapshot_refresh_failure_disconnects_the_subscription() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(async move {
        accept_request(&listener, "ping", fixture("pong.json")).await;
        accept_request(
            &listener,
            "session.snapshot",
            fixture("session_snapshot.json"),
        )
        .await;
        let mut subscription =
            accept_baseline_subscription(&listener, fixture("session_snapshot.json")).await;
        write_lines(
            &mut subscription,
            &[json!({"event": "pane_created", "data": {"type": "pane_created", "pane": {"pane_id": "w1:p2"}}})],
        )
        .await;
        accept_request(&listener, "session.snapshot", fixture("error.json")).await;
    });
    let (update_tx, mut update_rx) = mpsc::channel(16);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let supervisor = tokio::spawn(test_supervisor(path).run(update_tx, shutdown_rx));

    assert!(matches!(
        next_update(&mut update_rx).await,
        ConnectionUpdate::Connected(_)
    ));
    assert!(matches!(
        next_update(&mut update_rx).await,
        ConnectionUpdate::Event(event) if event.event == "pane_created"
    ));
    assert!(matches!(
        next_update(&mut update_rx).await,
        ConnectionUpdate::Disconnected(message) if message.contains("pane w9:p9 does not exist")
    ));
    assert!(matches!(
        next_update(&mut update_rx).await,
        ConnectionUpdate::Reconnecting {
            attempt: 1,
            delay
        } if delay == Duration::from_millis(1)
    ));

    shutdown_tx.send(true).unwrap();
    supervisor.await.unwrap();
    server.await.unwrap();
}
