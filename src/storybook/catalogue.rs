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
    Interactions,
}

impl Category {
    pub const ALL: [Self; 2] = [Self::Worlds, Self::Interactions];
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
    vec![
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
    StoryFixture::SceneApplication(model)
}

fn guild_world(context: StoryContext) -> StoryFixture {
    scene(fixtures::guild_world_fixture(context))
}

fn delve_world(context: StoryContext) -> StoryFixture {
    scene(fixtures::delve_world_fixture(context))
}

fn selected(context: StoryContext) -> StoryFixture {
    scene(fixtures::selected_adventurer_interaction_fixture(context))
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
