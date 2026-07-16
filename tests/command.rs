use std::path::PathBuf;

use questmancer::{
    command::{AgentCommand, CommandExecutor, CommandResult},
    domain::PaneId,
    herdr::client::HerdrClient,
};
use serde_json::{Value, json};
use tempfile::TempDir;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
};

fn listener() -> (TempDir, PathBuf, UnixListener) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("herdr.sock");
    let listener = UnixListener::bind(&path).unwrap();
    (directory, path, listener)
}

async fn request(stream: &mut UnixStream) -> Value {
    let mut line = String::new();
    BufReader::new(stream).read_line(&mut line).await.unwrap();
    serde_json::from_str(&line).unwrap()
}

async fn respond(stream: &mut UnixStream, request: &Value, result: Value) {
    let value = json!({"id": request["id"], "result": result});
    stream
        .write_all(serde_json::to_string(&value).unwrap().as_bytes())
        .await
        .unwrap();
    stream.write_all(b"\n").await.unwrap();
}

#[tokio::test]
async fn output_load_returns_a_ui_ready_preview() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let request = request(&mut stream).await;
        respond(
            &mut stream,
            &request,
            json!({"type": "pane_read", "read": {
                "pane_id": "w1:p1", "workspace_id": "w1", "tab_id": "w1:t1",
                "source": "recent_unwrapped", "format": "text", "text": "done",
                "revision": 12, "truncated": false
            }}),
        )
        .await;
    });
    let executor = CommandExecutor::new(HerdrClient::new(path), None);

    let result = executor
        .execute(AgentCommand::LoadOutput {
            pane_id: PaneId::new("w1:p1"),
            lines: 80,
        })
        .await;

    assert!(matches!(
        result,
        CommandResult::OutputLoaded { pane_id, revision: 12, text, .. }
            if pane_id == PaneId::new("w1:p1") && text == "done"
    ));
    server.await.unwrap();
}

#[tokio::test]
async fn server_failure_becomes_a_non_blocking_result() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let request = request(&mut stream).await;
        let value = json!({"id": request["id"], "error": {
            "code": "pane_not_found", "message": "pane vanished"
        }});
        stream
            .write_all(serde_json::to_string(&value).unwrap().as_bytes())
            .await
            .unwrap();
        stream.write_all(b"\n").await.unwrap();
    });
    let executor = CommandExecutor::new(HerdrClient::new(path), None);

    let result = executor
        .execute(AgentCommand::FocusPane(PaneId::new("w1:p9")))
        .await;

    assert!(matches!(
        result,
        CommandResult::Failed { operation, message }
            if operation == "focus pane" && message.contains("pane vanished")
    ));
    server.await.unwrap();
}

#[tokio::test]
async fn reviewr_discovery_checks_the_exact_qualified_action() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let request = request(&mut stream).await;
        respond(
            &mut stream,
            &request,
            json!({"type": "plugin_action_list", "actions": [
                {"plugin_id": "acme.diff", "action_id": "inspect", "title": "inspect", "command": []}
            ]}),
        )
        .await;
    });
    let executor = CommandExecutor::new(HerdrClient::new(path), None);

    let result = executor
        .execute(AgentCommand::DiscoverReviewr {
            qualified_id: "acme.diff.inspect".to_owned(),
        })
        .await;

    assert_eq!(result, CommandResult::ReviewrAvailable(true));
    server.await.unwrap();
}

#[tokio::test]
async fn opening_reviewr_focuses_the_agent_before_invocation() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(async move {
        let (mut focus_stream, _) = listener.accept().await.unwrap();
        let focus = request(&mut focus_stream).await;
        assert_eq!(focus["method"], "pane.focus");
        respond(
            &mut focus_stream,
            &focus,
            json!({"type": "pane_info", "pane": {
                "pane_id": "w1:p1", "terminal_id": "t1", "workspace_id": "w1",
                "tab_id": "w1:t1", "focused": true, "agent_status": "done",
                "revision": 7, "state_labels": {}
            }}),
        )
        .await;

        let (mut invoke_stream, _) = listener.accept().await.unwrap();
        let invoke = request(&mut invoke_stream).await;
        assert_eq!(invoke["method"], "plugin.action.invoke");
        assert_eq!(invoke["params"]["plugin_id"], "acme.diff");
        assert_eq!(invoke["params"]["action_id"], "inspect");
        respond(
            &mut invoke_stream,
            &invoke,
            json!({
                "type": "plugin_action_invoked",
                "action": {"plugin_id": "acme.diff", "action_id": "inspect", "title": "inspect", "command": []},
                "context": {"focused_pane_id": "w1:p1"}, "log": {"log_id": "l1"}
            }),
        )
        .await;
    });
    let executor = CommandExecutor::new(HerdrClient::new(path), None);

    let result = executor
        .execute(AgentCommand::InspectSpoils {
            pane_id: PaneId::new("w1:p1"),
            qualified_id: "acme.diff.inspect".to_owned(),
        })
        .await;

    assert_eq!(result, CommandResult::SpoilsOpened);
    server.await.unwrap();
}

#[tokio::test]
async fn non_splittable_reviewr_action_is_unavailable_and_invocation_fails_non_fatally() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("missing.sock");
    let executor = CommandExecutor::new(HerdrClient::new(path), None);

    let discovery = executor
        .execute(AgentCommand::DiscoverReviewr {
            qualified_id: "inspect".to_owned(),
        })
        .await;
    let invocation = executor
        .execute(AgentCommand::InspectSpoils {
            pane_id: PaneId::new("w1:p1"),
            qualified_id: "inspect".to_owned(),
        })
        .await;

    assert_eq!(discovery, CommandResult::ReviewrAvailable(false));
    assert!(matches!(
        invocation,
        CommandResult::Failed { operation: "open reviewr", message }
            if message.contains("qualified")
    ));
}

#[tokio::test]
async fn reply_sends_the_composed_text_to_the_selected_pane() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let request = request(&mut stream).await;
        assert_eq!(request["method"], "pane.send_text");
        respond(&mut stream, &request, json!({"type": "ok"})).await;
        request
    });
    let executor = CommandExecutor::new(HerdrClient::new(path), None);

    let result = executor
        .execute(AgentCommand::SendCounsel {
            pane_id: PaneId::new("w1:p1"),
            text: "use jsonb".into(),
        })
        .await;
    let request = server.await.unwrap();

    assert_eq!(request["params"]["text"], "use jsonb");
    assert_eq!(result, CommandResult::CounselSent(PaneId::new("w1:p1")));
}

#[tokio::test]
async fn snapshot_refresh_returns_a_domain_ready_snapshot() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let request = request(&mut stream).await;
        assert_eq!(request["method"], "session.snapshot");
        let fixture: Value =
            serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
        respond(&mut stream, &request, fixture["result"].clone()).await;
    });
    let executor = CommandExecutor::new(HerdrClient::new(path), None);

    let result = executor.execute(AgentCommand::RefreshSnapshot).await;

    assert!(matches!(
        result,
        CommandResult::SnapshotLoaded(snapshot) if snapshot.protocol == 16
    ));
    server.await.unwrap();
}

#[tokio::test]
async fn managed_pane_effects_are_refused_before_socket_io() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("missing.sock");
    let managed = PaneId::new("w2:p3");
    let executor = CommandExecutor::new(HerdrClient::new(path), Some(managed.clone()));

    let commands = [
        (AgentCommand::FocusPane(managed.clone()), "focus pane"),
        (
            AgentCommand::SendCounsel {
                pane_id: managed.clone(),
                text: "do not send".to_owned(),
            },
            "send counsel",
        ),
        (
            AgentCommand::LoadOutput {
                pane_id: managed.clone(),
                lines: 80,
            },
            "load output",
        ),
        (
            AgentCommand::InspectSpoils {
                pane_id: managed,
                qualified_id: "acme.diff.inspect".to_owned(),
            },
            "open reviewr",
        ),
    ];

    for (command, expected_operation) in commands {
        let result = executor.execute(command).await;
        assert!(matches!(
            result,
            CommandResult::Failed { operation, message }
                if operation == expected_operation
                    && message == "refused operation on the Questmancer guild pane"
        ));
    }
}
