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
            | Self::AdventurerDeparted => 0,
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
        }
    }

    /// A single-glyph mark, so a narrow Chronicle still distinguishes events.
    #[must_use]
    pub const fn sigil(self) -> char {
        match self {
            Self::AdventurerJoined => '+',
            Self::DelveBegan => '>',
            Self::CounselRequested => '!',
            Self::SpoilsReturned => '*',
            Self::AdventurerRested => 'z',
            Self::AdventurerDeparted => '-',
            Self::CampaignClosed => '#',
        }
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
