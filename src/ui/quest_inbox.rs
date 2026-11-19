use crate::{
    app::{Modal, Model},
    domain::Timestamp,
    scene::{
        assets::inbox_scroll,
        pixel::{PixelPoint, RgbBuffer},
        sprite::blit,
    },
    ui::{
        scene_adapter::flush_rgb,
        theme::{PARCHMENT, PARCHMENT_BORDER, PARCHMENT_RGB},
        work_board::{area, wrap_text},
    },
    work::{self, ArtifactKind, Freshness, Obligation, ObligationState, WorkSelection, inbox},
};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    widgets::{Block, Borders, Clear, Paragraph},
};

pub(crate) fn render(frame: &mut Frame<'_>, model: &Model, detail: bool) {
    let viewport = frame.area();
    if viewport.width < 8 || viewport.height < 6 {
        frame.render_widget(Paragraph::new("Inbox").style(PARCHMENT), viewport);
        return;
    }
    let mut area = area(viewport.width, viewport.height);
    area.x += viewport.x;
    area.y += viewport.y;
    frame.render_widget(Clear, area);
    let block = Block::bordered()
        .title(if detail {
            " QUEST INBOX / REVIEW "
        } else {
            " QUEST INBOX "
        })
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(PARCHMENT)
        .border_style(PARCHMENT_BORDER);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let footer = wrap_text(&key_legend(model, detail), inner.width);
    let footer_height = u16::try_from(footer.len())
        .unwrap_or(u16::MAX)
        .min(inner.height.saturating_sub(1));
    let body_height = inner.height.saturating_sub(footer_height);
    let body = Rect::new(inner.x, inner.y, inner.width, body_height);
    let (rows, text_area) = if detail {
        let rows = detail_lines(model, viewport.width);
        let scroll = usize::from(model.reading_scroll()).min(rows.len().saturating_sub(1));
        (
            rows.into_iter()
                .skip(scroll)
                .take(usize::from(body_height))
                .collect(),
            body,
        )
    } else {
        let text_area = queue_text_area(frame, body, model);
        (queue_lines(model, text_area.width, body_height), text_area)
    };
    frame.render_widget(Paragraph::new(rows.join("\n")).style(PARCHMENT), text_area);
    frame.render_widget(
        Paragraph::new(footer.join("\n")).style(PARCHMENT_BORDER),
        Rect::new(inner.x, body.bottom(), inner.width, footer_height),
    );
}

fn queue_text_area(frame: &mut Frame<'_>, body: Rect, model: &Model) -> Rect {
    if body.width < 68 || body.height < 6 {
        return body;
    }
    let sprite = inbox_scroll::illustration();
    let size = sprite.size();
    let mut pixels = RgbBuffer::filled(size.width, size.height, PARCHMENT_RGB);
    blit(sprite, PixelPoint::new(0, 0), &mut pixels);
    let icon = Rect::new(
        body.right() - size.width - 1,
        body.y + 1,
        size.width,
        size.height.div_ceil(2),
    );
    flush_rgb(
        frame.buffer_mut(),
        icon,
        &pixels,
        PARCHMENT_RGB,
        model.preferences().color_mode,
    );
    Rect {
        width: icon.x - body.x - 1,
        ..body
    }
}

fn key_legend(model: &Model, detail: bool) -> String {
    if !detail {
        let back = if model.inbox_is_home() {
            "[b] Board · Esc Room"
        } else {
            "Esc Board"
        };
        return format!(
            "j/k quests · Enter read · ! urgent\nSpace seen · s defer/restore · v review\n[a] Actions · [o] Observe · {back}"
        );
    }
    let mut legend = vec!["Tab next summons · ↑/↓ read · Esc back"];
    if inbox::owned_review(model).is_some_and(|(o, _)| o.state == ObligationState::Open) {
        legend.push(
            if inbox::current_detail(model)
                .is_some_and(|d| d.review.check_passed() && d.changes.is_ok())
            {
                "[c] Import check · [x] Settle review"
            } else {
                "[c] Import check · [g] Refresh"
            },
        );
    }
    if model.selected_agent().is_some() {
        legend.push("[l] Link · [o] Observe");
    }
    legend.join("\n")
}

fn queue_lines(model: &Model, width: u16, height: u16) -> Vec<String> {
    let rows = inbox::rows(model.work(), model.now());
    let selected = model
        .work_selection()
        .and_then(|s| rows.iter().position(|row| row.selected(s)));
    let due = rows
        .iter()
        .filter(|row| row.due && row.quest.is_some())
        .count();
    let campaign_due = rows
        .iter()
        .filter(|row| row.due && row.quest.is_none())
        .count();
    let quests = if due == 1 { "quest" } else { "quests" };
    let mut lines = vec![format!(
        "{due} {quests} due · {campaign_due} campaign summons"
    )];
    if let Some(feedback) = model.action_feedback() {
        lines.extend(wrap_text(feedback, width));
    }
    if rows.is_empty() {
        let prompt = if model.authoring().metadata.is_some() {
            "No summons. Select a quest on the Board and press v to request a review."
        } else {
            "No summons. Press a for Actions to open or create a local store."
        };
        lines.extend(wrap_text(prompt, width));
        return lines;
    }
    let capacity = usize::from(height).saturating_sub(lines.len()).max(1);
    let start = selected
        .unwrap_or(0)
        .saturating_sub(capacity / 2)
        .min(rows.len().saturating_sub(capacity));
    for (index, row) in rows.iter().enumerate().skip(start).take(capacity) {
        let Some(o) = model.work().obligation(&row.items[0]) else {
            continue;
        };
        let marker = if selected == Some(index) { '>' } else { ' ' };
        let title = title(model, o);
        let more = if row.items.len() > 1 {
            format!(" +{}", row.items.len() - 1)
        } else {
            String::new()
        };
        let action = summary(model, o);
        lines.push(format!("{marker} {action} {title}{more}"));
    }
    lines
}

fn title<'a>(model: &'a Model, o: &'a Obligation) -> &'a str {
    o.quest
        .as_ref()
        .and_then(|id| model.work().quest(id))
        .map_or_else(
            || {
                model
                    .work()
                    .campaign(&o.campaign)
                    .map_or(o.prompt.as_str(), |campaign| campaign.name.as_str())
            },
            |quest| &quest.title,
        )
}

fn summary(model: &Model, o: &Obligation) -> String {
    let status = match o.state {
        ObligationState::Resolved => "Reviewed".into(),
        ObligationState::Superseded => "Replaced review".into(),
        ObligationState::Open => inbox::deferred_until(model.work(), o, model.now()).map_or_else(
            || o.action.clone(),
            |until| format!("Deferred to {}", utc_time(until)),
        ),
    };
    let verified = model.work().sources.values().any(|source| {
        source.freshness == Freshness::Current && source.inventory.obligations.contains_key(&o.id)
    });
    let seen = model.work().intent.get(&o.id).is_some_and(|intent| {
        intent.incarnation == o.incarnation && intent.seen && o.state == ObligationState::Open
    });
    let freshness = if verified { "" } else { "Unverified · " };
    let seen = if seen { "Seen · " } else { "" };
    format!("{freshness}{seen}{status}")
}

fn utc_time(timestamp: Timestamp) -> String {
    time::OffsetDateTime::from_unix_timestamp(timestamp.as_millis().div_euclid(1000)).map_or_else(
        |_| format!("{} ms UTC", timestamp.as_millis()),
        |at| format!("{:02}:{:02} UTC", at.hour(), at.minute()),
    )
}

pub(crate) fn detail_lines(model: &Model, width: u16) -> Vec<String> {
    let Some(WorkSelection::Obligation(id)) = model.work_selection() else {
        return vec!["No selected summons".into()];
    };
    let Some(o) = model.work().obligation(id) else {
        return vec!["Summons unavailable".into()];
    };
    let mut lines = vec![format!("{} · {}", title(model, o), summary(model, o))];
    if let Some(feedback) = model.action_feedback() {
        lines.push(feedback.into());
    }
    if model.authoring().busy() {
        lines.push(format!("{} · [a] Actions", model.authoring().phase.label()));
    }
    if inbox::reviewed_obligation(model).is_some() {
        if let Some(detail) = inbox::current_detail(model) {
            lines.extend(evidence_lines(detail, o));
        } else {
            lines.push(if inbox::owned_review(model).is_some() {
                "Loading selected review evidence…".into()
            } else {
                "Review evidence unavailable from the owned store.".into()
            });
            lines.push(format!("Criterion · {}", o.prompt));
        }
        append_pull_requests(model, o, &mut lines);
    } else {
        lines.extend(work::view::details(model));
    }
    if let Some(agent) = inbox::observer(model) {
        lines.push(format!("Linked adventurer · {}", agent.name));
    } else if let Some(agent) = model.selected_agent() {
        lines.push(format!("[l] Link selected adventurer · {}", agent.name));
    }
    if !matches!(model.modal(), Modal::QuestInbox { detail: true }) {
        return lines;
    }
    let inner = area(width, 40).width.saturating_sub(2);
    wrap_text(&lines.join("\n"), inner)
}

fn evidence_lines(
    detail: &crate::owned_work::authoring::ReviewDetail,
    o: &Obligation,
) -> Vec<String> {
    let review = &detail.review;
    let target = &review.target;
    let mut lines = if let Some(settlement) = &review.settlement {
        vec![format!(
            "Settled by {} at {}",
            settlement.actor,
            utc_time(settlement.at)
        )]
    } else if o.state == ObligationState::Superseded {
        vec!["A newer review replaced this request.".into()]
    } else if detail.changes.is_err() {
        vec!["Next · Restore access to the pinned change summary.".into()]
    } else if review.check_passed() {
        vec!["Next · Compare the evidence, then [x] Settle review.".into()]
    } else {
        vec!["Next · Run the required check and [c] Import its receipt.".into()]
    };
    lines.push(format!("Candidate · {}", target.head));
    lines.push(format!("Required check · {}", target.recipe));
    if let Some(receipt) = &review.check {
        lines.push(format!(
            "{:?} · {} · {}",
            receipt.result,
            receipt.runner,
            utc_time(receipt.captured_at)
        ));
    } else {
        lines.push(format!("Awaiting receipt · {}", target.runner));
    }
    lines.push(format!("Criterion · {}", o.prompt));
    lines.push(format!("Base · {}", target.base));
    lines.push(format!("Checkout · {}", target.repository.display()));
    match &detail.changes {
        Ok(files) if files.is_empty() => {
            lines.push("No changed files between the pinned commits.".into());
        }
        Ok(files) => {
            lines.push(format!("Changed files · {}", files.len()));
            lines.extend(files.iter().cloned());
        }
        Err(error) => lines.push(format!("Change summary unavailable · {error}")),
    }
    if let Some(report) = &review.report {
        lines.push(format!("{:?} report · {}", report.author, report.summary));
    }
    lines
}

fn append_pull_requests(model: &Model, o: &Obligation, lines: &mut Vec<String>) {
    if o.quest.is_none() {
        return;
    }
    for artifact in model
        .work()
        .sources
        .values()
        .flat_map(|s| s.inventory.artifacts.values())
        .filter(|a| a.quest == o.quest && a.kind == ArtifactKind::PullRequest)
    {
        lines.push(format!("PR · {}", artifact.title));
        if let Some(uri) = &artifact.uri {
            lines.push(uri.clone());
        }
    }
}
