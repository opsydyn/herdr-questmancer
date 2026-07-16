use ratatui::style::Color;

use crate::app::ColorMode;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ColorRole {
    Stone,
    DarkStone,
    Timber,
    Parchment,
    Ink,
    Hearth,
    Moss,
    RuneGlow,
    Counsel,
    Spoils,
    Selection,
    Fog,
    Goblin,
    SkinLight,
    SkinMedium,
    SkinDark,
    HairDark,
    HairLight,
    Leather,
    Steel,
    ClothWarm,
    ClothCool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Palette {
    Xterm256,
    Ansi16,
}

impl From<ColorMode> for Palette {
    fn from(mode: ColorMode) -> Self {
        match mode {
            ColorMode::Xterm256 => Self::Xterm256,
            ColorMode::Ansi16 => Self::Ansi16,
        }
    }
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
        ColorRole::Stone => 244,
        ColorRole::DarkStone => 234,
        ColorRole::Timber => 94,
        ColorRole::Parchment => 230,
        ColorRole::Ink => 233,
        ColorRole::Hearth => 208,
        ColorRole::Moss => 65,
        ColorRole::RuneGlow => 81,
        ColorRole::Counsel => 214,
        ColorRole::Spoils => 220,
        ColorRole::Selection => 51,
        ColorRole::Fog => 250,
        ColorRole::Goblin => 70,
        ColorRole::SkinLight => 223,
        ColorRole::SkinMedium => 173,
        ColorRole::SkinDark => 95,
        ColorRole::HairDark => 52,
        ColorRole::HairLight => 179,
        ColorRole::Leather => 130,
        ColorRole::Steel => 248,
        ColorRole::ClothWarm => 160,
        ColorRole::ClothCool => 25,
    };
    Color::Indexed(index)
}

const fn ansi16(role: ColorRole) -> Color {
    match role {
        ColorRole::Stone => Color::Gray,
        ColorRole::DarkStone => Color::Black,
        ColorRole::Timber | ColorRole::Parchment => Color::White,
        ColorRole::Steel => Color::Cyan,
        ColorRole::Ink | ColorRole::HairDark => Color::DarkGray,
        ColorRole::Hearth | ColorRole::Spoils => Color::LightMagenta,
        ColorRole::ClothWarm | ColorRole::SkinMedium => Color::LightRed,
        ColorRole::Moss => Color::Green,
        ColorRole::RuneGlow | ColorRole::Selection => Color::LightCyan,
        ColorRole::Counsel => Color::Yellow,
        ColorRole::HairLight | ColorRole::SkinLight => Color::LightYellow,
        ColorRole::Leather => Color::Red,
        ColorRole::Fog => Color::LightBlue,
        ColorRole::Goblin => Color::LightGreen,
        ColorRole::SkinDark => Color::Magenta,
        ColorRole::ClothCool => Color::Blue,
    }
}
