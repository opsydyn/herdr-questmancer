use std::{
    collections::{HashMap, HashSet},
    fmt,
};

use super::AssetId;

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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Category {
    AssetAtlas,
    Widgets,
    FullScenes,
    Compatibility,
}

impl Category {
    pub const ALL: [Self; 4] = [
        Self::AssetAtlas,
        Self::Widgets,
        Self::FullScenes,
        Self::Compatibility,
    ];
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

pub type StoryBuilder = fn(&super::fixtures::StoryContext) -> super::fixtures::StoryFixture;

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

impl Story {
    #[allow(
        clippy::too_many_arguments,
        reason = "story metadata is intentionally explicit"
    )]
    pub const fn new(
        id: StoryId,
        title: &'static str,
        category: Category,
        description: &'static str,
        viewport: Viewport,
        build: StoryBuilder,
        owns: &'static [AssetId],
        shows: &'static [AssetId],
    ) -> Self {
        Self {
            id,
            title,
            category,
            description,
            viewport,
            build,
            owns,
            shows,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoverageReport {
    owned: usize,
    missing: Vec<AssetId>,
    duplicates: Vec<AssetId>,
    unknown: Vec<AssetId>,
}

impl CoverageReport {
    pub const fn owned(&self) -> usize {
        self.owned
    }

    pub fn missing(&self) -> &[AssetId] {
        &self.missing
    }

    pub fn duplicates(&self) -> &[AssetId] {
        &self.duplicates
    }

    pub fn unknown(&self) -> &[AssetId] {
        &self.unknown
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoverageError {
    missing: Vec<AssetId>,
    duplicates: Vec<AssetId>,
    unknown: Vec<AssetId>,
}

impl CoverageError {
    pub fn missing(&self) -> &[AssetId] {
        &self.missing
    }

    pub fn duplicates(&self) -> &[AssetId] {
        &self.duplicates
    }

    pub fn unknown(&self) -> &[AssetId] {
        &self.unknown
    }
}

pub fn validate_coverage(
    inventory: &[AssetId],
    stories: &[Story],
) -> Result<CoverageReport, CoverageError> {
    let inventory = inventory.iter().copied().collect::<HashSet<_>>();
    let mut owners = HashMap::<AssetId, Vec<StoryId>>::new();
    for story in stories {
        for asset in story.owns {
            owners.entry(*asset).or_default().push(story.id);
        }
    }

    let mut missing = inventory
        .iter()
        .filter(|asset| !owners.contains_key(asset))
        .copied()
        .collect::<Vec<_>>();
    let mut duplicates = owners
        .iter()
        .filter(|(asset, story_ids)| inventory.contains(asset) && story_ids.len() > 1)
        .map(|(asset, _)| *asset)
        .collect::<Vec<_>>();
    let mut unknown = owners
        .keys()
        .filter(|asset| !inventory.contains(asset))
        .copied()
        .collect::<Vec<_>>();
    sort_assets(&mut missing);
    sort_assets(&mut duplicates);
    sort_assets(&mut unknown);

    if missing.is_empty() && duplicates.is_empty() && unknown.is_empty() {
        Ok(CoverageReport {
            owned: inventory.len(),
            missing,
            duplicates,
            unknown,
        })
    } else {
        Err(CoverageError {
            missing,
            duplicates,
            unknown,
        })
    }
}

fn sort_assets(assets: &mut [AssetId]) {
    assets.sort_by_key(|asset| asset.label());
}

impl fmt::Display for CoverageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Storybook coverage failed; missing: [{}]; duplicates: [{}]; unknown: [{}]",
            labels(&self.missing),
            labels(&self.duplicates),
            labels(&self.unknown),
        )
    }
}

impl std::error::Error for CoverageError {}

fn labels(assets: &[AssetId]) -> String {
    assets
        .iter()
        .map(|asset| asset.label())
        .collect::<Vec<_>>()
        .join(", ")
}
