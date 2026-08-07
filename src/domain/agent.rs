use serde::{Deserialize, Serialize};

use crate::herdr::protocol::{AgentInfo, AgentStatus};

use super::{
    AdventurerPersona, AgentKey, GuildAttention, GuildSummons, PaneId, PersonaKey, TabId,
    Timestamp, WorkspaceId,
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
    pub persona: AdventurerPersona,
}

impl Agent {
    /// How loudly this adventurer is asking for a human, lowest first.
    ///
    /// `None` means nobody is waiting on you for this one. The three ranks are
    /// the only distinctions the guild actually draws: an unanswered call for
    /// counsel, a call somebody has seen but not resolved, and the quieter
    /// summons that still deserve a look.
    ///
    /// This lives in the domain because two things order by it — the `!` jump
    /// inside Questmancer and the rank token Herdr sorts its own sidebar by.
    /// Two definitions of "urgent" would drift, and the sidebar would disagree
    /// with the key.
    #[must_use]
    pub fn urgency_rank(&self, now: Timestamp) -> Option<u8> {
        if let GuildAttention::Deferred { until, .. } = self.attention
            && until.as_millis() > now.as_millis()
        {
            // Deferring said "not now". Honour it until it expires.
            return None;
        }
        match (&self.attention, self.presence) {
            (
                GuildAttention::Unread {
                    summons: GuildSummons::CounselRequested,
                    ..
                },
                _,
            ) => Some(0),
            (_, Presence::Blocked) => Some(1),
            (GuildAttention::Unread { .. } | GuildAttention::Deferred { .. }, _) => Some(2),
            _ => None,
        }
    }

    /// The rank as a single sortable digit, `3` meaning "nothing wanted".
    ///
    /// One digit on purpose: Herdr sorts custom tokens as the strings they are,
    /// and a single digit orders identically whether the comparison is
    /// lexicographic or numeric. Anything wider would depend on which.
    #[must_use]
    pub fn urgency_digit(&self, now: Timestamp) -> String {
        self.urgency_rank(now).map_or(3, |rank| rank).to_string()
    }

    #[must_use]
    pub fn from_snapshot(
        agent: &AgentInfo,
        workspace_root: Option<&str>,
        observed_at: Timestamp,
    ) -> Self {
        let persona = AdventurerPersona::for_agent(agent, workspace_root);
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
