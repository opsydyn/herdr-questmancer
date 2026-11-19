//! A grouped view of the existing attention queue and revision reviews.
use super::{
    Obligation, ObligationId, ObligationKind, ObligationState, Projection, QuestId, Responsible,
    WorkSelection,
};
use crate::{
    app::{ConnectionState, Model},
    domain::{Agent, AgentKey, CaptureIdentity, PaneId, Presence, Timestamp},
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdventurerLink {
    pub quest: QuestId,
    adventurer: AgentKey,
    pane: PaneId,
    identity: CaptureIdentity,
}

impl AdventurerLink {
    pub(crate) fn new(model: &Model, quest: &QuestId) -> Option<Self> {
        let agent = model.selected_agent()?;
        if model.connection() != &ConnectionState::Connected
            || agent.presence == Presence::Exited
            || model.managed_pane_id() == Some(&agent.pane_id)
        {
            return None;
        }
        if !agent.capture_identity.is_qualified() {
            return None;
        }
        Some(Self {
            quest: quest.clone(),
            adventurer: agent.key.clone(),
            pane: agent.pane_id.clone(),
            identity: agent.capture_identity.clone(),
        })
    }

    pub(crate) fn resolve<'a>(&self, model: &'a Model) -> Option<&'a Agent> {
        let agent = model.domain().agents.get(&self.adventurer)?;
        (model.connection() == &ConnectionState::Connected
            && model.managed_pane_id() != Some(&agent.pane_id)
            && agent.presence != Presence::Exited
            && agent.pane_id == self.pane
            && agent.capture_identity == self.identity)
            .then_some(agent)
    }
}

pub(crate) fn observer(model: &Model) -> Option<&Agent> {
    let quest = super::view::selected_quest(model)?;
    model
        .inbox_adventurer(quest)
        .or_else(|| super::bound_agent(model, quest))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Row {
    pub quest: Option<QuestId>,
    pub items: Vec<ObligationId>,
    pub due: bool,
}

impl Row {
    pub(crate) fn selected(&self, selection: &WorkSelection) -> bool {
        match selection {
            WorkSelection::Quest(id) => self.quest.as_ref() == Some(id),
            WorkSelection::Obligation(id) => self.items.contains(id),
        }
    }
}

pub(crate) fn deferred_until(
    work: &Projection,
    obligation: &Obligation,
    now: Timestamp,
) -> Option<Timestamp> {
    work.intent
        .get(&obligation.id)
        .filter(|intent| intent.incarnation == obligation.incarnation)
        .and_then(|intent| intent.deferred_until)
        .filter(|until| *until > now && obligation.state == ObligationState::Open)
}

pub(crate) fn rows(work: &Projection, now: Timestamp) -> Vec<Row> {
    let due = work.eligible_obligations(now);
    let due_ids = due.iter().collect::<BTreeSet<_>>();
    let (mut deferred, mut history): (Vec<_>, Vec<_>) = work
        .sources
        .values()
        .flat_map(|source| source.inventory.obligations.values())
        .filter(|o| o.responsible == Responsible::Human && !due_ids.contains(&o.id))
        .filter(|o| o.state == ObligationState::Open || o.kind == ObligationKind::Review)
        .partition(|o| o.state == ObligationState::Open);
    deferred.sort_by_key(|o| (deferred_until(work, o, now), &o.id));
    history.sort_by_key(|o| {
        (
            std::cmp::Reverse(o.requested_at),
            std::cmp::Reverse(o.revision),
            &o.id,
        )
    });
    let mut rows = Vec::<Row>::new();
    let mut quests = BTreeMap::new();
    for o in due
        .iter()
        .filter_map(|id| work.obligation(id))
        .chain(deferred)
        .chain(history)
    {
        if let Some(index) = o.quest.as_ref().and_then(|id| quests.get(id)).copied() {
            let row: &mut Row = &mut rows[index];
            row.items.push(o.id.clone());
        } else {
            if let Some(quest) = &o.quest {
                quests.insert(quest.clone(), rows.len());
            }
            rows.push(Row {
                quest: o.quest.clone(),
                items: vec![o.id.clone()],
                due: due_ids.contains(&o.id),
            });
        }
    }
    rows
}

pub(crate) fn reviewed_obligation(model: &crate::app::Model) -> Option<&Obligation> {
    let Some(WorkSelection::Obligation(id)) = model.work_selection() else {
        return None;
    };
    let o = model.work().obligation(id)?;
    (o.kind == ObligationKind::Review).then_some(o)
}

pub(crate) fn owned_review(model: &crate::app::Model) -> Option<(&Obligation, u64)> {
    let o = reviewed_obligation(model)?;
    let metadata = model.authoring().metadata.as_ref()?;
    let source = model.work().sources.get(&metadata.source)?;
    (source.freshness == super::Freshness::Current
        && source.cursor == metadata.revision
        && metadata.inventory.obligations.get(&o.id) == Some(o))
    .then_some((o, metadata.revision))
}

pub(crate) fn current_detail(
    model: &crate::app::Model,
) -> Option<&crate::owned_work::authoring::ReviewDetail> {
    let (o, revision) = owned_review(model)?;
    model
        .authoring()
        .review
        .as_ref()
        .filter(|d| d.id == o.id && d.revision == revision)
}
