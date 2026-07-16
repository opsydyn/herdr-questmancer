use crate::{
    domain::{
        AccentTone, Footwear, Garb, HairTone, Keepsake, Legwear, PersonaAppearance, SkinTone,
    },
    ui::pixel::{ColorRole, Palette},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AppearanceRoles {
    pub skin: ColorRole,
    pub hair: ColorRole,
    pub garb: ColorRole,
    pub legwear: ColorRole,
    pub footwear: ColorRole,
    pub keepsake: ColorRole,
    pub accent: ColorRole,
    pub highlight: ColorRole,
    pub shadow: ColorRole,
}

pub fn appearance_roles(appearance: &PersonaAppearance) -> AppearanceRoles {
    AppearanceRoles {
        skin: skin_role(appearance.skin_tone),
        hair: hair_role(appearance.hair_tone),
        garb: garb_role(appearance.garb),
        legwear: legwear_role(appearance.legwear),
        footwear: footwear_role(appearance.footwear),
        keepsake: keepsake_role(appearance.keepsake),
        accent: accent_role(appearance.accent),
        highlight: ColorRole::Parchment,
        shadow: ColorRole::Ink,
    }
}

pub fn appearance_roles_for_palette(
    appearance: &PersonaAppearance,
    palette: Palette,
) -> AppearanceRoles {
    let canonical = appearance_roles(appearance);
    let skin = canonical.skin;
    let hair = contrasting(canonical.hair, &[skin, ColorRole::DarkStone], palette);
    let garb = contrasting(canonical.garb, &[skin, hair], palette);
    let legwear = contrasting(canonical.legwear, &[garb], palette);
    let footwear = contrasting(
        canonical.footwear,
        &[legwear, ColorRole::DarkStone],
        palette,
    );
    let keepsake = contrasting(canonical.keepsake, &[garb, skin, hair], palette);
    let accent = contrasting(canonical.accent, &[garb, skin, hair, keepsake], palette);

    AppearanceRoles {
        skin,
        hair,
        garb,
        legwear,
        footwear,
        keepsake,
        accent,
        highlight: ColorRole::Parchment,
        shadow: ColorRole::Ink,
    }
}

fn contrasting(preferred: ColorRole, neighbours: &[ColorRole], palette: Palette) -> ColorRole {
    if contrasts_with_all(preferred, neighbours, palette) {
        return preferred;
    }

    CONTRAST_FALLBACKS
        .iter()
        .copied()
        .find(|candidate| contrasts_with_all(*candidate, neighbours, palette))
        .unwrap_or(preferred)
}

fn contrasts_with_all(role: ColorRole, neighbours: &[ColorRole], palette: Palette) -> bool {
    neighbours
        .iter()
        .all(|neighbour| palette.roles_contrast(role, *neighbour))
}

const CONTRAST_FALLBACKS: [ColorRole; 13] = [
    ColorRole::ClothCool,
    ColorRole::ClothWarm,
    ColorRole::Leather,
    ColorRole::Steel,
    ColorRole::Moss,
    ColorRole::Counsel,
    ColorRole::RuneGlow,
    ColorRole::Spoils,
    ColorRole::Selection,
    ColorRole::Goblin,
    ColorRole::Fog,
    ColorRole::Parchment,
    ColorRole::Ink,
];

const fn skin_role(tone: SkinTone) -> ColorRole {
    match tone {
        SkinTone::Porcelain | SkinTone::Rose => ColorRole::SkinLight,
        SkinTone::Sand | SkinTone::Umber => ColorRole::SkinMedium,
        SkinTone::Sienna | SkinTone::Ebony => ColorRole::SkinDark,
    }
}

const fn hair_role(tone: HairTone) -> ColorRole {
    match tone {
        HairTone::Black | HairTone::Espresso | HairTone::Chestnut => ColorRole::HairDark,
        HairTone::Copper | HairTone::Gold | HairTone::Silver => ColorRole::HairLight,
    }
}

const fn garb_role(garb: Garb) -> ColorRole {
    match garb {
        Garb::Armour => ColorRole::Steel,
        Garb::Cloak | Garb::Robes => ColorRole::ClothCool,
        Garb::Doublet | Garb::Vestments => ColorRole::ClothWarm,
        Garb::Leathers | Garb::WorkApron => ColorRole::Leather,
    }
}

const fn legwear_role(legwear: Legwear) -> ColorRole {
    match legwear {
        Legwear::BootsAndBreeches => ColorRole::ClothCool,
        Legwear::Greaves => ColorRole::Steel,
        Legwear::RobeHem => ColorRole::ClothWarm,
        Legwear::TravelingSkirt => ColorRole::Leather,
    }
}

const fn footwear_role(footwear: Footwear) -> ColorRole {
    match footwear {
        Footwear::Boots | Footwear::SoftShoes | Footwear::Sandals => ColorRole::Leather,
        Footwear::Sabatons => ColorRole::Steel,
    }
}

const fn keepsake_role(keepsake: Keepsake) -> ColorRole {
    match keepsake {
        Keepsake::Feather => ColorRole::Parchment,
        Keepsake::LuckyCoin => ColorRole::Spoils,
        Keepsake::Mug => ColorRole::Hearth,
        Keepsake::PressedLeaf => ColorRole::Moss,
        Keepsake::Ribbon => ColorRole::Counsel,
        Keepsake::TinyFamiliar => ColorRole::Goblin,
    }
}

const fn accent_role(accent: AccentTone) -> ColorRole {
    match accent {
        AccentTone::Amber => ColorRole::Counsel,
        AccentTone::Cyan => ColorRole::RuneGlow,
        AccentTone::Lime => ColorRole::Moss,
        AccentTone::Magenta => ColorRole::Spoils,
        AccentTone::Red => ColorRole::Hearth,
        AccentTone::Blue => ColorRole::Selection,
        AccentTone::Violet => ColorRole::Fog,
        AccentTone::Teal => ColorRole::Goblin,
    }
}
