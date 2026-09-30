//! Deterministic table composition in RGB coordinates (two pixels per text row).
use super::pixel::{PixelRect, PixelSize};
use crate::work::{QuestId, graph::CampaignGraph};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MapMode {
    Dependencies,
    Inventory,
    DetailsOnly,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuestPlacement {
    pub id: QuestId,
    pub ordinal: usize,
    pub bounds: PixelRect,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TablePlan {
    pub board: PixelRect,
    pub details: PixelRect,
    pub map: Option<PixelRect>,
    pub mode: MapMode,
    pub cards: Vec<QuestPlacement>,
}
impl TablePlan {
    pub fn project(graph: Option<&CampaignGraph>, viewport: PixelSize) -> Self {
        let width = viewport.width.saturating_sub(4).min(120);
        let rows = (viewport.height / 2).saturating_sub(2).min(38);
        let board = PixelRect::new(
            i32::from((viewport.width - width) / 2),
            i32::from((viewport.height / 2 - rows) / 2) * 2,
            width,
            rows * 2,
        );
        let body = PixelRect::new(
            board.x + 1,
            board.y + 8,
            width.saturating_sub(2),
            rows.saturating_sub(7) * 2,
        );
        let mut plan = Self {
            board,
            details: body,
            map: None,
            mode: MapMode::DetailsOnly,
            cards: Vec::new(),
        };
        if body.width < 106 || body.height < 16 {
            return plan;
        }
        let map_width = body.width * 3 / 5;
        let map = PixelRect::new(body.x, body.y, map_width, body.height);
        plan.map = Some(map);
        plan.details = PixelRect::new(
            body.x + i32::from(map_width) + 2,
            body.y,
            body.width - map_width - 2,
            body.height,
        );
        let Some(graph) = graph else {
            return plan;
        };
        if graph.nodes.is_empty() {
            return plan;
        }
        let layers = graph
            .nodes
            .values()
            .filter_map(|n| n.layer)
            .max()
            .unwrap_or(0)
            + 1;
        let max_rows = usize::from(map.height / 16);
        let dag_fits = !graph.has_cycle()
            && layers <= usize::from(map.width) / 18
            && (0..layers).all(|layer| {
                graph
                    .nodes
                    .values()
                    .filter(|n| n.layer == Some(layer))
                    .count()
                    <= max_rows
            })
            && graph.nodes.values().all(|n| n.layer.is_some());
        if dag_fits {
            plan.mode = MapMode::Dependencies;
            let columns = u16::try_from(layers).expect("fitting layer count");
            let card_width = ((map.width - (columns - 1) * 4) / columns).min(30);
            for layer in 0..layers {
                let nodes = graph
                    .nodes
                    .iter()
                    .enumerate()
                    .filter(|(_, (_, n))| n.layer == Some(layer))
                    .collect::<Vec<_>>();
                let count = u16::try_from(nodes.len()).expect("fitting layer height");
                let top = map.y + i32::from((map.height - (count * 16 - 2)) / 2);
                for (row, (index, (id, _))) in nodes.into_iter().enumerate() {
                    plan.cards.push(QuestPlacement {
                        id: id.clone(),
                        ordinal: index + 1,
                        bounds: PixelRect::new(
                            map.x + i32::try_from(layer).unwrap_or(0) * i32::from(card_width + 4),
                            top + i32::try_from(row).unwrap_or(0) * 16,
                            card_width,
                            14,
                        ),
                    });
                }
            }
        } else {
            plan.mode = MapMode::Inventory;
            plan.cards = inventory_cards(graph, map);
        }
        plan
    }
}

fn inventory_cards(graph: &CampaignGraph, map: PixelRect) -> Vec<QuestPlacement> {
    let mut cards = Vec::new();
    let max_rows = usize::from(map.height / 16);
    let columns = usize::from(map.width / 30).max(1);
    let capacity = columns * max_rows;
    let selected = graph
        .selected
        .as_ref()
        .and_then(|id| graph.nodes.keys().position(|q| q == id))
        .unwrap_or(0);
    let start = selected / capacity * capacity;
    for (index, id) in graph.nodes.keys().enumerate().skip(start).take(capacity) {
        let slot = index - start;
        cards.push(QuestPlacement {
            id: id.clone(),
            ordinal: index + 1,
            bounds: PixelRect::new(
                map.x + i32::try_from(slot % columns).unwrap_or(0) * 32,
                map.y + i32::try_from(slot / columns).unwrap_or(0) * 16,
                30,
                14,
            ),
        });
    }
    cards
}
