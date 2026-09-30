//! Pure campaign dependency facts. Layout and navigation never change source work.
use super::{
    CampaignId, Freshness, ObligationKind, ObligationState, Projection, QuestId, QuestState,
    Responsible, SourceId, WorkCampaign, view,
};
use crate::{app::Model, domain::Timestamp};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuestNode {
    pub title: String,
    pub state: QuestState,
    pub assignment: Option<String>,
    pub human_requests: usize,
    pub eligible_requests: usize,
    pub review_requests: usize,
    pub artifact_count: usize,
    pub layer: Option<usize>,
    pub cyclic: bool,
    pub missing_dependencies: Vec<QuestId>,
    pub outside_dependencies: Vec<QuestId>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CampaignGraph {
    pub campaign: WorkCampaign,
    pub source: SourceId,
    pub freshness: Freshness,
    pub nodes: BTreeMap<QuestId, QuestNode>,
    /// Prerequisite -> dependent; never a plan or execution command.
    pub edges: BTreeSet<(QuestId, QuestId)>,
    pub selected: Option<QuestId>,
    pub human_requests: usize,
    pub artifact_count: usize,
}

pub fn campaign_ids(model: &Model) -> Vec<CampaignId> {
    model
        .work()
        .sources
        .values()
        .flat_map(|s| s.inventory.campaigns.keys().cloned())
        .collect()
}
pub fn campaign_id(model: &Model) -> Option<CampaignId> {
    model
        .war_campaign()
        .filter(|id| model.work().campaign(id).is_some())
        .cloned()
        .or_else(|| {
            view::selected_quest(model)
                .and_then(|id| model.work().quest(id))
                .map(|q| q.campaign.clone())
        })
        .or_else(|| match model.work_selection() {
            Some(super::WorkSelection::Obligation(id)) => {
                model.work().obligation(id).map(|o| o.campaign.clone())
            }
            _ => None,
        })
        .or_else(|| campaign_ids(model).into_iter().next())
}
impl CampaignGraph {
    pub fn from_model(model: &Model) -> Option<Self> {
        Self::project(
            model.work(),
            &campaign_id(model)?,
            view::selected_quest(model),
            model.now(),
        )
    }
    pub fn project(
        work: &Projection,
        id: &CampaignId,
        selected: Option<&QuestId>,
        now: Timestamp,
    ) -> Option<Self> {
        let (source, state) = work
            .sources
            .iter()
            .find(|(_, s)| s.inventory.campaigns.contains_key(id))?;
        let inventory = &state.inventory;
        let quests = inventory
            .quests
            .values()
            .filter(|q| &q.campaign == id)
            .collect::<Vec<_>>();
        let ids = quests.iter().map(|q| q.id.clone()).collect::<BTreeSet<_>>();
        let eligible = work
            .eligible_obligations(now)
            .into_iter()
            .collect::<BTreeSet<_>>();
        let open = inventory
            .obligations
            .values()
            .filter(|o| {
                &o.campaign == id
                    && o.state == ObligationState::Open
                    && o.responsible == Responsible::Human
            })
            .collect::<Vec<_>>();
        let edges = quests
            .iter()
            .flat_map(|q| {
                q.dependencies
                    .iter()
                    .filter(|d| ids.contains(*d))
                    .map(|d| (d.clone(), q.id.clone()))
            })
            .collect::<BTreeSet<_>>();
        let layers = dependency_layers(&ids, &edges);
        let nodes = quests
            .into_iter()
            .map(|q| {
                let requests = open
                    .iter()
                    .filter(|o| o.quest.as_ref() == Some(&q.id))
                    .collect::<Vec<_>>();
                (
                    q.id.clone(),
                    QuestNode {
                        title: q.title.clone(),
                        state: q.state,
                        assignment: q.assignment.as_ref().map(|a| a.name.clone()),
                        human_requests: requests.len(),
                        eligible_requests: requests
                            .iter()
                            .filter(|o| eligible.contains(&o.id))
                            .count(),
                        review_requests: requests
                            .iter()
                            .filter(|o| o.kind == ObligationKind::Review)
                            .count(),
                        artifact_count: inventory
                            .artifacts
                            .values()
                            .filter(|a| a.quest.as_ref() == Some(&q.id))
                            .count(),
                        layer: layers.get(&q.id).copied(),
                        cyclic: cycle_includes(&q.id, &edges),
                        missing_dependencies: q
                            .dependencies
                            .iter()
                            .filter(|d| work.quest(d).is_none())
                            .cloned()
                            .collect(),
                        outside_dependencies: q
                            .dependencies
                            .iter()
                            .filter(|d| !ids.contains(*d) && work.quest(d).is_some())
                            .cloned()
                            .collect(),
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        Some(Self {
            campaign: inventory.campaigns[id].clone(),
            source: source.clone(),
            freshness: state.freshness,
            selected: selected
                .filter(|id| nodes.contains_key(*id))
                .cloned()
                .or_else(|| nodes.keys().next().cloned()),
            nodes,
            edges,
            human_requests: open.len(),
            artifact_count: inventory
                .artifacts
                .values()
                .filter(|a| &a.campaign == id)
                .count(),
        })
    }
    pub fn has_cycle(&self) -> bool {
        self.nodes.values().any(|n| n.cyclic)
    }
}
fn dependency_layers(
    ids: &BTreeSet<QuestId>,
    edges: &BTreeSet<(QuestId, QuestId)>,
) -> BTreeMap<QuestId, usize> {
    let mut layers = BTreeMap::new();
    loop {
        let next = ids
            .iter()
            .filter(|id| !layers.contains_key(*id))
            .filter_map(|id| {
                let dependencies = edges
                    .iter()
                    .filter(|(_, to)| to == id)
                    .map(|(from, _)| from)
                    .collect::<Vec<_>>();
                dependencies
                    .iter()
                    .all(|d| layers.contains_key(*d))
                    .then(|| {
                        let layer = dependencies
                            .iter()
                            .filter_map(|d| layers.get(*d))
                            .max()
                            .map_or(0, |v| v + 1);
                        (id.clone(), layer)
                    })
            })
            .collect::<Vec<_>>();
        if next.is_empty() {
            break;
        }
        layers.extend(next);
    }
    layers
}
fn cycle_includes(start: &QuestId, edges: &BTreeSet<(QuestId, QuestId)>) -> bool {
    let mut pending = edges
        .iter()
        .filter(|(_, to)| to == start)
        .map(|(from, _)| from)
        .collect::<Vec<_>>();
    let mut visited = BTreeSet::new();
    while let Some(id) = pending.pop() {
        if id == start {
            return true;
        }
        if visited.insert(id) {
            pending.extend(
                edges
                    .iter()
                    .filter(|(_, to)| to == id)
                    .map(|(from, _)| from),
            );
        }
    }
    false
}

pub const fn freshness_label(freshness: Freshness) -> &'static str {
    match freshness {
        Freshness::Current => "current",
        Freshness::Cached => "cached / unverified",
        Freshness::Unavailable => "unavailable / last known",
        Freshness::Reconcile => "reconciliation required",
    }
}

pub fn details(model: &Model) -> Vec<String> {
    let mut lines = view::details(model);
    let Some(graph) = CampaignGraph::from_model(model) else {
        return lines;
    };
    if model.work_selection().is_none() {
        lines = vec![
            format!(
                "Campaign: {} | {:?}",
                graph.campaign.name, graph.campaign.state
            ),
            format!(
                "Objective: {}",
                graph.campaign.objective.as_deref().unwrap_or("Unknown")
            ),
            format!(
                "Source: {} | {}",
                graph.source.0,
                freshness_label(graph.freshness)
            ),
            "No quests are supplied for this campaign.".into(),
        ];
        view::append_source_notice(&mut lines, model, Some(&graph.source));
    }
    if let Some(id) = &graph.selected {
        lines.push(format!("Quest identity: {}", id.as_str()));
        if graph.nodes[id].cyclic {
            lines.push("Dependency cycle includes this quest; no execution order inferred.".into());
        }
        for dep in &graph.nodes[id].outside_dependencies {
            lines.push(format!("Prerequisite outside campaign: {}", dep.as_str()));
        }
        for (_, to) in graph.edges.iter().filter(|(from, _)| from == id) {
            lines.push(format!(
                "Dependent: {} | {:?}",
                graph.nodes[to].title, graph.nodes[to].state
            ));
        }
        for q in model
            .work()
            .sources
            .values()
            .flat_map(|s| s.inventory.quests.values())
            .filter(|q| q.campaign != graph.campaign.id && q.dependencies.contains(id))
        {
            lines.push(format!(
                "Outside-campaign dependent: {} | {} | {:?}",
                q.id.as_str(),
                q.title,
                q.state
            ));
        }
        for o in model
            .work()
            .sources
            .values()
            .flat_map(|s| s.inventory.obligations.values())
            .filter(|o| o.quest.as_ref() == Some(id) && o.state == ObligationState::Open)
        {
            lines.extend(view::obligation_details(model, &o.id));
        }
    }
    for o in model
        .work()
        .sources
        .values()
        .flat_map(|s| s.inventory.obligations.values())
        .filter(|o| {
            o.campaign == graph.campaign.id && o.quest.is_none() && o.state == ObligationState::Open
        })
    {
        lines.push("Campaign summons (no quest binding):".into());
        lines.extend(view::obligation_details(model, &o.id));
    }
    for artifact in model
        .work()
        .sources
        .values()
        .flat_map(|s| s.inventory.artifacts.values())
        .filter(|a| a.campaign == graph.campaign.id && a.quest.is_none())
    {
        lines.push("Campaign spoils (no quest binding):".into());
        lines.extend(view::artifact_details(artifact));
    }
    lines
}
