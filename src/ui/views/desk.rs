use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Rect},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::{
    app::{ConnectionState, Model},
    domain::{Agent, CampaignStatus, GuildSummons, Presence},
    ui::theme::{ACCENT, INK, MUTED},
};

pub(crate) fn render(frame: &mut Frame<'_>, model: &Model) {
    let area = frame.area();
    if area.width < 4 || area.height < 3 {
        frame.render_widget(Paragraph::new("W").style(ACCENT), area);
        return;
    }

    let [body, footer] =
        ratatui::layout::Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(area);
    let title = format!(
        " WEBMASTER CONTROL CENTRE - {} ",
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

    if model.domain().agents.is_empty() {
        render_empty(frame, inner);
    } else if inner.width >= 116 {
        let [sites, mail, live] = ratatui::layout::Layout::horizontal([
            Constraint::Percentage(25),
            Constraint::Percentage(34),
            Constraint::Min(44),
        ])
        .areas(inner);
        render_sites(frame, sites, model);
        render_mail(frame, mail, model);
        render_agent(frame, live, model);
    } else if inner.width >= 76 {
        let [mail, live] =
            ratatui::layout::Layout::horizontal([Constraint::Percentage(45), Constraint::Min(38)])
                .areas(inner);
        render_mail(frame, mail, model);
        render_agent(frame, live, model);
    } else {
        render_agent(frame, inner, model);
    }

    let mut footer_actions = vec!["[1] desk", "[2] cafe", "[tab] region"];
    if !model.domain().agents.is_empty() {
        footer_actions.push("[/] search");
    }
    if let Some(agent) = model.selected_agent() {
        footer_actions.extend(["[enter] visit", "[r] reply", "[o] output"]);
        if agent.attention.is_unread() {
            footer_actions.push("[space] seen");
        }
        if model.reviewr_available() {
            footer_actions.push("[v] reviewr");
        }
    }
    frame.render_widget(
        Paragraph::new(footer_actions.join("  "))
            .alignment(Alignment::Center)
            .style(MUTED),
        footer,
    );
}

fn render_empty(frame: &mut Frame<'_>, area: Rect) {
    let message = Text::from(vec![
        Line::from(""),
        Line::styled("No agents online", ACCENT),
        Line::from(""),
        Line::styled("Start an agent to put a site under construction", MUTED),
    ]);
    frame.render_widget(
        Paragraph::new(message)
            .alignment(Alignment::Center)
            .style(INK),
        area,
    );
}

fn render_sites(frame: &mut Frame<'_>, area: Rect, model: &Model) {
    let lines = model
        .domain()
        .campaigns
        .values()
        .map(|site| {
            let status = site.status(&model.domain().agents);
            Line::from(format!(
                "{} {}  {} contributor{}",
                site_marker(status),
                site.label,
                site.party.len(),
                if site.party.len() == 1 { "" } else { "s" }
            ))
        })
        .collect::<Vec<_>>();
    render_panel(frame, area, " YOUR SITES ", Text::from(lines));
}

fn render_mail(frame: &mut Frame<'_>, area: Rect, model: &Model) {
    let mut lines = Vec::new();
    for agent in model.domain().agents.values() {
        if agent.attention.is_unread() || agent.presence == Presence::Blocked {
            let subject = attention_label(agent).unwrap_or_else(|| presence_label(agent.presence));
            lines.push(Line::styled(
                format!("NEW {} - {subject}", agent.name),
                ACCENT,
            ));
            if let Some(status) = &agent.custom_status {
                lines.push(Line::styled(format!("    {status}"), MUTED));
            }
        }
    }
    if lines.is_empty() {
        lines.push(Line::styled("No unread webmaster mail", MUTED));
    }
    lines.push(Line::from(""));
    lines.push(Line::styled("GUESTBOOK", ACCENT));
    for entry in model.domain().chronicle.entries().iter().rev().take(4) {
        lines.push(Line::from(entry.summary.clone()));
    }
    render_panel(frame, area, " WEBMASTER MAIL ", Text::from(lines));
}

fn render_agent(frame: &mut Frame<'_>, area: Rect, model: &Model) {
    let Some(agent) = model.selected_agent() else {
        render_panel(frame, area, " LIVE PAGE ", Text::from("No agent selected"));
        return;
    };
    let presence = if model.settings().show_elapsed_time {
        format!(
            "{} {}",
            presence_label(agent.presence),
            elapsed_label(agent, model)
        )
    } else {
        presence_label(agent.presence).to_owned()
    };
    let site = model
        .domain()
        .campaigns
        .get(&agent.workspace_id)
        .map_or("unknown site", |site| site.label.as_str());
    let mut lines = vec![
        Line::styled(agent.name.clone(), ACCENT),
        Line::from(format!("{site} / {}", agent.persona.handle)),
        Line::from(presence),
    ];
    if let Some(label) = attention_label(agent) {
        lines.push(Line::styled(label, ACCENT));
    }
    if let Some(status) = &agent.custom_status {
        lines.push(Line::from(format!("status: {status}")));
    }
    lines.push(Line::from(""));
    lines.push(Line::styled("RECENT OUTPUT", ACCENT));
    if let Some(preview) = model
        .output_preview()
        .filter(|preview| preview.pane_id == agent.pane_id)
        .filter(|_| model.managed_pane_id() != Some(&agent.pane_id))
    {
        lines.extend(preview.text.lines().map(|line| Line::from(line.to_owned())));
    } else {
        lines.push(Line::styled("loading selected page...", MUTED));
    }
    if let Some(status) = model.status_message() {
        lines.push(Line::from(""));
        lines.push(Line::styled(status.to_owned(), MUTED));
    }
    render_panel(frame, area, " LIVE PAGE ", Text::from(lines));
}

fn attention_label(agent: &Agent) -> Option<&'static str> {
    if agent.presence == Presence::Exited {
        return Some("BROKEN LINK");
    }
    if agent.attention.is_unread() {
        return match agent.attention.summons() {
            Some(GuildSummons::CounselRequested) => Some("NEEDS WEBMASTER"),
            Some(GuildSummons::SpoilsReturned) => Some("UPDATE READY - AWAITING WEBMASTER"),
            Some(GuildSummons::AdventurerDeparted) => Some("BROKEN LINK"),
            None => None,
        };
    }
    (agent.presence == Presence::Blocked).then_some("NEEDS WEBMASTER")
}

fn render_panel(frame: &mut Frame<'_>, area: Rect, title: &str, text: Text<'_>) {
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
        Presence::Done => "done",
        Presence::Idle => "idle",
        Presence::Exited => "exited",
        Presence::Unknown => "unknown",
    }
}

const fn site_marker(status: CampaignStatus) -> &'static str {
    match status {
        CampaignStatus::CounselRequired => "!",
        CampaignStatus::SpoilsAwaitingInspection => "+",
        CampaignStatus::ExpeditionActive => "~",
        CampaignStatus::PartyAtRest => "*",
        CampaignStatus::Abandoned => "x",
    }
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
