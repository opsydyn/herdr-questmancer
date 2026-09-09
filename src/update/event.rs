use crate::{
    domain::{AgentKey, ChronicleEntry, PaneId, Timestamp, WorkspaceId},
    herdr::protocol::{AgentStatus, SessionSnapshot},
};

#[derive(Clone, Debug, PartialEq)]
pub enum AppEvent {
    SnapshotReplaced {
        purpose: crate::snapshot_refresh::SnapshotPurpose,
        snapshot: Box<SessionSnapshot>,
        observed_at: Timestamp,
        excluded_pane: Option<PaneId>,
    },
    AgentStatusChanged {
        identity: crate::domain::CaptureIdentity,
        pane_id: PaneId,
        status: AgentStatus,
        custom_status: Option<String>,
        revision: u64,
        occurred_at: Timestamp,
    },
    PaneExited {
        identity: crate::domain::CaptureIdentity,
        pane_id: PaneId,
        revision: u64,
        occurred_at: Timestamp,
    },
    WorkspaceCloseHint(WorkspaceId),
    DeferSummons {
        agent_key: crate::domain::AgentKey,
        until: Timestamp,
    },
    MarkRead(AgentKey),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
    RequestSnapshot,
    AppendChronicle(ChronicleEntry),
    PersistState,
}

impl Command {
    #[must_use]
    pub const fn is_chronicle_append(&self) -> bool {
        matches!(self, Self::AppendChronicle(_))
    }
}
