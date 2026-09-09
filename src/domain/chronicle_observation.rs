//! Version-two observations retain their source and captured subject identity.
use serde::{Deserialize, Serialize};

use super::{
    Agent, AgentKey, Campaign, ChronicleEvent, EventId, ObservationStamp, PaneId, Presence,
    WorkspaceId,
};
use crate::snapshot_refresh::SnapshotRequest;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservedPresence {
    Working,
    Blocked,
    Done,
    Idle,
    Unknown,
}

impl ObservedPresence {
    pub const fn from_presence(presence: Presence) -> Option<Self> {
        match presence {
            Presence::Working => Some(Self::Working),
            Presence::Blocked => Some(Self::Blocked),
            Presence::Done => Some(Self::Done),
            Presence::Idle => Some(Self::Idle),
            Presence::Unknown => Some(Self::Unknown),
            Presence::Exited => None,
        }
    }

    pub const fn observed_copy(self) -> &'static str {
        match self {
            Self::Working => "was observed delving",
            Self::Blocked => "was observed needing counsel",
            Self::Done => "was observed with spoils reported",
            Self::Idle => "was observed resting",
            Self::Unknown => "whereabouts were observed as unknown",
        }
    }

    pub const fn status_event(self) -> ChronicleEvent {
        match self {
            Self::Working => ChronicleEvent::DelveBegan,
            Self::Blocked => ChronicleEvent::CounselRequested,
            Self::Done => ChronicleEvent::SpoilsReturned,
            Self::Idle => ChronicleEvent::AdventurerRested,
            Self::Unknown => ChronicleEvent::WhereaboutsUnknown,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ObservationSubject {
    Adventurer {
        key: AgentKey,
        campaign: WorkspaceId,
        pane: PaneId,
        incarnation: String,
        revision: u64,
        name: String,
    },
    Campaign {
        id: WorkspaceId,
        name: String,
    },
}

impl ObservationSubject {
    pub fn adventurer(agent: &Agent) -> Option<Self> {
        Some(Self::Adventurer {
            key: agent.key.clone(),
            campaign: agent.workspace_id.clone(),
            pane: agent.pane_id.clone(),
            incarnation: agent.capture_identity.fingerprint()?,
            revision: agent.pane_revision,
            name: agent.name.clone(),
        })
    }
    pub fn campaign(campaign: &Campaign) -> Self {
        Self::Campaign {
            id: campaign.workspace_id.clone(),
            name: campaign.label.clone(),
        }
    }
    pub fn name(&self) -> &str {
        match self {
            Self::Adventurer { name, .. } | Self::Campaign { name, .. } => name,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RemovalEvidence {
    NoLongerVisible,
    CloseHintConfirmed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SnapshotObservation {
    PresenceObserved { presence: ObservedPresence },
    AdventurerObserved { presence: ObservedPresence },
    AdventurerNoLongerVisible,
    CampaignRemoved { evidence: RemovalEvidence },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "source", rename_all = "snake_case", deny_unknown_fields)]
pub enum ObservationEvidence {
    Snapshot {
        request: SnapshotRequest,
        observation: SnapshotObservation,
    },
    PaneMetadata {
        presence: ObservedPresence,
    },
    LifecycleDeparture,
}

impl ObservationEvidence {
    pub const fn event(&self) -> ChronicleEvent {
        match self {
            Self::Snapshot { observation, .. } => match observation {
                SnapshotObservation::PresenceObserved {
                    presence: ObservedPresence::Unknown,
                } => ChronicleEvent::WhereaboutsUnknown,
                SnapshotObservation::PresenceObserved { .. } => ChronicleEvent::PresenceObserved,
                SnapshotObservation::AdventurerObserved { .. } => {
                    ChronicleEvent::AdventurerObserved
                }
                SnapshotObservation::AdventurerNoLongerVisible => {
                    ChronicleEvent::AdventurerNoLongerVisible
                }
                SnapshotObservation::CampaignRemoved { .. } => ChronicleEvent::CampaignRemoved,
            },
            Self::PaneMetadata { presence } => presence.status_event(),
            Self::LifecycleDeparture => ChronicleEvent::AdventurerDeparted,
        }
    }

    pub const fn label(&self) -> &'static str {
        match self {
            Self::Snapshot { .. } => "snapshot observation",
            Self::PaneMetadata { .. } => "pane metadata observation",
            Self::LifecycleDeparture => "corroborated departure",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CapturedObservation {
    pub stamp: ObservationStamp,
    pub subject: ObservationSubject,
    pub evidence: ObservationEvidence,
}

impl CapturedObservation {
    pub fn event_id(&self) -> EventId {
        // Display text and wall-clock time never participate in observation IDs.
        // Names are captured for presentation but are not subject identity.
        let subject = match &self.subject {
            ObservationSubject::Adventurer {
                key,
                campaign,
                pane,
                incarnation,
                revision,
                ..
            } => serde_json::json!(["adventurer", key, campaign, pane, incarnation, revision]),
            ObservationSubject::Campaign { id, .. } => serde_json::json!(["campaign", id]),
        };
        let bytes = serde_json::to_vec(&(&self.stamp, subject, self.evidence.event()))
            .expect("observation identity contains only JSON-safe typed values");
        let mut hash = blake3::Hasher::new();
        hash.update(b"questmancer-chronicle-observation-v2\0");
        hash.update(&bytes);
        EventId::new(format!("observation-{}", hash.finalize().to_hex()))
    }

    pub fn valid(&self) -> bool {
        if self.stamp.run.as_str().trim().is_empty()
            || self.stamp.epoch.0 == 0
            || self.stamp.ordinal == 0
        {
            return false;
        }
        let campaign_event = matches!(
            &self.evidence,
            ObservationEvidence::Snapshot {
                observation: SnapshotObservation::CampaignRemoved { .. },
                ..
            }
        );
        match &self.subject {
            ObservationSubject::Campaign { id, .. }
                if !campaign_event || id.as_str().is_empty() =>
            {
                return false;
            }
            ObservationSubject::Adventurer {
                key,
                campaign,
                pane,
                incarnation,
                ..
            } => {
                if campaign_event
                    || [key.as_str(), campaign.as_str(), pane.as_str(), incarnation]
                        .iter()
                        .any(|id| id.is_empty())
                {
                    return false;
                }
            }
            ObservationSubject::Campaign { .. } => {}
        }
        match &self.evidence {
            ObservationEvidence::Snapshot { request, .. } => {
                request.epoch == self.stamp.epoch && request.id.0 > 0
            }
            ObservationEvidence::PaneMetadata { .. } | ObservationEvidence::LifecycleDeparture => {
                true
            }
        }
    }

    pub fn summary(&self) -> String {
        let name = self.subject.name();
        match &self.evidence {
            ObservationEvidence::Snapshot { observation, .. } => match observation {
                SnapshotObservation::PresenceObserved {
                    presence: ObservedPresence::Unknown,
                } => format!("{name}'s whereabouts were observed as unknown"),
                SnapshotObservation::PresenceObserved { presence } => {
                    format!("{name} {}", presence.observed_copy())
                }
                SnapshotObservation::AdventurerObserved { .. } => {
                    format!("{name} was first observed in the guild")
                }
                SnapshotObservation::AdventurerNoLongerVisible => {
                    format!("{name} was no longer visible in the guild")
                }
                SnapshotObservation::CampaignRemoved {
                    evidence: RemovalEvidence::NoLongerVisible,
                } => format!("Campaign {name} was no longer visible"),
                SnapshotObservation::CampaignRemoved {
                    evidence: RemovalEvidence::CloseHintConfirmed,
                } => format!("Campaign {name} closed"),
            },
            ObservationEvidence::PaneMetadata { presence } => match presence {
                ObservedPresence::Working => format!("{name} began a delve"),
                ObservedPresence::Blocked => format!("{name} requested counsel"),
                ObservedPresence::Done => format!("{name} returned with spoils"),
                ObservedPresence::Idle => format!("{name} made camp"),
                ObservedPresence::Unknown => format!("{name}'s whereabouts became unknown"),
            },
            ObservationEvidence::LifecycleDeparture => format!("{name} departed the guild"),
        }
    }
}
