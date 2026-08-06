#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SceneFirstAsset {
    GuildHall,
    Delve,
    SelectedAdventurer,
    CounselParchment,
    SearchParchment,
    ScryingParchment,
    LibrarianLedger,
    NarrowParchment,
    CoreWorldMasters,
    BarbarianV2PoseFamily,
    PersonaPaletteFamily,
    RosterSilhouetteFamilies,
    CustomClassMasters,
    CorePortraitMasters,
    NativeArtificerPortrait,
    NativeBarbarianPortrait,
    NativeBardPortrait,
    NativeClericPortrait,
    NativeDruidPortrait,
    NativePaladinPortrait,
    NativeRangerPortrait,
    NativeRoguePortrait,
    NativeTestmenderPortrait,
    NativeWizardPortrait,
    NativeRunewrightPortrait,
    NativePathseekerPortrait,
    NativeGoblinPortrait,
    NativeOrcPortrait,
    GoblinEasterEgg,
    LibrarianAssets,
}

impl SceneFirstAsset {
    pub const ALL: &'static [Self] = &[
        Self::GuildHall,
        Self::Delve,
        Self::SelectedAdventurer,
        Self::CounselParchment,
        Self::SearchParchment,
        Self::ScryingParchment,
        Self::LibrarianLedger,
        Self::NarrowParchment,
        Self::CoreWorldMasters,
        Self::BarbarianV2PoseFamily,
        Self::PersonaPaletteFamily,
        Self::RosterSilhouetteFamilies,
        Self::CustomClassMasters,
        Self::CorePortraitMasters,
        Self::NativeArtificerPortrait,
        Self::NativeBarbarianPortrait,
        Self::NativeBardPortrait,
        Self::NativeClericPortrait,
        Self::NativeDruidPortrait,
        Self::NativePaladinPortrait,
        Self::NativeRangerPortrait,
        Self::NativeRoguePortrait,
        Self::NativeTestmenderPortrait,
        Self::NativeWizardPortrait,
        Self::NativeRunewrightPortrait,
        Self::NativePathseekerPortrait,
        Self::NativeGoblinPortrait,
        Self::NativeOrcPortrait,
        Self::GoblinEasterEgg,
        Self::LibrarianAssets,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::GuildHall => "world: Guild Hall",
            Self::Delve => "world: Delve",
            Self::SelectedAdventurer => "interaction: selected adventurer",
            Self::CounselParchment => "interaction: counsel parchment",
            Self::SearchParchment => "interaction: search parchment",
            Self::ScryingParchment => "interaction: scrying parchment",
            Self::LibrarianLedger => "interaction: Librarian's Ledger",
            Self::NarrowParchment => "interaction: narrow parchment",
            Self::CoreWorldMasters => "asset: core world masters",
            Self::BarbarianV2PoseFamily => "asset: Barbarian v2 pose family",
            Self::PersonaPaletteFamily => "asset: persona palette family",
            Self::RosterSilhouetteFamilies => "asset: roster silhouette families",
            Self::CustomClassMasters => "asset: custom class masters",
            Self::CorePortraitMasters => "asset: core portrait masters",
            Self::NativeArtificerPortrait => "asset: native Artificer card portrait",
            Self::NativeBarbarianPortrait => "asset: native Barbarian card portrait",
            Self::NativeBardPortrait => "asset: native Bard card portrait",
            Self::NativeClericPortrait => "asset: native Cleric card portrait",
            Self::NativeDruidPortrait => "asset: native Druid card portrait",
            Self::NativePaladinPortrait => "asset: native Paladin card portrait",
            Self::NativeRangerPortrait => "asset: native Ranger card portrait",
            Self::NativeRoguePortrait => "asset: native Rogue card portrait",
            Self::NativeTestmenderPortrait => "asset: native Testmender card portrait",
            Self::NativeWizardPortrait => "asset: native Wizard card portrait",
            Self::NativeRunewrightPortrait => "asset: native Runewright card portrait",
            Self::NativePathseekerPortrait => "asset: native Pathseeker card portrait",
            Self::NativeGoblinPortrait => "asset: reserved Goblin event portrait",
            Self::NativeOrcPortrait => "asset: reserved Orc event portrait",
            Self::GoblinEasterEgg => "asset: Goblin Easter egg",
            Self::LibrarianAssets => "asset: Librarian world and ledger sprites",
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
