use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::app::View;

#[derive(Debug)]
pub struct RuntimeRegistration {
    path: PathBuf,
    pane_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct RuntimeState {
    pane_id: String,
    pid: u32,
    started_at: u64,
    initial_view: String,
}

impl RuntimeRegistration {
    pub fn from_env(initial_view: View) -> Result<Option<Self>> {
        let (Ok(state_dir), Ok(pane_id)) = (
            env::var("HERDR_PLUGIN_STATE_DIR"),
            env::var("HERDR_PANE_ID"),
        ) else {
            return Ok(None);
        };

        Self::register(Path::new(&state_dir), &pane_id, initial_view).map(Some)
    }

    pub fn register(state_dir: &Path, pane_id: &str, initial_view: View) -> Result<Self> {
        fs::create_dir_all(state_dir).context("create plugin state directory")?;
        let path = state_dir.join("runtime.json");
        let temporary = state_dir.join(format!("runtime.json.tmp.{}", process::id()));
        let state = RuntimeState {
            pane_id: pane_id.to_owned(),
            pid: process::id(),
            started_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .context("system clock is before Unix epoch")?
                .as_secs(),
            initial_view: match initial_view {
                View::Desk => "desk",
                View::Cafe => "cafe",
            }
            .to_owned(),
        };

        fs::write(&temporary, serde_json::to_vec(&state)?).context("write runtime state")?;
        fs::rename(&temporary, &path).context("publish runtime state")?;

        Ok(Self {
            path,
            pane_id: pane_id.to_owned(),
        })
    }
}

impl Drop for RuntimeRegistration {
    fn drop(&mut self) {
        let Ok(contents) = fs::read(&self.path) else {
            return;
        };
        let Ok(state) = serde_json::from_slice::<RuntimeState>(&contents) else {
            return;
        };
        if state.pane_id == self.pane_id {
            let _ = fs::remove_file(&self.path);
        }
    }
}
