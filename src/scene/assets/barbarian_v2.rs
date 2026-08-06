//! Compact, pose-specific Barbarian world masters.
//!
//! These deliberately follow the same constraints that make Pixtuoid's small
//! coworkers readable: a large face, connected colour clusters, short limbs,
//! a restrained value palette and equipment that remains at least two pixels
//! thick. The 16x24 canvas preserves every existing station and hit-region
//! contract while the painted silhouette stays compact.

use std::sync::OnceLock;

use crate::scene::{pixel::Rgb, sprite::SpriteFrame, stage::ScenePose};

use super::{IndexedPaletteEntry, indexed_sprite};

const PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(30, 22, 27)),
    },
    IndexedPaletteEntry {
        key: 'H',
        colour: Some(Rgb::new(91, 45, 30)),
    },
    IndexedPaletteEntry {
        key: 'h',
        colour: Some(Rgb::new(157, 76, 39)),
    },
    IndexedPaletteEntry {
        key: 'S',
        colour: Some(Rgb::new(244, 163, 94)),
    },
    // Oxblood rather than oak-brown: the torso is the sprite's largest fill
    // and must keep colour distance from the Hall's oak floor.
    IndexedPaletteEntry {
        key: 'L',
        colour: Some(Rgb::new(92, 42, 44)),
    },
    IndexedPaletteEntry {
        key: 'l',
        colour: Some(Rgb::new(184, 91, 42)),
    },
    IndexedPaletteEntry {
        key: 'M',
        colour: Some(Rgb::new(218, 226, 217)),
    },
    IndexedPaletteEntry {
        key: 'm',
        colour: Some(Rgb::new(112, 132, 134)),
    },
    IndexedPaletteEntry {
        key: 'a',
        colour: Some(Rgb::new(210, 55, 48)),
    },
];

const SETTLED: &[&str] = &[
    "................",
    "................",
    "....oHHHHHHo....",
    "...oHhhhhhhHo...",
    "...oHSSSSSSHo...",
    "...oHSooSSoHo...",
    "...oHSSSSSSHo...",
    "...oHHHSSHHo....",
    "....oHHHHHHo....",
    "mMM.oLLLLLLo....",
    "MMm.oLllllLo....",
    ".m..oLLaaLLo....",
    ".m.oLLLLLLLLo...",
    ".m.oLLLaaLLLo...",
    ".m.oLLLLLLLLo...",
    ".m..oLLLLLLo....",
    ".m...oLLLLo.....",
    ".m...oLLLLo.....",
    ".m...oLooLo.....",
    ".m..oo....oo....",
    ".m..oo....oo....",
    "....ooo..ooo....",
    "................",
    "................",
];

const WORKING_0: &[&str] = &[
    "................",
    "................",
    "....oHHHHHHo....",
    "...oHhhhhhhHo...",
    "...oHSSSSSSHo...",
    "...oHSooSSoHo...",
    "...oHSSSSSSHo...",
    "...oHHHSSHHo....",
    "....oHHHHHHo....",
    "....oLLLLLLo.MMm",
    "....oLllllLo.mMM",
    "....oLLaaLLo..m.",
    "...oLLLLLLLLo.m.",
    "...oLLLaaLLLom..",
    "...oLLLLLLLLo...",
    "....oLLLLLLo....",
    ".....oLLLLo.....",
    ".....oLLLLo.....",
    ".....oLooLo.....",
    "....oo....oo....",
    "...oo......oo...",
    "...ooo....ooo...",
    "................",
    "................",
];

const WORKING_1: &[&str] = &[
    "................",
    "....m...........",
    "...mMMHHHHHo....",
    "..mMMhhhhhhHo...",
    "..m.oHSSSSSHo...",
    ".m..oHSooSoHo...",
    ".m..oHSSSSSHo...",
    ".m..oHHSSHHHo...",
    ".m...oHHHHHo....",
    ".m..oLLLLLLo....",
    ".m.oLLllllLLo...",
    ".m.oLLLaaLLLo...",
    ".m.oLLLLLLLLo...",
    ".m.oLLLaaLLLo...",
    ".m.oLLLLLLLLo...",
    ".m..oLLLLLLo....",
    ".m...oLLLLo.....",
    ".m...oLLLLo.....",
    ".m...oLooLo.....",
    ".m..oo....oo....",
    ".m.oo......oo...",
    "...ooo....ooo...",
    "................",
    "................",
];

const COUNSEL: &[&str] = &[
    "...........So...",
    "..........oSo...",
    "....oHHHHHHo....",
    "...oHhhhhhhHo...",
    "...oHSSSSSSHo...",
    "...oHSooSSoHo...",
    "...oHSSSSSSHo...",
    "...oHHHSSHHo....",
    "....oHHHHHHo....",
    "mMM.oLLLLLLLo...",
    "MMm.oLllllLLSo..",
    ".m..oLLaaLLLo...",
    ".m.oLLLLLLLLo...",
    ".m.oLLLaaLLLo...",
    ".m.oLLLLLLLLo...",
    ".m..oLLLLLLo....",
    ".m...oLLLLo.....",
    ".m...oLLLLo.....",
    ".m...oLooLo.....",
    ".m..oo....oo....",
    ".m..oo....oo....",
    "....ooo..ooo....",
    "................",
    "................",
];

const SPOILS: &[&str] = &[
    "................",
    "................",
    "....oHHHHHHo....",
    "...oHhhhhhhHo...",
    "...oHSSSSSSHo...",
    "...oHSooSSoHo...",
    "...oHSSSSSSHo...",
    "...oHHHSSHHo....",
    "....oHHHHHHo....",
    "mMM.oLLLLLLo....",
    "MMm.oLllllLo....",
    ".m..oLLaaLLo....",
    ".m.oLLLLLLLLo...",
    ".m.oLLLaaLLLo...",
    ".m.oLLLLLLLLo...",
    ".m..oLLLLLLo....",
    ".m...oLLLLo.....",
    ".m..oMLLMMo.....",
    ".m..oMaaaaMo....",
    ".m..oMMMMMMo....",
    ".m...oo..oo.....",
    "....ooo..ooo....",
    "................",
    "................",
];

const RESTING: &[&str] = &[
    "................",
    "................",
    "................",
    "....oHHHHHHo....",
    "...oHhhhhhhHo...",
    "...oHSSSSSSHo...",
    "...oHSooSSoHo...",
    "...oHSSSSSSHo...",
    "...oHHHSSHHo....",
    "....oHHHHHHo....",
    "mMM.oLLLLLLo....",
    "MMm.oLllllLo....",
    ".m..oLLaaLLo....",
    ".m.oLLLLLLLLo...",
    ".m.oLLLaaLLLo...",
    ".m..oLLLLLLo....",
    ".m..ooLLLLoo....",
    ".m.oLLooooLLo...",
    ".m.oLLo..oLLo...",
    ".m..oo....oo....",
    "....ooo..ooo....",
    "................",
    "................",
    "................",
];

const UNKNOWN: &[&str] = &[
    "................",
    "................",
    ".....oooooo.....",
    "....oLLLLLLo....",
    "...oLLooooLLo...",
    "...oLo....oLo...",
    "...oLo.oo.oLo...",
    "...oLo....oLo...",
    "....oLLLLLLo....",
    "mMM.oLLLLLLo....",
    "MMm.oLllllLo....",
    ".m..oLLaaLLo....",
    ".m.oLLLLLLLLo...",
    ".m.oLLLaaLLLo...",
    ".m.oLLLLLLLLo...",
    ".m..oLLLLLLo....",
    ".m...oLLLLo.....",
    ".m...oLLLLo.....",
    ".m...oLooLo.....",
    ".m..oo....oo....",
    ".m..oo....oo....",
    "....ooo..ooo....",
    "................",
    "................",
];

/// The authoring palette for every Barbarian pose master. Exposed so persona
/// substitution can locate role colours; the Barbarian grammar differs from
/// the standard archetype roles (`S` skin, `H`/`h` hair, `L`/`l` leather).
#[must_use]
pub(crate) const fn palette() -> &'static [IndexedPaletteEntry] {
    PALETTE
}

#[must_use]
pub(crate) fn frame(pose: ScenePose, animation_frame: u8) -> SpriteFrame {
    let (cache, rows) = match pose {
        ScenePose::Working if animation_frame % 2 == 1 => (&WORKING_1_FRAME, WORKING_1),
        ScenePose::Working => (&WORKING_0_FRAME, WORKING_0),
        ScenePose::SeekingCounsel => (&COUNSEL_FRAME, COUNSEL),
        ScenePose::ReturningWithSpoils => (&SPOILS_FRAME, SPOILS),
        ScenePose::Resting => (&RESTING_FRAME, RESTING),
        ScenePose::Unknown => (&UNKNOWN_FRAME, UNKNOWN),
        ScenePose::Settled => (&SETTLED_FRAME, SETTLED),
    };
    cache
        .get_or_init(|| indexed_sprite(rows, PALETTE).expect("Barbarian v2 world master is valid"))
        .clone()
}

static SETTLED_FRAME: OnceLock<SpriteFrame> = OnceLock::new();
static WORKING_0_FRAME: OnceLock<SpriteFrame> = OnceLock::new();
static WORKING_1_FRAME: OnceLock<SpriteFrame> = OnceLock::new();
static COUNSEL_FRAME: OnceLock<SpriteFrame> = OnceLock::new();
static SPOILS_FRAME: OnceLock<SpriteFrame> = OnceLock::new();
static RESTING_FRAME: OnceLock<SpriteFrame> = OnceLock::new();
static UNKNOWN_FRAME: OnceLock<SpriteFrame> = OnceLock::new();
