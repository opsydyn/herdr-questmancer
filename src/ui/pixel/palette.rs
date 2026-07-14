use ratatui::style::Color;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SkinShade {
    Porcelain,
    Rose,
    Sand,
    Umber,
    Sienna,
    Ebony,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HairShade {
    Black,
    Espresso,
    Chestnut,
    Copper,
    Gold,
    Silver,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FabricShade {
    Navy,
    Cobalt,
    Teal,
    Green,
    Mustard,
    Orange,
    Crimson,
    Plum,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FootwearShade {
    Black,
    Charcoal,
    White,
    Blue,
    Magenta,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AccentShade {
    Amber,
    Cyan,
    Lime,
    Magenta,
    Red,
    Blue,
    Violet,
    Teal,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ColorRole {
    PanelBackground,
    RoomWall,
    RoomFloor,
    Desk,
    Chair,
    CrtCase,
    CrtScreen,
    CrtGlow,
    Highlight,
    Shadow,
    SkinTone(SkinShade),
    HairTone(HairShade),
    Fabric(FabricShade),
    Footwear(FootwearShade),
    AccentTone(AccentShade),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Palette {
    Xterm256,
    Ansi16,
}

impl Palette {
    pub fn roles_contrast(self, first: ColorRole, second: ColorRole) -> bool {
        self.resolve(first) != self.resolve(second)
    }

    pub(crate) const fn resolve(self, role: ColorRole) -> Color {
        match self {
            Self::Xterm256 => xterm256(role),
            Self::Ansi16 => ansi16(role),
        }
    }
}

const fn xterm256(role: ColorRole) -> Color {
    let index = match role {
        ColorRole::PanelBackground => 234,
        ColorRole::RoomWall => 180,
        ColorRole::RoomFloor => 94,
        ColorRole::Desk => 130,
        ColorRole::Chair => 88,
        ColorRole::CrtCase => 58,
        ColorRole::CrtScreen => 22,
        ColorRole::CrtGlow => 46,
        ColorRole::Highlight => 229,
        ColorRole::Shadow => 236,
        ColorRole::SkinTone(shade) => match shade {
            SkinShade::Porcelain => 230,
            SkinShade::Rose => 224,
            SkinShade::Sand => 222,
            SkinShade::Umber => 137,
            SkinShade::Sienna => 173,
            SkinShade::Ebony => 94,
        },
        ColorRole::HairTone(shade) => match shade {
            HairShade::Black => 233,
            HairShade::Espresso => 52,
            HairShade::Chestnut => 94,
            HairShade::Copper => 166,
            HairShade::Gold => 178,
            HairShade::Silver => 248,
        },
        ColorRole::Fabric(shade) => match shade {
            FabricShade::Navy => 18,
            FabricShade::Cobalt => 33,
            FabricShade::Teal => 30,
            FabricShade::Green => 28,
            FabricShade::Mustard => 178,
            FabricShade::Orange => 208,
            FabricShade::Crimson => 160,
            FabricShade::Plum => 90,
        },
        ColorRole::Footwear(shade) => match shade {
            FootwearShade::Black => 232,
            FootwearShade::Charcoal => 238,
            FootwearShade::White => 255,
            FootwearShade::Blue => 25,
            FootwearShade::Magenta => 126,
        },
        ColorRole::AccentTone(shade) => match shade {
            AccentShade::Amber => 214,
            AccentShade::Cyan => 51,
            AccentShade::Lime => 118,
            AccentShade::Magenta => 201,
            AccentShade::Red => 196,
            AccentShade::Blue => 39,
            AccentShade::Violet => 135,
            AccentShade::Teal => 37,
        },
    };

    Color::Indexed(index)
}

const fn ansi16(role: ColorRole) -> Color {
    match role {
        ColorRole::PanelBackground => Color::Black,
        ColorRole::RoomWall => Color::Yellow,
        ColorRole::RoomFloor => Color::Red,
        ColorRole::Desk => Color::LightRed,
        ColorRole::Chair => Color::Magenta,
        ColorRole::CrtCase => Color::Gray,
        ColorRole::CrtScreen => Color::Green,
        ColorRole::CrtGlow => Color::LightGreen,
        ColorRole::Shadow => Color::DarkGray,
        ColorRole::Highlight => Color::White,
        ColorRole::SkinTone(shade) => match shade {
            SkinShade::Porcelain | SkinShade::Sand => Color::LightYellow,
            SkinShade::Rose => Color::LightRed,
            SkinShade::Umber => Color::Yellow,
            SkinShade::Sienna => Color::Red,
            SkinShade::Ebony => Color::Magenta,
        },
        ColorRole::HairTone(shade) => match shade {
            HairShade::Black => Color::Black,
            HairShade::Espresso => Color::DarkGray,
            HairShade::Chestnut => Color::Red,
            HairShade::Copper => Color::LightRed,
            HairShade::Gold => Color::Yellow,
            HairShade::Silver => Color::Gray,
        },
        ColorRole::Fabric(shade) => match shade {
            FabricShade::Navy => Color::Blue,
            FabricShade::Cobalt => Color::LightBlue,
            FabricShade::Teal => Color::Cyan,
            FabricShade::Green => Color::Green,
            FabricShade::Mustard => Color::Yellow,
            FabricShade::Orange => Color::LightRed,
            FabricShade::Crimson => Color::Red,
            FabricShade::Plum => Color::Magenta,
        },
        ColorRole::Footwear(shade) => match shade {
            FootwearShade::Black => Color::Black,
            FootwearShade::Charcoal => Color::DarkGray,
            FootwearShade::White => Color::White,
            FootwearShade::Blue => Color::Blue,
            FootwearShade::Magenta => Color::Magenta,
        },
        ColorRole::AccentTone(shade) => match shade {
            AccentShade::Amber => Color::Yellow,
            AccentShade::Cyan => Color::LightCyan,
            AccentShade::Lime => Color::LightGreen,
            AccentShade::Magenta => Color::LightMagenta,
            AccentShade::Red => Color::LightRed,
            AccentShade::Blue => Color::LightBlue,
            AccentShade::Violet => Color::Magenta,
            AccentShade::Teal => Color::Cyan,
        },
    }
}
