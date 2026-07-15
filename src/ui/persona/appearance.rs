use crate::{
    domain::{
        AccentTone, Footwear, Garb, HairTone, Keepsake, Legwear, PersonaAppearance, SkinTone,
    },
    ui::pixel::{
        AccentShade, ColorRole, FabricShade, FootwearShade, HairShade, Palette, SkinShade,
    },
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
    let skin = ColorRole::SkinTone(skin_shade(appearance.skin_tone));
    let hair = ColorRole::HairTone(hair_shade(appearance.hair_tone));
    let garb = ColorRole::Fabric(garb_shade(appearance.garb));
    let legwear = ColorRole::Fabric(legwear_shade(appearance.legwear));
    let footwear = ColorRole::Footwear(footwear_shade(appearance.footwear));
    let keepsake = ColorRole::AccentTone(keepsake_shade(appearance.keepsake));
    let accent = ColorRole::AccentTone(accent_shade(appearance.accent));

    AppearanceRoles {
        skin,
        hair,
        garb,
        legwear,
        footwear,
        keepsake,
        accent,
        highlight: ColorRole::Highlight,
        shadow: ColorRole::Shadow,
    }
}

pub fn appearance_roles_for_palette(
    appearance: &PersonaAppearance,
    palette: Palette,
) -> AppearanceRoles {
    let canonical = appearance_roles(appearance);
    let skin = canonical.skin;
    let hair = contrasting(
        canonical.hair,
        &[skin, ColorRole::PanelBackground],
        &HAIR_FALLBACKS,
        palette,
    );
    let garb = contrasting(canonical.garb, &[skin, hair], &FABRIC_FALLBACKS, palette);
    let legwear = contrasting(canonical.legwear, &[garb], &FABRIC_FALLBACKS, palette);
    let footwear = contrasting(
        canonical.footwear,
        &[legwear, ColorRole::PanelBackground],
        &FOOTWEAR_FALLBACKS,
        palette,
    );
    let keepsake = contrasting(
        canonical.keepsake,
        &[garb, skin, hair],
        &ACCENT_FALLBACKS,
        palette,
    );
    let accent = contrasting(
        canonical.accent,
        &[garb, skin, hair, keepsake],
        &ACCENT_FALLBACKS,
        palette,
    );

    AppearanceRoles {
        skin,
        hair,
        garb,
        legwear,
        footwear,
        keepsake,
        accent,
        highlight: ColorRole::Highlight,
        shadow: ColorRole::Shadow,
    }
}

fn contrasting(
    preferred: ColorRole,
    neighbours: &[ColorRole],
    fallbacks: &[ColorRole],
    palette: Palette,
) -> ColorRole {
    if contrasts_with_all(preferred, neighbours, palette) {
        return preferred;
    }

    fallbacks
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

const HAIR_FALLBACKS: [ColorRole; 6] = [
    ColorRole::HairTone(HairShade::Black),
    ColorRole::HairTone(HairShade::Espresso),
    ColorRole::HairTone(HairShade::Chestnut),
    ColorRole::HairTone(HairShade::Copper),
    ColorRole::HairTone(HairShade::Gold),
    ColorRole::HairTone(HairShade::Silver),
];
const FABRIC_FALLBACKS: [ColorRole; 8] = [
    ColorRole::Fabric(FabricShade::Navy),
    ColorRole::Fabric(FabricShade::Cobalt),
    ColorRole::Fabric(FabricShade::Teal),
    ColorRole::Fabric(FabricShade::Green),
    ColorRole::Fabric(FabricShade::Mustard),
    ColorRole::Fabric(FabricShade::Orange),
    ColorRole::Fabric(FabricShade::Crimson),
    ColorRole::Fabric(FabricShade::Plum),
];
const FOOTWEAR_FALLBACKS: [ColorRole; 5] = [
    ColorRole::Footwear(FootwearShade::Black),
    ColorRole::Footwear(FootwearShade::Charcoal),
    ColorRole::Footwear(FootwearShade::White),
    ColorRole::Footwear(FootwearShade::Blue),
    ColorRole::Footwear(FootwearShade::Magenta),
];
const ACCENT_FALLBACKS: [ColorRole; 8] = [
    ColorRole::AccentTone(AccentShade::Amber),
    ColorRole::AccentTone(AccentShade::Cyan),
    ColorRole::AccentTone(AccentShade::Lime),
    ColorRole::AccentTone(AccentShade::Magenta),
    ColorRole::AccentTone(AccentShade::Red),
    ColorRole::AccentTone(AccentShade::Blue),
    ColorRole::AccentTone(AccentShade::Violet),
    ColorRole::AccentTone(AccentShade::Teal),
];

const fn skin_shade(tone: SkinTone) -> SkinShade {
    match tone {
        SkinTone::Porcelain => SkinShade::Porcelain,
        SkinTone::Rose => SkinShade::Rose,
        SkinTone::Sand => SkinShade::Sand,
        SkinTone::Umber => SkinShade::Umber,
        SkinTone::Sienna => SkinShade::Sienna,
        SkinTone::Ebony => SkinShade::Ebony,
    }
}

const fn hair_shade(tone: HairTone) -> HairShade {
    match tone {
        HairTone::Black => HairShade::Black,
        HairTone::Espresso => HairShade::Espresso,
        HairTone::Chestnut => HairShade::Chestnut,
        HairTone::Copper => HairShade::Copper,
        HairTone::Gold => HairShade::Gold,
        HairTone::Silver => HairShade::Silver,
    }
}

const fn garb_shade(garb: Garb) -> FabricShade {
    match garb {
        Garb::Armour => FabricShade::Navy,
        Garb::Cloak => FabricShade::Cobalt,
        Garb::Doublet => FabricShade::Teal,
        Garb::Leathers => FabricShade::Green,
        Garb::Robes => FabricShade::Mustard,
        Garb::Vestments => FabricShade::Crimson,
        Garb::WorkApron => FabricShade::Plum,
    }
}

const fn legwear_shade(legwear: Legwear) -> FabricShade {
    match legwear {
        Legwear::BootsAndBreeches => FabricShade::Navy,
        Legwear::Greaves => FabricShade::Plum,
        Legwear::RobeHem => FabricShade::Green,
        Legwear::TravelingSkirt => FabricShade::Crimson,
    }
}

const fn footwear_shade(footwear: Footwear) -> FootwearShade {
    match footwear {
        Footwear::Boots => FootwearShade::Charcoal,
        Footwear::Sabatons => FootwearShade::White,
        Footwear::Sandals => FootwearShade::Blue,
        Footwear::SoftShoes => FootwearShade::Black,
    }
}

const fn keepsake_shade(keepsake: Keepsake) -> AccentShade {
    match keepsake {
        Keepsake::Feather => AccentShade::Amber,
        Keepsake::LuckyCoin => AccentShade::Cyan,
        Keepsake::Mug => AccentShade::Lime,
        Keepsake::PressedLeaf => AccentShade::Red,
        Keepsake::Ribbon => AccentShade::Violet,
        Keepsake::TinyFamiliar => AccentShade::Teal,
    }
}

const fn accent_shade(accent: AccentTone) -> AccentShade {
    match accent {
        AccentTone::Amber => AccentShade::Amber,
        AccentTone::Cyan => AccentShade::Cyan,
        AccentTone::Lime => AccentShade::Lime,
        AccentTone::Magenta => AccentShade::Magenta,
        AccentTone::Red => AccentShade::Red,
        AccentTone::Blue => AccentShade::Blue,
        AccentTone::Violet => AccentShade::Violet,
        AccentTone::Teal => AccentShade::Teal,
    }
}
