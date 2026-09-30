use crate::{domain::Timestamp, work::SourceId};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

pub const FORMAT: &str = "herdr-projects-0.2.34";
pub const INSPECTED_REVISION: &str = "4e4548c3c43888e6be1c96906215998f07dc63f0";
pub const MAX_THREADS: usize = 1_000;
pub const MAX_SNAPSHOT_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selection {
    pub(super) source: SourceId,
    pub(super) campaign: String,
    pub(super) path: PathBuf,
}
impl Selection {
    pub fn new(source: SourceId, campaign: &str, path: PathBuf) -> Result<Self, ReadError> {
        if !source.valid() || !token(campaign) || !path.is_absolute() {
            return Err(ReadError::Selection);
        }
        Ok(Self {
            source,
            campaign: campaign.into(),
            path,
        })
    }
}
#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error("source selection needs valid configured identities and an absolute project path")]
    Selection,
    #[error("source read {path}: {message}")]
    Io { path: String, message: String },
    #[error("invalid source record {path}: {message}")]
    Invalid { path: String, message: String },
    #[error("source writer holds the project lock")]
    Busy,
    #[error("source changed during capture; no snapshot accepted")]
    Changed,
    #[error("source exceeds bounded snapshot limits")]
    Limit,
    #[error("selected project directory disappeared or was replaced; explicit rebind required")]
    Lease,
    #[error("source capture cancelled")]
    Cancelled,
    #[error("source capture exceeded its cooperative time budget")]
    TimedOut,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Coverage {
    Partial,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Coherence {
    CooperativeRecordLock,
    UnlockedObservation,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectStatus {
    Active,
    Paused,
    Archived,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ThreadStatus {
    Starting,
    Open,
    Failed,
    Resolved,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Placement {
    Worktree,
    Tab,
    Checkout,
    Adopted,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RecordedGroup {
    ReadyForReview,
    WaitingOnYou,
    Working,
    Landing,
    Idle,
    Resolved,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum SupportState {
    Supported,
    Unsupported,
}
impl SupportState {
    pub const fn is_supported(self) -> bool {
        matches!(self, Self::Supported)
    }
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Capabilities {
    pub project_metadata: SupportState,
    pub thread_inventory: SupportState,
    pub report_references: SupportState,
    pub dependencies: SupportState,
    pub structured_obligations: SupportState,
    pub live_bindings: SupportState,
    pub durable_identity: SupportState,
    pub revision_bound_validation: SupportState,
}
impl Capabilities {
    pub(super) fn supported() -> Self {
        Self {
            project_metadata: SupportState::Supported,
            thread_inventory: SupportState::Supported,
            report_references: SupportState::Supported,
            dependencies: SupportState::Unsupported,
            structured_obligations: SupportState::Unsupported,
            live_bindings: SupportState::Unsupported,
            durable_identity: SupportState::Unsupported,
            revision_bound_validation: SupportState::Unsupported,
        }
    }
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Diagnostic {
    PartialCoverage,
    NoNativeDurableIdentity,
    MissingRecordLock,
    ReportMissing(String),
    ReportHashMismatch(String),
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectRecord {
    pub name: String,
    pub goal: String,
    pub status: ProjectStatus,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HomeReport {
    pub relative_path: String,
    pub sha256: String,
    pub matches_reported_hash: bool,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ThreadRecord {
    pub id: String,
    /// Evidence only: neither this timestamp nor a digest is a native stable ID.
    pub created: String,
    pub title: String,
    pub status: ThreadStatus,
    pub placement: Placement,
    pub resolved_reason: String,
    pub error: String,
    pub recorded_group: Option<RecordedGroup>,
    pub updated: String,
    pub agent: String,
    pub profile: String,
    pub report_hash: String,
    pub acked_report_hash: String,
    pub pr: String,
    pub home_report: Option<HomeReport>,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub version: u32,
    /// Declared pinned layout, not discovery/verification of an installed binary.
    pub format: String,
    pub inspected_revision: String,
    pub source: SourceId,
    pub campaign: String,
    pub captured_at: Timestamp,
    pub coverage: Coverage,
    pub coherence: Coherence,
    pub capabilities: Capabilities,
    pub project: ProjectRecord,
    #[serde(deserialize_with = "unique_threads")]
    pub threads: BTreeMap<String, ThreadRecord>,
    pub diagnostics: Vec<Diagnostic>,
}
pub fn decode_snapshot(bytes: &[u8]) -> Result<Snapshot, ReadError> {
    if bytes.len() > MAX_SNAPSHOT_BYTES {
        return Err(ReadError::Limit);
    }
    let snapshot: Snapshot =
        serde_json::from_slice(bytes).map_err(|e| invalid("snapshot", e.to_string()))?;
    validate(&snapshot)?;
    Ok(snapshot)
}
fn unique_threads<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, ThreadRecord>, D::Error> {
    struct Threads;
    impl<'de> serde::de::Visitor<'de> for Threads {
        type Value = BTreeMap<String, ThreadRecord>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a bounded map of unique thread identities")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut map: A,
        ) -> Result<Self::Value, A::Error> {
            let mut threads = BTreeMap::new();
            while let Some((key, value)) = map.next_entry::<String, ThreadRecord>()? {
                if threads.insert(key, value).is_some() || threads.len() > MAX_THREADS {
                    return Err(serde::de::Error::custom(
                        "duplicate or excessive thread identity",
                    ));
                }
            }
            Ok(threads)
        }
    }
    deserializer.deserialize_map(Threads)
}
pub(super) fn validate(s: &Snapshot) -> Result<(), ReadError> {
    if s.version != 1
        || s.format != FORMAT
        || s.inspected_revision != INSPECTED_REVISION
        || !s.source.valid()
        || !token(&s.campaign)
        || s.capabilities != Capabilities::supported()
        || !text(&s.project.name, true)
        || !text(&s.project.goal, false)
        || !s.diagnostics.contains(&Diagnostic::PartialCoverage)
        || !s.diagnostics.contains(&Diagnostic::NoNativeDurableIdentity)
    {
        return Err(invalid("snapshot", "unsupported or inconsistent contract"));
    }
    if s.threads.len() > MAX_THREADS || s.diagnostics.len() > MAX_THREADS + 3 {
        return Err(ReadError::Limit);
    }
    for (key, t) in &s.threads {
        if key != &t.id
            || !thread_id(key)
            || !timestamp(&t.created)
            || (!t.updated.is_empty() && !timestamp(&t.updated))
            || !text(&t.title, true)
            || [&t.resolved_reason, &t.error, &t.agent, &t.profile, &t.pr]
                .iter()
                .any(|v| !text(v, false))
            || !optional_hash(&t.report_hash)
            || !optional_hash(&t.acked_report_hash)
        {
            return Err(invalid(key, "invalid thread evidence"));
        }
        if let Some(r) = &t.home_report
            && (r.relative_path != format!("threads/{key}.md")
                || !hash(&r.sha256)
                || r.matches_reported_hash != (r.sha256 == t.report_hash))
        {
            return Err(invalid(key, "invalid report evidence"));
        }
    }
    let mut expected = vec![
        Diagnostic::PartialCoverage,
        Diagnostic::NoNativeDurableIdentity,
    ];
    if s.coherence == Coherence::UnlockedObservation {
        expected.push(Diagnostic::MissingRecordLock);
    }
    for t in s.threads.values() {
        match &t.home_report {
            Some(r) if !r.matches_reported_hash => {
                expected.push(Diagnostic::ReportHashMismatch(t.id.clone()));
            }
            None if !t.report_hash.is_empty() => {
                expected.push(Diagnostic::ReportMissing(t.id.clone()));
            }
            _ => {}
        }
    }
    if s.diagnostics != expected {
        return Err(invalid("snapshot", "inconsistent diagnostics"));
    }
    Ok(())
}
pub(super) fn invalid(path: &str, message: impl Into<String>) -> ReadError {
    ReadError::Invalid {
        path: path.into(),
        message: message.into(),
    }
}
pub(super) fn thread_id(s: &str) -> bool {
    s.strip_prefix("t-")
        .is_some_and(|n| n.len() >= 4 && n.len() <= 20 && n.bytes().all(|b| b.is_ascii_digit()))
}
pub(super) fn token(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._:-".contains(&b))
}
fn text(s: &str, required: bool) -> bool {
    (!required || !s.trim().is_empty())
        && s.len() <= 8192
        && !s.chars().any(|c| c.is_control() && c != '\n')
}
fn timestamp(s: &str) -> bool {
    time::OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339).is_ok()
}
fn hash(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn optional_hash(s: &str) -> bool {
    s.is_empty() || hash(s)
}
