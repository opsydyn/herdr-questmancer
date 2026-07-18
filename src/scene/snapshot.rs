use crate::{
    app::{ConnectionState, Model, Motion},
    domain::{AdventurerPersona, AgentKey, GuildSummons, Presence, Timestamp, WorkspaceId},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SceneConnection {
    Offline,
    Connecting,
    Connected,
    Reconnecting { attempt: u32 },
    Incompatible { expected: u32, actual: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SceneSnapshot {
    pub connection: SceneConnection,
    pub campaigns: Vec<SceneCampaign>,
    pub agents: Vec<SceneAgent>,
    pub motion: Motion,
    pub now: Timestamp,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SceneCampaign {
    pub workspace_id: WorkspaceId,
    pub label: String,
    pub variant_seed: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SceneAgent {
    pub key: AgentKey,
    pub workspace_id: WorkspaceId,
    pub name: String,
    pub custom_status: Option<String>,
    pub presence: Presence,
    pub presence_since: Timestamp,
    pub transition: Option<SceneTransition>,
    pub focused: bool,
    pub persona: AdventurerPersona,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneTransition {
    pub summons: GuildSummons,
    pub since: Timestamp,
}

impl SceneSnapshot {
    #[must_use]
    pub fn from_model(model: &Model) -> Self {
        let mut campaigns = model
            .domain()
            .campaigns
            .values()
            .map(|campaign| SceneCampaign {
                workspace_id: campaign.workspace_id.clone(),
                label: campaign.label.clone(),
                variant_seed: variant_seed(&campaign.workspace_id),
            })
            .collect::<Vec<_>>();
        campaigns.sort_by(|left, right| left.workspace_id.cmp(&right.workspace_id));

        let mut agents = model
            .domain()
            .agents
            .values()
            .map(|agent| SceneAgent {
                key: agent.key.clone(),
                workspace_id: agent.workspace_id.clone(),
                name: agent.name.clone(),
                custom_status: agent.custom_status.clone(),
                presence: agent.presence,
                presence_since: agent.presence_since,
                transition: agent
                    .attention
                    .summons()
                    .zip(agent.attention.since())
                    .map(|(summons, since)| SceneTransition { summons, since }),
                focused: agent.focused,
                persona: agent.persona.clone(),
            })
            .collect::<Vec<_>>();
        agents.sort_by(|left, right| left.key.cmp(&right.key));

        Self {
            connection: SceneConnection::from(model.connection()),
            campaigns,
            agents,
            motion: model.preferences().motion,
            now: model.now(),
        }
    }
}

impl From<&ConnectionState> for SceneConnection {
    fn from(connection: &ConnectionState) -> Self {
        match connection {
            ConnectionState::Offline => Self::Offline,
            ConnectionState::Connecting => Self::Connecting,
            ConnectionState::Connected => Self::Connected,
            ConnectionState::Reconnecting { attempt } => Self::Reconnecting { attempt: *attempt },
            ConnectionState::Incompatible { expected, actual } => Self::Incompatible {
                expected: *expected,
                actual: *actual,
            },
        }
    }
}

fn variant_seed(workspace_id: &WorkspaceId) -> u64 {
    let digest = blake3::hash(workspace_id.as_str().as_bytes());
    let bytes = digest.as_bytes();
    u64::from_le_bytes(
        bytes[..8]
            .try_into()
            .expect("BLAKE3 digest has eight bytes"),
    )
}
