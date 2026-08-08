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
    /// How loudly this adventurer is asking for a human.
    ///
    /// `None` means nobody is waiting on you for this one.
    ///
    /// This lives in the domain because two things order by it — the `!` jump
    /// inside Questmancer and the rank token Herdr sorts its own sidebar by.
    /// Two definitions of "urgent" would drift, and the sidebar would disagree
    /// with the key.
    #[must_use]
    pub fn urgency(&self, now: Timestamp) -> Option<Urgency> {
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
            ) => Some(Urgency::UnansweredCounsel),
            (_, Presence::Blocked) => Some(Urgency::SeenCounsel),
            (GuildAttention::Unread { .. } | GuildAttention::Deferred { .. }, _) => {
                Some(Urgency::QuieterSummons)
            }
            _ => None,
        }
    }

    /// The urgency as the single sortable digit Herdr orders its list by.
    ///
    /// One digit on purpose: Herdr sorts custom tokens as the strings they are,
    /// and a single digit orders identically whether the comparison is
    /// lexicographic or numeric. Anything wider would depend on which.
    #[must_use]
    pub fn urgency_digit(&self, now: Timestamp) -> String {
        self.urgency(now)
            .map_or(Urgency::NOTHING_WANTED, Urgency::digit)
            .to_string()
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

/// How loudly an adventurer is asking for a human.
///
/// This was `Option<u8>` with `Some(0)`, `Some(1)`, `Some(2)` and a bare `3`
/// written into the one place that needed "nobody is waiting". The type
/// admitted `Some(47)`, the ordering lived in the numbers rather than in
/// anything named, and the meaning of `3` was recorded in prose in a different
/// file. Three states the guild actually draws deserve three names.
///
/// Declaration order is priority order and `Ord` follows it, so the urgency
/// jump and Herdr's sidebar sort agree without either restating the ranking.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Urgency {
    /// A call for counsel nobody has answered yet.
    UnansweredCounsel,
    /// A call somebody has seen and not yet resolved.
    SeenCounsel,
    /// A quieter summons that still deserves a look.
    QuieterSummons,
}

impl Urgency {
    /// Every urgency, most pressing first.
    pub const ALL: &'static [Self] = &[
        Self::UnansweredCounsel,
        Self::SeenCounsel,
        Self::QuieterSummons,
    ];

    /// The digit for an adventurer nobody is waiting on.
    ///
    /// Higher than every real urgency so it sorts last, and named here beside
    /// them rather than written into the one call site that needed it.
    pub const NOTHING_WANTED: u8 = 3;

    /// The sortable digit Herdr orders its own agent list by.
    #[must_use]
    pub const fn digit(self) -> u8 {
        match self {
            Self::UnansweredCounsel => 0,
            Self::SeenCounsel => 1,
            Self::QuieterSummons => 2,
        }
    }
}
