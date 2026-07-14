use std::{path::PathBuf, time::Duration};

use herdr_webmaster::herdr::{
    client::HerdrClient,
    supervisor::{Backoff, ConnectionSupervisor, ConnectionUpdate},
};
use serde_json::{Value, json};
use tempfile::TempDir;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
    sync::{mpsc, watch},
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
        let mut subscription = accept_subscription(&listener).await;
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
        ConnectionUpdate::Connected(snapshot) if snapshot.protocol == 16
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
async fn rejects_an_incompatible_protocol_without_subscribing() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(async move {
        let mut pong = fixture("pong.json");
        pong["result"]["protocol"] = json!(15);
        accept_request(&listener, "ping", pong).await;
    });
    let (update_tx, mut update_rx) = mpsc::channel(4);
    let (_shutdown_tx, shutdown_rx) = watch::channel(false);

    test_supervisor(path).run(update_tx, shutdown_rx).await;

    assert!(matches!(
        next_update(&mut update_rx).await,
        ConnectionUpdate::Incompatible {
            expected: 16,
            actual: 15
        }
    ));
    assert!(update_rx.recv().await.is_none());
    server.await.unwrap();
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
        drop(accept_subscription(&listener).await);

        accept_request(&listener, "ping", fixture("pong.json")).await;
        let mut second_snapshot = fixture("session_snapshot.json");
        second_snapshot["result"]["snapshot"]["workspaces"][0]["label"] =
            json!("resnapshotted-site");
        accept_request(&listener, "session.snapshot", second_snapshot).await;
        let _subscription = accept_subscription(&listener).await;
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
        let mut first_subscription = accept_subscription(&listener).await;
        write_lines(
            &mut first_subscription,
            &[json!({"event": "pane_created", "data": {"type": "pane_created", "pane": {"pane_id": "w1:p2"}}})],
        )
        .await;

        accept_request(&listener, "ping", fixture("pong.json")).await;
        let mut second_snapshot = fixture("session_snapshot.json");
        second_snapshot["result"]["snapshot"]["panes"][0]["revision"] = json!(99);
        accept_request(&listener, "session.snapshot", second_snapshot).await;
        let _second_subscription = accept_subscription(&listener).await;
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
