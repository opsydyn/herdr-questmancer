use crate::{
    domain::PaneId,
    herdr::{client::HerdrClient, protocol::SessionSnapshot},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgentCommand {
    FocusPane(PaneId),
    SendCounsel {
        pane_id: PaneId,
        text: String,
    },
    LoadOutput {
        pane_id: PaneId,
        lines: u32,
    },
    RefreshSnapshot,
    DiscoverReviewr {
        qualified_id: String,
    },
    InspectSpoils {
        pane_id: PaneId,
        qualified_id: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum CommandResult {
    Focused(PaneId),
    CounselSent(PaneId),
    OutputLoaded {
        pane_id: PaneId,
        revision: u64,
        text: String,
        truncated: bool,
    },
    OutputFailed {
        pane_id: PaneId,
        message: String,
    },
    ReviewrAvailable(bool),
    SpoilsOpened,
    SnapshotLoaded(Box<SessionSnapshot>),
    Failed {
        operation: &'static str,
        message: String,
    },
}

#[derive(Clone, Debug)]
pub struct CommandExecutor {
    client: HerdrClient,
    managed_pane_id: Option<PaneId>,
}

impl CommandExecutor {
    #[must_use]
    pub const fn new(client: HerdrClient, managed_pane_id: Option<PaneId>) -> Self {
        Self {
            client,
            managed_pane_id,
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

    pub async fn execute(&self, command: AgentCommand) -> CommandResult {
        match command {
            AgentCommand::FocusPane(pane_id) => {
                if self.is_managed_pane(&pane_id) {
                    return Self::refused_managed_pane("focus pane");
                }
                match self.client.focus_pane(pane_id.as_str()).await {
                    Ok(_) => CommandResult::Focused(pane_id),
                    Err(error) => failed("focus pane", error),
                }
            }
            AgentCommand::SendCounsel { pane_id, text } => {
                if self.is_managed_pane(&pane_id) {
                    return Self::refused_managed_pane("send counsel");
                }
                match self.client.send_text(pane_id.as_str(), text).await {
                    Ok(()) => CommandResult::CounselSent(pane_id),
                    Err(error) => failed("send counsel", error),
                }
            }
            AgentCommand::LoadOutput { pane_id, lines } => {
                if self.is_managed_pane(&pane_id) {
                    return CommandResult::OutputFailed {
                        pane_id,
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
                        revision: read.revision,
                        text: read.text,
                        truncated: read.truncated,
                    },
                    Err(error) => CommandResult::OutputFailed {
                        pane_id,
                        message: error.to_string(),
                    },
                }
            }
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
            AgentCommand::RefreshSnapshot => match self.client.snapshot().await {
                Ok(snapshot) => CommandResult::SnapshotLoaded(Box::new(snapshot)),
                Err(error) => failed("refresh snapshot", error),
            },
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
        }
    }
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
