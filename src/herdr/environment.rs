use std::{env, path::PathBuf};

use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HerdrEnvironment {
    socket_path: PathBuf,
    bin_path: PathBuf,
}

impl HerdrEnvironment {
    pub fn new(socket_path: impl Into<PathBuf>, bin_path: impl Into<PathBuf>) -> Self {
        Self {
            socket_path: socket_path.into(),
            bin_path: bin_path.into(),
        }
    }

    pub fn from_env() -> Result<Self, EnvironmentError> {
        Self::from_lookup(|name| env::var(name).ok())
    }

    pub fn from_lookup(
        mut lookup: impl FnMut(&str) -> Option<String>,
    ) -> Result<Self, EnvironmentError> {
        let socket_path = lookup("HERDR_SOCKET_PATH")
            .filter(|value| !value.is_empty())
            .ok_or(EnvironmentError::Missing("HERDR_SOCKET_PATH"))?;
        let bin_path = lookup("HERDR_BIN_PATH")
            .filter(|value| !value.is_empty())
            .ok_or(EnvironmentError::Missing("HERDR_BIN_PATH"))?;

        Ok(Self::new(socket_path, bin_path))
    }

    pub fn socket_path(&self) -> PathBuf {
        self.socket_path.clone()
    }

    pub fn bin_path(&self) -> PathBuf {
        self.bin_path.clone()
    }
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum EnvironmentError {
    #[error("required plugin environment variable {0} is missing")]
    Missing(&'static str),
}
