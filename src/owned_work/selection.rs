use crate::work::SourceId;
use serde::{Deserialize, Serialize};
use std::path::{Component, PathBuf};

/// A local intent pin, never work authority.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StoreSelection {
    pub path: PathBuf,
    pub source: SourceId,
}
impl StoreSelection {
    pub fn new(path: PathBuf, source: SourceId) -> Result<Self, super::CommandError> {
        let selection = Self { path, source };
        selection.validate()?;
        Ok(selection)
    }
    pub fn validate(&self) -> Result<(), super::CommandError> {
        if !self.path.is_absolute()
            || self
                .path
                .components()
                .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
            || self
                .path
                .to_str()
                .is_none_or(|s| s.len() > 4096 || s.chars().any(char::is_control))
        {
            return Err(super::CommandError::Invalid);
        }
        super::OwnedState::new(self.source.clone()).map(drop)
    }
}
