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
    let mut stories = vec![
        story!(
            "world.guild",
            "World / Guild Hall",
            Category::Worlds,
            "The production Guild Hall with a truthful mixed party.",
            WORLD_VIEWPORT,
            guild_world,
            GuildHall
        ),
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
            "interaction.help",
            "Interaction / Help Parchment",
            Category::Interactions,
            "The production field guide over the Guild Hall.",
            WORLD_VIEWPORT,
            help,
            HelpParchment
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

fn asset_stories() -> [Story; 8] {
    [
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
            "asset.native-barbarian-portrait",
            "Asset / Native Barbarian Card",
            Category::Assets,
            "The production card uses the embedded PNG on native protocols and its authored sprite fallback everywhere else.",
            ASSET_VIEWPORT,
            native_barbarian_portrait,
            NativeBarbarianPortrait
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
            "asset.native-wizard-portrait",
            "Asset / Native Wizard Card",
            Category::Assets,
            "The production Wizard card uses its embedded PNG on native protocols and the authored sprite fallback elsewhere.",
            ASSET_VIEWPORT,
            native_wizard_portrait,
            NativeWizardPortrait
        ),
        story!(
            "asset.native-goblin-portrait",
            "Asset / Native Goblin Card",
            Category::Assets,
            "Goblin ancestry takes priority over class when its embedded native portrait is available.",
            ASSET_VIEWPORT,
            native_goblin_portrait,
            NativeGoblinPortrait
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

fn goblin_easter_egg(_: StoryContext) -> StoryFixture {
    StoryFixture::ArchetypeGallery(fixtures::ArchetypeGallery::GoblinEasterEgg)
}

fn selected(context: StoryContext) -> StoryFixture {
    scene(fixtures::selected_adventurer_interaction_fixture(context))
}

fn native_barbarian_portrait(context: StoryContext) -> StoryFixture {
    scene(fixtures::native_barbarian_portrait_fixture(context))
}

fn native_rogue_portrait(context: StoryContext) -> StoryFixture {
    scene(fixtures::native_rogue_portrait_fixture(context))
}

fn native_wizard_portrait(context: StoryContext) -> StoryFixture {
    scene(fixtures::native_wizard_portrait_fixture(context))
}

fn native_goblin_portrait(context: StoryContext) -> StoryFixture {
    scene(fixtures::native_goblin_portrait_fixture(context))
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

fn help(context: StoryContext) -> StoryFixture {
    scene(fixtures::help_interaction_fixture(context))
}

fn narrow(context: StoryContext) -> StoryFixture {
    scene(fixtures::narrow_interaction_fixture(context))
}
