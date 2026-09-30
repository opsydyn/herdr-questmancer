//! Pure, literal details. No source queries or inferred execution outcomes.
use super::{Freshness, ObligationState, WorkSelection};
use crate::app::Model;
pub fn items(model: &Model) -> Vec<WorkSelection> {
    let mut obligations = model
        .work()
        .sources
        .values()
        .flat_map(|s| s.inventory.obligations.values())
        .collect::<Vec<_>>();
    obligations.sort_by_key(|o| {
        (
            o.state != ObligationState::Open,
            o.requested_at,
            o.id.clone(),
        )
    });
    obligations
        .into_iter()
        .map(|o| WorkSelection::Obligation(o.id.clone()))
        .chain(
            model
                .work()
                .sources
                .values()
                .flat_map(|s| s.inventory.quests.keys().cloned())
                .map(WorkSelection::Quest),
        )
        .collect()
}
pub fn label(model: &Model, selection: &WorkSelection) -> String {
    match selection {
        WorkSelection::Obligation(id) => model.work().obligation(id).map_or_else(
            || "Removed summons".into(),
            |o| format!("{:?}: {}", o.state, o.prompt),
        ),
        WorkSelection::Quest(id) => model.work().quest(id).map_or_else(
            || "Removed quest".into(),
            |q| format!("{:?}: {}", q.state, q.title),
        ),
    }
}
pub fn selected_quest(model: &Model) -> Option<&super::QuestId> {
    match model.work_selection()? {
        WorkSelection::Quest(id) => Some(id),
        WorkSelection::Obligation(id) => model.work().obligation(id)?.quest.as_ref(),
    }
}
pub fn details(model: &Model) -> Vec<String> {
    let Some(selection) = model.work_selection() else {
        let mut lines = vec![
            "No work source is connected.".into(),
            "Herdr adventurer summons remain available with !.".into(),
            "Work appears when a campaign source supplies it.".into(),
        ];
        if let Some(notice) = model.work_source_notice() {
            lines.insert(0, format!("Source {}: {}", notice.source.0, notice.message));
        }
        return lines;
    };
    let campaign = match selection {
        WorkSelection::Obligation(id) => model.work().obligation(id).map(|o| &o.campaign),
        WorkSelection::Quest(id) => model.work().quest(id).map(|q| &q.campaign),
    };
    let mut lines = Vec::new();
    if let Some(c) = campaign.and_then(|id| model.work().campaign(id)) {
        lines.push(format!("Campaign: {}", c.name));
        lines.push(format!(
            "Objective: {}",
            c.objective.as_deref().unwrap_or("Unknown")
        ));
    }
    let source = model.work().sources.iter().find(|(_, s)| match selection {
        WorkSelection::Obligation(id) => s.inventory.obligations.contains_key(id),
        WorkSelection::Quest(id) => s.inventory.quests.contains_key(id),
    });
    if let Some((id, s)) = source {
        lines.push(format!(
            "Source: {} | {}",
            id.0,
            match s.freshness {
                Freshness::Current => "current",
                Freshness::Cached => "cached / unverified",
                Freshness::Unavailable => "unavailable / last known",
                Freshness::Reconcile => "reconciliation required",
            }
        ));
    }
    append_source_notice(&mut lines, model, source.map(|(id, _)| id));
    append_obligation(&mut lines, model, selection);
    append_quest(&mut lines, model);
    lines
}

pub(crate) fn append_source_notice(
    lines: &mut Vec<String>,
    model: &Model,
    selected: Option<&super::SourceId>,
) {
    if let Some(notice) = model.work_source_notice() {
        lines.push(if selected == Some(&notice.source) {
            notice.message.clone()
        } else {
            format!("Attachment {}: {}", notice.source.0, notice.message)
        });
    }
}

fn append_obligation(lines: &mut Vec<String>, model: &Model, selection: &WorkSelection) {
    if let WorkSelection::Obligation(id) = selection {
        lines.extend(obligation_details(model, id));
    }
}

pub fn obligation_details(model: &Model, id: &super::ObligationId) -> Vec<String> {
    let mut lines = Vec::new();
    append_obligation_id(&mut lines, model, id);
    lines
}
fn append_obligation_id(lines: &mut Vec<String>, model: &Model, id: &super::ObligationId) {
    if let Some(o) = model.work().obligation(id) {
        lines.push(String::new());
        lines.push(format!(
            "Summons: {:?} | {:?} | {:?}",
            o.kind, o.state, o.responsible
        ));
        lines.push(o.prompt.clone());
        if let Some(context) = &o.context {
            lines.push(context.clone());
        }
        lines.push(format!("Action: {}", o.action));
        lines.push(format!("Settled only by: {}", o.resolution));
        if let Some(intent) = model
            .work()
            .intent
            .get(id)
            .filter(|i| i.incarnation == o.incarnation && o.state == ObligationState::Open)
        {
            if intent.seen {
                lines.push("Seen locally; source obligation is unchanged.".into());
            }
            if let Some(until) = intent.deferred_until
                && until > model.now()
            {
                lines.push(format!(
                    "Set aside locally for another {}m; still open at source.",
                    model.now().elapsed_until(until).as_secs().div_ceil(60)
                ));
            }
        }
        if model.pending_work().is_some() {
            lines.push("Saving local intent; not yet acknowledged.".into());
        }
    }
}

fn append_quest(lines: &mut Vec<String>, model: &Model) {
    if let Some(id) = selected_quest(model)
        && let Some(q) = model.work().quest(id)
    {
        lines.push(String::new());
        lines.push(format!(
            "Quest: {} | {} | {:?}",
            q.title,
            quest_kind(q.kind),
            q.state
        ));
        if let Some(description) = &q.description {
            lines.push(description.clone());
        }
        lines.push(format!(
            "Assigned adventurer (source history): {}",
            q.assignment
                .as_ref()
                .map_or("Unassigned", |a| a.name.as_str())
        ));
        for dep in &q.dependencies {
            lines.push(model.work().quest(dep).map_or_else(
                || format!("Dependency: {} | missing", dep.as_str()),
                |d| format!("Dependency: {} | {:?}", d.title, d.state),
            ));
        }
        if !q.dependencies.is_empty() {
            lines.push(format!(
                "Dependency evidence: {}",
                if model.work().dependencies_ready(id) {
                    "complete; source still owns readiness"
                } else {
                    "incomplete, missing or cyclic"
                }
            ));
        }
        for artifact in model
            .work()
            .sources
            .values()
            .flat_map(|s| s.inventory.artifacts.values())
            .filter(|a| a.quest.as_ref() == Some(id))
        {
            lines.extend(artifact_details(artifact));
        }
        lines.push(if super::bound_agent(model, id).is_some() {
            "Enter: observe verified adventurer pane".into()
        } else {
            "Observe unavailable: no verified live adventurer binding.".into()
        });
    }
}

const fn quest_kind(kind: super::QuestKind) -> &'static str {
    match kind {
        super::QuestKind::Unspecified => "Unspecified",
        super::QuestKind::Scout => "Expedition",
        super::QuestKind::Ship => "Commission",
        super::QuestKind::Review => "Inspection",
        super::QuestKind::Validate => "Trial",
    }
}

pub fn artifact_details(artifact: &super::Artifact) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!("Spoils: {} | {:?}", artifact.title, artifact.kind));
    if let Some(uri) = &artifact.uri {
        lines.push(uri.clone());
    }
    lines.push(format!(
        "Head: {} | Validation: {}",
        artifact.head_revision.as_deref().unwrap_or("Unknown"),
        artifact
            .validation
            .as_ref()
            .filter(|v| artifact.head_revision.as_ref() == Some(&v.head_revision))
            .map_or("Unknown", |v| v.summary.as_str())
    ));
    lines
}
