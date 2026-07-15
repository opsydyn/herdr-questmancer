use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Rect},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::{
    app::{ConnectionState, Model, Region},
    domain::{Agent, CampaignStatus, GuildSummons, Presence},
    ui::{
        copy::{self, EMPTY_GUILD, SCRYING_CLOUDED, SCRYING_STILL},
        theme::{ACCENT, INK, MUTED},
    },
};

pub(crate) fn render(frame: &mut Frame<'_>, model: &Model) {
    let area = frame.area();
    if area.width < 4 || area.height < 3 {
        frame.render_widget(Paragraph::new("G").style(ACCENT), area);
        return;
    }

    let footer_height = if area.width <= 80 { 2 } else { 1 };
    let [body, footer] =
        ratatui::layout::Layout::vertical([Constraint::Min(1), Constraint::Length(footer_height)])
            .areas(area);
    let title = format!(
        " QUESTMANCER'S GUILD HALL - {} ",
        connection_label(model.connection())
    );
    let outer = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(ACCENT)
        .style(INK);
    let inner = outer.inner(body);
    frame.render_widget(outer, body);

    let content = render_connection_banner(frame, inner, model.connection());
    if model.domain().agents.is_empty() {
        render_empty(frame, content);
    } else if area.width >= 120 {
        render_wide(frame, content, model);
    } else if area.width >= 80 {
        render_medium(frame, content, model);
    } else {
        render_focused(frame, content, model);
    }

    render_footer(frame, footer, model);
}

fn render_connection_banner(
    frame: &mut Frame<'_>,
    area: Rect,
    connection: &ConnectionState,
) -> Rect {
    let message = match connection {
        ConnectionState::Connecting => {
            Some("The scrying pool is waking. Connecting to Herdr.".to_owned())
        }
        ConnectionState::Offline | ConnectionState::Connected => None,
        ConnectionState::Reconnecting { attempt } => Some(format!(
            "{SCRYING_CLOUDED} Cause: reconnect attempt {attempt}."
        )),
        ConnectionState::Incompatible { expected, actual } => Some(format!(
            "The scrying pool rejects this omen. Cause: protocol {actual}; expected {expected}."
        )),
    };
    let Some(message) = message else {
        return area;
    };
    if area.is_empty() {
        return area;
    }
    let [banner, content] =
        ratatui::layout::Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(area);
    frame.render_widget(Paragraph::new(message).style(ACCENT), banner);
    content
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

fn render_wide(frame: &mut Frame<'_>, area: Rect, model: &Model) {
    let [board, guild, selected] = ratatui::layout::Layout::horizontal([
        Constraint::Percentage(25),
        Constraint::Percentage(36),
        Constraint::Min(42),
    ])
    .areas(area);
    render_quest_board(frame, board, model);

    let [party, summons, chronicle] = ratatui::layout::Layout::vertical([
        Constraint::Percentage(34),
        Constraint::Percentage(30),
        Constraint::Min(5),
    ])
    .areas(guild);
    render_party(frame, party, model);
    render_summons(frame, summons, model);
    render_chronicle(frame, chronicle, model);

    let [adventurer, scrying, spoils] = ratatui::layout::Layout::vertical([
        Constraint::Length(selected.height.min(9)),
        Constraint::Min(5),
        Constraint::Length(selected.height.min(5)),
    ])
    .areas(selected);
    render_adventurer(frame, adventurer, model, true);
    render_scrying(frame, scrying, model);
    render_spoils(frame, spoils, model);
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
    render_scrying(frame, scrying, model);
}

fn render_focused(frame: &mut Frame<'_>, area: Rect, model: &Model) {
    match model.region() {
        Region::QuestBoard => render_quest_board(frame, area, model),
        Region::Party => render_party(frame, area, model),
        Region::Summons => render_summons(frame, area, model),
        Region::Chronicle => render_chronicle(frame, area, model),
        Region::Adventurer => {
            let card_height = area.height.saturating_sub(4).min(7);
            let [adventurer, scrying] = ratatui::layout::Layout::vertical([
                Constraint::Length(card_height),
                Constraint::Min(4),
            ])
            .areas(area);
            render_adventurer(frame, adventurer, model, false);
            render_scrying(frame, scrying, model);
        }
    }
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
                    campaign.label,
                    campaign.party.len(),
                    if campaign.party.len() == 1 { "" } else { "s" }
                ))
            })
            .collect()
    };
    render_panel(frame, area, " QUEST BOARD ", Text::from(lines));
}

fn render_party(frame: &mut Frame<'_>, area: Rect, model: &Model) {
    let selected = model.selected_agent_key();
    let lines = model
        .domain()
        .agents
        .iter()
        .map(|(key, agent)| {
            let marker = if selected == Some(key) { ">" } else { " " };
            Line::from(format!(
                "{marker} {}  {}",
                agent.persona.name,
                presence_with_elapsed(agent, model)
            ))
        })
        .collect::<Vec<_>>();
    render_panel(frame, area, " PARTY ROSTER ", Text::from(lines));
}

fn render_summons(frame: &mut Frame<'_>, area: Rect, model: &Model) {
    let mut lines = model
        .domain()
        .agents
        .values()
        .filter_map(summons_line)
        .collect::<Vec<_>>();
    if lines.is_empty() {
        lines.push(Line::styled("No calls await the Questmancer.", MUTED));
    }
    render_panel(frame, area, " CALLS FOR COUNSEL ", Text::from(lines));
}

fn summons_line(agent: &Agent) -> Option<Line<'static>> {
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
    Some(Line::styled(message, ACCENT))
}

fn render_chronicle(frame: &mut Frame<'_>, area: Rect, model: &Model) {
    let mut lines = model
        .domain()
        .chronicle
        .entries()
        .iter()
        .rev()
        .take(5)
        .map(|entry| Line::from(entry.summary.clone()))
        .collect::<Vec<_>>();
    if lines.is_empty() {
        lines.push(Line::styled("No deeds recorded yet.", MUTED));
    }
    render_panel(frame, area, " CHRONICLE ", Text::from(lines));
}

fn render_adventurer(frame: &mut Frame<'_>, area: Rect, model: &Model, include_summons: bool) {
    let Some(agent) = model.selected_agent() else {
        render_panel(
            frame,
            area,
            " ADVENTURER ",
            Text::from("No adventurer selected."),
        );
        return;
    };
    let campaign = model
        .domain()
        .campaigns
        .get(&agent.workspace_id)
        .map_or("Unknown campaign", |campaign| campaign.label.as_str());
    let mut lines = vec![
        Line::styled(agent.persona.name.clone(), ACCENT),
        Line::from(format!(
            "{:?} {:?} - {}",
            agent.persona.ancestry,
            agent.persona.class,
            agent.persona.epithet.as_str()
        )),
        Line::from(format!("Campaign: {campaign}")),
        Line::from(format!(
            "{} ({})",
            presence_with_elapsed(agent, model),
            agent.name
        )),
    ];
    if include_summons && let Some(line) = summons_line(agent) {
        lines.push(line);
    }
    if let Some(status) = &agent.custom_status {
        lines.push(Line::styled(status.clone(), MUTED));
    }
    render_panel(frame, area, " ADVENTURER ", Text::from(lines));
}

fn render_scrying(frame: &mut Frame<'_>, area: Rect, model: &Model) {
    let Some(agent) = model.selected_agent() else {
        render_panel(frame, area, " SCRYING TABLE ", Text::from(SCRYING_STILL));
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
            lines.push(Line::from(format!("Cause: {error}")));
        } else if preview.loading {
            lines.push(Line::styled(SCRYING_STILL, MUTED));
            lines.push(Line::styled("Tracing the selected adventurer...", MUTED));
        } else {
            lines.extend(preview.text.lines().map(|line| Line::from(line.to_owned())));
        }
    } else {
        lines.push(Line::styled(SCRYING_STILL, MUTED));
        lines.push(Line::styled("Select refresh to trace recent deeds.", MUTED));
    }
    if let Some(status) = model.status_message() {
        lines.push(Line::from(""));
        lines.push(Line::styled(status.to_owned(), MUTED));
    }
    render_panel(frame, area, " SCRYING TABLE ", Text::from(lines));
}

fn render_spoils(frame: &mut Frame<'_>, area: Rect, model: &Model) {
    let mut lines = vec![if model.reviewr_available() {
        Line::from("Reviewr stands ready.")
    } else {
        Line::styled("Reviewr is unavailable.", MUTED)
    }];
    if let Some(status) = model
        .status_message()
        .filter(|status| status.starts_with("The spoils cannot"))
    {
        lines.push(Line::styled(status.to_owned(), ACCENT));
    }
    render_panel(frame, area, " SPOILS DESK ", Text::from(lines));
}

fn render_footer(frame: &mut Frame<'_>, area: Rect, model: &Model) {
    if area.is_empty() {
        return;
    }
    let mut actions = vec!["[1] Guild Hall", "[2] Delve", "[tab] Open Chronicle"];
    if !model.domain().agents.is_empty() {
        actions.extend(["[j/k] Choose", "[/] Search"]);
        if let Some(agent) = model.selected_agent() {
            actions.extend(["[enter] Observe", "[r] Issue counsel", "[o] Scry again"]);
            if agent.attention.is_unread() {
                actions.push("[space] Acknowledge summons");
            }
            if model.reviewr_available() {
                actions.push("[v] Inspect spoils");
            }
        }
    }

    if area.height == 1 {
        frame.render_widget(
            Paragraph::new(actions.join("  "))
                .alignment(if area.width >= 120 {
                    Alignment::Center
                } else {
                    Alignment::Left
                })
                .style(MUTED),
            area,
        );
    } else {
        let split = actions.len().div_ceil(2);
        frame.render_widget(
            Paragraph::new(Text::from(vec![
                Line::from(actions[..split].join("  ")),
                Line::from(actions[split..].join("  ")),
            ]))
            .style(MUTED),
            area,
        );
    }
}

fn render_panel(frame: &mut Frame<'_>, area: Rect, title: &str, text: Text<'_>) {
    if area.is_empty() {
        return;
    }
    frame.render_widget(
        Paragraph::new(text)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(MUTED),
            )
            .style(INK)
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn presence_with_elapsed(agent: &Agent, model: &Model) -> String {
    let presence = presence_label(agent.presence);
    if !model.settings().show_elapsed_time {
        return presence.to_owned();
    }
    format!("{presence} {}", elapsed_label(agent, model))
}

fn elapsed_label(agent: &Agent, model: &Model) -> String {
    let elapsed = agent.presence_since.elapsed_until(model.now()).as_secs();
    if elapsed >= 60 {
        format!("{}m", elapsed / 60)
    } else {
        format!("{elapsed}s")
    }
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
