use std::{collections::BTreeSet, path::Path};

use serde::Serialize;
use serde_json::Value;
use tokio::{io::BufReader, net::UnixStream};

use super::{
    client::ClientError,
    framing::{read_json_line, read_optional_json_line, write_json_line},
    protocol::{ErrorResponse, SessionSnapshot, SuccessResponse, WireEvent},
};

const GLOBAL_SUBSCRIPTIONS: &[&str] = &[
    "workspace.created",
    "workspace.updated",
    "workspace.renamed",
    "workspace.moved",
    "workspace.closed",
    "workspace.focused",
    "worktree.created",
    "worktree.opened",
    "worktree.removed",
    "tab.created",
    "tab.closed",
    "tab.focused",
    "tab.renamed",
    "tab.moved",
    "pane.created",
    "pane.closed",
    "pane.focused",
    "pane.moved",
    "pane.exited",
    "pane.agent_detected",
    "layout.updated",
];

#[derive(Clone, Debug, Serialize)]
pub struct SubscriptionRequest {
    id: String,
    method: &'static str,
    params: EventsSubscribeParams,
}

impl SubscriptionRequest {
    #[must_use]
    pub fn for_snapshot(snapshot: &SessionSnapshot) -> Self {
        let mut subscriptions = GLOBAL_SUBSCRIPTIONS
            .iter()
            .map(|kind| SubscriptionSpec {
                kind: (*kind).to_owned(),
                pane_id: None,
            })
            .collect::<Vec<_>>();

        let pane_ids = snapshot
            .panes
            .iter()
            .map(|pane| pane.pane_id.as_str())
            .chain(snapshot.agents.iter().map(|agent| agent.pane_id.as_str()))
            .collect::<BTreeSet<_>>();
        subscriptions.extend(pane_ids.into_iter().map(|pane_id| SubscriptionSpec {
            kind: "pane.agent_status_changed".to_owned(),
            pane_id: Some(pane_id.to_owned()),
        }));

        Self {
            id: "webmaster-subscription".to_owned(),
            method: "events.subscribe",
            params: EventsSubscribeParams { subscriptions },
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct EventsSubscribeParams {
    subscriptions: Vec<SubscriptionSpec>,
}

#[derive(Clone, Debug, Serialize)]
struct SubscriptionSpec {
    #[serde(rename = "type")]
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pane_id: Option<String>,
}

#[derive(Debug)]
pub struct HerdrSubscription {
    reader: BufReader<UnixStream>,
}

impl HerdrSubscription {
    pub async fn connect(
        socket_path: impl AsRef<Path>,
        request: SubscriptionRequest,
    ) -> Result<Self, ClientError> {
        let mut stream = UnixStream::connect(socket_path).await?;
        write_json_line(&mut stream, &request).await?;
        let mut reader = BufReader::new(stream);
        let response: Value = read_json_line(&mut reader).await?;
        validate_ack(response, &request.id)?;
        Ok(Self { reader })
    }

    pub async fn next_event(&mut self) -> Result<Option<WireEvent>, ClientError> {
        read_optional_json_line(&mut self.reader)
            .await
            .map_err(ClientError::Framing)
    }
}

fn validate_ack(response: Value, request_id: &str) -> Result<(), ClientError> {
    if response.get("error").is_some() {
        let error: ErrorResponse = serde_json::from_value(response)?;
        ensure_response_id(request_id, &error.id)?;
        return Err(ClientError::Server {
            code: error.error.code,
            message: error.error.message,
        });
    }

    let success: SuccessResponse<Value> = serde_json::from_value(response)?;
    ensure_response_id(request_id, &success.id)?;
    let actual = success.result.get("type").and_then(Value::as_str);
    if actual != Some("subscription_started") {
        return Err(ClientError::UnexpectedResult {
            expected: "subscription_started",
            actual: actual.unwrap_or("<missing>").to_owned(),
        });
    }
    Ok(())
}

fn ensure_response_id(expected: &str, actual: &str) -> Result<(), ClientError> {
    if actual == expected {
        Ok(())
    } else {
        Err(ClientError::MismatchedId {
            expected: expected.to_owned(),
            actual: actual.to_owned(),
        })
    }
}
