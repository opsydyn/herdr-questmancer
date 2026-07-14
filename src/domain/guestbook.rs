use std::collections::{BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

use super::{AgentKey, EventId, PaneId, Timestamp, WorkspaceId};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GuestbookEntry {
    pub id: EventId,
    pub occurred_at: Timestamp,
    pub agent: Option<AgentKey>,
    pub workspace: Option<WorkspaceId>,
    pub pane: Option<PaneId>,
    pub pane_revision: u64,
    pub kind: GuestbookEvent,
    pub summary: String,
}

impl GuestbookEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        occurred_at: Timestamp,
        agent: Option<AgentKey>,
        workspace: Option<WorkspaceId>,
        pane: Option<PaneId>,
        pane_revision: u64,
        kind: GuestbookEvent,
        summary: impl Into<String>,
    ) -> Self {
        let identity = format!(
            "{}\0{}\0{}\0{}",
            kind.as_str(),
            pane.as_ref().map_or("-", PaneId::as_str),
            pane_revision,
            occurred_at.as_millis()
        );
        let hash = blake3::hash(identity.as_bytes()).to_hex();
        Self {
            id: EventId::new(format!("event-{}", &hash[..24])),
            occurred_at,
            agent,
            workspace,
            pane,
            pane_revision,
            kind,
            summary: summary.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GuestbookEvent {
    AgentDetected,
    WorkStarted,
    WebmasterNeeded,
    WorkCompleted,
    AgentBecameIdle,
    PaneExited,
    PaneClosed,
}

impl GuestbookEvent {
    const fn as_str(self) -> &'static str {
        match self {
            Self::AgentDetected => "agent_detected",
            Self::WorkStarted => "work_started",
            Self::WebmasterNeeded => "webmaster_needed",
            Self::WorkCompleted => "work_completed",
            Self::AgentBecameIdle => "agent_became_idle",
            Self::PaneExited => "pane_exited",
            Self::PaneClosed => "pane_closed",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Guestbook {
    maximum_entries: usize,
    entries: VecDeque<GuestbookEntry>,
    seen: BTreeSet<EventId>,
}

impl Guestbook {
    #[must_use]
    pub fn new(maximum_entries: usize) -> Self {
        Self {
            maximum_entries,
            entries: VecDeque::new(),
            seen: BTreeSet::new(),
        }
    }

    pub fn append(&mut self, entry: GuestbookEntry) -> bool {
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
    pub fn entries(&self) -> &VecDeque<GuestbookEntry> {
        &self.entries
    }
}

impl Default for Guestbook {
    fn default() -> Self {
        Self::new(500)
    }
}
