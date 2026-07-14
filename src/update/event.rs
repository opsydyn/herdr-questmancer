use crate::{
    domain::{AgentKey, GuestbookEntry, PaneId, Timestamp, WorkspaceId},
    herdr::protocol::{AgentStatus, SessionSnapshot},
};

#[derive(Clone, Debug, PartialEq)]
pub enum AppEvent {
    SnapshotReplaced {
        snapshot: SessionSnapshot,
        observed_at: Timestamp,
    },
    AgentStatusChanged {
        pane_id: PaneId,
        status: AgentStatus,
        custom_status: Option<String>,
        revision: u64,
        occurred_at: Timestamp,
    },
    PaneExited {
        pane_id: PaneId,
        revision: u64,
        occurred_at: Timestamp,
    },
    WorkspaceClosed(WorkspaceId),
    MarkSeen(AgentKey),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
    RequestSnapshot,
    AppendGuestbook(GuestbookEntry),
    PersistState,
}

impl Command {
    #[must_use]
    pub const fn is_guestbook_append(&self) -> bool {
        matches!(self, Self::AppendGuestbook(_))
    }
}
