use std::sync::OnceLock;

use crate::domain::{AdventurerPersona, Ancestry, FaceDetail};

use super::{IndexedPaletteEntry, indexed_sprite, palette::adventurer_palette};
use crate::scene::{pixel::Rgb, sprite::SpriteFrame, stage::ScenePose};

const SKIN_SLOT: Rgb = Rgb::new(1, 0, 0);
const HAIR_SLOT: Rgb = Rgb::new(2, 0, 0);
const CLOTH_SLOT: Rgb = Rgb::new(3, 0, 0);
const METAL_SLOT: Rgb = Rgb::new(4, 0, 0);
const ACCENT_SLOT: Rgb = Rgb::new(5, 0, 0);
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
