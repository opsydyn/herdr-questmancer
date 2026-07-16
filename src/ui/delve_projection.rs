use std::collections::{BTreeMap, BTreeSet};

use ratatui::{
    layout::{Constraint, Rect},
    widgets::{Block, Borders},
};

use crate::{
    app::Model,
    domain::{AgentKey, Campaign},
    ui::delve_scene::{DelveVariant, layout_delves},
};

/// Projects the adventurers that the Delve renderer can draw in `terminal_area`.
///
/// This is the shared visibility boundary for rendering and animation scheduling.
/// It intentionally follows responsive campaign selection and compact paging, and
/// returns no adventurers for empty or zero-sized surfaces.
#[must_use]
pub fn visible_agent_keys(model: &Model, terminal_area: Rect) -> BTreeSet<AgentKey> {
    render_projection(model, terminal_area).visible_agent_keys
}

#[derive(Debug)]
pub(crate) struct DelveRenderProjection {
    pub(crate) footer_lines: Vec<String>,
    pub(crate) visible_agent_keys: BTreeSet<AgentKey>,
    pub(crate) chamber_areas: BTreeMap<AgentKey, Rect>,
    pub(crate) visible_variants: BTreeSet<DelveVariant>,
    pub(crate) connected_scene_visible: bool,
}

pub(crate) fn render_projection(model: &Model, terminal_area: Rect) -> DelveRenderProjection {
    if terminal_area.width < 4 || terminal_area.height < 3 || model.domain().agents.is_empty() {
        return DelveRenderProjection {
            footer_lines: footer_lines(model, terminal_area.width),
            visible_agent_keys: BTreeSet::new(),
            chamber_areas: BTreeMap::new(),
            visible_variants: BTreeSet::new(),
            connected_scene_visible: false,
        };
    }

    let footer_lines = footer_lines(model, terminal_area.width);
    let footer_height = u16::try_from(footer_lines.len()).unwrap_or(u16::MAX);
    let [body, _footer] =
        ratatui::layout::Layout::vertical([Constraint::Min(1), Constraint::Length(footer_height)])
            .areas(terminal_area);
    let inner = Block::default().borders(Borders::ALL).inner(body);

    let (chamber_areas, visible_variants, connected_scene_visible) = if inner.width >= 78 {
        project_connected_chambers(model, inner)
    } else {
        (
            project_compact_chambers(model, inner),
            BTreeSet::new(),
            false,
        )
    };
    let visible_agent_keys = chamber_areas.keys().cloned().collect();
    DelveRenderProjection {
        footer_lines,
        visible_agent_keys,
        chamber_areas,
        visible_variants,
        connected_scene_visible,
    }
}

fn footer_lines(model: &Model, width: u16) -> Vec<String> {
    if width <= 80 {
        let (global, selected) = footer_action_groups(model);
        let mut lines = pack_footer_actions(&global, width, " ");
        lines.extend(pack_footer_actions(&selected, width, " "));
        return lines;
    }
    pack_footer_actions(&wide_footer_actions(model), width, "  ")
}

fn footer_action_groups(model: &Model) -> (Vec<&'static str>, Vec<&'static str>) {
    let mut global = vec!["[1] guild", "[2] delves"];
    let mut selected = Vec::new();
    if !model.domain().agents.is_empty() {
        global.extend(["[j/k] navigate", "[/] search"]);
        if model.selected_agent().is_some() {
            selected.extend(["[enter] observe", "[r] counsel", "[o] refresh"]);
            if model
                .selected_agent()
                .is_some_and(|agent| agent.attention.is_unread())
            {
                selected.push("[space] acknowledge summons");
            }
            if model.reviewr_available() {
                global.push("[v] inspect spoils");
            }
        }
    }
    (global, selected)
}

fn wide_footer_actions(model: &Model) -> Vec<&'static str> {
    let mut actions = vec!["[1] guild", "[2] delves"];
    if !model.domain().agents.is_empty() {
        actions.push("[j/k] navigate");
        if model.selected_agent().is_some() {
            actions.extend(["[enter] observe", "[r] counsel", "[o] refresh"]);
            if model
                .selected_agent()
                .is_some_and(|agent| agent.attention.is_unread())
            {
                actions.push("[space] acknowledge summons");
            }
            if model.reviewr_available() {
                actions.push("[v] inspect spoils");
            }
        }
        actions.push("[/] search");
    }
    actions
}

fn pack_footer_actions(actions: &[&str], width: u16, separator: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for action in actions {
        if !line.is_empty()
            && line
                .len()
                .saturating_add(separator.len())
                .saturating_add(action.len())
                > usize::from(width)
        {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push_str(separator);
        }
        line.push_str(action);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

fn project_connected_chambers(
    model: &Model,
    area: Rect,
) -> (BTreeMap<AgentKey, Rect>, BTreeSet<DelveVariant>, bool) {
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
            return (BTreeMap::new(), BTreeSet::new(), false);
        };
        let Some(site) = sites.get(&active_delve.workspace_id) else {
            return (BTreeMap::new(), BTreeSet::new(), false);
        };
        let mut active_site = site.clone();
        active_site.party.clone_from(&active_delve.adventurers);
        let active_sites = BTreeMap::from([(active_delve.workspace_id.clone(), active_site)]);
        let remapped = layout_delves(
            &active_sites,
            &model.domain().agents,
            active_area,
            Some(&active_delve.workspace_id),
        )
        .into_iter()
        .next()
        .map_or_else(Vec::new, |delve| delve.chambers);
        let chamber_areas = active_delve
            .adventurers
            .iter()
            .cloned()
            .zip(
                remapped
                    .into_iter()
                    .map(|anchor| Rect::new(anchor.x, anchor.y, anchor.width, anchor.height)),
            )
            .collect();
        return (
            chamber_areas,
            [active_delve.variant].into_iter().collect(),
            true,
        );
    }

    let visible_variants = delves.iter().map(|delve| delve.variant).collect();
    let chamber_areas = delves
        .iter()
        .filter(|delve| sites.contains_key(&delve.workspace_id))
        .flat_map(|delve| {
            delve.adventurers.iter().cloned().zip(
                delve
                    .chambers
                    .iter()
                    .map(|anchor| Rect::new(anchor.x, anchor.y, anchor.width, anchor.height)),
            )
        })
        .collect();
    (chamber_areas, visible_variants, !delves.is_empty())
}

fn project_compact_chambers(model: &Model, area: Rect) -> BTreeMap<AgentKey, Rect> {
    if area.height <= 1 {
        return BTreeMap::new();
    }
    let list_height = area.height.saturating_sub(1);
    let capacity = usize::from(list_height / 2).max(1);
    let selected_index = model
        .selected_agent_key()
        .and_then(|selected| model.domain().agents.keys().position(|key| key == selected))
        .unwrap_or_default();
    let page_start = selected_index / capacity * capacity;
    let visible = model
        .domain()
        .agents
        .keys()
        .skip(page_start)
        .take(capacity)
        .cloned()
        .collect::<Vec<_>>();
    let count = visible.len();
    if count == 0 {
        return BTreeMap::new();
    }
    let list = Rect::new(
        area.x,
        area.y.saturating_add(1),
        area.width,
        area.height.saturating_sub(1),
    );
    let item_height = (list.height / u16::try_from(count).unwrap_or(1)).max(1);
    visible
        .into_iter()
        .enumerate()
        .filter_map(|(index, key)| {
            let y = list.y.saturating_add(
                u16::try_from(index)
                    .unwrap_or_default()
                    .saturating_mul(item_height),
            );
            (y < list.bottom()).then(|| {
                (
                    key,
                    Rect::new(
                        list.x,
                        y,
                        list.width,
                        item_height.min(list.bottom().saturating_sub(y)),
                    ),
                )
            })
        })
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
