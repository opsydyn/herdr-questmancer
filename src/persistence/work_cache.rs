//! Atomic projection + typed observer checkpoints. Invalid files stay protected.
use super::{PersistenceError, atomic_json::publish_bytes, strict_json};
use crate::{
    sources::herdr_projects::{Checkpoint, Proposal},
    work::{Projection, SourceId},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, io::ErrorKind, path::Path};
use tokio::io::AsyncReadExt;
const MAX_BYTES: u64 = 16 * 1024 * 1024;
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "checkpoint", deny_unknown_fields)]
pub enum SourceCheckpoint {
    HerdrProjects(Checkpoint),
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkCache {
    pub projection: Projection,
    pub checkpoints: BTreeMap<SourceId, SourceCheckpoint>,
}
impl WorkCache {
    pub fn herdr_projects(&self, source: &SourceId) -> Option<&Checkpoint> {
        self.checkpoints.get(source).map(|c| match c {
            SourceCheckpoint::HerdrProjects(c) => c,
        })
    }
    fn validate(&self) -> Result<(), String> {
        self.projection.validate().map_err(|e| e.to_string())?;
        if self.checkpoints.len() > 128 {
            return Err("too many observer checkpoints".into());
        }
        for (source, checkpoint) in &self.checkpoints {
            match checkpoint {
                SourceCheckpoint::HerdrProjects(c) => {
                    if source != c.registration().source() {
                        return Err("checkpoint source mismatch".into());
                    }
                    c.validate(&self.projection).map_err(|e| e.to_string())?;
                }
            }
        }
        Ok(())
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct V1 {
    version: u32,
    projection: Projection,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct V2 {
    version: u32,
    projection: Projection,
    checkpoints: BTreeMap<SourceId, SourceCheckpoint>,
}
fn error(path: &Path, message: &impl ToString) -> PersistenceError {
    PersistenceError {
        operation: "work cache",
        path: path.to_owned(),
        line: None,
        source_message: message.to_string(),
    }
}
async fn read(path: &Path) -> Result<Option<(u32, WorkCache)>, PersistenceError> {
    let file = match tokio::fs::File::open(path).await {
        Ok(file) => file,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(error(path, &e)),
    };
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .await
        .map_err(|e| error(path, &e))?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(error(
            path,
            &"work cache exceeds byte limit; file protected",
        ));
    }
    let value = strict_json::parse(&bytes).map_err(|e| error(path, &e))?;
    let (version, mut cache) = match value.get("version").and_then(serde_json::Value::as_u64) {
        Some(1) => {
            let s: V1 = serde_json::from_value(value).map_err(|e| error(path, &e))?;
            (
                s.version,
                WorkCache {
                    projection: s.projection,
                    checkpoints: BTreeMap::new(),
                },
            )
        }
        Some(2) => {
            let s: V2 = serde_json::from_value(value).map_err(|e| error(path, &e))?;
            (
                s.version,
                WorkCache {
                    projection: s.projection,
                    checkpoints: s.checkpoints,
                },
            )
        }
        _ => {
            return Err(error(
                path,
                &"unsupported work snapshot version; file protected",
            ));
        }
    };
    cache.validate().map_err(|e| error(path, &e))?;
    cache.projection.restore_cached();
    Ok(Some((version, cache)))
}
pub async fn load_work_cache(path: &Path) -> Result<Option<WorkCache>, PersistenceError> {
    Ok(read(path).await?.map(|(_, cache)| cache))
}
pub async fn load_work(path: &Path) -> Result<Option<Projection>, PersistenceError> {
    Ok(load_work_cache(path).await?.map(|cache| cache.projection))
}
pub async fn publish_work(path: &Path, projection: &Projection) -> Result<bool, PersistenceError> {
    let previous = read(path).await?;
    let mut next = WorkCache {
        projection: projection.clone(),
        checkpoints: previous
            .as_ref()
            .map_or_else(BTreeMap::new, |(_, c)| c.checkpoints.clone()),
    };
    next.projection.restore_cached();
    // Ordinary intent/fixture publication preserves registered checkpoints.
    if let Some((_, old)) = &previous {
        for source in old.checkpoints.keys() {
            if next.projection.sources.get(source) != old.projection.sources.get(source) {
                return Err(error(
                    path,
                    &"registered facts require an acknowledged observer proposal",
                ));
            }
        }
    }
    publish(path, &next, previous.as_ref()).await
}
pub async fn publish_work_proposal(
    path: &Path,
    proposal: &Proposal,
) -> Result<bool, PersistenceError> {
    let previous = read(path).await?;
    let empty = WorkCache::default();
    let old = previous.as_ref().map_or(&empty, |(_, c)| c);
    let mut expected = proposal.base.clone();
    expected.restore_cached();
    if expected != old.projection
        || proposal.previous.as_ref() != old.herdr_projects(proposal.source())
    {
        return Err(error(
            path,
            &"proposal was based on an obsolete accepted checkpoint",
        ));
    }
    let mut next = WorkCache {
        projection: proposal.projection.clone(),
        checkpoints: old.checkpoints.clone(),
    };
    next.projection.restore_cached();
    next.checkpoints.insert(
        proposal.source().clone(),
        SourceCheckpoint::HerdrProjects(proposal.checkpoint.clone()),
    );
    publish(path, &next, previous.as_ref()).await
}
async fn publish(
    path: &Path,
    next: &WorkCache,
    previous: Option<&(u32, WorkCache)>,
) -> Result<bool, PersistenceError> {
    next.validate().map_err(|e| error(path, &e))?;
    if previous.is_some_and(|(v, c)| *v == 2 && c == next) {
        return Ok(false);
    }
    let mut bytes = serde_json::to_vec_pretty(&V2 {
        version: 2,
        projection: next.projection.clone(),
        checkpoints: next.checkpoints.clone(),
    })
    .map_err(|e| error(path, &e))?;
    if bytes.len() as u64 >= MAX_BYTES {
        return Err(error(path, &"work cache exceeds byte limit"));
    }
    bytes.push(b'\n');
    publish_bytes(path, &bytes).await?;
    Ok(true)
}
