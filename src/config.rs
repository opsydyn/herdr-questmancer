use std::{env, path::PathBuf};

use serde::Deserialize;
use thiserror::Error;

use crate::app::{CharacterSet, ColorMode, DisplayPreferences, Motion, RuntimeSettings, View};

/// A setting whose range is part of its meaning.
///
/// These were bare numbers validated once, in `TryFrom<ConfigFile>`, and then
/// carried as `u32` and `usize` everywhere afterwards. The invariant existed
/// for exactly the width of that one function and was forgotten by the type,
/// so anything downstream could hold a value the config loader would have
/// rejected. Parsing into a type that cannot be out of range keeps the promise
/// where the value goes.
macro_rules! bounded_setting {
    ($name:ident, $inner:ty, $low:expr, $high:expr, $default:expr, $message:literal) => {
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        pub struct $name($inner);

        impl $name {
            pub const RANGE: std::ops::RangeInclusive<$inner> = $low..=$high;

            /// The only way to make one, so the range travels with the value.
            pub fn new(value: $inner) -> Result<Self, ConfigError> {
                if Self::RANGE.contains(&value) {
                    Ok(Self(value))
                } else {
                    Err(ConfigError::Validation($message))
                }
            }

            #[must_use]
            pub const fn get(self) -> $inner {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self($default)
            }
        }

        impl TryFrom<$inner> for $name {
            type Error = ConfigError;

            fn try_from(value: $inner) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }
    };
}

bounded_setting!(
    OutputPreviewLines,
    u32,
    10,
    500,
    80,
    "output_preview_lines must be between 10 and 500"
);
bounded_setting!(
    ChronicleMaxEntries,
    usize,
    50,
    10_000,
    500,
    "chronicle_max_entries must be between 50 and 10000"
);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuestmancerConfig {
    pub default_view: View,
    pub preferences: DisplayPreferences,
    pub output_preview_lines: OutputPreviewLines,
    pub chronicle_max_entries: ChronicleMaxEntries,
    pub reviewr_action: String,
    pub show_elapsed_time: bool,
    pub sidebar_urgency_order: bool,
}

impl Default for QuestmancerConfig {
    fn default() -> Self {
        Self {
            default_view: View::Guild,
            preferences: DisplayPreferences::default(),
            output_preview_lines: OutputPreviewLines::default(),
            chronicle_max_entries: ChronicleMaxEntries::default(),
            reviewr_action: "persiyanov.reviewr.open".to_owned(),
            show_elapsed_time: true,
            sidebar_urgency_order: false,
        }
    }
}

impl QuestmancerConfig {
    pub fn parse(bytes: &[u8]) -> Result<Self, ConfigError> {
        toml::from_slice::<ConfigFile>(bytes)?.try_into()
    }

    pub fn runtime_settings(&self) -> RuntimeSettings {
        RuntimeSettings {
            sidebar_urgency_order: self.sidebar_urgency_order,
            output_preview_lines: self.output_preview_lines,
            reviewr_action: self.reviewr_action.clone(),
            show_elapsed_time: self.show_elapsed_time,
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid questmancer configuration: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("invalid questmancer configuration: {0}")]
    Validation(&'static str),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PersistencePaths {
    config_dir: Option<PathBuf>,
    state_dir: Option<PathBuf>,
}

impl PersistencePaths {
    pub fn from_env() -> Self {
        Self::from_lookup(|name| env::var(name).ok())
    }

    pub fn from_lookup(mut lookup: impl FnMut(&str) -> Option<String>) -> Self {
        Self {
            config_dir: lookup("HERDR_PLUGIN_CONFIG_DIR").map(PathBuf::from),
            state_dir: lookup("HERDR_PLUGIN_STATE_DIR").map(PathBuf::from),
        }
    }

    pub fn config_path(&self) -> Option<PathBuf> {
        self.config_dir
            .as_ref()
            .map(|path| path.join("config.toml"))
    }

    pub fn state_path(&self) -> Option<PathBuf> {
        self.state_dir.as_ref().map(|path| path.join("state.json"))
    }

    pub fn chronicle_path(&self) -> Option<PathBuf> {
        self.state_dir
            .as_ref()
            .map(|path| path.join("chronicle.jsonl"))
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct ConfigFile {
    default_view: View,
    motion: Motion,
    character_set: CharacterSet,
    color_mode: ColorMode,
    output_preview_lines: u32,
    chronicle_max_entries: usize,
    reviewr_action: String,
    show_elapsed_time: bool,
    /// Opt-in: let Questmancer order Herdr's own agent list by urgency.
    ///
    /// Off by default because it changes shared Herdr UI rather than anything
    /// inside Questmancer's pane. The sidebar belongs to the user.
    sidebar_urgency_order: bool,
}

impl Default for ConfigFile {
    fn default() -> Self {
        let config = QuestmancerConfig::default();
        Self {
            default_view: config.default_view,
            motion: config.preferences.motion,
            character_set: config.preferences.character_set,
            color_mode: config.preferences.color_mode,
            output_preview_lines: config.output_preview_lines.get(),
            chronicle_max_entries: config.chronicle_max_entries.get(),
            reviewr_action: config.reviewr_action,
            show_elapsed_time: config.show_elapsed_time,
            sidebar_urgency_order: config.sidebar_urgency_order,
        }
    }
}

impl TryFrom<ConfigFile> for QuestmancerConfig {
    type Error = ConfigError;

    fn try_from(file: ConfigFile) -> Result<Self, Self::Error> {
        if file.reviewr_action.trim().is_empty() {
            return Err(ConfigError::Validation("reviewr_action must not be empty"));
        }

        Ok(Self {
            default_view: file.default_view,
            preferences: DisplayPreferences {
                motion: file.motion,
                character_set: file.character_set,
                color_mode: file.color_mode,
            },
            output_preview_lines: OutputPreviewLines::new(file.output_preview_lines)?,
            chronicle_max_entries: ChronicleMaxEntries::new(file.chronicle_max_entries)?,
            reviewr_action: file.reviewr_action,
            show_elapsed_time: file.show_elapsed_time,
            sidebar_urgency_order: file.sidebar_urgency_order,
        })
    }
}
