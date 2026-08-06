//! Authored 8x12 roster masters.
//!
//! The responsive contract recomposes rather than scaling, so a pane too
//! narrow for a 16x24 party gets its own authored size instead of a squeezed
//! world master. These are deliberately one master per *silhouette family*
//! rather than one per class: at eight pixels wide, class gear is the only
//! thing a silhouette can carry, and persona palette substitution supplies the
//! per-adventurer identity that the family cannot.
//!
//! Pose is not authored at this scale. A roster adventurer's state is carried
//! by its grounding, counsel marker and nameplate, never by the sprite.

use std::sync::OnceLock;

use crate::{domain::AdventurerClass, scene::pixel::Rgb, scene::sprite::SpriteFrame};

use super::{IndexedPaletteEntry, indexed_sprite};

pub const WIDTH: u16 = 8;
pub const HEIGHT: u16 = 12;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RosterFamily {
    Caster,
    Armoured,
    Brute,
    Skirmisher,
    Ranger,
}

impl RosterFamily {
    pub const ALL: &'static [Self] = &[
        Self::Caster,
        Self::Armoured,
        Self::Brute,
        Self::Skirmisher,
        Self::Ranger,
    ];
}

/// Routes a class to the silhouette family whose authored gear reads closest
/// to its world master.
#[must_use]
pub const fn family_for(class: AdventurerClass) -> RosterFamily {
    match class {
        AdventurerClass::Wizard
        | AdventurerClass::Artificer
        | AdventurerClass::Runewright
        | AdventurerClass::Druid => RosterFamily::Caster,
        AdventurerClass::Cleric | AdventurerClass::Paladin | AdventurerClass::Testmender => {
            RosterFamily::Armoured
        }
        AdventurerClass::Barbarian => RosterFamily::Brute,
        AdventurerClass::Rogue | AdventurerClass::Bard => RosterFamily::Skirmisher,
        AdventurerClass::Ranger | AdventurerClass::Pathseeker => RosterFamily::Ranger,
    }
}

// Shared material roles. Only the cloth pair and the outline tint change per
// family, so the party reads as one guild while the gear silhouette separates
// the families.
//
// The outline is *tinted*, not black. At eight pixels wide a one-pixel border
// is roughly half the sprite, so a near-black outline turns every adventurer
// into a dark rectangle against the Hall floor; a dark tint of the family's
// own cloth reads as a rim instead.
const SKIN_SHADOW: Rgb = Rgb::new(139, 82, 52);
const SKIN_BASE: Rgb = Rgb::new(232, 164, 103);
const SKIN_HIGHLIGHT: Rgb = Rgb::new(255, 224, 164);
const HAIR_SHADOW: Rgb = Rgb::new(58, 37, 32);
const HAIR_BASE: Rgb = Rgb::new(105, 56, 38);
const METAL_SHADOW: Rgb = Rgb::new(122, 131, 138);
const METAL_LIGHT: Rgb = Rgb::new(214, 224, 220);
const LEATHER_SHADOW: Rgb = Rgb::new(92, 55, 34);
const LEATHER_BASE: Rgb = Rgb::new(157, 93, 45);
const TRIM: Rgb = Rgb::new(237, 181, 77);
const ACCENT: Rgb = Rgb::new(227, 150, 47);
const FOCAL: Rgb = Rgb::new(112, 220, 255);

macro_rules! roster_palette {
    ($outline:expr, $cloth_shadow:expr, $cloth_base:expr) => {
        &[
            IndexedPaletteEntry {
                key: 'o',
                colour: Some($outline),
            },
            IndexedPaletteEntry {
                key: 'k',
                colour: Some(SKIN_SHADOW),
            },
            IndexedPaletteEntry {
                key: 'K',
                colour: Some(SKIN_BASE),
            },
            IndexedPaletteEntry {
                key: 'h',
                colour: Some(SKIN_HIGHLIGHT),
            },
            IndexedPaletteEntry {
                key: 'r',
                colour: Some(HAIR_SHADOW),
            },
            IndexedPaletteEntry {
                key: 'R',
                colour: Some(HAIR_BASE),
            },
            IndexedPaletteEntry {
                key: 'c',
                colour: Some($cloth_shadow),
            },
            IndexedPaletteEntry {
                key: 'C',
                colour: Some($cloth_base),
            },
            IndexedPaletteEntry {
                key: 'l',
                colour: Some(TRIM),
            },
            IndexedPaletteEntry {
                key: 'm',
                colour: Some(METAL_SHADOW),
            },
            IndexedPaletteEntry {
                key: 'M',
                colour: Some(METAL_LIGHT),
            },
            IndexedPaletteEntry {
                key: 'd',
                colour: Some(LEATHER_SHADOW),
            },
            IndexedPaletteEntry {
                key: 'D',
                colour: Some(LEATHER_BASE),
            },
            IndexedPaletteEntry {
                key: 'a',
                colour: Some(ACCENT),
            },
            IndexedPaletteEntry {
                key: 'e',
                colour: Some(FOCAL),
            },
        ]
    };
}

const CASTER_PALETTE: &[IndexedPaletteEntry] = roster_palette!(
    Rgb::new(30, 25, 58),
    Rgb::new(60, 49, 126),
    Rgb::new(105, 81, 180)
);
const ARMOURED_PALETTE: &[IndexedPaletteEntry] = roster_palette!(
    Rgb::new(25, 36, 57),
    Rgb::new(48, 74, 120),
    Rgb::new(82, 120, 175)
);
const BRUTE_PALETTE: &[IndexedPaletteEntry] = roster_palette!(
    Rgb::new(45, 21, 23),
    Rgb::new(92, 42, 44),
    Rgb::new(140, 66, 58)
);
const SKIRMISHER_PALETTE: &[IndexedPaletteEntry] = roster_palette!(
    Rgb::new(27, 20, 39),
    Rgb::new(52, 38, 78),
    Rgb::new(98, 65, 137)
);
const RANGER_PALETTE: &[IndexedPaletteEntry] = roster_palette!(
    Rgb::new(16, 28, 15),
    Rgb::new(26, 47, 26),
    Rgb::new(79, 125, 57)
);

// Every family tapers: a head narrower than its shoulders and legs parted by
// negative space. Without that the one-pixel outline closes into a rectangle
// and the whole party reads as a row of boxes rather than adventurers.

// Pointed hat, beard and a staff held clear of the body.
#[rustfmt::skip]
const CASTER: &[&str] = &[
    "...oo...",
    "..occo..",
    ".occCCo.",
    "..oKKo..",
    "..oKho.m",
    "..orro.d",
    ".ocCCcod",
    "ocCCCCod",
    "ocCaCCod",
    ".ocCCco.",
    ".od..do.",
    ".oo..oo.",
];

// Helm and pauldrons: the widest shoulders in the party, and no held weapon.
#[rustfmt::skip]
const ARMOURED: &[&str] = &[
    "........",
    "..oooo..",
    ".omMMmo.",
    "..oKKo..",
    "..oKho..",
    ".ocCCco.",
    "omCCCCmo",
    "oMcCaCMo",
    ".ocCCco.",
    ".ocCCco.",
    ".od..do.",
    ".oo..oo.",
];

// Spiked hair, bare shoulders and an axe head breaking the left silhouette.
#[rustfmt::skip]
const BRUTE: &[&str] = &[
    "........",
    "..oRRo..",
    ".oRrrRo.",
    "..oKKo..",
    "..oKho..",
    "..orro..",
    "MoKKKKo.",
    "MMcCCco.",
    "moCaCCo.",
    ".ocCCco.",
    ".od..do.",
    ".oo..oo.",
];

// Deep hood shadowing the face, with paired daggers held wide.
#[rustfmt::skip]
const SKIRMISHER: &[&str] = &[
    "........",
    "..oooo..",
    ".occCCo.",
    ".ocKKco.",
    "..okko..",
    "..occo..",
    "mocCaCom",
    "MocCCCoM",
    ".ocCCco.",
    ".ocCCco.",
    ".od..do.",
    ".oo..oo.",
];

// Half hood and a bow standing the full height of the right edge.
#[rustfmt::skip]
const RANGER: &[&str] = &[
    "........",
    "..oooo.d",
    ".occCcod",
    "..oKKo.d",
    "..oKho.d",
    ".ocCCcod",
    "ocCCCCod",
    "ocCaCCod",
    ".ocCCco.",
    ".ocCCco.",
    ".od..do.",
    ".oo..oo.",
];

/// Returns the authored roster master and its palette, so persona
/// substitution can locate role colours without duplicating the routing.
#[must_use]
pub fn master(family: RosterFamily) -> (SpriteFrame, &'static [IndexedPaletteEntry]) {
    let (cell, rows, palette) = match family {
        RosterFamily::Caster => (&CASTER_FRAME, CASTER, CASTER_PALETTE),
        RosterFamily::Armoured => (&ARMOURED_FRAME, ARMOURED, ARMOURED_PALETTE),
        RosterFamily::Brute => (&BRUTE_FRAME, BRUTE, BRUTE_PALETTE),
        RosterFamily::Skirmisher => (&SKIRMISHER_FRAME, SKIRMISHER, SKIRMISHER_PALETTE),
        RosterFamily::Ranger => (&RANGER_FRAME, RANGER, RANGER_PALETTE),
    };
    let frame = cell
        .get_or_init(|| indexed_sprite(rows, palette).expect("authored roster master is valid"))
        .clone();
    (frame, palette)
}

static CASTER_FRAME: OnceLock<SpriteFrame> = OnceLock::new();
static ARMOURED_FRAME: OnceLock<SpriteFrame> = OnceLock::new();
static BRUTE_FRAME: OnceLock<SpriteFrame> = OnceLock::new();
static SKIRMISHER_FRAME: OnceLock<SpriteFrame> = OnceLock::new();
static RANGER_FRAME: OnceLock<SpriteFrame> = OnceLock::new();
