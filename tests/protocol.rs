use questmancer::herdr::protocol::{
    AgentStatus, EmptyParams, ErrorResponse, Pong, Request, SessionSnapshotResult, SuccessResponse,
    WireEvent,
};

#[test]
fn request_serializes_schema_shape() {
    let request = Request::new("req-1", "ping", EmptyParams {});
    let value = serde_json::to_value(request).expect("request JSON");

    assert_eq!(value["id"], "req-1");
    assert_eq!(value["method"], "ping");
    assert_eq!(value["params"], serde_json::json!({}));
}

#[test]
fn pong_tolerates_unknown_fields() {
    let response: SuccessResponse<Pong> =
        serde_json::from_str(include_str!("fixtures/herdr/pong.json")).expect("valid pong");

    assert_eq!(response.id, "req-1");
    assert_eq!(response.result.kind, "pong");
    assert_eq!(response.result.version, "0.7.3");
    assert_eq!(response.result.protocol, 19);
}

#[test]
fn snapshot_tolerates_unknown_fields() {
    let response: SuccessResponse<SessionSnapshotResult> =
        serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json"))
            .expect("valid snapshot");

    assert_eq!(response.result.kind, "session_snapshot");
    assert_eq!(response.result.snapshot.protocol, 19);
    assert_eq!(response.result.snapshot.workspaces[0].workspace_id, "w1");
    assert_eq!(response.result.snapshot.panes[0].revision, 7);
    assert_eq!(
        response.result.snapshot.agents[0].agent_status,
        AgentStatus::Blocked
    );
    assert_eq!(
        response.result.snapshot.agents[0]
            .agent_session
            .as_ref()
            .expect("agent session")
            .value,
        "session-123"
    );
}

#[test]
fn mixed_and_unknown_events_decode_without_loss() {
    let events = include_str!("fixtures/herdr/events.jsonl")
        .lines()
        .map(|line| serde_json::from_str::<WireEvent>(line).expect("valid event"))
        .collect::<Vec<_>>();

    assert_eq!(events[0].event, "workspace_created");
    assert_eq!(events[1].event, "pane.agent_status_changed");
    assert_eq!(events[1].data["agent_status"], "done");
    assert_eq!(events[2].event, "future.event");
    assert_eq!(events[2].data["payload"], true);
}

#[test]
fn error_response_retains_code_and_message() {
    let response: ErrorResponse =
        serde_json::from_str(include_str!("fixtures/herdr/error.json")).expect("valid error");

    assert_eq!(response.id, "req-3");
    assert_eq!(response.error.code, "pane_not_found");
    assert_eq!(response.error.message, "pane w9:p9 does not exist");
}
