//! Pure geometry for the connected campaign Delves.

use std::collections::BTreeMap;

use ratatui::layout::Rect;

use crate::domain::{Agent, AgentKey, Campaign, WorkspaceId};

macro_rules! delve_variants {
    ($($variant:ident),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub enum DelveVariant { $($variant),+ }

        impl DelveVariant {
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];
        }
    };
}

delve_variants!(ForgottenLibrary, MossyUndercroft, OldWatchtower);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CampaignDelve {
    pub workspace_id: WorkspaceId,
    pub variant: DelveVariant,
    pub rect: Rect,
    pub chambers: Vec<ChamberAnchor>,
    pub adventurers: Vec<AgentKey>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChamberAnchor {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

/// Select an authored room variant from a stable workspace identity.
#[must_use]
pub fn variant_for_campaign(workspace_id: &WorkspaceId) -> DelveVariant {
    let mut input = Vec::with_capacity(27 + workspace_id.as_str().len());
    input.extend_from_slice(b"questmancer-delve-variant\0");
    input.extend_from_slice(workspace_id.as_str().as_bytes());
    match blake3::hash(&input).as_bytes()[0] % 3 {
        0 => DelveVariant::ForgottenLibrary,
        1 => DelveVariant::MossyUndercroft,
        _ => DelveVariant::OldWatchtower,
    }
}

/// Lay out one connected Delve per campaign chunk, preserving lexical order.
///
/// Chamber coordinates are absolute terminal coordinates. The selected workspace is
/// intentionally not allowed to perturb geometry: selection is a rendering concern,
/// keeping this model stable for snapshots and persistence.
#[must_use]
pub fn layout_delves(
    sites: &BTreeMap<WorkspaceId, Campaign>,
    agents: &BTreeMap<AgentKey, Agent>,
    area: Rect,
    _selected: Option<&WorkspaceId>,
) -> Vec<CampaignDelve> {
    if sites.is_empty() || area.width == 0 || area.height == 0 {
        return Vec::new();
    }

    let mut entries = Vec::new();
    for (workspace_id, site) in sites {
        let keys = site
            .party
            .iter()
            .filter(|key| agents.contains_key(*key))
            .cloned()
            .collect::<Vec<_>>();
        let chunk_capacity = match variant_for_campaign(workspace_id) {
            DelveVariant::ForgottenLibrary => 4,
            _ => 2,
        };
        let chunks = keys.len().max(1).div_ceil(chunk_capacity);
        for chunk in keys.chunks(chunk_capacity) {
            entries.push((workspace_id.clone(), chunk.to_vec()));
        }
        if keys.is_empty() && chunks == 1 {
            entries.push((workspace_id.clone(), Vec::new()));
        }
    }
    let count = u32::try_from(entries.len()).unwrap_or(u32::MAX);
    let columns = integer_sqrt_ceil(count).min(u32::from(area.width)).max(1);
    // Keep every Delve even when a compact surface cannot display every row;
    // zero-sized off-screen Delves simply contribute no chamber anchors.
    let rows = count.div_ceil(columns).max(1);
    let mut delves = Vec::with_capacity(entries.len());

    for (index, (workspace_id, agent_keys)) in entries.iter().enumerate() {
        let index = u32::try_from(index).unwrap_or(u32::MAX);
        let column = index % columns;
        let row = index / columns;
        if row >= rows {
            break;
        }
        let delve = partition(area, column, row, columns, rows);
        let variant = variant_for_campaign(workspace_id);
        let chambers = authored_chambers(variant, agent_keys.len(), delve);
        delves.push(CampaignDelve {
            workspace_id: workspace_id.clone(),
            variant,
            rect: delve,
            chambers,
            adventurers: agent_keys.clone(),
        });
    }
    delves
}

fn integer_sqrt_ceil(value: u32) -> u32 {
    if value == 0 {
        return 0;
    }
    let mut root = 1u32;
    while u64::from(root) * u64::from(root) < u64::from(value) {
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

fn authored_chambers(variant: DelveVariant, count: usize, delve: Rect) -> Vec<ChamberAnchor> {
    if count == 0 || delve.width == 0 || delve.height == 0 {
        return Vec::new();
    }
    let width = u32::from(delve.width);
    let height = u32::from(delve.height);
    let preferred_columns = match variant {
        DelveVariant::ForgottenLibrary => u32::try_from(count.min(4)).unwrap_or(4),
        DelveVariant::MossyUndercroft | DelveVariant::OldWatchtower => 2,
    };
    let columns = preferred_columns.min(width).max(1);
    let requested_rows = u32::try_from(count)
        .unwrap_or(u32::MAX)
        .div_ceil(columns)
        .max(1);
    let rows = requested_rows.min(height).max(1);
    let capacity = columns.saturating_mul(rows);
    let chamber_count = u32::try_from(count).unwrap_or(u32::MAX).min(capacity) as usize;
    // State theatre labels such as COUNSEL REQUESTED must remain readable when
    // the terminal gives a chamber enough room; compact areas still clamp to
    // the partition and preserve the legacy tiny-surface behavior.
    let chamber_width = (width / columns).clamp(1, 36);
    let chamber_height = (height / rows).clamp(1, 6);
    let mut chambers = Vec::with_capacity(chamber_count);
    for index in 0..chamber_count {
        let index = u32::try_from(index).unwrap_or(u32::MAX);
        let col = index % columns;
        let row = index / columns;
        let row = if variant == DelveVariant::OldWatchtower {
            rows - 1 - row
        } else {
            row
        };
        let cell_x = u32::from(delve.x) + width * col / columns;
        let cell_right = u32::from(delve.x) + width * (col + 1) / columns;
        let cell_y = u32::from(delve.y) + height * row / rows;
        let cell_bottom = u32::from(delve.y) + height * (row + 1) / rows;
        let cell_width = cell_right.saturating_sub(cell_x);
        let cell_height = cell_bottom.saturating_sub(cell_y);
        let (x, y) = if width <= 8 || height <= 8 {
            (cell_x, cell_y)
        } else {
            match variant {
                DelveVariant::ForgottenLibrary => (cell_x, cell_y.saturating_add(cell_height / 3)),
                DelveVariant::MossyUndercroft => (
                    cell_x.saturating_add(cell_width / 8),
                    cell_y.saturating_add(cell_height / 5),
                ),
                DelveVariant::OldWatchtower => (cell_x.saturating_add(cell_width / 10), cell_y),
            }
        };
        let x = x.min(cell_right.saturating_sub(chamber_width));
        let y = y.min(cell_bottom.saturating_sub(chamber_height));
        chambers.push(ChamberAnchor {
            x: u16::try_from(x).unwrap_or(u16::MAX),
            y: u16::try_from(y).unwrap_or(u16::MAX),
            width: u16::try_from(chamber_width).unwrap_or(u16::MAX),
            height: u16::try_from(chamber_height).unwrap_or(u16::MAX),
        });
    }
    chambers
}
