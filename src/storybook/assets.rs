use std::collections::HashSet;

use crate::{
    domain::{
        AccentTone, AdventurerClass, AdventuringGear, Ancestry, BodyProportions, FaceDetail,
        Footwear, Garb, HairShape, HairTone, HeadShape, Keepsake, Legwear, SkinTone,
    },
    ui::{
        delve_scene::DelveVariant,
        goblins::GoblinSighting,
        guild_room_projection::{GuildLandmarkKind, GuildRoomMode, TruthfulStationKind},
        pixel::ColorRole,
        theatre::TheatrePose,
    },
};

pub type LandmarkAsset = GuildLandmarkKind;
pub type TruthfulStationAsset = TruthfulStationKind;
pub type RoomCameraAsset = GuildRoomMode;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AssetId {
    Class(AdventurerClass),
    Gear(AdventuringGear),
    Ancestry(Ancestry),
    BodyProportions(BodyProportions),
    HeadShape(HeadShape),
    SkinTone(SkinTone),
    HairShape(HairShape),
    HairTone(HairTone),
    FaceDetail(FaceDetail),
    Garb(Garb),
    Legwear(Legwear),
    Footwear(Footwear),
    Keepsake(Keepsake),
    AccentTone(AccentTone),
    ColorRole(ColorRole),
    Pose(TheatrePose),
    DelveVariant(DelveVariant),
    GoblinSighting(GoblinSighting),
    GoblinOutbreak,
    Landmark(LandmarkAsset),
    TruthfulStation(TruthfulStationAsset),
    RoomCamera(RoomCameraAsset),
    Widget(WidgetAsset),
    Scene(SceneAsset),
    SceneFirst(SceneFirstAsset),
    Compatibility(CompatibilityAsset),
}

macro_rules! storybook_asset_enum {
    ($name:ident { $($variant:ident),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub enum $name { $($variant),+ }

        impl $name {
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];
        }
    };
}

storybook_asset_enum!(WidgetAsset {
    AdventurerCardFull,
    AdventurerCardCompact,
    ChamberFull,
    ChamberCompact,
    QuestBoard,
    Party,
    Summons,
    Chronicle,
    AdventurerProfile,
    Scrying,
    Spoils,
    Counsel,
    Search,
    Help,
});

storybook_asset_enum!(SceneAsset {
    GuildEmpty,
    GuildPopulated,
    GuildOneCampaign,
    GuildMixedAttention,
    GuildDisconnected,
    GuildConnecting,
    GuildReconnecting,
    GuildIncompatible,
    GuildReviewrUnavailable,
    GuildScryingFailed,
    GuildCroppedRoom,
    GuildLandmarkCamera,
    ConnectedDelves,
    MixedStateDelve,
    NarrowGuild,
    NarrowDelve,
});

storybook_asset_enum!(CompatibilityAsset {
    UnicodeXterm256,
    UnicodeAnsi16,
    AsciiAnsi16,
    MotionFull,
    MotionReduced,
    MotionNone,
});

storybook_asset_enum!(SceneFirstAsset {
    CalibrationRoom,
    CompactAdventurers,
});

impl AssetId {
    #[must_use]
    #[allow(
        clippy::too_many_lines,
        reason = "one exhaustive match is the compile-time gate for every asset label"
    )]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Class(value) => match value {
                AdventurerClass::Barbarian => "class: barbarian",
                AdventurerClass::Bard => "class: bard",
                AdventurerClass::Cleric => "class: cleric",
                AdventurerClass::Paladin => "class: paladin",
                AdventurerClass::Ranger => "class: ranger",
                AdventurerClass::Rogue => "class: rogue",
                AdventurerClass::Wizard => "class: wizard",
                AdventurerClass::Artificer => "class: artificer",
                AdventurerClass::Runewright => "class: runewright",
                AdventurerClass::Testmender => "class: testmender",
                AdventurerClass::Pathseeker => "class: pathseeker",
            },
            Self::Gear(value) => match value {
                AdventuringGear::Axe => "gear: axe",
                AdventuringGear::BowAndQuiver => "gear: bow and quiver",
                AdventuringGear::HolySymbol => "gear: holy symbol",
                AdventuringGear::Lute => "gear: lute",
                AdventuringGear::MapAndCompass => "gear: map and compass",
                AdventuringGear::RuneChisel => "gear: rune chisel",
                AdventuringGear::Shield => "gear: shield",
                AdventuringGear::SpellbookAndStaff => "gear: spellbook and staff",
                AdventuringGear::TestKit => "gear: test kit",
                AdventuringGear::ThievesTools => "gear: thieves' tools",
                AdventuringGear::Toolkit => "gear: toolkit",
            },
            Self::Ancestry(value) => match value {
                Ancestry::Human => "ancestry: human",
                Ancestry::Dwarf => "ancestry: dwarf",
                Ancestry::Elf => "ancestry: elf",
                Ancestry::Halfling => "ancestry: halfling",
                Ancestry::Orc => "ancestry: orc",
                Ancestry::Gnome => "ancestry: gnome",
                Ancestry::Goblin => "ancestry: goblin",
            },
            Self::BodyProportions(value) => match value {
                BodyProportions::Compact => "body proportions: compact",
                BodyProportions::Average => "body proportions: average",
                BodyProportions::Tall => "body proportions: tall",
                BodyProportions::Broad => "body proportions: broad",
            },
            Self::HeadShape(value) => match value {
                HeadShape::Round => "head shape: round",
                HeadShape::Square => "head shape: square",
                HeadShape::Long => "head shape: long",
                HeadShape::Angular => "head shape: angular",
            },
            Self::SkinTone(value) => match value {
                SkinTone::Porcelain => "skin tone: porcelain",
                SkinTone::Rose => "skin tone: rose",
                SkinTone::Sand => "skin tone: sand",
                SkinTone::Umber => "skin tone: umber",
                SkinTone::Sienna => "skin tone: sienna",
                SkinTone::Ebony => "skin tone: ebony",
            },
            Self::HairShape(value) => match value {
                HairShape::Crop => "hair shape: crop",
                HairShape::Fringe => "hair shape: fringe",
                HairShape::Curls => "hair shape: curls",
                HairShape::Quiff => "hair shape: quiff",
                HairShape::Bob => "hair shape: bob",
                HairShape::Spikes => "hair shape: spikes",
                HairShape::Ponytail => "hair shape: ponytail",
                HairShape::Shaved => "hair shape: shaved",
            },
            Self::HairTone(value) => match value {
                HairTone::Black => "hair tone: black",
                HairTone::Espresso => "hair tone: espresso",
                HairTone::Chestnut => "hair tone: chestnut",
                HairTone::Copper => "hair tone: copper",
                HairTone::Gold => "hair tone: gold",
                HairTone::Silver => "hair tone: silver",
            },
            Self::FaceDetail(value) => match value {
                FaceDetail::None => "face detail: none",
                FaceDetail::RoundGlasses => "face detail: round glasses",
                FaceDetail::SquareGlasses => "face detail: square glasses",
                FaceDetail::Visor => "face detail: visor",
                FaceDetail::Freckles => "face detail: freckles",
                FaceDetail::Moustache => "face detail: moustache",
            },
            Self::Garb(value) => match value {
                Garb::Armour => "garb: armour",
                Garb::Cloak => "garb: cloak",
                Garb::Doublet => "garb: doublet",
                Garb::Leathers => "garb: leathers",
                Garb::Robes => "garb: robes",
                Garb::Vestments => "garb: vestments",
                Garb::WorkApron => "garb: work apron",
            },
            Self::Legwear(value) => match value {
                Legwear::BootsAndBreeches => "legwear: boots and breeches",
                Legwear::Greaves => "legwear: greaves",
                Legwear::RobeHem => "legwear: robe hem",
                Legwear::TravelingSkirt => "legwear: traveling skirt",
            },
            Self::Footwear(value) => match value {
                Footwear::Boots => "footwear: boots",
                Footwear::Sabatons => "footwear: sabatons",
                Footwear::Sandals => "footwear: sandals",
                Footwear::SoftShoes => "footwear: soft shoes",
            },
            Self::Keepsake(value) => match value {
                Keepsake::Feather => "keepsake: feather",
                Keepsake::LuckyCoin => "keepsake: lucky coin",
                Keepsake::Mug => "keepsake: mug",
                Keepsake::PressedLeaf => "keepsake: pressed leaf",
                Keepsake::Ribbon => "keepsake: ribbon",
                Keepsake::TinyFamiliar => "keepsake: tiny familiar",
            },
            Self::AccentTone(value) => match value {
                AccentTone::Amber => "accent tone: amber",
                AccentTone::Cyan => "accent tone: cyan",
                AccentTone::Lime => "accent tone: lime",
                AccentTone::Magenta => "accent tone: magenta",
                AccentTone::Red => "accent tone: red",
                AccentTone::Blue => "accent tone: blue",
                AccentTone::Violet => "accent tone: violet",
                AccentTone::Teal => "accent tone: teal",
            },
            Self::ColorRole(value) => match value {
                ColorRole::Stone => "color role: stone",
                ColorRole::DarkStone => "color role: dark stone",
                ColorRole::Timber => "color role: timber",
                ColorRole::Parchment => "color role: parchment",
                ColorRole::Ink => "color role: ink",
                ColorRole::Hearth => "color role: hearth",
                ColorRole::Moss => "color role: moss",
                ColorRole::RuneGlow => "color role: rune glow",
                ColorRole::Counsel => "color role: counsel",
                ColorRole::Spoils => "color role: spoils",
                ColorRole::Selection => "color role: selection",
                ColorRole::Fog => "color role: fog",
                ColorRole::Goblin => "color role: goblin",
                ColorRole::SkinLight => "color role: skin light",
                ColorRole::SkinMedium => "color role: skin medium",
                ColorRole::SkinDark => "color role: skin dark",
                ColorRole::HairDark => "color role: hair dark",
                ColorRole::HairLight => "color role: hair light",
                ColorRole::Leather => "color role: leather",
                ColorRole::Steel => "color role: steel",
                ColorRole::ClothWarm => "color role: cloth warm",
                ColorRole::ClothCool => "color role: cloth cool",
            },
            Self::Pose(value) => match value {
                TheatrePose::Delving => "pose: delving",
                TheatrePose::SeekingCounsel => "pose: seeking counsel",
                TheatrePose::SpoilsUnopened => "pose: spoils unopened",
                TheatrePose::VictoryRecorded => "pose: victory recorded",
                TheatrePose::Resting => "pose: resting",
                TheatrePose::Departed => "pose: departed",
                TheatrePose::Unknown => "pose: unknown",
            },
            Self::DelveVariant(value) => match value {
                DelveVariant::ForgottenLibrary => "delve variant: forgotten library",
                DelveVariant::MossyUndercroft => "delve variant: mossy undercroft",
                DelveVariant::OldWatchtower => "delve variant: old watchtower",
            },
            Self::GoblinSighting(value) => match value {
                GoblinSighting::ChestEyes => "goblin sighting: chest eyes",
                GoblinSighting::ChronicleHand => "goblin sighting: chronicle hand",
                GoblinSighting::RaftersScroll => "goblin sighting: rafters scroll",
                GoblinSighting::StolenBiscuit => "goblin sighting: stolen biscuit",
            },
            Self::GoblinOutbreak => "goblin outbreak",
            Self::Landmark(value) => match value {
                LandmarkAsset::GuildDoor => "landmark: guild door",
                LandmarkAsset::QuestWall => "landmark: quest wall",
                LandmarkAsset::CampaignTable => "landmark: campaign table",
                LandmarkAsset::CounselBell => "landmark: counsel bell",
                LandmarkAsset::Hearth => "landmark: hearth",
                LandmarkAsset::ChronicleLectern => "landmark: chronicle lectern",
                LandmarkAsset::ScryingAlcove => "landmark: scrying alcove",
                LandmarkAsset::SpoilsVault => "landmark: spoils vault",
            },
            Self::TruthfulStation(value) => match value {
                TruthfulStationAsset::CampaignToken => "truthful station: campaign token",
                TruthfulStationAsset::CounselProjection => "truthful station: counsel projection",
                TruthfulStationAsset::HearthAdventurer => "truthful station: hearth adventurer",
                TruthfulStationAsset::SpoilsAdventurer => "truthful station: spoils adventurer",
            },
            Self::RoomCamera(value) => match value {
                RoomCameraAsset::WholeRoom => "room camera: whole room",
                RoomCameraAsset::CroppedRoom => "room camera: cropped room",
                RoomCameraAsset::LandmarkCamera => "room camera: landmark",
            },
            Self::Widget(value) => match value {
                WidgetAsset::AdventurerCardFull => "widget: adventurer card full",
                WidgetAsset::AdventurerCardCompact => "widget: adventurer card compact",
                WidgetAsset::ChamberFull => "widget: chamber full",
                WidgetAsset::ChamberCompact => "widget: chamber compact",
                WidgetAsset::QuestBoard => "widget: quest board",
                WidgetAsset::Party => "widget: party",
                WidgetAsset::Summons => "widget: summons",
                WidgetAsset::Chronicle => "widget: chronicle",
                WidgetAsset::AdventurerProfile => "widget: adventurer profile",
                WidgetAsset::Scrying => "widget: scrying",
                WidgetAsset::Spoils => "widget: spoils",
                WidgetAsset::Counsel => "widget: counsel",
                WidgetAsset::Search => "widget: search",
                WidgetAsset::Help => "widget: help",
            },
            Self::Scene(value) => match value {
                SceneAsset::GuildEmpty => "scene: guild empty",
                SceneAsset::GuildPopulated => "scene: guild populated",
                SceneAsset::GuildOneCampaign => "scene: guild one campaign",
                SceneAsset::GuildMixedAttention => "scene: guild mixed attention",
                SceneAsset::GuildDisconnected => "scene: guild disconnected",
                SceneAsset::GuildConnecting => "scene: guild connecting",
                SceneAsset::GuildReconnecting => "scene: guild reconnecting",
                SceneAsset::GuildIncompatible => "scene: guild incompatible",
                SceneAsset::GuildReviewrUnavailable => "scene: guild Reviewr unavailable",
                SceneAsset::GuildScryingFailed => "scene: guild Scrying failed",
                SceneAsset::GuildCroppedRoom => "scene: guild cropped room",
                SceneAsset::GuildLandmarkCamera => "scene: guild landmark camera",
                SceneAsset::ConnectedDelves => "scene: connected delves",
                SceneAsset::MixedStateDelve => "scene: mixed-state delve",
                SceneAsset::NarrowGuild => "scene: narrow guild",
                SceneAsset::NarrowDelve => "scene: narrow delve",
            },
            Self::SceneFirst(value) => match value {
                SceneFirstAsset::CalibrationRoom => "scene first: RGB calibration room",
                SceneFirstAsset::CompactAdventurers => "scene first: compact adventurers",
            },
            Self::Compatibility(value) => match value {
                CompatibilityAsset::UnicodeXterm256 => "compatibility: unicode xterm-256",
                CompatibilityAsset::UnicodeAnsi16 => "compatibility: unicode ANSI 16",
                CompatibilityAsset::AsciiAnsi16 => "compatibility: ASCII ANSI 16",
                CompatibilityAsset::MotionFull => "compatibility: full motion",
                CompatibilityAsset::MotionReduced => "compatibility: reduced motion",
                CompatibilityAsset::MotionNone => "compatibility: no motion",
            },
        }
    }
}

macro_rules! asset_family {
    ($visibility:vis $constant:ident, $builder:ident, $type:ty, $variant:ident) => {
        const fn $builder() -> [AssetId; <$type>::ALL.len()] {
            let mut assets = [AssetId::$variant(<$type>::ALL[0]); <$type>::ALL.len()];
            let mut index = 0;
            while index < <$type>::ALL.len() {
                assets[index] = AssetId::$variant(<$type>::ALL[index]);
                index += 1;
            }
            assets
        }
        $visibility const $constant: &[AssetId] = &$builder();
    };
}

asset_family!(pub(super) CLASSES, class_assets, AdventurerClass, Class);
asset_family!(pub(super) GEAR, gear_assets, AdventuringGear, Gear);
asset_family!(pub(super) ANCESTRIES, ancestry_assets, Ancestry, Ancestry);
asset_family!(
    pub(super) BODY_PROPORTIONS,
    body_proportion_assets,
    BodyProportions,
    BodyProportions
);
asset_family!(pub(super) HEAD_SHAPES, head_shape_assets, HeadShape, HeadShape);
asset_family!(pub(super) SKIN_TONES, skin_tone_assets, SkinTone, SkinTone);
asset_family!(pub(super) HAIR_SHAPES, hair_shape_assets, HairShape, HairShape);
asset_family!(pub(super) HAIR_TONES, hair_tone_assets, HairTone, HairTone);
asset_family!(pub(super) FACE_DETAILS, face_detail_assets, FaceDetail, FaceDetail);
asset_family!(pub(super) GARBS, garb_assets, Garb, Garb);
asset_family!(pub(super) LEGWEAR, legwear_assets, Legwear, Legwear);
asset_family!(pub(super) FOOTWEAR, footwear_assets, Footwear, Footwear);
asset_family!(pub(super) KEEPSAKES, keepsake_assets, Keepsake, Keepsake);
asset_family!(pub(super) ACCENT_TONES, accent_tone_assets, AccentTone, AccentTone);
asset_family!(pub(super) COLOR_ROLES, color_role_assets, ColorRole, ColorRole);
asset_family!(pub(super) POSES, pose_assets, TheatrePose, Pose);
asset_family!(
    DELVE_VARIANTS,
    delve_variant_assets,
    DelveVariant,
    DelveVariant
);
asset_family!(
    GOBLIN_SIGHTINGS,
    goblin_sighting_assets,
    GoblinSighting,
    GoblinSighting
);
const GOBLIN_OUTBREAK: &[AssetId] = &[AssetId::GoblinOutbreak];
asset_family!(WIDGETS, widget_assets, WidgetAsset, Widget);
asset_family!(pub(super) LANDMARKS, landmark_assets, LandmarkAsset, Landmark);
asset_family!(
    pub(super)
    TRUTHFUL_STATIONS,
    truthful_station_assets,
    TruthfulStationAsset,
    TruthfulStation
);
asset_family!(
    pub(super)
    ROOM_CAMERAS,
    room_camera_assets,
    RoomCameraAsset,
    RoomCamera
);
asset_family!(SCENES, scene_assets, SceneAsset, Scene);
asset_family!(SCENE_FIRST, scene_first_assets, SceneFirstAsset, SceneFirst);
asset_family!(
    COMPATIBILITY,
    compatibility_assets,
    CompatibilityAsset,
    Compatibility
);

#[must_use]
pub fn asset_inventory() -> Vec<AssetId> {
    let inventory = [
        CLASSES,
        GEAR,
        ANCESTRIES,
        BODY_PROPORTIONS,
        HEAD_SHAPES,
        SKIN_TONES,
        HAIR_SHAPES,
        HAIR_TONES,
        FACE_DETAILS,
        GARBS,
        LEGWEAR,
        FOOTWEAR,
        KEEPSAKES,
        ACCENT_TONES,
        COLOR_ROLES,
        POSES,
        DELVE_VARIANTS,
        GOBLIN_SIGHTINGS,
        GOBLIN_OUTBREAK,
        LANDMARKS,
        TRUTHFUL_STATIONS,
        ROOM_CAMERAS,
        WIDGETS,
        SCENES,
        SCENE_FIRST,
        COMPATIBILITY,
    ]
    .into_iter()
    .flatten()
    .copied()
    .collect::<Vec<_>>();

    let unique = inventory.iter().copied().collect::<HashSet<_>>();
    debug_assert_eq!(
        inventory.len(),
        unique.len(),
        "duplicate Storybook asset IDs"
    );
    inventory
}
