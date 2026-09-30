//! Static RGB work map, painted inside the existing scene frame.
use crate::scene::{
    assets::{
        palette::{
            AMBER_LIGHT, INK_BLUE, PARCHMENT, PARCHMENT_DARK, RUG_DARK, SELECTION_RUNE, WINE_LIGHT,
        },
        roster,
    },
    pixel::{PixelPoint, PixelRect, RgbBuffer},
    sprite::blit,
    stage::ScenePose,
    war_table::{MapMode, TablePlan},
};
use crate::work::{QuestState, graph::CampaignGraph};
pub fn paint(graph: Option<&CampaignGraph>, plan: &TablePlan, target: &mut RgbBuffer) {
    target.fill_rect(plan.board, RUG_DARK);
    target.fill_rect(plan.details, PARCHMENT);
    if let Some(map) = plan.map {
        target.fill_rect(map, PARCHMENT_DARK);
    }
    let Some(graph) = graph else {
        return;
    };
    if plan.mode == MapMode::Dependencies {
        for (from, to) in &graph.edges {
            let Some(a) = plan.cards.iter().find(|p| &p.id == from) else {
                continue;
            };
            let Some(b) = plan.cards.iter().find(|p| &p.id == to) else {
                continue;
            };
            let x1 = a.bounds.x + i32::from(a.bounds.width);
            let y1 = a.bounds.y + 7;
            let x2 = b.bounds.x;
            let y2 = b.bounds.y + 7;
            let middle = i32::midpoint(x1, x2);
            horizontal(target, x1, middle, y1);
            target.fill_rect(
                PixelRect::new(
                    middle,
                    y1.min(y2),
                    1,
                    u16::try_from(y1.abs_diff(y2) + 1).unwrap_or(0),
                ),
                INK_BLUE,
            );
            horizontal(target, middle, x2 - 1, y2);
            target.put(x2 - 2, y2 - 1, INK_BLUE);
            target.put(x2 - 2, y2 + 1, INK_BLUE);
        }
    }
    for card in &plan.cards {
        let node = &graph.nodes[&card.id];
        let border = if graph.selected.as_ref() == Some(&card.id) {
            SELECTION_RUNE
        } else if node.human_requests > 0 {
            WINE_LIGHT
        } else {
            INK_BLUE
        };
        target.fill_rect(card.bounds, border);
        target.fill_rect(
            PixelRect::new(
                card.bounds.x + 1,
                card.bounds.y + 1,
                card.bounds.width - 2,
                card.bounds.height - 2,
            ),
            PARCHMENT,
        );
        let origin = PixelPoint::new(card.bounds.x + 1, card.bounds.y + 2);
        match node.state {
            QuestState::Active => blit(roster::state_marker(ScenePose::Working), origin, target),
            QuestState::Completed => blit(roster::state_marker(ScenePose::Settled), origin, target),
            QuestState::Failed => {
                for i in 0..5 {
                    target.put(origin.x + i, origin.y + i, WINE_LIGHT);
                    target.put(origin.x + 4 - i, origin.y + i, WINE_LIGHT);
                }
            }
            QuestState::Closed | QuestState::Cancelled => {
                target.fill_rect(PixelRect::new(origin.x, origin.y + 2, 5, 1), INK_BLUE);
            }
            QuestState::Unknown | QuestState::Planned | QuestState::Ready => {
                target.fill_rect(PixelRect::new(origin.x, origin.y, 5, 5), AMBER_LIGHT);
                if node.state != QuestState::Ready {
                    target.fill_rect(PixelRect::new(origin.x + 1, origin.y + 1, 3, 3), PARCHMENT);
                }
            }
        }
    }
}
fn horizontal(target: &mut RgbBuffer, start: i32, end: i32, y: i32) {
    target.fill_rect(
        PixelRect::new(
            start.min(end),
            y,
            u16::try_from(start.abs_diff(end) + 1).unwrap_or(0),
            1,
        ),
        INK_BLUE,
    );
}
