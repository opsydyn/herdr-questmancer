//! Ephemeral ownership of snapshot observations. Never persisted with local intent.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ConnectionEpoch(pub u64);

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct SnapshotRequestId(pub u64);

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct LiveFactGeneration(pub u64);

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct SnapshotRequest {
    pub epoch: ConnectionEpoch,
    pub id: SnapshotRequestId,
    pub generation: LiveFactGeneration,
}

/// Both purposes are quiet in C1. A refresh is eligible for future observation
/// capture only after its owner has checked correlation and live-fact freshness.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum SnapshotPurpose {
    #[default]
    Baseline,
    Refresh(SnapshotRequest),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SnapshotCompletion {
    Ignore,
    Apply,
    Retry(SnapshotRequest),
    Resubscribe,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct SnapshotRefresh {
    epoch: ConnectionEpoch,
    generation: LiveFactGeneration,
    next_id: u64,
    connected: bool,
    subscribed_panes: Option<BTreeSet<String>>,
    active: Option<SnapshotRequest>,
    pending: bool,
    superseded: u8,
}

impl SnapshotRefresh {
    pub(crate) const fn epoch(&self) -> ConnectionEpoch {
        self.epoch
    }
    pub(crate) fn connection_changed(&mut self, connected: bool) {
        self.epoch.0 = self.epoch.0.wrapping_add(1);
        self.generation = LiveFactGeneration::default();
        self.connected = connected;
        self.subscribed_panes = None;
        self.active = None;
        self.pending = false;
        self.superseded = 0;
    }

    pub(crate) fn baseline(&mut self, snapshot: &crate::herdr::protocol::SessionSnapshot) {
        self.subscribed_panes = Some(crate::herdr::supervisor::pane_subscription_ids(snapshot));
    }

    pub(crate) fn subscriptions_match(
        &self,
        snapshot: &crate::herdr::protocol::SessionSnapshot,
    ) -> bool {
        self.subscribed_panes.as_ref()
            == Some(&crate::herdr::supervisor::pane_subscription_ids(snapshot))
    }

    pub(crate) fn facts_changed(&mut self) {
        self.generation.0 = self.generation.0.wrapping_add(1);
    }

    pub(crate) fn request(&mut self) -> Option<SnapshotRequest> {
        if !self.connected || self.subscribed_panes.is_none() {
            return None;
        }
        if self.active.is_some() {
            self.pending = true;
            return None;
        }
        self.next_id = self.next_id.wrapping_add(1);
        let request = SnapshotRequest {
            epoch: self.epoch,
            id: SnapshotRequestId(self.next_id),
            generation: self.generation,
        };
        self.active = Some(request);
        Some(request)
    }

    fn owns(&self, request: SnapshotRequest) -> bool {
        self.connected && self.active == Some(request) && self.epoch == request.epoch
    }

    pub(crate) fn complete(
        &mut self,
        request: SnapshotRequest,
        valid: bool,
        subscribed: bool,
    ) -> SnapshotCompletion {
        if !self.owns(request) {
            return SnapshotCompletion::Ignore;
        }
        if !subscribed {
            self.connection_changed(false);
            return SnapshotCompletion::Resubscribe;
        }
        self.active = None;
        if valid && request.generation == self.generation {
            self.superseded = 0;
            return SnapshotCompletion::Apply;
        }
        self.pending = false;
        self.superseded += 1;
        if self.superseded >= 2 {
            self.connection_changed(false);
            SnapshotCompletion::Resubscribe
        } else {
            SnapshotCompletion::Retry(self.request().expect("owned request was connected"))
        }
    }

    pub(crate) fn fail(&mut self, request: SnapshotRequest) -> bool {
        if !self.owns(request) {
            return false;
        }
        self.active = None;
        self.superseded = 0;
        true
    }

    /// Called after installing accepted facts, so a pending request uses their
    /// generation rather than immediately superseding itself on completion.
    pub(crate) fn take_pending(&mut self) -> Option<SnapshotRequest> {
        if std::mem::take(&mut self.pending) {
            self.request()
        } else {
            None
        }
    }
}

/// A transient comparison of reducer-owned live facts, not a second stored model.
#[derive(Eq, PartialEq)]
pub(crate) struct LiveFacts {
    agents: Vec<AgentFacts>,
    campaigns: Vec<crate::domain::WorkspaceId>,
}

#[derive(Eq, PartialEq)]
struct AgentFacts {
    key: crate::domain::AgentKey,
    pane: crate::domain::PaneId,
    workspace: crate::domain::WorkspaceId,
    tab: crate::domain::TabId,
    presence: crate::domain::Presence,
    revision: u64,
    identity: crate::domain::CaptureIdentity,
}

impl LiveFacts {
    pub(crate) fn from_domain(domain: &crate::domain::DomainState) -> Self {
        Self {
            agents: domain
                .agents
                .values()
                .map(|agent| AgentFacts {
                    key: agent.key.clone(),
                    pane: agent.pane_id.clone(),
                    workspace: agent.workspace_id.clone(),
                    tab: agent.tab_id.clone(),
                    presence: agent.presence,
                    revision: agent.pane_revision,
                    identity: agent.capture_identity.clone(),
                })
                .collect(),
            campaigns: domain.campaigns.keys().cloned().collect(),
        }
    }
}
