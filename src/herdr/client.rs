use std::{
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
        EmptyParams, ErrorResponse, OkResult, PaneInfo, PaneInfoResult, PaneReadParams,
        PaneReadResult, PaneReadResultEnvelope, PaneSendTextParams, PaneTarget, PluginActionInfo,
        PluginActionInvokeParams, PluginActionInvokedResult, PluginActionListParams,
        PluginActionListResult, PluginInvocationContext, Pong, ReadFormat, ReadSource, Request,
        SessionSnapshot, SessionSnapshotResult, SuccessResponse,
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
                        invocation_source: Some("opsydyn.webmaster".to_owned()),
                        ..PluginInvocationContext::default()
                    }),
                },
                "plugin_action_invoked",
            )
            .await?;
        Ok(())
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
        let request_id = format!("webmaster-{}", self.next_id.fetch_add(1, Ordering::Relaxed));
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
        let actual_kind = success.result.get("type").and_then(Value::as_str);
        if actual_kind != Some(expected_kind) {
            return Err(ClientError::UnexpectedResult {
                expected: expected_kind,
                actual: actual_kind.unwrap_or("<missing>").to_owned(),
            });
        }
        serde_json::from_value(success.result).map_err(ClientError::Json)
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
