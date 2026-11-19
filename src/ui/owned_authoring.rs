//! Contextual parchment using the existing production scene overlay path.
use crate::{
    app::Model,
    owned_work::authoring::{Authoring, FormKind, FormTarget, Surface, menu_entries},
    ui::{
        theme::{PARCHMENT, PARCHMENT_BORDER},
        work_board::wrap_text,
    },
};
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Clear, Paragraph},
};

pub fn heading(kind: FormKind) -> &'static str {
    match kind {
        FormKind::CreateStore => "Create local store",
        FormKind::OpenStore => "Open existing store",
        FormKind::Campaign => "Create campaign",
        FormKind::Quest => "Create quest",
        FormKind::QuestLifecycle => "Change quest state",
        FormKind::QuestPrerequisites => "Edit prerequisites",
        FormKind::QuestAssignment => "Edit quest assignment",
        FormKind::Request => "Request decision",
        FormKind::Answer => "Answer decision",
        FormKind::Artifact => "Register artifact",
        FormKind::RequestReview => "Request revision review",
        FormKind::RecordCheck => "Import check receipt",
        FormKind::SettleReview => "Settle review",
    }
}
fn detail_rows(a: &Authoring, compose_hint: bool) -> Vec<String> {
    let Some(d) = &a.detail else {
        return vec!["Loading/checking selected decision...".into()];
    };
    let mut rows = vec![
        d.prompt.clone(),
        format!("Requested by: {}", d.requested_by),
    ];
    if let Some(context) = &d.context {
        rows.push(format!("Context: {context}"));
    }
    if let Some(answer) = &d.answer {
        rows.push(format!("Answer: {}", answer.text));
        rows.push(format!("Answered by: {} at {:?}", answer.actor, answer.at));
    } else {
        rows.push(if compose_hint {
            "Open decision. Enter: compose answer".into()
        } else {
            "Open decision".into()
        });
    }
    rows
}
/// Returns wrapped body and focus anchor. Tests exercise content and scrolling, not artwork.
pub fn rows(a: &Authoring, width: u16) -> (Vec<String>, usize) {
    let mut body = vec![format!("Owned campaign: {}", a.campaign_label())];
    if !a.feedback.is_empty() {
        body.push(a.feedback.clone());
    }
    let mut anchor = 0;
    match a.surface {
        Surface::Menu => {
            for (i, label) in menu_entries().iter().enumerate() {
                let marker = if i == a.cursor { ">" } else { " " };
                if i == a.cursor {
                    anchor = body.iter().map(|s| wrap_text(s, width).len()).sum();
                }
                body.push(format!("{marker} {label}"));
            }
        }
        Surface::Campaigns => {
            body.push("Choose an owned campaign (including empty campaigns)".into());
            for (i, c) in a
                .metadata
                .iter()
                .flat_map(|m| m.inventory.campaigns.values())
                .enumerate()
            {
                if i == a.cursor {
                    anchor = body.iter().map(|s| wrap_text(s, width).len()).sum();
                }
                body.push(format!(
                    "{} {} [{}]",
                    if i == a.cursor { ">" } else { " " },
                    c.name,
                    c.id.as_str()
                ));
            }
        }
        Surface::Decisions => {
            body.push("Decisions: open and answered".into());
            for (i, o) in a
                .metadata
                .iter()
                .flat_map(|m| m.inventory.obligations.values())
                .filter(|o| {
                    o.kind == crate::work::ObligationKind::Decision
                        && a.campaign.as_ref().is_none_or(|c| c == &o.campaign)
                })
                .enumerate()
            {
                if i == a.cursor {
                    anchor = body.iter().map(|s| wrap_text(s, width).len()).sum();
                }
                body.push(format!(
                    "{} {:?}: {} [{}]",
                    if i == a.cursor { ">" } else { " " },
                    o.state,
                    o.prompt,
                    o.id.as_str()
                ));
            }
        }
        Surface::Detail => body.extend(detail_rows(a, true)),
        Surface::Store => {
            if let Some(s) = &a.selection {
                body.push(format!("Store path: {}", s.path.display()));
                body.push(format!("Expected source ID: {}", s.source.0));
            } else {
                body.push(
                    "No selected store. Choose Create local store or Open existing store.".into(),
                );
            }
            body.push(format!("Owner: {}", a.phase.label()));
            body.extend(registered_spoils(a));
            body.push("Only acknowledged owned records are writable. Store creation never imports cached or external work.".into());
        }
        Surface::Form(kind) => {
            anchor = form_rows(a, kind, width, &mut body);
        }
    }
    (
        body.into_iter()
            .flat_map(|s| wrap_text(&s, width))
            .collect(),
        anchor,
    )
}
pub fn render(frame: &mut Frame<'_>, model: &Model) {
    let v = frame.area();
    let a = model.authoring();
    if v.width < 28 || v.height < 12 {
        frame.render_widget(Clear, v);
        frame.render_widget(Paragraph::new("Owned work: resize to at least 28 columns and 12 rows. Draft retained; submission disabled.").style(PARCHMENT).wrap(ratatui::widgets::Wrap{trim:false}),v);
        return;
    }
    let width = v.width.saturating_sub(4).min(100);
    let height = v.height.saturating_sub(2).min(38);
    let area = Rect::new(
        v.x + (v.width - width) / 2,
        v.y + (v.height - height) / 2,
        width,
        height,
    );
    frame.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" OWNED WORK / QUEST BOARD ")
        .style(PARCHMENT)
        .border_style(PARCHMENT_BORDER);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let footer_height = if inner.width < 48 { 4 } else { 3 };
    let title = match a.surface {
        Surface::Form(kind) => heading(kind),
        Surface::Detail => "Decision details",
        Surface::Store => "Owned store / result",
        _ => "Owned work actions",
    };
    frame.render_widget(
        Paragraph::new(title).style(PARCHMENT_BORDER),
        Rect::new(inner.x, inner.y, inner.width, 1),
    );
    let body = Rect::new(
        inner.x,
        inner.y + 1,
        inner.width,
        inner.height.saturating_sub(footer_height + 1),
    );
    let (rows, anchor) = rows(a, body.width);
    let scroll = if matches!(
        a.surface,
        Surface::Menu | Surface::Campaigns | Surface::Decisions
    ) || a.follow_focus && matches!(a.surface, Surface::Form(_))
    {
        anchor.saturating_sub(usize::from(body.height).saturating_sub(1))
    } else {
        a.scroll
    }
    .min(rows.len().saturating_sub(usize::from(body.height)));
    frame.render_widget(
        Paragraph::new(
            rows.into_iter()
                .skip(scroll)
                .take(usize::from(body.height))
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .style(PARCHMENT),
        body,
    );
    let footer = if inner.width < 48 {
        "Tab: field/action\nEnter: button/newline\nUp/Down: choose/read\nEsc: back / keep draft"
    } else if matches!(a.surface, Surface::Form(_)) {
        "Tab: field/action  Up/Down: choose/read\nEnter: newline or focused button\nEsc: keep draft  Ctrl-U: clear field"
    } else {
        "Up/Down: choose/read  Enter: open\nEsc: back / Board\nDrafts are volatile; quit discards them."
    };
    frame.render_widget(
        Paragraph::new(footer)
            .style(PARCHMENT_BORDER)
            .wrap(ratatui::widgets::Wrap { trim: false }),
        Rect::new(inner.x, body.bottom(), inner.width, footer_height),
    );
}

fn form_context(a: &Authoring, kind: FormKind, body: &mut Vec<String>) {
    match (kind, &a.form_target) {
        (FormKind::RequestReview, Some(FormTarget::Quest { title, .. })) => {
            body.push(format!("Quest: {title}"));
            body.push("A new request replaces any open review for this quest.".into());
        }
        (FormKind::RecordCheck | FormKind::SettleReview, Some(FormTarget::Review { title, head, .. })) => {
            body.push(format!("Quest: {title}"));
            body.push(format!("Candidate: {head}"));
            if kind == FormKind::RecordCheck {
                body.push("Import only the receipt from the runner you chose for this review.".into());
            }
        }
        (FormKind::QuestLifecycle, Some(FormTarget::QuestLifecycle { title, change, .. })) => {
            let (from, to) = change.states();
            body.push(format!("Quest: {title}"));
            body.push(format!("{} · {from:?} → {to:?}", change.label()));
        }
        (FormKind::QuestPrerequisites, Some(FormTarget::QuestPrerequisites { title, .. })) => {
            body.push(format!("Dependent quest: {title}"));
        }
        (FormKind::QuestAssignment, Some(FormTarget::QuestAssignment { title, current, .. })) => {
            body.push(format!("Quest: {title}"));
            let name = current.as_ref().map_or("Unassigned", |current| current.name.as_str());
            body.push(format!("Current assignee: {name}"));
        }
        (FormKind::Answer, _) => body.extend(detail_rows(a, false)),
        (FormKind::CreateStore | FormKind::OpenStore, _) => body.push(
            "Parent must already exist physically. Existing/missing selections are never replaced automatically.".into(),
        ),
        _ => {}
    }
}

fn form_rows(a: &Authoring, kind: FormKind, width: u16, body: &mut Vec<String>) -> usize {
    let mut anchor = 0;
    body.push(heading(kind).into());
    form_context(a, kind, body);
    for (i, f) in a.fields.iter().enumerate() {
        let marker = if a.focus == i { ">" } else { " " };
        body.push(format!("{marker} {}:", f.label));
        body.push(format!(
            "{}{}",
            f.display(),
            if a.focus == i && f.choices.is_empty() {
                "|"
            } else {
                ""
            }
        ));
        if a.focus == i {
            anchor = body
                .iter()
                .map(|s| wrap_text(s, width).len())
                .sum::<usize>()
                .saturating_sub(1);
        }
    }
    body.push(format!(
        "{} [{}]",
        if a.focus == a.fields.len() { ">" } else { " " },
        match kind {
            FormKind::Answer => "Save answer",
            FormKind::QuestPrerequisites => "Save prerequisite change",
            FormKind::QuestAssignment => "Save assignment change",
            _ => heading(kind),
        }
    ));
    if a.focus == a.fields.len() {
        anchor = body
            .iter()
            .map(|s| wrap_text(s, width).len())
            .sum::<usize>()
            .saturating_sub(1);
    }
    body.push(format!(
        "{} [Cancel / keep draft]",
        if a.focus == a.fields.len() + 1 {
            ">"
        } else {
            " "
        }
    ));
    if a.focus == a.fields.len() + 1 {
        anchor = body
            .iter()
            .map(|s| wrap_text(s, width).len())
            .sum::<usize>()
            .saturating_sub(1);
    }
    anchor
}

fn registered_spoils(a: &Authoring) -> Vec<String> {
    a.metadata
        .iter()
        .flat_map(|m| m.inventory.artifacts.values())
        .filter(|artifact| a.campaign.as_ref().is_none_or(|c| c == &artifact.campaign))
        .flat_map(|artifact| {
            let mut lines = vec![format!(
                "Registered spoils: {} | {:?}",
                artifact.title, artifact.kind
            )];
            if let Some(uri) = &artifact.uri {
                lines.push(format!("URI: {uri}"));
            }
            lines
        })
        .collect()
}
