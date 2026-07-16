use std::{
    collections::{HashMap, HashSet},
    fmt,
};

use crate::domain::{AdventurerClass, AdventuringGear};

use super::{
    AssetId,
    assets::{
        ACCENT_TONES, ANCESTRIES, BODY_PROPORTIONS, CLASSES, COLOR_ROLES, FACE_DETAILS, FOOTWEAR,
        GARBS, GEAR, HAIR_SHAPES, HAIR_TONES, HEAD_SHAPES, KEEPSAKES, LEGWEAR, POSES, SKIN_TONES,
    },
    atlas,
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

const ATLAS_VIEWPORT: Viewport = Viewport::new(120, 36, 60, 18);

const CLASS_AND_GEAR: &[AssetId] = &[
    AssetId::Class(AdventurerClass::Barbarian),
    AssetId::Class(AdventurerClass::Bard),
    AssetId::Class(AdventurerClass::Cleric),
    AssetId::Class(AdventurerClass::Paladin),
    AssetId::Class(AdventurerClass::Ranger),
    AssetId::Class(AdventurerClass::Rogue),
    AssetId::Class(AdventurerClass::Wizard),
    AssetId::Class(AdventurerClass::Artificer),
    AssetId::Class(AdventurerClass::Runewright),
    AssetId::Class(AdventurerClass::Testmender),
    AssetId::Class(AdventurerClass::Pathseeker),
    AssetId::Gear(AdventuringGear::Axe),
    AssetId::Gear(AdventuringGear::BowAndQuiver),
    AssetId::Gear(AdventuringGear::HolySymbol),
    AssetId::Gear(AdventuringGear::Lute),
    AssetId::Gear(AdventuringGear::MapAndCompass),
    AssetId::Gear(AdventuringGear::RuneChisel),
    AssetId::Gear(AdventuringGear::Shield),
    AssetId::Gear(AdventuringGear::SpellbookAndStaff),
    AssetId::Gear(AdventuringGear::TestKit),
    AssetId::Gear(AdventuringGear::ThievesTools),
    AssetId::Gear(AdventuringGear::Toolkit),
];

macro_rules! atlas_story {
    ($id:literal, $title:literal, $description:literal, $builder:path, $owns:expr) => {
        Story::new(
            StoryId::new($id),
            $title,
            Category::AssetAtlas,
            $description,
            ATLAS_VIEWPORT,
            $builder,
            $owns,
            &[],
        )
    };
}

static STORIES: [Story; 15] = [
    atlas_story!(
        "atlas.classes",
        "Classes and Gear",
        "Every adventurer class with its production class gear.",
        atlas::classes,
        CLASS_AND_GEAR
    ),
    atlas_story!(
        "atlas.ancestries",
        "Ancestries",
        "Every production adventurer ancestry.",
        atlas::ancestries,
        ANCESTRIES
    ),
    atlas_story!(
        "atlas.body-proportions",
        "Body Proportions",
        "Every production adventurer body proportion.",
        atlas::body_proportions,
        BODY_PROPORTIONS
    ),
    atlas_story!(
        "atlas.head-shapes",
        "Head Shapes",
        "Every production adventurer head shape.",
        atlas::head_shapes,
        HEAD_SHAPES
    ),
    atlas_story!(
        "atlas.skin-tones",
        "Skin Tones",
        "Every production adventurer skin tone.",
        atlas::skin_tones,
        SKIN_TONES
    ),
    atlas_story!(
        "atlas.hair-shapes",
        "Hair Shapes",
        "Every production adventurer hair shape.",
        atlas::hair_shapes,
        HAIR_SHAPES
    ),
    atlas_story!(
        "atlas.hair-tones",
        "Hair Tones",
        "Every production adventurer hair tone.",
        atlas::hair_tones,
        HAIR_TONES
    ),
    atlas_story!(
        "atlas.face-details",
        "Face Details",
        "Every production adventurer face detail.",
        atlas::face_details,
        FACE_DETAILS
    ),
    atlas_story!(
        "atlas.garb",
        "Garb",
        "Every production adventurer garb style.",
        atlas::garb,
        GARBS
    ),
    atlas_story!(
        "atlas.legwear",
        "Legwear",
        "Every production adventurer legwear style.",
        atlas::legwear,
        LEGWEAR
    ),
    atlas_story!(
        "atlas.footwear",
        "Footwear",
        "Every production adventurer footwear style.",
        atlas::footwear,
        FOOTWEAR
    ),
    atlas_story!(
        "atlas.keepsakes",
        "Keepsakes",
        "Every production adventurer keepsake.",
        atlas::keepsakes,
        KEEPSAKES
    ),
    atlas_story!(
        "atlas.accent-tones",
        "Accent Tones",
        "Every production adventurer accent tone.",
        atlas::accent_tones,
        ACCENT_TONES
    ),
    atlas_story!(
        "atlas.palette-roles",
        "Palette Roles",
        "Every production colour role in the Xterm-256 palette.",
        atlas::palette_roles,
        COLOR_ROLES
    ),
    atlas_story!(
        "atlas.poses",
        "Theatre Poses",
        "Every production adventurer theatre pose.",
        atlas::poses,
        POSES
    ),
];

pub fn catalogue() -> &'static [Story] {
    debug_assert!(CLASS_AND_GEAR.starts_with(CLASSES));
    debug_assert!(CLASS_AND_GEAR.ends_with(GEAR));
    &STORIES
}
