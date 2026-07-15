//! Pure geometry for the connected cybercafé scene.

use std::collections::BTreeMap;

use ratatui::layout::Rect;

use crate::domain::{Agent, AgentKey, Site, WorkspaceId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BayVariant {
    WallRow,
    CornerBooth,
    BackRoomLab,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CafeBay {
    pub workspace_id: WorkspaceId,
    pub variant: BayVariant,
    pub rect: Rect,
    pub seats: Vec<SeatAnchor>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SeatAnchor {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

/// Select an authored room variant from a stable workspace identity.
#[must_use]
pub fn variant_for_workspace(workspace_id: &WorkspaceId) -> BayVariant {
    let mut input = Vec::with_capacity(12 + workspace_id.as_str().len());
    input.extend_from_slice(b"cafe-variant\0");
    input.extend_from_slice(workspace_id.as_str().as_bytes());
    match blake3::hash(&input).as_bytes()[0] % 3 {
        0 => BayVariant::WallRow,
        1 => BayVariant::CornerBooth,
        _ => BayVariant::BackRoomLab,
    }
}

/// Lay out one connected bay per workspace, preserving the map's lexical order.
///
/// Seat coordinates are absolute terminal coordinates. The selected workspace is
/// intentionally not allowed to perturb geometry: selection is a rendering concern,
/// keeping this model stable for snapshots and persistence.
#[must_use]
pub fn layout_bays(
    sites: &BTreeMap<WorkspaceId, Site>,
    agents: &BTreeMap<AgentKey, Agent>,
    area: Rect,
    _selected: Option<&WorkspaceId>,
) -> Vec<CafeBay> {
    if sites.is_empty() || area.width == 0 || area.height == 0 {
        return Vec::new();
    }

    let count = u32::try_from(sites.len()).unwrap_or(u32::MAX);
    let columns = integer_sqrt_ceil(count).min(u32::from(area.width)).max(1);
    // Keep one bay per workspace even when a compact surface cannot display
    // every row; zero-sized off-screen bays simply contribute no seat anchors.
    let rows = count.div_ceil(columns).max(1);
    let mut bays = Vec::with_capacity(sites.len());

    for (index, (workspace_id, site)) in sites.iter().enumerate() {
        let index = u32::try_from(index).unwrap_or(u32::MAX);
        let column = index % columns;
        let row = index / columns;
        if row >= rows {
            break;
        }
        let bay = partition(area, column, row, columns, rows);
        let count = site
            .agents
            .iter()
            .filter(|key| agents.contains_key(*key))
            .count();
        let variant = variant_for_workspace(workspace_id);
        let seats = authored_seats(variant, count, bay);
        bays.push(CafeBay {
            workspace_id: workspace_id.clone(),
            variant,
            rect: bay,
            seats,
        });
    }
    bays
}

fn integer_sqrt_ceil(value: u32) -> u32 {
    if value == 0 {
        return 0;
    }
    let mut root = 1;
    while root < value / root {
        root += 1;
    }
    root
}

fn partition(area: Rect, column: u32, row: u32, columns: u32, rows: u32) -> Rect {
    let x0 = u32::from(area.x) + u32::from(area.width) * column / columns;
    let x1 = u32::from(area.x) + u32::from(area.width) * (column + 1) / columns;
    let y0 = u32::from(area.y) + u32::from(area.height) * row / rows;
    let y1 = u32::from(area.y) + u32::from(area.height) * (row + 1) / rows;
    Rect::new(
        u16::try_from(x0).unwrap_or(u16::MAX),
        u16::try_from(y0).unwrap_or(u16::MAX),
        u16::try_from(x1 - x0).unwrap_or(u16::MAX),
        u16::try_from(y1 - y0).unwrap_or(u16::MAX),
    )
}

fn authored_seats(variant: BayVariant, count: usize, bay: Rect) -> Vec<SeatAnchor> {
    if count == 0 || bay.width == 0 || bay.height == 0 {
        return Vec::new();
    }
    let width = u32::from(bay.width);
    let height = u32::from(bay.height);
    let preferred_columns = match variant {
        BayVariant::WallRow => u32::try_from(count.min(4)).unwrap_or(4),
        BayVariant::CornerBooth | BayVariant::BackRoomLab => 2,
    };
    let columns = preferred_columns.min(width).max(1);
    let requested_rows = u32::try_from(count)
        .unwrap_or(u32::MAX)
        .div_ceil(columns)
        .max(1);
    let rows = requested_rows.min(height).max(1);
    let capacity = columns.saturating_mul(rows);
    let seat_count = u32::try_from(count).unwrap_or(u32::MAX).min(capacity) as usize;
    let seat_width = (width / columns).clamp(1, 14);
    let seat_height = (height / rows).clamp(1, 6);
    let mut seats = Vec::with_capacity(seat_count);
    for index in 0..seat_count {
        let index = u32::try_from(index).unwrap_or(u32::MAX);
        let col = index % columns;
        let row = index / columns;
        let row = if variant == BayVariant::BackRoomLab {
            rows - 1 - row
        } else {
            row
        };
        let base_x = u32::from(bay.x) + width * col / columns;
        let base_y = u32::from(bay.y) + height * row / rows;
        let (x, y) = if width <= 8 || height <= 8 {
            (base_x, base_y)
        } else {
            match variant {
                BayVariant::WallRow => (base_x, base_y.saturating_add(height / 3)),
                BayVariant::CornerBooth => (
                    base_x.saturating_add(width / 8),
                    base_y.saturating_add(height / 5),
                ),
                BayVariant::BackRoomLab => (
                    base_x.saturating_add(width / 10),
                    base_y.saturating_sub(height / 8),
                ),
            }
        };
        let x = x.min(u32::from(bay.right()).saturating_sub(seat_width));
        let y = y.min(u32::from(bay.bottom()).saturating_sub(seat_height));
        seats.push(SeatAnchor {
            x: u16::try_from(x).unwrap_or(u16::MAX),
            y: u16::try_from(y).unwrap_or(u16::MAX),
            width: u16::try_from(seat_width).unwrap_or(u16::MAX),
            height: u16::try_from(seat_height).unwrap_or(u16::MAX),
        });
    }
    seats
}
