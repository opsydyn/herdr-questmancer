use std::sync::OnceLock;

use crate::scene::{pixel::Rgb, sprite::SpriteFrame};

use super::{IndexedPaletteEntry, indexed_sprite};

pub const WORLD_WIDTH: u16 = 16;
pub const WORLD_HEIGHT: u16 = 24;
pub const PORTRAIT_WIDTH: u16 = 24;
pub const PORTRAIT_HEIGHT: u16 = 32;

// Orangutan fur, low gold spectacles, purple robe and carried books. The
// Librarian is a static help NPC; these assets have no animation clock.
// Keeper of the Stacks direction selected on 2026-11-17.
const PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(44, 34, 41)),
    },
    IndexedPaletteEntry {
        key: 'f',
        colour: Some(Rgb::new(107, 59, 43)),
    },
    IndexedPaletteEntry {
        key: 'F',
        colour: Some(Rgb::new(183, 96, 50)),
    },
    IndexedPaletteEntry {
        key: 'H',
        colour: Some(Rgb::new(215, 135, 67)),
    },
    IndexedPaletteEntry {
        key: 't',
        colour: Some(Rgb::new(120, 104, 89)),
    },
    IndexedPaletteEntry {
        key: 's',
        colour: Some(Rgb::new(173, 154, 128)),
    },
    IndexedPaletteEntry {
        key: 'S',
        colour: Some(Rgb::new(209, 185, 154)),
    },
    IndexedPaletteEntry {
        key: 'e',
        colour: Some(Rgb::new(49, 44, 40)),
    },
    IndexedPaletteEntry {
        key: 'g',
        colour: Some(Rgb::new(204, 164, 91)),
    },
    IndexedPaletteEntry {
        key: 'G',
        colour: Some(Rgb::new(237, 208, 141)),
    },
    IndexedPaletteEntry {
        key: 'p',
        colour: Some(Rgb::new(57, 39, 63)),
    },
    IndexedPaletteEntry {
        key: 'P',
        colour: Some(Rgb::new(101, 67, 109)),
    },
    IndexedPaletteEntry {
        key: 'v',
        colour: Some(Rgb::new(145, 103, 149)),
    },
    IndexedPaletteEntry {
        key: 'r',
        colour: Some(Rgb::new(144, 60, 67)),
    },
    IndexedPaletteEntry {
        key: 'u',
        colour: Some(Rgb::new(54, 85, 107)),
    },
    IndexedPaletteEntry {
        key: 'U',
        colour: Some(Rgb::new(99, 124, 143)),
    },
    IndexedPaletteEntry {
        key: 'b',
        colour: Some(Rgb::new(76, 102, 71)),
    },
    IndexedPaletteEntry {
        key: 'B',
        colour: Some(Rgb::new(132, 148, 104)),
    },
    IndexedPaletteEntry {
        key: 'w',
        colour: Some(Rgb::new(223, 201, 159)),
    },
    IndexedPaletteEntry {
        key: 'd',
        colour: Some(Rgb::new(116, 83, 59)),
    },
    IndexedPaletteEntry {
        key: 'D',
        colour: Some(Rgb::new(177, 132, 74)),
    },
];

const WORLD_ROWS: &[&str] = &[
    "................",
    ".......oHo......",
    ".....ofHHFfo....",
    "....oFHHFFFfo...",
    "...oFFtssstFFo..",
    "..oFFtsSSstFfo..",
    "..oFtsGgsGgtFFo.",
    "..oFtsgeggegtFo.",
    "..oFtssestFFo...",
    "..oFtSSSstFFo...",
    "...oFstttFFFo...",
    "..oppFHHFFFpo...",
    ".opPvPgBgPvPpo..",
    ".orrrogBbgvPPpo.",
    "orwwwrgBgPuuPpo.",
    "obBBbPgbgPuwuFo.",
    "obwwbPgBgPuuFFo.",
    ".ouUuPdgdpFFfo..",
    ".ouwwuPgPvPpo...",
    "..opPvgPPvpo....",
    "...fttf.fttf....",
    "...fstf.fstf....",
    "................",
    "................",
];

// Independent Ledger face, fingers and book spines; never a scaled world sprite.
const PORTRAIT_ROWS: &[&str] = &[
    "........................",
    "............oHo.........",
    "..........ofHHFfo.......",
    "........ofFHHHFFfo......",
    ".......oFFHHHHFFFfo.....",
    "......oFFFttssttFFFo....",
    ".....oFFFtsSSSsstFFFo...",
    "....oFFtssSSSSsstFFfo...",
    "....oFFtsGGgssGGgstFFo..",
    "....oFFtsgSegggSegstFo..",
    "....oFFtsgegggegstFFo...",
    ".....oFFtsseesstFFfo....",
    ".....oFFtSSSSSstFFo.....",
    "......oFFstttstFFFo.....",
    "......ofFHHHHFFFfo......",
    "....oopPPgBBgPPppoo.....",
    "...opPvPPgbbgPPvPPpo....",
    "..orrrogPgBBgPuuPPppo...",
    ".orwwwwrgBbbgPuwuPPpo...",
    ".orrrrrgPgBBgPuuUFFfo...",
    ".obBBBbgPggggPuUwUFFFo..",
    ".obwwwwbgPddgPuuUFFFo...",
    ".obbbbbgPdgDgPPFffFPo...",
    "..ouUUugPPgPPvPPppPo....",
    "..ouwwuugPvPPvPvPpo.....",
    "...ouuuugPPvPPvPpo......",
    "....opPvPvgPPvPpo.......",
    ".....opPPgPPPppo........",
    "......oftttfoftttfo.....",
    "......oftstfoftstfo.....",
    "........................",
    "........................",
];

#[must_use]
pub fn world() -> &'static SpriteFrame {
    static FRAME: OnceLock<SpriteFrame> = OnceLock::new();
    FRAME.get_or_init(|| {
        indexed_sprite(WORLD_ROWS, PALETTE).expect("built-in Librarian world art must be valid")
    })
}

#[must_use]
pub fn ledger_portrait() -> &'static SpriteFrame {
    static FRAME: OnceLock<SpriteFrame> = OnceLock::new();
    FRAME.get_or_init(|| {
        indexed_sprite(PORTRAIT_ROWS, PALETTE)
            .expect("built-in Librarian portrait art must be valid")
    })
}
