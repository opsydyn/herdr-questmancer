use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};

use thiserror::Error;
use tokio::io::AsyncWriteExt;

use super::PersistedStateV1;

#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error("{operation} {path:?}: {source_message}")]
pub struct PersistenceDiagnostic {
    pub operation: &'static str,
    pub path: std::path::PathBuf,
    pub line: Option<usize>,
    pub source_message: String,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error("{operation} {path:?}: {source_message}")]
pub struct PersistenceError {
    pub operation: &'static str,
    pub path: PathBuf,
    pub line: Option<usize>,
    pub source_message: String,
}

pub fn parse_state(path: &Path, bytes: &[u8]) -> Result<PersistedStateV1, PersistenceDiagnostic> {
    let state = serde_json::from_slice::<PersistedStateV1>(bytes).map_err(|error| {
        PersistenceDiagnostic {
            operation: "parse state",
            path: path.to_owned(),
            line: (error.line() > 0).then(|| error.line()),
            source_message: error.to_string(),
        }
    })?;
    state.validate().map_err(|error| PersistenceDiagnostic {
        operation: "validate state",
        path: path.to_owned(),
        line: None,
        source_message: error.to_string(),
    })?;
    Ok(state)
}

pub async fn load_state(path: &Path) -> Result<Option<PersistedStateV1>, PersistenceDiagnostic> {
    let bytes = match tokio::fs::read(path).await {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(PersistenceDiagnostic {
                operation: "read state",
                path: path.to_owned(),
                line: None,
                source_message: error.to_string(),
            });
        }
    };
    parse_state(path, &bytes).map(Some)
}

pub async fn publish_state(path: &Path, state: &PersistedStateV1) -> Result<(), PersistenceError> {
    let mut bytes = serde_json::to_vec_pretty(state).map_err(|error| PersistenceError {
        operation: "serialize state",
        path: path.to_owned(),
        line: None,
        source_message: error.to_string(),
    })?;
    bytes.push(b'\n');

    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    tokio::fs::create_dir_all(parent)
        .await
        .map_err(|error| PersistenceError {
            operation: "create state directory",
            path: parent.to_owned(),
            line: None,
            source_message: error.to_string(),
        })?;

    let temporary_path = temporary_path(path);
    let publication = async {
        let mut file = tokio::fs::File::create(&temporary_path)
            .await
            .map_err(|error| PersistenceError {
                operation: "create temporary state",
                path: temporary_path.clone(),
                line: None,
                source_message: error.to_string(),
            })?;
        file.write_all(&bytes)
            .await
            .map_err(|error| PersistenceError {
                operation: "write temporary state",
                path: temporary_path.clone(),
                line: None,
                source_message: error.to_string(),
            })?;
        file.sync_all().await.map_err(|error| PersistenceError {
            operation: "sync temporary state",
            path: temporary_path.clone(),
            line: None,
            source_message: error.to_string(),
        })?;
        drop(file);
        tokio::fs::rename(&temporary_path, path)
            .await
            .map_err(|error| PersistenceError {
                operation: "rename state",
                path: path.to_owned(),
                line: None,
                source_message: error.to_string(),
            })
    }
    .await;

    if let Err(error) = publication {
        let _ = tokio::fs::remove_file(&temporary_path).await;
        return Err(error);
    }

    let parent = parent.to_owned();
    let _ = tokio::task::spawn_blocking(move || {
        std::fs::File::open(parent).and_then(|directory| directory.sync_all())
    })
    .await;
    Ok(())
}

fn temporary_path(path: &Path) -> PathBuf {
    let mut temporary = path.as_os_str().to_os_string();
    temporary.push(".tmp");
    temporary.into()
}
