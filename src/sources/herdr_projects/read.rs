use super::{
    contract::{
        self, Capabilities, Coherence, Coverage, Diagnostic, FORMAT, HomeReport,
        INSPECTED_REVISION, MAX_SNAPSHOT_BYTES, MAX_THREADS, Placement, ProjectRecord,
        ProjectStatus, ReadError, Selection, Snapshot, ThreadRecord, ThreadStatus,
    },
    files::{Directory, identity, io},
};
use crate::domain::Timestamp;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs::File};

#[derive(Debug, Deserialize)]
struct Settings {
    name: String,
    #[serde(default)]
    goal: String,
}
#[derive(Debug, Deserialize)]
struct State {
    status: ProjectStatus,
}
#[derive(Debug, Deserialize)]
struct Thread {
    id: String,
    title: String,
    status: ThreadStatus,
    kind: Placement,
    created: String,
    #[serde(default)]
    resolved_reason: String,
    #[serde(default)]
    error: String,
    #[serde(default)]
    last_group: String,
    #[serde(default)]
    updated: String,
    #[serde(default)]
    agent: String,
    #[serde(default)]
    profile: String,
    #[serde(default)]
    report_hash: String,
    #[serde(default)]
    acked_report_hash: String,
    #[serde(default)]
    pr: String,
}
#[derive(Debug)]
struct Batch {
    files: BTreeMap<String, Vec<u8>>,
    total: usize,
}
impl Batch {
    fn read(
        &mut self,
        dir: &Directory,
        name: &str,
        key: &str,
        required: bool,
    ) -> Result<(), ReadError> {
        let Some(bytes) = dir.bytes(name)? else {
            return if required {
                Err(io(key, "required record missing"))
            } else {
                Ok(())
            };
        };
        self.total += bytes.len();
        if self.total > MAX_SNAPSHOT_BYTES {
            return Err(ReadError::Limit);
        }
        self.files.insert(key.to_owned(), bytes);
        Ok(())
    }
}
/// Read only the explicitly selected project. Callers own I/O scheduling.
pub fn capture(selection: &Selection, now: Timestamp) -> Result<Snapshot, ReadError> {
    let root = Directory::root(&selection.path)?;
    capture_at(selection, now, &root, &|| Ok(()))
}

/// A held descriptor prevents inode reuse for this running attachment. It is
/// ephemeral evidence of attachment, never a durable project identity.
#[derive(Debug)]
pub(super) struct Lease(Directory);
impl Lease {
    pub(super) fn open(selection: &Selection) -> Result<Self, ReadError> {
        Directory::root(&selection.path)
            .map(Self)
            .map_err(|_| ReadError::Lease)
    }
    pub(super) fn capture(
        &self,
        selection: &Selection,
        now: Timestamp,
        check: &impl Fn() -> Result<(), ReadError>,
    ) -> Result<Snapshot, ReadError> {
        self.check(selection)?;
        let snapshot = capture_at(selection, now, &self.0, check);
        self.check(selection)?;
        snapshot
    }
    pub(super) fn check(&self, selection: &Selection) -> Result<(), ReadError> {
        let selected = Directory::root(&selection.path).and_then(|d| d.identity());
        if selected.ok() != Some(self.0.identity()?) {
            return Err(ReadError::Lease);
        }
        Ok(())
    }
}
fn capture_at(
    selection: &Selection,
    now: Timestamp,
    root: &Directory,
    check: &impl Fn() -> Result<(), ReadError>,
) -> Result<Snapshot, ReadError> {
    check()?;
    let state = root.child(".state")?;
    let threads = root.child("threads")?;
    let lock = lock(&state)?;
    let first = batch(root, &state, &threads, check)?;
    let second = batch(root, &state, &threads, check)?;
    check()?;
    if first.files != second.files
        || root.identity()? != Directory::root(&selection.path)?.identity()?
        || state.identity()? != root.child(".state")?.identity()?
        || threads.identity()? != root.child("threads")?.identity()?
        || lock_identity(lock.as_ref())? != lock_identity(state.file("lock")?.as_ref())?
    {
        return Err(ReadError::Changed);
    }
    let snapshot = project(selection, now, lock.is_some(), &first.files)?;
    contract::validate(&snapshot)?;
    if serde_json::to_vec(&snapshot)
        .map_err(|e| contract::invalid("snapshot", e.to_string()))?
        .len()
        > MAX_SNAPSHOT_BYTES
    {
        return Err(ReadError::Limit);
    }
    check()?;
    Ok(snapshot)
}
fn lock(state: &Directory) -> Result<Option<File>, ReadError> {
    let Some(file) = state.file("lock")? else {
        return Ok(None);
    };
    match file.try_lock_shared() {
        Ok(()) => Ok(Some(file)),
        Err(std::fs::TryLockError::WouldBlock) => Err(ReadError::Busy),
        Err(std::fs::TryLockError::Error(e)) => Err(io("lock", e.to_string())),
    }
}
fn lock_identity(lock: Option<&File>) -> Result<Option<(i128, u64)>, ReadError> {
    lock.map(identity).transpose()
}
fn batch(
    root: &Directory,
    state: &Directory,
    threads: &Directory,
    check: &impl Fn() -> Result<(), ReadError>,
) -> Result<Batch, ReadError> {
    let mut batch = Batch {
        files: BTreeMap::new(),
        total: 0,
    };
    check()?;
    batch.read(root, "PROJECT.md", "PROJECT.md", true)?;
    check()?;
    batch.read(state, "project.json", ".state/project.json", true)?;
    let mut count = 0;
    for name in threads.names()? {
        check()?;
        let Some(id) = name.strip_suffix(".toml") else {
            continue;
        };
        if !contract::thread_id(id) {
            return Err(contract::invalid(&name, "invalid thread filename"));
        }
        count += 1;
        if count > MAX_THREADS {
            return Err(ReadError::Limit);
        }
        batch.read(threads, &name, &format!("threads/{name}"), true)?;
        check()?;
        let report = format!("{id}.md");
        batch.read(threads, &report, &format!("threads/{report}"), false)?;
    }
    Ok(batch)
}
fn project(
    selection: &Selection,
    now: Timestamp,
    locked: bool,
    files: &BTreeMap<String, Vec<u8>>,
) -> Result<Snapshot, ReadError> {
    let text = std::str::from_utf8(&files["PROJECT.md"])
        .map_err(|e| contract::invalid("PROJECT.md", e.to_string()))?;
    let rest = text
        .strip_prefix("+++\n")
        .ok_or_else(|| contract::invalid("PROJECT.md", "missing front matter"))?;
    let front = rest
        .split_once("\n+++\n")
        .map(|(front, _)| front)
        .or_else(|| rest.strip_suffix("\n+++"))
        .ok_or_else(|| contract::invalid("PROJECT.md", "unclosed front matter"))?;
    let settings: Settings =
        toml::from_str(front).map_err(|e| contract::invalid("PROJECT.md", e.message()))?;
    let state: State = serde_json::from_slice(&files[".state/project.json"])
        .map_err(|e| contract::invalid("project.json", e.to_string()))?;
    let mut diagnostics = vec![
        Diagnostic::PartialCoverage,
        Diagnostic::NoNativeDurableIdentity,
    ];
    if !locked {
        diagnostics.push(Diagnostic::MissingRecordLock);
    }
    let mut threads = BTreeMap::new();
    for (path, bytes) in files {
        if path.strip_suffix(".toml").is_none() {
            continue;
        }
        let raw: Thread = toml::from_str(
            std::str::from_utf8(bytes).map_err(|e| contract::invalid(path, e.to_string()))?,
        )
        .map_err(|e| contract::invalid(path, e.message()))?;
        if path != &format!("threads/{}.toml", raw.id) {
            return Err(contract::invalid(path, "filename/record identity mismatch"));
        }
        let record = thread(raw, files)?;
        match &record.home_report {
            Some(r) if !r.matches_reported_hash => {
                diagnostics.push(Diagnostic::ReportHashMismatch(record.id.clone()));
            }
            None if !record.report_hash.is_empty() => {
                diagnostics.push(Diagnostic::ReportMissing(record.id.clone()));
            }
            _ => {}
        }
        if threads.insert(record.id.clone(), record).is_some() {
            return Err(contract::invalid(path, "duplicate thread identity"));
        }
    }
    Ok(Snapshot {
        version: 1,
        format: FORMAT.into(),
        inspected_revision: INSPECTED_REVISION.into(),
        source: selection.source.clone(),
        campaign: selection.campaign.clone(),
        captured_at: now,
        coverage: Coverage::Partial,
        coherence: if locked {
            Coherence::CooperativeRecordLock
        } else {
            Coherence::UnlockedObservation
        },
        capabilities: Capabilities::supported(),
        project: ProjectRecord {
            name: settings.name,
            goal: settings.goal,
            status: state.status,
        },
        threads,
        diagnostics,
    })
}
fn thread(t: Thread, files: &BTreeMap<String, Vec<u8>>) -> Result<ThreadRecord, ReadError> {
    let recorded_group = if t.last_group.is_empty() {
        None
    } else {
        Some(
            serde_json::from_value(serde_json::Value::String(t.last_group.clone()))
                .map_err(|e| contract::invalid(&t.id, e.to_string()))?,
        )
    };
    let path = format!("threads/{}.md", t.id);
    let home_report = files.get(&path).map(|bytes| {
        let sha256 = format!("{:x}", Sha256::digest(bytes));
        HomeReport {
            relative_path: path,
            matches_reported_hash: sha256 == t.report_hash,
            sha256,
        }
    });
    Ok(ThreadRecord {
        id: t.id,
        created: t.created,
        title: t.title,
        status: t.status,
        placement: t.kind,
        resolved_reason: t.resolved_reason,
        error: t.error,
        recorded_group,
        updated: t.updated,
        agent: t.agent,
        profile: t.profile,
        report_hash: t.report_hash,
        acked_report_hash: t.acked_report_hash,
        pr: t.pr,
        home_report,
    })
}
