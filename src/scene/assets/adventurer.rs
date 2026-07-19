use std::sync::OnceLock;

use crate::domain::{AdventurerPersona, Ancestry, FaceDetail};

use super::{IndexedPaletteEntry, indexed_sprite, palette::adventurer_palette};
use crate::{
    domain::AdventurerClass,
    scene::{pixel::Rgb, sprite::SpriteFrame, stage::ScenePose},
};

const SKIN_SLOT: Rgb = Rgb::new(1, 0, 0);
const HAIR_SLOT: Rgb = Rgb::new(2, 0, 0);
const CLOTH_SLOT: Rgb = Rgb::new(3, 0, 0);
const METAL_SLOT: Rgb = Rgb::new(4, 0, 0);
const ACCENT_SLOT: Rgb = Rgb::new(5, 0, 0);
const DRUID_ACCENT: Rgb = Rgb::new(85, 174, 206);
const EYE_SLOT: Rgb = Rgb::new(6, 0, 0);

const AUTHORING_PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'S',
        colour: Some(SKIN_SLOT),
    },
    IndexedPaletteEntry {
        key: 'H',
        colour: Some(HAIR_SLOT),
    },
    IndexedPaletteEntry {
        key: 'C',
        colour: Some(CLOTH_SLOT),
    },
    IndexedPaletteEntry {
        key: 'M',
        colour: Some(METAL_SLOT),
    },
    IndexedPaletteEntry {
        key: 'A',
        colour: Some(ACCENT_SLOT),
    },
    IndexedPaletteEntry {
        key: 'E',
        colour: Some(EYE_SLOT),
    },
];

const WORKING: &[&str] = &[
    "........", "..HHHH..", ".HHSSHH.", ".HSEESH.", "..SSSS..", ".ACCCCA.", ".ACMMCA.", "..CCCC..",
    "..CCCC..", "..C..C..", ".MM..MM.", "..C..C..", ".MM..MM.", "........",
];
const WORKING_ALT: &[&str] = &[
    "........", "..HHHH..", ".HHSSHH.", ".HSEESH.", "..SSSS..", "..CCCCA.", ".AMMCCA.", "..CCCC..",
    "..CCCC..", ".C...C..", ".MM..MM.", "..C...C.", ".MM..MM.", "........",
];
const WALKING_ALT: &[&str] = &[
    "........", "..HHHH..", ".HHSSHH.", ".HSEESH.", "..SSSS..", ".ACCCC..", ".ACCMMMA", "..CCCC..",
    "..CCCC..", "...C.C..", "..MM.MM.", ".C...C..", ".MM...MM", "........",
];
const COUNSEL: &[&str] = &[
    "........", "..HHHH..", ".HHSSHH.", ".HSEESH.", "..SSSS..", "..CCCC..", ".ACCCCA.", "AACCCCAA",
    "..CCCC..", "..C..C..", ".MM..MM.", "..C..C..", ".MM..MM.", "........",
];
const SPOILS: &[&str] = &[
    "........", "..HHHH..", ".HHSSHH.", ".HSEESH.", "..SSSS..", ".ACCCC..", ".ACCCMMA", "..CCCCAA",
    "..CCCCAA", "..C..C..", ".MM..MM.", "..C..C..", ".MM..MM.", "........",
];
const SETTLED: &[&str] = &[
    "........", "..HHHH..", ".HHSSHH.", ".HSEESH.", "..SSSS..", "..CCCC..", ".ACCCCA.", "..CCCC..",
    "..CCCC..", "..C..C..", ".MM..MM.", "..C..C..", ".MM..MM.", "........",
];
const RESTING: &[&str] = &[
    "........", "........", "..HHHH..", ".HHSSHH.", ".HSEESH.", "..SSSS..", ".ACCCCA.", "..CCCC..",
    ".CCCCCC.", ".CC..CC.", ".MM..MM.", "........", "........", "........",
];
const UNKNOWN: &[&str] = &[
    "........", "..HHHH..", ".HHSSHH.", ".HSEESH.", "..SSSS..", ".ACCCCA.", ".ACCCCA.", "..CCCC..",
    "..CCCC..", "..C..C..", ".MM..MM.", "..C..C..", ".MM..MM.", "....A...",
];

// The first production master. This is intentionally a complete authored
// sprite rather than a palette swap of the compact fallback: the hood,
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

/// Returns the highest-fidelity authored scene frame currently available for
/// the adventurer. Unpromoted classes retain the compact semantic fallback.
#[must_use]
pub fn adventurer_animation_frame(
    persona: &AdventurerPersona,
    pose: ScenePose,
    animation_frame: u8,
) -> SpriteFrame {
    if persona.class == AdventurerClass::Druid {
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
        compact_adventurer_animation_frame(persona, pose, animation_frame)
    }
}

#[must_use]
pub fn compact_adventurer_frame(
    persona: &AdventurerPersona,
    pose: ScenePose,
    alternate: bool,
) -> SpriteFrame {
    compact_adventurer_animation_frame(persona, pose, u8::from(alternate))
}

#[must_use]
pub fn compact_adventurer_animation_frame(
    persona: &AdventurerPersona,
    pose: ScenePose,
    animation_frame: u8,
) -> SpriteFrame {
    let base = base_frame(pose, animation_frame);
    let colours = adventurer_palette(
        persona.appearance.skin_tone,
        persona.appearance.hair_tone,
        persona.appearance.garb,
        persona.class,
        persona.appearance.accent,
    );
    let mut pixels = base
        .pixels()
        .iter()
        .map(|pixel| {
            pixel.map(|colour| match colour {
                SKIN_SLOT => colours.skin,
                HAIR_SLOT => colours.hair,
                CLOTH_SLOT => colours.cloth,
                METAL_SLOT => colours.metal,
                ACCENT_SLOT => colours.accent,
                EYE_SLOT => colours.eye,
                other => other,
            })
        })
        .collect::<Vec<_>>();
    apply_persona_details(persona, &mut pixels);
    SpriteFrame::from_pixels(8, 14, pixels)
}

fn apply_persona_details(persona: &AdventurerPersona, pixels: &mut [Option<Rgb>]) {
    let detail_x = match persona.appearance.face_detail {
        FaceDetail::None | FaceDetail::Freckles => 3,
        FaceDetail::RoundGlasses | FaceDetail::Moustache => 4,
        FaceDetail::SquareGlasses | FaceDetail::Visor => 5,
    };
    pixels[4 * 8 + detail_x] = Some(if persona.appearance.face_detail == FaceDetail::None {
        adventurer_palette(
            persona.appearance.skin_tone,
            persona.appearance.hair_tone,
            persona.appearance.garb,
            persona.class,
            persona.appearance.accent,
        )
        .skin
    } else {
        adventurer_palette(
            persona.appearance.skin_tone,
            persona.appearance.hair_tone,
            persona.appearance.garb,
            persona.class,
            persona.appearance.accent,
        )
        .accent
    });

    let ancestry_x = match persona.ancestry {
        Ancestry::Elf | Ancestry::Goblin => 0,
        Ancestry::Dwarf | Ancestry::Gnome | Ancestry::Halfling => 1,
        Ancestry::Human | Ancestry::Orc => 7,
    };
    pixels[3 * 8 + ancestry_x] = Some(
        adventurer_palette(
            persona.appearance.skin_tone,
            persona.appearance.hair_tone,
            persona.appearance.garb,
            persona.class,
            persona.appearance.accent,
        )
        .skin,
    );
}

fn base_frame(pose: ScenePose, animation_frame: u8) -> &'static SpriteFrame {
    static WORKING_BASE: OnceLock<SpriteFrame> = OnceLock::new();
    static WORKING_ONE: OnceLock<SpriteFrame> = OnceLock::new();
    static WORKING_TWO: OnceLock<SpriteFrame> = OnceLock::new();
    static COUNSEL_BASE: OnceLock<SpriteFrame> = OnceLock::new();
    static SPOILS_BASE: OnceLock<SpriteFrame> = OnceLock::new();
    static SETTLED_BASE: OnceLock<SpriteFrame> = OnceLock::new();
    static RESTING_BASE: OnceLock<SpriteFrame> = OnceLock::new();
    static UNKNOWN_BASE: OnceLock<SpriteFrame> = OnceLock::new();

    let (cell, rows) = match pose {
        ScenePose::Working => match animation_frame % 3 {
            0 => (&WORKING_BASE, WORKING),
            1 => (&WORKING_ONE, WORKING_ALT),
            _ => (&WORKING_TWO, WALKING_ALT),
        },
        ScenePose::SeekingCounsel => (&COUNSEL_BASE, COUNSEL),
        ScenePose::ReturningWithSpoils => (&SPOILS_BASE, SPOILS),
        ScenePose::Settled => (&SETTLED_BASE, SETTLED),
        ScenePose::Resting => (&RESTING_BASE, RESTING),
        ScenePose::Unknown => (&UNKNOWN_BASE, UNKNOWN),
    };
    cell.get_or_init(|| {
        indexed_sprite(rows, AUTHORING_PALETTE)
            .expect("built-in compact adventurer assets are valid")
    })
}
