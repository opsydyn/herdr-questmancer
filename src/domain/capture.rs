//! Ephemeral capture ownership and incarnation evidence. No persisted live topology.
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::WorkspaceId;
use crate::{herdr::protocol::AgentSessionInfo, snapshot_refresh::ConnectionEpoch};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct CaptureRunId(String);

impl CaptureRunId {
    /// Supplied by the runtime; deterministic callers can supply a fixed test ID.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A complete session tuple plus its terminal. Persona fallback keys alone are
/// insufficient evidence that two reports concern the same running adventurer.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum CaptureIdentity {
    #[default]
    Unqualified,
    /// Terminal evidence still invalidates per-pane I/O, but cannot qualify
    /// an adventurer observation without a complete session identity.
    Terminal {
        terminal: String,
        session: Option<AgentSessionInfo>,
    },
    Session {
        terminal: String,
        session: AgentSessionInfo,
    },
}

impl CaptureIdentity {
    pub fn from_parts(terminal: &str, session: Option<&AgentSessionInfo>) -> Self {
        if terminal.trim().is_empty() {
            return Self::Unqualified;
        }
        let Some(session) = session.filter(|session| {
            [
                &session.source,
                &session.agent,
                &session.kind,
                &session.value,
            ]
            .iter()
            .all(|part| !part.trim().is_empty())
        }) else {
            return Self::Terminal {
                terminal: terminal.to_owned(),
                session: session.cloned(),
            };
        };
        Self::Session {
            terminal: terminal.to_owned(),
            session: session.clone(),
        }
    }

    pub const fn is_qualified(&self) -> bool {
        matches!(self, Self::Session { .. })
    }

    pub fn same_incarnation(&self, other: &Self) -> bool {
        self.is_qualified() && self == other
    }

    /// Only a digest is written to history. Raw session metadata stays ephemeral.
    pub fn fingerprint(&self) -> Option<String> {
        let Self::Session { terminal, session } = self else {
            return None;
        };
        let mut hash = blake3::Hasher::new();
        hash.update(b"questmancer-incarnation-v1");
        for part in [
            terminal,
            &session.source,
            &session.agent,
            &session.kind,
            &session.value,
        ] {
            hash.update(&(part.len() as u64).to_le_bytes());
            hash.update(part.as_bytes());
        }
        Some(hash.finalize().to_hex().to_string())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ObservationStamp {
    pub run: CaptureRunId,
    pub epoch: ConnectionEpoch,
    pub ordinal: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CaptureClock {
    run: Option<CaptureRunId>,
    epoch: ConnectionEpoch,
    connected: bool,
    ordinal: u64,
    close_hints: BTreeSet<WorkspaceId>,
}

impl CaptureClock {
    pub fn start(&mut self, run: CaptureRunId) {
        self.run = (!run.as_str().trim().is_empty()).then_some(run);
        self.ordinal = 0;
        self.close_hints.clear();
    }

    pub fn connection_changed(&mut self, epoch: ConnectionEpoch, connected: bool) {
        self.epoch = epoch;
        self.connected = connected;
        self.close_hints.clear();
    }

    pub fn qualifies(&self, epoch: ConnectionEpoch) -> bool {
        self.connected && self.run.is_some() && self.epoch == epoch
    }

    /// Commit the ordinal only when the record is accepted. Validation failures
    /// and duplicate records consume no identity from this capture run.
    pub(crate) fn accept<T>(
        &mut self,
        record: impl FnOnce(ObservationStamp) -> Option<T>,
    ) -> Option<T> {
        if !self.connected {
            return None;
        }
        let run = self.run.clone()?;
        let ordinal = self.ordinal.checked_add(1)?;
        let accepted = record(ObservationStamp {
            run,
            epoch: self.epoch,
            ordinal,
        })?;
        self.ordinal = ordinal;
        Some(accepted)
    }

    pub fn note_close(&mut self, workspace: WorkspaceId) {
        if self.connected {
            self.close_hints.insert(workspace);
        }
    }
    pub fn close_was_hinted(&self, workspace: &WorkspaceId) -> bool {
        self.close_hints.contains(workspace)
    }
    pub fn clear_hints(&mut self) {
        self.close_hints.clear();
    }
}
