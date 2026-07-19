#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SceneFirstAsset {
    GuildHall,
    Delve,
    SelectedAdventurer,
    CounselParchment,
    SearchParchment,
    ScryingParchment,
    HelpParchment,
    NarrowParchment,
}

impl SceneFirstAsset {
    pub const ALL: &'static [Self] = &[
        Self::GuildHall,
        Self::Delve,
        Self::SelectedAdventurer,
        Self::CounselParchment,
        Self::SearchParchment,
        Self::ScryingParchment,
        Self::HelpParchment,
        Self::NarrowParchment,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::GuildHall => "world: Guild Hall",
            Self::Delve => "world: Delve",
            Self::SelectedAdventurer => "interaction: selected adventurer",
            Self::CounselParchment => "interaction: counsel parchment",
            Self::SearchParchment => "interaction: search parchment",
            Self::ScryingParchment => "interaction: scrying parchment",
            Self::HelpParchment => "interaction: help parchment",
            Self::NarrowParchment => "interaction: narrow parchment",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AssetId(pub SceneFirstAsset);

impl AssetId {
    pub const fn label(self) -> &'static str {
        self.0.label()
    }
}

pub fn asset_inventory() -> Vec<AssetId> {
    SceneFirstAsset::ALL.iter().copied().map(AssetId).collect()
}
