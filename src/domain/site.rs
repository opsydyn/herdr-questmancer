use std::{collections::BTreeMap, path::PathBuf};

use serde::{Deserialize, Serialize};

use super::{Agent, AgentKey, AttentionReason, Presence, WorkspaceId};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Site {
    pub workspace_id: WorkspaceId,
    pub label: String,
    pub cwd: PathBuf,
    pub agents: Vec<AgentKey>,
}

impl Site {
    #[must_use]
    pub fn status(&self, agents: &BTreeMap<AgentKey, Agent>) -> SiteStatus {
        let site_agents = self.agents.iter().filter_map(|key| agents.get(key));
        SiteStatus::derive(site_agents)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SiteStatus {
    NeedsWebmaster,
    UpdateReady,
    Updating,
    Online,
    Offline,
}

impl SiteStatus {
    fn derive<'a>(mut agents: impl Iterator<Item = &'a Agent> + Clone) -> Self {
        if agents
            .clone()
            .any(|agent| agent.presence == Presence::Blocked)
        {
            Self::NeedsWebmaster
        } else if agents.clone().any(|agent| {
            agent.attention.is_unseen()
                && agent.attention.reason() == Some(AttentionReason::WorkCompleted)
        }) {
            Self::UpdateReady
        } else if agents
            .clone()
            .any(|agent| agent.presence == Presence::Working)
        {
            Self::Updating
        } else if agents.any(|agent| agent.presence != Presence::Exited) {
            Self::Online
        } else {
            Self::Offline
        }
    }
}
