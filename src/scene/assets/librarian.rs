use std::sync::OnceLock;

use crate::scene::{pixel::Rgb, sprite::SpriteFrame};

use super::{IndexedPaletteEntry, indexed_sprite};

pub const WORLD_WIDTH: u16 = 16;
pub const WORLD_HEIGHT: u16 = 24;
pub const PORTRAIT_WIDTH: u16 = 24;
pub const PORTRAIT_HEIGHT: u16 = 32;

const PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(28, 18, 23)),
    },
    IndexedPaletteEntry {
        key: 'b',
        colour: Some(Rgb::new(82, 43, 29)),
    },
    IndexedPaletteEntry {
        key: 'B',
        colour: Some(Rgb::new(142, 76, 42)),
    },
    IndexedPaletteEntry {
        key: 'H',
        colour: Some(Rgb::new(195, 116, 59)),
    },
    IndexedPaletteEntry {
        key: 's',
        colour: Some(Rgb::new(225, 166, 102)),
    },
    IndexedPaletteEntry {
        key: 'P',
        colour: Some(Rgb::new(242, 218, 163)),
    },
    IndexedPaletteEntry {
        key: 'p',
        colour: Some(Rgb::new(91, 49, 111)),
    },
    IndexedPaletteEntry {
        key: 'g',
        colour: Some(Rgb::new(218, 164, 55)),
    },
    IndexedPaletteEntry {
        key: 'r',
        colour: Some(Rgb::new(130, 37, 52)),
    },
];

const WORLD_ROWS: &[&str] = &[
    "................",
    ".....oooo.......",
    "...oobbbboo.....",
    "..obBBBBBBbo....",
    ".obBBHBBHBBbo...",
    ".obBssssssBbo...",
    ".obsoPooPosbo...",
    "..bsosssosb.....",
    "..obBssssBbo....",
    "...obBBBBbo.....",
    "..oobppppboo....",
    ".obbbpggpbbbo...",
    "obBbbpPPpbbBbo..",
    "obBbbppppbbBbo..",
    "obBbbpggpbbBbo..",
    "obBbbbbbbbbBbo..",
    ".oBbbbrrbbbBo...",
    ".obbbrrrrbbbo...",
    "..obbbbbbbbo....",
    "..obb....bbo....",
    ".obb......bbo...",
    ".ob........bo...",
    "..oo......oo....",
    "................",
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
        let source = world();
        let mut pixels = vec![None; usize::from(PORTRAIT_WIDTH) * usize::from(PORTRAIT_HEIGHT)];
        for y in 0..WORLD_HEIGHT {
            for x in 0..WORLD_WIDTH {
                let source_index = usize::from(y) * usize::from(WORLD_WIDTH) + usize::from(x);
                let target_x = x + 4;
                let target_y = y + 4;
                let target_index =
                    usize::from(target_y) * usize::from(PORTRAIT_WIDTH) + usize::from(target_x);
                pixels[target_index] = source.pixels()[source_index];
            }
        }
        SpriteFrame::from_pixels(PORTRAIT_WIDTH, PORTRAIT_HEIGHT, pixels)
    })
}
