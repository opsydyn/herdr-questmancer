use std::{collections::BTreeMap, path::PathBuf};

use serde::{Deserialize, Serialize};

use super::{Agent, AgentKey, GuildSummons, Presence, WorkspaceId};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Campaign {
    pub workspace_id: WorkspaceId,
    pub label: String,
    pub cwd: PathBuf,
    pub party: Vec<AgentKey>,
}

impl Campaign {
    #[must_use]
    pub fn status(&self, agents: &BTreeMap<AgentKey, Agent>) -> CampaignStatus {
        let party = self.party.iter().filter_map(|key| agents.get(key));
        CampaignStatus::derive(party)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CampaignStatus {
    CounselRequired,
    SpoilsAwaitingInspection,
    ExpeditionActive,
    PartyAtRest,
    Abandoned,
}

impl CampaignStatus {
    fn derive<'a>(mut agents: impl Iterator<Item = &'a Agent> + Clone) -> Self {
        if agents
            .clone()
            .any(|agent| agent.presence == Presence::Blocked)
        {
            Self::CounselRequired
        } else if agents.clone().any(|agent| {
            agent.attention.is_unread()
                && agent.attention.summons() == Some(GuildSummons::SpoilsReturned)
        }) {
            Self::SpoilsAwaitingInspection
        } else if agents
            .clone()
            .any(|agent| agent.presence == Presence::Working)
        {
            Self::ExpeditionActive
        } else if agents.any(|agent| agent.presence != Presence::Exited) {
            Self::PartyAtRest
        } else {
            Self::Abandoned
        }
    }
}
