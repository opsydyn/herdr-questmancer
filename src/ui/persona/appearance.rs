use crate::{
    domain::{
        AccentTone, Accessory, HairTone, OutfitBottom, OutfitTop, PersonaAppearance, Shoes,
        SkinTone,
    },
    ui::pixel::{
        AccentShade, ColorRole, FabricShade, FootwearShade, HairShade, Palette, SkinShade,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AppearanceRoles {
    pub skin: ColorRole,
    pub hair: ColorRole,
    pub top: ColorRole,
    pub bottom: ColorRole,
    pub shoes: ColorRole,
    pub accessory: ColorRole,
    pub accent: ColorRole,
    pub highlight: ColorRole,
    pub shadow: ColorRole,
}

pub fn appearance_roles(appearance: &PersonaAppearance) -> AppearanceRoles {
    let skin = ColorRole::SkinTone(skin_shade(appearance.skin_tone));
    let hair = ColorRole::HairTone(hair_shade(appearance.hair_tone));
    let top = ColorRole::Fabric(top_shade(appearance.top));
    let bottom = ColorRole::Fabric(bottom_shade(appearance.bottom));
    let shoes = ColorRole::Footwear(shoe_shade(appearance.shoes));
    let accessory = ColorRole::AccentTone(accessory_shade(appearance.accessory));
    let accent = ColorRole::AccentTone(accent_shade(appearance.accent));

    AppearanceRoles {
        skin,
        hair,
        top,
        bottom,
        shoes,
        accessory,
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
    let top = contrasting(canonical.top, &[skin, hair], &FABRIC_FALLBACKS, palette);
    let bottom = contrasting(canonical.bottom, &[top], &FABRIC_FALLBACKS, palette);
    let shoes = contrasting(
        canonical.shoes,
        &[bottom, ColorRole::PanelBackground],
        &FOOTWEAR_FALLBACKS,
        palette,
    );
    let accessory = contrasting(
        canonical.accessory,
        &[top, skin, hair],
        &ACCENT_FALLBACKS,
        palette,
    );
    let accent = contrasting(
        canonical.accent,
        &[top, skin, hair, accessory],
        &ACCENT_FALLBACKS,
        palette,
    );

    AppearanceRoles {
        skin,
        hair,
        top,
        bottom,
        shoes,
        accessory,
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

const fn top_shade(top: OutfitTop) -> FabricShade {
    match top {
        OutfitTop::BandTee => FabricShade::Navy,
        OutfitTop::StripeJumper => FabricShade::Cobalt,
        OutfitTop::HighCollar => FabricShade::Teal,
        OutfitTop::WorkShirt => FabricShade::Green,
        OutfitTop::Hoodie => FabricShade::Mustard,
        OutfitTop::Cardigan => FabricShade::Orange,
        OutfitTop::Waistcoat => FabricShade::Crimson,
        OutfitTop::TrackTop => FabricShade::Plum,
    }
}

const fn bottom_shade(bottom: OutfitBottom) -> FabricShade {
    match bottom {
        OutfitBottom::Jeans => FabricShade::Navy,
        OutfitBottom::Slacks => FabricShade::Plum,
        OutfitBottom::Cargos => FabricShade::Green,
        OutfitBottom::Skirt => FabricShade::Crimson,
        OutfitBottom::Shorts => FabricShade::Orange,
    }
}

const fn shoe_shade(shoes: Shoes) -> FootwearShade {
    match shoes {
        Shoes::Trainers => FootwearShade::White,
        Shoes::Boots => FootwearShade::Charcoal,
        Shoes::Loafers => FootwearShade::Black,
        Shoes::HighTops => FootwearShade::Blue,
        Shoes::Platforms => FootwearShade::Magenta,
    }
}

const fn accessory_shade(accessory: Accessory) -> AccentShade {
    match accessory {
        Accessory::Headphones => AccentShade::Amber,
        Accessory::Pager => AccentShade::Cyan,
        Accessory::Lanyard => AccentShade::Lime,
        Accessory::Wristband => AccentShade::Magenta,
        Accessory::Scarf => AccentShade::Red,
        Accessory::Badge => AccentShade::Blue,
        Accessory::PocketPen => AccentShade::Violet,
        Accessory::ShoulderBag => AccentShade::Teal,
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
