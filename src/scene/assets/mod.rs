pub mod adventurer;
pub mod archetypes;
pub(crate) mod barbarian_v2;
pub mod delve;
pub mod guild_hall;
pub mod librarian;
pub mod palette;
pub mod roster;

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

#[cfg(test)]
mod tests {
    use crate::domain::{AdventurerClass, Garb};

    use super::{
        archetypes, barbarian_v2, delve, guild_hall, librarian,
        palette::{self, OAK, SELECTION_RUNE, STONE},
        roster,
    };
    use crate::scene::pixel::Rgb;

    /// Perceptual-ish colour distance ("redmean" approximation). Large enough
    /// to separate an actor's body mass from a background fill at one glance.
    fn colour_distance(left: Rgb, right: Rgb) -> f64 {
        let mean_red = f64::midpoint(f64::from(left.r), f64::from(right.r));
        let dr = f64::from(left.r) - f64::from(right.r);
        let dg = f64::from(left.g) - f64::from(right.g);
        let db = f64::from(left.b) - f64::from(right.b);
        let weight_r = 2.0 + mean_red / 256.0;
        let weight_b = 2.0 + (255.0 - mean_red) / 256.0;
        (weight_r * dr * dr + 4.0 * dg * dg + weight_b * db * db).sqrt()
    }

    const MINIMUM_MASS_CONTRAST: f64 = 40.0;

    /// One hue, one meaning: the selection rune is the only thing in the
    /// world allowed to paint its exact colour. Any prop, adventurer or
    /// dungeon fixture that borrowed it would read as a selected adventurer.
    #[test]
    fn the_selection_rune_colour_is_reserved_for_selection() {
        let mut painted: Vec<(String, Rgb)> = Vec::new();
        let mut collect = |label: String, frame: &crate::scene::sprite::SpriteFrame| {
            for pixel in frame.pixels().iter().flatten() {
                painted.push((label.clone(), *pixel));
            }
        };
        for asset in guild_hall::GuildHallAsset::ALL {
            collect(format!("guild hall {asset:?}"), guild_hall::frame(*asset));
        }
        for asset in delve::DelveAsset::ALL {
            collect(format!("delve {asset:?}"), delve::frame(*asset));
        }
        collect("librarian world".to_owned(), librarian::world());
        collect("librarian ledger".to_owned(), librarian::ledger_portrait());
        for class in AdventurerClass::ALL {
            if let Some((frame, _)) = archetypes::world_master(*class) {
                collect(format!("{class:?} world master"), &frame);
            }
        }
        for family in roster::RosterFamily::ALL {
            let (frame, _) = roster::master(*family);
            collect(format!("roster {family:?}"), &frame);
        }

        for (label, colour) in painted {
            assert_ne!(
                colour, SELECTION_RUNE,
                "{label} paints the reserved selection rune colour"
            );
        }
    }

    /// The art direction requires actor palettes "protected from matching
    /// their immediate floor or wall". This proves it for every colour that
    /// can fill an actor's body mass: persona garb colours, each archetype's
    /// cloth base clusters (`c`/`C`) and the Barbarian's leather torso.
    #[test]
    fn garb_and_cloth_masses_contrast_with_world_materials() {
        let mut masses: Vec<(String, Rgb)> = Garb::ALL
            .iter()
            .map(|garb| {
                let colours = palette::adventurer_palette(
                    crate::domain::SkinTone::Sand,
                    crate::domain::HairTone::Espresso,
                    *garb,
                    AdventurerClass::Wizard,
                    crate::domain::AccentTone::Amber,
                );
                (format!("Garb::{garb:?}"), colours.cloth)
            })
            .collect();
        for class in AdventurerClass::ALL {
            let Some((_, class_palette)) = archetypes::world_master(*class) else {
                continue;
            };
            for entry in class_palette {
                if matches!(entry.key, 'c' | 'C')
                    && let Some(colour) = entry.colour
                {
                    masses.push((format!("{class:?} cloth '{}'", entry.key), colour));
                }
            }
        }
        for entry in barbarian_v2::palette() {
            if matches!(entry.key, 'L' | 'l')
                && let Some(colour) = entry.colour
            {
                masses.push((format!("Barbarian v2 leather '{}'", entry.key), colour));
            }
        }
        for family in roster::RosterFamily::ALL {
            let (_, family_palette) = roster::master(*family);
            for entry in family_palette {
                if matches!(entry.key, 'c' | 'C')
                    && let Some(colour) = entry.colour
                {
                    masses.push((format!("roster {family:?} cloth '{}'", entry.key), colour));
                }
            }
        }

        // Both worlds: the Hall's oak and stone, and the dungeon surfaces a
        // delving party actually stands on.
        for (label, colour) in masses {
            for (material, fill) in [
                ("OAK", OAK),
                ("STONE", STONE),
                ("delve FLOOR_DARK", delve::FLOOR_DARK),
                ("delve FLOOR_MID", delve::FLOOR_MID),
                ("delve STONE_MID", delve::STONE_MID),
                ("delve MOSS_DARK", delve::MOSS_DARK),
                ("delve MOSS_LIGHT", delve::MOSS_LIGHT),
            ] {
                let distance = colour_distance(colour, fill);
                assert!(
                    distance >= MINIMUM_MASS_CONTRAST,
                    "{label} ({colour:?}) sits {distance:.0} from {material} — actors would \
                     dissolve into the room (minimum {MINIMUM_MASS_CONTRAST})"
                );
            }
        }
    }
}
