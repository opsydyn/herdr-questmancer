//! Presentation-only campaign identity. Crests are not unique identifiers and
//! remain stable only for the lifetime of Herdr's workspace key.
use crate::domain::WorkspaceId;

use super::{
    pixel::{PixelPoint, Rgb},
    snapshot::SceneSnapshot,
    sprite::SpriteFrame,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrestCharge {
    Lozenge,
    Cross,
    Saltire,
    Bars,
}

impl CrestCharge {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Lozenge => "lozenge",
            Self::Cross => "cross",
            Self::Saltire => "saltire",
            Self::Bars => "bars",
        }
    }

    #[must_use]
    pub const fn symbol(self, ascii: bool) -> &'static str {
        match (self, ascii) {
            (Self::Lozenge, false) => "◆",
            (Self::Lozenge, true) => "<>",
            (Self::Cross, _) => "+",
            (Self::Saltire, false) => "×",
            (Self::Saltire, true) => "X",
            (Self::Bars, _) => "=",
        }
    }

    const fn marks(self, x: i32, y: i32) -> bool {
        match self {
            Self::Lozenge => (x - 2).abs() + (y - 2).abs() <= 2,
            Self::Cross => x == 2 || y == 2,
            Self::Saltire => x == y || x + y == 4,
            Self::Bars => y == 1 || y == 3,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrestField {
    Azure,
    Moss,
    Wine,
    Umber,
}

impl CrestField {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Azure => "Azure",
            Self::Moss => "Moss",
            Self::Wine => "Wine",
            Self::Umber => "Umber",
        }
    }

    #[must_use]
    pub const fn colour(self) -> Rgb {
        match self {
            Self::Azure => Rgb::new(45, 75, 110),
            Self::Moss => Rgb::new(44, 77, 48),
            Self::Wine => Rgb::new(103, 42, 64),
            Self::Umber => Rgb::new(91, 57, 30),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CampaignCrest {
    pub field: CrestField,
    pub charge: CrestCharge,
}

impl CampaignCrest {
    #[must_use]
    pub fn for_workspace(workspace: &WorkspaceId) -> Self {
        let mut hash = blake3::Hasher::new();
        hash.update(b"questmancer.campaign-crest.v1\0");
        hash.update(workspace.as_str().as_bytes());
        let digest = hash.finalize();
        let bytes = digest.as_bytes();
        Self {
            field: [
                CrestField::Azure,
                CrestField::Moss,
                CrestField::Wine,
                CrestField::Umber,
            ][usize::from(bytes[0] % 4)],
            charge: [
                CrestCharge::Lozenge,
                CrestCharge::Cross,
                CrestCharge::Saltire,
                CrestCharge::Bars,
            ][usize::from(bytes[1] % 4)],
        }
    }

    /// A seven-by-eight pennant, authored at its native world scale.
    #[must_use]
    pub fn frame(self) -> SpriteFrame {
        let edge = Rgb::new(31, 24, 26);
        let thread = Rgb::new(230, 207, 154);
        let mut pixels = vec![None; 7 * 8];
        for y in 0..8_usize {
            for x in 0..7_usize {
                if y == 7 && (x == 0 || x == 6) {
                    continue;
                }
                let colour = if y == 0 || x == 0 || x == 6 || y == 7 {
                    edge
                } else if y <= 5
                    && self.charge.marks(
                        i32::try_from(x - 1).expect("five-pixel charge"),
                        i32::try_from(y - 1).expect("five-pixel charge"),
                    )
                {
                    thread
                } else {
                    self.field.colour()
                };
                pixels[y * 7 + x] = Some(colour);
            }
        }
        SpriteFrame::from_pixels(7, 8, pixels)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CampaignPennant {
    pub workspace: WorkspaceId,
    pub origin: PixelPoint,
}

/// Uses the Hall's existing alternating table assignment. If a table cannot
/// hold every campaign's pennant, none are shown for that table. Actor targets
/// and the selected campaign's card remain unchanged.
#[must_use]
pub fn table_pennants(snapshot: &SceneSnapshot) -> Vec<CampaignPennant> {
    let mut result = Vec::new();
    for side in 0..2 {
        let campaigns: Vec<_> = snapshot.campaigns.iter().skip(side).step_by(2).collect();
        if campaigns.len() > 4 {
            continue;
        }
        for (slot, campaign) in campaigns.into_iter().enumerate() {
            result.push(CampaignPennant {
                workspace: campaign.workspace_id.clone(),
                origin: PixelPoint::new(
                    (if side == 0 { 40 } else { 82 }) + i32::try_from(slot).unwrap_or(0) * 8,
                    80,
                ),
            });
        }
    }
    result
}
