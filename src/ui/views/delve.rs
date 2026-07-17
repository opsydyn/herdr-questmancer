use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::Style,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    app::{CharacterSet, ConnectionState, DisplayPreferences, Model},
    ui::{
        delve_projection::{
            CampaignStripProjection, DelveContentProjection, DelveRenderProjection,
            ProjectedChamber, ProjectedDelve,
        },
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

pub(crate) fn render(frame: &mut Frame<'_>, model: &Model, projection: &DelveRenderProjection) {
    let styles = DelveStyles::from_preferences(*model.preferences());
    if let DelveContentProjection::Tiny { area } = projection.content {
        frame.render_widget(Paragraph::new("D").style(styles.accent), area);
        return;
    }
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
    frame.render_widget(block, projection.body_area);

    match &projection.content {
        DelveContentProjection::Tiny { .. } => unreachable!("tiny projection returned above"),
        DelveContentProjection::Empty { area } => render_empty(frame, *area, styles),
        DelveContentProjection::Compact {
            chambers,
            connection_overlay,
            ..
        } => {
            render_compact_list(frame, model, chambers);
            render_connection_overlay(frame, *connection_overlay, model.connection(), styles);
        }
        DelveContentProjection::Connected {
            delves,
            campaign_strip,
            connection_overlay,
            ..
        } => {
            render_connected_delves(frame, model, styles, delves, campaign_strip.as_ref());
            if let Some(area) = connection_overlay {
                render_connection_overlay(frame, *area, model.connection(), styles);
            }
        }
    }

    render_footer(
        frame,
        projection.footer_area,
        &projection.footer_lines,
        styles,
    );
}

fn render_connected_delves(
    frame: &mut Frame<'_>,
    model: &Model,
    styles: DelveStyles,
    delves: &[ProjectedDelve],
    campaign_strip: Option<&CampaignStripProjection>,
) {
    for delve in delves {
        render_delve_architecture(
            frame,
            delve.area,
            &delve.workspace_id,
            delve.variant,
            delve.active,
            styles,
        );
    }
    render_reconnecting_fog(frame, delves, model.connection(), styles);
    if campaign_strip.is_some() {
        if let Some(active) = delves.first() {
            render_route_home(
                frame,
                active.area,
                &active.chambers,
                model.preferences().character_set,
                styles,
            );
        }
    } else {
        render_connected_routes(frame, delves, model.preferences().character_set, styles);
    }
    for delve in delves {
        render_projected_chambers(frame, model, &delve.chambers);
    }
    if let Some(strip) = campaign_strip {
        frame.render_widget(
            Paragraph::new(strip.labels.join(" ")).style(styles.muted),
            strip.area,
        );
    }
}

fn render_reconnecting_fog(
    frame: &mut Frame<'_>,
    delves: &[ProjectedDelve],
    connection: &ConnectionState,
    styles: DelveStyles,
) {
    for delve in delves {
        render_reconnecting_fog_in_room(frame, delve.area, &delve.chambers, connection, styles);
    }
}

fn render_reconnecting_fog_in_room(
    frame: &mut Frame<'_>,
    area: Rect,
    chambers: &[ProjectedChamber],
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
    delves: &[ProjectedDelve],
    character_set: CharacterSet,
    styles: DelveStyles,
) {
    let Some(first) = delves.first() else {
        return;
    };
    render_route_home(frame, first.area, &first.chambers, character_set, styles);
    for pair in delves.windows(2) {
        let [previous, next] = pair else {
            continue;
        };
        if previous.area.y == next.area.y {
            render_adjacent_opening(frame, previous, next, character_set, styles);
        } else {
            render_row_wrap_corridor(frame, previous, next, character_set, styles);
        }
    }
}

fn render_route_home(
    frame: &mut Frame<'_>,
    area: Rect,
    chambers: &[ProjectedChamber],
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
        CharacterSet::Ascii => "<--- HOME PATH",
        CharacterSet::Unicode => "◄─── HOME PATH",
    };
    frame.render_widget(
        Paragraph::new(architecture_row(area, path)).style(styles.accent),
        Rect::new(area.x, y, area.width, 1),
    );
}

fn render_adjacent_opening(
    frame: &mut Frame<'_>,
    left: &ProjectedDelve,
    right: &ProjectedDelve,
    character_set: CharacterSet,
    styles: DelveStyles,
) {
    if left.area.right() != right.area.x || left.area.width < 2 || right.area.width < 2 {
        return;
    }
    let start_y = left.area.y.max(right.area.y).saturating_add(1);
    let end_y = left
        .area
        .bottom()
        .min(right.area.bottom())
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
    let x = left.area.right().saturating_sub(2);
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
    previous: &ProjectedDelve,
    next: &ProjectedDelve,
    character_set: CharacterSet,
    styles: DelveStyles,
) {
    if previous.area.bottom() != next.area.y || previous.area.height < 3 || next.area.height < 3 {
        return;
    }
    let seam_y = next.area.y;
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

fn unused_row(area: Rect, chambers: &[ProjectedChamber], preferred: u16) -> Option<u16> {
    let start = area.y.saturating_add(1);
    let end = area.bottom().saturating_sub(1);
    (start..end)
        .filter(|candidate| row_is_unused(*candidate, chambers))
        .min_by_key(|candidate| candidate.abs_diff(preferred))
}

fn unused_column(delve: &ProjectedDelve, rows: &[u16]) -> Option<u16> {
    let start = delve.area.x.saturating_add(2);
    let end = delve.area.right().saturating_sub(2);
    let preferred = delve.area.x.saturating_add(delve.area.width / 2);
    (start..end)
        .filter(|candidate| {
            delve.chambers.iter().all(|chamber| {
                rows.iter().all(|row| {
                    *candidate < chamber.area.x
                        || *candidate >= chamber.area.right()
                        || *row < chamber.area.y
                        || *row >= chamber.area.bottom()
                })
            })
        })
        .min_by_key(|candidate| candidate.abs_diff(preferred))
}

fn row_is_unused(y: u16, chambers: &[ProjectedChamber]) -> bool {
    chambers
        .iter()
        .all(|chamber| y < chamber.area.y || y >= chamber.area.bottom())
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

fn render_compact_list(frame: &mut Frame<'_>, model: &Model, chambers: &[ProjectedChamber]) {
    render_projected_chambers(frame, model, chambers);
}

fn render_projected_chambers(frame: &mut Frame<'_>, model: &Model, chambers: &[ProjectedChamber]) {
    for chamber in chambers {
        let Some(agent) = model.domain().agents.get(&chamber.key) else {
            continue;
        };
        render_chamber(
            frame,
            chamber.area,
            agent,
            frame_for(agent, model.now(), model.preferences()),
            model.selected_agent_key() == Some(&chamber.key),
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

fn render_footer(frame: &mut Frame<'_>, area: Rect, lines: &[String], styles: DelveStyles) {
    frame.render_widget(
        Paragraph::new(Text::from(
            lines.iter().cloned().map(Line::from).collect::<Vec<_>>(),
        ))
        .alignment(if lines.len() == 1 && area.width >= 100 {
            Alignment::Center
        } else {
            Alignment::Left
        })
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
