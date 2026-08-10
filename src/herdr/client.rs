use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use thiserror::Error;
use tokio::{io::BufReader, net::UnixStream};

use super::{
    framing::{FramingError, read_json_line, write_json_line},
    protocol::{
        AgentViewBuiltinSortField, AgentViewClearParams, AgentViewSetParams, AgentViewSort,
        AgentViewSortField, AgentViewSortOrder, EmptyParams, ErrorResponse, OkResult, PaneInfo,
        PaneInfoResult, PaneReadParams, PaneReadResult, PaneReadResultEnvelope,
        PaneReportMetadataParams, PaneSendKeysParams, PaneSendTextParams, PaneTarget,
        PluginActionInfo, PluginActionInvokeParams, PluginActionInvokedResult,
        PluginActionListParams, PluginActionListResult, PluginInvocationContext, Pong, ReadFormat,
        ReadSource, Request, SessionSnapshot, SessionSnapshotResult, SuccessResponse,
        WorkspaceReportMetadataParams,
    },
};

#[derive(Clone, Debug)]
pub struct HerdrClient {
    socket_path: PathBuf,
    next_id: Arc<AtomicU64>,
}

impl HerdrClient {
    #[must_use]
    pub fn new(socket_path: impl Into<PathBuf>) -> Self {
        Self {
            socket_path: socket_path.into(),
            next_id: Arc::new(AtomicU64::new(1)),
        }
    }

    pub async fn ping(&self) -> Result<Pong, ClientError> {
        self.request("ping", EmptyParams {}, "pong").await
    }

    pub async fn snapshot(&self) -> Result<SessionSnapshot, ClientError> {
        let result: SessionSnapshotResult = self
            .request("session.snapshot", EmptyParams {}, "session_snapshot")
            .await?;
        Ok(result.snapshot)
    }

    pub async fn focus_pane(&self, pane_id: impl Into<String>) -> Result<PaneInfo, ClientError> {
        let result: PaneInfoResult = self
            .request(
                "pane.focus",
                PaneTarget {
                    pane_id: pane_id.into(),
                },
                "pane_info",
            )
            .await?;
        Ok(result.pane)
    }

    pub async fn send_text(
        &self,
        pane_id: impl Into<String>,
        text: impl Into<String>,
    ) -> Result<(), ClientError> {
        let _: OkResult = self
            .request(
                "pane.send_text",
                PaneSendTextParams {
                    pane_id: pane_id.into(),
                    text: text.into(),
                },
                "ok",
            )
            .await?;
        Ok(())
    }

    /// Presses keys in a pane. `pane.send_text` is literal text, so this is
    /// what actually submits a message an agent is meant to act on.
    pub async fn send_keys(
        &self,
        pane_id: impl Into<String>,
        keys: &[&str],
    ) -> Result<(), ClientError> {
        let _: OkResult = self
            .request(
                "pane.send_keys",
                PaneSendKeysParams {
                    pane_id: pane_id.into(),
                    keys: keys.iter().map(|key| (*key).to_owned()).collect(),
                },
                "ok",
            )
            .await?;
        Ok(())
    }

    pub async fn read_recent_unwrapped(
        &self,
        pane_id: impl Into<String>,
        lines: u32,
    ) -> Result<PaneReadResult, ClientError> {
        let result: PaneReadResultEnvelope = self
            .request(
                "pane.read",
                PaneReadParams {
                    pane_id: pane_id.into(),
                    source: ReadSource::RecentUnwrapped,
                    lines: Some(lines),
                    format: ReadFormat::Text,
                    strip_ansi: true,
                },
                "pane_read",
            )
            .await?;
        Ok(result.read)
    }

    pub async fn list_plugin_actions(&self) -> Result<Vec<PluginActionInfo>, ClientError> {
        let result: PluginActionListResult = self
            .request(
                "plugin.action.list",
                PluginActionListParams::default(),
                "plugin_action_list",
            )
            .await?;
        Ok(result.actions)
    }

    pub async fn invoke_plugin_action(
        &self,
        plugin_id: impl Into<String>,
        action_id: impl Into<String>,
        focused_pane_id: impl Into<String>,
    ) -> Result<(), ClientError> {
        let _: PluginActionInvokedResult = self
            .request(
                "plugin.action.invoke",
                PluginActionInvokeParams {
                    action_id: action_id.into(),
                    plugin_id: Some(plugin_id.into()),
                    context: Some(PluginInvocationContext {
                        focused_pane_id: Some(focused_pane_id.into()),
                        invocation_source: Some("opsydyn.questmancer".to_owned()),
                        ..PluginInvocationContext::default()
                    }),
                },
                "plugin_action_invoked",
            )
            .await?;
        Ok(())
    }

    pub async fn report_pane_tokens(
        &self,
        pane_id: impl Into<String>,
        source: impl Into<String>,
        tokens: BTreeMap<String, Option<String>>,
        seq: u64,
    ) -> Result<(), ClientError> {
        self.request_acknowledged(
            "pane.report_metadata",
            PaneReportMetadataParams {
                pane_id: pane_id.into(),
                source: source.into(),
                tokens,
                seq,
            },
        )
        .await
    }

    /// Asks Herdr to order its own agent list by Questmancer's urgency rank.
    ///
    /// Sort only, never filter. Hiding an agent from Herdr's sidebar because
    /// Questmancer judged it uninteresting would take authority the plugin
    /// does not have; reordering leaves every agent reachable.
    pub async fn set_agent_view(
        &self,
        source: impl Into<String>,
        label: impl Into<String>,
    ) -> Result<(), ClientError> {
        self.request_acknowledged(
            "agent.view.set",
            AgentViewSetParams {
                source: source.into(),
                label: Some(label.into()),
                sort: vec![
                    AgentViewSort {
                        field: AgentViewSortField::Token {
                            token: crate::sidebar::QUEST_RANK.to_owned(),
                        },
                        order: AgentViewSortOrder::Asc,
                    },
                    AgentViewSort {
                        field: AgentViewSortField::Builtin(AgentViewBuiltinSortField::Attention),
                        order: AgentViewSortOrder::Desc,
                    },
                ],
            },
        )
        .await
    }

    /// Hands the ordering back to Herdr. Called on shutdown so Questmancer
    /// never leaves the sidebar sorted by a plugin that is no longer running.
    pub async fn clear_agent_view(&self, source: impl Into<String>) -> Result<(), ClientError> {
        self.request_acknowledged(
            "agent.view.clear",
            AgentViewClearParams {
                source: Some(source.into()),
            },
        )
        .await
    }

    pub async fn report_workspace_tokens(
        &self,
        workspace_id: impl Into<String>,
        source: impl Into<String>,
        tokens: BTreeMap<String, Option<String>>,
        seq: u64,
    ) -> Result<(), ClientError> {
        self.request_acknowledged(
            "workspace.report_metadata",
            WorkspaceReportMetadataParams {
                workspace_id: workspace_id.into(),
                source: source.into(),
                tokens,
                seq,
            },
        )
        .await
    }

    async fn request<P, T>(
        &self,
        method: &str,
        params: P,
        expected_kind: &'static str,
    ) -> Result<T, ClientError>
    where
        P: Serialize,
        T: DeserializeOwned,
    {
        let response = self.request_value(method, params).await?;
        let actual_kind = response.get("type").and_then(Value::as_str);
        if actual_kind != Some(expected_kind) {
            return Err(ClientError::UnexpectedResult {
                expected: expected_kind,
                actual: actual_kind.unwrap_or("<missing>").to_owned(),
            });
        }
        serde_json::from_value(response).map_err(ClientError::Json)
    }

    async fn request_acknowledged<P>(&self, method: &str, params: P) -> Result<(), ClientError>
    where
        P: Serialize,
    {
        self.request_value(method, params).await.map(drop)
    }

    async fn request_value<P>(&self, method: &str, params: P) -> Result<Value, ClientError>
    where
        P: Serialize,
    {
        let request_id = format!(
            "questmancer-{}",
            self.next_id.fetch_add(1, Ordering::Relaxed)
        );
        let request = Request::new(request_id.clone(), method, params);
        let mut stream = UnixStream::connect(&self.socket_path).await?;
        write_json_line(&mut stream, &request).await?;

        let mut reader = BufReader::new(stream);
        let response: Value = read_json_line(&mut reader).await?;

        if response.get("error").is_some() {
            let error: ErrorResponse = serde_json::from_value(response)?;
            ensure_response_id(&request_id, &error.id)?;
            return Err(ClientError::Server {
                code: error.error.code,
                message: error.error.message,
            });
        }

        let success: SuccessResponse<Value> = serde_json::from_value(response)?;
        ensure_response_id(&request_id, &success.id)?;
        Ok(success.result)
    }

    #[must_use]
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }
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

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("failed to communicate with Herdr: {0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Framing(#[from] FramingError),
    #[error("invalid Herdr response: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Herdr returned {code}: {message}")]
    Server { code: String, message: String },
    #[error("response id {actual:?} did not match request id {expected:?}")]
    MismatchedId { expected: String, actual: String },
    #[error("expected result type {expected:?}, received {actual:?}")]
    UnexpectedResult {
        expected: &'static str,
        actual: String,
    },
}
