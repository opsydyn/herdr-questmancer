use std::sync::OnceLock;

use crate::domain::AdventurerPersona;

use super::{IndexedPaletteEntry, archetypes, indexed_sprite, palette::adventurer_palette};
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

/// Returns the authored scene master registered for the adventurer's class.
#[must_use]
pub fn adventurer_animation_frame(
    persona: &AdventurerPersona,
    pose: ScenePose,
    animation_frame: u8,
) -> SpriteFrame {
    if persona.class == AdventurerClass::Barbarian {
        super::barbarian_v2::frame(pose, animation_frame)
    } else if persona.class == AdventurerClass::Druid {
        let frame = druid_world_frame();
        let accent = adventurer_palette(
            persona.appearance.skin_tone,
            persona.appearance.hair_tone,
            persona.appearance.garb,
            persona.class,
            persona.appearance.accent,
        )
        .accent;
        SpriteFrame::from_pixels(
            frame.size().width,
            frame.size().height,
            frame
                .pixels()
                .iter()
                .map(|pixel| {
                    pixel.map(|colour| {
                        if colour == DRUID_ACCENT {
                            accent
                        } else {
                            colour
                        }
                    })
                })
                .collect(),
        )
    } else {
        archetypes::world_frame(persona.class)
            .expect("every non-Druid production class has an authored sprite route")
    }
}
