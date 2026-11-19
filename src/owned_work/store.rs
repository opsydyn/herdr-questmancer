//! A single locked owner and acknowledged atomic snapshots. No cache is authority.
use super::{Command, CommandError, OwnedState, Receipt, random_token, transition};
use crate::{persistence::strict_json, work::SourceId};
use rustix::{
    fd::OwnedFd,
    fs::{self, AtFlags, Mode, OFlags},
};
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsString,
    fs::File,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

const MAX_BYTES: usize = 16 * 1024 * 1024;
const PREREQUISITE_VERSION: u32 = 4;
const READ: OFlags = OFlags::RDONLY
    .union(OFlags::CLOEXEC)
    .union(OFlags::NOFOLLOW)
    .union(OFlags::NONBLOCK);
type Identity = (i128, u64);

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)]
    Command(#[from] CommandError),
    #[error("invalid, missing or replaced owned store selection")]
    Selection,
    #[error("owned store has another writer")]
    Busy,
    #[error("owned store is protected: {0}")]
    Protected(String),
    #[error("owned store I/O failed: {0}")]
    Io(String),
    #[error("local review evidence unavailable: {0}")]
    ReviewEvidence(String),
    #[error("owned command {0:?} may have committed; reconcile before retrying")]
    Indeterminate(super::CommandId),
    #[error("owned store creation may have committed; explicitly reopen the same selection")]
    CreationIndeterminate,
}
#[derive(Debug)]
pub struct OwnedStore {
    lease: Lease,
    snapshot: Snapshot,
    stamp: Stamp,
    pending: Option<Pending>,
}
#[derive(Debug)]
struct Lease {
    parent: PathBuf,
    name: OsString,
    directory: OwnedFd,
    lock: File,
    lock_name: OsString,
}

impl Drop for Lease {
    fn drop(&mut self) {
        // A concurrent fork may briefly retain the descriptor until exec.
        // Release the lock when the owning lease ends.
        let _ = self.lock.unlock();
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Stamp {
    identity: Identity,
    digest: blake3::Hash,
}
#[derive(Debug)]
struct Pending {
    snapshot: Snapshot,
    bytes: Vec<u8>,
    stamp: Stamp,
    receipt: Receipt,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    version: u32,
    state: OwnedState,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PublishStep {
    FileSynced,
    Replaced,
}

impl OwnedStore {
    /// Explicit creation in an existing physical directory. Never imports a cache.
    pub fn create(path: &Path, source: SourceId) -> Result<Self, StoreError> {
        let snapshot = Snapshot {
            version: 1,
            state: OwnedState::new(source)?,
        };
        let lease = Lease::acquire(path)?;
        if lease.read()?.is_some() {
            return Err(StoreError::Selection);
        }
        let bytes = encode(&snapshot)?;
        let stamp = lease
            .publish(&bytes, None, |_| Ok(()))
            .map_err(|(e, replaced)| {
                if replaced.is_some() {
                    StoreError::CreationIndeterminate
                } else {
                    e
                }
            })?;
        Ok(Self {
            lease,
            snapshot,
            stamp,
            pending: None,
        })
    }
    /// Requires the selected source identity. Missing/future/malformed files stay protected.
    pub fn open(path: &Path, source: &SourceId) -> Result<Self, StoreError> {
        let lease = Lease::acquire(path)?;
        let (bytes, stamp) = lease.read()?.ok_or(StoreError::Selection)?;
        let snapshot = decode(&bytes)?;
        if snapshot.state.source() != source {
            return Err(StoreError::Selection);
        }
        lease.sync_current(&stamp)?;
        Ok(Self {
            lease,
            snapshot,
            stamp,
            pending: None,
        })
    }
    pub fn state(&self) -> &OwnedState {
        &self.snapshot.state
    }
    /// Validate the selected file before exposing acknowledged state.
    pub fn checked_state(&self) -> Result<&OwnedState, StoreError> {
        if let Some(p) = &self.pending {
            return Err(StoreError::Indeterminate(p.receipt.command.id.clone()));
        }
        self.lease.check()?;
        let (_, stamp) = self.lease.read()?.ok_or(StoreError::Selection)?;
        if stamp != self.stamp {
            return Err(StoreError::Selection);
        }
        Ok(self.state())
    }
    pub fn execute(&mut self, command: &Command) -> Result<Receipt, StoreError> {
        self.execute_with(command, |_| Ok(()))
    }
    fn execute_with(
        &mut self,
        command: &Command,
        hook: impl FnMut(PublishStep) -> Result<(), StoreError>,
    ) -> Result<Receipt, StoreError> {
        if let Some(p) = &self.pending {
            return Err(StoreError::Indeterminate(p.receipt.command.id.clone()));
        }
        self.lease.check()?;
        let (_, current) = self.lease.read()?.ok_or(StoreError::Selection)?;
        if current != self.stamp {
            return Err(StoreError::Selection);
        }
        let next = transition(self.state(), command)?;
        if next.replayed {
            return Ok(next.receipt);
        }
        let target = match &command.action {
            super::Action::RequestReview { target, .. } => Some(target),
            super::Action::SettleReview { id, .. } => {
                next.state.review(id).map(|review| &review.target)
            }
            _ => None,
        };
        if let Some(target) = target {
            super::review_git::changes(target).map_err(StoreError::ReviewEvidence)?;
        }
        let snapshot = Snapshot {
            version: self.snapshot.version.max(snapshot_version(&next.state)),
            state: next.state,
        };
        let bytes = encode(&snapshot)?;
        match self.lease.publish(&bytes, Some(&self.stamp), hook) {
            Ok(stamp) => {
                self.snapshot = snapshot;
                self.stamp = stamp;
                Ok(next.receipt)
            }
            Err((error, None)) => Err(error),
            Err((_, Some(stamp))) => {
                let id = next.receipt.command.id.clone();
                self.pending = Some(Pending {
                    snapshot,
                    bytes,
                    stamp,
                    receipt: next.receipt,
                });
                Err(StoreError::Indeterminate(id))
            }
        }
    }
    /// Re-read the receipt and re-establish durability; never apply a pending command twice.
    pub fn reconcile(&mut self) -> Result<Option<Receipt>, StoreError> {
        self.lease.check()?;
        let Some(pending) = &self.pending else {
            return Ok(None);
        };
        let (bytes, stamp) = self.lease.read()?.ok_or(StoreError::Selection)?;
        if stamp == pending.stamp && bytes == pending.bytes && decode(&bytes)? == pending.snapshot {
            self.lease.sync_current(&stamp)?;
            self.snapshot = pending.snapshot.clone();
            self.stamp = stamp;
            let receipt = pending.receipt.clone();
            self.pending = None;
            Ok(Some(receipt))
        } else if stamp == self.stamp {
            self.pending = None;
            Ok(None)
        } else {
            Err(StoreError::Selection)
        }
    }
}

fn encode(snapshot: &Snapshot) -> Result<Vec<u8>, StoreError> {
    snapshot.state.validate()?;
    let mut bytes = serde_json::to_vec(snapshot).map_err(io)?;
    bytes.push(b'\n');
    if bytes.len() > MAX_BYTES {
        return Err(StoreError::Protected("byte limit exceeded".into()));
    }
    Ok(bytes)
}
fn snapshot_version(state: &OwnedState) -> u32 {
    if state.has_prerequisite_edits() {
        PREREQUISITE_VERSION
    } else if state.has_quest_lifecycle() {
        3
    } else if state.has_reviews() {
        2
    } else {
        1
    }
}
fn decode(bytes: &[u8]) -> Result<Snapshot, StoreError> {
    let value = strict_json::parse(bytes).map_err(|e| StoreError::Protected(e.to_string()))?;
    if !matches!(
        value.get("version").and_then(serde_json::Value::as_u64),
        Some(1..=4)
    ) {
        return Err(StoreError::Protected("unsupported version".into()));
    }
    if value["version"] == 1 && value["state"].get("reviews").is_some() {
        return Err(StoreError::Protected(
            "reviews require owned format v2".into(),
        ));
    }
    let snapshot: Snapshot =
        serde_json::from_value(value).map_err(|e| StoreError::Protected(e.to_string()))?;
    // A v4 store stays v4 after its last link and edit receipt are gone.
    if snapshot.version < PREREQUISITE_VERSION
        && snapshot.version != snapshot_version(&snapshot.state)
    {
        return Err(StoreError::Protected(
            "owned snapshot contents do not match its version".into(),
        ));
    }
    snapshot
        .state
        .validate()
        .map_err(|e| StoreError::Protected(e.to_string()))?;
    Ok(snapshot)
}
impl Lease {
    fn acquire(path: &Path) -> Result<Self, StoreError> {
        if !path.is_absolute()
            || path
                .components()
                .any(|p| !matches!(p, Component::RootDir | Component::Normal(_)))
        {
            return Err(StoreError::Selection);
        }
        let parent = path.parent().ok_or(StoreError::Selection)?.to_owned();
        let name = path.file_name().ok_or(StoreError::Selection)?.to_owned();
        let directory = physical_directory(&parent)?;
        let mut lock_name = name.clone();
        lock_name.push(".lock");
        let lock = File::from(
            fs::openat(
                &directory,
                &lock_name,
                OFlags::RDWR
                    | OFlags::CREATE
                    | OFlags::CLOEXEC
                    | OFlags::NOFOLLOW
                    | OFlags::NONBLOCK,
                Mode::RUSR | Mode::WUSR,
            )
            .map_err(io)?,
        );
        if !lock.metadata().map_err(io)?.is_file() {
            return Err(StoreError::Selection);
        }
        lock.try_lock().map_err(|e| match e {
            std::fs::TryLockError::WouldBlock => StoreError::Busy,
            std::fs::TryLockError::Error(e) => io(e),
        })?;
        let lease = Self {
            parent,
            name,
            directory,
            lock,
            lock_name,
        };
        lease.check()?;
        Ok(lease)
    }
    fn check(&self) -> Result<(), StoreError> {
        let directory = physical_directory(&self.parent)?;
        let lock = fs::openat(&self.directory, &self.lock_name, READ, Mode::empty()).map_err(io)?;
        if identity(&directory)? != identity(&self.directory)?
            || identity(&lock)? != identity(&self.lock)?
        {
            return Err(StoreError::Selection);
        }
        Ok(())
    }
    fn file(&self) -> Result<Option<File>, StoreError> {
        let file = match fs::openat(&self.directory, &self.name, READ, Mode::empty()) {
            Ok(fd) => File::from(fd),
            Err(rustix::io::Errno::NOENT) => return Ok(None),
            Err(e) => return Err(io(e)),
        };
        let meta = file.metadata().map_err(io)?;
        if !meta.is_file() || meta.len() > MAX_BYTES as u64 {
            return Err(StoreError::Protected(
                "expected bounded regular file".into(),
            ));
        }
        Ok(Some(file))
    }
    fn read(&self) -> Result<Option<(Vec<u8>, Stamp)>, StoreError> {
        let Some(file) = self.file()? else {
            return Ok(None);
        };
        let identity = identity(&file)?;
        let mut bytes = Vec::new();
        file.take((MAX_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(io)?;
        if bytes.len() > MAX_BYTES {
            return Err(StoreError::Protected("byte limit exceeded".into()));
        }
        let digest = blake3::hash(&bytes);
        Ok(Some((bytes, Stamp { identity, digest })))
    }
    fn sync_current(&self, stamp: &Stamp) -> Result<(), StoreError> {
        let file = self.file()?.ok_or(StoreError::Selection)?;
        if identity(&file)? != stamp.identity {
            return Err(StoreError::Selection);
        }
        file.sync_all().map_err(io)?;
        fs::fsync(&self.directory).map_err(io)?;
        self.check()?;
        if self.read()?.as_ref().map(|(_, s)| s) != Some(stamp) {
            return Err(StoreError::Selection);
        }
        Ok(())
    }
    fn publish(
        &self,
        bytes: &[u8],
        expected: Option<&Stamp>,
        mut hook: impl FnMut(PublishStep) -> Result<(), StoreError>,
    ) -> Result<Stamp, (StoreError, Option<Stamp>)> {
        let name = format!(
            ".owned-work-{}.tmp",
            random_token().map_err(|e| (io(e), None))?
        );
        let mut replaced = None;
        let mut created = false;
        let result = (|| {
            self.check()?;
            let mut file = File::from(
                fs::openat(
                    &self.directory,
                    &name,
                    OFlags::WRONLY
                        | OFlags::CREATE
                        | OFlags::EXCL
                        | OFlags::CLOEXEC
                        | OFlags::NOFOLLOW,
                    Mode::RUSR | Mode::WUSR,
                )
                .map_err(io)?,
            );
            created = true;
            file.write_all(bytes).map_err(io)?;
            file.sync_all().map_err(io)?;
            hook(PublishStep::FileSynced)?;
            self.check()?;
            if self.read()?.as_ref().map(|(_, s)| s) != expected {
                return Err(StoreError::Selection);
            }
            let candidate = Stamp {
                identity: identity(&file)?,
                digest: blake3::hash(bytes),
            };
            if expected.is_some() {
                // Rename errors are conservative: verify the stored receipt before retry.
                replaced = Some(candidate.clone());
                fs::renameat(&self.directory, &name, &self.directory, &self.name).map_err(io)?;
            } else {
                fs::linkat(
                    &self.directory,
                    &name,
                    &self.directory,
                    &self.name,
                    AtFlags::empty(),
                )
                .map_err(io)?;
                replaced = Some(candidate);
            }
            hook(PublishStep::Replaced)?;
            fs::fsync(&self.directory).map_err(io)?;
            self.check()?;
            let (actual, stamp) = self.read()?.ok_or(StoreError::Selection)?;
            if actual != bytes {
                return Err(StoreError::Selection);
            }
            Ok(stamp)
        })();
        if created {
            let _ = fs::unlinkat(&self.directory, &name, AtFlags::empty());
        }
        result.map_err(|e| (e, replaced))
    }
}
fn physical_directory(path: &Path) -> Result<OwnedFd, StoreError> {
    let mut fd = fs::open("/", READ | OFlags::DIRECTORY, Mode::empty()).map_err(io)?;
    for part in path.components() {
        match part {
            Component::RootDir => (),
            Component::Normal(name) => {
                fd = fs::openat(&fd, name, READ | OFlags::DIRECTORY, Mode::empty()).map_err(io)?;
            }
            _ => return Err(StoreError::Selection),
        }
    }
    Ok(fd)
}
fn identity(fd: impl std::os::fd::AsFd) -> Result<Identity, StoreError> {
    let stat = fs::fstat(fd).map_err(io)?;
    Ok((i128::from(stat.st_dev), stat.st_ino))
}
// map_err consumes the native I/O error at this boundary.
#[allow(clippy::needless_pass_by_value)]
fn io(error: impl ToString) -> StoreError {
    StoreError::Io(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::Timestamp,
        owned_work::{Action, CommandId, PrerequisiteChange, QuestLifecycleChange},
        work::{CampaignId, QuestId, QuestKind, QuestState},
    };
    fn setup() -> (tempfile::TempDir, OwnedStore, Command) {
        let dir =
            tempfile::tempdir_in(std::fs::canonicalize(std::env::temp_dir()).unwrap()).unwrap();
        let source = SourceId::new("owned:faults");
        let store =
            OwnedStore::create(&dir.path().join("owned-work.json"), source.clone()).unwrap();
        let command = Command {
            id: CommandId::new("create"),
            source: source.clone(),
            expected_revision: 0,
            actor: "Questmancer".into(),
            at: Timestamp::from_millis(1),
            action: Action::CreateCampaign {
                id: CampaignId::new(&source, "campaign"),
                name: "Campaign".into(),
                objective: None,
            },
        };
        (dir, store, command)
    }
    #[test]
    fn before_replacement_failure_changes_no_records_and_retry_commits_once() {
        let (dir, mut store, command) = setup();
        let path = dir.path().join("owned-work.json");
        let original = std::fs::read(&path).unwrap();
        assert!(matches!(
            store.execute_with(&command, |step| {
                if step == PublishStep::FileSynced {
                    Err(io("injected file-sync acknowledgement loss"))
                } else {
                    Ok(())
                }
            }),
            Err(StoreError::Io(_))
        ));
        assert_eq!(std::fs::read(&path).unwrap(), original);
        assert_eq!(store.state().revision(), 0);
        assert!(store.reconcile().unwrap().is_none());
        assert_eq!(store.execute(&command).unwrap().revision, 1);
        assert_eq!(store.execute(&command).unwrap().revision, 1);
        assert_eq!(store.state().inventory().campaigns.len(), 1);
    }
    #[test]
    fn owner_shutdown_releases_lock_with_an_inherited_descriptor() {
        let (dir, store, command) = setup();
        let path = dir.path().join("owned-work.json");
        let inherited = store.lease.lock.try_clone().unwrap();
        assert!(matches!(
            OwnedStore::open(&path, &command.source),
            Err(StoreError::Busy)
        ));
        drop(store);
        let reopened = OwnedStore::open(&path, &command.source).unwrap();
        drop(inherited);
        assert!(matches!(
            OwnedStore::open(&path, &command.source),
            Err(StoreError::Busy)
        ));
        drop(reopened);
    }

    #[test]
    fn first_lifecycle_save_recovers_v3_after_an_indeterminate_commit() {
        let (dir, mut store, create_campaign) = setup();
        let source = create_campaign.source.clone();
        let quest = QuestId::new(&source, "quest");
        store.execute(&create_campaign).unwrap();
        let mut change = Command {
            id: CommandId::new("quest"),
            expected_revision: 1,
            action: Action::CreateQuest {
                id: quest.clone(),
                campaign: CampaignId::new(&source, "campaign"),
                title: "Recover a lifecycle change".into(),
                description: None,
                kind: QuestKind::Ship,
            },
            ..create_campaign
        };
        store.execute(&change).unwrap();
        change.id = CommandId::new("start");
        change.expected_revision = 2;
        change.action = Action::ChangeQuestLifecycle {
            id: quest.clone(),
            change: QuestLifecycleChange::MarkActive,
        };

        let path = dir.path().join("owned-work.json");
        let original = std::fs::read(&path).unwrap();
        assert!(matches!(
            store.execute_with(&change, |step| {
                if step == PublishStep::FileSynced {
                    Err(io("injected pre-rename failure"))
                } else {
                    Ok(())
                }
            }),
            Err(StoreError::Io(_))
        ));
        assert_eq!(std::fs::read(&path).unwrap(), original);
        assert!(matches!(
            store.execute_with(&change, |step| {
                if step == PublishStep::Replaced {
                    Err(io("injected directory sync failure"))
                } else {
                    Ok(())
                }
            }),
            Err(StoreError::Indeterminate(_))
        ));
        assert_eq!(
            store.state().inventory().quests[&quest].state,
            QuestState::Planned
        );
        let receipt = store.reconcile().unwrap().unwrap();
        assert_eq!(snapshot_version(store.checked_state().unwrap()), 3);
        assert_eq!(receipt.revision, 3);
        drop(store);
        let mut reopened = OwnedStore::open(&path, &source).unwrap();
        assert_eq!(
            reopened.state().inventory().quests[&quest].state,
            QuestState::Active
        );
        assert_eq!(reopened.execute(&change).unwrap(), receipt);
    }
    #[test]
    fn first_prerequisite_save_reconciles_the_v4_envelope() {
        let dir =
            tempfile::tempdir_in(std::fs::canonicalize(std::env::temp_dir()).unwrap()).unwrap();
        let path = dir.path().join("owned-work.json");
        let legacy = include_bytes!("../../tests/fixtures/owned-work/pre-lifecycle-v2.json");
        std::fs::write(&path, legacy).unwrap();
        let source = SourceId::new("owned:promo");
        let id = QuestId::new(&source, "picker");
        let prerequisite = QuestId::new(&source, "web");
        let command = Command {
            id: CommandId::new("first-prerequisite"),
            source: source.clone(),
            expected_revision: 5,
            actor: "Questmancer".into(),
            at: Timestamp::from_millis(1000),
            action: Action::ChangeQuestPrerequisite {
                id: id.clone(),
                prerequisite_id: prerequisite.clone(),
                change: PrerequisiteChange::Add,
            },
        };
        let mut store = OwnedStore::open(&path, &source).unwrap();
        assert!(matches!(
            store.execute_with(&command, |step| {
                if step == PublishStep::FileSynced {
                    Err(io("injected failure before replacement"))
                } else {
                    Ok(())
                }
            }),
            Err(StoreError::Io(_))
        ));
        assert_eq!(std::fs::read(&path).unwrap(), legacy);
        assert!(matches!(
            store.execute_with(&command, |step| {
                if step == PublishStep::Replaced {
                    Err(io("injected acknowledgement loss after replacement"))
                } else {
                    Ok(())
                }
            }),
            Err(StoreError::Indeterminate(_))
        ));
        assert!(matches!(
            store.checked_state(),
            Err(StoreError::Indeterminate(_))
        ));
        assert_eq!(store.snapshot.version, 2);
        let receipt = store.reconcile().unwrap().unwrap();
        assert_eq!(store.snapshot.version, PREREQUISITE_VERSION);
        assert_eq!(store.execute(&command).unwrap(), receipt);
        drop(store);
        let reopened = OwnedStore::open(&path, &source).unwrap();
        assert_eq!(reopened.snapshot.version, PREREQUISITE_VERSION);
        assert_eq!(
            reopened.state().inventory().quests[&id].dependencies,
            [prerequisite]
        );
    }

    #[test]
    fn after_replacement_loss_blocks_commands_then_recovers_the_same_receipt() {
        let (_dir, mut store, command) = setup();
        assert!(matches!(
            store.execute_with(&command, |step| {
                if step == PublishStep::Replaced {
                    Err(io("injected directory sync failure"))
                } else {
                    Ok(())
                }
            }),
            Err(StoreError::Indeterminate(_))
        ));
        assert_eq!(store.state().revision(), 0, "not exposed as acknowledged");
        assert!(matches!(
            store.execute(&command),
            Err(StoreError::Indeterminate(_))
        ));
        let receipt = store.reconcile().unwrap().unwrap();
        assert_eq!(store.execute(&command).unwrap(), receipt);
        assert_eq!(store.state().revision(), 1);
        assert_eq!(store.state().inventory().campaigns.len(), 1);
    }
    #[test]
    fn restart_after_replacement_loss_reestablishes_durability_without_reapplying() {
        let (dir, mut store, command) = setup();
        let path = dir.path().join("owned-work.json");
        assert!(matches!(
            store.execute_with(&command, |step| {
                if step == PublishStep::Replaced {
                    Err(io("injected final acknowledgement loss"))
                } else {
                    Ok(())
                }
            }),
            Err(StoreError::Indeterminate(_))
        ));
        drop(store);
        let mut store = OwnedStore::open(&path, &command.source).unwrap();
        assert_eq!(store.execute(&command).unwrap().revision, 1);
        assert_eq!(store.state().inventory().campaigns.len(), 1);
    }
    #[test]
    fn indeterminate_reconciliation_rejects_an_unrelated_replacement() {
        let (dir, mut store, command) = setup();
        assert!(matches!(
            store.execute_with(&command, |step| {
                if step == PublishStep::Replaced {
                    Err(io("injected final acknowledgement loss"))
                } else {
                    Ok(())
                }
            }),
            Err(StoreError::Indeterminate(_))
        ));
        std::fs::write(dir.path().join("owned-work.json"), b"unrelated").unwrap();
        assert!(matches!(store.reconcile(), Err(StoreError::Selection)));
        assert!(matches!(
            store.execute(&command),
            Err(StoreError::Indeterminate(_))
        ));
    }
    #[test]
    fn indeterminate_reconciliation_rejects_even_an_identical_replacement_inode() {
        let (dir, mut store, command) = setup();
        assert!(matches!(
            store.execute_with(&command, |step| {
                if step == PublishStep::Replaced {
                    Err(io("injected directory sync failure"))
                } else {
                    Ok(())
                }
            }),
            Err(StoreError::Indeterminate(_))
        ));
        let path = dir.path().join("owned-work.json");
        let bytes = std::fs::read(&path).unwrap();
        std::fs::rename(&path, dir.path().join("previous.json")).unwrap();
        std::fs::write(&path, bytes).unwrap();
        assert!(matches!(store.reconcile(), Err(StoreError::Selection)));
    }
}
