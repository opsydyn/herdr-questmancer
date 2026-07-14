use serde::{Deserialize, Serialize};

use super::Timestamp;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AttentionReason {
    NeedsInput,
    WorkCompleted,
    PaneExited,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Attention {
    Clear,
    Unseen {
        reason: AttentionReason,
        since: Timestamp,
    },
    Seen {
        reason: AttentionReason,
        since: Timestamp,
    },
    Snoozed {
        reason: AttentionReason,
        since: Timestamp,
        until: Timestamp,
    },
}

impl Attention {
    pub const fn unseen(reason: AttentionReason, since: Timestamp) -> Self {
        Self::Unseen { reason, since }
    }

    #[must_use]
    pub fn mark_seen(self) -> Self {
        match self {
            Self::Unseen { reason, since } | Self::Snoozed { reason, since, .. } => {
                Self::Seen { reason, since }
            }
            attention => attention,
        }
    }

    pub const fn reason(&self) -> Option<AttentionReason> {
        match self {
            Self::Clear => None,
            Self::Unseen { reason, .. }
            | Self::Seen { reason, .. }
            | Self::Snoozed { reason, .. } => Some(*reason),
        }
    }

    pub const fn since(&self) -> Option<Timestamp> {
        match self {
            Self::Clear => None,
            Self::Unseen { since, .. } | Self::Seen { since, .. } | Self::Snoozed { since, .. } => {
                Some(*since)
            }
        }
    }

    pub const fn is_unseen(&self) -> bool {
        matches!(self, Self::Unseen { .. })
    }
}

impl Default for Attention {
    fn default() -> Self {
        Self::Clear
    }
}
