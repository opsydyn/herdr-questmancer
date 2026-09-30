use super::{
    Artifact, CampaignId, Change, Freshness, Inventory, Message, ObligationState, Projection,
    QuestState, Retired, SourceId, SourceState, WorkError,
};
use std::collections::BTreeMap;

pub fn reduce(mut state: Projection, message: Message) -> Result<Projection, WorkError> {
    match message {
        message @ Message::Baseline { .. } => apply_baseline(&mut state, message)?,
        Message::Update {
            source,
            generation,
            occurred_at,
            cursor,
            event,
            changes,
        } => {
            let current = state.sources.get_mut(&source).ok_or(WorkError::Reconcile)?;
            if generation < current.generation
                || (generation == current.generation && cursor < current.cursor)
            {
                return Ok(state);
            }
            if generation == current.generation && cursor == current.cursor {
                if current.last_event.is_empty() {
                    return Ok(state);
                }
                return if event == current.last_event
                    && changes == current.last_changes
                    && current.last_occurrence == Some(occurred_at)
                {
                    Ok(state)
                } else {
                    Err(WorkError::Conflict)
                };
            }
            if current.freshness != Freshness::Current
                || generation != current.generation
                || current.cursor.checked_add(1) != Some(cursor)
                || !text(&event)
            {
                return Err(WorkError::Reconcile);
            }
            if event == current.last_event {
                return Err(WorkError::Conflict);
            }
            for change in changes.clone() {
                if !change_valid(&source, &change) {
                    return Err(WorkError::Invalid);
                }
                apply(&mut current.inventory, &mut current.retired, change)?;
            }
            validate(&source, &current.inventory)?;
            current.cursor = cursor;
            current.last_event = event;
            current.last_changes = changes;
            current.last_occurrence = Some(occurred_at);
        }
        Message::Disconnected(source) => {
            if let Some(s) = state.sources.get_mut(&source) {
                s.freshness = Freshness::Unavailable;
                s.bindings.clear();
            }
        }
        Message::BindingRemoved { source, quest } => {
            if let Some(s) = state.sources.get_mut(&source) {
                s.bindings.remove(&quest);
            }
        }
    }
    Ok(state)
}

fn apply(
    inventory: &mut Inventory,
    retired: &mut Retired,
    change: Change,
) -> Result<(), WorkError> {
    if ignore_retired(retired, &change)? {
        return Ok(());
    }
    if let Change::Obligation(value) = &change
        && value.state == ObligationState::Open
        && inventory.obligations.get(&value.id).is_some_and(|old| {
            old.state != ObligationState::Open
                && old.incarnation == value.incarnation
                && value.revision > old.revision
        })
    {
        return Err(WorkError::Conflict);
    }
    match change {
        Change::Campaign(value) => upsert(&mut inventory.campaigns, value.id.clone(), value, |v| {
            v.revision
        }),
        Change::Quest(value) => upsert(&mut inventory.quests, value.id.clone(), value, |v| {
            v.revision
        }),
        Change::Obligation(value) => {
            upsert(&mut inventory.obligations, value.id.clone(), value, |v| {
                v.revision
            })
        }
        Change::Artifact(value) => upsert(&mut inventory.artifacts, value.id.clone(), value, |v| {
            v.revision
        }),
        Change::Resolve {
            id,
            incarnation,
            revision,
        } => {
            close_obligation(
                inventory,
                retired,
                id,
                incarnation,
                revision,
                ObligationState::Resolved,
            );
            Ok(())
        }
        Change::RemoveObligation {
            id,
            incarnation,
            revision,
        } => {
            close_obligation(
                inventory,
                retired,
                id,
                incarnation,
                revision,
                ObligationState::Superseded,
            );
            Ok(())
        }
        Change::RemoveQuest { id, revision } => {
            if let Some(old) = inventory.quests.get_mut(&id)
                && revision > old.revision
            {
                old.state = QuestState::Cancelled;
                old.revision = revision;
            }
            Ok(())
        }
    }
}

fn upsert<K: Ord, T: Eq>(
    map: &mut BTreeMap<K, T>,
    key: K,
    value: T,
    revision: impl Fn(&T) -> u64,
) -> Result<(), WorkError> {
    if let Some(old) = map.get(&key) {
        if revision(&value) < revision(old) {
            return Ok(());
        }
        if revision(&value) == revision(old) {
            return if old == &value {
                Ok(())
            } else {
                Err(WorkError::Conflict)
            };
        }
    }
    map.insert(key, value);
    Ok(())
}
fn merge(target: &mut Inventory, retired: &mut Retired, input: Inventory) -> Result<(), WorkError> {
    for v in input.campaigns.into_values() {
        apply(target, retired, Change::Campaign(v))?;
    }
    for v in input.quests.into_values() {
        apply(target, retired, Change::Quest(v))?;
    }
    for v in input.obligations.into_values() {
        apply(target, retired, Change::Obligation(v))?;
    }
    for v in input.artifacts.into_values() {
        apply(target, retired, Change::Artifact(v))?;
    }
    Ok(())
}
fn optional_text(value: Option<&str>) -> bool {
    value.is_none_or(text)
}
fn text(value: &str) -> bool {
    !value.is_empty() && value.len() <= 8192 && !value.chars().any(|c| c.is_control() && c != '\n')
}
pub(super) fn validate(source: &SourceId, i: &Inventory) -> Result<(), WorkError> {
    let campaigns = i.campaigns.iter().all(|(id, v)| {
        id == &v.id
            && id.belongs_to(source)
            && v.revision > 0
            && text(&v.name)
            && optional_text(v.objective.as_deref())
    });
    let quests = i.quests.iter().all(|(id, v)| {
        id == &v.id
            && id.belongs_to(source)
            && v.revision > 0
            && text(&v.title)
            && optional_text(v.description.as_deref())
            && v.assignment
                .as_ref()
                .is_none_or(|a| text(&a.worker) && text(&a.name))
            && v.campaign.belongs_to(source)
            && i.campaigns.contains_key(&v.campaign)
            && v.dependencies.iter().all(|d| d.belongs_to(source))
    });
    let obligations = i.obligations.iter().all(|(id, v)| {
        id == &v.id
            && id.belongs_to(source)
            && v.revision > 0
            && text(&v.incarnation)
            && text(&v.prompt)
            && text(&v.action)
            && text(&v.resolution)
            && optional_text(v.context.as_deref())
            && i.campaigns.contains_key(&v.campaign)
            && v.quest.as_ref().is_none_or(|q| {
                i.quests
                    .get(q)
                    .is_some_and(|quest| quest.campaign == v.campaign)
            })
    });
    let artifacts = i
        .artifacts
        .iter()
        .all(|(id, v)| id == &v.id && id.belongs_to(source) && valid_artifact(v, source, i));
    if source.valid() && campaigns && quests && obligations && artifacts {
        Ok(())
    } else {
        Err(WorkError::Invalid)
    }
}
fn valid_artifact(v: &Artifact, source: &SourceId, i: &Inventory) -> bool {
    v.revision > 0
        && text(&v.title)
        && optional_text(v.uri.as_deref())
        && optional_text(v.head_revision.as_deref())
        && v.validation
            .as_ref()
            .is_none_or(|v| text(&v.head_revision) && text(&v.summary))
        && v.campaign.belongs_to(source)
        && i.campaigns.contains_key(&v.campaign)
        && v.quest.as_ref().is_none_or(|q| {
            i.quests
                .get(q)
                .is_some_and(|quest| quest.campaign == v.campaign)
        })
}

impl Projection {
    pub fn reconcile(&mut self, source: &SourceId) {
        if let Some(s) = self.sources.get_mut(source) {
            s.freshness = Freshness::Reconcile;
            s.bindings.clear();
        }
    }
    pub fn restore_cached(&mut self) {
        for s in self.sources.values_mut() {
            s.freshness = Freshness::Cached;
            s.bindings.clear();
        }
    }
    pub fn validate(&self) -> Result<(), WorkError> {
        for (source, s) in &self.sources {
            if s.generation == 0 {
                return Err(WorkError::Invalid);
            }
            validate(source, &s.inventory)?;
            if !s
                .retired
                .campaigns
                .iter()
                .all(|(k, r)| k.belongs_to(source) && *r > 0)
                || !s
                    .retired
                    .quests
                    .iter()
                    .all(|(k, r)| k.belongs_to(source) && *r > 0)
                || !s
                    .retired
                    .artifacts
                    .iter()
                    .all(|(k, r)| k.belongs_to(source) && *r > 0)
                || !s
                    .retired
                    .obligations
                    .iter()
                    .all(|(k, (r, i))| k.belongs_to(source) && *r > 0 && text(i))
            {
                return Err(WorkError::Invalid);
            }
        }
        if self
            .intent
            .iter()
            .any(|(id, i)| !text(&i.incarnation) || !self.sources.keys().any(|s| id.belongs_to(s)))
        {
            return Err(WorkError::Invalid);
        }
        Ok(())
    }
    pub fn obligation(&self, id: &super::ObligationId) -> Option<&super::Obligation> {
        self.sources
            .values()
            .find_map(|s| s.inventory.obligations.get(id))
    }
    pub fn quest(&self, id: &super::QuestId) -> Option<&super::Quest> {
        self.sources
            .values()
            .find_map(|s| s.inventory.quests.get(id))
    }
    pub fn campaign(&self, id: &CampaignId) -> Option<&super::WorkCampaign> {
        self.sources
            .values()
            .find_map(|s| s.inventory.campaigns.get(id))
    }
    pub fn acknowledge(
        &mut self,
        id: &super::ObligationId,
        until: Option<crate::domain::Timestamp>,
    ) {
        if let Some(o) = self
            .obligation(id)
            .filter(|o| o.state == ObligationState::Open)
        {
            let incarnation = o.incarnation.clone();
            let intent = self.intent.entry(id.clone()).or_insert(super::LocalIntent {
                incarnation: incarnation.clone(),
                seen: false,
                deferred_until: None,
            });
            if intent.incarnation != incarnation {
                *intent = super::LocalIntent {
                    incarnation,
                    seen: false,
                    deferred_until: None,
                };
            }
            if let Some(until) = until {
                intent.deferred_until = Some(until);
            } else {
                intent.seen = true;
            }
        }
    }
}

fn apply_baseline(state: &mut Projection, message: Message) -> Result<(), WorkError> {
    let Message::Baseline {
        source,
        generation,
        captured_at,
        cursor,
        inventory,
        complete,
        bindings,
    } = message
    else {
        unreachable!("baseline dispatcher")
    };

    if !source.valid() || generation == 0 {
        return Err(WorkError::Invalid);
    }
    if let Some(old) = state.sources.get(&source)
        && (generation < old.generation || (generation == old.generation && cursor < old.cursor))
    {
        return Ok(());
    }
    if !keys_valid(&source, &inventory) {
        return Err(WorkError::Invalid);
    }
    let mut next = state
        .sources
        .get(&source)
        .map_or_else(Inventory::default, |s| s.inventory.clone());
    let covered = inventory.clone();
    let mut retired = state
        .sources
        .get(&source)
        .map_or_else(Retired::default, |s| s.retired.clone());
    merge(&mut next, &mut retired, inventory)?;
    // A complete inventory may retire absent entities, but cannot
    // downgrade an entity already accepted at a newer revision.
    if complete {
        if covered
            .campaigns
            .iter()
            .any(|(k, v)| next.campaigns.get(k) != Some(v))
            || covered
                .quests
                .iter()
                .any(|(k, v)| next.quests.get(k) != Some(v))
            || covered
                .obligations
                .iter()
                .any(|(k, v)| next.obligations.get(k) != Some(v))
            || covered
                .artifacts
                .iter()
                .any(|(k, v)| next.artifacts.get(k) != Some(v))
        {
            return Err(WorkError::Reconcile);
        }
        remember_omissions(&next, &covered, &mut retired);
        next = covered;
    }
    if let Some(old) = state.sources.get(&source)
        && generation == old.generation
        && cursor == old.cursor
        && next != old.inventory
    {
        return Err(WorkError::Conflict);
    }
    validate(&source, &next)?;
    if bindings.keys().any(|q| !next.quests.contains_key(q)) {
        return Err(WorkError::Invalid);
    }
    let checkpoint = state
        .sources
        .get(&source)
        .filter(|old| old.generation == generation && old.cursor == cursor);
    let last_event = checkpoint.map_or_else(String::new, |s| s.last_event.clone());
    let last_changes = checkpoint.map_or_else(Vec::new, |s| s.last_changes.clone());
    let last_occurrence = checkpoint.and_then(|s| s.last_occurrence);
    state.sources.insert(
        source,
        SourceState {
            generation,
            captured_at,
            last_occurrence,
            cursor,
            last_event,
            last_changes,
            inventory: next,
            retired,
            freshness: Freshness::Current,
            bindings,
        },
    );
    Ok(())
}

fn ignore_retired(retired: &mut Retired, change: &Change) -> Result<bool, WorkError> {
    fn ignore<K: Ord>(map: &mut BTreeMap<K, u64>, id: &K, revision: u64) -> bool {
        if map.get(id).is_some_and(|old| revision <= *old) {
            return true;
        }
        map.remove(id);
        false
    }
    match change {
        Change::Campaign(v) => Ok(ignore(&mut retired.campaigns, &v.id, v.revision)),
        Change::Quest(v) => Ok(ignore(&mut retired.quests, &v.id, v.revision)),
        Change::Artifact(v) => Ok(ignore(&mut retired.artifacts, &v.id, v.revision)),
        Change::Obligation(v) => {
            if let Some((revision, incarnation)) = retired.obligations.get(&v.id) {
                if v.revision <= *revision {
                    return Ok(true);
                }
                if &v.incarnation == incarnation && v.state == ObligationState::Open {
                    return Err(WorkError::Conflict);
                }
            }
            retired.obligations.remove(&v.id);
            Ok(false)
        }
        _ => Ok(false),
    }
}
fn remember_omissions(previous: &Inventory, covered: &Inventory, retired: &mut Retired) {
    for (id, v) in &previous.campaigns {
        if !covered.campaigns.contains_key(id) {
            retired.campaigns.insert(id.clone(), v.revision);
        }
    }
    for (id, v) in &previous.quests {
        if !covered.quests.contains_key(id) {
            retired.quests.insert(id.clone(), v.revision);
        }
    }
    for (id, v) in &previous.artifacts {
        if !covered.artifacts.contains_key(id) {
            retired.artifacts.insert(id.clone(), v.revision);
        }
    }
    for (id, v) in &previous.obligations {
        if !covered.obligations.contains_key(id) {
            retired
                .obligations
                .insert(id.clone(), (v.revision, v.incarnation.clone()));
        }
    }
}

fn keys_valid(source: &SourceId, inventory: &Inventory) -> bool {
    !(!inventory
        .campaigns
        .iter()
        .all(|(k, v)| k == &v.id && k.belongs_to(source))
        || !inventory
            .quests
            .iter()
            .all(|(k, v)| k == &v.id && k.belongs_to(source))
        || !inventory
            .obligations
            .iter()
            .all(|(k, v)| k == &v.id && k.belongs_to(source))
        || !inventory
            .artifacts
            .iter()
            .all(|(k, v)| k == &v.id && k.belongs_to(source)))
}

fn close_obligation(
    inventory: &mut Inventory,
    retired: &mut Retired,
    id: super::ObligationId,
    incarnation: String,
    revision: u64,
    state: ObligationState,
) {
    if let Some(old) = inventory.obligations.get_mut(&id) {
        if old.incarnation == incarnation && revision > old.revision {
            old.state = state;
            old.revision = revision;
        }
    } else if let Some((previous, identity)) = retired.obligations.get_mut(&id) {
        if identity == &incarnation {
            *previous = (*previous).max(revision);
        }
    } else {
        retired.obligations.insert(id, (revision, incarnation));
    }
}
fn change_valid(source: &SourceId, change: &Change) -> bool {
    match change {
        Change::Campaign(v) => v.id.belongs_to(source) && v.revision > 0,
        Change::Quest(v) => v.id.belongs_to(source) && v.revision > 0,
        Change::Obligation(v) => v.id.belongs_to(source) && v.revision > 0,
        Change::Artifact(v) => v.id.belongs_to(source) && v.revision > 0,
        Change::Resolve {
            id,
            incarnation,
            revision,
        }
        | Change::RemoveObligation {
            id,
            incarnation,
            revision,
        } => id.belongs_to(source) && *revision > 0 && text(incarnation),
        Change::RemoveQuest { id, revision } => id.belongs_to(source) && *revision > 0,
    }
}
