use super::{
    AgentKey, CapturedObservation, EventId, ObservationSubject, PaneId, Timestamp, WorkspaceId,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, VecDeque};

/// Common display fields retain their legacy meaning. V2 times are explicitly
/// local observation times; the typed content owns event/source/subject facts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChronicleEntry {
    pub id: EventId,
    pub occurred_at: Timestamp,
    pub summary: String,
    content: ChronicleContent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ChronicleContent {
    Legacy(LegacyDetails),
    Observation(Box<CapturedObservation>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LegacyDetails {
    adventurer: Option<AgentKey>,
    campaign: Option<WorkspaceId>,
    pane: Option<PaneId>,
    pane_revision: u64,
    event: ChronicleEvent,
}

/// The exact flat v1 representation. Used only for legacy input/fixtures.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LegacyChronicleEntry {
    pub id: EventId,
    pub occurred_at: Timestamp,
    pub adventurer: Option<AgentKey>,
    pub campaign: Option<WorkspaceId>,
    pub pane: Option<PaneId>,
    pub pane_revision: u64,
    pub event: ChronicleEvent,
    pub summary: String,
}

impl From<LegacyChronicleEntry> for ChronicleEntry {
    fn from(entry: LegacyChronicleEntry) -> Self {
        Self {
            id: entry.id,
            occurred_at: entry.occurred_at,
            summary: entry.summary,
            content: ChronicleContent::Legacy(LegacyDetails {
                adventurer: entry.adventurer,
                campaign: entry.campaign,
                pane: entry.pane,
                pane_revision: entry.pane_revision,
                event: entry.event,
            }),
        }
    }
}

impl ChronicleEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        occurred_at: Timestamp,
        adventurer: Option<AgentKey>,
        campaign: Option<WorkspaceId>,
        pane: Option<PaneId>,
        pane_revision: u64,
        event: ChronicleEvent,
        summary: impl Into<String>,
    ) -> Self {
        let identity = format!(
            "{}\0{}\0{}\0{}",
            event.as_str(),
            pane.as_ref().map_or("-", PaneId::as_str),
            pane_revision,
            occurred_at.as_millis()
        );
        let hash = blake3::hash(identity.as_bytes()).to_hex();
        LegacyChronicleEntry {
            id: EventId::new(format!("event-{}", &hash[..24])),
            occurred_at,
            adventurer,
            campaign,
            pane,
            pane_revision,
            event,
            summary: summary.into(),
        }
        .into()
    }

    pub fn observed(at: Timestamp, observation: CapturedObservation) -> Option<Self> {
        if !observation.valid() {
            return None;
        }
        Some(Self {
            id: observation.event_id(),
            occurred_at: at,
            summary: observation.summary(),
            content: ChronicleContent::Observation(Box::new(observation)),
        })
    }

    pub fn observation(&self) -> Option<&CapturedObservation> {
        match &self.content {
            ChronicleContent::Observation(observation) => Some(observation),
            ChronicleContent::Legacy(_) => None,
        }
    }
    pub fn event(&self) -> ChronicleEvent {
        match &self.content {
            ChronicleContent::Legacy(legacy) => legacy.event,
            ChronicleContent::Observation(observation) => observation.evidence.event(),
        }
    }
    pub fn adventurer(&self) -> Option<&AgentKey> {
        match &self.content {
            ChronicleContent::Legacy(legacy) => legacy.adventurer.as_ref(),
            ChronicleContent::Observation(observation) => match &observation.subject {
                ObservationSubject::Adventurer { key, .. } => Some(key),
                ObservationSubject::Campaign { .. } => None,
            },
        }
    }
    pub fn campaign(&self) -> Option<&WorkspaceId> {
        match &self.content {
            ChronicleContent::Legacy(legacy) => legacy.campaign.as_ref(),
            ChronicleContent::Observation(observation) => match &observation.subject {
                ObservationSubject::Adventurer { campaign, .. } => Some(campaign),
                ObservationSubject::Campaign { id, .. } => Some(id),
            },
        }
    }
    pub fn pane(&self) -> Option<&PaneId> {
        match &self.content {
            ChronicleContent::Legacy(legacy) => legacy.pane.as_ref(),
            ChronicleContent::Observation(observation) => match &observation.subject {
                ObservationSubject::Adventurer { pane, .. } => Some(pane),
                ObservationSubject::Campaign { .. } => None,
            },
        }
    }
    pub fn pane_revision(&self) -> u64 {
        match &self.content {
            ChronicleContent::Legacy(legacy) => legacy.pane_revision,
            ChronicleContent::Observation(observation) => match &observation.subject {
                ObservationSubject::Adventurer { revision, .. } => *revision,
                ObservationSubject::Campaign { .. } => 0,
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordV2 {
    record_version: u32,
    id: EventId,
    observed_at: Timestamp,
    observation: CapturedObservation,
    summary: String,
}

impl Serialize for ChronicleEntry {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match &self.content {
            ChronicleContent::Legacy(legacy) if !legacy.event.is_legacy() => Err(
                serde::ser::Error::custom("observation event requires a version-two envelope"),
            ),
            ChronicleContent::Legacy(legacy) => LegacyChronicleEntry {
                id: self.id.clone(),
                occurred_at: self.occurred_at,
                adventurer: legacy.adventurer.clone(),
                campaign: legacy.campaign.clone(),
                pane: legacy.pane.clone(),
                pane_revision: legacy.pane_revision,
                event: legacy.event,
                summary: self.summary.clone(),
            }
            .serialize(serializer),
            ChronicleContent::Observation(observation) => RecordV2 {
                record_version: 2,
                id: self.id.clone(),
                observed_at: self.occurred_at,
                observation: *observation.clone(),
                summary: self.summary.clone(),
            }
            .serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for ChronicleEntry {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        if let Some(version) = value.get("record_version") {
            if version.as_u64() != Some(2) {
                return Err(serde::de::Error::custom(
                    "unsupported Chronicle record version",
                ));
            }
            let record: RecordV2 =
                serde_json::from_value(value).map_err(serde::de::Error::custom)?;
            if !record.observation.valid() || record.id != record.observation.event_id() {
                return Err(serde::de::Error::custom(
                    "invalid Chronicle observation evidence or identity",
                ));
            }
            Ok(Self {
                id: record.id,
                occurred_at: record.observed_at,
                summary: record.summary,
                content: ChronicleContent::Observation(Box::new(record.observation)),
            })
        } else {
            let legacy: LegacyChronicleEntry =
                serde_json::from_value(value).map_err(serde::de::Error::custom)?;
            if !legacy.event.is_legacy() {
                return Err(serde::de::Error::custom(
                    "observation event requires a version-two envelope",
                ));
            }
            Ok(legacy.into())
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChronicleEvent {
    AdventurerJoined,
    DelveBegan,
    CounselRequested,
    SpoilsReturned,
    AdventurerRested,
    AdventurerDeparted,
    CampaignClosed,
    PresenceObserved,
    WhereaboutsUnknown,
    AdventurerObserved,
    AdventurerNoLongerVisible,
    CampaignRemoved,
}

impl ChronicleEvent {
    /// What this event is worth to the guild's standing.
    ///
    /// Only work actually finished counts. An adventurer arriving, setting
    /// out or resting earns nothing: a score for having the plugin open would
    /// be a number invented to look like a game, which is what this vocabulary
    /// exists not to do.
    ///
    /// `CounselRequested` deliberately earns nothing either, though it is the
    /// event a guild master acts on most. It records an adventurer getting
    /// stuck, not anybody getting unstuck — paying for it would reward agents
    /// for blocking, which is precisely backwards.
    #[must_use]
    pub const fn experience(self) -> u64 {
        match self {
            Self::SpoilsReturned => 10,
            Self::CampaignClosed => 25,
            Self::AdventurerJoined
            | Self::DelveBegan
            | Self::CounselRequested
            | Self::AdventurerRested
            | Self::AdventurerDeparted
            | Self::PresenceObserved
            | Self::WhereaboutsUnknown
            | Self::AdventurerObserved
            | Self::AdventurerNoLongerVisible
            | Self::CampaignRemoved => 0,
        }
    }

    pub const ALL: &'static [Self] = &[
        Self::AdventurerJoined,
        Self::DelveBegan,
        Self::CounselRequested,
        Self::SpoilsReturned,
        Self::AdventurerRested,
        Self::AdventurerDeparted,
        Self::CampaignClosed,
        Self::PresenceObserved,
        Self::WhereaboutsUnknown,
        Self::AdventurerObserved,
        Self::AdventurerNoLongerVisible,
        Self::CampaignRemoved,
    ];

    /// Guild voice for the Chronicle view. Every event carries one, so an
    /// entry never renders as a bare enum name or an empty line.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::AdventurerJoined => "joined the guild",
            Self::DelveBegan => "set out on a delve",
            Self::CounselRequested => "requested counsel",
            Self::SpoilsReturned => "returned with spoils",
            Self::AdventurerRested => "rested at the hearth",
            Self::AdventurerDeparted => "departed",
            Self::CampaignClosed => "closed a campaign",
            Self::PresenceObserved => "presence was observed",
            Self::WhereaboutsUnknown => "whereabouts were unknown",
            Self::AdventurerObserved => "was first observed in the guild",
            Self::AdventurerNoLongerVisible => "was no longer visible in the guild",
            Self::CampaignRemoved => "campaign was no longer visible",
        }
    }

    /// A single-glyph mark, so a narrow Chronicle still distinguishes events.
    #[must_use]
    pub const fn sigil(self) -> char {
        match self {
            Self::AdventurerJoined | Self::AdventurerObserved => '+',
            Self::DelveBegan => '>',
            Self::CounselRequested => '!',
            Self::SpoilsReturned => '*',
            Self::AdventurerRested => 'z',
            Self::AdventurerDeparted | Self::AdventurerNoLongerVisible => '-',
            Self::CampaignClosed => '#',
            Self::PresenceObserved => '~',
            Self::WhereaboutsUnknown => '?',
            Self::CampaignRemoved => '/',
        }
    }

    pub const fn is_legacy(self) -> bool {
        matches!(
            self,
            Self::AdventurerJoined
                | Self::DelveBegan
                | Self::CounselRequested
                | Self::SpoilsReturned
                | Self::AdventurerRested
                | Self::AdventurerDeparted
                | Self::CampaignClosed
        )
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::AdventurerJoined => "adventurer_joined",
            Self::DelveBegan => "delve_began",
            Self::CounselRequested => "counsel_requested",
            Self::SpoilsReturned => "spoils_returned",
            Self::AdventurerRested => "adventurer_rested",
            Self::AdventurerDeparted => "adventurer_departed",
            Self::CampaignClosed => "campaign_closed",
            Self::PresenceObserved => "presence_observed",
            Self::WhereaboutsUnknown => "whereabouts_unknown",
            Self::AdventurerObserved => "adventurer_observed",
            Self::AdventurerNoLongerVisible => "adventurer_no_longer_visible",
            Self::CampaignRemoved => "campaign_removed",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Chronicle {
    maximum_entries: usize,
    entries: VecDeque<ChronicleEntry>,
    seen: BTreeSet<EventId>,
}

impl Chronicle {
    #[must_use]
    pub fn new(maximum_entries: usize) -> Self {
        Self {
            maximum_entries,
            entries: VecDeque::new(),
            seen: BTreeSet::new(),
        }
    }

    pub fn append(&mut self, entry: ChronicleEntry) -> bool {
        if self.seen.contains(&entry.id) {
            return false;
        }
        self.seen.insert(entry.id.clone());
        self.entries.push_back(entry);
        self.entries
            .make_contiguous()
            .sort_by_key(|entry| entry.occurred_at);
        while self.entries.len() > self.maximum_entries {
            if let Some(removed) = self.entries.pop_front() {
                self.seen.remove(&removed.id);
            }
        }
        true
    }

    #[must_use]
    pub fn entries(&self) -> &VecDeque<ChronicleEntry> {
        &self.entries
    }
}

impl Default for Chronicle {
    fn default() -> Self {
        Self::new(500)
    }
}
