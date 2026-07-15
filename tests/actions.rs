use std::path::PathBuf;

use questmancer::herdr::client::HerdrClient;
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

async fn read_request(stream: &mut UnixStream) -> Value {
    let mut line = String::new();
    BufReader::new(stream).read_line(&mut line).await.unwrap();
    serde_json::from_str(&line).unwrap()
}

async fn write_response(stream: &mut UnixStream, value: Value) {
    stream
        .write_all(serde_json::to_string(&value).unwrap().as_bytes())
        .await
        .unwrap();
    stream.write_all(b"\n").await.unwrap();
}

async fn serve_once(listener: UnixListener, expected_method: &'static str, result: Value) -> Value {
    let (mut stream, _) = listener.accept().await.unwrap();
    let request = read_request(&mut stream).await;
    assert_eq!(request["method"], expected_method);
    write_response(&mut stream, json!({"id": request["id"], "result": result})).await;
    request
}

#[tokio::test]
async fn focuses_an_exact_pane() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(serve_once(
        listener,
        "pane.focus",
        json!({
            "type": "pane_info",
            "pane": {
                "pane_id": "w1:p1", "terminal_id": "t1", "workspace_id": "w1",
                "tab_id": "w1:t1", "focused": true, "agent_status": "blocked",
                "revision": 7, "state_labels": {}
            }
        }),
    ));

    let pane = HerdrClient::new(path).focus_pane("w1:p1").await.unwrap();
    let request = server.await.unwrap();

    assert_eq!(request["params"], json!({"pane_id": "w1:p1"}));
    assert!(pane.focused);
}

#[tokio::test]
async fn sends_reply_text_without_mutating_it() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(serve_once(
        listener,
        "pane.send_text",
        json!({"type": "ok"}),
    ));

    HerdrClient::new(path)
        .send_text("w1:p1", "ship it\n")
        .await
        .unwrap();
    let request = server.await.unwrap();

    assert_eq!(
        request["params"],
        json!({"pane_id": "w1:p1", "text": "ship it\n"})
    );
}

#[tokio::test]
async fn reads_recent_unwrapped_text_with_a_line_cap() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(serve_once(
        listener,
        "pane.read",
        json!({
            "type": "pane_read",
            "read": {
                "pane_id": "w1:p1", "workspace_id": "w1", "tab_id": "w1:t1",
                "source": "recent_unwrapped", "format": "text", "text": "hello\nworld",
                "revision": 9, "truncated": false
            }
        }),
    ));

    let read = HerdrClient::new(path)
        .read_recent_unwrapped("w1:p1", 80)
        .await
        .unwrap();
    let request = server.await.unwrap();

    assert_eq!(request["params"]["source"], "recent_unwrapped");
    assert_eq!(request["params"]["format"], "text");
    assert_eq!(request["params"]["lines"], 80);
    assert_eq!(request["params"]["strip_ansi"], true);
    assert_eq!(read.text, "hello\nworld");
}

#[tokio::test]
async fn lists_actions_with_qualified_ids() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(serve_once(
        listener,
        "plugin.action.list",
        json!({
            "type": "plugin_action_list",
            "actions": [{
                "plugin_id": "persiyanov.reviewr", "action_id": "open",
                "title": "reviewr: open", "contexts": ["pane"], "command": ["reviewr"]
            }]
        }),
    ));

    let actions = HerdrClient::new(path).list_plugin_actions().await.unwrap();
    let request = server.await.unwrap();

    assert_eq!(request["params"], json!({}));
    assert_eq!(actions[0].qualified_id(), "persiyanov.reviewr.open");
}

#[tokio::test]
async fn invokes_a_plugin_action_with_focused_pane_context() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(serve_once(
        listener,
        "plugin.action.invoke",
        json!({
            "type": "plugin_action_invoked",
            "action": {"plugin_id": "persiyanov.reviewr", "action_id": "open", "title": "open", "command": []},
            "context": {"focused_pane_id": "w1:p1"},
            "log": {"log_id": "l1"}
        }),
    ));

    HerdrClient::new(path)
        .invoke_plugin_action("persiyanov.reviewr", "open", "w1:p1")
        .await
        .unwrap();
    let request = server.await.unwrap();

    assert_eq!(request["params"]["plugin_id"], "persiyanov.reviewr");
    assert_eq!(request["params"]["action_id"], "open");
    assert_eq!(request["params"]["context"]["focused_pane_id"], "w1:p1");
    assert_eq!(
        request["params"]["context"]["invocation_source"],
        "opsydyn.questmancer"
    );
}
