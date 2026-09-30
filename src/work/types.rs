use crate::domain::{AgentKey, PaneId, Timestamp};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

macro_rules! id {
    ($name:ident) => {
        #[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);
        impl $name {
            pub fn new(source: &SourceId, local: &str) -> Self {
                Self(format!("{}/{local}", source.0))
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
            pub(crate) fn belongs_to(&self, source: &SourceId) -> bool {
                self.0
                    .strip_prefix(&format!("{}/", source.0))
                    .is_some_and(|local| {
                        !local.is_empty()
                            && local.len() <= 256
                            && local
                                .bytes()
                                .all(|b| b.is_ascii_alphanumeric() || b"._:-".contains(&b))
                    })
            }
        }
    };
}
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct SourceId(pub String);
impl SourceId {
    pub fn new(value: &str) -> Self {
        Self(value.to_owned())
    }
    pub fn valid(&self) -> bool {
        !self.0.is_empty()
            && self.0.len() <= 128
            && self
                .0
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._:-".contains(&b))
    }
}
id!(CampaignId);
id!(QuestId);
id!(ObligationId);
id!(ArtifactId);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum CampaignState {
    Planned,
    Active,
    Paused,
    Completed,
    Abandoned,
    Archived,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum QuestKind {
    Unspecified,
    Scout,
    Ship,
    Review,
    Validate,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum QuestState {
    Unknown,
    Planned,
    Ready,
    Active,
    Completed,
    Failed,
    Cancelled,
    Closed,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ObligationKind {
    BlockedAction,
    Decision,
    Permission,
    ExecutionAction,
    Review,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Responsible {
    Human,
    Supervisor,
    External,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ObligationState {
    Open,
    Resolved,
    Superseded,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ArtifactKind {
    PullRequest,
    Commit,
    Diff,
    Report,
    TestResult,
    Benchmark,
    Document,
    Screenshot,
    Other,
}
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum Freshness {
    Current,
    #[default]
    Cached,
    Unavailable,
    Reconcile,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkCampaign {
    pub id: CampaignId,
    pub revision: u64,
    pub name: String,
    pub objective: Option<String>,
    pub state: CampaignState,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Assignment {
    pub worker: String,
    pub name: String,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Quest {
    pub id: QuestId,
    pub campaign: CampaignId,
    pub revision: u64,
    pub title: String,
    pub description: Option<String>,
    pub kind: QuestKind,
    pub state: QuestState,
    pub assignment: Option<Assignment>,
    pub dependencies: Vec<QuestId>,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Obligation {
    pub id: ObligationId,
    pub campaign: CampaignId,
    pub quest: Option<QuestId>,
    pub revision: u64,
    pub incarnation: String,
    pub kind: ObligationKind,
    pub responsible: Responsible,
    pub state: ObligationState,
    pub prompt: String,
    pub context: Option<String>,
    pub action: String,
    pub resolution: String,
    pub requested_at: Timestamp,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub id: ArtifactId,
    pub campaign: CampaignId,
    pub quest: Option<QuestId>,
    pub revision: u64,
    pub kind: ArtifactKind,
    pub title: String,
    pub uri: Option<String>,
    pub head_revision: Option<String>,
    pub validation: Option<Validation>,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Validation {
    pub head_revision: String,
    pub summary: String,
}
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Inventory {
    pub campaigns: BTreeMap<CampaignId, WorkCampaign>,
    pub quests: BTreeMap<QuestId, Quest>,
    pub obligations: BTreeMap<ObligationId, Obligation>,
    pub artifacts: BTreeMap<ArtifactId, Artifact>,
}
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Retired {
    pub campaigns: BTreeMap<CampaignId, u64>,
    pub quests: BTreeMap<QuestId, u64>,
    pub obligations: BTreeMap<ObligationId, (u64, String)>,
    pub artifacts: BTreeMap<ArtifactId, u64>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveBinding {
    pub agent: AgentKey,
    pub pane: PaneId,
    pub identity: String,
    pub counsel_since: Option<Timestamp>,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceState {
    pub generation: u64,
    pub captured_at: Timestamp,
    pub last_occurrence: Option<Timestamp>,
    pub cursor: u64,
    pub last_event: String,
    pub last_changes: Vec<Change>,
    pub inventory: Inventory,
    pub retired: Retired,
    #[serde(skip)]
    pub freshness: Freshness,
    #[serde(skip)]
    pub bindings: BTreeMap<QuestId, LiveBinding>,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalIntent {
    pub incarnation: String,
    pub seen: bool,
    pub deferred_until: Option<Timestamp>,
}
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Projection {
    pub sources: BTreeMap<SourceId, SourceState>,
    pub intent: BTreeMap<ObligationId, LocalIntent>,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Change {
    Campaign(WorkCampaign),
    Quest(Quest),
    Obligation(Obligation),
    Artifact(Artifact),
    Resolve {
        id: ObligationId,
        incarnation: String,
        revision: u64,
    },
    RemoveQuest {
        id: QuestId,
        revision: u64,
    },
    RemoveObligation {
        id: ObligationId,
        incarnation: String,
        revision: u64,
    },
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Message {
    Baseline {
        source: SourceId,
        generation: u64,
        captured_at: Timestamp,
        cursor: u64,
        inventory: Inventory,
        complete: bool,
        bindings: BTreeMap<QuestId, LiveBinding>,
    },
    Update {
        source: SourceId,
        generation: u64,
        occurred_at: Timestamp,
        cursor: u64,
        event: String,
        changes: Vec<Change>,
    },
    Disconnected(SourceId),
    BindingRemoved {
        source: SourceId,
        quest: QuestId,
    },
}
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum WorkError {
    #[error("invalid work source or inventory")]
    Invalid,
    #[error("work source requires baseline reconciliation")]
    Reconcile,
    #[error("conflicting work revision")]
    Conflict,
}
