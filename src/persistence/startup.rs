use std::{io::ErrorKind, path::PathBuf};

use crate::{
    app::{Model, View},
    config::{PersistencePaths, WebmasterConfig},
};

use super::{PersistenceDiagnostic, WorkerPaths, parse_state, replay_guestbook};

#[derive(Debug)]
pub struct StartupData {
    pub model: Model,
    pub paths: WorkerPaths,
    pub diagnostics: Vec<PersistenceDiagnostic>,
}

pub async fn load_startup(paths: PersistencePaths, explicit_view: Option<View>) -> StartupData {
    let config_path = paths.config_path();
    let state_path = paths.state_path();
    let guestbook_path = paths.guestbook_path();
    let worker_paths = WorkerPaths::new(state_path.clone(), guestbook_path.clone());

    let (config_read, state_read, guestbook_read) = tokio::join!(
        read_optional(config_path, "read config"),
        read_optional(state_path, "read state"),
        read_optional(guestbook_path, "read guestbook"),
    );

    let mut diagnostics = Vec::new();
    let config = interpret_config(config_read, &mut diagnostics);
    let persisted = interpret_state(state_read, &mut diagnostics);
    let replay = interpret_guestbook(
        guestbook_read,
        config.guestbook_max_entries,
        &mut diagnostics,
    );

    let view = effective_view(
        explicit_view,
        persisted.as_ref().map(|state| state.last_view),
        config.default_view,
    );
    let settings = config.runtime_settings();
    let mut model = Model::new(view);
    model.set_preferences(
        persisted
            .as_ref()
            .map_or(config.preferences, |state| state.preferences),
    );
    model.set_settings(settings.clone());
    if let Some(state) = &persisted {
        let seeded = model.durable_intent_mut().seed(state);
        debug_assert!(seeded.is_ok());
    }
    model.domain_mut().guestbook = replay;

    StartupData {
        model,
        paths: worker_paths,
        diagnostics,
    }
}

pub const fn effective_view(
    explicit: Option<View>,
    persisted: Option<View>,
    configured: View,
) -> View {
    match (explicit, persisted) {
        (Some(view), _) | (None, Some(view)) => view,
        (None, None) => configured,
    }
}

#[derive(Debug)]
struct RawRead {
    path: Option<PathBuf>,
    bytes: Option<Vec<u8>>,
    diagnostic: Option<PersistenceDiagnostic>,
}

async fn read_optional(path: Option<PathBuf>, operation: &'static str) -> RawRead {
    let Some(path) = path else {
        return RawRead {
            path: None,
            bytes: None,
            diagnostic: None,
        };
    };
    match tokio::fs::read(&path).await {
        Ok(bytes) => RawRead {
            path: Some(path),
            bytes: Some(bytes),
            diagnostic: None,
        },
        Err(error) if error.kind() == ErrorKind::NotFound => RawRead {
            path: Some(path),
            bytes: None,
            diagnostic: None,
        },
        Err(error) => RawRead {
            path: Some(path.clone()),
            bytes: None,
            diagnostic: Some(PersistenceDiagnostic {
                operation,
                path,
                line: None,
                source_message: error.to_string(),
            }),
        },
    }
}

fn interpret_config(
    read: RawRead,
    diagnostics: &mut Vec<PersistenceDiagnostic>,
) -> WebmasterConfig {
    if let Some(diagnostic) = read.diagnostic {
        diagnostics.push(diagnostic);
        return WebmasterConfig::default();
    }
    let (Some(path), Some(bytes)) = (read.path, read.bytes) else {
        return WebmasterConfig::default();
    };
    match WebmasterConfig::parse(&bytes) {
        Ok(config) => config,
        Err(error) => {
            diagnostics.push(PersistenceDiagnostic {
                operation: "parse config",
                path,
                line: None,
                source_message: error.to_string(),
            });
            WebmasterConfig::default()
        }
    }
}

fn interpret_state(
    read: RawRead,
    diagnostics: &mut Vec<PersistenceDiagnostic>,
) -> Option<super::PersistedStateV1> {
    if let Some(diagnostic) = read.diagnostic {
        diagnostics.push(diagnostic);
        return None;
    }
    let (Some(path), Some(bytes)) = (read.path, read.bytes) else {
        return None;
    };
    match parse_state(&path, &bytes) {
        Ok(state) => Some(state),
        Err(diagnostic) => {
            diagnostics.push(diagnostic);
            None
        }
    }
}

fn interpret_guestbook(
    read: RawRead,
    maximum_entries: usize,
    diagnostics: &mut Vec<PersistenceDiagnostic>,
) -> crate::domain::Guestbook {
    if let Some(diagnostic) = read.diagnostic {
        diagnostics.push(diagnostic);
        return crate::domain::Guestbook::new(maximum_entries);
    }
    let (Some(path), Some(bytes)) = (read.path, read.bytes) else {
        return crate::domain::Guestbook::new(maximum_entries);
    };
    let replay = replay_guestbook(&path, &bytes, maximum_entries);
    diagnostics.extend(replay.diagnostics);
    replay.guestbook
}
