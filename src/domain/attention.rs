use serde::{Deserialize, Serialize};

use super::Timestamp;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GuildSummons {
    CounselRequested,
    SpoilsReturned,
    AdventurerDeparted,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GuildAttention {
    Clear,
    Unread {
        summons: GuildSummons,
        since: Timestamp,
    },
    Read {
        summons: GuildSummons,
        since: Timestamp,
    },
    Deferred {
        summons: GuildSummons,
        since: Timestamp,
        until: Timestamp,
    },
}

impl GuildAttention {
    pub const fn unread(summons: GuildSummons, since: Timestamp) -> Self {
        Self::Unread { summons, since }
    }

    #[must_use]
    pub fn mark_read(self) -> Self {
        match self {
            Self::Unread { summons, since } | Self::Deferred { summons, since, .. } => {
                Self::Read { summons, since }
            }
            attention => attention,
        }
    }

    pub const fn summons(&self) -> Option<GuildSummons> {
        match self {
            Self::Clear => None,
            Self::Unread { summons, .. }
            | Self::Read { summons, .. }
            | Self::Deferred { summons, .. } => Some(*summons),
        }
    }

    pub const fn since(&self) -> Option<Timestamp> {
        match self {
            Self::Clear => None,
            Self::Unread { since, .. }
            | Self::Read { since, .. }
            | Self::Deferred { since, .. } => Some(*since),
        }
    }

    /// Sets a summons aside until `until`.
    ///
    /// This variant existed from the start and nothing ever built it: the
    /// reducer could mark a summons read, the urgency jump was already written
    /// to skip deferred ones, and no control could put an adventurer into the
    /// state. Deferring keeps the summons and its original `since`, so the
    /// Hall still shows the adventurer needs counsel — it says "not now",
    /// not "handled".
    #[must_use]
    pub fn defer_until(self, until: Timestamp) -> Self {
        match self {
            Self::Unread { summons, since }
            | Self::Read { summons, since }
            | Self::Deferred { summons, since, .. } => Self::Deferred {
                summons,
                since,
                until,
            },
            Self::Clear => Self::Clear,
        }
    }

    /// Whether this summons is currently set aside.
    #[must_use]
    pub fn is_deferred_at(&self, now: Timestamp) -> bool {
        matches!(self, Self::Deferred { until, .. } if until.as_millis() > now.as_millis())
    }

    pub const fn is_unread(&self) -> bool {
        matches!(self, Self::Unread { .. })
    }
}

impl Default for GuildAttention {
    fn default() -> Self {
        Self::Clear
    }
}
