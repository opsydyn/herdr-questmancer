use std::{path::PathBuf, time::Duration};

use herdr_webmaster::herdr::{
    client::ClientError,
    protocol::{SessionSnapshot, SessionSnapshotResult, SuccessResponse},
    subscription::{HerdrSubscription, SubscriptionRequest},
};
use serde_json::{Value, json};
use tempfile::TempDir;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
};

fn listener() -> (TempDir, PathBuf, UnixListener) {
    let directory = tempfile::tempdir().expect("temporary socket directory");
    let path = directory.path().join("herdr.sock");
    let listener = UnixListener::bind(&path).expect("bind fake Herdr socket");
    (directory, path, listener)
}

fn snapshot() -> SessionSnapshot {
    let response: SuccessResponse<SessionSnapshotResult> =
        serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
    response.result.snapshot
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

#[test]
fn request_contains_global_events_and_one_status_subscription_per_pane() {
    let mut snapshot = snapshot();
    snapshot.panes.push(snapshot.panes[0].clone());

    let value = serde_json::to_value(SubscriptionRequest::for_snapshot(&snapshot)).unwrap();
    let subscriptions = value["params"]["subscriptions"].as_array().unwrap();
    let types: Vec<_> = subscriptions
        .iter()
        .map(|entry| entry["type"].as_str().unwrap())
        .collect();

    assert_eq!(value["method"], "events.subscribe");
    assert!(types.starts_with(&[
        "workspace.created",
        "workspace.updated",
        "workspace.renamed",
        "workspace.moved",
        "workspace.closed",
        "workspace.focused",
    ]));
    assert!(types.contains(&"worktree.created"));
    assert!(types.contains(&"tab.moved"));
    assert!(types.contains(&"pane.exited"));
    assert!(types.contains(&"layout.updated"));
    let status_entries: Vec<_> = subscriptions
        .iter()
        .filter(|entry| entry["type"] == "pane.agent_status_changed")
        .collect();
    assert_eq!(status_entries.len(), snapshot.panes.len() - 1);
    assert_eq!(status_entries[0]["pane_id"], snapshot.panes[0].pane_id);
}

#[tokio::test]
async fn streams_coalesced_lifecycle_dotted_and_unknown_events() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let request = read_request(&mut stream).await;
        write_lines(
            &mut stream,
            &[
                json!({"id": request["id"], "result": {"type": "subscription_started"}}),
                json!({"event": "workspace_created", "data": {"type": "workspace_created"}}),
                json!({"event": "pane.agent_status_changed", "data": {"pane_id": "w1:p1", "agent_status": "blocked"}}),
                json!({"event": "future.event", "data": {"answer": 42}}),
            ],
        )
        .await;
        tokio::time::sleep(Duration::from_millis(10)).await;
    });

    let mut subscription =
        HerdrSubscription::connect(path, SubscriptionRequest::for_snapshot(&snapshot()))
            .await
            .unwrap();

    assert_eq!(
        subscription.next_event().await.unwrap().unwrap().event,
        "workspace_created"
    );
    assert_eq!(
        subscription.next_event().await.unwrap().unwrap().event,
        "pane.agent_status_changed"
    );
    assert_eq!(
        subscription.next_event().await.unwrap().unwrap().event,
        "future.event"
    );
    server.await.unwrap();
}

#[tokio::test]
async fn preserves_an_error_ack() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let request = read_request(&mut stream).await;
        write_lines(
            &mut stream,
            &[json!({"id": request["id"], "error": {"code": "invalid_subscription", "message": "bad event"}})],
        )
        .await;
    });

    let result =
        HerdrSubscription::connect(path, SubscriptionRequest::for_snapshot(&snapshot())).await;

    assert!(matches!(
        result,
        Err(ClientError::Server { code, message })
            if code == "invalid_subscription" && message == "bad event"
    ));
    server.await.unwrap();
}

#[tokio::test]
async fn rejects_an_unexpected_ack_kind() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let request = read_request(&mut stream).await;
        write_lines(
            &mut stream,
            &[json!({"id": request["id"], "result": {"type": "ok"}})],
        )
        .await;
    });

    let result =
        HerdrSubscription::connect(path, SubscriptionRequest::for_snapshot(&snapshot())).await;

    assert!(matches!(
        result,
        Err(ClientError::UnexpectedResult {
            expected: "subscription_started",
            ..
        })
    ));
    server.await.unwrap();
}

#[tokio::test]
async fn clean_disconnect_returns_none() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let request = read_request(&mut stream).await;
        write_lines(
            &mut stream,
            &[json!({"id": request["id"], "result": {"type": "subscription_started"}})],
        )
        .await;
    });
    let mut subscription =
        HerdrSubscription::connect(path, SubscriptionRequest::for_snapshot(&snapshot()))
            .await
            .unwrap();

    assert!(subscription.next_event().await.unwrap().is_none());
    server.await.unwrap();
}
