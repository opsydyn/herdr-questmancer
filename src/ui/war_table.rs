//! Contextual text over the RGB work map; all facts come from the work projection.
use super::{
    theme::{PARCHMENT, PARCHMENT_BORDER},
    work_board::wrap_text,
};
use crate::{
    app::Model,
    scene::{
        pixel::{PixelRect, PixelSize},
        presentation::ScenePresentation,
        war_table::{MapMode, TablePlan},
    },
    work::graph,
};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    text::Text,
    widgets::{Block, Borders, Clear, Paragraph},
};

fn cells(rect: PixelRect, offset: Rect) -> Rect {
    Rect::new(
        offset.x + u16::try_from(rect.x).unwrap_or(0),
        offset.y + u16::try_from(rect.y / 2).unwrap_or(0),
        rect.width,
        rect.height / 2,
    )
}
pub fn detail_line_count(model: &Model, width: u16, height: u16) -> usize {
    let plan = TablePlan::project(
        graph::CampaignGraph::from_model(model).as_ref(),
        PixelSize::new(width, height.saturating_mul(2)),
    );
    wrap_text(&graph::details(model).join("\n"), plan.details.width)
        .len()
        .max(1)
}
pub fn render(frame: &mut Frame<'_>, model: &Model, presentation: &ScenePresentation) {
    let viewport = frame.area();
    if viewport.width < 28 || viewport.height < 12 {
        paint_text(
            frame,
            Paragraph::new(
                "War Table
w/Esc: close",
            )
            .style(PARCHMENT),
            viewport,
        );
        return;
    }
    let graph = presentation.war_table.as_ref();
    let plan = TablePlan::project(
        graph,
        PixelSize::new(viewport.width, viewport.height.saturating_mul(2)),
    );
    let board = cells(plan.board, viewport);
    frame.render_widget(
        Block::default()
            .title(" WAR TABLE ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(PARCHMENT_BORDER)
            .title_style(PARCHMENT_BORDER),
        board,
    );
    render_heading(frame, board, graph, &plan);
    render_cards(frame, graph, &plan, viewport);
    let rows = wrap_text(&graph::details(model).join("\n"), plan.details.width);
    let scroll = usize::from(model.reading_scroll()).min(rows.len().saturating_sub(1));
    paint_text(
        frame,
        Paragraph::new(Text::from(
            rows.into_iter().skip(scroll).collect::<Vec<_>>().join("\n"),
        ))
        .style(PARCHMENT),
        cells(plan.details, viewport),
    );
    let footer = if board.width < 72 {
        "j/k quests  Tab campaigns\nUp/Down read  b Board  w close"
    } else {
        "j/k: quests   Tab/Shift-Tab: campaigns   Up/Down: read details\nb: Quest Board   Enter: observe verified pane   !: summons   w/Esc: close"
    };
    for (row, line) in footer.lines().enumerate() {
        paint_text(
            frame,
            Paragraph::new(fit(line, board.width.saturating_sub(2))).style(PARCHMENT_BORDER),
            Rect::new(
                board.x + 1,
                board.bottom().saturating_sub(3) + u16::try_from(row).unwrap_or(0),
                board.width.saturating_sub(2),
                1,
            ),
        );
    }
}
fn fit(text: &str, width: u16) -> String {
    let width = usize::from(width);
    if ratatui::text::Line::raw(text).width() <= width {
        return text.to_owned();
    }
    let budget = width.saturating_sub(3);
    let mut cells = 0;
    let mut result = String::new();
    for c in text.chars() {
        let size = ratatui::text::Span::raw(c.to_string()).width();
        if cells + size > budget {
            break;
        }
        result.push(c);
        cells += size;
    }
    result.push_str(&".".repeat(width.min(3)));
    result
}

fn render_heading(
    frame: &mut Frame<'_>,
    board: Rect,
    graph: Option<&graph::CampaignGraph>,
    plan: &TablePlan,
) {
    let header = if let Some(g) = graph {
        let complete = g
            .nodes
            .values()
            .filter(|n| n.state == crate::work::QuestState::Completed)
            .count();
        let selected = g
            .selected
            .as_ref()
            .and_then(|id| g.nodes.keys().position(|q| q == id))
            .map_or(0, |i| i + 1);
        vec![
            format!(
                "{} | {:?} | Completed {complete}/{}",
                g.campaign.name,
                g.campaign.state,
                g.nodes.len()
            ),
            format!(
                "Source: {} | {} | Human requests: {} | Spoils: {}",
                g.source.0,
                graph::freshness_label(g.freshness),
                g.human_requests,
                g.artifact_count
            ),
            format!(
                "Quest {selected}/{} | {}",
                g.nodes.len(),
                if g.nodes.is_empty() {
                    "No quests are supplied for this campaign"
                } else {
                    match plan.mode {
                        MapMode::Dependencies
                            if g.nodes.values().any(|n| {
                                !n.outside_dependencies.is_empty()
                                    || !n.missing_dependencies.is_empty()
                            }) =>
                        {
                            "Within-campaign map; outside/missing prerequisites in details"
                        }
                        MapMode::Dependencies => {
                            "Prerequisite -> dependent; ? = open human request; R = review"
                        }
                        MapMode::Inventory if g.has_cycle() => {
                            "Cycle present: inventory; dependencies in details"
                        }
                        MapMode::Inventory => {
                            "Map exceeds space: paged inventory; dependencies in details"
                        }
                        MapMode::DetailsOnly => "Map does not fit; read dependencies below",
                    }
                }
            ),
        ]
    } else {
        vec![
            "No campaign source is connected.".into(),
            "Herdr adventurer summons remain available with !.".into(),
            "The War Table awaits source-owned work.".into(),
        ]
    };
    for (row, line) in header.iter().enumerate() {
        paint_text(
            frame,
            Paragraph::new(fit(line, board.width.saturating_sub(2))).style(PARCHMENT_BORDER),
            Rect::new(
                board.x + 1,
                board.y + 1 + u16::try_from(row).unwrap_or(0),
                board.width.saturating_sub(2),
                1,
            ),
        );
    }
}

fn paint_text(frame: &mut Frame<'_>, paragraph: Paragraph<'_>, area: Rect) {
    frame.render_widget(Clear, area);
    frame.render_widget(paragraph, area);
}

fn render_cards(
    frame: &mut Frame<'_>,
    graph: Option<&graph::CampaignGraph>,
    plan: &TablePlan,
    viewport: Rect,
) {
    if let Some(g) = graph {
        for card in &plan.cards {
            let node = &g.nodes[&card.id];
            let rect = cells(card.bounds, viewport);
            paint_text(
                frame,
                Paragraph::new(format!("{:?}", node.state)).style(PARCHMENT),
                Rect::new(rect.x + 7, rect.y + 1, rect.width - 8, 1),
            );
            let mut title = wrap_text(&node.title, rect.width - 2);
            if title.len() > 2 {
                title[1] = fit(&format!("{} ...", title[1]), rect.width - 2);
            }
            for (row, line) in title.iter().take(2).enumerate() {
                paint_text(
                    frame,
                    Paragraph::new(line.clone()).style(PARCHMENT),
                    Rect::new(
                        rect.x + 1,
                        rect.y + 3 + u16::try_from(row).unwrap_or(0),
                        rect.width - 2,
                        1,
                    ),
                );
            }
            let suffix = format!(
                "{}{}",
                if node.human_requests > 0 {
                    format!(" ?{}", node.human_requests)
                } else {
                    String::new()
                },
                if node.review_requests > 0 {
                    format!(" R{}", node.review_requests)
                } else {
                    String::new()
                }
            );
            let prefix = format!("{}: ", card.ordinal);
            let width = usize::from(rect.width - 2).saturating_sub(prefix.len() + suffix.len());
            let owner = format!(
                "{}{}{}",
                prefix,
                fit(
                    node.assignment.as_deref().unwrap_or("Unassigned"),
                    u16::try_from(width).unwrap_or(0)
                ),
                suffix
            );
            paint_text(
                frame,
                Paragraph::new(fit(&owner, rect.width - 2)).style(PARCHMENT),
                Rect::new(rect.x + 1, rect.y + 5, rect.width - 2, 1),
            );
        }
    }
}
