pub mod adventurer;
pub mod archetypes;
pub(crate) mod barbarian_v2;
pub mod delve;
pub mod guild_hall;
pub mod palette;

use std::collections::HashMap;

use super::{pixel::Rgb, sprite::SpriteFrame};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IndexedPaletteEntry {
    pub key: char,
    pub colour: Option<Rgb>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssetError {
    EmptyRows,
    RaggedRows {
        row: usize,
        expected: usize,
        actual: usize,
    },
    DuplicatePaletteKey {
        key: char,
    },
    ReservedTransparencyKey,
    UnknownGlyph {
        glyph: char,
        row: usize,
        column: usize,
    },
    DimensionsTooLarge,
}

pub fn indexed_sprite(
    rows: &[&str],
    palette: &[IndexedPaletteEntry],
) -> Result<SpriteFrame, AssetError> {
    let Some(first) = rows.first() else {
        return Err(AssetError::EmptyRows);
    };
    let expected = first.chars().count();
    for (row, value) in rows.iter().enumerate().skip(1) {
        let actual = value.chars().count();
        if actual != expected {
            return Err(AssetError::RaggedRows {
                row,
                expected,
                actual,
            });
        }
    }

    let mut entries = HashMap::with_capacity(palette.len());
    for entry in palette {
        if entry.key == '.' {
            return Err(AssetError::ReservedTransparencyKey);
        }
        if entries.insert(entry.key, entry.colour).is_some() {
            return Err(AssetError::DuplicatePaletteKey { key: entry.key });
        }
    }

    let width = u16::try_from(expected).map_err(|_| AssetError::DimensionsTooLarge)?;
    let height = u16::try_from(rows.len()).map_err(|_| AssetError::DimensionsTooLarge)?;
    let mut pixels = Vec::with_capacity(expected.saturating_mul(rows.len()));
    for (row, value) in rows.iter().enumerate() {
        for (column, glyph) in value.chars().enumerate() {
            if glyph == '.' {
                pixels.push(None);
            } else if let Some(colour) = entries.get(&glyph) {
                pixels.push(*colour);
            } else {
                return Err(AssetError::UnknownGlyph { glyph, row, column });
            }
        }
    }
    Ok(SpriteFrame::from_pixels(width, height, pixels))
}
