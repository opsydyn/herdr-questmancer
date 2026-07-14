use std::{fmt, time::Duration};

use serde::{Deserialize, Serialize};

macro_rules! string_id {
    ($name:ident) => {
        #[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::new(value)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }
    };
}

string_id!(WorkspaceId);
string_id!(TabId);
string_id!(PaneId);
string_id!(AgentKey);
string_id!(PersonaKey);
string_id!(EventId);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct Timestamp(i64);

impl Timestamp {
    pub const fn from_millis(milliseconds: i64) -> Self {
        Self(milliseconds)
    }

    pub const fn as_millis(self) -> i64 {
        self.0
    }

    pub fn elapsed_until(self, now: Self) -> Duration {
        let milliseconds = now.0.saturating_sub(self.0).max(0).cast_unsigned();
        Duration::from_millis(milliseconds)
    }
}
