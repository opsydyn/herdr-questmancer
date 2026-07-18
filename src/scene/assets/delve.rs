use std::sync::OnceLock;

use crate::scene::{pixel::Rgb, sprite::SpriteFrame};

use super::{IndexedPaletteEntry, indexed_sprite};

pub const DEEP_BLUE_BLACK: Rgb = Rgb::new(8, 17, 27);
pub const DUNGEON_SHADOW: Rgb = Rgb::new(13, 29, 38);
pub const STONE_DARK: Rgb = Rgb::new(22, 43, 49);
pub const STONE_MID: Rgb = Rgb::new(37, 65, 68);
pub const STONE_LIGHT: Rgb = Rgb::new(68, 96, 94);
pub const FLOOR_DARK: Rgb = Rgb::new(23, 50, 54);
pub const FLOOR_MID: Rgb = Rgb::new(35, 75, 72);
pub const MOSS_DARK: Rgb = Rgb::new(31, 67, 45);
pub const MOSS_LIGHT: Rgb = Rgb::new(68, 104, 62);
pub const TEAL_GLOW: Rgb = Rgb::new(35, 155, 151);
pub const TEAL_LIGHT: Rgb = Rgb::new(76, 211, 197);
pub const MINERAL_VIOLET: Rgb = Rgb::new(75, 61, 119);
pub const VIOLET_LIGHT: Rgb = Rgb::new(119, 94, 164);
pub const OLD_OAK: Rgb = Rgb::new(82, 57, 39);
pub const TORCH_AMBER: Rgb = Rgb::new(218, 132, 43);
pub const TORCH_FLAME: Rgb = Rgb::new(250, 191, 76);
pub const BONE: Rgb = Rgb::new(170, 177, 151);
pub const RUST: Rgb = Rgb::new(113, 61, 42);
pub const CHEST_GOLD: Rgb = Rgb::new(170, 116, 40);
pub const WATER: Rgb = Rgb::new(30, 91, 111);
pub const WATER_LIGHT: Rgb = Rgb::new(48, 132, 145);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DelveAsset {
    DressedStoneWall,
    CrackedMossyFloor,
    Arch,
    Door,
    DescendingStair,
    ActivePassage,
    SealedGate,
    ExitLanding,
    Camp,
    Torch,
    Brazier,
    RuneStones,
    Roots,
    Columns,
    Rubble,
    Puddles,
    Bones,
    Chests,
    DungeonClutter,
}

impl DelveAsset {
    pub const ALL: &'static [Self] = &[
        Self::DressedStoneWall,
        Self::CrackedMossyFloor,
        Self::Arch,
        Self::Door,
        Self::DescendingStair,
        Self::ActivePassage,
        Self::SealedGate,
        Self::ExitLanding,
        Self::Camp,
        Self::Torch,
        Self::Brazier,
        Self::RuneStones,
        Self::Roots,
        Self::Columns,
        Self::Rubble,
        Self::Puddles,
        Self::Bones,
        Self::Chests,
        Self::DungeonClutter,
    ];
}

const PALETTE: &[IndexedPaletteEntry] = &[
    entry('a', DEEP_BLUE_BLACK),
    entry('d', DUNGEON_SHADOW),
    entry('s', STONE_DARK),
    entry('S', STONE_MID),
    entry('L', STONE_LIGHT),
    entry('f', FLOOR_DARK),
    entry('F', FLOOR_MID),
    entry('m', MOSS_DARK),
    entry('M', MOSS_LIGHT),
    entry('t', TEAL_GLOW),
    entry('T', TEAL_LIGHT),
    entry('v', MINERAL_VIOLET),
    entry('V', VIOLET_LIGHT),
    entry('o', OLD_OAK),
    entry('A', TORCH_AMBER),
    entry('x', TORCH_FLAME),
    entry('b', BONE),
    entry('r', RUST),
    entry('g', CHEST_GOLD),
    entry('w', WATER),
    entry('W', WATER_LIGHT),
];

const fn entry(key: char, colour: Rgb) -> IndexedPaletteEntry {
    IndexedPaletteEntry {
        key,
        colour: Some(colour),
    }
}

#[must_use]
pub fn frame(asset: DelveAsset) -> &'static SpriteFrame {
    static FRAMES: OnceLock<Vec<SpriteFrame>> = OnceLock::new();
    &FRAMES.get_or_init(build_frames)[asset as usize]
}

fn build_frames() -> Vec<SpriteFrame> {
    DelveAsset::ALL
        .iter()
        .copied()
        .map(|asset| indexed_sprite(rows(asset), PALETTE).expect("built-in Delve asset is valid"))
        .collect()
}

#[allow(
    clippy::too_many_lines,
    reason = "the original indexed Delve atlas is kept together"
)]
const fn rows(asset: DelveAsset) -> &'static [&'static str] {
    match asset {
        DelveAsset::DressedStoneWall => &[
            "ssSSSSLLSSSS",
            "sSSSSLLSSSSs",
            "SSLLssssSSSS",
            "SLLssssSSSSS",
            "ssssSSSSLLss",
            "sssSSSSLLsss",
        ],
        DelveAsset::CrackedMossyFloor => &[
            "fFFFFFffmFFF",
            "FFFFFffmMFFF",
            "FFFffsFFFmFF",
            "FFffsFFFFmMF",
            "FffFFFFsFFFF",
            "ffmMFFFFFsFF",
        ],
        DelveAsset::Arch => &[
            "...sSSSSSSs...",
            ".sSLLSSSSLLSs.",
            "sSLs......sLSs",
            "SLs........sLS",
            "Ss..........sS",
            "Ss..........sS",
            "Ss..........sS",
            "SS..........SS",
            "LS..........SL",
            "SS..........SS",
        ],
        DelveAsset::Door => &[
            "sSSSSSSSSSSs",
            "SooooooooooS",
            "SorroooooorS",
            "SooooooooooS",
            "SoooosoooooS",
            "SooooooooooS",
            "SoooooooAooS",
            "SooooooooooS",
            "SooooooooooS",
            "sSSSSSSSSSSs",
        ],
        DelveAsset::DescendingStair => &[
            "LLLLLLLLLLLLLLLL",
            ".SSSSSSSSSSSSSS.",
            "..FFFFFFFFFFFF..",
            "...SSSSSSSSSS...",
            "....ffffffff....",
            ".....SSSSSS.....",
            "......dddd......",
            ".......aa.......",
        ],
        DelveAsset::ActivePassage => &[
            "ssSSSSSSSSSSss",
            "sSffffFFFFFFSs",
            "SffFtFFFFFFffS",
            "SfFtTtFFFFFFfS",
            "SfFtFFFFFFFmfS",
            "SfffFFFFFFmmfS",
            "sSffffffffffSs",
            "ssSSSSSSSSSSss",
        ],
        DelveAsset::SealedGate => &[
            "SSSSSSSSSSSSSSSS",
            "SsvvvvvvvvvvvvsS",
            "SsrsrsrsrsrsrsSS",
            "SsrsrsrsrsrsrsSS",
            "SsrsrsTTrsrsrsSS",
            "SsrsrstTrsrsrsSS",
            "SsrsrsrsrsrsrsSS",
            "SsrsrsrsrsrsrsSS",
            "SsvvvvvvvvvvvvsS",
            "SSSSSSSSSSSSSSSS",
        ],
        DelveAsset::ExitLanding => &[
            "ssSSSSSSSSSSss",
            "sSFFFFFFFFFFSs",
            "SFFVFFFFFFVFFS",
            "SFFFVFFFFVFFFS",
            "SFFFFVFFVFFFFS",
            "SFFFFFVVFFFFFS",
            "sSFFFFFFFFFFSs",
            "ssSSSSSSSSSSss",
        ],
        DelveAsset::Camp => &[
            "....oooooo....",
            "..ooMMMMMMoo..",
            ".oMMMAxAMMMMo.",
            "oMMMMxxxMMMMMo",
            "oMMMxxxxxMMMMo",
            ".oMMMAxAMMMMo.",
            "..ooororooo...",
            "....oooooo....",
        ],
        DelveAsset::Torch => &[
            "..x...", ".xxx..", ".AxA..", "..A...", "..r...", "..r...", ".rrr..",
        ],
        DelveAsset::Brazier => &[
            "...x.x...",
            "..xxxxx..",
            ".xAxAxAx.",
            "..AAAAA..",
            ".rrrrrrr.",
            "..rrrrr..",
            "...r.r...",
            "..rr.rr..",
        ],
        DelveAsset::RuneStones => &[
            "..vvv...vvv..",
            ".vVTVv.vVTVv.",
            ".vTtVv.vTtVv.",
            ".vvVvv.vvVvv.",
            "..sss...sss..",
            ".sssss.sssss.",
        ],
        DelveAsset::Roots => &[
            "m.........m.",
            ".m.......mm.",
            "..mm...mm...",
            "...m..mm....",
            "...mmm.m....",
            "..mm.mmm....",
            ".mm..m.mm...",
            "mm....m..mm.",
        ],
        DelveAsset::Columns => &[
            "sSSSSSSs", "SLSSSSLS", ".SSSSSS.", ".sSSSSs.", ".sSSSSs.", ".sSSSSs.", ".sSSSSs.",
            ".sSSSSs.", ".sSSSSs.", "sSSSSSSs", "SSLLLLSS",
        ],
        DelveAsset::Rubble => &[
            "......s.....",
            "..s..sSs....",
            ".sSs.SLLs.s.",
            "sLLSssSSsSs.",
            "SSSSSSLLLLSs",
        ],
        DelveAsset::Puddles => &[
            "....wwww....",
            "..wwWwwwWw..",
            ".wWwwwwwwWw.",
            "..wwwwWwww..",
            "....wwww....",
        ],
        DelveAsset::Bones => &[
            "b.........b.",
            ".b.......bb.",
            "..bbbbbbb...",
            ".bb....bbb..",
            "b.........b.",
        ],
        DelveAsset::Chests => &[
            ".oooooooooo.",
            "oggggggggggo",
            "ogrrrrrrrrgo",
            "ogrrggrrrrgo",
            "ogrrrrrrrrgo",
            ".oooooooooo.",
        ],
        DelveAsset::DungeonClutter => &[
            "o..b...v..r.",
            ".o.bb.vV.rr.",
            "ooo..vvV..rr",
            "rroosssvvvrr",
            "ssSSSssssSSs",
        ],
    }
}
