#![allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "StoryBuilder intentionally accepts a borrowed fixed StoryContext"
)]

use std::{
    collections::{HashMap, HashSet},
    fmt,
    sync::OnceLock,
};

use crate::{
    app::{CharacterSet, ColorMode, DisplayPreferences, Modal, Motion},
    domain::{AdventurerClass, AdventurerPersona, AdventuringGear, PersonaKey},
    ui::{
        delve_projection::visible_agent_keys, delve_scene::DelveVariant, goblins::GoblinSighting,
        theatre::frame_for,
    },
};
use ratatui::layout::Rect;

use super::{
    AssetId, CompatibilityAsset, SceneAsset, WidgetAsset,
    assets::{
        ACCENT_TONES, ANCESTRIES, BODY_PROPORTIONS, CLASSES, COLOR_ROLES, FACE_DETAILS, FOOTWEAR,
        GARBS, GEAR, HAIR_SHAPES, HAIR_TONES, HEAD_SHAPES, KEEPSAKES, LEGWEAR, POSES, SKIN_TONES,
    },
    atlas,
    fixtures::{self, AtlasContent, StoryContext, StoryFixture},
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
const WIDGET_VIEWPORT: Viewport = Viewport::new(120, 36, 60, 18);
const GUILD_REGIONS_VIEWPORT: Viewport = Viewport::new(122, 36, 122, 36);
const SCENE_VIEWPORT: Viewport = Viewport::new(130, 36, 60, 18);
const NARROW_VIEWPORT: Viewport = Viewport::new(64, 24, 48, 18);
const COMPATIBILITY_VIEWPORT: Viewport = Viewport::new(130, 36, 60, 18);

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
    ($id:literal, $title:literal, $description:literal, $builder:path, $owns:expr, $reused:expr) => {
        complete_story(
            $id,
            $title,
            Category::AssetAtlas,
            $description,
            ATLAS_VIEWPORT,
            $builder,
            $owns,
            $reused,
        )
    };
}

const ADVENTURER_CARDS: &[AssetId] = &[
    AssetId::Widget(WidgetAsset::AdventurerCardFull),
    AssetId::Widget(WidgetAsset::AdventurerCardCompact),
];
const CHAMBERS: &[AssetId] = &[
    AssetId::Widget(WidgetAsset::ChamberFull),
    AssetId::Widget(WidgetAsset::ChamberCompact),
];
const GUILD_REGIONS: &[AssetId] = &[
    AssetId::Widget(WidgetAsset::QuestBoard),
    AssetId::Widget(WidgetAsset::Party),
    AssetId::Widget(WidgetAsset::Summons),
    AssetId::Widget(WidgetAsset::Chronicle),
    AssetId::Widget(WidgetAsset::AdventurerProfile),
    AssetId::Widget(WidgetAsset::Scrying),
    AssetId::Widget(WidgetAsset::Spoils),
];
const COUNSEL: &[AssetId] = &[AssetId::Widget(WidgetAsset::Counsel)];
const SEARCH: &[AssetId] = &[AssetId::Widget(WidgetAsset::Search)];
const HELP: &[AssetId] = &[AssetId::Widget(WidgetAsset::Help)];
const NAMED_DELVE_REUSES: &[AssetId] = &[
    AssetId::Scene(SceneAsset::ConnectedDelves),
    AssetId::Widget(WidgetAsset::ChamberCompact),
];
const CONNECTED_DELVE_REUSES: &[AssetId] = &[
    AssetId::DelveVariant(DelveVariant::ForgottenLibrary),
    AssetId::DelveVariant(DelveVariant::MossyUndercroft),
    AssetId::DelveVariant(DelveVariant::OldWatchtower),
    AssetId::Widget(WidgetAsset::ChamberCompact),
];
const GUILD_REUSES: &[AssetId] = GUILD_REGIONS;
const NARROW_GUILD_REUSES: &[AssetId] = &[AssetId::Widget(WidgetAsset::QuestBoard)];
const NARROW_DELVE_REUSES: &[AssetId] = &[AssetId::Widget(WidgetAsset::ChamberCompact)];
const GUILD_REGION_SHOWS: &[AssetId] = &[AssetId::Scene(SceneAsset::GuildPopulated)];
const MODAL_GUILD_REUSES: &[AssetId] = &[
    AssetId::Scene(SceneAsset::GuildMixedAttention),
    AssetId::Widget(WidgetAsset::QuestBoard),
    AssetId::Widget(WidgetAsset::Party),
    AssetId::Widget(WidgetAsset::Summons),
    AssetId::Widget(WidgetAsset::Chronicle),
    AssetId::Widget(WidgetAsset::AdventurerProfile),
    AssetId::Widget(WidgetAsset::Scrying),
    AssetId::Widget(WidgetAsset::Spoils),
];
const GOBLIN_REUSES: &[AssetId] = &[
    AssetId::Scene(SceneAsset::GuildPopulated),
    AssetId::Widget(WidgetAsset::QuestBoard),
    AssetId::Widget(WidgetAsset::Party),
    AssetId::Widget(WidgetAsset::Summons),
    AssetId::Widget(WidgetAsset::Chronicle),
    AssetId::Widget(WidgetAsset::AdventurerProfile),
    AssetId::Widget(WidgetAsset::Scrying),
    AssetId::Widget(WidgetAsset::Spoils),
];
const GOBLIN_OUTBREAK_REUSES: &[AssetId] = &[
    AssetId::Scene(SceneAsset::GuildMixedAttention),
    AssetId::Widget(WidgetAsset::QuestBoard),
    AssetId::Widget(WidgetAsset::Party),
    AssetId::Widget(WidgetAsset::Summons),
    AssetId::Widget(WidgetAsset::Chronicle),
    AssetId::Widget(WidgetAsset::AdventurerProfile),
    AssetId::Widget(WidgetAsset::Scrying),
    AssetId::Widget(WidgetAsset::Spoils),
];
const COMPATIBILITY_REUSES: &[AssetId] = &[
    AssetId::Scene(SceneAsset::ConnectedDelves),
    AssetId::DelveVariant(DelveVariant::ForgottenLibrary),
    AssetId::DelveVariant(DelveVariant::MossyUndercroft),
    AssetId::DelveVariant(DelveVariant::OldWatchtower),
    AssetId::Widget(WidgetAsset::ChamberCompact),
];

#[allow(
    clippy::too_many_lines,
    reason = "the complete fixed catalogue keeps its prescribed order visible in one table"
)]
fn build_catalogue() -> Vec<Story> {
    let profile_reuses = persona_reuses("storybook-atlas");
    let pose_reuses = persona_reuses("storybook-pose-atlas");
    let adventurer_card_reuses = widget_atlas_shows(atlas::adventurer_cards);
    let chamber_reuses = widget_atlas_shows(atlas::chambers);
    let connected_delves_reuses = delve_shows(
        connected_delves,
        SCENE_VIEWPORT,
        CONNECTED_DELVE_REUSES,
        true,
    );
    let mixed_state_delve_reuses = delve_shows(
        mixed_state_delve,
        SCENE_VIEWPORT,
        CONNECTED_DELVE_REUSES,
        true,
    );
    let narrow_delve_reuses =
        delve_shows(narrow_delve, NARROW_VIEWPORT, NARROW_DELVE_REUSES, false);
    let mut stories = vec![
        atlas_story!(
            "atlas.classes",
            "Classes and Gear",
            "Every adventurer class with its production class gear.",
            atlas::classes,
            CLASS_AND_GEAR,
            profile_reuses
        ),
        atlas_story!(
            "atlas.ancestries",
            "Ancestries",
            "Every production adventurer ancestry.",
            atlas::ancestries,
            ANCESTRIES,
            profile_reuses
        ),
        atlas_story!(
            "atlas.body-proportions",
            "Body Proportions",
            "Every production adventurer body proportion.",
            atlas::body_proportions,
            BODY_PROPORTIONS,
            profile_reuses
        ),
        atlas_story!(
            "atlas.head-shapes",
            "Head Shapes",
            "Every production adventurer head shape.",
            atlas::head_shapes,
            HEAD_SHAPES,
            profile_reuses
        ),
        atlas_story!(
            "atlas.skin-tones",
            "Skin Tones",
            "Every production adventurer skin tone.",
            atlas::skin_tones,
            SKIN_TONES,
            profile_reuses
        ),
        atlas_story!(
            "atlas.hair-shapes",
            "Hair Shapes",
            "Every production adventurer hair shape.",
            atlas::hair_shapes,
            HAIR_SHAPES,
            profile_reuses
        ),
        atlas_story!(
            "atlas.hair-tones",
            "Hair Tones",
            "Every production adventurer hair tone.",
            atlas::hair_tones,
            HAIR_TONES,
            profile_reuses
        ),
        atlas_story!(
            "atlas.face-details",
            "Face Details",
            "Every production adventurer face detail.",
            atlas::face_details,
            FACE_DETAILS,
            profile_reuses
        ),
        atlas_story!(
            "atlas.garb",
            "Garb",
            "Every production adventurer garb style.",
            atlas::garb,
            GARBS,
            profile_reuses
        ),
        atlas_story!(
            "atlas.legwear",
            "Legwear",
            "Every production adventurer legwear style.",
            atlas::legwear,
            LEGWEAR,
            profile_reuses
        ),
        atlas_story!(
            "atlas.footwear",
            "Footwear",
            "Every production adventurer footwear style.",
            atlas::footwear,
            FOOTWEAR,
            profile_reuses
        ),
        atlas_story!(
            "atlas.keepsakes",
            "Keepsakes",
            "Every production adventurer keepsake.",
            atlas::keepsakes,
            KEEPSAKES,
            profile_reuses
        ),
        atlas_story!(
            "atlas.accent-tones",
            "Accent Tones",
            "Every production adventurer accent tone.",
            atlas::accent_tones,
            ACCENT_TONES,
            profile_reuses
        ),
        atlas_story!(
            "atlas.palette-roles",
            "Palette Roles",
            "Every production colour role in the Xterm-256 palette.",
            atlas::palette_roles,
            COLOR_ROLES,
            &[]
        ),
        atlas_story!(
            "atlas.poses",
            "Theatre Poses",
            "Every production adventurer theatre pose.",
            atlas::poses,
            POSES,
            pose_reuses
        ),
    ];

    stories.extend([
        complete_story(
            "widgets.adventurer-cards",
            "Adventurer Cards",
            Category::Widgets,
            "Full and compact production adventurer cards.",
            WIDGET_VIEWPORT,
            atlas::adventurer_cards,
            ADVENTURER_CARDS,
            adventurer_card_reuses,
        ),
        complete_story(
            "widgets.chambers",
            "Chambers",
            Category::Widgets,
            "Full and compact production Delve chambers.",
            WIDGET_VIEWPORT,
            atlas::chambers,
            CHAMBERS,
            chamber_reuses,
        ),
        complete_story(
            "widgets.guild-regions",
            "Guild Regions",
            Category::Widgets,
            "Every fixed production Guild region in one populated hall.",
            GUILD_REGIONS_VIEWPORT,
            atlas::guild_regions,
            GUILD_REGIONS,
            GUILD_REGION_SHOWS,
        ),
        complete_story(
            "widgets.counsel",
            "Counsel",
            Category::Widgets,
            "The production counsel modal with a fixed draft.",
            WIDGET_VIEWPORT,
            counsel,
            COUNSEL,
            MODAL_GUILD_REUSES,
        ),
        complete_story(
            "widgets.search",
            "Search",
            Category::Widgets,
            "The production search modal with a fixed query.",
            WIDGET_VIEWPORT,
            search,
            SEARCH,
            MODAL_GUILD_REUSES,
        ),
        complete_story(
            "widgets.help",
            "Help",
            Category::Widgets,
            "The production help modal over a fixed Guild hall.",
            WIDGET_VIEWPORT,
            help,
            HELP,
            MODAL_GUILD_REUSES,
        ),
        scene_story(
            "scenes.guild-empty",
            "Guild Empty",
            SceneAsset::GuildEmpty,
            guild_empty,
            SCENE_VIEWPORT,
            &[],
        ),
        scene_story(
            "scenes.guild-populated",
            "Guild Populated",
            SceneAsset::GuildPopulated,
            guild_populated,
            SCENE_VIEWPORT,
            GUILD_REUSES,
        ),
        scene_story(
            "scenes.guild-mixed-attention",
            "Guild Mixed Attention",
            SceneAsset::GuildMixedAttention,
            guild_mixed_attention,
            SCENE_VIEWPORT,
            GUILD_REUSES,
        ),
        scene_story(
            "scenes.guild-disconnected",
            "Guild Disconnected",
            SceneAsset::GuildDisconnected,
            guild_disconnected,
            SCENE_VIEWPORT,
            GUILD_REUSES,
        ),
        scene_story(
            "scenes.guild-reconnecting",
            "Guild Reconnecting",
            SceneAsset::GuildReconnecting,
            guild_reconnecting,
            SCENE_VIEWPORT,
            GUILD_REUSES,
        ),
        delve_variant_story(
            "scenes.delve-library",
            "Forgotten Library",
            DelveVariant::ForgottenLibrary,
            delve_library,
        ),
        delve_variant_story(
            "scenes.delve-undercroft",
            "Mossy Undercroft",
            DelveVariant::MossyUndercroft,
            delve_undercroft,
        ),
        delve_variant_story(
            "scenes.delve-watchtower",
            "Old Watchtower",
            DelveVariant::OldWatchtower,
            delve_watchtower,
        ),
        scene_story(
            "scenes.connected-delves",
            "Connected Delves",
            SceneAsset::ConnectedDelves,
            connected_delves,
            SCENE_VIEWPORT,
            connected_delves_reuses,
        ),
        scene_story(
            "scenes.mixed-state-delve",
            "Mixed-State Delve",
            SceneAsset::MixedStateDelve,
            mixed_state_delve,
            SCENE_VIEWPORT,
            mixed_state_delve_reuses,
        ),
        scene_story(
            "scenes.narrow-guild",
            "Narrow Guild",
            SceneAsset::NarrowGuild,
            narrow_guild,
            NARROW_VIEWPORT,
            NARROW_GUILD_REUSES,
        ),
        scene_story(
            "scenes.narrow-delve",
            "Narrow Delve",
            SceneAsset::NarrowDelve,
            narrow_delve,
            NARROW_VIEWPORT,
            narrow_delve_reuses,
        ),
        goblin_story(
            "goblins.chest-eyes",
            "Chest Eyes",
            AssetId::GoblinSighting(GoblinSighting::ChestEyes),
            goblin_chest,
            GOBLIN_REUSES,
        ),
        goblin_story(
            "goblins.chronicle-hand",
            "Chronicle Hand",
            AssetId::GoblinSighting(GoblinSighting::ChronicleHand),
            goblin_hand,
            GOBLIN_REUSES,
        ),
        goblin_story(
            "goblins.rafters-scroll",
            "Rafters Scroll",
            AssetId::GoblinSighting(GoblinSighting::RaftersScroll),
            goblin_scroll,
            GOBLIN_REUSES,
        ),
        goblin_story(
            "goblins.stolen-biscuit",
            "Stolen Biscuit",
            AssetId::GoblinSighting(GoblinSighting::StolenBiscuit),
            goblin_biscuit,
            GOBLIN_REUSES,
        ),
        goblin_story(
            "goblins.outbreak",
            "Goblin Outbreak",
            AssetId::GoblinOutbreak,
            goblin_outbreak,
            GOBLIN_OUTBREAK_REUSES,
        ),
        compatibility_story(
            "compat.unicode-xterm256",
            "Unicode / Xterm-256",
            CompatibilityAsset::UnicodeXterm256,
            unicode_xterm256,
        ),
        compatibility_story(
            "compat.unicode-ansi16",
            "Unicode / ANSI-16",
            CompatibilityAsset::UnicodeAnsi16,
            unicode_ansi16,
        ),
        compatibility_story(
            "compat.ascii-ansi16",
            "ASCII / ANSI-16",
            CompatibilityAsset::AsciiAnsi16,
            ascii_ansi16,
        ),
        compatibility_story(
            "compat.motion-full",
            "Full Motion",
            CompatibilityAsset::MotionFull,
            motion_full,
        ),
        compatibility_story(
            "compat.motion-reduced",
            "Reduced Motion",
            CompatibilityAsset::MotionReduced,
            motion_reduced,
        ),
        compatibility_story(
            "compat.motion-none",
            "No Motion",
            CompatibilityAsset::MotionNone,
            motion_none,
        ),
    ]);

    stories
}

fn persona_reuses(key: &'static str) -> &'static [AssetId] {
    let persona = AdventurerPersona::for_key(PersonaKey::new(key));
    Box::leak(
        vec![
            AssetId::Class(persona.class),
            AssetId::Ancestry(persona.ancestry),
        ]
        .into_boxed_slice(),
    )
}

#[allow(
    clippy::too_many_arguments,
    reason = "fixed story metadata is intentionally explicit"
)]
fn complete_story(
    id: &'static str,
    title: &'static str,
    category: Category,
    description: &'static str,
    viewport: Viewport,
    build: StoryBuilder,
    owns: &'static [AssetId],
    reused: &'static [AssetId],
) -> Story {
    let shows = canonical_shows(owns, reused);
    Story::new(
        StoryId::new(id),
        title,
        category,
        description,
        viewport,
        build,
        owns,
        shows,
    )
}

fn canonical_shows(owns: &[AssetId], reused: &[AssetId]) -> &'static [AssetId] {
    let mut shows = reused.to_vec();
    shows.retain(|asset| !owns.contains(asset));
    sort_assets(&mut shows);
    shows.dedup();
    Box::leak(shows.into_boxed_slice())
}

fn widget_atlas_shows(build: StoryBuilder) -> &'static [AssetId] {
    let StoryFixture::AssetAtlas(atlas) = build(&StoryContext::fixed()) else {
        unreachable!("widget atlas builders must produce AssetAtlas fixtures");
    };
    let mut shows = Vec::new();
    for tile in atlas.tiles {
        match tile.content {
            AtlasContent::AdventurerCard { agent, theatre, .. }
            | AtlasContent::Chamber { agent, theatre, .. } => {
                shows.push(AssetId::Class(agent.persona.class));
                shows.push(AssetId::Ancestry(agent.persona.ancestry));
                shows.push(AssetId::Pose(theatre.pose));
            }
            AtlasContent::Pixel { .. } | AtlasContent::Application { .. } => {}
        }
    }
    canonical_shows(&[], &shows)
}

fn delve_shows(
    build: StoryBuilder,
    viewport: Viewport,
    fixed: &[AssetId],
    reference_has_persona_sprites: bool,
) -> &'static [AssetId] {
    let StoryFixture::Application(model) = build(&StoryContext::fixed()) else {
        unreachable!("Delve stories must produce Application fixtures");
    };
    let reference = Rect::new(0, 0, viewport.reference_width, viewport.reference_height);
    let minimum = Rect::new(0, 0, viewport.minimum_width, viewport.minimum_height);
    let reference_agents = visible_agent_keys(&model, reference);
    let visible_agents = reference_agents
        .iter()
        .chain(visible_agent_keys(&model, minimum).iter())
        .cloned()
        .collect::<HashSet<_>>();
    let mut shows = fixed.to_vec();
    for key in &visible_agents {
        let agent = model
            .domain()
            .agents
            .get(key)
            .expect("the production visibility projection returns known agents");
        shows.push(AssetId::Pose(
            frame_for(agent, model.now(), model.preferences()).pose,
        ));
    }
    if reference_has_persona_sprites && model.preferences().character_set == CharacterSet::Unicode {
        for key in reference_agents {
            let agent = model
                .domain()
                .agents
                .get(&key)
                .expect("the production visibility projection returns known agents");
            shows.push(AssetId::Class(agent.persona.class));
            shows.push(AssetId::Ancestry(agent.persona.ancestry));
        }
    }
    canonical_shows(&[], &shows)
}

fn scene_story(
    id: &'static str,
    title: &'static str,
    asset: SceneAsset,
    build: StoryBuilder,
    viewport: Viewport,
    reused: &'static [AssetId],
) -> Story {
    let owns = Box::leak(vec![AssetId::Scene(asset)].into_boxed_slice());
    complete_story(
        id,
        title,
        Category::FullScenes,
        "A fixed production application scene.",
        viewport,
        build,
        owns,
        reused,
    )
}

fn delve_variant_story(
    id: &'static str,
    title: &'static str,
    asset: DelveVariant,
    build: StoryBuilder,
) -> Story {
    let owns = Box::leak(vec![AssetId::DelveVariant(asset)].into_boxed_slice());
    let reused = delve_shows(build, SCENE_VIEWPORT, NAMED_DELVE_REUSES, true);
    complete_story(
        id,
        title,
        Category::FullScenes,
        "One named production Delve variant.",
        SCENE_VIEWPORT,
        build,
        owns,
        reused,
    )
}

fn goblin_story(
    id: &'static str,
    title: &'static str,
    asset: AssetId,
    build: StoryBuilder,
    reused: &'static [AssetId],
) -> Story {
    let owns = Box::leak(vec![asset].into_boxed_slice());
    complete_story(
        id,
        title,
        Category::FullScenes,
        "A fixed goblin interruption in the production Guild hall.",
        SCENE_VIEWPORT,
        build,
        owns,
        reused,
    )
}

fn compatibility_story(
    id: &'static str,
    title: &'static str,
    asset: CompatibilityAsset,
    build: StoryBuilder,
) -> Story {
    let owns = Box::leak(vec![AssetId::Compatibility(asset)].into_boxed_slice());
    let reused = delve_shows(build, COMPATIBILITY_VIEWPORT, COMPATIBILITY_REUSES, true);
    complete_story(
        id,
        title,
        Category::Compatibility,
        "The fixed production Delve under one display preference profile.",
        COMPATIBILITY_VIEWPORT,
        build,
        owns,
        reused,
    )
}

fn application(model: crate::app::Model) -> StoryFixture {
    StoryFixture::Application(model)
}

fn counsel(_: &StoryContext) -> StoryFixture {
    application(fixtures::modal_fixture(Modal::Counsel {
        draft: String::new(),
    }))
}
fn search(_: &StoryContext) -> StoryFixture {
    application(fixtures::modal_fixture(Modal::Search {
        query: String::new(),
    }))
}
fn help(_: &StoryContext) -> StoryFixture {
    application(fixtures::modal_fixture(Modal::Help))
}
fn guild_empty(context: &StoryContext) -> StoryFixture {
    application(fixtures::guild_empty_fixture(context))
}
fn guild_populated(context: &StoryContext) -> StoryFixture {
    application(fixtures::guild_populated_fixture(context))
}
fn guild_mixed_attention(context: &StoryContext) -> StoryFixture {
    application(fixtures::guild_fixture(context))
}
fn guild_disconnected(context: &StoryContext) -> StoryFixture {
    application(fixtures::guild_disconnected_fixture(context))
}
fn guild_reconnecting(context: &StoryContext) -> StoryFixture {
    application(fixtures::guild_reconnecting_fixture(context))
}
fn delve_library(context: &StoryContext) -> StoryFixture {
    application(fixtures::library_delve_fixture(context))
}
fn delve_undercroft(context: &StoryContext) -> StoryFixture {
    application(fixtures::undercroft_delve_fixture(context))
}
fn delve_watchtower(context: &StoryContext) -> StoryFixture {
    application(fixtures::watchtower_delve_fixture(context))
}
fn connected_delves(context: &StoryContext) -> StoryFixture {
    application(fixtures::connected_delves_fixture(context))
}
fn mixed_state_delve(context: &StoryContext) -> StoryFixture {
    application(fixtures::delve_fixture(context))
}
fn narrow_guild(context: &StoryContext) -> StoryFixture {
    application(fixtures::guild_fixture(context))
}
fn narrow_delve(context: &StoryContext) -> StoryFixture {
    application(fixtures::delve_fixture(context))
}
fn goblin_chest(context: &StoryContext) -> StoryFixture {
    application(fixtures::goblin_chest_fixture(context))
}
fn goblin_hand(context: &StoryContext) -> StoryFixture {
    application(fixtures::goblin_hand_fixture(context))
}
fn goblin_scroll(context: &StoryContext) -> StoryFixture {
    application(fixtures::goblin_scroll_fixture(context))
}
fn goblin_biscuit(context: &StoryContext) -> StoryFixture {
    application(fixtures::goblin_biscuit_fixture(context))
}
fn goblin_outbreak(context: &StoryContext) -> StoryFixture {
    application(fixtures::goblin_outbreak_fixture(context))
}

const fn preferences(
    motion: Motion,
    character_set: CharacterSet,
    color_mode: ColorMode,
) -> DisplayPreferences {
    DisplayPreferences {
        motion,
        character_set,
        color_mode,
    }
}

fn compatible(preferences: DisplayPreferences) -> StoryFixture {
    application(fixtures::compatibility_fixture(preferences))
}
fn unicode_xterm256(_: &StoryContext) -> StoryFixture {
    compatible(preferences(
        Motion::Full,
        CharacterSet::Unicode,
        ColorMode::Xterm256,
    ))
}
fn unicode_ansi16(_: &StoryContext) -> StoryFixture {
    compatible(preferences(
        Motion::Full,
        CharacterSet::Unicode,
        ColorMode::Ansi16,
    ))
}
fn ascii_ansi16(_: &StoryContext) -> StoryFixture {
    compatible(preferences(
        Motion::Full,
        CharacterSet::Ascii,
        ColorMode::Ansi16,
    ))
}
fn motion_full(_: &StoryContext) -> StoryFixture {
    compatible(preferences(
        Motion::Full,
        CharacterSet::Unicode,
        ColorMode::Xterm256,
    ))
}
fn motion_reduced(_: &StoryContext) -> StoryFixture {
    compatible(preferences(
        Motion::Reduced,
        CharacterSet::Unicode,
        ColorMode::Xterm256,
    ))
}
fn motion_none(_: &StoryContext) -> StoryFixture {
    compatible(preferences(
        Motion::None,
        CharacterSet::Unicode,
        ColorMode::Xterm256,
    ))
}

pub fn catalogue() -> &'static [Story] {
    static CATALOGUE: OnceLock<Vec<Story>> = OnceLock::new();
    debug_assert!(CLASS_AND_GEAR.starts_with(CLASSES));
    debug_assert!(CLASS_AND_GEAR.ends_with(GEAR));
    CATALOGUE.get_or_init(build_catalogue).as_slice()
}

pub fn validate_catalogue() -> Result<CoverageReport, CoverageError> {
    validate_coverage(&super::asset_inventory(), catalogue())
}
