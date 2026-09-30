//! Pure mapping under explicit registration, never native incarnation authority.
use super::{
    Coherence, FORMAT, MAX_THREADS, Placement, ProjectStatus, RecordedGroup, Snapshot,
    ThreadRecord, ThreadStatus, contract,
};
use crate::work::{
    self, Artifact, ArtifactId, ArtifactKind, CampaignId, CampaignState, Freshness, Inventory,
    Message, Projection, Quest, QuestId, QuestKind, QuestState, SourceId, WorkCampaign,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
enum IdentityPolicy {
    RegisteredRetainedRecords,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Registration {
    source: SourceId,
    campaign: String,
    format: String,
    policy: IdentityPolicy,
}
impl Registration {
    /// Caller explicitly registers retained-record continuity; paths are locators.
    pub fn retained_records(source: SourceId, campaign: &str) -> Result<Self, MappingError> {
        let value = Self {
            source,
            campaign: campaign.into(),
            format: FORMAT.into(),
            policy: IdentityPolicy::RegisteredRetainedRecords,
        };
        value.validate()?;
        Ok(value)
    }
    pub fn source(&self) -> &SourceId {
        &self.source
    }
    pub fn next_generation(&self, projection: &Projection) -> Result<u64, MappingError> {
        projection.sources.get(&self.source).map_or(Ok(1), |s| {
            s.generation.checked_add(1).ok_or(MappingError::Exhausted)
        })
    }
    fn validate(&self) -> Result<(), MappingError> {
        if self.source.valid() && contract::token(&self.campaign) && self.format == FORMAT {
            Ok(())
        } else {
            Err(MappingError::Invalid)
        }
    }
    fn campaign_id(&self) -> CampaignId {
        CampaignId::new(&self.source, &self.campaign)
    }
    fn local(&self, id: &str) -> String {
        format!("{}.thread.{id}", self.campaign)
    }
    fn quest_id(&self, id: &str) -> QuestId {
        QuestId::new(&self.source, &self.local(id))
    }
    fn artifact_id(&self, id: &str, slot: &str) -> ArtifactId {
        ArtifactId::new(&self.source, &format!("{}.{slot}", self.local(id)))
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct Creation {
    seconds: i64,
    nanoseconds: u32,
}
impl Creation {
    fn parse(value: &str) -> Result<Self, MappingError> {
        let t = OffsetDateTime::parse(value, &Rfc3339).map_err(|_| MappingError::Invalid)?;
        Ok(Self {
            seconds: t.unix_timestamp(),
            nanoseconds: t.nanosecond(),
        })
    }
    fn valid(self) -> bool {
        OffsetDateTime::from_unix_timestamp(self.seconds).is_ok()
            && self.nanoseconds < 1_000_000_000
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Checkpoint {
    registration: Registration,
    threads: BTreeMap<String, Creation>,
}
impl Checkpoint {
    pub fn registration(&self) -> &Registration {
        &self.registration
    }
    pub(crate) fn validate(&self, projection: &Projection) -> Result<(), MappingError> {
        self.registration.validate()?;
        let r = &self.registration;
        let s = projection
            .sources
            .get(&r.source)
            .ok_or(MappingError::Identity)?;
        let i = &s.inventory;
        if self.threads.len() > MAX_THREADS
            || s.generation == 0
            || s.cursor == 0
            || !i.obligations.is_empty()
            || s.retired != work::Retired::default()
            || i.campaigns.len() != 1
            || !i.campaigns.contains_key(&r.campaign_id())
            || i.quests.len() != self.threads.len()
            || !matches!(
                i.campaigns[&r.campaign_id()].state,
                CampaignState::Active | CampaignState::Paused | CampaignState::Archived
            )
        {
            return Err(MappingError::Invalid);
        }
        for (id, created) in &self.threads {
            if !contract::thread_id(id) || !created.valid() {
                return Err(MappingError::Invalid);
            }
            let q = i.quests.get(&r.quest_id(id)).ok_or(MappingError::Invalid)?;
            if q.campaign != r.campaign_id()
                || q.kind != QuestKind::Unspecified
                || q.assignment.is_some()
                || !q.dependencies.is_empty()
                || !matches!(
                    q.state,
                    QuestState::Unknown | QuestState::Closed | QuestState::Failed
                )
            {
                return Err(MappingError::Invalid);
            }
        }
        for a in i.artifacts.values() {
            let matches = self.threads.keys().any(|id| {
                a.quest.as_ref() == Some(&r.quest_id(id))
                    && ((a.id == r.artifact_id(id, "report")
                        && a.kind == ArtifactKind::Report
                        && a.uri.is_none())
                        || (a.id == r.artifact_id(id, "pr")
                            && a.kind == ArtifactKind::PullRequest
                            && a.uri.as_deref().is_none_or(valid_pr)))
            });
            if !matches
                || a.campaign != r.campaign_id()
                || a.head_revision.is_some()
                || a.validation.is_some()
            {
                return Err(MappingError::Invalid);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub(crate) base: Projection,
    pub(crate) previous: Option<Checkpoint>,
    pub(crate) projection: Projection,
    pub(crate) checkpoint: Checkpoint,
}
impl Proposal {
    pub fn projection(&self) -> &Projection {
        &self.projection
    }
    pub fn checkpoint(&self) -> &Checkpoint {
        &self.checkpoint
    }
    pub fn source(&self) -> &SourceId {
        self.checkpoint.registration.source()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum MappingError {
    #[error("invalid registered source evidence")]
    Invalid,
    #[error(
        "source identity changed or has no matching checkpoint; explicit reconciliation required"
    )]
    Identity,
    #[error("known thread omitted; retained inventory requires reconciliation")]
    MissingThread,
    #[error("unlocked export is inspection-only")]
    Unlocked,
    #[error("obsolete observation generation")]
    Stale,
    #[error("observation counter exhausted")]
    Exhausted,
}
impl MappingError {
    /// A future producer must latch this error until explicit reconciliation.
    pub fn requires_rebind(&self) -> bool {
        *self == Self::Identity
    }
}

/// Registration/generation come from the trusted caller, not the source DTO.
/// This pure proof does not certify a directory lease; that is the producer's job.
pub fn map_snapshot(
    r: &Registration,
    generation: u64,
    snapshot: &Snapshot,
    accepted: &Projection,
    prior: Option<&Checkpoint>,
) -> Result<Option<Proposal>, MappingError> {
    validate_context(r, generation, snapshot, accepted, prior)?;
    let old = accepted.sources.get(&r.source);
    let threads: BTreeMap<_, _> = snapshot
        .threads
        .iter()
        .map(|(id, t)| Ok((id.clone(), Creation::parse(&t.created)?)))
        .collect::<Result<_, MappingError>>()?;
    if let Some(prior) = prior {
        for (id, created) in &prior.threads {
            let current = threads.get(id).ok_or(MappingError::MissingThread)?;
            if current != created {
                return Err(MappingError::Identity);
            }
        }
    }
    let checkpoint = Checkpoint {
        registration: r.clone(),
        threads,
    };
    let empty = Inventory::default();
    let old_inventory = old.map_or(&empty, |s| &s.inventory);
    let mut inventory = Inventory::default();
    let campaign = WorkCampaign {
        id: r.campaign_id(),
        revision: 1,
        name: snapshot.project.name.clone(),
        objective: (!snapshot.project.goal.is_empty()).then(|| snapshot.project.goal.clone()),
        state: match snapshot.project.status {
            ProjectStatus::Active => CampaignState::Active,
            ProjectStatus::Paused => CampaignState::Paused,
            ProjectStatus::Archived => CampaignState::Archived,
        },
    };
    let campaign = revised(campaign, old_inventory.campaigns.get(&r.campaign_id()))?;
    inventory.campaigns.insert(campaign.id.clone(), campaign);
    for (id, t) in &snapshot.threads {
        let description = context(t);
        if description.len() > 8192 {
            return Err(MappingError::Invalid);
        }
        let q = Quest {
            id: r.quest_id(id),
            campaign: r.campaign_id(),
            revision: 1,
            title: t.title.clone(),
            description: Some(description),
            kind: QuestKind::Unspecified,
            state: match t.status {
                ThreadStatus::Starting | ThreadStatus::Open => QuestState::Unknown,
                ThreadStatus::Failed => QuestState::Failed,
                ThreadStatus::Resolved => QuestState::Closed,
            },
            assignment: None,
            dependencies: vec![],
        };
        let q = revised(q, old_inventory.quests.get(&r.quest_id(id)))?;
        inventory.quests.insert(q.id.clone(), q);
        references(r, t, old_inventory, &mut inventory)?;
    }
    if old.is_some_and(|s| {
        s.generation == generation && s.freshness == Freshness::Current && s.inventory == inventory
    }) && prior == Some(&checkpoint)
    {
        return Ok(None);
    }
    let cursor = old.map_or(Ok(1), |s| {
        s.cursor.checked_add(1).ok_or(MappingError::Exhausted)
    })?;
    let projection = work::reduce(
        accepted.clone(),
        Message::Baseline {
            source: r.source.clone(),
            generation,
            captured_at: snapshot.captured_at,
            cursor,
            inventory,
            complete: false,
            bindings: BTreeMap::new(),
        },
    )
    .map_err(|_| MappingError::Invalid)?;
    checkpoint.validate(&projection)?;
    Ok(Some(Proposal {
        base: accepted.clone(),
        previous: prior.cloned(),
        projection,
        checkpoint,
    }))
}

fn validate_context(
    r: &Registration,
    generation: u64,
    snapshot: &Snapshot,
    accepted: &Projection,
    prior: Option<&Checkpoint>,
) -> Result<(), MappingError> {
    r.validate()?;
    accepted.validate().map_err(|_| MappingError::Invalid)?;
    let old = accepted.sources.get(&r.source);
    if generation == 0 || old.is_none() && generation != 1 {
        return Err(MappingError::Stale);
    }
    if let Some(old) = old
        && (generation < old.generation
            || generation > old.generation && old.generation.checked_add(1) != Some(generation)
            || generation == old.generation && old.freshness == Freshness::Cached)
    {
        return Err(MappingError::Stale);
    }
    contract::validate(snapshot).map_err(|_| MappingError::Invalid)?;
    if snapshot.source != r.source || snapshot.campaign != r.campaign {
        return Err(MappingError::Identity);
    }
    if snapshot.coherence != Coherence::CooperativeRecordLock {
        return Err(MappingError::Unlocked);
    }
    match (old, prior) {
        (Some(_), Some(c)) if c.registration == *r => c.validate(accepted)?,
        (None, None) => {}
        _ => return Err(MappingError::Identity),
    }
    Ok(())
}

fn context(t: &ThreadRecord) -> String {
    let status = match t.status {
        ThreadStatus::Starting => "starting",
        ThreadStatus::Open => "open",
        ThreadStatus::Failed => "failed",
        ThreadStatus::Resolved => "resolved",
    };
    let placement = match t.placement {
        Placement::Worktree => "worktree",
        Placement::Tab => "tab",
        Placement::Checkout => "checkout",
        Placement::Adopted => "adopted",
    };
    let group = match t.recorded_group {
        None => "not supplied",
        Some(RecordedGroup::ReadyForReview) => "ready-for-review",
        Some(RecordedGroup::WaitingOnYou) => "waiting-on-you",
        Some(RecordedGroup::Working) => "working",
        Some(RecordedGroup::Landing) => "landing",
        Some(RecordedGroup::Idle) => "idle",
        Some(RecordedGroup::Resolved) => "resolved",
    };
    let report = t.home_report.as_ref().map_or_else(
        || "unavailable".into(),
        |r| {
            format!(
                "{} | bytes {} | {}",
                r.relative_path,
                r.sha256,
                if r.matches_reported_hash {
                    "matches recorded hash; validation unknown"
                } else {
                    "hash mismatch; unavailable"
                }
            )
        },
    );
    format!(
        "Herdr Projects: registered / retained records; partial coverage; native identity unsupported.\nDependencies, structured summons and live bindings unavailable.\nSource thread status: {status}\nPlacement: {placement}\nRecorded group (last recorded, not live execution): {group}\nClosure reason: {}\nSource error: {}\nSource agent label: {}\nSource profile label: {}\nReport reference: {report}\nRecorded report hash: {}\nAcknowledged report hash: {}\nPR reference: {}",
        t.resolved_reason, t.error, t.agent, t.profile, t.report_hash, t.acked_report_hash, t.pr
    )
}
fn references(
    r: &Registration,
    t: &ThreadRecord,
    old: &Inventory,
    next: &mut Inventory,
) -> Result<(), MappingError> {
    let report_ok = t
        .home_report
        .as_ref()
        .is_some_and(|r| r.matches_reported_hash);
    for (slot, kind, present, title, uri) in [
        (
            "report",
            ArtifactKind::Report,
            report_ok,
            if report_ok {
                let report = t.home_report.as_ref().expect("matched report exists");
                format!(
                    "Latest report reference: {} | SHA-256 {} | validation unknown",
                    report.relative_path, report.sha256
                )
            } else {
                "Latest report reference unavailable".into()
            },
            None,
        ),
        (
            "pr",
            ArtifactKind::PullRequest,
            !t.pr.is_empty(),
            if t.pr.is_empty() {
                "Latest PR reference unavailable".into()
            } else {
                format!("Latest PR reference: {} | validation unknown", t.pr)
            },
            valid_pr(&t.pr).then(|| t.pr.clone()),
        ),
    ] {
        let id = r.artifact_id(&t.id, slot);
        let prior = old.artifacts.get(&id);
        if !present && prior.is_none() {
            continue;
        }
        let a = Artifact {
            id,
            campaign: r.campaign_id(),
            quest: Some(r.quest_id(&t.id)),
            revision: 1,
            kind,
            title,
            uri,
            head_revision: None,
            validation: None,
        };
        let a = revised(a, prior)?;
        next.artifacts.insert(a.id.clone(), a);
    }
    Ok(())
}
fn valid_pr(value: &str) -> bool {
    let Some(path) = value.strip_prefix("https://github.com/") else {
        return false;
    };
    let parts: Vec<_> = path.split('/').collect();
    parts.len() == 4
        && parts[2] == "pull"
        && parts[..2].iter().all(|p| {
            !p.is_empty()
                && *p != "."
                && *p != ".."
                && p.len() <= 100
                && p.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        })
        && !parts[3].is_empty()
        && parts[3].bytes().all(|b| b.is_ascii_digit())
        && parts[3].parse::<u64>().is_ok_and(|n| n > 0)
}
trait Revision: Eq {
    fn revision(&self) -> u64;
    fn set_revision(&mut self, v: u64);
}
macro_rules! revisions { ($($ty:ty),+) => {$ (impl Revision for $ty { fn revision(&self)->u64 { self.revision } fn set_revision(&mut self,v:u64) { self.revision=v; } })+}; }
revisions!(WorkCampaign, Quest, Artifact);
fn revised<T: Revision>(mut value: T, old: Option<&T>) -> Result<T, MappingError> {
    if let Some(old) = old {
        value.set_revision(old.revision());
        if value != *old {
            value.set_revision(
                old.revision()
                    .checked_add(1)
                    .ok_or(MappingError::Exhausted)?,
            );
        }
    }
    Ok(value)
}
