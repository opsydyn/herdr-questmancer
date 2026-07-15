use std::{env, path::PathBuf};

use serde::Deserialize;
use thiserror::Error;

use crate::app::{CharacterSet, ColorMode, DisplayPreferences, Motion, RuntimeSettings, View};

const OUTPUT_PREVIEW_LINES_RANGE: std::ops::RangeInclusive<u32> = 10..=500;
const GUESTBOOK_MAX_ENTRIES_RANGE: std::ops::RangeInclusive<usize> = 50..=10_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WebmasterConfig {
    pub default_view: View,
    pub preferences: DisplayPreferences,
    pub output_preview_lines: u32,
    pub guestbook_max_entries: usize,
    pub reviewr_action: String,
    pub show_elapsed_time: bool,
}

impl Default for WebmasterConfig {
    fn default() -> Self {
        Self {
            default_view: View::Guild,
            preferences: DisplayPreferences::default(),
            output_preview_lines: 80,
            guestbook_max_entries: 500,
            reviewr_action: "persiyanov.reviewr.open".to_owned(),
            show_elapsed_time: true,
        }
    }
}

impl WebmasterConfig {
    pub fn parse(bytes: &[u8]) -> Result<Self, ConfigError> {
        toml::from_slice::<ConfigFile>(bytes)?.try_into()
    }

    pub fn runtime_settings(&self) -> RuntimeSettings {
        RuntimeSettings {
            output_preview_lines: self.output_preview_lines,
            reviewr_action: self.reviewr_action.clone(),
            show_elapsed_time: self.show_elapsed_time,
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid webmaster configuration: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("invalid webmaster configuration: {0}")]
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

    pub fn guestbook_path(&self) -> Option<PathBuf> {
        self.state_dir
            .as_ref()
            .map(|path| path.join("guestbook.jsonl"))
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
    guestbook_max_entries: usize,
    reviewr_action: String,
    show_elapsed_time: bool,
}

impl Default for ConfigFile {
    fn default() -> Self {
        let config = WebmasterConfig::default();
        Self {
            default_view: config.default_view,
            motion: config.preferences.motion,
            character_set: config.preferences.character_set,
            color_mode: config.preferences.color_mode,
            output_preview_lines: config.output_preview_lines,
            guestbook_max_entries: config.guestbook_max_entries,
            reviewr_action: config.reviewr_action,
            show_elapsed_time: config.show_elapsed_time,
        }
    }
}

impl TryFrom<ConfigFile> for WebmasterConfig {
    type Error = ConfigError;

    fn try_from(file: ConfigFile) -> Result<Self, Self::Error> {
        if !OUTPUT_PREVIEW_LINES_RANGE.contains(&file.output_preview_lines) {
            return Err(ConfigError::Validation(
                "output_preview_lines must be between 10 and 500",
            ));
        }
        if !GUESTBOOK_MAX_ENTRIES_RANGE.contains(&file.guestbook_max_entries) {
            return Err(ConfigError::Validation(
                "guestbook_max_entries must be between 50 and 10000",
            ));
        }
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
            output_preview_lines: file.output_preview_lines,
            guestbook_max_entries: file.guestbook_max_entries,
            reviewr_action: file.reviewr_action,
            show_elapsed_time: file.show_elapsed_time,
        })
    }
}
