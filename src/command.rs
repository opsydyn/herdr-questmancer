use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering},
};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    app::{CounselRequest, OutputRequest, StatusRequest},
    domain::{CaptureIdentity, PaneId, Presence},
    herdr::{
        client::{ClientError, HerdrClient},
        protocol::SessionSnapshot,
    },
    pane_header::PaneHeaderProjection,
    sidebar::{SIDEBAR_SOURCE, SidebarProjection},
    snapshot_refresh::SnapshotRequest,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgentCommand {
    FocusPane(PaneId),
    FocusVerifiedPane {
        pane_id: PaneId,
        identity: CaptureIdentity,
    },
    SendCounsel {
        pane_id: PaneId,
        text: String,
        request: CounselRequest,
    },
    /// Retries only the Enter boundary after counsel text was already
    /// accepted by the pane.
    SubmitCounsel {
        pane_id: PaneId,
        request: CounselRequest,
    },
    LoadOutput {
        pane_id: PaneId,
        lines: u32,
        request: OutputRequest,
    },
    ExplainStatus(StatusRequest),
    RefreshSnapshot(SnapshotRequest),
    DiscoverReviewr {
        qualified_id: String,
    },
    InspectSpoils {
        pane_id: PaneId,
        qualified_id: String,
    },
    PublishMarginalia(SidebarProjection),
    PublishPaneHeaders(Vec<PaneHeaderProjection>),
    SetUrgencyView,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CommandResult {
    Focused(PaneId),
    CounselSent {
        pane_id: PaneId,
        request: CounselRequest,
    },
    /// Counsel has its own failure rather than the generic `Failed`, because
    /// the open parchment can only settle on a result it can prove is its own.
    CounselTextFailed {
        request: CounselRequest,
        message: String,
    },
    /// The text request was written but no trustworthy acknowledgement came
    /// back. Retrying the text could duplicate instructions.
    CounselTextUncertain {
        pane_id: PaneId,
        request: CounselRequest,
        message: String,
    },
    CounselSubmissionFailed {
        pane_id: PaneId,
        request: CounselRequest,
        message: String,
    },
    OutputLoaded {
        pane_id: PaneId,
        request: OutputRequest,
        revision: u64,
        text: String,
        truncated: bool,
    },
    OutputFailed {
        pane_id: PaneId,
        request: OutputRequest,
        message: String,
    },
    StatusExplained {
        serial: u64,
        result: Result<crate::herdr::protocol::AgentExplanation, String>,
    },
    ReviewrAvailable(bool),
    SpoilsOpened,
    UrgencyViewSet,
    UrgencyViewFailed {
        message: String,
    },
    MarginaliaPublished,
    MarginaliaFailed {
        message: String,
    },
    PaneHeadersPublished {
        available: bool,
    },
    PaneHeadersFailed {
        message: String,
    },
    SnapshotLoaded {
        request: SnapshotRequest,
        snapshot: Box<SessionSnapshot>,
    },
    SnapshotFailed {
        request: SnapshotRequest,
        message: String,
    },
    Failed {
        operation: &'static str,
        message: String,
    },
}

#[derive(Clone, Debug)]
pub struct CommandExecutor {
    client: HerdrClient,
    managed_pane_id: Option<PaneId>,
    next_sidebar_sequence: Arc<AtomicU64>,
    header_capability: Arc<AtomicU8>,
    headers_attempted: Arc<AtomicBool>,
}

impl CommandExecutor {
    #[must_use]
    pub fn new(client: HerdrClient, managed_pane_id: Option<PaneId>) -> Self {
        Self {
            client,
            managed_pane_id,
            next_sidebar_sequence: Arc::new(AtomicU64::new(sidebar_sequence_seed())),
            header_capability: Arc::new(AtomicU8::new(0)),
            headers_attempted: Arc::new(AtomicBool::new(false)),
        }
    }

    fn is_managed_pane(&self, pane_id: &PaneId) -> bool {
        self.managed_pane_id.as_ref() == Some(pane_id)
    }

    fn refused_managed_pane(operation: &'static str) -> CommandResult {
        CommandResult::Failed {
            operation,
            message: "refused operation on the Questmancer guild pane".to_owned(),
        }
    }

    /// Delivers counsel and submits it.
    ///
    /// `pane.send_text` is literal text and nothing else. Counsel used to stop
    /// there, which typed the message onto the adventurer's prompt and left it
    /// sitting unsent — an apparently successful send that delivered nothing.
    /// Enter is what submits it, so both calls have to succeed before this
    /// reports that counsel was issued.
    async fn send_counsel(
        &self,
        pane_id: PaneId,
        text: String,
        request: CounselRequest,
    ) -> CommandResult {
        if self.is_managed_pane(&pane_id) {
            return CommandResult::CounselTextFailed {
                request,
                message: "refused operation on the Questmancer guild pane".to_owned(),
            };
        }
        if let Err(error) = self.client.send_text(pane_id.as_str(), text).await {
            return match &error {
                // A protocol rejection is an acknowledgement that Herdr did
                // not accept the operation. Other failures may happen after
                // the full request was written, so their outcome is unknown.
                ClientError::Server { .. } => CommandResult::CounselTextFailed {
                    request,
                    message: error.to_string(),
                },
                _ => CommandResult::CounselTextUncertain {
                    pane_id,
                    request,
                    message: error.to_string(),
                },
            };
        }
        self.submit_counsel(pane_id, request).await
    }

    async fn submit_counsel(&self, pane_id: PaneId, request: CounselRequest) -> CommandResult {
        if self.is_managed_pane(&pane_id) {
            return CommandResult::CounselSubmissionFailed {
                pane_id,
                request,
                message: "refused operation on the Questmancer guild pane".to_owned(),
            };
        }
        match self.client.send_keys(pane_id.as_str(), &["enter"]).await {
            Ok(()) => CommandResult::CounselSent { pane_id, request },
            Err(error) => CommandResult::CounselSubmissionFailed {
                pane_id,
                request,
                message: error.to_string(),
            },
        }
    }

    async fn refresh_snapshot(&self, request: SnapshotRequest) -> CommandResult {
        match self.client.snapshot().await {
            Ok(snapshot) => CommandResult::SnapshotLoaded {
                request,
                snapshot: Box::new(snapshot),
            },
            Err(error) => CommandResult::SnapshotFailed {
                request,
                message: error.to_string(),
            },
        }
    }

    async fn explain_status(
        &self,
        request: &StatusRequest,
    ) -> Result<crate::herdr::protocol::AgentExplanation, String> {
        let pane = request.pane.as_str();
        if self.is_managed_pane(&request.pane) {
            return Err("Status is unavailable for the Questmancer pane.".into());
        }
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            let explanation = self
                .client
                .explain_agent(pane)
                .await
                .map_err(|error| error.to_string())?;
            let current = self
                .client
                .get_pane(pane)
                .await
                .map_err(|error| error.to_string())?;
            let identity =
                CaptureIdentity::from_parts(&current.terminal_id, current.agent_session.as_ref());
            if current.pane_id != pane
                || identity != request.identity
                || Presence::from(current.agent_status) != request.presence
            {
                return Err("Adventurer status changed. Reopen to refresh.".into());
            }
            Ok(explanation)
        })
        .await
        .unwrap_or_else(|_| Err("Herdr status explanation timed out.".into()))
    }

    async fn focus_pane(&self, pane_id: PaneId) -> CommandResult {
        if self.is_managed_pane(&pane_id) {
            return Self::refused_managed_pane("focus pane");
        }
        match self.client.focus_pane(pane_id.as_str()).await {
            Ok(_) => CommandResult::Focused(pane_id),
            Err(error) => failed("focus pane", error),
        }
    }

    async fn focus_verified_pane(
        &self,
        pane_id: PaneId,
        identity: &CaptureIdentity,
    ) -> CommandResult {
        if self.is_managed_pane(&pane_id) {
            return Self::refused_managed_pane("focus pane");
        }
        let operation = async {
            let current = self
                .client
                .get_pane(pane_id.as_str())
                .await
                .map_err(|e| e.to_string())?;
            let observed =
                CaptureIdentity::from_parts(&current.terminal_id, current.agent_session.as_ref());
            if current.pane_id != pane_id.as_str()
                || !observed.is_qualified()
                || &observed != identity
                || Presence::from(current.agent_status) == Presence::Exited
            {
                return Err(
                    "The adventurer session has changed. Link its current session to observe."
                        .into(),
                );
            }
            let focused = self
                .client
                .focus_pane(pane_id.as_str())
                .await
                .map_err(|e| e.to_string())?;
            let observed =
                CaptureIdentity::from_parts(&focused.terminal_id, focused.agent_session.as_ref());
            if !focused.focused || focused.pane_id != pane_id.as_str() || &observed != identity {
                return Err(
                    "Herdr could not confirm focus on the verified adventurer session.".into(),
                );
            }
            Ok::<(), String>(())
        };
        match tokio::time::timeout(std::time::Duration::from_secs(2), operation).await {
            Ok(Ok(())) => CommandResult::Focused(pane_id),
            Ok(Err(message)) => failed("focus pane", message),
            Err(_) => failed(
                "focus pane",
                "Verifying or focusing the adventurer timed out",
            ),
        }
    }

    pub async fn execute(&self, command: AgentCommand) -> CommandResult {
        match command {
            AgentCommand::FocusPane(pane_id) => self.focus_pane(pane_id).await,
            AgentCommand::FocusVerifiedPane { pane_id, identity } => {
                self.focus_verified_pane(pane_id, &identity).await
            }
            AgentCommand::SendCounsel {
                pane_id,
                text,
                request,
            } => self.send_counsel(pane_id, text, request).await,
            AgentCommand::SubmitCounsel { pane_id, request } => {
                self.submit_counsel(pane_id, request).await
            }
            AgentCommand::LoadOutput {
                pane_id,
                lines,
                request,
            } => {
                if self.is_managed_pane(&pane_id) {
                    return CommandResult::OutputFailed {
                        pane_id,
                        request,
                        message: "refused operation on the Questmancer guild pane".to_owned(),
                    };
                }
                match self
                    .client
                    .read_recent_unwrapped(pane_id.as_str(), lines)
                    .await
                {
                    Ok(read) => CommandResult::OutputLoaded {
                        pane_id,
                        request,
                        revision: read.revision,
                        text: read.text,
                        truncated: read.truncated,
                    },
                    Err(error) => CommandResult::OutputFailed {
                        pane_id,
                        request,
                        message: error.to_string(),
                    },
                }
            }
            AgentCommand::ExplainStatus(request) => CommandResult::StatusExplained {
                serial: request.serial,
                result: self.explain_status(&request).await,
            },
            AgentCommand::DiscoverReviewr { qualified_id } => {
                if split_qualified_action(&qualified_id).is_none() {
                    return CommandResult::ReviewrAvailable(false);
                }
                match self.client.list_plugin_actions().await {
                    Ok(actions) => CommandResult::ReviewrAvailable(
                        actions
                            .iter()
                            .any(|action| action.qualified_id() == qualified_id),
                    ),
                    Err(error) => failed("discover reviewr", error),
                }
            }
            AgentCommand::RefreshSnapshot(request) => self.refresh_snapshot(request).await,
            AgentCommand::InspectSpoils {
                pane_id,
                qualified_id,
            } => {
                if self.is_managed_pane(&pane_id) {
                    return Self::refused_managed_pane("open reviewr");
                }
                let Some((plugin_id, action_id)) = split_qualified_action(&qualified_id) else {
                    return CommandResult::Failed {
                        operation: "open reviewr",
                        message: format!(
                            "reviewr action {qualified_id:?} must be qualified as <plugin>.<action>"
                        ),
                    };
                };
                if let Err(error) = self.client.focus_pane(pane_id.as_str()).await {
                    return failed("open reviewr", error);
                }
                match self
                    .client
                    .invoke_plugin_action(plugin_id, action_id, pane_id.as_str())
                    .await
                {
                    Ok(()) => CommandResult::SpoilsOpened,
                    Err(error) => failed("open reviewr", error),
                }
            }
            AgentCommand::SetUrgencyView
            | AgentCommand::PublishMarginalia(_)
            | AgentCommand::PublishPaneHeaders(_) => self.execute_sidebar(command).await,
        }
    }

    async fn publish_marginalia(&self, projection: SidebarProjection) -> Result<(), ClientError> {
        for agent in projection.agents {
            self.client
                .report_pane_tokens(
                    agent.pane_id.as_str(),
                    SIDEBAR_SOURCE,
                    optional_tokens(agent.tokens),
                    self.next_sidebar_sequence.fetch_add(1, Ordering::Relaxed),
                )
                .await?;
        }
        for campaign in projection.campaigns {
            self.client
                .report_workspace_tokens(
                    campaign.workspace_id.as_str(),
                    SIDEBAR_SOURCE,
                    optional_tokens(campaign.tokens),
                    self.next_sidebar_sequence.fetch_add(1, Ordering::Relaxed),
                )
                .await?;
        }
        Ok(())
    }

    pub fn has_pane_header_attempts(&self) -> bool {
        self.headers_attempted.load(Ordering::Relaxed)
    }

    pub fn reset_header_capability(&self) {
        self.header_capability.store(0, Ordering::Relaxed);
    }

    async fn publish_pane_headers(
        &self,
        projections: Vec<PaneHeaderProjection>,
    ) -> Result<bool, ClientError> {
        let projections = projections
            .into_iter()
            .filter(|header| !self.is_managed_pane(&header.pane_id))
            .collect::<Vec<_>>();
        if projections.is_empty() {
            self.clear_pane_headers().await?;
            return Ok(self.header_capability.load(Ordering::Relaxed) == 1);
        }
        let available = match self.header_capability.load(Ordering::Relaxed) {
            1 => true,
            2 => false,
            _ => {
                let available = self.client.pane_headers_supported().await?;
                self.header_capability
                    .store(if available { 1 } else { 2 }, Ordering::Relaxed);
                available
            }
        };
        if !available {
            return Ok(false);
        }
        // Track possible delivery, including a timeout after the server accepted it.
        self.headers_attempted.store(true, Ordering::Relaxed);
        self.client
            .sync_pane_headers(
                SIDEBAR_SOURCE,
                self.next_sidebar_sequence.fetch_add(1, Ordering::Relaxed),
                projections,
            )
            .await?;
        Ok(true)
    }

    /// The two commands that write to Herdr's own sidebar rather than to a
    /// pane: publishing marginalia tokens, and asking for the urgency order.
    async fn execute_sidebar(&self, command: AgentCommand) -> CommandResult {
        match command {
            AgentCommand::SetUrgencyView => match self.set_urgency_view().await {
                Ok(()) => CommandResult::UrgencyViewSet,
                Err(error) => CommandResult::UrgencyViewFailed {
                    message: error.to_string(),
                },
            },
            AgentCommand::PublishMarginalia(projection) => {
                match self.publish_marginalia(projection).await {
                    Ok(()) => CommandResult::MarginaliaPublished,
                    Err(error) => CommandResult::MarginaliaFailed {
                        message: error.to_string(),
                    },
                }
            }
            AgentCommand::PublishPaneHeaders(projection) => {
                match tokio::time::timeout(
                    std::time::Duration::from_secs(2),
                    self.publish_pane_headers(projection),
                )
                .await
                {
                    Ok(Ok(available)) => CommandResult::PaneHeadersPublished { available },
                    Ok(Err(error)) => CommandResult::PaneHeadersFailed {
                        message: error.to_string(),
                    },
                    Err(_) => CommandResult::PaneHeadersFailed {
                        message: "header request timed out".into(),
                    },
                }
            }
            _ => CommandResult::MarginaliaPublished,
        }
    }

    /// Asks Herdr to sort its agent list by Questmancer's urgency rank.
    pub async fn set_urgency_view(&self) -> Result<(), ClientError> {
        self.client
            .set_agent_view(SIDEBAR_SOURCE, "Questmancer urgency")
            .await
    }

    /// Hands the ordering back. Best-effort on shutdown: a Questmancer that
    /// has stopped must not leave Herdr's sidebar sorted on its behalf.
    pub async fn clear_urgency_view(&self) -> Result<(), ClientError> {
        self.client.clear_agent_view(SIDEBAR_SOURCE).await
    }

    pub async fn clear_pane_headers(&self) -> Result<(), ClientError> {
        if !self.headers_attempted.load(Ordering::Relaxed) {
            return Ok(());
        }
        self.client
            .sync_pane_headers(
                SIDEBAR_SOURCE,
                self.next_sidebar_sequence.fetch_add(1, Ordering::Relaxed),
                Vec::new(),
            )
            .await?;
        self.headers_attempted.store(false, Ordering::Relaxed);
        Ok(())
    }

    pub async fn clear_marginalia(
        &self,
        projection: &SidebarProjection,
    ) -> Result<(), ClientError> {
        for agent in &projection.agents {
            self.client
                .report_pane_tokens(
                    agent.pane_id.as_str(),
                    SIDEBAR_SOURCE,
                    cleared_tokens(&agent.tokens),
                    self.next_sidebar_sequence.fetch_add(1, Ordering::Relaxed),
                )
                .await?;
        }
        for campaign in &projection.campaigns {
            self.client
                .report_workspace_tokens(
                    campaign.workspace_id.as_str(),
                    SIDEBAR_SOURCE,
                    cleared_tokens(&campaign.tokens),
                    self.next_sidebar_sequence.fetch_add(1, Ordering::Relaxed),
                )
                .await?;
        }
        Ok(())
    }
}

fn optional_tokens(
    tokens: std::collections::BTreeMap<String, String>,
) -> std::collections::BTreeMap<String, Option<String>> {
    tokens
        .into_iter()
        .map(|(token, value)| (token, Some(value)))
        .collect()
}

fn cleared_tokens(
    tokens: &std::collections::BTreeMap<String, String>,
) -> std::collections::BTreeMap<String, Option<String>> {
    tokens.keys().cloned().map(|token| (token, None)).collect()
}

fn sidebar_sequence_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(1, |duration| {
            u64::try_from(duration.as_millis()).unwrap_or(u64::MAX.saturating_sub(1))
        })
}

fn split_qualified_action(qualified_id: &str) -> Option<(&str, &str)> {
    let (plugin_id, action_id) = qualified_id.rsplit_once('.')?;
    (!plugin_id.is_empty() && !action_id.is_empty()).then_some((plugin_id, action_id))
}

fn failed(operation: &'static str, error: impl std::fmt::Display) -> CommandResult {
    CommandResult::Failed {
        operation,
        message: error.to_string(),
    }
}
