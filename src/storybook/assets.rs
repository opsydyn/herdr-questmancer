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
    CoreWorldMasters,
    BarbarianV2PoseFamily,
    CorePortraitMasters,
    NativeBarbarianPortrait,
    NativeRoguePortrait,
    NativeWizardPortrait,
    NativeGoblinPortrait,
    GoblinEasterEgg,
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
        Self::CoreWorldMasters,
        Self::BarbarianV2PoseFamily,
        Self::CorePortraitMasters,
        Self::NativeBarbarianPortrait,
        Self::NativeRoguePortrait,
        Self::NativeWizardPortrait,
        Self::NativeGoblinPortrait,
        Self::GoblinEasterEgg,
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
            Self::CoreWorldMasters => "asset: core world masters",
            Self::BarbarianV2PoseFamily => "asset: Barbarian v2 pose family",
            Self::CorePortraitMasters => "asset: core portrait masters",
            Self::NativeBarbarianPortrait => "asset: native Barbarian card portrait",
            Self::NativeRoguePortrait => "asset: native Rogue card portrait",
            Self::NativeWizardPortrait => "asset: native Wizard card portrait",
            Self::NativeGoblinPortrait => "asset: native Goblin card portrait",
            Self::GoblinEasterEgg => "asset: Goblin Easter egg",
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
