use std::{io::ErrorKind, path::PathBuf};

use crate::{
    app::{Model, View},
    config::{PersistencePaths, QuestmancerConfig},
};

use super::{PersistenceDiagnostic, WorkerPaths, parse_state, replay_chronicle};

#[derive(Debug)]
pub struct StartupData {
    pub model: Model,
    pub paths: WorkerPaths,
    pub diagnostics: Vec<PersistenceDiagnostic>,
}

pub async fn load_startup(paths: PersistencePaths, explicit_view: Option<View>) -> StartupData {
    let config_path = paths.config_path();
    let state_path = paths.state_path();
    let chronicle_path = paths.chronicle_path();

    let (config_read, state_read, chronicle_read) = tokio::join!(
        read_optional(config_path, "read config"),
        read_optional(state_path.clone(), "read state"),
        read_optional(chronicle_path.clone(), "read chronicle"),
    );

    let mut diagnostics = Vec::new();
    let config = interpret_config(config_read, &mut diagnostics);
    let state_load = interpret_state(state_read, &mut diagnostics);
    let worker_state_path = if matches!(state_load, StateLoad::Protected) {
        None
    } else {
        state_path
    };
    let persisted = match state_load {
        StateLoad::Loaded(state) => Some(state),
        StateLoad::Missing | StateLoad::Protected => None,
    };
    let worker_paths = WorkerPaths::new(worker_state_path, chronicle_path);
    let replay = interpret_chronicle(
        chronicle_read,
        config.chronicle_max_entries,
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
    model.domain_mut().chronicle = replay;

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
) -> QuestmancerConfig {
    if let Some(diagnostic) = read.diagnostic {
        diagnostics.push(diagnostic);
        return QuestmancerConfig::default();
    }
    let (Some(path), Some(bytes)) = (read.path, read.bytes) else {
        return QuestmancerConfig::default();
    };
    match QuestmancerConfig::parse(&bytes) {
        Ok(config) => config,
        Err(error) => {
            diagnostics.push(PersistenceDiagnostic {
                operation: "parse config",
                path,
                line: None,
                source_message: error.to_string(),
            });
            QuestmancerConfig::default()
        }
    }
}

enum StateLoad {
    Missing,
    Loaded(super::PersistedStateV1),
    Protected,
}

fn interpret_state(read: RawRead, diagnostics: &mut Vec<PersistenceDiagnostic>) -> StateLoad {
    if let Some(diagnostic) = read.diagnostic {
        diagnostics.push(diagnostic);
        return StateLoad::Protected;
    }
    let (Some(path), Some(bytes)) = (read.path, read.bytes) else {
        return StateLoad::Missing;
    };
    match parse_state(&path, &bytes) {
        Ok(state) => StateLoad::Loaded(state),
        Err(diagnostic) => {
            diagnostics.push(diagnostic);
            StateLoad::Protected
        }
    }
}

fn interpret_chronicle(
    read: RawRead,
    maximum_entries: usize,
    diagnostics: &mut Vec<PersistenceDiagnostic>,
) -> crate::domain::Chronicle {
    if let Some(diagnostic) = read.diagnostic {
        diagnostics.push(diagnostic);
        return crate::domain::Chronicle::new(maximum_entries);
    }
    let (Some(path), Some(bytes)) = (read.path, read.bytes) else {
        return crate::domain::Chronicle::new(maximum_entries);
    };
    let replay = replay_chronicle(&path, &bytes, maximum_entries);
    diagnostics.extend(replay.diagnostics);
    replay.chronicle
}
