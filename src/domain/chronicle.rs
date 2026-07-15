use std::collections::{BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

use super::{AgentKey, EventId, PaneId, Timestamp, WorkspaceId};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ChronicleEntry {
    pub id: EventId,
    pub occurred_at: Timestamp,
    pub adventurer: Option<AgentKey>,
    pub campaign: Option<WorkspaceId>,
    pub pane: Option<PaneId>,
    pub pane_revision: u64,
    pub event: ChronicleEvent,
    pub summary: String,
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
        Self {
            id: EventId::new(format!("event-{}", &hash[..24])),
            occurred_at,
            adventurer,
            campaign,
            pane,
            pane_revision,
            event,
            summary: summary.into(),
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
}

impl ChronicleEvent {
    const fn as_str(self) -> &'static str {
        match self {
            Self::AdventurerJoined => "adventurer_joined",
            Self::DelveBegan => "delve_began",
            Self::CounselRequested => "counsel_requested",
            Self::SpoilsReturned => "spoils_returned",
            Self::AdventurerRested => "adventurer_rested",
            Self::AdventurerDeparted => "adventurer_departed",
            Self::CampaignClosed => "campaign_closed",
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
