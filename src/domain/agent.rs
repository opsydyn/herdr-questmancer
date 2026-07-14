use serde::{Deserialize, Serialize};

use crate::herdr::protocol::AgentStatus;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Presence {
    Working,
    Blocked,
    Done,
    Idle,
    Exited,
    Unknown,
}

impl From<AgentStatus> for Presence {
    fn from(status: AgentStatus) -> Self {
        match status {
            AgentStatus::Working => Self::Working,
            AgentStatus::Blocked => Self::Blocked,
            AgentStatus::Done => Self::Done,
            AgentStatus::Idle => Self::Idle,
            AgentStatus::Unknown => Self::Unknown,
        }
    }
}
