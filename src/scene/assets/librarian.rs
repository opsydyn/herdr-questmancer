use std::sync::OnceLock;

use crate::scene::{pixel::Rgb, sprite::SpriteFrame};

use super::{IndexedPaletteEntry, indexed_sprite};

pub const WORLD_WIDTH: u16 = 16;
pub const WORLD_HEIGHT: u16 = 24;
pub const PORTRAIT_WIDTH: u16 = 24;
pub const PORTRAIT_HEIGHT: u16 = 32;

// Orangutan fur, low gold spectacles, purple robe and carried books. The
// Librarian is a static help NPC; these assets have no animation clock.
const PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(28, 18, 23)),
    },
    IndexedPaletteEntry {
        key: 'b',
        colour: Some(Rgb::new(91, 51, 33)),
    },
    IndexedPaletteEntry {
        key: 'B',
        colour: Some(Rgb::new(180, 91, 37)),
    },
    IndexedPaletteEntry {
        key: 'H',
        colour: Some(Rgb::new(233, 142, 56)),
    },
    IndexedPaletteEntry {
        key: 's',
        colour: Some(Rgb::new(185, 170, 143)),
    },
    IndexedPaletteEntry {
        key: 'P',
        colour: Some(Rgb::new(248, 225, 170)),
    },
    IndexedPaletteEntry {
        key: 'p',
        colour: Some(Rgb::new(99, 55, 119)),
    },
    IndexedPaletteEntry {
        key: 'g',
        colour: Some(Rgb::new(222, 172, 66)),
    },
    IndexedPaletteEntry {
        key: 'r',
        colour: Some(Rgb::new(146, 48, 48)),
    },
    IndexedPaletteEntry {
        key: 'G',
        colour: Some(Rgb::new(49, 104, 71)),
    },
    IndexedPaletteEntry {
        key: 'U',
        colour: Some(Rgb::new(43, 82, 117)),
    },
    IndexedPaletteEntry {
        key: 'D',
        colour: Some(Rgb::new(103, 68, 40)),
    },
];

const WORLD_ROWS: &[&str] = &[
    "................",
    "......oHo.......",
    "....ooHHBoo.....",
    "...oBHHHBBBoo...",
    "..oBHHBBBBBBBo..",
    "..oBBssssssBBo..",
    ".oBBsggssggsBBo.",
    ".oBBgPoggoPgBBo.",
    ".oBBsgssssgsBBo.",
    "..oBBssoossBBo..",
    "..oBBssssssBBo..",
    "...oBBHHBBBo....",
    "..ooBpggpBoo....",
    ".opppgGGgpppo...",
    "oBBrroggoppppo..",
    "oBrPPrrgoUUppo..",
    "oBrPPrrgoUPpBo..",
    "oBBrrrrgoUUoBo..",
    ".oppppggpppppo..",
    "..opppppppppo...",
    "...obbboobbbo...",
    "...oooo.oooo....",
    "................",
    "................",
];

// Independently authored face, book spines and robe for the Ledger canvas.
const PORTRAIT_ROWS: &[&str] = &[
    "........................",
    "...........HH...........",
    ".........oHHHBo.........",
    ".......ooBHHHBBoo.......",
    "......oBBHHHHBBBBo......",
    ".....oBBHHBBBBBBBBo.....",
    "....oBBBssssssssBBBo....",
    "....oBBssssssssssBBo....",
    "...oBBBsgggssgggsBBBo...",
    "...oBBsgPooggooPgsBBo...",
    "...oBBBsgggssgggsBBBo...",
    "...oBBBssoossoossBBBo...",
    "....oBBssssssssssBBo....",
    "....oBBBsssoosssBBBo....",
    ".....oBBBssssssBBBo.....",
    ".....ooBBHHHHBBBoo......",
    "...oopppBggggBppppoo....",
    "..opppppgGGGGgppppppo...",
    "..opppppgGGGGgppppppo...",
    ".oBBrrrpgGGGGgppUUpppo..",
    ".oBrPPrrgGGGGgpUPUpppo..",
    ".oBrPPrrgDDDDgpUPUppBo..",
    ".oBBrrrpgDggDgpUUUppBo..",
    ".oBGGGGpgDggDgpppppBBo..",
    ".oBgPPGgppggpppppppBo...",
    "..oBGGGgppppppppppppo...",
    "...opppppppppppppppo....",
    "....opppppppppppppo.....",
    ".....obbbbbooobbbo......",
    ".....oooooo.oooooo......",
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
