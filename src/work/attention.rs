use super::{ObligationId, ObligationState, Projection, QuestId, QuestState, Responsible};
use crate::{
    app::{ConnectionState, Model},
    domain::{AgentKey, GuildSummons, Timestamp},
};
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttentionTarget {
    Obligation(ObligationId),
    Agent(AgentKey),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkSelection {
    Obligation(ObligationId),
    Quest(QuestId),
}
impl Projection {
    pub fn open_decisions(&self) -> Vec<ObligationId> {
        self.sources
            .values()
            .flat_map(|s| s.inventory.obligations.values())
            .filter(|o| {
                o.state == ObligationState::Open
                    && matches!(
                        o.kind,
                        super::ObligationKind::Decision | super::ObligationKind::Permission
                    )
            })
            .map(|o| o.id.clone())
            .collect()
    }
    pub fn next_attention_in(&self, now: Timestamp) -> Option<std::time::Duration> {
        self.intent
            .iter()
            .filter_map(|(id, intent)| {
                let o = self.obligation(id)?;
                if o.state != ObligationState::Open
                    || o.responsible != Responsible::Human
                    || o.incarnation != intent.incarnation
                {
                    return None;
                }
                intent
                    .deferred_until
                    .filter(|until| *until > now)
                    .map(|until| now.elapsed_until(until))
            })
            .min()
    }

    pub fn eligible_obligations(&self, now: Timestamp) -> Vec<ObligationId> {
        let mut list = self
            .sources
            .values()
            .flat_map(|s| s.inventory.obligations.values())
            .filter(|o| {
                o.state == ObligationState::Open
                    && o.responsible == Responsible::Human
                    && self.intent.get(&o.id).is_none_or(|intent| {
                        intent.incarnation != o.incarnation
                            || intent.deferred_until.is_none_or(|until| until <= now)
                    })
            })
            .collect::<Vec<_>>();
        list.sort_by(|a, b| {
            rank(a.kind)
                .cmp(&rank(b.kind))
                .then(a.requested_at.cmp(&b.requested_at))
                .then(a.id.cmp(&b.id))
        });
        list.into_iter().map(|o| o.id.clone()).collect()
    }
    pub fn dependencies_ready(&self, id: &QuestId) -> bool {
        fn visit(p: &Projection, id: &QuestId, path: &mut Vec<QuestId>) -> bool {
            if path.contains(id) {
                return false;
            }
            let Some(q) = p.quest(id) else {
                return false;
            };
            path.push(id.clone());
            let ready = q.dependencies.iter().all(|d| {
                p.quest(d).is_some_and(|q| q.state == QuestState::Completed) && visit(p, d, path)
            });
            path.pop();
            ready
        }
        visit(self, id, &mut Vec::new())
    }
}
const fn rank(kind: super::ObligationKind) -> u8 {
    match kind {
        super::ObligationKind::BlockedAction => 0,
        super::ObligationKind::Decision | super::ObligationKind::Permission => 1,
        super::ObligationKind::ExecutionAction => 2,
        super::ObligationKind::Review => 3,
    }
}
pub fn bound_agent<'a>(model: &'a Model, quest: &QuestId) -> Option<&'a crate::domain::Agent> {
    if model.connection() != &ConnectionState::Connected {
        return None;
    }
    let binding = model.work().sources.values().find_map(|s| {
        (s.freshness == super::Freshness::Current)
            .then(|| s.bindings.get(quest))
            .flatten()
    })?;
    let agent = model.domain().agents.get(&binding.agent)?;
    (model.managed_pane_id() != Some(&agent.pane_id)
        && agent.pane_id == binding.pane
        && agent.capture_identity.fingerprint().as_deref() == Some(binding.identity.as_str())
        && agent.presence != crate::domain::Presence::Exited)
        .then_some(agent)
}
pub fn queue(model: &Model) -> Vec<AttentionTarget> {
    let ids = model.work().eligible_obligations(model.now());
    let mut result = ids
        .iter()
        .cloned()
        .map(AttentionTarget::Obligation)
        .collect::<Vec<_>>();
    for key in model.adventurers_awaiting_a_human() {
        let duplicate = ids.iter().any(|id| {
            let Some(o) = model.work().obligation(id) else {
                return false;
            };
            if o.kind != super::ObligationKind::BlockedAction {
                return false;
            }
            let Some(q) = o.quest.as_ref() else {
                return false;
            };
            let Some(agent) = bound_agent(model, q) else {
                return false;
            };
            agent.key == key
                && agent.attention.summons() == Some(GuildSummons::CounselRequested)
                && model
                    .work()
                    .sources
                    .values()
                    .find_map(|s| s.bindings.get(q))
                    .is_some_and(|b| b.counsel_since == agent.attention.since())
        });
        if !duplicate {
            result.push(AttentionTarget::Agent(key));
        }
    }
    result
}
