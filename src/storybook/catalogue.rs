use std::{collections::HashMap, fmt, sync::OnceLock};

use super::{
    AssetId, SceneFirstAsset, asset_inventory,
    fixtures::{self, StoryContext, StoryFixture},
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StoryId(&'static str);

impl StoryId {
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Category {
    Worlds,
    Assets,
    Interactions,
}

impl Category {
    pub const ALL: [Self; 3] = [Self::Worlds, Self::Assets, Self::Interactions];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Viewport {
    pub reference_width: u16,
    pub reference_height: u16,
    pub minimum_width: u16,
    pub minimum_height: u16,
}

impl Viewport {
    pub const fn new(
        reference_width: u16,
        reference_height: u16,
        minimum_width: u16,
        minimum_height: u16,
    ) -> Self {
        Self {
            reference_width,
            reference_height,
            minimum_width,
            minimum_height,
        }
    }
}

pub type StoryBuilder = fn(StoryContext) -> StoryFixture;

#[derive(Clone, Debug)]
pub struct Story {
    pub id: StoryId,
    pub title: &'static str,
    pub category: Category,
    pub description: &'static str,
    pub viewport: Viewport,
    pub build: StoryBuilder,
    pub owns: &'static [AssetId],
    pub shows: &'static [AssetId],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoverageReport {
    owned: usize,
}

impl CoverageReport {
    pub const fn owned(&self) -> usize {
        self.owned
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoverageError {
    issues: Vec<String>,
}

impl CoverageError {
    pub fn missing(&self) -> impl Iterator<Item = &str> {
        self.issues
            .iter()
            .filter_map(|issue| issue.strip_prefix("missing "))
    }

    pub fn duplicates(&self) -> impl Iterator<Item = &str> {
        self.issues
            .iter()
            .filter_map(|issue| issue.strip_prefix("duplicate "))
    }

    pub fn unknown(&self) -> impl Iterator<Item = &str> {
        self.issues
            .iter()
            .filter_map(|issue| issue.strip_prefix("unknown "))
    }
}

impl fmt::Display for CoverageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Storybook coverage failed: {}",
            self.issues.join(", ")
        )
    }
}

impl std::error::Error for CoverageError {}

pub fn validate_coverage(
    inventory: &[AssetId],
    stories: &[Story],
) -> Result<CoverageReport, CoverageError> {
    let mut counts = HashMap::<AssetId, usize>::new();
    for story in stories {
        for asset in story.owns {
            *counts.entry(*asset).or_default() += 1;
        }
    }
    let mut issues = Vec::new();
    for asset in inventory {
        match counts.get(asset).copied().unwrap_or_default() {
            0 => issues.push(format!("missing {}", asset.label())),
            1 => {}
            _ => issues.push(format!("duplicate {}", asset.label())),
        }
    }
    for asset in counts.keys().filter(|asset| !inventory.contains(asset)) {
        issues.push(format!("unknown {}", asset.label()));
    }
    if issues.is_empty() {
        Ok(CoverageReport {
            owned: inventory.len(),
        })
    } else {
        Err(CoverageError { issues })
    }
}

const WORLD_VIEWPORT: Viewport = Viewport::new(160, 45, 80, 24);
const ASSET_VIEWPORT: Viewport = Viewport::new(120, 36, 80, 28);
const NARROW_VIEWPORT: Viewport = Viewport::new(64, 24, 48, 18);

macro_rules! story {
    ($id:literal, $title:literal, $category:expr, $description:literal, $viewport:expr, $builder:ident, $asset:ident) => {
        Story {
            id: StoryId::new($id),
            title: $title,
            category: $category,
            description: $description,
            viewport: $viewport,
            build: $builder,
            owns: &[AssetId(SceneFirstAsset::$asset)],
            shows: &[],
        }
    };
}

fn build_catalogue() -> Vec<Story> {
    let mut guild = story!(
        "world.guild",
        "World / Guild Hall",
        Category::Worlds,
        "The production Guild Hall with a truthful mixed party.",
        WORLD_VIEWPORT,
        guild_world,
        GuildHall
    );
    guild.shows = &[AssetId(SceneFirstAsset::LibrarianAssets)];
    let mut stories = vec![
        guild,
        story!(
            "world.delve",
            "World / Delve",
            Category::Worlds,
            "The production connected Delve with the same live party.",
            WORLD_VIEWPORT,
            delve_world,
            Delve
        ),
        story!(
            "interaction.selected",
            "Interaction / Selected Adventurer",
            Category::Interactions,
            "The production selection rune around one adventurer.",
            WORLD_VIEWPORT,
            selected,
            SelectedAdventurer
        ),
        story!(
            "interaction.counsel",
            "Interaction / Counsel Parchment",
            Category::Interactions,
            "The production counsel parchment over the Guild Hall.",
            WORLD_VIEWPORT,
            counsel,
            CounselParchment
        ),
        story!(
            "interaction.search",
            "Interaction / Search Parchment",
            Category::Interactions,
            "The production search parchment over the Guild Hall.",
            WORLD_VIEWPORT,
            search,
            SearchParchment
        ),
        story!(
            "interaction.scrying",
            "Interaction / Scrying Parchment",
            Category::Interactions,
            "The production scrying parchment over the Guild Hall.",
            WORLD_VIEWPORT,
            scrying,
            ScryingParchment
        ),
        story!(
            "interaction.librarian-ledger",
            "Interaction / Librarian's Ledger",
            Category::Interactions,
            "The fixed production handbook opened from the persistent Librarian.",
            WORLD_VIEWPORT,
            librarian_ledger,
            LibrarianLedger
        ),
        story!(
            "interaction.narrow",
            "Interaction / Narrow Parchment",
            Category::Interactions,
            "The production counsel parchment at the narrow boundary.",
            NARROW_VIEWPORT,
            narrow,
            NarrowParchment
        ),
    ];
    stories.splice(2..2, asset_stories());
    stories
}

fn asset_stories() -> Vec<Story> {
    let mut stories = vec![
        story!(
            "asset.world-masters",
            "Assets / Core World Masters",
            Category::Assets,
            "Every classic production archetype at scene scale.",
            ASSET_VIEWPORT,
            core_world_masters,
            CoreWorldMasters
        ),
        story!(
            "asset.barbarian-v2-poses",
            "Assets / Barbarian v2 Poses",
            Category::Assets,
            "Legacy comparison and every truthful production pose for the compact Barbarian v2 experiment.",
            ASSET_VIEWPORT,
            barbarian_v2_poses,
            BarbarianV2PoseFamily
        ),
        story!(
            "asset.persona-palettes",
            "Assets / Persona Palette Family",
            Category::Assets,
            "One shared class master across the persona skin, hair and accent range.",
            ASSET_VIEWPORT,
            persona_palettes,
            PersonaPaletteFamily
        ),
        story!(
            "asset.roster-families",
            "Assets / Roster Silhouette Families",
            Category::Assets,
            "The authored 8x12 masters a narrow pane recomposes the whole party into.",
            ASSET_VIEWPORT,
            roster_families,
            RosterSilhouetteFamilies
        ),
        story!(
            "asset.custom-class-masters",
            "Assets / Custom Class Masters",
            Category::Assets,
            "Artificer, Runewright, Testmender and Pathseeker: the classes that used to borrow another class's body.",
            ASSET_VIEWPORT,
            custom_class_masters,
            CustomClassMasters
        ),
        story!(
            "asset.portrait-masters",
            "Assets / Core Portrait Masters",
            Category::Assets,
            "Every classic production archetype at portrait scale.",
            ASSET_VIEWPORT,
            core_portrait_masters,
            CorePortraitMasters
        ),
        story!(
            "asset.goblin",
            "Assets / Goblin Easter Egg",
            Category::Assets,
            "The authored Goblin ancestry callback at both production scales.",
            ASSET_VIEWPORT,
            goblin_easter_egg,
            GoblinEasterEgg
        ),
        story!(
            "asset.librarian",
            "Assets / Librarian",
            Category::Assets,
            "The persistent Guild Hall Librarian at both authored production scales.",
            ASSET_VIEWPORT,
            librarian_assets,
            LibrarianAssets
        ),
    ];
    stories.extend(native_card_stories());
    stories
}

fn native_card_stories() -> Vec<Story> {
    let mut stories = native_foundation_card_stories().to_vec();
    stories.extend(native_adventurer_card_stories());
    stories.extend(native_custom_class_card_stories());
    stories.extend(reserved_event_art_stories());
    stories
}

fn native_foundation_card_stories() -> [Story; 5] {
    [
        story!(
            "asset.native-artificer-portrait",
            "Asset / Native Artificer Card",
            Category::Assets,
            "The production Artificer card uses its embedded PNG on native protocols and the authored sprite fallback elsewhere.",
            ASSET_VIEWPORT,
            native_artificer_portrait,
            NativeArtificerPortrait
        ),
        story!(
            "asset.native-barbarian-portrait",
            "Asset / Native Barbarian Card",
            Category::Assets,
            "The production card uses the embedded PNG on native protocols and its authored sprite fallback everywhere else.",
            ASSET_VIEWPORT,
            native_barbarian_portrait,
            NativeBarbarianPortrait
        ),
        story!(
            "asset.native-bard-portrait",
            "Asset / Native Bard Card",
            Category::Assets,
            "The production Bard card uses its embedded PNG on native protocols and the authored sprite fallback elsewhere.",
            ASSET_VIEWPORT,
            native_bard_portrait,
            NativeBardPortrait
        ),
        story!(
            "asset.native-cleric-portrait",
            "Asset / Native Cleric Card",
            Category::Assets,
            "The production Cleric card uses its embedded PNG on native protocols and the authored sprite fallback elsewhere.",
            ASSET_VIEWPORT,
            native_cleric_portrait,
            NativeClericPortrait
        ),
        story!(
            "asset.native-druid-portrait",
            "Asset / Native Druid Card",
            Category::Assets,
            "The production Druid card uses its embedded PNG on native protocols and the authored sprite fallback elsewhere.",
            ASSET_VIEWPORT,
            native_druid_portrait,
            NativeDruidPortrait
        ),
    ]
}

fn native_custom_class_card_stories() -> [Story; 2] {
    [
        story!(
            "asset.native-runewright-portrait",
            "Asset / Native Runewright Card",
            Category::Assets,
            "The production Runewright card uses its embedded PNG on native protocols and the authored sprite fallback elsewhere.",
            ASSET_VIEWPORT,
            native_runewright_portrait,
            NativeRunewrightPortrait
        ),
        story!(
            "asset.native-pathseeker-portrait",
            "Asset / Native Pathseeker Card",
            Category::Assets,
            "The production Pathseeker card uses its embedded PNG on native protocols and the authored sprite fallback elsewhere.",
            ASSET_VIEWPORT,
            native_pathseeker_portrait,
            NativePathseekerPortrait
        ),
    ]
}

fn native_adventurer_card_stories() -> [Story; 5] {
    [
        story!(
            "asset.native-paladin-portrait",
            "Asset / Native Paladin Card",
            Category::Assets,
            "The production Paladin card uses its embedded PNG on native protocols and the authored sprite fallback elsewhere.",
            ASSET_VIEWPORT,
            native_paladin_portrait,
            NativePaladinPortrait
        ),
        story!(
            "asset.native-ranger-portrait",
            "Asset / Native Ranger Card",
            Category::Assets,
            "The production Ranger card uses its embedded PNG on native protocols and the authored sprite fallback elsewhere.",
            ASSET_VIEWPORT,
            native_ranger_portrait,
            NativeRangerPortrait
        ),
        story!(
            "asset.native-rogue-portrait",
            "Asset / Native Rogue Card",
            Category::Assets,
            "The production Rogue card uses its embedded PNG on native protocols and the authored sprite fallback elsewhere.",
            ASSET_VIEWPORT,
            native_rogue_portrait,
            NativeRoguePortrait
        ),
        story!(
            "asset.native-testmender-portrait",
            "Asset / Native Testmender Card",
            Category::Assets,
            "The production Testmender card uses its embedded PNG on native protocols and the authored sprite fallback elsewhere.",
            ASSET_VIEWPORT,
            native_testmender_portrait,
            NativeTestmenderPortrait
        ),
        story!(
            "asset.native-wizard-portrait",
            "Asset / Native Wizard Card",
            Category::Assets,
            "The production Wizard card uses its embedded PNG on native protocols and the authored sprite fallback elsewhere.",
            ASSET_VIEWPORT,
            native_wizard_portrait,
            NativeWizardPortrait
        ),
    ]
}

fn reserved_event_art_stories() -> [Story; 2] {
    [
        story!(
            "asset.native-goblin-portrait",
            "Asset / Reserved Goblin Event Art",
            Category::Assets,
            "Reserved event/NPC art. It is not selected for ordinary adventurer cards.",
            ASSET_VIEWPORT,
            native_goblin_portrait,
            NativeGoblinPortrait
        ),
        story!(
            "asset.native-orc-portrait",
            "Asset / Reserved Orc Event Art",
            Category::Assets,
            "Reserved event/NPC art. It is not selected for ordinary adventurer cards.",
            ASSET_VIEWPORT,
            native_orc_portrait,
            NativeOrcPortrait
        ),
    ]
}

pub fn catalogue() -> &'static [Story] {
    static CATALOGUE: OnceLock<Vec<Story>> = OnceLock::new();
    CATALOGUE.get_or_init(build_catalogue)
}

pub fn validate_catalogue() -> Result<CoverageReport, CoverageError> {
    validate_coverage(&asset_inventory(), catalogue())
}

fn scene(model: crate::app::Model) -> StoryFixture {
    StoryFixture::SceneApplication(Box::new(model))
}

fn guild_world(context: StoryContext) -> StoryFixture {
    scene(fixtures::guild_world_fixture(context))
}

fn barbarian_v2_poses(_context: StoryContext) -> StoryFixture {
    fixtures::barbarian_v2_pose_fixture()
}

fn delve_world(context: StoryContext) -> StoryFixture {
    scene(fixtures::delve_world_fixture(context))
}

fn core_world_masters(_: StoryContext) -> StoryFixture {
    StoryFixture::ArchetypeGallery(fixtures::ArchetypeGallery::WorldMasters)
}

fn core_portrait_masters(_: StoryContext) -> StoryFixture {
    StoryFixture::ArchetypeGallery(fixtures::ArchetypeGallery::PortraitMasters)
}

fn persona_palettes(_: StoryContext) -> StoryFixture {
    StoryFixture::ArchetypeGallery(fixtures::ArchetypeGallery::PersonaPalettes)
}

fn roster_families(_: StoryContext) -> StoryFixture {
    StoryFixture::ArchetypeGallery(fixtures::ArchetypeGallery::RosterFamilies)
}

fn custom_class_masters(_: StoryContext) -> StoryFixture {
    StoryFixture::ArchetypeGallery(fixtures::ArchetypeGallery::CustomClassMasters)
}

fn goblin_easter_egg(_: StoryContext) -> StoryFixture {
    StoryFixture::ArchetypeGallery(fixtures::ArchetypeGallery::GoblinEasterEgg)
}

fn librarian_assets(_: StoryContext) -> StoryFixture {
    StoryFixture::ArchetypeGallery(fixtures::ArchetypeGallery::Librarian)
}

fn selected(context: StoryContext) -> StoryFixture {
    scene(fixtures::selected_adventurer_interaction_fixture(context))
}

fn native_artificer_portrait(context: StoryContext) -> StoryFixture {
    scene(fixtures::native_artificer_portrait_fixture(context))
}

fn native_barbarian_portrait(context: StoryContext) -> StoryFixture {
    scene(fixtures::native_barbarian_portrait_fixture(context))
}

fn native_bard_portrait(context: StoryContext) -> StoryFixture {
    scene(fixtures::native_bard_portrait_fixture(context))
}

fn native_cleric_portrait(context: StoryContext) -> StoryFixture {
    scene(fixtures::native_cleric_portrait_fixture(context))
}

fn native_druid_portrait(context: StoryContext) -> StoryFixture {
    scene(fixtures::native_druid_portrait_fixture(context))
}

fn native_paladin_portrait(context: StoryContext) -> StoryFixture {
    scene(fixtures::native_paladin_portrait_fixture(context))
}

fn native_ranger_portrait(context: StoryContext) -> StoryFixture {
    scene(fixtures::native_ranger_portrait_fixture(context))
}

fn native_rogue_portrait(context: StoryContext) -> StoryFixture {
    scene(fixtures::native_rogue_portrait_fixture(context))
}

fn native_testmender_portrait(context: StoryContext) -> StoryFixture {
    scene(fixtures::native_testmender_portrait_fixture(context))
}

fn native_wizard_portrait(context: StoryContext) -> StoryFixture {
    scene(fixtures::native_wizard_portrait_fixture(context))
}

fn native_runewright_portrait(context: StoryContext) -> StoryFixture {
    scene(fixtures::native_runewright_portrait_fixture(context))
}

fn native_pathseeker_portrait(context: StoryContext) -> StoryFixture {
    scene(fixtures::native_pathseeker_portrait_fixture(context))
}

fn native_goblin_portrait(context: StoryContext) -> StoryFixture {
    scene(fixtures::native_goblin_portrait_fixture(context))
}

fn native_orc_portrait(context: StoryContext) -> StoryFixture {
    scene(fixtures::native_orc_portrait_fixture(context))
}

fn counsel(context: StoryContext) -> StoryFixture {
    scene(fixtures::counsel_interaction_fixture(context))
}

fn search(context: StoryContext) -> StoryFixture {
    scene(fixtures::search_interaction_fixture(context))
}

fn scrying(context: StoryContext) -> StoryFixture {
    scene(fixtures::scrying_interaction_fixture(context))
}

fn librarian_ledger(context: StoryContext) -> StoryFixture {
    scene(fixtures::librarian_ledger_fixture(context))
}

fn narrow(context: StoryContext) -> StoryFixture {
    scene(fixtures::narrow_interaction_fixture(context))
}
