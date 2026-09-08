pub mod adventurer;
pub mod archetypes;
pub mod cat;
pub mod delve;
pub mod guild_hall;
pub mod keepsakes;
pub mod librarian;
pub mod palette;
pub(crate) mod rituals;
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
        archetypes, delve, guild_hall, librarian,
        palette::{self, OAK, SELECTION_RUNE, STONE},
        rituals, roster,
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
        for class in [
            AdventurerClass::Wizard,
            AdventurerClass::Ranger,
            AdventurerClass::Barbarian,
        ] {
            for pose in [
                crate::scene::stage::ScenePose::Working,
                crate::scene::stage::ScenePose::SeekingCounsel,
                crate::scene::stage::ScenePose::ReturningWithSpoils,
                crate::scene::stage::ScenePose::Settled,
                crate::scene::stage::ScenePose::Resting,
                crate::scene::stage::ScenePose::Unknown,
            ] {
                for index in 0..2 {
                    let (frame, _) =
                        rituals::world_master(class, pose, index).expect("pilot class");
                    collect(format!("ritual {class:?} {pose:?} {index}"), &frame);
                }
            }
            let (frame, _) = rituals::roster_master(class).expect("pilot roster");
            collect(format!("ritual roster {class:?}"), &frame);
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

    /// A dungeon whose floor matches its walls is not a dungeon, it is a
    /// texture. The Delve's architecture defines seven named regions and seven
    /// doorways, and every one of them is drawn purely as a change of surface
    /// — there is no outline, no shadow, nothing else marking where a wall
    /// stops and a room begins. So the layout is visible exactly as far as
    /// these colours differ, and no further.
    ///
    /// They did not differ. `STONE_MID` sat 21 from `FLOOR_MID` and
    /// `STONE_DARK` 17 from `FLOOR_DARK`, roughly half the distance this same
    /// module already demands between an actor and the ground it stands on.
    /// The labyrinth was authored, walkability-masked, tested for reachability
    /// — and invisible. Props read as stuck to a wall because no floor plane
    /// was perceptible for them to rest on.
    ///
    /// Note that `MOSS_DARK` deliberately appears on both surfaces and so
    /// cannot carry this distinction; moss grows on walls and floors alike.
    /// The separation has to come from the stone and the ground themselves.
    #[test]
    fn dungeon_floors_read_as_floors_against_their_walls() {
        for (floor_label, floor) in [
            ("FLOOR_MID", delve::FLOOR_MID),
            ("FLOOR_DARK", delve::FLOOR_DARK),
        ] {
            for (wall_label, wall) in [
                ("STONE_MID", delve::STONE_MID),
                ("STONE_DARK", delve::STONE_DARK),
                ("STONE_LIGHT", delve::STONE_LIGHT),
            ] {
                let distance = colour_distance(floor, wall);
                assert!(
                    distance >= MINIMUM_MASS_CONTRAST,
                    "{floor_label} ({floor:?}) sits {distance:.0} from {wall_label} \
                     ({wall:?}) — the delve's rooms and corridors would dissolve into \
                     one flat field (minimum {MINIMUM_MASS_CONTRAST})"
                );
            }
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
        for class in [
            AdventurerClass::Wizard,
            AdventurerClass::Ranger,
            AdventurerClass::Barbarian,
        ] {
            let (_, class_palette) =
                rituals::world_master(class, crate::scene::stage::ScenePose::Working, 0)
                    .expect("pilot class");
            for entry in class_palette {
                if matches!(entry.key, 'c' | 'C')
                    && let Some(colour) = entry.colour
                {
                    masses.push((format!("ritual {class:?} cloth '{}'", entry.key), colour));
                }
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
