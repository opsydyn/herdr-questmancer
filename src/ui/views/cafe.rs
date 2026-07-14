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
        widgets::{render_profile_card, render_workstation},
    },
};

const PROFILE_WIDTH: u16 = 43;
const MAX_WORKSTATION_HEIGHT: u16 = 12;
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
struct CafeStyles {
    ink: Style,
    accent: Style,
    muted: Style,
    wall: Style,
    floor: Style,
}

impl CafeStyles {
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
    let styles = CafeStyles::from_preferences(*model.preferences());
    if area.width < 4 || area.height < 3 {
        frame.render_widget(Paragraph::new("C").style(styles.accent), area);
        return;
    }

    let footer_height = if area.width < 80 { 2 } else { 1 };
    let [body, footer] =
        ratatui::layout::Layout::vertical([Constraint::Min(1), Constraint::Length(footer_height)])
            .areas(area);
    let title = format!(
        " THE HERDR CYBERCAFE - {} ",
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
        paint_room(frame, inner, styles);
        if inner.width >= 118 {
            let grid_width = inner.width.saturating_sub(PROFILE_WIDTH);
            render_grid(
                frame,
                Rect::new(inner.x, inner.y, grid_width, inner.height),
                model,
            );
            if let Some(agent) = model.selected_agent() {
                let profile_area = Rect::new(
                    inner.x.saturating_add(grid_width),
                    inner.y.saturating_add(1),
                    PROFILE_WIDTH,
                    inner.height.saturating_sub(1).min(21),
                );
                render_profile_card(
                    frame,
                    profile_area,
                    agent,
                    frame_for(agent, model.now(), model.preferences()),
                    model.preferences(),
                );
            }
        } else if inner.width >= 78 {
            render_grid(frame, inner, model);
        } else {
            render_compact_list(frame, inner, model);
        }
        render_connection_overlay(frame, inner, model.connection(), styles);
    }

    render_footer(frame, footer, model, styles);
}

fn paint_room(frame: &mut Frame<'_>, area: Rect, styles: CafeStyles) {
    if area.is_empty() {
        return;
    }
    let mut lines = vec![Line::styled(
        " CAFE WALL / 56K CABLE RUN / COUNTER ",
        styles.wall,
    )];
    lines.resize(usize::from(area.height), Line::from(""));
    if area.height > 1 {
        let floor = usize::from(area.height - 1);
        lines[floor] = Line::styled("-- CABLE RUN ---------------- [COUNTER] --", styles.floor);
    }
    frame.render_widget(Paragraph::new(Text::from(lines)).style(styles.ink), area);
}

fn render_grid(frame: &mut Frame<'_>, area: Rect, model: &Model) {
    let count = model.domain().agents.len();
    if count == 0 || area.width < 28 || area.height <= 1 {
        return;
    }
    let columns = usize::from(area.width / 28).max(1).min(count);
    let rows = count.div_ceil(columns);
    let grid = Rect::new(
        area.x,
        area.y.saturating_add(1),
        area.width,
        area.height.saturating_sub(1),
    );
    let row_height =
        (grid.height / u16::try_from(rows).unwrap_or(1)).clamp(1, MAX_WORKSTATION_HEIGHT);

    for (index, (key, agent)) in model.domain().agents.iter().enumerate() {
        let column = index % columns;
        let row = index / columns;
        let x0 = u32::from(grid.x)
            + u32::try_from(column).unwrap_or_default() * u32::from(grid.width)
                / u32::try_from(columns).unwrap_or(1);
        let x1 = u32::from(grid.x)
            + u32::try_from(column + 1).unwrap_or_default() * u32::from(grid.width)
                / u32::try_from(columns).unwrap_or(1);
        let y = grid.y.saturating_add(
            u16::try_from(row)
                .unwrap_or_default()
                .saturating_mul(row_height),
        );
        if y >= grid.bottom() {
            break;
        }
        let cell = Rect::new(
            u16::try_from(x0).unwrap_or(u16::MAX),
            y,
            u16::try_from(x1.saturating_sub(x0)).unwrap_or_default(),
            row_height.min(grid.bottom().saturating_sub(y)),
        );
        render_workstation(
            frame,
            cell,
            agent,
            frame_for(agent, model.now(), model.preferences()),
            model.selected_agent_key() == Some(key),
            model.preferences(),
        );
    }
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
    let item_height = (list.height / u16::try_from(count).unwrap_or(1)).max(1);
    for (index, (key, agent)) in model.domain().agents.iter().enumerate() {
        let y = list.y.saturating_add(
            u16::try_from(index)
                .unwrap_or_default()
                .saturating_mul(item_height),
        );
        if y >= list.bottom() {
            break;
        }
        render_workstation(
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

fn render_empty(frame: &mut Frame<'_>, area: Rect, styles: CafeStyles) {
    let message = Text::from(vec![
        Line::from(""),
        Line::styled("All workstations are free", styles.accent),
        Line::from(""),
        Line::styled("Start an agent to put a workstation online", styles.muted),
    ]);
    frame.render_widget(
        Paragraph::new(message)
            .alignment(Alignment::Center)
            .style(styles.ink),
        area,
    );
}

fn render_footer(frame: &mut Frame<'_>, area: Rect, model: &Model, styles: CafeStyles) {
    if area.height > 1 {
        render_narrow_footer(frame, area, model, styles);
        return;
    }
    let mut actions = vec!["[1] desk", "[2] cafe"];
    if !model.domain().agents.is_empty() {
        actions.push("[j/k] navigate");
        if model.selected_agent().is_some() {
            actions.extend(["[enter] visit", "[r] reply", "[o] refresh"]);
            if model
                .selected_agent()
                .is_some_and(|agent| agent.attention.is_unseen())
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

fn render_narrow_footer(frame: &mut Frame<'_>, area: Rect, model: &Model, styles: CafeStyles) {
    let mut global = vec!["[1] desk", "[2] cafe"];
    let mut selected = Vec::new();
    if !model.domain().agents.is_empty() {
        global.extend(["[j/k] navigate", "[/] search"]);
        if model.selected_agent().is_some() {
            selected.extend(["[enter] visit", "[r] reply", "[o] refresh"]);
            if model
                .selected_agent()
                .is_some_and(|agent| agent.attention.is_unseen())
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
    styles: CafeStyles,
) {
    let label = match connection {
        ConnectionState::Offline => Some("DISCONNECTED - LAST POSES PRESERVED".to_owned()),
        ConnectionState::Reconnecting { attempt } => {
            Some(format!("RECONNECTING #{attempt} - LAST POSES PRESERVED"))
        }
        ConnectionState::Incompatible { expected, actual } => Some(format!(
            "INCOMPATIBLE PROTOCOL {actual} - NEED {expected} - LAST POSES PRESERVED"
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
        ConnectionState::Connecting => "connecting at 56k".to_owned(),
        ConnectionState::Connected => "connected at 56k".to_owned(),
        ConnectionState::Reconnecting { attempt } => format!("reconnecting #{attempt}"),
        ConnectionState::Incompatible { expected, actual } => {
            format!("protocol {actual} unsupported - need {expected}")
        }
    }
}
