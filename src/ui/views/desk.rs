use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Rect},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::{
    app::{ConnectionState, Model},
    domain::{Agent, AttentionReason, Presence, SiteStatus},
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
        if agent.attention.is_unseen() {
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
        .sites
        .values()
        .map(|site| {
            let status = site.status(&model.domain().agents);
            Line::from(format!(
                "{} {}  {} contributor{}",
                site_marker(status),
                site.label,
                site.agents.len(),
                if site.agents.len() == 1 { "" } else { "s" }
            ))
        })
        .collect::<Vec<_>>();
    render_panel(frame, area, " YOUR SITES ", Text::from(lines));
}

fn render_mail(frame: &mut Frame<'_>, area: Rect, model: &Model) {
    let mut lines = Vec::new();
    for agent in model.domain().agents.values() {
        if agent.attention.is_unseen() || agent.presence == Presence::Blocked {
            let subject = match agent.attention.reason() {
                Some(AttentionReason::NeedsInput) => "NEEDS WEBMASTER",
                Some(AttentionReason::WorkCompleted) => "UPDATE READY",
                Some(AttentionReason::PaneExited) => "BROKEN LINK",
                None => presence_label(agent.presence),
            };
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
    for entry in model.domain().guestbook.entries().iter().rev().take(4) {
        lines.push(Line::from(entry.summary.clone()));
    }
    render_panel(frame, area, " WEBMASTER MAIL ", Text::from(lines));
}

fn render_agent(frame: &mut Frame<'_>, area: Rect, model: &Model) {
    let Some(agent) = model.selected_agent() else {
        render_panel(frame, area, " LIVE PAGE ", Text::from("No agent selected"));
        return;
    };
    let elapsed = elapsed_label(agent, model);
    let site = model
        .domain()
        .sites
        .get(&agent.workspace_id)
        .map_or("unknown site", |site| site.label.as_str());
    let mut lines = vec![
        Line::styled(agent.name.clone(), ACCENT),
        Line::from(format!("{site} / {}", agent.persona.handle)),
        Line::from(format!("{} {elapsed}", presence_label(agent.presence))),
    ];
    if agent.presence == Presence::Blocked {
        lines.push(Line::styled("NEEDS WEBMASTER", ACCENT));
    }
    if let Some(status) = &agent.custom_status {
        lines.push(Line::from(format!("status: {status}")));
    }
    lines.push(Line::from(""));
    lines.push(Line::styled("RECENT OUTPUT", ACCENT));
    if let Some(preview) = model.output_preview() {
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

const fn site_marker(status: SiteStatus) -> &'static str {
    match status {
        SiteStatus::NeedsWebmaster => "!",
        SiteStatus::UpdateReady => "+",
        SiteStatus::Updating => "~",
        SiteStatus::Online => "*",
        SiteStatus::Offline => "x",
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
