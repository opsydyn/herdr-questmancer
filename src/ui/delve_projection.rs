use std::collections::{BTreeMap, BTreeSet};

use ratatui::{
    layout::{Constraint, Rect},
    widgets::{Block, Borders},
};

use crate::{
    app::Model,
    domain::{AgentKey, Campaign},
    ui::delve_scene::layout_delves,
};

/// Projects the adventurers that the Delve renderer can draw in `terminal_area`.
///
/// This is the shared visibility boundary for rendering and animation scheduling.
/// It intentionally follows responsive campaign selection and compact paging, and
/// returns no adventurers for empty or zero-sized surfaces.
#[must_use]
pub fn visible_agent_keys(model: &Model, terminal_area: Rect) -> BTreeSet<AgentKey> {
    if terminal_area.width < 4 || terminal_area.height < 3 || model.domain().agents.is_empty() {
        return BTreeSet::new();
    }

    let footer_height = if terminal_area.width <= 80 { 2 } else { 1 };
    let [body, _footer] =
        ratatui::layout::Layout::vertical([Constraint::Min(1), Constraint::Length(footer_height)])
            .areas(terminal_area);
    let inner = Block::default().borders(Borders::ALL).inner(body);

    if inner.width >= 78 {
        visible_connected_agents(model, inner)
    } else {
        visible_compact_agents(model, inner)
    }
}

fn visible_connected_agents(model: &Model, area: Rect) -> BTreeSet<AgentKey> {
    let sites = campaign_sites(model);
    let selected_workspace = model
        .selected_agent()
        .map(|agent| agent.workspace_id.clone());
    let delves = layout_delves(
        &sites,
        &model.domain().agents,
        area,
        selected_workspace.as_ref(),
    );

    if area.width < 116 && delves.len() > 1 {
        let strip_height = 2.min(area.height);
        let active_area = Rect::new(
            area.x,
            area.y,
            area.width,
            area.height.saturating_sub(strip_height),
        );
        let selected_key = model.selected_agent_key();
        let Some(active_delve) = delves
            .iter()
            .find(|delve| selected_key.is_some_and(|key| delve.adventurers.contains(key)))
            .or_else(|| {
                delves
                    .iter()
                    .find(|delve| selected_workspace.as_ref() == Some(&delve.workspace_id))
            })
            .or_else(|| delves.first())
        else {
            return BTreeSet::new();
        };
        let Some(site) = sites.get(&active_delve.workspace_id) else {
            return BTreeSet::new();
        };
        let mut active_site = site.clone();
        active_site.party.clone_from(&active_delve.adventurers);
        let active_sites = BTreeMap::from([(active_delve.workspace_id.clone(), active_site)]);
        let chamber_count = layout_delves(
            &active_sites,
            &model.domain().agents,
            active_area,
            Some(&active_delve.workspace_id),
        )
        .into_iter()
        .next()
        .map_or(0, |delve| delve.chambers.len());
        return active_delve
            .adventurers
            .iter()
            .take(chamber_count)
            .cloned()
            .collect();
    }

    delves
        .iter()
        .filter(|delve| sites.contains_key(&delve.workspace_id))
        .flat_map(|delve| delve.adventurers.iter().take(delve.chambers.len()).cloned())
        .collect()
}

fn visible_compact_agents(model: &Model, area: Rect) -> BTreeSet<AgentKey> {
    if area.height <= 1 {
        return BTreeSet::new();
    }
    let list_height = area.height.saturating_sub(1);
    let capacity = usize::from(list_height / 2).max(1);
    let selected_index = model
        .selected_agent_key()
        .and_then(|selected| model.domain().agents.keys().position(|key| key == selected))
        .unwrap_or_default();
    let page_start = selected_index / capacity * capacity;
    model
        .domain()
        .agents
        .keys()
        .skip(page_start)
        .take(capacity)
        .cloned()
        .collect()
}

fn campaign_sites(model: &Model) -> BTreeMap<crate::domain::WorkspaceId, Campaign> {
    if !model.domain().campaigns.is_empty() {
        return model.domain().campaigns.clone();
    }

    let mut derived = BTreeMap::new();
    for agent in model.domain().agents.values() {
        let id = agent.workspace_id.clone();
        derived
            .entry(id.clone())
            .or_insert_with(|| Campaign {
                workspace_id: id.clone(),
                label: id.to_string(),
                cwd: std::path::PathBuf::new(),
                party: Vec::new(),
            })
            .party
            .push(agent.key.clone());
    }
    derived
}
