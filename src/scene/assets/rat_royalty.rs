//! The Gutter Crown, an opt-in adventurer with a crown, sceptre and velvet coat.

use std::sync::OnceLock;

use crate::scene::{pixel::Rgb, sprite::SpriteFrame};

use super::{IndexedPaletteEntry, indexed_sprite};

pub(super) const PALETTE: &[IndexedPaletteEntry] = &[
    IndexedPaletteEntry {
        key: 'o',
        colour: Some(Rgb::new(37, 22, 30)),
    },
    IndexedPaletteEntry {
        key: 'g',
        colour: Some(Rgb::new(170, 108, 36)),
    },
    IndexedPaletteEntry {
        key: 'G',
        colour: Some(Rgb::new(246, 193, 83)),
    },
    IndexedPaletteEntry {
        key: 'B',
        colour: Some(Rgb::new(98, 158, 237)),
    },
    IndexedPaletteEntry {
        key: 'k',
        colour: Some(Rgb::new(177, 87, 90)),
    },
    IndexedPaletteEntry {
        key: 'P',
        colour: Some(Rgb::new(232, 145, 127)),
    },
    IndexedPaletteEntry {
        key: 'f',
        colour: Some(Rgb::new(100, 74, 71)),
    },
    IndexedPaletteEntry {
        key: 'F',
        colour: Some(Rgb::new(162, 124, 112)),
    },
    IndexedPaletteEntry {
        key: 'E',
        colour: Some(Rgb::new(248, 177, 73)),
    },
    IndexedPaletteEntry {
        key: 'n',
        colour: Some(Rgb::new(209, 116, 109)),
    },
    IndexedPaletteEntry {
        key: 'w',
        colour: Some(Rgb::new(155, 120, 95)),
    },
    IndexedPaletteEntry {
        key: 'W',
        colour: Some(Rgb::new(242, 216, 176)),
    },
    IndexedPaletteEntry {
        key: 'e',
        colour: Some(Rgb::new(217, 149, 55)),
    },
    IndexedPaletteEntry {
        key: 'c',
        colour: Some(Rgb::new(87, 27, 49)),
    },
    IndexedPaletteEntry {
        key: 'C',
        colour: Some(Rgb::new(147, 39, 67)),
    },
    IndexedPaletteEntry {
        key: 'r',
        colour: Some(Rgb::new(198, 70, 91)),
    },
    IndexedPaletteEntry {
        key: 'd',
        colour: Some(Rgb::new(78, 50, 42)),
    },
];

#[rustfmt::skip]
const WORLD: [&str; 24] = [
    "................",
    "....g.G.G.g.....",
    "....gGGGGGg.....",
    "...ogGGBGgo.....",
    "..ooogGGgooo....",
    ".okkkoffokko....",
    ".okPofFFfokoo...",
    "..oooffEFFFo.go.",
    "...offFFFFFnoGBg",
    "...offFffFFwogBg",
    "....offfWWoo.og.",
    "...owWfWWgeo.og.",
    "...oWWowWgCo.og.",
    "..ocCCGCCCrCoPg.",
    "..oCCrCCfrCCokg.",
    "..oCrCCCdCCcoog.",
    ".koCCCrdGCCco.g.",
    "k.oCCrcddCCco...",
    "PkocCcdoCCco....",
    ".kkocgdodgco....",
    "...oodd.ddoo....",
    "....ooo.ooo.....",
    "................",
    "................",
];

#[rustfmt::skip]
pub(super) const ROSTER: &[&str] = &[
    "..gGgG..",
    "..gGBg..",
    ".koffok.",
    ".ofEFFo.",
    "..ofFnog",
    "..owWwoB",
    "..oCGCPg",
    ".koCrCog",
    "k.oCCco.",
    ".kocdco.",
    "..oo.oo.",
    "........",
];

const fn with_rows<const N: usize>(changes: [(usize, &'static str); N]) -> [&'static str; 24] {
    let mut rows = WORLD;
    let mut i = 0;
    while i < N {
        let (row, pixels) = changes[i];
        rows[row] = pixels;
        i += 1;
    }
    rows
}

// Keep the crown, ears and feet anchored across poses. Glint, raised sceptre,
// chest, bent knees and a lowered sceptre carry the state close to the body.
pub(super) const FRAMES: [&[&str]; 8] = [
    &WORLD,
    &with_rows([(3, "...ogGWWGgo....."), (9, "...offFffFFwogWg")]),
    &with_rows([
        (6, ".okPofFFfokoo.go"),
        (7, "..oooffEFFFoogBg"),
        (8, "...offFFFFFnogBg"),
        (9, "...offFffFFwoog."),
    ]),
    &WORLD,
    &with_rows([
        (14, "..oCCoooooCCokg."),
        (15, "..oCCogGgoCcoog."),
        (16, ".koCCogBgoCco.g."),
        (17, "k.oCCoooooCco..."),
    ]),
    &with_rows([
        (18, "PkocCcdoCCCoo..."),
        (19, ".kkoddooddddo..."),
        (20, "...odd...ddo...."),
        (21, "...ooo...ooo...."),
    ]),
    &with_rows([(8, "...offFFFFFnoGgg"), (9, "...offFffFFwoggg")]),
    &WORLD,
];

pub(super) const WALK: [&[&str]; 2] = [
    &with_rows([
        (19, ".kkocgdodgco...."),
        (20, "...odd..ddoo...."),
        (21, "..oooo...oo....."),
    ]),
    &with_rows([
        (19, ".kkocgdodgco...."),
        (20, "....ood.dddo...."),
        (21, ".....oo.ooooo..."),
    ]),
];

/// The authored 16x24 world sprite with feet grounded on row 21.
#[must_use]
pub fn world() -> &'static SpriteFrame {
    static FRAME: OnceLock<SpriteFrame> = OnceLock::new();
    FRAME.get_or_init(|| indexed_sprite(&WORLD, PALETTE).expect("valid Rat Royalty world art"))
}

/// The independently authored 8x12 sprite for crowded compositions.
#[must_use]
pub fn roster() -> &'static SpriteFrame {
    static FRAME: OnceLock<SpriteFrame> = OnceLock::new();
    FRAME.get_or_init(|| indexed_sprite(ROSTER, PALETTE).expect("valid Rat Royalty roster art"))
}

/// The matching 24x32 RGB profile fallback, with the world sprite centred at 1x.
#[must_use]
pub fn profile() -> &'static SpriteFrame {
    const WIDTH: usize = 24;
    const HEIGHT: usize = 32;
    const WORLD_WIDTH: usize = 16;
    const MARGIN: usize = 4;
    static FRAME: OnceLock<SpriteFrame> = OnceLock::new();
    FRAME.get_or_init(|| {
        let mut pixels = vec![None; WIDTH * HEIGHT];
        for (y, row) in world().pixels().chunks_exact(WORLD_WIDTH).enumerate() {
            let left = (y + MARGIN) * WIDTH + MARGIN;
            pixels[left..left + WORLD_WIDTH].copy_from_slice(row);
        }
        SpriteFrame::from_pixels(24, 32, pixels)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::pixel::PixelSize;

    #[test]
    fn the_profile_centres_the_world_sprite_and_the_roster_keeps_its_foot_anchor() {
        let world = world();
        let roster = roster();
        let profile = profile();
        assert_eq!(world.size(), PixelSize::new(16, 24));
        assert_eq!(roster.size(), PixelSize::new(8, 12));
        assert_eq!(profile.size(), PixelSize::new(24, 32));
        for (y, row) in profile.pixels().chunks_exact(24).enumerate() {
            for (x, pixel) in row.iter().enumerate() {
                let expected = if (4..20).contains(&x) && (4..28).contains(&y) {
                    world.pixels()[(y - 4) * 16 + x - 4]
                } else {
                    None
                };
                assert_eq!(*pixel, expected, "profile pixel at ({x}, {y})");
            }
        }
        assert_eq!(
            world
                .pixels()
                .iter()
                .rposition(Option::is_some)
                .map(|i| i / 16),
            Some(21)
        );
        assert_eq!(
            roster
                .pixels()
                .iter()
                .rposition(Option::is_some)
                .map(|i| i / 8),
            Some(10)
        );
    }

    #[test]
    fn the_native_card_has_the_standard_dimensions_and_transparent_margin() {
        let card = image::load_from_memory(include_bytes!(
            "../../assets/portraits/rat-royalty-card.png"
        ))
        .expect("Rat Royalty portrait decodes")
        .to_rgba8();
        assert_eq!(card.dimensions(), (384, 512));
        // A rounded alpha value of 1/255 on the outer edge is imperceptible.
        assert!(
            card.enumerate_pixels()
                .filter(|(x, y, _)| *x == 0 || *x == 383 || *y == 0 || *y == 511)
                .all(|(_, _, pixel)| pixel[3] <= 1)
        );
        assert!(card.pixels().any(|pixel| pixel[3] == 255));
    }
}
