use ratatui::{
    Frame,
    buffer::CellWidth,
    layout::{Alignment, Constraint, Rect},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use std::time::Duration;

use crate::{
    app::{CharacterSet, ConnectionState, GuildFocus, Model},
    domain::{Agent, AgentKey, CampaignStatus, GuildSummons, Presence},
    ui::{
        EffectCells, GuildGoblinEvidence, GuildPresentation, RenderProjection,
        copy::{self, EMPTY_GUILD, SCRYING_CLOUDED, SCRYING_STILL},
        goblins,
        guild_room_projection::{
            AdventurerRepresentation, GuildRoomMode, GuildRoomProjection,
            project as project_guild_room,
        },
        theme::{ACCENT, INK, MUTED},
        widgets::presentation::present,
    },
};

use super::great_room;

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

pub(crate) fn render(
    frame: &mut Frame<'_>,
    model: &Model,
    projection: &RenderProjection,
    layout: &GuildLayout,
) -> GuildGoblinEvidence {
    let area = frame.area();
    if area.width < 4 || area.height < 3 {
        frame.render_widget(Paragraph::new("G").style(ACCENT), area);
        return GuildGoblinEvidence::default();
    }

    let body = layout.body;
    let footer = layout.footer;
    if let Some(room) = projection.guild_room.as_ref() {
        let marginalia = great_room::render(frame, body, model, room);
        let sprites = goblins::render(frame, body, model);
        render_footer(frame, footer, &layout.footer_projection);
        return GuildGoblinEvidence {
            sprites,
            marginalia,
        };
    }

    let title = format!(
        " QUESTMANCER'S GUILD HALL - {} ",
        connection_label(model.connection())
    );
    let mut outer = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(ACCENT)
        .style(INK);
    if model.preferences().character_set == CharacterSet::Ascii {
        outer = outer.border_set(ASCII_BORDER);
    }
    let inner = outer.inner(body);
    frame.render_widget(outer, body);

    let content = render_connection_banner(frame, inner, model);
    let marginalia_visible = match projection.guild_presentation {
        GuildPresentation::Tiny | GuildPresentation::Wide => EffectCells::default(),
        GuildPresentation::Empty => {
            render_empty(frame, content);
            EffectCells::default()
        }
        GuildPresentation::Medium => {
            render_medium(frame, content, model);
            EffectCells::default()
        }
        GuildPresentation::Focused => render_focused(frame, content, model),
    };

    let sprite_visible = goblins::render(frame, content, model);
    render_footer(frame, footer, &layout.footer_projection);
    GuildGoblinEvidence {
        sprites: sprite_visible,
        marginalia: marginalia_visible,
    }
}

fn render_connection_banner(frame: &mut Frame<'_>, area: Rect, model: &Model) -> Rect {
    let Some(lines) = connection_banner_lines(model) else {
        return area;
    };
    if area.is_empty() {
        return area;
    }
    let banner_height = u16::try_from(lines.len()).unwrap_or(u16::MAX);
    let [banner, content] =
        ratatui::layout::Layout::vertical([Constraint::Length(banner_height), Constraint::Min(0)])
            .areas(area);
    frame.render_widget(Paragraph::new(Text::from(lines)).style(ACCENT), banner);
    content
}

fn connection_banner_lines(model: &Model) -> Option<Vec<Line<'static>>> {
    match model.connection() {
        ConnectionState::Connecting => Some(vec![Line::from(
            "The scrying pool is waking. Connecting to Herdr.",
        )]),
        ConnectionState::Offline => connection_notice_message(model).map(|cause| {
            vec![Line::from(format!(
                "The scrying pool is dark. Cause: {}",
                present(cause, model.preferences().character_set)
            ))]
        }),
        ConnectionState::Connected => None,
        ConnectionState::Reconnecting { attempt } => {
            let mut lines = vec![Line::from(SCRYING_CLOUDED)];
            if let Some(cause) = connection_notice_message(model) {
                lines.push(Line::from(format!(
                    "Cause: {}",
                    present(cause, model.preferences().character_set)
                )));
            }
            lines.push(Line::from(format!("Reconnect attempt {attempt}.")));
            Some(lines)
        }
        ConnectionState::Incompatible { expected, actual } => Some(vec![Line::from(format!(
            "The scrying pool rejects this omen. Cause: protocol {actual}; expected {expected}."
        ))]),
    }
}

fn render_empty(frame: &mut Frame<'_>, area: Rect) {
    let message = Text::from(vec![
        Line::from(""),
        Line::styled(EMPTY_GUILD, ACCENT),
        Line::from(""),
        Line::styled("Open a campaign when the next quest arrives.", MUTED),
    ]);
    frame.render_widget(
        Paragraph::new(message)
            .alignment(Alignment::Center)
            .style(INK)
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_medium(frame: &mut Frame<'_>, area: Rect, model: &Model) {
    let [overview, selected] =
        ratatui::layout::Layout::horizontal([Constraint::Percentage(44), Constraint::Min(38)])
            .areas(area);
    let [board, party] =
        ratatui::layout::Layout::vertical([Constraint::Percentage(45), Constraint::Min(5)])
            .areas(overview);
    render_quest_board(frame, board, model);
    render_party(frame, party, model);

    let [adventurer, scrying] = ratatui::layout::Layout::vertical([
        Constraint::Length(selected.height.min(9)),
        Constraint::Min(4),
    ])
    .areas(selected);
    render_adventurer(frame, adventurer, model, true);
    render_scrying(frame, scrying, model, true, true);
}

fn render_focused(frame: &mut Frame<'_>, area: Rect, model: &Model) -> EffectCells {
    let [primary, diagnostic] = if ordinary_notice_message(model).is_some() {
        ratatui::layout::Layout::vertical([Constraint::Min(1), Constraint::Length(2)]).areas(area)
    } else {
        [area, Rect::default()]
    };
    let marginalia_visible = match model.guild_focus() {
        GuildFocus::QuestWall | GuildFocus::Door => {
            render_quest_board(frame, primary, model);
            EffectCells::default()
        }
        GuildFocus::CampaignTables | GuildFocus::Hearth => {
            render_party(frame, primary, model);
            EffectCells::default()
        }
        GuildFocus::CounselBell => {
            render_summons(frame, primary, model);
            EffectCells::default()
        }
        GuildFocus::Chronicle => render_chronicle(frame, primary, model),
        GuildFocus::Scrying | GuildFocus::Spoils => {
            let card_height = primary.height.saturating_sub(4).min(7);
            let [adventurer, scrying] = ratatui::layout::Layout::vertical([
                Constraint::Length(card_height),
                Constraint::Min(4),
            ])
            .areas(primary);
            render_adventurer(frame, adventurer, model, false);
            render_scrying(frame, scrying, model, false, false);
            EffectCells::default()
        }
    };
    if let Some(status) = ordinary_notice_message(model) {
        frame.render_widget(
            Paragraph::new(present(status, model.preferences().character_set).into_owned())
                .style(ACCENT),
            diagnostic,
        );
    }
    marginalia_visible
}

fn render_quest_board(frame: &mut Frame<'_>, area: Rect, model: &Model) {
    let lines = if model.domain().campaigns.is_empty() {
        vec![Line::styled("No commissions posted.", MUTED)]
    } else {
        model
            .domain()
            .campaigns
            .values()
            .map(|campaign| {
                let status = campaign.status(&model.domain().agents);
                Line::from(format!(
                    "{} {}  {} adventurer{}",
                    campaign_marker(status),
                    present(&campaign.label, model.preferences().character_set),
                    campaign.party.len(),
                    if campaign.party.len() == 1 { "" } else { "s" }
                ))
            })
            .collect()
    };
    render_panel(
        frame,
        area,
        " QUEST BOARD ",
        Text::from(lines),
        model.preferences().character_set,
    );
}

fn render_party(frame: &mut Frame<'_>, area: Rect, model: &Model) {
    let lines = model
        .domain()
        .agents
        .iter()
        .map(|(key, agent)| party_line(key, agent, model, area))
        .collect::<Vec<_>>();
    render_panel_unwrapped(
        frame,
        area,
        " PARTY ROSTER ",
        Text::from(lines),
        model.preferences().character_set,
    );
}

fn party_line(key: &AgentKey, agent: &Agent, model: &Model, area: Rect) -> Line<'static> {
    let marker = if model.selected_agent_key() == Some(key) {
        ">"
    } else {
        " "
    };
    let presence = if party_elapsed_fits(agent, model, area) {
        presence_with_elapsed(agent, model)
    } else {
        presence_label(agent.presence).to_owned()
    };
    Line::from(format!(
        "{marker} {}  {}",
        presence,
        present(&agent.persona.name, model.preferences().character_set)
    ))
}

fn render_summons(frame: &mut Frame<'_>, area: Rect, model: &Model) {
    let mut lines = model
        .domain()
        .agents
        .values()
        .filter_map(|agent| summons_line(agent, model.preferences().character_set))
        .collect::<Vec<_>>();
    if lines.is_empty() {
        lines.push(Line::styled("No calls await the Questmancer.", MUTED));
    }
    render_panel(
        frame,
        area,
        " CALLS FOR COUNSEL ",
        Text::from(lines),
        model.preferences().character_set,
    );
}

fn summons_line(agent: &Agent, character_set: CharacterSet) -> Option<Line<'static>> {
    let summons = agent.attention.summons().or_else(|| {
        (agent.presence == Presence::Blocked).then_some(GuildSummons::CounselRequested)
    })?;
    let message = match summons {
        GuildSummons::CounselRequested => copy::counsel_requested(&agent.persona.name),
        GuildSummons::SpoilsReturned => copy::spoils_returned(&agent.persona.name),
        GuildSummons::AdventurerDeparted => {
            format!("{} has departed the guild.", agent.persona.name)
        }
    };
    Some(Line::styled(
        present(&message, character_set).into_owned(),
        ACCENT,
    ))
}

fn render_chronicle(frame: &mut Frame<'_>, area: Rect, model: &Model) -> EffectCells {
    let mut lines = model
        .domain()
        .chronicle
        .entries()
        .iter()
        .rev()
        .take(5)
        .map(|entry| {
            Line::from(present(&entry.summary, model.preferences().character_set).into_owned())
        })
        .collect::<Vec<_>>();
    if lines.is_empty() {
        lines.push(Line::styled("No deeds recorded yet.", MUTED));
    }
    render_panel(
        frame,
        area,
        " CHRONICLE ",
        Text::from(lines.clone()),
        model.preferences().character_set,
    );
    if !model.goblins().is_visible(model.now()) {
        return EffectCells::default();
    }
    let baseline = frame.buffer_mut().clone();
    render_panel(
        frame,
        area,
        " CHRONICLE - CREATURES DETECTED ",
        Text::from(lines),
        model.preferences().character_set,
    );
    EffectCells::changed_between(&baseline, frame.buffer_mut(), area)
}

fn render_adventurer(frame: &mut Frame<'_>, area: Rect, model: &Model, include_summons: bool) {
    let Some(agent) = model.selected_agent() else {
        render_panel(
            frame,
            area,
            " ADVENTURER ",
            Text::from("No adventurer selected."),
            model.preferences().character_set,
        );
        return;
    };
    let lines = adventurer_lines(
        agent,
        model,
        include_summons,
        adventurer_elapsed_fits(agent, model, area),
    );
    render_panel_unwrapped(
        frame,
        area,
        " ADVENTURER ",
        Text::from(lines),
        model.preferences().character_set,
    );
}

fn adventurer_lines(
    agent: &Agent,
    model: &Model,
    include_summons: bool,
    show_elapsed: bool,
) -> Vec<Line<'static>> {
    let campaign = model
        .domain()
        .campaigns
        .get(&agent.workspace_id)
        .map_or("Unknown campaign", |campaign| campaign.label.as_str());
    let mut lines = vec![
        Line::styled(
            present(&agent.persona.name, model.preferences().character_set).into_owned(),
            ACCENT,
        ),
        Line::from(format!(
            "{:?} {:?} - {}",
            agent.persona.ancestry,
            agent.persona.class,
            present(
                agent.persona.epithet.as_str(),
                model.preferences().character_set
            )
        )),
        Line::from(format!(
            "Campaign: {}",
            present(campaign, model.preferences().character_set)
        )),
        Line::from(format!(
            "{} ({})",
            if show_elapsed {
                presence_with_elapsed(agent, model)
            } else {
                presence_label(agent.presence).to_owned()
            },
            present(&agent.name, model.preferences().character_set)
        )),
    ];
    if include_summons && let Some(line) = summons_line(agent, model.preferences().character_set) {
        lines.push(line);
    }
    if let Some(status) = &agent.custom_status {
        lines.push(Line::styled(
            present(status, model.preferences().character_set).into_owned(),
            MUTED,
        ));
    }
    lines
}

fn render_scrying(
    frame: &mut Frame<'_>,
    area: Rect,
    model: &Model,
    include_status: bool,
    include_integration_notice: bool,
) {
    let Some(agent) = model.selected_agent() else {
        render_panel(
            frame,
            area,
            " SCRYING TABLE ",
            Text::from(SCRYING_STILL),
            model.preferences().character_set,
        );
        return;
    };
    let managed = model.managed_pane_id() == Some(&agent.pane_id);
    let mut lines = Vec::new();
    if managed {
        lines.push(Line::styled(SCRYING_STILL, MUTED));
        lines.push(Line::styled(
            "Cause: the Questmancer's own pane is never observed.",
            MUTED,
        ));
    } else if let Some(preview) = model
        .output_preview()
        .filter(|preview| preview.pane_id == agent.pane_id)
    {
        if let Some(error) = &preview.error {
            lines.push(Line::styled("The scrying pool has clouded.", ACCENT));
            lines.push(Line::from(format!(
                "Cause: {}",
                present(error, model.preferences().character_set)
            )));
        } else if preview.loading {
            lines.push(Line::styled(SCRYING_STILL, MUTED));
            lines.push(Line::styled("Tracing the selected adventurer...", MUTED));
        } else {
            lines.extend(preview.text.lines().map(|line| {
                Line::from(present(line, model.preferences().character_set).into_owned())
            }));
        }
    } else {
        lines.push(Line::styled(SCRYING_STILL, MUTED));
        lines.push(Line::styled("Select refresh to trace recent deeds.", MUTED));
    }
    if include_status
        && let Some(status) = scrying_notice_message(model, include_integration_notice)
    {
        lines.push(Line::from(""));
        lines.push(Line::styled(
            present(status, model.preferences().character_set).into_owned(),
            MUTED,
        ));
    }
    render_panel(
        frame,
        area,
        " SCRYING TABLE ",
        Text::from(lines),
        model.preferences().character_set,
    );
}

#[derive(Debug)]
struct FooterProjection {
    notice_lines: Vec<String>,
    ledger_lines: Vec<String>,
    actions: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct GuildLayout {
    body: Rect,
    footer: Rect,
    footer_projection: FooterProjection,
}

impl GuildLayout {
    pub(crate) const fn room_area(&self) -> Rect {
        self.body
    }
}

pub(crate) fn layout(model: &Model, area: Rect) -> GuildLayout {
    let footer_projection = footer_projection(model, area);
    if area.width < 4 || area.height < 3 {
        return GuildLayout {
            body: area,
            footer: Rect::default(),
            footer_projection,
        };
    }
    let [body, footer] = ratatui::layout::Layout::vertical([
        Constraint::Min(1),
        Constraint::Length(footer_projection.height()),
    ])
    .areas(area);
    GuildLayout {
        body,
        footer,
        footer_projection,
    }
}

impl FooterProjection {
    fn height(&self) -> u16 {
        u16::try_from(
            self.notice_lines
                .len()
                .saturating_add(self.ledger_lines.len())
                .saturating_add(self.actions.len()),
        )
        .unwrap_or(u16::MAX)
    }
}

fn footer_projection(model: &Model, area: Rect) -> FooterProjection {
    let width = area.width;
    let mut actions = vec!["[1] Guild Hall", "[2] Delve"];
    if width < 80 {
        actions.push("[tab] Next landmark");
    }
    if !model.domain().agents.is_empty() {
        actions.extend(["[j/k] Choose", "[/] Search"]);
        if let Some(agent) = model.selected_agent() {
            let managed = model.managed_pane_id() == Some(&agent.pane_id);
            if !managed {
                actions.extend(["[enter] Observe", "[r] Issue counsel", "[o] Scry again"]);
            }
            if agent.attention.is_unread() {
                actions.push("[space] Acknowledge summons");
            }
            if !managed && model.reviewr_available() {
                actions.push("[v] Inspect spoils");
            }
        }
    }
    let actions = pack_actions(&actions, usize::from(width));
    let mut notice_lines = [model.action_feedback(), model.persistence_diagnostic()]
        .into_iter()
        .flatten()
        .flat_map(|message| {
            let message = present(message, model.preferences().character_set);
            wrap_footer_notice(&message, width)
        })
        .collect::<Vec<_>>();
    let provisional_footer_height = notice_lines.len().saturating_add(actions.len());
    let provisional_body = body_for_footer(
        area,
        u16::try_from(provisional_footer_height).unwrap_or(u16::MAX),
    );
    let provisional_room = project_guild_room(model, provisional_body);
    if great_room::integration_continuation_required(model, &provisional_room)
        && let Some(message) = model.integration_diagnostic()
    {
        let message = present(message, model.preferences().character_set);
        notice_lines.extend(wrap_footer_notice(&message, width));
    }
    let mut projection = FooterProjection {
        notice_lines,
        ledger_lines: Vec::new(),
        actions,
    };
    projection.ledger_lines = stable_token_ledger_lines(model, area, projection.height());
    projection
}

fn body_for_footer(area: Rect, footer_height: u16) -> Rect {
    let [body, _footer] =
        ratatui::layout::Layout::vertical([Constraint::Min(1), Constraint::Length(footer_height)])
            .areas(area);
    body
}

fn stable_token_ledger_lines(model: &Model, area: Rect, base_footer_height: u16) -> Vec<String> {
    let campaign_count = model.domain().campaigns.len();
    let mut lower = 0;
    let mut upper = campaign_count;
    while lower < upper {
        let midpoint = lower + (upper - lower) / 2;
        let footer_height =
            base_footer_height.saturating_add(u16::try_from(midpoint).unwrap_or(u16::MAX));
        let count = token_ledger_entries(model, body_for_footer(area, footer_height)).len();
        if count <= midpoint {
            upper = midpoint;
        } else {
            lower = midpoint.saturating_add(1);
        }
    }
    let footer_height = base_footer_height.saturating_add(u16::try_from(lower).unwrap_or(u16::MAX));
    token_ledger_entries(model, body_for_footer(area, footer_height))
        .into_iter()
        .map(|entry| {
            format!(
                "TABLE #{:02}: {} TOKENS",
                entry.table_ordinal, entry.token_count
            )
        })
        .collect()
}

fn token_ledger_entries(model: &Model, body: Rect) -> Vec<great_room::TokenLedgerEntry> {
    if body.width < 120 {
        return Vec::new();
    }
    let room = project_guild_room(model, body);
    if room.mode != GuildRoomMode::WholeRoom {
        return Vec::new();
    }
    great_room::unrepresentable_token_ledger_entries(&room)
}

fn wrap_footer_notice(message: &str, width: u16) -> Vec<String> {
    if width == 0 {
        return Vec::new();
    }
    let mut rows = Vec::new();
    let mut row = String::new();
    let mut row_width = 0_u16;
    for word in message.split_whitespace() {
        let mut chunks = Vec::new();
        let mut chunk = String::new();
        let mut chunk_width = 0_u16;
        for character in word.chars() {
            let mut encoded = [0; 4];
            let character_width = character.encode_utf8(&mut encoded).cell_width();
            if !chunk.is_empty() && chunk_width.saturating_add(character_width) > width {
                chunks.push(std::mem::take(&mut chunk));
                chunk_width = 0;
            }
            chunk.push(character);
            chunk_width = chunk_width.saturating_add(character_width);
        }
        if !chunk.is_empty() {
            chunks.push(chunk);
        }
        for chunk in chunks {
            let chunk_width = chunk.cell_width();
            let additional = chunk_width.saturating_add(u16::from(!row.is_empty()));
            if !row.is_empty() && row_width.saturating_add(additional) > width {
                rows.push(std::mem::take(&mut row));
                row_width = 0;
            }
            if !row.is_empty() {
                row.push(' ');
                row_width = row_width.saturating_add(1);
            }
            row.push_str(&chunk);
            row_width = row_width.saturating_add(chunk_width);
        }
    }
    if !row.is_empty() {
        rows.push(row);
    }
    rows
}

fn pack_actions(actions: &[&str], width: usize) -> Vec<String> {
    let mut rows = Vec::new();
    let mut row = String::new();
    for action in actions {
        let additional = action.len() + usize::from(!row.is_empty()) * 2;
        if !row.is_empty() && row.len().saturating_add(additional) > width {
            rows.push(std::mem::take(&mut row));
        }
        if !row.is_empty() {
            row.push_str("  ");
        }
        row.push_str(action);
    }
    if !row.is_empty() {
        rows.push(row);
    }
    rows
}

fn render_footer(frame: &mut Frame<'_>, area: Rect, projection: &FooterProjection) {
    if area.is_empty() {
        return;
    }
    frame.render_widget(Clear, area);
    let [notice_area, actions_area] = ratatui::layout::Layout::vertical([
        Constraint::Length(
            u16::try_from(
                projection
                    .notice_lines
                    .len()
                    .saturating_add(projection.ledger_lines.len()),
            )
            .unwrap_or(u16::MAX),
        ),
        Constraint::Min(0),
    ])
    .areas(area);
    frame.render_widget(
        Paragraph::new(Text::from(
            projection
                .notice_lines
                .iter()
                .chain(&projection.ledger_lines)
                .map(|line| Line::from(line.as_str()))
                .collect::<Vec<_>>(),
        ))
        .style(MUTED),
        notice_area,
    );
    frame.render_widget(
        Paragraph::new(Text::from(
            projection
                .actions
                .iter()
                .map(|line| Line::from(line.as_str()))
                .collect::<Vec<_>>(),
        ))
        .style(MUTED),
        actions_area,
    );
}

fn render_panel(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    text: Text<'_>,
    character_set: CharacterSet,
) {
    render_panel_with_wrapping(frame, area, title, text, character_set, true);
}

fn render_panel_unwrapped(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    text: Text<'_>,
    character_set: CharacterSet,
) {
    render_panel_with_wrapping(frame, area, title, text, character_set, false);
}

fn render_panel_with_wrapping(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    text: Text<'_>,
    character_set: CharacterSet,
    wrap: bool,
) {
    if area.is_empty() {
        return;
    }
    let mut block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(MUTED);
    if character_set == CharacterSet::Ascii {
        block = block.border_set(ASCII_BORDER);
    }
    let mut paragraph = Paragraph::new(text).block(block).style(INK);
    if wrap {
        paragraph = paragraph.wrap(Wrap { trim: false });
    }
    frame.render_widget(paragraph, area);
}

fn presence_with_elapsed(agent: &Agent, model: &Model) -> String {
    let presence = presence_label(agent.presence);
    if !model.settings().show_elapsed_time {
        return presence.to_owned();
    }
    format!("{presence} {}", elapsed_label_at(agent, model.now()))
}

fn elapsed_label_at(agent: &Agent, now: crate::domain::Timestamp) -> String {
    let elapsed = agent.presence_since.elapsed_until(now).as_secs();
    if elapsed >= 60 {
        format!("{}m", elapsed / 60)
    } else {
        format!("{elapsed}s")
    }
}

pub(crate) fn next_elapsed_label_in(
    model: &Model,
    projection: Option<&GuildRoomProjection>,
) -> Option<Duration> {
    if !model.settings().show_elapsed_time || model.domain().agents.is_empty() {
        return None;
    }
    let projection = projection?;
    projection
        .adventurers
        .iter()
        .filter_map(|representation| {
            let agent = model
                .domain()
                .agents
                .get(representation_agent(representation))?;
            next_projected_elapsed_change(model, projection, representation, agent)
        })
        .min()
}

fn connection_notice_message(model: &Model) -> Option<&str> {
    model.connection_diagnostic()
}

fn ordinary_notice_message(model: &Model) -> Option<&str> {
    model
        .action_feedback()
        .or_else(|| model.persistence_diagnostic())
        .or_else(|| model.reviewr_availability_diagnostic())
        .or_else(|| model.integration_diagnostic())
}

fn scrying_notice_message(model: &Model, include_integration_notice: bool) -> Option<&str> {
    model
        .action_feedback()
        .or_else(|| model.persistence_diagnostic())
        .or_else(|| {
            include_integration_notice
                .then(|| {
                    model
                        .reviewr_availability_diagnostic()
                        .or_else(|| model.integration_diagnostic())
                })
                .flatten()
        })
}

fn party_elapsed_fits(agent: &Agent, model: &Model, area: Rect) -> bool {
    party_elapsed_fits_at(agent, model, area, model.now())
}

fn party_elapsed_fits_at(
    agent: &Agent,
    model: &Model,
    area: Rect,
    now: crate::domain::Timestamp,
) -> bool {
    if !model.settings().show_elapsed_time {
        return false;
    }
    let prefix = Line::from(format!(
        "> {} {}",
        presence_label(agent.presence),
        elapsed_label_at(agent, now)
    ));
    prefix.width() <= usize::from(area.width.saturating_sub(2))
}

fn adventurer_elapsed_fits(agent: &Agent, model: &Model, area: Rect) -> bool {
    adventurer_elapsed_fits_at(agent, model, area, model.now())
}

fn adventurer_elapsed_fits_at(
    agent: &Agent,
    model: &Model,
    area: Rect,
    now: crate::domain::Timestamp,
) -> bool {
    model.settings().show_elapsed_time
        && Line::from(format!(
            "{} {}",
            presence_label(agent.presence),
            elapsed_label_at(agent, now)
        ))
        .width()
            <= usize::from(area.width.saturating_sub(2))
}

fn next_projected_elapsed_change(
    model: &Model,
    projection: &GuildRoomProjection,
    representation: &AdventurerRepresentation,
    agent: &Agent,
) -> Option<Duration> {
    let now = model.now();
    let immediate_delay = next_elapsed_boundary(agent, now);
    let immediate = timestamp_after(now, immediate_delay);
    if great_room::elapsed_label_is_visible(model, projection, representation, now)
        || great_room::elapsed_label_is_visible(model, projection, representation, immediate)
    {
        return Some(immediate_delay);
    }

    let elapsed = agent.presence_since.elapsed_until(now);
    if now >= agent.presence_since && elapsed.as_secs() >= 60 {
        return None;
    }
    let minute_boundary = timestamp_after(agent.presence_since, Duration::from_secs(60));
    great_room::elapsed_label_is_visible(model, projection, representation, minute_boundary)
        .then(|| now.elapsed_until(minute_boundary))
}

fn representation_agent(representation: &AdventurerRepresentation) -> &AgentKey {
    match representation {
        AdventurerRepresentation::Physical { agent, .. }
        | AdventurerRepresentation::Token { agent, .. }
        | AdventurerRepresentation::Projection { agent, .. } => agent,
    }
}

fn timestamp_after(
    timestamp: crate::domain::Timestamp,
    duration: Duration,
) -> crate::domain::Timestamp {
    let milliseconds = i64::try_from(duration.as_millis()).unwrap_or(i64::MAX);
    crate::domain::Timestamp::from_millis(timestamp.as_millis().saturating_add(milliseconds))
}

fn next_elapsed_boundary(agent: &Agent, now: crate::domain::Timestamp) -> Duration {
    if now < agent.presence_since {
        return now
            .elapsed_until(agent.presence_since)
            .saturating_add(Duration::from_secs(1));
    }

    let elapsed = agent.presence_since.elapsed_until(now);
    let unit_millis = if elapsed.as_secs() < 60 {
        1_000_u128
    } else {
        60_000_u128
    };
    let elapsed_millis = elapsed.as_millis();
    let next_boundary = (elapsed_millis / unit_millis + 1).saturating_mul(unit_millis);
    let delay = next_boundary.saturating_sub(elapsed_millis).max(1);
    Duration::from_millis(u64::try_from(delay).unwrap_or(u64::MAX))
}

const fn presence_label(presence: Presence) -> &'static str {
    match presence {
        Presence::Working => "working",
        Presence::Blocked => "blocked",
        Presence::Done => "completed",
        Presence::Idle => "resting",
        Presence::Exited => "departed",
        Presence::Unknown => "unknown",
    }
}

const fn campaign_marker(status: CampaignStatus) -> &'static str {
    match status {
        CampaignStatus::CounselRequired => "!",
        CampaignStatus::SpoilsAwaitingInspection => "+",
        CampaignStatus::ExpeditionActive => "~",
        CampaignStatus::PartyAtRest => "*",
        CampaignStatus::Abandoned => "x",
    }
}

const fn connection_label(state: &ConnectionState) -> &'static str {
    match state {
        ConnectionState::Offline => "OFFLINE",
        ConnectionState::Connecting => "CONNECTING",
        ConnectionState::Connected => "CONNECTED",
        ConnectionState::Reconnecting { .. } => "RECONNECTING",
        ConnectionState::Incompatible { .. } => "INCOMPATIBLE",
    }
}
