use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Rect},
    style::Style,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    app::{CharacterSet, ConnectionState, DisplayPreferences, Model},
    domain::AgentKey,
    ui::{
        delve_projection::{footer_height, visible_agent_keys},
        pixel::{ColorRole, Palette},
        theatre::frame_for,
        widgets::render_chamber,
    },
};

const ASCII_BORDER: border::Set<'static> = border::Set {
    top_left: "+",
    top_right: "+",
    bottom_left: "+",
    bottom_right: "+",
    vertical_left: "|",
    vertical_right: "|",
    horizontal_top: "-",
    horizontal_bottom: "-",
};

#[derive(Clone, Copy)]
struct DelveStyles {
    ink: Style,
    accent: Style,
    muted: Style,
    wall: Style,
    floor: Style,
}

impl DelveStyles {
    fn from_preferences(preferences: DisplayPreferences) -> Self {
        let palette = Palette::from(preferences.color_mode);
        let background = palette.resolve(ColorRole::DarkStone);
        Self {
            ink: Style::new()
                .fg(palette.resolve(ColorRole::Parchment))
                .bg(background),
            accent: Style::new()
                .fg(palette.resolve(ColorRole::RuneGlow))
                .bg(background),
            muted: Style::new()
                .fg(palette.resolve(ColorRole::Fog))
                .bg(background),
            wall: Style::new()
                .fg(palette.resolve(ColorRole::Stone))
                .bg(background),
            floor: Style::new()
                .fg(palette.resolve(ColorRole::Timber))
                .bg(background),
        }
    }
}

pub(crate) fn render(frame: &mut Frame<'_>, model: &Model) {
    let area = frame.area();
    let styles = DelveStyles::from_preferences(*model.preferences());
    if area.width < 4 || area.height < 3 {
        frame.render_widget(Paragraph::new("D").style(styles.accent), area);
        return;
    }
    let visible_agents = visible_agent_keys(model, area);

    let footer_height = footer_height(area.width);
    let [body, footer] =
        ratatui::layout::Layout::vertical([Constraint::Min(1), Constraint::Length(footer_height)])
            .areas(area);
    let title = format!(
        " QUESTMANCER DELVES - {} ",
        connection_label(model.connection())
    );
    let mut block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(styles.accent)
        .style(styles.ink);
    if model.preferences().character_set == crate::app::CharacterSet::Ascii {
        block = block.border_set(ASCII_BORDER);
    }
    let inner = block.inner(body);
    frame.render_widget(block, body);

    if model.domain().agents.is_empty() {
        render_empty(frame, inner, styles);
    } else if inner.width >= 78 {
        render_connected_delves(frame, inner, model, styles, &visible_agents);
        if !matches!(model.connection(), ConnectionState::Reconnecting { .. }) {
            render_connection_overlay(frame, inner, model.connection(), styles);
        }
    } else {
        render_compact_list(frame, inner, model, &visible_agents);
        render_connection_overlay(
            frame,
            Rect::new(inner.x, inner.y, inner.width, 1),
            model.connection(),
            styles,
        );
    }

    render_footer(frame, footer, model, styles);
}

#[allow(clippy::too_many_lines)]
fn render_connected_delves(
    frame: &mut Frame<'_>,
    area: Rect,
    model: &Model,
    styles: DelveStyles,
    visible_agents: &std::collections::BTreeSet<AgentKey>,
) {
    use crate::ui::delve_scene::layout_delves;
    let sites = if model.domain().campaigns.is_empty() {
        let mut derived = std::collections::BTreeMap::new();
        for agent in model.domain().agents.values() {
            let id = agent.workspace_id.clone();
            derived
                .entry(id.clone())
                .or_insert_with(|| crate::domain::Campaign {
                    workspace_id: id.clone(),
                    label: id.to_string(),
                    cwd: std::path::PathBuf::new(),
                    party: Vec::new(),
                })
                .party
                .push(agent.key.clone());
        }
        derived
    } else {
        model.domain().campaigns.clone()
    };
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
        let active_delve = delves
            .iter()
            .find(|delve| {
                delve
                    .adventurers
                    .iter()
                    .any(|key| visible_agents.contains(key))
            })
            .unwrap_or(&delves[0]);
        render_delve_architecture(
            frame,
            active_area,
            &active_delve.workspace_id,
            active_delve.variant,
            true,
            styles,
        );
        if let Some(site) = sites.get(&active_delve.workspace_id) {
            let mut active_site = site.clone();
            active_site.party.clone_from(&active_delve.adventurers);
            let active_sites = std::collections::BTreeMap::from([(
                active_delve.workspace_id.clone(),
                active_site,
            )]);
            let remapped = layout_delves(
                &active_sites,
                &model.domain().agents,
                active_area,
                Some(&active_delve.workspace_id),
            )
            .into_iter()
            .next()
            .map(|delve| delve.chambers)
            .unwrap_or_default();
            render_reconnecting_fog_in_room(
                frame,
                active_area,
                &remapped,
                model.connection(),
                styles,
            );
            render_route_home(
                frame,
                active_area,
                &remapped,
                model.preferences().character_set,
                styles,
            );
            for (index, key) in active_delve.adventurers.iter().enumerate() {
                if !visible_agents.contains(key) {
                    continue;
                }
                if let (Some(agent), Some(anchor)) =
                    (model.domain().agents.get(key), remapped.get(index).copied())
                {
                    render_chamber(
                        frame,
                        anchor,
                        agent,
                        frame_for(agent, model.now(), model.preferences()),
                        model.selected_agent_key() == Some(key),
                        model.preferences(),
                    );
                }
            }
        }
        let mut labels = sites
            .keys()
            .map(|id| format!("[{}]", id.as_str()))
            .collect::<Vec<_>>();
        if delves.len() > sites.len() {
            labels.push("[more chambers]".to_owned());
        }
        frame.render_widget(
            Paragraph::new(labels.join(" ")).style(styles.muted),
            Rect::new(
                area.x,
                area.bottom().saturating_sub(strip_height),
                area.width,
                strip_height,
            ),
        );
        return;
    }
    for delve in &delves {
        let active = selected_workspace.as_ref() == Some(&delve.workspace_id);
        render_delve_architecture(
            frame,
            delve.rect,
            &delve.workspace_id,
            delve.variant,
            active,
            styles,
        );
    }
    render_reconnecting_fog(frame, &delves, model.connection(), styles);
    render_connected_routes(frame, &delves, model.preferences().character_set, styles);
    for delve in &delves {
        if !sites.contains_key(&delve.workspace_id) {
            continue;
        }
        for (index, key) in delve.adventurers.iter().enumerate() {
            if !visible_agents.contains(key) {
                continue;
            }
            let Some(agent) = model.domain().agents.get(key) else {
                continue;
            };
            let Some(anchor) = delve.chambers.get(index).copied() else {
                continue;
            };
            render_chamber(
                frame,
                anchor,
                agent,
                frame_for(agent, model.now(), model.preferences()),
                model.selected_agent_key() == Some(key),
                model.preferences(),
            );
        }
    }
}

fn render_reconnecting_fog(
    frame: &mut Frame<'_>,
    delves: &[crate::ui::delve_scene::CampaignDelve],
    connection: &ConnectionState,
    styles: DelveStyles,
) {
    for delve in delves {
        render_reconnecting_fog_in_room(frame, delve.rect, &delve.chambers, connection, styles);
    }
}

fn render_reconnecting_fog_in_room(
    frame: &mut Frame<'_>,
    area: Rect,
    chambers: &[crate::ui::delve_scene::ChamberAnchor],
    connection: &ConnectionState,
    styles: DelveStyles,
) {
    let ConnectionState::Reconnecting { attempt } = connection else {
        return;
    };
    if area.width < 9 || area.height < 5 {
        return;
    }
    let home_row = unused_row(area, chambers, area.y.saturating_add(2));
    let preferred = area.y.saturating_add(3);
    let start = area.y.saturating_add(1);
    let end = area.bottom().saturating_sub(1);
    let Some(y) = (start..end)
        .filter(|candidate| Some(*candidate) != home_row && row_is_unused(*candidate, chambers))
        .min_by_key(|candidate| candidate.abs_diff(preferred))
    else {
        return;
    };
    let fog = format!("~ FOG ~ RECONNECTING #{attempt} ~ LAST TALES PRESERVED ~");
    frame.render_widget(
        Paragraph::new(fog).style(styles.muted),
        Rect::new(area.x.saturating_add(2), y, area.width.saturating_sub(4), 1),
    );
}

fn render_connected_routes(
    frame: &mut Frame<'_>,
    delves: &[crate::ui::delve_scene::CampaignDelve],
    character_set: CharacterSet,
    styles: DelveStyles,
) {
    let Some(first) = delves.first() else {
        return;
    };
    render_route_home(frame, first.rect, &first.chambers, character_set, styles);
    for pair in delves.windows(2) {
        let [previous, next] = pair else {
            continue;
        };
        if previous.rect.y == next.rect.y {
            render_adjacent_opening(frame, previous, next, character_set, styles);
        } else {
            render_row_wrap_corridor(frame, previous, next, character_set, styles);
        }
    }
}

fn render_route_home(
    frame: &mut Frame<'_>,
    area: Rect,
    chambers: &[crate::ui::delve_scene::ChamberAnchor],
    character_set: CharacterSet,
    styles: DelveStyles,
) {
    if area.width < 9 || area.height < 4 {
        return;
    }
    let Some(y) = unused_row(area, chambers, area.y.saturating_add(2)) else {
        return;
    };
    let path = match character_set {
        CharacterSet::Ascii => "<--- HOME",
        CharacterSet::Unicode => "◄─── HOME",
    };
    frame.render_widget(
        Paragraph::new(path).style(styles.accent),
        Rect::new(area.x, y, area.width.min(9), 1),
    );
}

fn render_adjacent_opening(
    frame: &mut Frame<'_>,
    left: &crate::ui::delve_scene::CampaignDelve,
    right: &crate::ui::delve_scene::CampaignDelve,
    character_set: CharacterSet,
    styles: DelveStyles,
) {
    if left.rect.right() != right.rect.x || left.rect.width < 2 || right.rect.width < 2 {
        return;
    }
    let start_y = left.rect.y.max(right.rect.y).saturating_add(1);
    let end_y = left
        .rect
        .bottom()
        .min(right.rect.bottom())
        .saturating_sub(1);
    let preferred = start_y.saturating_add(end_y.saturating_sub(start_y) / 2);
    let Some(y) = (start_y..end_y)
        .min_by_key(|candidate| candidate.abs_diff(preferred))
        .filter(|candidate| {
            row_is_unused(*candidate, &left.chambers) && row_is_unused(*candidate, &right.chambers)
        })
        .or_else(|| {
            (start_y..end_y).find(|candidate| {
                row_is_unused(*candidate, &left.chambers)
                    && row_is_unused(*candidate, &right.chambers)
            })
        })
    else {
        return;
    };
    let x = left.rect.right().saturating_sub(2);
    let glyphs = match character_set {
        CharacterSet::Ascii => "----",
        CharacterSet::Unicode => "────",
    };
    frame.render_widget(
        Paragraph::new(glyphs).style(styles.accent),
        Rect::new(x, y, 4, 1),
    );
}

fn render_row_wrap_corridor(
    frame: &mut Frame<'_>,
    previous: &crate::ui::delve_scene::CampaignDelve,
    next: &crate::ui::delve_scene::CampaignDelve,
    character_set: CharacterSet,
    styles: DelveStyles,
) {
    if previous.rect.bottom() != next.rect.y || previous.rect.height < 3 || next.rect.height < 3 {
        return;
    }
    let seam_y = next.rect.y;
    let previous_rows = [seam_y.saturating_sub(2), seam_y.saturating_sub(1)];
    let next_rows = [seam_y, seam_y.saturating_add(1)];
    let Some(previous_x) = unused_column(previous, &previous_rows) else {
        return;
    };
    let Some(next_x) = unused_column(next, &next_rows) else {
        return;
    };
    let start_x = previous_x.min(next_x);
    let end_x = previous_x.max(next_x);
    let width = end_x.saturating_sub(start_x).saturating_add(1);
    let horizontal = match character_set {
        CharacterSet::Ascii => "-".repeat(usize::from(width)),
        CharacterSet::Unicode => "─".repeat(usize::from(width)),
    };
    frame.render_widget(
        Paragraph::new(horizontal).style(styles.accent),
        Rect::new(start_x, seam_y.saturating_sub(1), width, 1),
    );
    let (previous_turn, next_turn) = match character_set {
        CharacterSet::Ascii => ("|\n+", "+\n|\n|"),
        CharacterSet::Unicode => ("│\n└", "┐\n│\n│"),
    };
    frame.render_widget(
        Paragraph::new(previous_turn).style(styles.accent),
        Rect::new(previous_x, seam_y.saturating_sub(2), 1, 2),
    );
    frame.render_widget(
        Paragraph::new(next_turn).style(styles.accent),
        Rect::new(next_x, seam_y.saturating_sub(1), 1, 3),
    );
}

fn unused_row(
    area: Rect,
    chambers: &[crate::ui::delve_scene::ChamberAnchor],
    preferred: u16,
) -> Option<u16> {
    let start = area.y.saturating_add(1);
    let end = area.bottom().saturating_sub(1);
    (start..end)
        .filter(|candidate| row_is_unused(*candidate, chambers))
        .min_by_key(|candidate| candidate.abs_diff(preferred))
}

fn unused_column(delve: &crate::ui::delve_scene::CampaignDelve, rows: &[u16]) -> Option<u16> {
    let start = delve.rect.x.saturating_add(2);
    let end = delve.rect.right().saturating_sub(2);
    let preferred = delve.rect.x.saturating_add(delve.rect.width / 2);
    (start..end)
        .filter(|candidate| {
            delve.chambers.iter().all(|chamber| {
                rows.iter().all(|row| {
                    *candidate < chamber.x
                        || *candidate >= chamber.x.saturating_add(chamber.width)
                        || *row < chamber.y
                        || *row >= chamber.y.saturating_add(chamber.height)
                })
            })
        })
        .min_by_key(|candidate| candidate.abs_diff(preferred))
}

fn row_is_unused(y: u16, chambers: &[crate::ui::delve_scene::ChamberAnchor]) -> bool {
    chambers
        .iter()
        .all(|chamber| y < chamber.y || y >= chamber.y.saturating_add(chamber.height))
}

fn render_delve_architecture(
    frame: &mut Frame<'_>,
    area: Rect,
    workspace: &crate::domain::WorkspaceId,
    variant: crate::ui::delve_scene::DelveVariant,
    active: bool,
    styles: DelveStyles,
) {
    if area.is_empty() {
        return;
    }
    let label = format!(" {} ", workspace.as_str());
    let architecture = DelveArchitecture::for_variant(variant);
    let mut lines = Vec::with_capacity(usize::from(area.height));
    lines.push(Line::styled(
        format!(
            "+{label:-^width$}+",
            width = usize::from(area.width.saturating_sub(2))
        ),
        styles.wall,
    ));
    for row in 1..area.height {
        let text = if row == 1 {
            architecture_row(area, architecture.name)
        } else if row == 2 {
            architecture_row(
                area,
                if active {
                    "TORCHLIT PATH"
                } else {
                    "SHADOWED PASSAGE"
                },
            )
        } else if row + 2 >= area.height {
            architecture_row(area, "== == ==")
        } else if row == area.height / 2 {
            architecture_row(area, architecture.connection)
        } else if row == 3 {
            architecture_row(area, architecture.wall)
        } else if row == 4 {
            architecture_row(area, architecture.furniture)
        } else if row == 5 {
            architecture_row(area, architecture.detail)
        } else {
            format!(
                "|{:width$}|",
                "",
                width = usize::from(area.width.saturating_sub(2))
            )
        };
        lines.push(Line::styled(
            text,
            if row + 2 >= area.height {
                styles.floor
            } else {
                styles.wall
            },
        ));
    }
    frame.render_widget(Paragraph::new(Text::from(lines)).style(styles.ink), area);
}

struct DelveArchitecture {
    name: &'static str,
    wall: &'static str,
    furniture: &'static str,
    detail: &'static str,
    connection: &'static str,
}

impl DelveArchitecture {
    const fn for_variant(variant: crate::ui::delve_scene::DelveVariant) -> Self {
        match variant {
            crate::ui::delve_scene::DelveVariant::ForgottenLibrary => Self {
                name: "FORGOTTEN LIBRARY",
                wall: "SHELVES / READING ALCOVE",
                furniture: "==== RUNE TABLE ====",
                detail: "SHELVES / CONNECTING ARCH",
                connection: "READING ALCOVE / CONNECTING ARCH",
            },
            crate::ui::delve_scene::DelveVariant::MossyUndercroft => Self {
                name: "MOSSY UNDERCROFT",
                wall: "STONE WALL / ROOT BREAK",
                furniture: "#\\__ CAMP JUNCTION __/#",
                detail: "ROOT BREAK / DESCENDING PASSAGE",
                connection: "CAMP JUNCTION / DESCENDING PASSAGE",
            },
            crate::ui::delve_scene::DelveVariant::OldWatchtower => Self {
                name: "OLD WATCHTOWER",
                wall: "MAP WALL / SIGNAL BRAZIER",
                furniture: "[STAIR] [MAP] [BRAZIER]",
                detail: "NARROW LANDING / SIGNAL BRAZIER",
                connection: "STAIR / NARROW LANDING",
            },
        }
    }
}

fn architecture_row(area: Rect, content: &str) -> String {
    format!(
        "| {content:width$}|",
        width = usize::from(area.width.saturating_sub(3))
    )
}

fn render_compact_list(
    frame: &mut Frame<'_>,
    area: Rect,
    model: &Model,
    visible_agents: &std::collections::BTreeSet<AgentKey>,
) {
    let count = visible_agents.len();
    if count == 0 || area.height <= 1 {
        return;
    }
    let list = Rect::new(
        area.x,
        area.y.saturating_add(1),
        area.width,
        area.height.saturating_sub(1),
    );
    let item_height = (list.height / u16::try_from(count).unwrap_or(1)).max(1);
    for (index, (key, agent)) in model
        .domain()
        .agents
        .iter()
        .filter(|(key, _agent)| visible_agents.contains(*key))
        .enumerate()
    {
        let y = list.y.saturating_add(
            u16::try_from(index)
                .unwrap_or_default()
                .saturating_mul(item_height),
        );
        if y >= list.bottom() {
            break;
        }
        render_chamber(
            frame,
            Rect::new(
                list.x,
                y,
                list.width,
                item_height.min(list.bottom().saturating_sub(y)),
            ),
            agent,
            frame_for(agent, model.now(), model.preferences()),
            model.selected_agent_key() == Some(key),
            model.preferences(),
        );
    }
}

fn render_empty(frame: &mut Frame<'_>, area: Rect, styles: DelveStyles) {
    let message = Text::from(vec![
        Line::from(""),
        Line::styled("All Delves await a party", styles.accent),
        Line::from(""),
        Line::styled("Start an adventurer to open a chamber", styles.muted),
    ]);
    frame.render_widget(
        Paragraph::new(message)
            .alignment(Alignment::Center)
            .style(styles.ink),
        area,
    );
}

fn render_footer(frame: &mut Frame<'_>, area: Rect, model: &Model, styles: DelveStyles) {
    if area.height > 1 {
        render_narrow_footer(frame, area, model, styles);
        return;
    }
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
    frame.render_widget(
        Paragraph::new(actions.join("  "))
            .alignment(if area.width >= 100 {
                Alignment::Center
            } else {
                Alignment::Left
            })
            .style(styles.muted),
        area,
    );
}

fn render_narrow_footer(frame: &mut Frame<'_>, area: Rect, model: &Model, styles: DelveStyles) {
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
    let mut lines = pack_footer_actions(&global, area.width);
    lines.extend(pack_footer_actions(&selected, area.width));
    frame.render_widget(
        Paragraph::new(Text::from(
            lines.into_iter().map(Line::from).collect::<Vec<_>>(),
        ))
        .style(styles.muted),
        area,
    );
}

fn pack_footer_actions(actions: &[&str], width: u16) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for action in actions {
        let separator = usize::from(!line.is_empty());
        if !line.is_empty()
            && line
                .len()
                .saturating_add(separator)
                .saturating_add(action.len())
                > usize::from(width)
        {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(action);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

fn render_connection_overlay(
    frame: &mut Frame<'_>,
    area: Rect,
    connection: &ConnectionState,
    styles: DelveStyles,
) {
    let label = match connection {
        ConnectionState::Offline => Some("DISCONNECTED - LAST TALES PRESERVED".to_owned()),
        ConnectionState::Reconnecting { attempt } => Some(format!(
            "FOG - RECONNECTING #{attempt} - LAST TALES PRESERVED"
        )),
        ConnectionState::Incompatible { expected, actual } => Some(format!(
            "INCOMPATIBLE PROTOCOL {actual} - NEED {expected} - LAST TALES PRESERVED"
        )),
        ConnectionState::Connecting | ConnectionState::Connected => None,
    };
    let Some(label) = label else {
        return;
    };
    if area.is_empty() {
        return;
    }
    frame.render_widget(
        Paragraph::new(label)
            .alignment(Alignment::Center)
            .style(styles.accent),
        Rect::new(area.x, area.bottom().saturating_sub(1), area.width, 1),
    );
}

fn connection_label(state: &ConnectionState) -> String {
    match state {
        ConnectionState::Offline => "offline".to_owned(),
        ConnectionState::Connecting => "entering the depths".to_owned(),
        ConnectionState::Connected => "paths joined".to_owned(),
        ConnectionState::Reconnecting { attempt } => format!("reconnecting #{attempt}"),
        ConnectionState::Incompatible { expected, actual } => {
            format!("protocol {actual} unsupported - need {expected}")
        }
    }
}
