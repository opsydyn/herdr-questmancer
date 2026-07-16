use std::time::Duration;

use ratatui::{
    Frame,
    buffer::{Buffer, CellWidth},
    layout::Rect,
    style::Style,
};

use crate::{
    app::{CharacterSet, Model, Motion},
    domain::{Timestamp, WorkspaceId},
    ui::{
        EffectCells,
        pixel::{ColorRole, Palette},
    },
};

const OUTBREAK_FPS: u8 = 4;
const ASCII_GOBLIN: [&str; 2] = ["/{g}\\", " /|\\ "];
const UNICODE_GOBLIN: [&str; 2] = ["╭g╮", "╰┬╯"];

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GoblinSighting {
    ChestEyes,
    ChronicleHand,
    RaftersScroll,
    StolenBiscuit,
}

#[must_use]
pub fn sighting_for_campaign(workspace_id: &WorkspaceId) -> Option<GoblinSighting> {
    let digest = labelled_campaign_hash("questmancer-goblin-sighting", workspace_id);
    (digest[0] == 0).then(|| match digest[1] % 4 {
        0 => GoblinSighting::ChestEyes,
        1 => GoblinSighting::ChronicleHand,
        2 => GoblinSighting::RaftersScroll,
        _ => GoblinSighting::StolenBiscuit,
    })
}

fn labelled_campaign_hash(label: &str, workspace_id: &WorkspaceId) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(label.as_bytes());
    hasher.update(&[0]);
    hasher.update(workspace_id.as_str().as_bytes());
    *hasher.finalize().as_bytes()
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GoblinState {
    released_at: Option<Timestamp>,
}

impl GoblinState {
    pub const OUTBREAK_DURATION: Duration = Duration::from_secs(3);

    pub const fn release(&mut self, now: Timestamp) {
        self.released_at = Some(now);
    }

    #[must_use]
    pub fn is_visible(self, now: Timestamp) -> bool {
        self.released_at
            .is_some_and(|start| now >= start && start.elapsed_until(now) < Self::OUTBREAK_DURATION)
    }

    pub(crate) fn next_visible_frame_in(self, now: Timestamp, motion: Motion) -> Option<Duration> {
        let start = self.released_at?;
        if now < start {
            return None;
        }
        let elapsed = start.elapsed_until(now);
        if elapsed >= Self::OUTBREAK_DURATION {
            return None;
        }
        let remaining = Self::OUTBREAK_DURATION.saturating_sub(elapsed);
        match motion {
            Motion::Full => Some(next_step_delay(elapsed, OUTBREAK_FPS).min(remaining)),
            Motion::Reduced | Motion::None => Some(remaining),
        }
    }

    fn animation_frame(self, now: Timestamp) -> usize {
        self.released_at.map_or(0, |start| {
            let elapsed = start.elapsed_until(now).as_millis();
            usize::try_from(elapsed * u128::from(OUTBREAK_FPS) / 1_000).unwrap_or(usize::MAX)
        })
    }
}

pub(crate) fn render(frame: &mut Frame<'_>, area: Rect, model: &Model) -> EffectCells {
    if area.width == 0 || area.height == 0 || model.preferences().motion == Motion::None {
        return EffectCells::default();
    }

    let outbreak = model.goblins().is_visible(model.now());
    let sighting = (!outbreak)
        .then(|| {
            model
                .domain()
                .campaigns
                .keys()
                .find_map(sighting_for_campaign)
        })
        .flatten();
    if !outbreak && sighting.is_none() {
        return EffectCells::default();
    }

    let pattern = match model.preferences().character_set {
        CharacterSet::Ascii => &ASCII_GOBLIN,
        CharacterSet::Unicode => &UNICODE_GOBLIN,
    };
    let frame_index = if outbreak && model.preferences().motion == Motion::Full {
        model.goblins().animation_frame(model.now())
    } else {
        sighting.map_or(0, |kind| kind as usize)
    };
    let count = if outbreak && model.preferences().motion == Motion::Full {
        3
    } else {
        1
    };
    let style =
        Style::new().fg(Palette::from(model.preferences().color_mode).resolve(ColorRole::Goblin));
    let baseline = outbreak.then(|| frame.buffer_mut().clone());
    let mut offset = frame_index.saturating_mul(17);
    let mut rendered = false;
    for _ in 0..count {
        let Some(origin) = blank_origin(frame.buffer_mut(), area, pattern, offset) else {
            break;
        };
        paint(frame.buffer_mut(), origin, pattern, style);
        rendered = true;
        offset = offset.saturating_add(31);
    }
    if !rendered {
        return EffectCells::default();
    }
    baseline.map_or_else(EffectCells::default, |before| {
        EffectCells::changed_between(&before, frame.buffer_mut(), area)
    })
}

fn blank_origin(
    buffer: &Buffer,
    area: Rect,
    pattern: &[&str],
    offset: usize,
) -> Option<(u16, u16)> {
    let width = pattern.iter().map(|line| line.chars().count()).max()?;
    let width = u16::try_from(width).ok()?;
    let height = u16::try_from(pattern.len()).ok()?;
    if width == 0 || height == 0 || area.width < width || area.height < height {
        return None;
    }

    let x_count = area.width - width + 1;
    let y_count = area.height - height + 1;
    let candidate_count = usize::from(x_count) * usize::from(y_count);
    let occupied = occupied_cells(buffer, area);
    for step in 0..candidate_count {
        let candidate = (offset + step) % candidate_count;
        let x = area.x + u16::try_from(candidate % usize::from(x_count)).ok()?;
        let y = area.y + u16::try_from(candidate / usize::from(x_count)).ok()?;
        if pattern.iter().enumerate().all(|(row, line)| {
            line.chars().enumerate().all(|(column, glyph)| {
                if glyph == ' ' {
                    return true;
                }
                let cell_x = x + u16::try_from(column).unwrap_or(u16::MAX);
                let cell_y = y + u16::try_from(row).unwrap_or(u16::MAX);
                !occupied[cell_index(area, cell_x, cell_y)]
            })
        }) {
            return Some((x, y));
        }
    }
    None
}

fn occupied_cells(buffer: &Buffer, area: Rect) -> Vec<bool> {
    let mut occupied = vec![false; usize::from(area.width) * usize::from(area.height)];
    for y in area.y..area.bottom() {
        for x in buffer.area.x..area.right() {
            let Some(cell) = buffer.cell((x, y)) else {
                continue;
            };
            let span = cell.cell_width();
            if cell.symbol() == " " && span <= 1 {
                continue;
            }
            let start = x.max(area.x);
            let end = x.saturating_add(span).min(area.right());
            for covered_x in start..end {
                occupied[cell_index(area, covered_x, y)] = true;
            }
        }
    }
    occupied
}

fn cell_index(area: Rect, x: u16, y: u16) -> usize {
    usize::from(y - area.y) * usize::from(area.width) + usize::from(x - area.x)
}

fn paint(buffer: &mut Buffer, origin: (u16, u16), pattern: &[&str], style: Style) {
    for (row, line) in pattern.iter().enumerate() {
        for (column, glyph) in line.chars().enumerate() {
            if glyph == ' ' {
                continue;
            }
            let Some(cell) = buffer.cell_mut((
                origin.0 + u16::try_from(column).unwrap_or(u16::MAX),
                origin.1 + u16::try_from(row).unwrap_or(u16::MAX),
            )) else {
                continue;
            };
            cell.set_char(glyph).set_style(style);
        }
    }
}

fn next_step_delay(elapsed: Duration, fps: u8) -> Duration {
    let elapsed_millis = elapsed.as_millis();
    let completed_steps = elapsed_millis * u128::from(fps) / 1_000;
    let next_boundary = ((completed_steps + 1) * 1_000).div_ceil(u128::from(fps));
    let delay = next_boundary.saturating_sub(elapsed_millis).max(1);
    Duration::from_millis(u64::try_from(delay).unwrap_or(u64::MAX))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_origin_skips_cells_covered_by_a_wide_grapheme() {
        let area = Rect::new(0, 0, 5, 1);
        let mut buffer = Buffer::empty(area);
        buffer.set_string(0, 0, "界   ", Style::new());

        assert_eq!(blank_origin(&buffer, area, &["g"], 1), Some((2, 0)));
    }
}
