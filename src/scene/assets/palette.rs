use crate::domain::{AccentTone, AdventurerClass, Garb, HairTone, SkinTone};

use super::super::pixel::Rgb;

pub const VOID: Rgb = Rgb::new(13, 12, 18);
pub const SHADOW: Rgb = Rgb::new(24, 22, 30);
pub const STONE_DARK: Rgb = Rgb::new(48, 45, 54);
pub const STONE: Rgb = Rgb::new(75, 70, 74);
pub const STONE_LIGHT: Rgb = Rgb::new(105, 96, 92);
pub const OAK_DARK: Rgb = Rgb::new(59, 36, 29);
pub const OAK: Rgb = Rgb::new(104, 63, 37);
pub const OAK_LIGHT: Rgb = Rgb::new(151, 93, 48);
pub const RUG_DARK: Rgb = Rgb::new(76, 25, 35);
pub const RUG: Rgb = Rgb::new(130, 43, 48);
pub const RUG_GOLD: Rgb = Rgb::new(196, 126, 48);
pub const FLAME: Rgb = Rgb::new(255, 188, 70);
pub const EMBER: Rgb = Rgb::new(225, 82, 33);
pub const STEEL: Rgb = Rgb::new(151, 164, 171);
pub const EYE: Rgb = Rgb::new(22, 19, 24);
pub const PARCHMENT_DARK: Rgb = Rgb::new(174, 151, 105);
pub const PARCHMENT: Rgb = Rgb::new(230, 207, 154);
pub const PARCHMENT_LIGHT: Rgb = Rgb::new(248, 232, 184);
pub const AMBER_LIGHT: Rgb = Rgb::new(248, 196, 92);
pub const BRASS_DARK: Rgb = Rgb::new(112, 75, 34);
pub const BRASS: Rgb = Rgb::new(181, 128, 49);
pub const BRASS_LIGHT: Rgb = Rgb::new(236, 184, 76);
pub const WINE_DARK: Rgb = Rgb::new(67, 22, 31);
pub const WINE: Rgb = Rgb::new(126, 42, 48);
pub const WINE_LIGHT: Rgb = Rgb::new(177, 64, 63);
pub const INK_BLUE: Rgb = Rgb::new(48, 69, 91);
pub const MOSS: Rgb = Rgb::new(60, 83, 55);
pub const ASH_HIGHLIGHT: Rgb = Rgb::new(132, 120, 111);
pub const SELECTION_RUNE: Rgb = Rgb::new(69, 229, 255);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdventurerPalette {
    pub skin: Rgb,
    pub hair: Rgb,
    pub cloth: Rgb,
    pub metal: Rgb,
    pub accent: Rgb,
    pub eye: Rgb,
}

pub const fn adventurer_palette(
    skin: SkinTone,
    hair: HairTone,
    garb: Garb,
    class: AdventurerClass,
    accent: AccentTone,
) -> AdventurerPalette {
    AdventurerPalette {
        skin: skin_colour(skin),
        hair: hair_colour(hair),
        cloth: cloth_colour(garb),
        metal: metal_colour(class),
        accent: accent_colour(accent),
        eye: EYE,
    }
}

const fn skin_colour(value: SkinTone) -> Rgb {
    match value {
        SkinTone::Porcelain => Rgb::new(244, 205, 178),
        SkinTone::Rose => Rgb::new(224, 169, 146),
        SkinTone::Sand => Rgb::new(198, 141, 105),
        SkinTone::Umber => Rgb::new(126, 82, 61),
        SkinTone::Sienna => Rgb::new(153, 91, 65),
        SkinTone::Ebony => Rgb::new(83, 55, 49),
    }
}

const fn hair_colour(value: HairTone) -> Rgb {
    match value {
        HairTone::Black => Rgb::new(28, 24, 30),
        HairTone::Espresso => Rgb::new(58, 37, 32),
        HairTone::Chestnut => Rgb::new(105, 56, 38),
        HairTone::Copper => Rgb::new(166, 73, 39),
        HairTone::Gold => Rgb::new(213, 163, 69),
        HairTone::Silver => Rgb::new(180, 183, 184),
    }
}

const fn cloth_colour(value: Garb) -> Rgb {
    match value {
        Garb::Armour => Rgb::new(79, 93, 105),
        Garb::Cloak => Rgb::new(48, 79, 80),
        Garb::Doublet => Rgb::new(101, 42, 57),
        Garb::Leathers => Rgb::new(104, 65, 40),
        Garb::Robes => Rgb::new(64, 51, 112),
        Garb::Vestments => Rgb::new(175, 150, 92),
        Garb::WorkApron => Rgb::new(94, 75, 56),
    }
}

const fn metal_colour(value: AdventurerClass) -> Rgb {
    match value {
        AdventurerClass::Cleric | AdventurerClass::Paladin => Rgb::new(189, 183, 145),
        AdventurerClass::Wizard | AdventurerClass::Runewright => Rgb::new(111, 139, 179),
        AdventurerClass::Druid => MOSS,
        AdventurerClass::Rogue | AdventurerClass::Ranger | AdventurerClass::Pathseeker => {
            Rgb::new(117, 108, 87)
        }
        AdventurerClass::Barbarian
        | AdventurerClass::Bard
        | AdventurerClass::Artificer
        | AdventurerClass::Testmender => STEEL,
    }
}

const fn accent_colour(value: AccentTone) -> Rgb {
    match value {
        AccentTone::Amber => Rgb::new(227, 150, 47),
        AccentTone::Cyan => Rgb::new(48, 183, 190),
        AccentTone::Lime => Rgb::new(137, 188, 73),
        AccentTone::Magenta => Rgb::new(190, 68, 143),
        AccentTone::Red => Rgb::new(195, 57, 55),
        AccentTone::Blue => Rgb::new(64, 105, 190),
        AccentTone::Violet => Rgb::new(130, 82, 185),
        AccentTone::Teal => Rgb::new(48, 139, 127),
    }
}
