use std::sync::OnceLock;

use crate::domain::AdventurerPersona;

use super::{
    IndexedPaletteEntry, archetypes, barbarian_v2, indexed_sprite,
    palette::{AdventurerPalette, adventurer_palette},
    roster,
};
use crate::{
    domain::AdventurerClass,
    scene::{pixel::Rgb, sprite::SpriteFrame, stage::ScenePose},
};

const DRUID_ACCENT: Rgb = Rgb::new(85, 174, 206);

// The first production master. This is intentionally a complete authored
// sprite rather than a palette swap of another class master: the hood,
// antlers, beard, foliage and living staff are all readable at world scale.
const DRUID_PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(22, 20, 26)),
    },
    IndexedPaletteEntry {
        key: 'c',
        colour: Some(Rgb::new(35, 68, 38)),
    },
    IndexedPaletteEntry {
        key: 'C',
        colour: Some(Rgb::new(65, 116, 55)),
    },
    IndexedPaletteEntry {
        key: 'v',
        colour: Some(Rgb::new(117, 162, 69)),
    },
    IndexedPaletteEntry {
        key: 'k',
        colour: Some(Rgb::new(134, 77, 51)),
    },
    IndexedPaletteEntry {
        key: 'K',
        colour: Some(Rgb::new(225, 153, 99)),
    },
    IndexedPaletteEntry {
        key: 'h',
        colour: Some(Rgb::new(255, 218, 151)),
    },
    IndexedPaletteEntry {
        key: 'w',
        colour: Some(Rgb::new(125, 118, 99)),
    },
    IndexedPaletteEntry {
        key: 'W',
        colour: Some(Rgb::new(219, 211, 183)),
    },
    IndexedPaletteEntry {
        key: 'd',
        colour: Some(Rgb::new(84, 51, 32)),
    },
    IndexedPaletteEntry {
        key: 'D',
        colour: Some(Rgb::new(158, 100, 53)),
    },
    IndexedPaletteEntry {
        key: 'm',
        colour: Some(Rgb::new(110, 86, 52)),
    },
    IndexedPaletteEntry {
        key: 'M',
        colour: Some(Rgb::new(191, 163, 104)),
    },
    IndexedPaletteEntry {
        key: 'l',
        colour: Some(Rgb::new(217, 179, 78)),
    },
    IndexedPaletteEntry {
        key: 'e',
        colour: Some(Rgb::new(105, 232, 150)),
    },
    IndexedPaletteEntry {
        key: 'a',
        colour: Some(DRUID_ACCENT),
    },
];

const DRUID_WORLD: &[&str] = &[
    "................",
    "....m......m....",
    ".....m....m.....",
    "......ocCCo.....",
    ".....ocCCCco....",
    "....ocCvvvCco...",
    "....ocCKhKCCo...",
    "....ocCKoKCCo.d.",
    "....ocCKKKCCo.d.",
    "...owWWWWWWwo.d.",
    "...owWwWWwWwo.d.",
    "...ocCCCCCCco.d.",
    "...ocCCaCCCco.d.",
    "...ocCCllCCCco..",
    "...ocCCCCCCco...",
    "...oddddddddo...",
    "...odDddddDdo...",
    "...odDddddDdo...",
    "...oddddddddo...",
    "...oddo..oddo...",
    "...ooo....ooo...",
    "................",
    "................",
    "................",
];

const DRUID_PORTRAIT: &[&str] = &[
    "........................",
    "....m..............m....",
    ".....m............m.....",
    "......M..........M......",
    ".......occccccco........",
    "......ocCCCCCCCCco......",
    ".....ocCCvvvvvvCCco.....",
    ".....ocCCCKhhKCCCco.....",
    ".....ocCCCKooKCCCco..d..",
    ".....ocCCCKKKKCCCco.dD..",
    ".....ocCCCCCCCCCCco.dD..",
    ".....oowWWWWWWWWwoo.dD..",
    ".....owWwwWWWWwwWwo.dD..",
    ".....owWWWWWWWWWWwo.dD..",
    ".....ocCCCccccCCCco.dD..",
    "....ocCCCCCCaCCCCCco.d..",
    "....ocCCCCClllCCCCCco...",
    "....ocCCCCCCCCCCCCCco...",
    "....ocCCCCC.CCCCCCCco...",
    "....ocCCCCC..CCCCCco....",
    "....ocCCCCC...CCCCco....",
    "....oddddddddddddddo....",
    "....odDdddddddddddDo....",
    "....odDddddeedddddDo....",
    "....odDdddddddddddDo....",
    "....oddddddddddddddo....",
    ".....oddddo..oddddo.....",
    ".....odddo....odddo.....",
    "....ooooo......oooo.....",
    "........................",
    "........................",
    "........................",
];

#[must_use]
pub fn druid_world_frame() -> SpriteFrame {
    static FRAME: OnceLock<SpriteFrame> = OnceLock::new();
    FRAME
        .get_or_init(|| {
            indexed_sprite(DRUID_WORLD, DRUID_PALETTE).expect("Druid world master is valid")
        })
        .clone()
}

#[must_use]
pub fn druid_portrait_frame() -> SpriteFrame {
    static FRAME: OnceLock<SpriteFrame> = OnceLock::new();
    FRAME
        .get_or_init(|| {
            indexed_sprite(DRUID_PORTRAIT, DRUID_PALETTE).expect("Druid portrait master is valid")
        })
        .clone()
}

#[must_use]
pub fn adventurer_portrait_frame(persona: &AdventurerPersona) -> Option<SpriteFrame> {
    if persona.class == AdventurerClass::Druid {
        Some(druid_portrait_frame())
    } else {
        archetypes::portrait_frame(persona.class)
    }
}

/// Roles a persona may recolour inside an authored master. Class-owned
/// clusters (cloth, metal, gear, eyes and gems) stay authored so the class
/// silhouette and material read never change.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PersonaRole {
    SkinShadow,
    SkinBase,
    SkinHighlight,
    HairShadow,
    HairBase,
    Garb,
    Accent,
}

/// The shared archetype grammar from the art-direction doc.
const fn standard_role(key: char) -> Option<PersonaRole> {
    match key {
        'k' => Some(PersonaRole::SkinShadow),
        'K' => Some(PersonaRole::SkinBase),
        'h' => Some(PersonaRole::SkinHighlight),
        'r' => Some(PersonaRole::HairShadow),
        'R' => Some(PersonaRole::HairBase),
        'l' => Some(PersonaRole::Garb),
        'a' => Some(PersonaRole::Accent),
        _ => None,
    }
}

/// The Barbarian v2 masters predate the shared grammar: `S` is skin,
/// `H`/`h` are hair and `L`/`l` are class-owned leather.
const fn barbarian_role(key: char) -> Option<PersonaRole> {
    match key {
        'S' => Some(PersonaRole::SkinBase),
        'H' => Some(PersonaRole::HairShadow),
        'h' => Some(PersonaRole::HairBase),
        'a' => Some(PersonaRole::Accent),
        _ => None,
    }
}

fn role_colour(role: PersonaRole, colours: &AdventurerPalette) -> Rgb {
    match role {
        PersonaRole::SkinShadow => mix(colours.skin, Rgb::BLACK, 35),
        PersonaRole::SkinBase => colours.skin,
        PersonaRole::SkinHighlight => mix(colours.skin, Rgb::new(255, 255, 255), 30),
        PersonaRole::HairShadow => mix(colours.hair, Rgb::BLACK, 30),
        PersonaRole::HairBase => colours.hair,
        // Garb reads in the trim band rather than the body mass. Recolouring
        // the cloth itself collided with world materials in a different place
        // at every tint strength tried: a Bard in Vestments landed on Hall
        // stone, a Cleric in Armour on dungeon floor, a Ranger in Armour on
        // dungeon moss. Garb colours are already proven against both worlds on
        // their own, so using them directly is safe where a blend was not.
        PersonaRole::Garb => colours.cloth,
        PersonaRole::Accent => colours.accent,
    }
}

fn mix(base: Rgb, tint: Rgb, amount: u16) -> Rgb {
    let keep = 100 - amount.min(100);
    let channel = |base: u8, tint: u8| {
        u8::try_from((u16::from(base) * keep + u16::from(tint) * amount) / 100)
            .expect("weighted channel remains within u8")
    };
    Rgb::new(
        channel(base.r, tint.r),
        channel(base.g, tint.g),
        channel(base.b, tint.b),
    )
}

/// Recolours a master's persona roles while leaving every class-owned colour
/// and the transparency mask untouched.
fn personalise(
    frame: &SpriteFrame,
    source_palette: &[IndexedPaletteEntry],
    role_for: fn(char) -> Option<PersonaRole>,
    colours: &AdventurerPalette,
) -> SpriteFrame {
    let substitutions = source_palette
        .iter()
        .filter_map(|entry| {
            let role = role_for(entry.key)?;
            let from = entry.colour?;
            Some((from, role_colour(role, colours)))
        })
        .collect::<Vec<_>>();

    SpriteFrame::from_pixels(
        frame.size().width,
        frame.size().height,
        frame
            .pixels()
            .iter()
            .map(|pixel| {
                pixel.map(|colour| {
                    substitutions
                        .iter()
                        .find(|(from, _)| *from == colour)
                        .map_or(colour, |(_, to)| *to)
                })
            })
            .collect(),
    )
}

/// An authored pose decoration, drawn over a class's own master so the sprite
/// itself carries state instead of leaving it all to labels and markers.
///
/// Glyphs resolve against the class's palette, so a Wizard's carried chest is
/// wizard leather and a Ranger's is ranger leather, and class identity
/// survives the pose. Decorations sit in the torso and leg zones: class gear
/// lives on the left and right edges, so those are the only regions free on
/// every master.
struct PoseArt {
    rows: &'static [&'static str],
    x: i32,
    y: i32,
}

// Spoils held in front of the chest. `ReturningWithSpoils` previously had only
// a three-second effect flash, so an adventurer that had already returned
// looked exactly like one still working.
#[rustfmt::skip]
const SPOILS_CHEST: PoseArt = PoseArt {
    rows: &[
        "oooooo",
        "olDDlo",
        "oDllDo",
        "oooooo",
    ],
    x: 5,
    y: 14,
};

// A wider, lower stance. The art direction forbids drifting feet, so resting
// changes the legs rather than sliding the whole body down.
#[rustfmt::skip]
const RESTING_STANCE: PoseArt = PoseArt {
    rows: &[
        "oddddddddo",
        "odDddddDdo",
        "oddddddddo",
        "ooo....ooo",
        ".oo....oo.",
    ],
    x: 3,
    y: 18,
};

const fn pose_decoration(pose: ScenePose) -> Option<&'static PoseArt> {
    match pose {
        ScenePose::ReturningWithSpoils => Some(&SPOILS_CHEST),
        ScenePose::Resting => Some(&RESTING_STANCE),
        // Working and Settled are the resting state of the art itself, and a
        // blocked adventurer already carries the authored counsel marker, so
        // adding a second signal there would only compete with it.
        ScenePose::Working
        | ScenePose::Settled
        | ScenePose::SeekingCounsel
        | ScenePose::Unknown => None,
    }
}

fn palette_colour(palette: &[IndexedPaletteEntry], key: char) -> Option<Rgb> {
    palette
        .iter()
        .find(|entry| entry.key == key)
        .and_then(|entry| entry.colour)
}

/// Paints a pose decoration over a personalised master.
fn apply_pose(
    frame: &SpriteFrame,
    pose: ScenePose,
    palette: &[IndexedPaletteEntry],
    colours: &AdventurerPalette,
) -> SpriteFrame {
    let Some(art) = pose_decoration(pose) else {
        return frame.clone();
    };
    let width = i32::from(frame.size().width);
    let height = i32::from(frame.size().height);
    let mut pixels = frame.pixels().to_vec();

    for (row, line) in art.rows.iter().enumerate() {
        for (column, glyph) in line.chars().enumerate() {
            if glyph == '.' {
                continue;
            }
            let colour = match glyph {
                'K' => Some(colours.skin),
                key => palette_colour(palette, key)
                    // A master without leather or trim falls back to its
                    // outline rather than dropping the decoration.
                    .or_else(|| palette_colour(palette, 'o')),
            };
            let Some(colour) = colour else {
                continue;
            };
            let x = art.x + i32::try_from(column).unwrap_or(i32::MAX);
            let y = art.y + i32::try_from(row).unwrap_or(i32::MAX);
            if x < 0 || y < 0 || x >= width || y >= height {
                continue;
            }
            let index = usize::try_from(y * width + x).expect("in-bounds index fits usize");
            pixels[index] = Some(colour);
        }
    }
    SpriteFrame::from_pixels(frame.size().width, frame.size().height, pixels)
}

/// Returns the authored scene master registered for the adventurer's class,
/// with the persona's skin, hair and accent substituted into the master's
/// role clusters.
#[must_use]
pub fn adventurer_animation_frame(
    persona: &AdventurerPersona,
    pose: ScenePose,
    animation_frame: u8,
) -> SpriteFrame {
    let colours = adventurer_palette(
        persona.appearance.skin_tone,
        persona.appearance.hair_tone,
        persona.appearance.garb,
        persona.class,
        persona.appearance.accent,
    );
    if persona.class == AdventurerClass::Barbarian {
        personalise(
            &barbarian_v2::frame(pose, animation_frame),
            barbarian_v2::palette(),
            barbarian_role,
            &colours,
        )
    } else if persona.class == AdventurerClass::Druid {
        let frame = personalise(&druid_world_frame(), DRUID_PALETTE, standard_role, &colours);
        apply_pose(&frame, pose, DRUID_PALETTE, &colours)
    } else {
        let (frame, palette) = archetypes::world_master(persona.class)
            .expect("every non-Druid production class has an authored sprite route");
        let frame = personalise(&frame, palette, standard_role, &colours);
        apply_pose(&frame, pose, palette, &colours)
    }
}

/// Returns the authored 8x12 roster master for the adventurer's silhouette
/// family, personalised from the same palette as its world master. Roster
/// masters carry no pose: state is told by grounding, markers and nameplates.
#[must_use]
pub fn adventurer_roster_frame(persona: &AdventurerPersona) -> SpriteFrame {
    let colours = adventurer_palette(
        persona.appearance.skin_tone,
        persona.appearance.hair_tone,
        persona.appearance.garb,
        persona.class,
        persona.appearance.accent,
    );
    let (frame, palette) = roster::master(roster::family_for(persona.class));
    personalise(&frame, palette, standard_role, &colours)
}

/// Whether time can select different authored pixels for this adventurer pose.
/// Renderers use this to avoid scheduling frames for static class masters.
#[must_use]
pub fn adventurer_pose_is_animated(persona: &AdventurerPersona, pose: ScenePose) -> bool {
    persona.class == AdventurerClass::Barbarian && matches!(pose, ScenePose::Working)
}

#[cfg(test)]
mod tests {
    use super::*;

    type MasterPaletteFixture = (
        String,
        &'static [IndexedPaletteEntry],
        fn(char) -> Option<PersonaRole>,
    );

    /// Persona substitution matches pixels by colour, so a role colour that
    /// also appears under a class-owned key would silently recolour that
    /// cluster too. Every master palette must keep role colours unique.
    #[test]
    fn persona_role_colours_are_unique_within_every_master_palette() {
        let mut palettes: Vec<MasterPaletteFixture> = vec![
            ("Druid".to_owned(), DRUID_PALETTE, standard_role),
            (
                "Barbarian v2".to_owned(),
                barbarian_v2::palette(),
                barbarian_role,
            ),
        ];
        for class in AdventurerClass::ALL {
            if let Some((_, palette)) = archetypes::world_master(*class) {
                palettes.push((format!("{class:?}"), palette, standard_role));
            }
        }
        for family in roster::RosterFamily::ALL {
            let (_, palette) = roster::master(*family);
            palettes.push((format!("roster {family:?}"), palette, standard_role));
        }

        for (label, palette, role_for) in palettes {
            for entry in palette {
                if role_for(entry.key).is_none() {
                    continue;
                }
                let Some(colour) = entry.colour else {
                    continue;
                };
                for other in palette {
                    if other.key == entry.key {
                        continue;
                    }
                    assert_ne!(
                        other.colour,
                        Some(colour),
                        "{label}: role key '{}' shares a colour with key '{}'",
                        entry.key,
                        other.key
                    );
                }
            }
        }
    }
}
