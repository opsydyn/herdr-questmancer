use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Rect},
    style::Style,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    app::{ConnectionState, DisplayPreferences, Model},
    ui::{
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
        let background = palette.resolve(ColorRole::PanelBackground);
        Self {
            ink: Style::new()
                .fg(palette.resolve(ColorRole::Highlight))
                .bg(background),
            accent: Style::new()
                .fg(palette.resolve(ColorRole::CrtGlow))
                .bg(background),
            muted: Style::new()
                .fg(palette.resolve(ColorRole::CrtCase))
                .bg(background),
            wall: Style::new()
                .fg(palette.resolve(ColorRole::RoomWall))
                .bg(background),
            floor: Style::new()
                .fg(palette.resolve(ColorRole::RoomFloor))
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

    let footer_height = if area.width <= 80 { 2 } else { 1 };
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
    } else {
        if inner.width >= 78 {
            render_connected_delves(frame, inner, model, styles);
        } else {
            render_compact_list(frame, inner, model);
        }
        render_connection_overlay(frame, inner, model.connection(), styles);
    }

    render_footer(frame, footer, model, styles);
}

#[allow(clippy::too_many_lines)]
fn render_connected_delves(frame: &mut Frame<'_>, area: Rect, model: &Model, styles: DelveStyles) {
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
        let selected_key = model.selected_agent_key();
        let active_delve = delves
            .iter()
            .find(|delve| selected_key.is_some_and(|key| delve.adventurers.contains(key)))
            .or_else(|| {
                delves
                    .iter()
                    .find(|delve| selected_workspace.as_ref() == Some(&delve.workspace_id))
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
            let active_sites = std::collections::BTreeMap::from([(
                active_delve.workspace_id.clone(),
                site.clone(),
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
            for (index, key) in active_delve.adventurers.iter().enumerate() {
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
    for (index, delve) in delves.iter().enumerate() {
        let active = selected_workspace.as_ref() == Some(&delve.workspace_id);
        render_delve_architecture(
            frame,
            delve.rect,
            &delve.workspace_id,
            delve.variant,
            active,
            styles,
        );
        if index > 0 {
            let previous = delves[index - 1].rect;
            let (transition, glyphs) = if previous.y == delve.rect.y {
                let x = previous.right().saturating_sub(1);
                let glyphs = if model.preferences().character_set == crate::app::CharacterSet::Ascii
                {
                    "|\n+\n|"
                } else {
                    "│\n╫\n│"
                };
                (Rect::new(x, delve.rect.y, 1, delve.rect.height), glyphs)
            } else {
                let y = previous.bottom().saturating_sub(1);
                let glyphs = if model.preferences().character_set == crate::app::CharacterSet::Ascii
                {
                    "---+---"
                } else {
                    "───╫───"
                };
                (
                    Rect::new(delve.rect.x, y, delve.rect.width.min(7), 1),
                    glyphs,
                )
            };
            frame.render_widget(Paragraph::new(glyphs).style(styles.accent), transition);
        }
        if !sites.contains_key(&delve.workspace_id) {
            continue;
        }
        for (index, key) in delve.adventurers.iter().enumerate() {
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

fn render_compact_list(frame: &mut Frame<'_>, area: Rect, model: &Model) {
    let count = model.domain().agents.len();
    if count == 0 || area.height <= 1 {
        return;
    }
    let list = Rect::new(
        area.x,
        area.y.saturating_add(1),
        area.width,
        area.height.saturating_sub(1),
    );
    let capacity = usize::from(list.height / 2).max(1);
    let page_start = selected_page_start(model, capacity);
    let visible_count = count.saturating_sub(page_start).min(capacity);
    let item_height = (list.height / u16::try_from(visible_count).unwrap_or(1)).max(1);
    for (index, (key, agent)) in model
        .domain()
        .agents
        .iter()
        .skip(page_start)
        .take(capacity)
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

fn selected_page_start(model: &Model, capacity: usize) -> usize {
    let selected_index = model
        .selected_agent_key()
        .and_then(|selected| model.domain().agents.keys().position(|key| key == selected))
        .unwrap_or_default();
    selected_index / capacity * capacity
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
            actions.extend(["[enter] visit", "[r] reply", "[o] refresh"]);
            if model
                .selected_agent()
                .is_some_and(|agent| agent.attention.is_unread())
            {
                actions.push("[space] seen");
            }
            if model.reviewr_available() {
                actions.push("[v] reviewr");
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
            selected.extend(["[enter] visit", "[r] reply", "[o] refresh"]);
            if model
                .selected_agent()
                .is_some_and(|agent| agent.attention.is_unread())
            {
                selected.push("[space] seen");
            }
            if model.reviewr_available() {
                global.push("[v] reviewr");
            }
        }
    }
    frame.render_widget(
        Paragraph::new(Text::from(vec![
            Line::from(global.join(" ")),
            Line::from(selected.join(" ")),
        ]))
        .style(styles.muted),
        area,
    );
}

fn render_connection_overlay(
    frame: &mut Frame<'_>,
    area: Rect,
    connection: &ConnectionState,
    styles: DelveStyles,
) {
    let label = match connection {
        ConnectionState::Offline => Some("DISCONNECTED - LAST TALES PRESERVED".to_owned()),
        ConnectionState::Reconnecting { attempt } => {
            Some(format!("RECONNECTING #{attempt} - LAST TALES PRESERVED"))
        }
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
