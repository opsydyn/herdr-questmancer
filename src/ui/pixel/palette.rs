use ratatui::style::Color;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorRole {
    PanelBackground,
    RoomWall,
    RoomFloor,
    Desk,
    Chair,
    CrtCase,
    CrtScreen,
    CrtGlow,
    Skin,
    Hair,
    Top,
    Bottom,
    Shoes,
    Accessory,
    Accent,
    Highlight,
    Shadow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Palette {
    Xterm256,
    Ansi16,
}

impl Palette {
    pub(super) const fn resolve(self, role: ColorRole) -> Color {
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
        ColorRole::Skin => 223,
        ColorRole::Hair => 52,
        ColorRole::Top => 33,
        ColorRole::Bottom => 18,
        ColorRole::Shoes => 232,
        ColorRole::Accessory => 208,
        ColorRole::Accent => 48,
        ColorRole::Highlight => 229,
        ColorRole::Shadow => 236,
    };

    Color::Indexed(index)
}

const fn ansi16(role: ColorRole) -> Color {
    match role {
        ColorRole::PanelBackground => Color::Black,
        ColorRole::RoomWall => Color::Yellow,
        ColorRole::RoomFloor | ColorRole::Hair => Color::Red,
        ColorRole::Desk => Color::LightRed,
        ColorRole::Chair => Color::Magenta,
        ColorRole::CrtCase | ColorRole::Shoes => Color::Gray,
        ColorRole::CrtScreen => Color::Green,
        ColorRole::CrtGlow => Color::LightGreen,
        ColorRole::Skin => Color::LightYellow,
        ColorRole::Top => Color::LightBlue,
        ColorRole::Bottom => Color::Blue,
        ColorRole::Shadow => Color::DarkGray,
        ColorRole::Accessory => Color::LightMagenta,
        ColorRole::Accent => Color::LightCyan,
        ColorRole::Highlight => Color::White,
    }
}
