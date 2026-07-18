use std::sync::OnceLock;

use crate::scene::sprite::SpriteFrame;

use super::{
    IndexedPaletteEntry, indexed_sprite,
    palette::{
        AMBER_LIGHT, ASH_HIGHLIGHT, BRASS, BRASS_DARK, BRASS_LIGHT, EMBER, FLAME, INK_BLUE, MOSS,
        OAK, OAK_DARK, OAK_LIGHT, PARCHMENT, PARCHMENT_DARK, PARCHMENT_LIGHT, RUG, RUG_DARK,
        RUG_GOLD, SHADOW, STEEL, STONE, STONE_DARK, STONE_LIGHT, VOID, WINE, WINE_DARK, WINE_LIGHT,
    },
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GuildHallAsset {
    AshlarWall,
    TimberBeam,
    PlankFloor,
    Rug,
    GuildDoor,
    QuestMapWall,
    CampaignTable,
    Chair,
    CounselBell,
    Hearth,
    SpoilsBench,
    Shelf,
    Banner,
    Candle,
    Mug,
    Dice,
    WaxSeal,
    Scroll,
    Clutter,
}

impl GuildHallAsset {
    pub const ALL: &'static [Self] = &[
        Self::AshlarWall,
        Self::TimberBeam,
        Self::PlankFloor,
        Self::Rug,
        Self::GuildDoor,
        Self::QuestMapWall,
        Self::CampaignTable,
        Self::Chair,
        Self::CounselBell,
        Self::Hearth,
        Self::SpoilsBench,
        Self::Shelf,
        Self::Banner,
        Self::Candle,
        Self::Mug,
        Self::Dice,
        Self::WaxSeal,
        Self::Scroll,
        Self::Clutter,
    ];
}

const PALETTE: &[IndexedPaletteEntry] = &[
    entry('d', STONE_DARK),
    entry('m', STONE),
    entry('l', STONE_LIGHT),
    entry('h', ASH_HIGHLIGHT),
    entry('o', OAK_DARK),
    entry('O', OAK),
    entry('L', OAK_LIGHT),
    entry('r', RUG_DARK),
    entry('R', RUG),
    entry('g', RUG_GOLD),
    entry('p', PARCHMENT_DARK),
    entry('P', PARCHMENT),
    entry('q', PARCHMENT_LIGHT),
    entry('s', SHADOW),
    entry('v', VOID),
    entry('a', AMBER_LIGHT),
    entry('e', EMBER),
    entry('f', FLAME),
    entry('t', STEEL),
    entry('b', BRASS_DARK),
    entry('B', BRASS),
    entry('i', BRASS_LIGHT),
    entry('w', WINE_DARK),
    entry('W', WINE),
    entry('x', WINE_LIGHT),
    entry('n', INK_BLUE),
    entry('M', MOSS),
];

const fn entry(key: char, colour: crate::scene::pixel::Rgb) -> IndexedPaletteEntry {
    IndexedPaletteEntry {
        key,
        colour: Some(colour),
    }
}

#[must_use]
pub fn frame(asset: GuildHallAsset) -> &'static SpriteFrame {
    static FRAMES: OnceLock<Vec<SpriteFrame>> = OnceLock::new();
    &FRAMES.get_or_init(build_frames)[asset as usize]
}

fn build_frames() -> Vec<SpriteFrame> {
    GuildHallAsset::ALL
        .iter()
        .copied()
        .map(|asset| {
            indexed_sprite(rows(asset), PALETTE).expect("built-in Guild Hall asset is valid")
        })
        .collect()
}

#[allow(
    clippy::too_many_lines,
    reason = "the original indexed asset atlas is kept together"
)]
const fn rows(asset: GuildHallAsset) -> &'static [&'static str] {
    match asset {
        GuildHallAsset::AshlarWall => &[
            "ddddmmmmllll",
            "dddmmmmllllh",
            "mmmmllllhddd",
            "mmmllllhdddd",
            "llllhddddmmm",
            "lllhddddmmmm",
        ],
        GuildHallAsset::TimberBeam => &[
            "oooooooooooo",
            "oOOOOOOOOOOo",
            "oOLLOOOLLOOo",
            "oOOOOOOOOOOo",
            "oooooooooooo",
        ],
        GuildHallAsset::PlankFloor => &[
            "ooOOOOOOLLLL",
            "oOOOOOLLLLoo",
            "OOOOOLLLLooO",
            "OOOLLLLooOOO",
            "OLLLLooOOOOO",
            "LLLooOOOOOLL",
        ],
        GuildHallAsset::Rug => &[
            "gggggggggggggggg",
            "grrrrrrrrrrrrrrg",
            "grRWRRWRRWRRWRrg",
            "grWRRWRRWRRWRRrg",
            "grRWRRWRRWRRWRrg",
            "grWRRWRRWRRWRRrg",
            "grrrrrrrrrrrrrrg",
            "gggggggggggggggg",
        ],
        GuildHallAsset::GuildDoor => &[
            "oooooooooooooooooo",
            "oLLLLLLLLLLLLLLLLo",
            "oLOOOOOOOOOOOOOOLo",
            "oLOooooooooooooOLo",
            "oLOoOOOOOOOOOOoOLo",
            "oLOoOOOOOOOOOOoOLo",
            "oLOoOOOooOOOOOoOLo",
            "oLOoOOOOOOOOOOoOLo",
            "oLOoOOOOOOOOOOoOLo",
            "oLOoOOOOOOOOOOoOLo",
            "oLOoOOOOOOOOOOoOLo",
            "oLOoOOOOOOOOOOoOLo",
            "oLOoOOOOOOOOOOoOLo",
            "oLOoOOOOOOOOOOoOLo",
            "oLOoOOOOOOOOOOoOLo",
            "oLOoOOOOOOOOOOoOLo",
            "oLOoOOOOOOOOOOoOLo",
            "oLOoOOOOOOOOOOoOLo",
            "oLOoOOOOOOOOOOoOLo",
            "oLOoOOOOOOOOOOoOLo",
            "oLOoOOOOOOOOOOoOLo",
            "oLOoOOOOOOOOOOoOLo",
            "oLOoOOOOOOOOOOoOLo",
            "oLOooooooooooooOLo",
            "oLOOOOOOOOOOOOOOLo",
            "oLLLLLLLLLLLLLLLLo",
            "oooooooooooooooooo",
        ],
        GuildHallAsset::QuestMapWall => &[
            "ooooooooooooooooooooooooooooo",
            "oLLLLLLLLLLLLLLLLLLLLLLLLLLLo",
            "oLPPPPPPPPPPPPPPPPPPPPPPPPPLo",
            "oLPqPPPPPPPPPPPPPPPPPPPPqPPLo",
            "oLPPnnPPPPPPPPPPPPnnnPPPPPPLo",
            "oLPPPnPPPPppPPPPPPnPnPPPPPPLo",
            "oLPPPPPPppPPPPPPPPnnnPPPPPPLo",
            "oLPPPPppPPPPPPPPPPPPPPPPPPPLo",
            "oLPPppPPPPPPPPPPPPPMMPPPPPPLo",
            "oLPPPPPPPPPPPPPPPMMMPPPPPPPLo",
            "oLPPPPPPPPPPPPPPPPMMPPPPPPPlo",
            "oLPPPPPPPPPPPPPPPPPPPPPPPPPlo",
            "oLPPPPPPPPPPPPPPPPPPPPPPPPPlo",
            "oLPPPPPPPPPPPPPPPPPPPPPPPPPlo",
            "oLLLLLLLLLLLLLLLLLLLLLLLLLLLo",
            "ooooooooooooooooooooooooooooo",
        ],
        GuildHallAsset::CampaignTable => &[
            "..oooooooooooooooooooooooo..",
            ".oLLLLLLLLLLLLLLLLLLLLLLLLo.",
            "oLOOOOOOOOOOOOOOOOOOOOOOOOLo",
            "oLOOOPPPPPPOOOOPPPPPPOOOOOLo",
            "oLOOOPnPnPPOOOOPnPnPPOOOOOLo",
            "oLOOOOOOOOOOOOOOOOOOOOOOOOLo",
            ".oOOOOOOOOOOOOOOOOOOOOOOOOo.",
            "..oooooooooooooooooooooooo..",
            "....oo................oo....",
            "....oo................oo....",
            "....oo................oo....",
            "...oooo..............oooo...",
        ],
        GuildHallAsset::Chair => &[".oooo.", "oLOOLo", "oLOOLo", ".oOOo.", "..oo..", ".oooo."],
        GuildHallAsset::CounselBell => &[
            "....ii....",
            "...iBBi...",
            "....BB....",
            "...BBBB...",
            "..BBBBBB..",
            ".BBBBBBBB.",
            "BBBBBBBBBB",
            "bBBBBBBBBb",
            "..bBBBBb..",
            "....bb....",
            "...oooo...",
            "..oLOOLo..",
        ],
        GuildHallAsset::Hearth => &[
            "dddddddddddddddddddddddd",
            "dlllllllllllllllllllllld",
            "dlmmmmmmmmmmmmmmmmmmmmld",
            "dlmddddddddddddddddddmld",
            "dlmdssssssssssssssssdmld",
            "dlmdssssssssssssssssdmld",
            "dlmdsssssffffsssssssdmld",
            "dlmdssssffffffssssssdmld",
            "dlmdssseffffefssssssdmld",
            "dlmdsssefeeeefssssssdmld",
            "dlmdssseeeeeefssssssdmld",
            "dlmdsssseeffeeesssssdmld",
            "dlmdssseeeeeesssssssdmld",
            "dlmdssssssssssssssssdmld",
            "dlmddddddddddddddddddmld",
            "dlllllllllllllllllllllld",
            "dddddddddddddddddddddddd",
        ],
        GuildHallAsset::SpoilsBench => &[
            "oooooooooooooooooooooooooooooooo",
            "oLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLo",
            "oLOOOOOOOOOOOOOOOOOOOOOOOOOOOOLo",
            "oLOOOwwwwwwwwwwwwwwwwwwwwOOOOOLo",
            "oLOOOwxxxxxxxxxxxxxxxxxxxxwOOOLo",
            "oLOOOwxxBBBBxxxxBBBBxxxxxxwOOOLo",
            "oLOOOwxxxxxxxxxxxxxxxxxxxxwOOOLo",
            "oLOOOwwwwwwwwwwwwwwwwwwwwOOOOOLo",
            "oLOOOOOOOOOOOOOOOOOOOOOOOOOOOOLo",
            "oLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLo",
            "oooooooooooooooooooooooooooooooo",
            "...oooo..................oooo...",
            "...oooo..................oooo...",
        ],
        GuildHallAsset::Shelf => &[
            "oooooooooooooooooooo",
            "oLnnWPPnMMWnPPnnMMLo",
            "oLnnWPPnMMWnPPnnMMLo",
            "oooooooooooooooooooo",
            "oLPPnnWMMnPPWnnPMMLo",
            "oLPPnnWMMnPPWnnPMMLo",
            "oooooooooooooooooooo",
        ],
        GuildHallAsset::Banner => &[
            "wwwwwwwwww",
            "wxxxxxxxxw",
            "wxxggggxxw",
            "wxxgWWgxxw",
            "wxxggggxxw",
            "wxxxxxxxxw",
            ".wxxxxxxw.",
            "..wxxxxw..",
            "...wxxw...",
        ],
        GuildHallAsset::Candle => &[".a.", ".f.", ".i.", ".i.", "bbb"],
        GuildHallAsset::Mug => &[".OO.", "OLL.", "OLLo", ".oo."],
        GuildHallAsset::Dice => &["qiq", "iBi", "qiq"],
        GuildHallAsset::WaxSeal => &[".x.", "xWx", ".w."],
        GuildHallAsset::Scroll => &["pppppp", "PqPPqP", "PPPPPP", "pppppp"],
        GuildHallAsset::Clutter => &["B.t.g.M", ".i.t.M.", "g...B.t"],
    }
}
