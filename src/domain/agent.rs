use serde::{Deserialize, Serialize};

use crate::herdr::protocol::{AgentInfo, AgentStatus};

use super::{
    AgentKey, AgentPersona, GuildAttention, GuildSummons, PaneId, PersonaKey, TabId, Timestamp,
    WorkspaceId,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Presence {
    Working,
    Blocked,
    Done,
    Idle,
    Exited,
    Unknown,
}

impl From<AgentStatus> for Presence {
    fn from(status: AgentStatus) -> Self {
        match status {
            AgentStatus::Working => Self::Working,
            AgentStatus::Blocked => Self::Blocked,
            AgentStatus::Done => Self::Done,
            AgentStatus::Idle => Self::Idle,
            AgentStatus::Unknown => Self::Unknown,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Agent {
    pub key: AgentKey,
    pub pane_id: PaneId,
    pub workspace_id: WorkspaceId,
    pub tab_id: TabId,
    pub name: String,
    pub custom_status: Option<String>,
    pub presence: Presence,
    pub presence_since: Timestamp,
    pub attention: GuildAttention,
    pub focused: bool,
    pub pane_revision: u64,
    pub persona: AgentPersona,
}

impl Agent {
    #[must_use]
    pub fn from_snapshot(
        agent: &AgentInfo,
        workspace_root: Option<&str>,
        observed_at: Timestamp,
    ) -> Self {
        let persona = AgentPersona::for_agent(agent, workspace_root);
        let key = AgentKey::from_persona_key(&persona.key);
        let presence = Presence::from(agent.agent_status);
        let attention = match presence {
            Presence::Blocked => {
                GuildAttention::unread(GuildSummons::CounselRequested, observed_at)
            }
            Presence::Done => GuildAttention::unread(GuildSummons::SpoilsReturned, observed_at),
            Presence::Working | Presence::Idle | Presence::Exited | Presence::Unknown => {
                GuildAttention::Clear
            }
        };
        Self {
            key,
            pane_id: PaneId::new(&agent.pane_id),
            workspace_id: WorkspaceId::new(&agent.workspace_id),
            tab_id: TabId::new(&agent.tab_id),
            name: display_name(agent),
            custom_status: agent.custom_status.clone(),
            presence,
            presence_since: observed_at,
            attention,
            focused: agent.focused,
            pane_revision: agent.revision,
            persona,
        }
    }
}

impl AgentKey {
    pub(crate) fn from_persona_key(key: &PersonaKey) -> Self {
        Self::new(key.as_str().replacen("persona-", "agent-", 1))
    }
}

fn display_name(agent: &AgentInfo) -> String {
    agent
        .display_agent
        .as_deref()
        .or(agent.name.as_deref())
        .or(agent.agent.as_deref())
        .or(agent.title.as_deref())
        .unwrap_or("unknown agent")
        .to_owned()
}
