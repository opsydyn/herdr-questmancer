use std::collections::HashSet;

use crate::{
    domain::{
        AccentTone, AdventurerClass, AdventuringGear, Ancestry, BodyProportions, FaceDetail,
        Footwear, Garb, HairShape, HairTone, HeadShape, Keepsake, Legwear, SkinTone,
    },
    ui::{
        delve_scene::DelveVariant, goblins::GoblinSighting, pixel::ColorRole, theatre::TheatrePose,
    },
};

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
    Widget(WidgetAsset),
    Scene(SceneAsset),
    Compatibility(CompatibilityAsset),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WidgetAsset {
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
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SceneAsset {
    GuildEmpty,
    GuildPopulated,
    GuildMixedAttention,
    GuildDisconnected,
    GuildReconnecting,
    ConnectedDelves,
    MixedStateDelve,
    NarrowGuild,
    NarrowDelve,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CompatibilityAsset {
    UnicodeXterm256,
    UnicodeAnsi16,
    AsciiAnsi16,
    MotionFull,
    MotionReduced,
    MotionNone,
}

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
                SceneAsset::GuildMixedAttention => "scene: guild mixed attention",
                SceneAsset::GuildDisconnected => "scene: guild disconnected",
                SceneAsset::GuildReconnecting => "scene: guild reconnecting",
                SceneAsset::ConnectedDelves => "scene: connected delves",
                SceneAsset::MixedStateDelve => "scene: mixed-state delve",
                SceneAsset::NarrowGuild => "scene: narrow guild",
                SceneAsset::NarrowDelve => "scene: narrow delve",
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

const CLASSES: &[AssetId] = &[
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
];
const GEAR: &[AssetId] = &[
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
const ANCESTRIES: &[AssetId] = &[
    AssetId::Ancestry(Ancestry::Human),
    AssetId::Ancestry(Ancestry::Dwarf),
    AssetId::Ancestry(Ancestry::Elf),
    AssetId::Ancestry(Ancestry::Halfling),
    AssetId::Ancestry(Ancestry::Orc),
    AssetId::Ancestry(Ancestry::Gnome),
    AssetId::Ancestry(Ancestry::Goblin),
];
const BODY_PROPORTIONS: &[AssetId] = &[
    AssetId::BodyProportions(BodyProportions::Compact),
    AssetId::BodyProportions(BodyProportions::Average),
    AssetId::BodyProportions(BodyProportions::Tall),
    AssetId::BodyProportions(BodyProportions::Broad),
];
const HEAD_SHAPES: &[AssetId] = &[
    AssetId::HeadShape(HeadShape::Round),
    AssetId::HeadShape(HeadShape::Square),
    AssetId::HeadShape(HeadShape::Long),
    AssetId::HeadShape(HeadShape::Angular),
];
const SKIN_TONES: &[AssetId] = &[
    AssetId::SkinTone(SkinTone::Porcelain),
    AssetId::SkinTone(SkinTone::Rose),
    AssetId::SkinTone(SkinTone::Sand),
    AssetId::SkinTone(SkinTone::Umber),
    AssetId::SkinTone(SkinTone::Sienna),
    AssetId::SkinTone(SkinTone::Ebony),
];
const HAIR_SHAPES: &[AssetId] = &[
    AssetId::HairShape(HairShape::Crop),
    AssetId::HairShape(HairShape::Fringe),
    AssetId::HairShape(HairShape::Curls),
    AssetId::HairShape(HairShape::Quiff),
    AssetId::HairShape(HairShape::Bob),
    AssetId::HairShape(HairShape::Spikes),
    AssetId::HairShape(HairShape::Ponytail),
    AssetId::HairShape(HairShape::Shaved),
];
const HAIR_TONES: &[AssetId] = &[
    AssetId::HairTone(HairTone::Black),
    AssetId::HairTone(HairTone::Espresso),
    AssetId::HairTone(HairTone::Chestnut),
    AssetId::HairTone(HairTone::Copper),
    AssetId::HairTone(HairTone::Gold),
    AssetId::HairTone(HairTone::Silver),
];
const FACE_DETAILS: &[AssetId] = &[
    AssetId::FaceDetail(FaceDetail::None),
    AssetId::FaceDetail(FaceDetail::RoundGlasses),
    AssetId::FaceDetail(FaceDetail::SquareGlasses),
    AssetId::FaceDetail(FaceDetail::Visor),
    AssetId::FaceDetail(FaceDetail::Freckles),
    AssetId::FaceDetail(FaceDetail::Moustache),
];
const GARBS: &[AssetId] = &[
    AssetId::Garb(Garb::Armour),
    AssetId::Garb(Garb::Cloak),
    AssetId::Garb(Garb::Doublet),
    AssetId::Garb(Garb::Leathers),
    AssetId::Garb(Garb::Robes),
    AssetId::Garb(Garb::Vestments),
    AssetId::Garb(Garb::WorkApron),
];
const LEGWEAR: &[AssetId] = &[
    AssetId::Legwear(Legwear::BootsAndBreeches),
    AssetId::Legwear(Legwear::Greaves),
    AssetId::Legwear(Legwear::RobeHem),
    AssetId::Legwear(Legwear::TravelingSkirt),
];
const FOOTWEAR: &[AssetId] = &[
    AssetId::Footwear(Footwear::Boots),
    AssetId::Footwear(Footwear::Sabatons),
    AssetId::Footwear(Footwear::Sandals),
    AssetId::Footwear(Footwear::SoftShoes),
];
const KEEPSAKES: &[AssetId] = &[
    AssetId::Keepsake(Keepsake::Feather),
    AssetId::Keepsake(Keepsake::LuckyCoin),
    AssetId::Keepsake(Keepsake::Mug),
    AssetId::Keepsake(Keepsake::PressedLeaf),
    AssetId::Keepsake(Keepsake::Ribbon),
    AssetId::Keepsake(Keepsake::TinyFamiliar),
];
const ACCENT_TONES: &[AssetId] = &[
    AssetId::AccentTone(AccentTone::Amber),
    AssetId::AccentTone(AccentTone::Cyan),
    AssetId::AccentTone(AccentTone::Lime),
    AssetId::AccentTone(AccentTone::Magenta),
    AssetId::AccentTone(AccentTone::Red),
    AssetId::AccentTone(AccentTone::Blue),
    AssetId::AccentTone(AccentTone::Violet),
    AssetId::AccentTone(AccentTone::Teal),
];
const COLOR_ROLES: &[AssetId] = &[
    AssetId::ColorRole(ColorRole::Stone),
    AssetId::ColorRole(ColorRole::DarkStone),
    AssetId::ColorRole(ColorRole::Timber),
    AssetId::ColorRole(ColorRole::Parchment),
    AssetId::ColorRole(ColorRole::Ink),
    AssetId::ColorRole(ColorRole::Hearth),
    AssetId::ColorRole(ColorRole::Moss),
    AssetId::ColorRole(ColorRole::RuneGlow),
    AssetId::ColorRole(ColorRole::Counsel),
    AssetId::ColorRole(ColorRole::Spoils),
    AssetId::ColorRole(ColorRole::Selection),
    AssetId::ColorRole(ColorRole::Fog),
    AssetId::ColorRole(ColorRole::Goblin),
    AssetId::ColorRole(ColorRole::SkinLight),
    AssetId::ColorRole(ColorRole::SkinMedium),
    AssetId::ColorRole(ColorRole::SkinDark),
    AssetId::ColorRole(ColorRole::HairDark),
    AssetId::ColorRole(ColorRole::HairLight),
    AssetId::ColorRole(ColorRole::Leather),
    AssetId::ColorRole(ColorRole::Steel),
    AssetId::ColorRole(ColorRole::ClothWarm),
    AssetId::ColorRole(ColorRole::ClothCool),
];
const POSES: &[AssetId] = &[
    AssetId::Pose(TheatrePose::Delving),
    AssetId::Pose(TheatrePose::SeekingCounsel),
    AssetId::Pose(TheatrePose::SpoilsUnopened),
    AssetId::Pose(TheatrePose::VictoryRecorded),
    AssetId::Pose(TheatrePose::Resting),
    AssetId::Pose(TheatrePose::Departed),
    AssetId::Pose(TheatrePose::Unknown),
];
const DELVE_VARIANTS: &[AssetId] = &[
    AssetId::DelveVariant(DelveVariant::ForgottenLibrary),
    AssetId::DelveVariant(DelveVariant::MossyUndercroft),
    AssetId::DelveVariant(DelveVariant::OldWatchtower),
];
const GOBLIN_SIGHTINGS: &[AssetId] = &[
    AssetId::GoblinSighting(GoblinSighting::ChestEyes),
    AssetId::GoblinSighting(GoblinSighting::ChronicleHand),
    AssetId::GoblinSighting(GoblinSighting::RaftersScroll),
    AssetId::GoblinSighting(GoblinSighting::StolenBiscuit),
];
const GOBLIN_OUTBREAK: &[AssetId] = &[AssetId::GoblinOutbreak];
const WIDGETS: &[AssetId] = &[
    AssetId::Widget(WidgetAsset::AdventurerCardFull),
    AssetId::Widget(WidgetAsset::AdventurerCardCompact),
    AssetId::Widget(WidgetAsset::ChamberFull),
    AssetId::Widget(WidgetAsset::ChamberCompact),
    AssetId::Widget(WidgetAsset::QuestBoard),
    AssetId::Widget(WidgetAsset::Party),
    AssetId::Widget(WidgetAsset::Summons),
    AssetId::Widget(WidgetAsset::Chronicle),
    AssetId::Widget(WidgetAsset::AdventurerProfile),
    AssetId::Widget(WidgetAsset::Scrying),
    AssetId::Widget(WidgetAsset::Spoils),
    AssetId::Widget(WidgetAsset::Counsel),
    AssetId::Widget(WidgetAsset::Search),
    AssetId::Widget(WidgetAsset::Help),
];
const SCENES: &[AssetId] = &[
    AssetId::Scene(SceneAsset::GuildEmpty),
    AssetId::Scene(SceneAsset::GuildPopulated),
    AssetId::Scene(SceneAsset::GuildMixedAttention),
    AssetId::Scene(SceneAsset::GuildDisconnected),
    AssetId::Scene(SceneAsset::GuildReconnecting),
    AssetId::Scene(SceneAsset::ConnectedDelves),
    AssetId::Scene(SceneAsset::MixedStateDelve),
    AssetId::Scene(SceneAsset::NarrowGuild),
    AssetId::Scene(SceneAsset::NarrowDelve),
];
const COMPATIBILITY: &[AssetId] = &[
    AssetId::Compatibility(CompatibilityAsset::UnicodeXterm256),
    AssetId::Compatibility(CompatibilityAsset::UnicodeAnsi16),
    AssetId::Compatibility(CompatibilityAsset::AsciiAnsi16),
    AssetId::Compatibility(CompatibilityAsset::MotionFull),
    AssetId::Compatibility(CompatibilityAsset::MotionReduced),
    AssetId::Compatibility(CompatibilityAsset::MotionNone),
];

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
        WIDGETS,
        SCENES,
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
