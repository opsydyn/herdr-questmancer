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
    domain::{AdventurerPersona, PersonaKey},
    ui::{
        ChamberPresentation, GuildRegion, PersonaRenderMode,
        delve_scene::DelveVariant,
        goblins::GoblinSighting,
        persona_render_mode_for_chamber, render_projection_for,
        widgets::{AdventurerCardPresentation, adventurer_card_presentation, chamber_presentation},
    },
};
use ratatui::layout::Rect;

use super::{
    AssetId, CompatibilityAsset, SceneAsset, SceneFirstAsset, WidgetAsset,
    assets::{
        ACCENT_TONES, ANCESTRIES, BODY_PROPORTIONS, CLASSES, COLOR_ROLES, FACE_DETAILS, FOOTWEAR,
        GARBS, GEAR, HAIR_SHAPES, HAIR_TONES, HEAD_SHAPES, KEEPSAKES, LANDMARKS, LEGWEAR, POSES,
        RoomCameraAsset, SKIN_TONES, TRUTHFUL_STATIONS,
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
const CROPPED_ROOM_VIEWPORT: Viewport = Viewport::new(100, 32, 80, 18);
const LANDMARK_CAMERA_VIEWPORT: Viewport = Viewport::new(78, 26, 48, 18);
const COMPATIBILITY_VIEWPORT: Viewport = Viewport::new(130, 36, 60, 18);
const PIXEL_SCENE_VIEWPORT: Viewport = Viewport::new(120, 36, 40, 18);
const GUILD_HALL_PIXEL_VIEWPORT: Viewport = Viewport::new(160, 45, 80, 24);
const GUILD_HALL_MINIMUM_VIEWPORT: Viewport = Viewport::new(80, 24, 40, 18);
const DELVE_PIXEL_VIEWPORT: Viewport = Viewport::new(160, 45, 80, 24);
const DELVE_MINIMUM_VIEWPORT: Viewport = Viewport::new(80, 24, 40, 18);

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
const WHOLE_ROOM_CAMERA: &[AssetId] = &[AssetId::RoomCamera(RoomCameraAsset::WholeRoom)];
const CROPPED_ROOM_CAMERA: &[AssetId] = &[AssetId::RoomCamera(RoomCameraAsset::CroppedRoom)];
const LANDMARK_CAMERA: &[AssetId] = &[AssetId::RoomCamera(RoomCameraAsset::LandmarkCamera)];

#[allow(
    clippy::too_many_lines,
    reason = "the complete fixed catalogue keeps its prescribed order visible in one table"
)]
fn build_catalogue() -> Vec<Story> {
    let class_and_gear = Box::leak(
        CLASSES
            .iter()
            .chain(GEAR)
            .copied()
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );
    let profile_reuses = persona_reuses("storybook-atlas");
    let pose_reuses = persona_reuses("storybook-pose-atlas");
    let adventurer_card_reuses = widget_atlas_shows(atlas::adventurer_cards);
    let chamber_reuses = widget_atlas_shows(atlas::chambers);
    let guild_region_reuses = widget_atlas_shows(atlas::guild_regions);
    let counsel_reuses = application_shows(counsel, WIDGET_VIEWPORT);
    let search_reuses = application_shows(search, WIDGET_VIEWPORT);
    let help_reuses = application_shows(help, WIDGET_VIEWPORT);
    let mut stories = vec![
        atlas_story!(
            "atlas.classes",
            "Classes and Gear",
            "Every adventurer class with its production class gear.",
            atlas::classes,
            class_and_gear,
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
        atlas_story!(
            "atlas.great-room-landmarks",
            "Great Room Landmarks",
            "Every authored Great Room landmark through the production room renderer.",
            atlas::great_room_landmarks,
            LANDMARKS,
            widget_atlas_shows(atlas::great_room_landmarks)
        ),
        atlas_story!(
            "atlas.truthful-stations",
            "Truthful Stations",
            "Every authored adventurer representation at its truthful station.",
            atlas::truthful_stations,
            TRUTHFUL_STATIONS,
            widget_atlas_shows(atlas::truthful_stations)
        ),
        complete_story(
            "atlas.camera-whole-room",
            "Whole Room Camera",
            Category::AssetAtlas,
            "The production Great Room at the whole-room breakpoint.",
            Viewport::new(130, 40, 120, 36),
            atlas::great_room_whole_camera,
            WHOLE_ROOM_CAMERA,
            widget_atlas_shows(atlas::great_room_whole_camera),
        ),
        complete_story(
            "atlas.camera-cropped-room",
            "Cropped Room Camera",
            Category::AssetAtlas,
            "The production Great Room at the cropped-room breakpoint.",
            Viewport::new(108, 36, 80, 30),
            atlas::great_room_cropped_camera,
            CROPPED_ROOM_CAMERA,
            widget_atlas_shows(atlas::great_room_cropped_camera),
        ),
        complete_story(
            "atlas.camera-landmark",
            "Landmark Camera",
            Category::AssetAtlas,
            "The production Great Room at the landmark-camera breakpoint.",
            Viewport::new(80, 30, 48, 24),
            atlas::great_room_landmark_camera,
            LANDMARK_CAMERA,
            widget_atlas_shows(atlas::great_room_landmark_camera),
        ),
        complete_story(
            "atlas.compact-scene-adventurers",
            "Compact Scene Adventurers",
            Category::AssetAtlas,
            "The original compact scene adventurer vocabulary as a separate atlas.",
            PIXEL_SCENE_VIEWPORT,
            compact_scene_adventurers,
            &[AssetId::SceneFirst(SceneFirstAsset::CompactAdventurers)],
            &[],
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
            guild_region_reuses,
        ),
        complete_story(
            "widgets.counsel",
            "Counsel",
            Category::Widgets,
            "The production counsel modal with a fixed draft.",
            WIDGET_VIEWPORT,
            counsel,
            COUNSEL,
            counsel_reuses,
        ),
        complete_story(
            "widgets.search",
            "Search",
            Category::Widgets,
            "The production search modal with a fixed query.",
            WIDGET_VIEWPORT,
            search,
            SEARCH,
            search_reuses,
        ),
        complete_story(
            "widgets.help",
            "Help",
            Category::Widgets,
            "The production help modal over a fixed Guild hall.",
            WIDGET_VIEWPORT,
            help,
            HELP,
            help_reuses,
        ),
        complete_story(
            "scenes.rgb-calibration-room",
            "RGB Calibration Room",
            Category::FullScenes,
            "An original continuous room proving the terminal-independent RGB renderer.",
            PIXEL_SCENE_VIEWPORT,
            rgb_calibration_room,
            &[AssetId::SceneFirst(SceneFirstAsset::CalibrationRoom)],
            &[],
        ),
        complete_story(
            "scenes.guild-hall-empty",
            "Guild Hall Empty",
            Category::FullScenes,
            "The authored Guild Hall ready for a party, with no invented activity.",
            GUILD_HALL_PIXEL_VIEWPORT,
            guild_hall_empty,
            &[AssetId::SceneFirst(SceneFirstAsset::GuildHallEmpty)],
            &[],
        ),
        complete_story(
            "scenes.guild-hall-mixed-party",
            "Guild Hall Mixed Party",
            Category::FullScenes,
            "Working tokens, a resting adventurer and settled spoils in one truthful room.",
            GUILD_HALL_PIXEL_VIEWPORT,
            guild_hall_mixed_party,
            &[AssetId::SceneFirst(SceneFirstAsset::GuildHallMixedParty)],
            &[],
        ),
        complete_story(
            "scenes.guild-hall-counsel-requested",
            "Guild Hall Counsel Requested",
            Category::FullScenes,
            "A blocked adventurer projected at the Counsel Bell.",
            GUILD_HALL_PIXEL_VIEWPORT,
            guild_hall_counsel_requested,
            &[AssetId::SceneFirst(
                SceneFirstAsset::GuildHallCounselRequested,
            )],
            &[],
        ),
        complete_story(
            "scenes.guild-hall-spoils-returned",
            "Guild Hall Spoils Returned",
            Category::FullScenes,
            "A fixed frame inside the bounded one-shot return celebration.",
            GUILD_HALL_PIXEL_VIEWPORT,
            guild_hall_spoils_returned,
            &[AssetId::SceneFirst(
                SceneFirstAsset::GuildHallSpoilsReturned,
            )],
            &[],
        ),
        complete_story(
            "scenes.guild-hall-reconnecting",
            "Guild Hall Reconnecting",
            Category::FullScenes,
            "The room remains visible under lowered light while the door carries connection truth.",
            GUILD_HALL_PIXEL_VIEWPORT,
            guild_hall_reconnecting,
            &[AssetId::SceneFirst(SceneFirstAsset::GuildHallReconnecting)],
            &[],
        ),
        complete_story(
            "scenes.guild-hall-minimum-viewport",
            "Guild Hall Minimum Viewport",
            Category::FullScenes,
            "A focused crop of the same authored room without scaling or dashboard chrome.",
            GUILD_HALL_MINIMUM_VIEWPORT,
            guild_hall_minimum_viewport,
            &[AssetId::SceneFirst(
                SceneFirstAsset::GuildHallMinimumViewport,
            )],
            &[],
        ),
        complete_story(
            "scenes.delve-active-party",
            "Delve Active Party",
            Category::FullScenes,
            "A working party occupying the connected active passage.",
            DELVE_PIXEL_VIEWPORT,
            delve_active_party,
            &[AssetId::SceneFirst(SceneFirstAsset::DelveActiveParty)],
            &[],
        ),
        complete_story(
            "scenes.delve-mixed-states",
            "Delve Mixed States",
            Category::FullScenes,
            "Every truthful Delve station inside one connected dungeon.",
            DELVE_PIXEL_VIEWPORT,
            delve_mixed_states,
            &[AssetId::SceneFirst(SceneFirstAsset::DelveMixedStates)],
            &[],
        ),
        complete_story(
            "scenes.delve-sealed-gate",
            "Delve Sealed Gate",
            Category::FullScenes,
            "A blocked adventurer waiting truthfully before the sealed gate.",
            DELVE_PIXEL_VIEWPORT,
            delve_sealed_gate,
            &[AssetId::SceneFirst(SceneFirstAsset::DelveSealedGate)],
            &[],
        ),
        complete_story(
            "scenes.delve-reconnecting",
            "Delve Reconnecting",
            Category::FullScenes,
            "The same dungeon under lowered light with connection truth at the entrance.",
            DELVE_PIXEL_VIEWPORT,
            delve_reconnecting,
            &[AssetId::SceneFirst(SceneFirstAsset::DelveReconnecting)],
            &[],
        ),
        complete_story(
            "scenes.delve-minimum-viewport",
            "Delve Minimum Viewport",
            Category::FullScenes,
            "A focused crop of the canonical dungeon without scaling or chamber cards.",
            DELVE_MINIMUM_VIEWPORT,
            delve_minimum_viewport,
            &[AssetId::SceneFirst(SceneFirstAsset::DelveMinimumViewport)],
            &[],
        ),
        scene_story(
            "scenes.guild-empty",
            "Guild Empty",
            SceneAsset::GuildEmpty,
            guild_empty,
            SCENE_VIEWPORT,
        ),
        scene_story(
            "scenes.guild-populated",
            "Guild Populated",
            SceneAsset::GuildPopulated,
            guild_populated,
            SCENE_VIEWPORT,
        ),
        scene_story(
            "scenes.guild-one-campaign",
            "Guild One Campaign",
            SceneAsset::GuildOneCampaign,
            guild_one_campaign,
            SCENE_VIEWPORT,
        ),
        scene_story(
            "scenes.guild-mixed-attention",
            "Guild Mixed Attention",
            SceneAsset::GuildMixedAttention,
            guild_mixed_attention,
            SCENE_VIEWPORT,
        ),
        scene_story(
            "scenes.guild-disconnected",
            "Guild Disconnected",
            SceneAsset::GuildDisconnected,
            guild_disconnected,
            SCENE_VIEWPORT,
        ),
        scene_story(
            "scenes.guild-connecting",
            "Guild Connecting",
            SceneAsset::GuildConnecting,
            guild_connecting,
            SCENE_VIEWPORT,
        ),
        scene_story(
            "scenes.guild-reconnecting",
            "Guild Reconnecting",
            SceneAsset::GuildReconnecting,
            guild_reconnecting,
            SCENE_VIEWPORT,
        ),
        scene_story(
            "scenes.guild-incompatible",
            "Guild Incompatible",
            SceneAsset::GuildIncompatible,
            guild_incompatible,
            SCENE_VIEWPORT,
        ),
        scene_story(
            "scenes.guild-reviewr-unavailable",
            "Reviewr Unavailable",
            SceneAsset::GuildReviewrUnavailable,
            guild_reviewr_unavailable,
            SCENE_VIEWPORT,
        ),
        scene_story(
            "scenes.guild-scrying-failed",
            "Scrying Failed",
            SceneAsset::GuildScryingFailed,
            guild_scrying_failed,
            SCENE_VIEWPORT,
        ),
        scene_story(
            "scenes.guild-cropped-room",
            "Guild Cropped Room",
            SceneAsset::GuildCroppedRoom,
            guild_populated,
            CROPPED_ROOM_VIEWPORT,
        ),
        scene_story(
            "scenes.guild-landmark-camera",
            "Guild Landmark Camera",
            SceneAsset::GuildLandmarkCamera,
            guild_landmark_camera,
            LANDMARK_CAMERA_VIEWPORT,
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
        ),
        scene_story(
            "scenes.mixed-state-delve",
            "Mixed-State Delve",
            SceneAsset::MixedStateDelve,
            mixed_state_delve,
            SCENE_VIEWPORT,
        ),
        scene_story(
            "scenes.narrow-guild",
            "Narrow Guild",
            SceneAsset::NarrowGuild,
            narrow_guild,
            NARROW_VIEWPORT,
        ),
        scene_story(
            "scenes.narrow-delve",
            "Narrow Delve",
            SceneAsset::NarrowDelve,
            narrow_delve,
            NARROW_VIEWPORT,
        ),
        goblin_story(
            "goblins.chest-eyes",
            "Chest Eyes",
            AssetId::GoblinSighting(GoblinSighting::ChestEyes),
            goblin_chest,
        ),
        goblin_story(
            "goblins.chronicle-hand",
            "Chronicle Hand",
            AssetId::GoblinSighting(GoblinSighting::ChronicleHand),
            goblin_hand,
        ),
        goblin_story(
            "goblins.rafters-scroll",
            "Rafters Scroll",
            AssetId::GoblinSighting(GoblinSighting::RaftersScroll),
            goblin_scroll,
        ),
        goblin_story(
            "goblins.stolen-biscuit",
            "Stolen Biscuit",
            AssetId::GoblinSighting(GoblinSighting::StolenBiscuit),
            goblin_biscuit,
        ),
        goblin_story(
            "goblins.outbreak",
            "Goblin Outbreak",
            AssetId::GoblinOutbreak,
            goblin_outbreak,
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
    canonical_shows(&[], &complete_persona_assets(&persona))
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
        let area = Rect::new(
            0,
            0,
            tile.preferred_width.saturating_sub(2),
            tile.preferred_height.saturating_sub(2),
        );
        match tile.content {
            AtlasContent::AdventurerCard {
                agent,
                theatre,
                preferences,
            } => {
                shows.push(AssetId::Pose(theatre.pose));
                match adventurer_card_presentation(area) {
                    AdventurerCardPresentation::Hidden => {}
                    AdventurerCardPresentation::Compact => {
                        if agent.custom_status.is_none() {
                            shows.push(AssetId::Keepsake(agent.persona.appearance.keepsake));
                        }
                        shows.push(AssetId::Widget(WidgetAsset::AdventurerCardCompact));
                    }
                    AdventurerCardPresentation::Full => {
                        shows.push(AssetId::Widget(WidgetAsset::AdventurerCardFull));
                        shows.extend([
                            AssetId::Class(agent.persona.class),
                            AssetId::Gear(agent.persona.class.gear()),
                            AssetId::Ancestry(agent.persona.ancestry),
                            AssetId::Keepsake(agent.persona.appearance.keepsake),
                        ]);
                        if preferences.character_set == CharacterSet::Unicode {
                            shows.extend(complete_persona_assets(&agent.persona));
                        }
                    }
                }
            }
            AtlasContent::Chamber {
                agent,
                theatre,
                preferences,
                ..
            } => {
                let chamber = chamber_presentation(area);
                shows.push(AssetId::Pose(theatre.pose));
                push_chamber_assets(
                    &mut shows,
                    &agent.persona,
                    chamber,
                    persona_render_mode_for_chamber(
                        chamber,
                        theatre.pose,
                        preferences.character_set,
                    ),
                );
            }
            AtlasContent::Application { model } => {
                shows.extend(projection_assets(
                    &model,
                    &render_projection_for(&model, area),
                ));
            }
            AtlasContent::Pixel { .. } | AtlasContent::RgbSprite { .. } => {}
        }
    }
    canonical_shows(&[], &shows)
}

fn application_shows(build: StoryBuilder, viewport: Viewport) -> &'static [AssetId] {
    let StoryFixture::Application(model) = build(&StoryContext::fixed()) else {
        unreachable!("application stories must produce Application fixtures");
    };
    let mut shows = Vec::new();
    let widths = representative_axis(
        viewport.minimum_width,
        viewport.reference_width,
        &[79, 80, 119, 120],
    );
    let heights = representative_axis(
        viewport.minimum_height,
        viewport.reference_height,
        &[19, 20, 23, 24, 31, 32],
    );
    for width in widths {
        for &height in &heights {
            let area = Rect::new(0, 0, width, height);
            shows.extend(projection_assets(
                &model,
                &render_projection_for(&model, area),
            ));
        }
    }
    canonical_shows(&[], &shows)
}

fn representative_axis(minimum: u16, reference: u16, thresholds: &[u16]) -> Vec<u16> {
    let mut values = vec![minimum, reference];
    values.extend(
        thresholds
            .iter()
            .copied()
            .filter(|value| *value >= minimum && *value <= reference),
    );
    values.sort_unstable();
    values.dedup();
    values
}

fn projection_assets(
    model: &crate::app::Model,
    projection: &crate::ui::RenderProjection,
) -> Vec<AssetId> {
    let mut assets = Vec::new();
    for region in &projection.guild_regions {
        assets.push(AssetId::Widget(match region {
            GuildRegion::QuestBoard => WidgetAsset::QuestBoard,
            GuildRegion::Party => WidgetAsset::Party,
            GuildRegion::Summons => WidgetAsset::Summons,
            GuildRegion::Chronicle => WidgetAsset::Chronicle,
            GuildRegion::AdventurerProfile => WidgetAsset::AdventurerProfile,
            GuildRegion::Scrying => WidgetAsset::Scrying,
            GuildRegion::Spoils => WidgetAsset::Spoils,
        }));
    }
    if let Some(key) = &projection.guild_profile_agent
        && let Some(agent) = model.domain().agents.get(key)
    {
        assets.extend([
            AssetId::Class(agent.persona.class),
            AssetId::Ancestry(agent.persona.ancestry),
        ]);
    }
    if projection.delve_connected_scene_visible {
        assets.push(AssetId::Scene(SceneAsset::ConnectedDelves));
    }
    assets.extend(
        projection
            .delve_variants
            .iter()
            .copied()
            .map(AssetId::DelveVariant),
    );
    for projected in &projection.visible_agents {
        let Some(agent) = model.domain().agents.get(&projected.key) else {
            continue;
        };
        assets.push(AssetId::Pose(projected.pose));
        if let Some(chamber) = projected.chamber {
            push_chamber_assets(&mut assets, &agent.persona, chamber, projected.persona);
        }
    }
    assets
}

fn push_chamber_assets(
    assets: &mut Vec<AssetId>,
    persona: &AdventurerPersona,
    chamber: ChamberPresentation,
    persona_mode: PersonaRenderMode,
) {
    match chamber {
        ChamberPresentation::Hidden => {}
        ChamberPresentation::Text | ChamberPresentation::CompactScene => {
            assets.push(AssetId::Widget(WidgetAsset::ChamberCompact));
        }
        ChamberPresentation::Full => {
            assets.push(AssetId::Widget(WidgetAsset::ChamberFull));
        }
    }
    if persona_mode == PersonaRenderMode::Full {
        assets.extend(complete_persona_assets(persona));
    }
}

fn complete_persona_assets(persona: &AdventurerPersona) -> Vec<AssetId> {
    let appearance = persona.appearance;
    vec![
        AssetId::Class(persona.class),
        AssetId::Gear(persona.class.gear()),
        AssetId::Ancestry(persona.ancestry),
        AssetId::BodyProportions(appearance.proportions),
        AssetId::HeadShape(appearance.head_shape),
        AssetId::SkinTone(appearance.skin_tone),
        AssetId::HairShape(appearance.hair),
        AssetId::HairTone(appearance.hair_tone),
        AssetId::FaceDetail(appearance.face_detail),
        AssetId::Garb(appearance.garb),
        AssetId::Legwear(appearance.legwear),
        AssetId::Footwear(appearance.footwear),
        AssetId::Keepsake(appearance.keepsake),
        AssetId::AccentTone(appearance.accent),
    ]
}

fn scene_story(
    id: &'static str,
    title: &'static str,
    asset: SceneAsset,
    build: StoryBuilder,
    viewport: Viewport,
) -> Story {
    let owns = Box::leak(vec![AssetId::Scene(asset)].into_boxed_slice());
    let reused = application_shows(build, viewport);
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
    let reused = application_shows(build, SCENE_VIEWPORT);
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
) -> Story {
    let owns = Box::leak(vec![asset].into_boxed_slice());
    let reused = application_shows(build, SCENE_VIEWPORT);
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
    let reused = application_shows(build, COMPATIBILITY_VIEWPORT);
    complete_story(
        id,
        title,
        Category::Compatibility,
        "The fixed production Great Room under one display preference profile.",
        COMPATIBILITY_VIEWPORT,
        build,
        owns,
        reused,
    )
}

fn application(model: crate::app::Model) -> StoryFixture {
    StoryFixture::Application(model)
}

fn rgb_calibration_room(context: &StoryContext) -> StoryFixture {
    StoryFixture::PixelScene(fixtures::calibration_room_scene_fixture(context))
}

fn guild_hall_empty(context: &StoryContext) -> StoryFixture {
    StoryFixture::PixelScene(fixtures::guild_hall_empty_scene_fixture(context))
}

fn guild_hall_mixed_party(context: &StoryContext) -> StoryFixture {
    StoryFixture::PixelScene(fixtures::guild_hall_mixed_party_scene_fixture(context))
}

fn guild_hall_counsel_requested(context: &StoryContext) -> StoryFixture {
    StoryFixture::PixelScene(fixtures::guild_hall_counsel_requested_scene_fixture(
        context,
    ))
}

fn guild_hall_spoils_returned(context: &StoryContext) -> StoryFixture {
    StoryFixture::PixelScene(fixtures::guild_hall_spoils_returned_scene_fixture(context))
}

fn guild_hall_reconnecting(context: &StoryContext) -> StoryFixture {
    StoryFixture::PixelScene(fixtures::guild_hall_reconnecting_scene_fixture(context))
}

fn guild_hall_minimum_viewport(context: &StoryContext) -> StoryFixture {
    StoryFixture::PixelScene(fixtures::guild_hall_minimum_viewport_scene_fixture(context))
}

fn delve_active_party(context: &StoryContext) -> StoryFixture {
    StoryFixture::PixelScene(fixtures::delve_active_party_scene_fixture(context))
}

fn delve_mixed_states(context: &StoryContext) -> StoryFixture {
    StoryFixture::PixelScene(fixtures::delve_mixed_states_scene_fixture(context))
}

fn delve_sealed_gate(context: &StoryContext) -> StoryFixture {
    StoryFixture::PixelScene(fixtures::delve_sealed_gate_scene_fixture(context))
}

fn delve_reconnecting(context: &StoryContext) -> StoryFixture {
    StoryFixture::PixelScene(fixtures::delve_reconnecting_scene_fixture(context))
}

fn delve_minimum_viewport(context: &StoryContext) -> StoryFixture {
    StoryFixture::PixelScene(fixtures::delve_minimum_viewport_scene_fixture(context))
}

fn compact_scene_adventurers(context: &StoryContext) -> StoryFixture {
    StoryFixture::AssetAtlas(fixtures::compact_adventurers_atlas_fixture(context))
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
    application(fixtures::great_room_empty_fixture(context))
}
fn guild_populated(context: &StoryContext) -> StoryFixture {
    application(fixtures::great_room_fixture(context))
}
fn guild_one_campaign(context: &StoryContext) -> StoryFixture {
    application(fixtures::great_room_one_campaign_fixture(context))
}
fn guild_mixed_attention(context: &StoryContext) -> StoryFixture {
    application(fixtures::great_room_fixture(context))
}
fn guild_disconnected(context: &StoryContext) -> StoryFixture {
    application(fixtures::guild_disconnected_fixture(context))
}
fn guild_reconnecting(context: &StoryContext) -> StoryFixture {
    application(fixtures::guild_reconnecting_fixture(context))
}
fn guild_connecting(context: &StoryContext) -> StoryFixture {
    application(fixtures::guild_connecting_fixture(context))
}
fn guild_incompatible(context: &StoryContext) -> StoryFixture {
    application(fixtures::guild_incompatible_fixture(context))
}
fn guild_reviewr_unavailable(context: &StoryContext) -> StoryFixture {
    application(fixtures::great_room_reviewr_unavailable_fixture(context))
}
fn guild_scrying_failed(context: &StoryContext) -> StoryFixture {
    application(fixtures::great_room_scrying_failed_fixture(context))
}
fn guild_landmark_camera(context: &StoryContext) -> StoryFixture {
    application(fixtures::great_room_focus_fixture(
        context,
        crate::app::GuildFocus::Scrying,
    ))
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
    application(fixtures::motion_compatibility_fixture(Motion::Full))
}
fn motion_reduced(_: &StoryContext) -> StoryFixture {
    application(fixtures::motion_compatibility_fixture(Motion::Reduced))
}
fn motion_none(_: &StoryContext) -> StoryFixture {
    application(fixtures::motion_compatibility_fixture(Motion::None))
}

pub fn catalogue() -> &'static [Story] {
    static CATALOGUE: OnceLock<Vec<Story>> = OnceLock::new();
    CATALOGUE.get_or_init(build_catalogue).as_slice()
}

pub fn validate_catalogue() -> Result<CoverageReport, CoverageError> {
    validate_coverage(&super::asset_inventory(), catalogue())
}
