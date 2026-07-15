use crate::{
    domain::PaneId,
    herdr::{client::HerdrClient, protocol::SessionSnapshot},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeskCommand {
    FocusPane(PaneId),
    SendReply {
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
    OpenReviewr {
        pane_id: PaneId,
        qualified_id: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum CommandResult {
    Focused(PaneId),
    ReplySent(PaneId),
    OutputLoaded {
        pane_id: PaneId,
        revision: u64,
        text: String,
        truncated: bool,
    },
    ReviewrAvailable(bool),
    ReviewrOpened,
    SnapshotLoaded(Box<SessionSnapshot>),
    Failed {
        operation: &'static str,
        message: String,
    },
}

#[derive(Clone, Debug)]
pub struct CommandExecutor {
    client: HerdrClient,
}

impl CommandExecutor {
    #[must_use]
    pub const fn new(client: HerdrClient) -> Self {
        Self { client }
    }

    pub async fn execute(&self, command: DeskCommand) -> CommandResult {
        match command {
            DeskCommand::FocusPane(pane_id) => {
                match self.client.focus_pane(pane_id.as_str()).await {
                    Ok(_) => CommandResult::Focused(pane_id),
                    Err(error) => failed("focus pane", error),
                }
            }
            DeskCommand::SendReply { pane_id, text } => {
                match self.client.send_text(pane_id.as_str(), text).await {
                    Ok(()) => CommandResult::ReplySent(pane_id),
                    Err(error) => failed("send reply", error),
                }
            }
            DeskCommand::LoadOutput { pane_id, lines } => {
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
                    Err(error) => failed("load output", error),
                }
            }
            DeskCommand::DiscoverReviewr { qualified_id } => {
                match self.client.list_plugin_actions().await {
                    Ok(actions) => CommandResult::ReviewrAvailable(
                        actions
                            .iter()
                            .any(|action| action.qualified_id() == qualified_id),
                    ),
                    Err(error) => failed("discover reviewr", error),
                }
            }
            DeskCommand::RefreshSnapshot => match self.client.snapshot().await {
                Ok(snapshot) => CommandResult::SnapshotLoaded(Box::new(snapshot)),
                Err(error) => failed("refresh snapshot", error),
            },
            DeskCommand::OpenReviewr {
                pane_id,
                qualified_id,
            } => {
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
                    Ok(()) => CommandResult::ReviewrOpened,
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
