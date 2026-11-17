use std::{collections::HashMap, fmt, sync::OnceLock};

use crate::domain::AdventurerClass;

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

// Scene fixtures delegate every positive size to the production renderer.
// Only authored asset galleries need a minimum canvas to preserve their layout.
const WORLD_VIEWPORT: Viewport = Viewport::new(160, 45, 1, 1);
const CARD_VIEWPORT: Viewport = Viewport::new(120, 36, 1, 1);
const ASSET_VIEWPORT: Viewport = Viewport::new(120, 36, 80, 28);
const NARROW_VIEWPORT: Viewport = Viewport::new(64, 24, 1, 1);

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

fn fit_stories() -> Vec<Story> {
    vec![
        Story {
            id: StoryId::new("world.guild-fit"),
            title: "Prototype / Hall Fit — Five",
            category: Category::Worlds,
            description: "Storybook-only native fit. Enter inspect; Tab selects; c card; L Ledger; click actors.",
            viewport: WORLD_VIEWPORT,
            build: guild_fit,
            owns: &[],
            shows: &[AssetId(SceneFirstAsset::GuildHall)],
        },
        Story {
            id: StoryId::new("world.guild-fit-eleven"),
            title: "Prototype / Hall Fit — Eleven",
            category: Category::Worlds,
            description: "Distributed eleven-person fixture; all actors must fit before native output is allowed.",
            viewport: WORLD_VIEWPORT,
            build: guild_fit_eleven,
            owns: &[],
            shows: &[AssetId(SceneFirstAsset::GuildHall)],
        },
        Story {
            id: StoryId::new("world.guild-fit-crowded"),
            title: "Prototype / Hall Fit — Crowded",
            category: Category::Worlds,
            description: "Eleven full masters retain the complete fitted Hall even when source stations are crowded.",
            viewport: WORLD_VIEWPORT,
            build: guild_fit_crowded,
            owns: &[],
            shows: &[AssetId(SceneFirstAsset::GuildHall)],
        },
        Story {
            id: StoryId::new("world.delve-fit"),
            title: "Fit / Delve — Five",
            category: Category::Worlds,
            description: "Complete dungeon at integer pixel scale. Enter inspect; Tab selects; c card; click actors.",
            viewport: WORLD_VIEWPORT,
            build: delve_fit,
            owns: &[],
            shows: &[AssetId(SceneFirstAsset::Delve)],
        },
        Story {
            id: StoryId::new("world.delve-fit-crowded"),
            title: "Fit / Delve — Crowded",
            category: Category::Worlds,
            description: "Automatic density retains eleven full masters in steady distributed dungeon seats.",
            viewport: WORLD_VIEWPORT,
            build: delve_fit_crowded,
            owns: &[],
            shows: &[AssetId(SceneFirstAsset::Delve)],
        },
        Story {
            id: StoryId::new("world.delve-fit-resting"),
            title: "Fit / Delve — Resting Crowd",
            category: Category::Worlds,
            description: "Four resting adventurers plus one unknown retain their full masters and the dungeon backdrop.",
            viewport: WORLD_VIEWPORT,
            build: delve_fit_resting,
            owns: &[],
            shows: &[AssetId(SceneFirstAsset::Delve)],
        },
        Story {
            id: StoryId::new("world.guild-fit-profile"),
            title: "Fit / Hall — Profile Open",
            category: Category::Worlds,
            description: "The existing profile overlays the fitted Hall without moving its camera or changing scale.",
            viewport: WORLD_VIEWPORT,
            build: guild_fit_profile,
            owns: &[],
            shows: &[AssetId(SceneFirstAsset::GuildHall)],
        },
        Story {
            id: StoryId::new("world.delve-fit-profile"),
            title: "Fit / Delve — Profile Open",
            category: Category::Worlds,
            description: "The existing profile overlays the fitted dungeon; resize or Esc without a rendering-mode jump.",
            viewport: WORLD_VIEWPORT,
            build: delve_fit_profile,
            owns: &[],
            shows: &[AssetId(SceneFirstAsset::Delve)],
        },
    ]
}

fn capacity_stories() -> Vec<Story> {
    use super::density::{PartySize, fixture};
    use crate::app::View;
    let entries: [(&'static str, &'static str, StoryBuilder, SceneFirstAsset); 8] = [
        (
            "world.capacity-hall-7-large",
            "Experiment / Hall capacity / 7 large",
            |context| {
                StoryFixture::WorldFit(Box::new(fixture(context, PartySize::Seven, View::Guild)))
            },
            SceneFirstAsset::GuildHall,
        ),
        (
            "world.capacity-hall-7-small",
            "Experiment / Hall capacity / 7 small",
            |context| {
                StoryFixture::WorldFit(Box::new(fixture(context, PartySize::Seven, View::Guild)))
            },
            SceneFirstAsset::GuildHall,
        ),
        (
            "world.capacity-hall-24-small",
            "Experiment / Hall capacity / 24 small",
            |context| {
                StoryFixture::WorldFit(Box::new(fixture(
                    context,
                    PartySize::TwentyFour,
                    View::Guild,
                )))
            },
            SceneFirstAsset::GuildHall,
        ),
        (
            "world.capacity-hall-32-small",
            "Experiment / Hall capacity / 32 small",
            |context| {
                StoryFixture::WorldFit(Box::new(fixture(
                    context,
                    PartySize::ThirtyTwo,
                    View::Guild,
                )))
            },
            SceneFirstAsset::GuildHall,
        ),
        (
            "world.capacity-delve-7-large",
            "Experiment / Delve capacity / 7 large",
            |context| {
                StoryFixture::WorldFit(Box::new(fixture(context, PartySize::Seven, View::Delve)))
            },
            SceneFirstAsset::Delve,
        ),
        (
            "world.capacity-delve-7-small",
            "Experiment / Delve capacity / 7 small",
            |context| {
                StoryFixture::WorldFit(Box::new(fixture(context, PartySize::Seven, View::Delve)))
            },
            SceneFirstAsset::Delve,
        ),
        (
            "world.capacity-delve-24-small",
            "Experiment / Delve capacity / 24 small",
            |context| {
                StoryFixture::WorldFit(Box::new(fixture(
                    context,
                    PartySize::TwentyFour,
                    View::Delve,
                )))
            },
            SceneFirstAsset::Delve,
        ),
        (
            "world.capacity-delve-32-small",
            "Experiment / Delve capacity / 32 small",
            |context| {
                StoryFixture::WorldFit(Box::new(fixture(
                    context,
                    PartySize::ThirtyTwo,
                    View::Delve,
                )))
            },
            SceneFirstAsset::Delve,
        ),
    ];
    entries.into_iter().map(|(id,title,build,asset)| Story {
        id: StoryId::new(id), title, category: Category::Worlds,
        description: "Experimental distributed seating; existing full or authored small sprites. Tab selects; c profile; click actors. Production unchanged.",
        viewport: WORLD_VIEWPORT, build, owns: &[],
        shows: if asset == SceneFirstAsset::GuildHall { &[AssetId(SceneFirstAsset::GuildHall)] } else { &[AssetId(SceneFirstAsset::Delve)] },
    }).collect()
}

fn automatic_density_stories() -> [Story; 2] {
    use super::density::{PartySize, fixture};
    use crate::app::View;
    [
        Story {
            id: StoryId::new("world.guild-fit-auto"),
            title: "Fit / Hall / Automatic density — 32",
            category: Category::Worlds,
            description: "Production automatic density: 32 smaller authored adventurers in the complete Hall. Tab selects; c profile; click actors.",
            viewport: WORLD_VIEWPORT,
            build: |context| {
                StoryFixture::WorldFit(Box::new(fixture(
                    context,
                    PartySize::ThirtyTwo,
                    View::Guild,
                )))
            },
            owns: &[],
            shows: &[AssetId(SceneFirstAsset::GuildHall)],
        },
        Story {
            id: StoryId::new("world.delve-fit-auto"),
            title: "Fit / Delve / Automatic density — 32",
            category: Category::Worlds,
            description: "Production automatic density: 32 smaller authored adventurers in the complete dungeon. Tab selects; c profile; click actors.",
            viewport: WORLD_VIEWPORT,
            build: |context| {
                StoryFixture::WorldFit(Box::new(fixture(
                    context,
                    PartySize::ThirtyTwo,
                    View::Delve,
                )))
            },
            owns: &[],
            shows: &[AssetId(SceneFirstAsset::Delve)],
        },
    ]
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
    stories.splice(1..1, fit_stories());
    stories.extend(capacity_stories());
    stories.extend(automatic_density_stories());
    stories.extend(board_stories());
    stories.extend(table_stories());
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
            "asset.barbarian-poses",
            "Assets / Barbarian Poses",
            Category::Assets,
            "Production working, counsel, spoils and quiet poses for the Barbarian.",
            ASSET_VIEWPORT,
            barbarian_poses,
            BarbarianPoseFamily
        ),
        story!(
            "asset.wizard-poses",
            "Assets / Wizard Poses",
            Category::Assets,
            "Production working, counsel, spoils and quiet poses for the Wizard.",
            ASSET_VIEWPORT,
            wizard_poses,
            WizardPoseFamily
        ),
        story!(
            "asset.ranger-poses",
            "Assets / Ranger Poses",
            Category::Assets,
            "Production working, counsel, spoils and quiet poses for the Ranger.",
            ASSET_VIEWPORT,
            ranger_poses,
            RangerPoseFamily
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
    stories.splice(4..4, tool_ritual_stories());
    stories.splice(7..7, support_ritual_stories());
    stories.splice(10..10, trail_ritual_stories());
    stories.splice(13..13, arcane_ritual_stories());
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
            CARD_VIEWPORT,
            native_artificer_portrait,
            NativeArtificerPortrait
        ),
        story!(
            "asset.native-barbarian-portrait",
            "Asset / Native Barbarian Card",
            Category::Assets,
            "The production card uses the embedded PNG on native protocols and its authored sprite fallback everywhere else.",
            CARD_VIEWPORT,
            native_barbarian_portrait,
            NativeBarbarianPortrait
        ),
        story!(
            "asset.native-bard-portrait",
            "Asset / Native Bard Card",
            Category::Assets,
            "The production Bard card uses its embedded PNG on native protocols and the authored sprite fallback elsewhere.",
            CARD_VIEWPORT,
            native_bard_portrait,
            NativeBardPortrait
        ),
        story!(
            "asset.native-cleric-portrait",
            "Asset / Native Cleric Card",
            Category::Assets,
            "The production Cleric card uses its embedded PNG on native protocols and the authored sprite fallback elsewhere.",
            CARD_VIEWPORT,
            native_cleric_portrait,
            NativeClericPortrait
        ),
        story!(
            "asset.native-druid-portrait",
            "Asset / Native Druid Card",
            Category::Assets,
            "The production Druid card uses its embedded PNG on native protocols and the authored sprite fallback elsewhere.",
            CARD_VIEWPORT,
            native_druid_portrait,
            NativeDruidPortrait
        ),
    ]
}

fn native_custom_class_card_stories() -> [Story; 4] {
    [
        story!(
            "asset.native-runewright-portrait",
            "Asset / Native Runewright Card",
            Category::Assets,
            "The production Runewright card uses its embedded PNG on native protocols and the authored sprite fallback elsewhere.",
            CARD_VIEWPORT,
            native_runewright_portrait,
            NativeRunewrightPortrait
        ),
        story!(
            "asset.native-pathseeker-portrait",
            "Asset / Native Pathseeker Card",
            Category::Assets,
            "The production Pathseeker card uses its embedded PNG on native protocols and the authored sprite fallback elsewhere.",
            CARD_VIEWPORT,
            native_pathseeker_portrait,
            NativePathseekerPortrait
        ),
        story!(
            "asset.native-mage-portrait",
            "Asset / Native Mage Card",
            Category::Assets,
            "The production Mage card uses its embedded PNG on native protocols and the authored sprite fallback elsewhere.",
            CARD_VIEWPORT,
            native_mage_portrait,
            NativeMagePortrait
        ),
        story!(
            "asset.native-sorcerer-portrait",
            "Asset / Native Sorcerer Card",
            Category::Assets,
            "The production Sorcerer card uses its embedded PNG on native protocols and the authored sprite fallback elsewhere.",
            CARD_VIEWPORT,
            native_sorcerer_portrait,
            NativeSorcererPortrait
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
            CARD_VIEWPORT,
            native_paladin_portrait,
            NativePaladinPortrait
        ),
        story!(
            "asset.native-ranger-portrait",
            "Asset / Native Ranger Card",
            Category::Assets,
            "The production Ranger card uses its embedded PNG on native protocols and the authored sprite fallback elsewhere.",
            CARD_VIEWPORT,
            native_ranger_portrait,
            NativeRangerPortrait
        ),
        story!(
            "asset.native-rogue-portrait",
            "Asset / Native Rogue Card",
            Category::Assets,
            "The production Rogue card uses its embedded PNG on native protocols and the authored sprite fallback elsewhere.",
            CARD_VIEWPORT,
            native_rogue_portrait,
            NativeRoguePortrait
        ),
        story!(
            "asset.native-testmender-portrait",
            "Asset / Native Testmender Card",
            Category::Assets,
            "The production Testmender card uses its embedded PNG on native protocols and the authored sprite fallback elsewhere.",
            CARD_VIEWPORT,
            native_testmender_portrait,
            NativeTestmenderPortrait
        ),
        story!(
            "asset.native-wizard-portrait",
            "Asset / Native Wizard Card",
            Category::Assets,
            "The production Wizard card uses its embedded PNG on native protocols and the authored sprite fallback elsewhere.",
            CARD_VIEWPORT,
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
            CARD_VIEWPORT,
            native_goblin_portrait,
            NativeGoblinPortrait
        ),
        story!(
            "asset.native-orc-portrait",
            "Asset / Reserved Orc Event Art",
            Category::Assets,
            "Reserved event/NPC art. It is not selected for ordinary adventurer cards.",
            CARD_VIEWPORT,
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

fn guild_fit(context: StoryContext) -> StoryFixture {
    StoryFixture::WorldFit(Box::new(fixtures::hall_fit_fixture(
        context,
        fixtures::FitParty::Five,
    )))
}
fn guild_fit_eleven(context: StoryContext) -> StoryFixture {
    StoryFixture::WorldFit(Box::new(fixtures::hall_fit_fixture(
        context,
        fixtures::FitParty::Eleven,
    )))
}
fn guild_fit_crowded(context: StoryContext) -> StoryFixture {
    StoryFixture::WorldFit(Box::new(fixtures::hall_fit_fixture(
        context,
        fixtures::FitParty::Crowded,
    )))
}

fn delve_fit(context: StoryContext) -> StoryFixture {
    StoryFixture::WorldFit(Box::new(fixtures::delve_fit_fixture(
        context,
        fixtures::FitParty::Five,
    )))
}
fn delve_fit_crowded(context: StoryContext) -> StoryFixture {
    StoryFixture::WorldFit(Box::new(fixtures::delve_fit_fixture(
        context,
        fixtures::FitParty::Crowded,
    )))
}

fn delve_fit_resting(context: StoryContext) -> StoryFixture {
    StoryFixture::WorldFit(Box::new(fixtures::delve_resting_fit_fixture(context)))
}

fn guild_fit_profile(context: StoryContext) -> StoryFixture {
    let mut model = fixtures::hall_fit_fixture(context, fixtures::FitParty::Five);
    model.show_adventurer_card();
    StoryFixture::WorldFit(Box::new(model))
}
fn delve_fit_profile(context: StoryContext) -> StoryFixture {
    let mut model = fixtures::delve_fit_fixture(context, fixtures::FitParty::Five);
    model.show_adventurer_card();
    StoryFixture::WorldFit(Box::new(model))
}

fn barbarian_poses(_context: StoryContext) -> StoryFixture {
    fixtures::ritual_pose_fixture(AdventurerClass::Barbarian)
}

fn wizard_poses(_context: StoryContext) -> StoryFixture {
    fixtures::ritual_pose_fixture(AdventurerClass::Wizard)
}

fn ranger_poses(_context: StoryContext) -> StoryFixture {
    fixtures::ritual_pose_fixture(AdventurerClass::Ranger)
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

fn native_mage_portrait(context: StoryContext) -> StoryFixture {
    scene(fixtures::native_mage_portrait_fixture(context))
}

fn native_sorcerer_portrait(context: StoryContext) -> StoryFixture {
    scene(fixtures::native_sorcerer_portrait_fixture(context))
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

fn bard_poses(_context: StoryContext) -> StoryFixture {
    fixtures::ritual_pose_fixture(AdventurerClass::Bard)
}

fn artificer_poses(_context: StoryContext) -> StoryFixture {
    fixtures::ritual_pose_fixture(AdventurerClass::Artificer)
}

fn testmender_poses(_context: StoryContext) -> StoryFixture {
    fixtures::ritual_pose_fixture(AdventurerClass::Testmender)
}

fn tool_ritual_stories() -> [Story; 3] {
    [
        story!(
            "asset.bard-poses",
            "Assets / Bard Poses",
            Category::Assets,
            "Production working, counsel, spoils and quiet poses for the Bard.",
            ASSET_VIEWPORT,
            bard_poses,
            BardPoseFamily
        ),
        story!(
            "asset.artificer-poses",
            "Assets / Artificer Poses",
            Category::Assets,
            "Production working, counsel, spoils and quiet poses for the Artificer.",
            ASSET_VIEWPORT,
            artificer_poses,
            ArtificerPoseFamily
        ),
        story!(
            "asset.testmender-poses",
            "Assets / Testmender Poses",
            Category::Assets,
            "Production working, counsel, spoils and quiet poses for the Testmender.",
            ASSET_VIEWPORT,
            testmender_poses,
            TestmenderPoseFamily
        ),
    ]
}

fn cleric_poses(_context: StoryContext) -> StoryFixture {
    fixtures::ritual_pose_fixture(AdventurerClass::Cleric)
}

fn paladin_poses(_context: StoryContext) -> StoryFixture {
    fixtures::ritual_pose_fixture(AdventurerClass::Paladin)
}

fn druid_poses(_context: StoryContext) -> StoryFixture {
    fixtures::ritual_pose_fixture(AdventurerClass::Druid)
}

fn support_ritual_stories() -> [Story; 3] {
    [
        story!(
            "asset.cleric-poses",
            "Assets / Cleric Poses",
            Category::Assets,
            "Production working, counsel, spoils and quiet poses for the Cleric.",
            ASSET_VIEWPORT,
            cleric_poses,
            ClericPoseFamily
        ),
        story!(
            "asset.paladin-poses",
            "Assets / Paladin Poses",
            Category::Assets,
            "Production working, counsel, spoils and quiet poses for the Paladin.",
            ASSET_VIEWPORT,
            paladin_poses,
            PaladinPoseFamily
        ),
        story!(
            "asset.druid-poses",
            "Assets / Druid Poses",
            Category::Assets,
            "Production working, counsel, spoils and quiet poses for the Druid.",
            ASSET_VIEWPORT,
            druid_poses,
            DruidPoseFamily
        ),
    ]
}

fn rogue_poses(_context: StoryContext) -> StoryFixture {
    fixtures::ritual_pose_fixture(AdventurerClass::Rogue)
}

fn pathseeker_poses(_context: StoryContext) -> StoryFixture {
    fixtures::ritual_pose_fixture(AdventurerClass::Pathseeker)
}

fn runewright_poses(_context: StoryContext) -> StoryFixture {
    fixtures::ritual_pose_fixture(AdventurerClass::Runewright)
}

fn trail_ritual_stories() -> [Story; 3] {
    [
        story!(
            "asset.rogue-poses",
            "Assets / Rogue Poses",
            Category::Assets,
            "Production working, counsel, spoils and quiet poses for the Rogue.",
            ASSET_VIEWPORT,
            rogue_poses,
            RoguePoseFamily
        ),
        story!(
            "asset.pathseeker-poses",
            "Assets / Pathseeker Poses",
            Category::Assets,
            "Production working, counsel, spoils and quiet poses for the Pathseeker.",
            ASSET_VIEWPORT,
            pathseeker_poses,
            PathseekerPoseFamily
        ),
        story!(
            "asset.runewright-poses",
            "Assets / Runewright Poses",
            Category::Assets,
            "Production working, counsel, spoils and quiet poses for the Runewright.",
            ASSET_VIEWPORT,
            runewright_poses,
            RunewrightPoseFamily
        ),
    ]
}

fn mage_poses(_context: StoryContext) -> StoryFixture {
    fixtures::ritual_pose_fixture(AdventurerClass::Mage)
}

fn sorcerer_poses(_context: StoryContext) -> StoryFixture {
    fixtures::ritual_pose_fixture(AdventurerClass::Sorcerer)
}

fn arcane_ritual_stories() -> [Story; 2] {
    [
        story!(
            "asset.mage-poses",
            "Assets / Mage Poses",
            Category::Assets,
            "Production working, counsel, spoils and quiet poses for the Mage.",
            ASSET_VIEWPORT,
            mage_poses,
            MagePoseFamily
        ),
        story!(
            "asset.sorcerer-poses",
            "Assets / Sorcerer Poses",
            Category::Assets,
            "Production working, counsel, spoils and quiet poses for the Sorcerer.",
            ASSET_VIEWPORT,
            sorcerer_poses,
            SorcererPoseFamily
        ),
    ]
}

fn board_populated(context: StoryContext) -> StoryFixture {
    StoryFixture::SceneApplication(Box::new(fixtures::quest_board_fixture(
        context,
        fixtures::WorkBoardState::Populated,
    )))
}

fn board_empty(context: StoryContext) -> StoryFixture {
    StoryFixture::SceneApplication(Box::new(fixtures::quest_board_fixture(
        context,
        fixtures::WorkBoardState::Empty,
    )))
}

fn board_offline(context: StoryContext) -> StoryFixture {
    StoryFixture::SceneApplication(Box::new(fixtures::quest_board_fixture(
        context,
        fixtures::WorkBoardState::Offline,
    )))
}

fn board_deferred(context: StoryContext) -> StoryFixture {
    StoryFixture::SceneApplication(Box::new(fixtures::quest_board_fixture(
        context,
        fixtures::WorkBoardState::Deferred,
    )))
}

fn board_resolved(context: StoryContext) -> StoryFixture {
    StoryFixture::SceneApplication(Box::new(fixtures::quest_board_fixture(
        context,
        fixtures::WorkBoardState::Resolved,
    )))
}

fn board_narrow(context: StoryContext) -> StoryFixture {
    StoryFixture::SceneApplication(Box::new(fixtures::quest_board_fixture(
        context,
        fixtures::WorkBoardState::Narrow,
    )))
}

fn board_exited(context: StoryContext) -> StoryFixture {
    StoryFixture::SceneApplication(Box::new(fixtures::quest_board_fixture(
        context,
        fixtures::WorkBoardState::WorkerExited,
    )))
}

fn board_stories() -> Vec<Story> {
    let mut stories = Vec::new();
    let board_variants: [(&str, &str, StoryBuilder); 7] = [
        (
            "interaction.board",
            "Interaction / Quest Board",
            board_populated,
        ),
        (
            "interaction.board-empty",
            "Interaction / Empty Quest Board",
            board_empty,
        ),
        (
            "interaction.board-offline",
            "Interaction / Offline Quest Board",
            board_offline,
        ),
        (
            "interaction.board-deferred",
            "Interaction / Deferred Quest Board",
            board_deferred,
        ),
        (
            "interaction.board-resolved",
            "Interaction / Resolved Quest Board",
            board_resolved,
        ),
        (
            "interaction.board-narrow",
            "Interaction / Narrow Quest Board",
            board_narrow,
        ),
        (
            "interaction.board-exited",
            "Interaction / Departed Worker Quest Board",
            board_exited,
        ),
    ];
    for (index, (id, title, build)) in board_variants.into_iter().enumerate() {
        stories.push(Story{id:StoryId::new(id),title,category:Category::Interactions,description:"Synthetic work facts through the production parchment; no live services or persistence.",viewport:if index==5 {NARROW_VIEWPORT}else{CARD_VIEWPORT},build,owns:if index==0 {&[AssetId(SceneFirstAsset::QuestBoardParchment)]}else{&[]},shows:if index==0 {&[]}else{&[AssetId(SceneFirstAsset::QuestBoardParchment)]}});
    }
    for (id, title, build, viewport) in [
        (
            "interaction.owned-actions",
            "Interaction / Owned Actions",
            owned_actions as StoryBuilder,
            CARD_VIEWPORT,
        ),
        (
            "interaction.owned-empty-campaign",
            "Interaction / Empty Owned Campaign",
            owned_empty,
            CARD_VIEWPORT,
        ),
        (
            "interaction.owned-answer",
            "Interaction / Owned Answer Form",
            owned_answer,
            CARD_VIEWPORT,
        ),
        (
            "interaction.owned-answer-narrow",
            "Interaction / Narrow Owned Answer",
            owned_narrow,
            Viewport::new(40, 20, 1, 1),
        ),
        (
            "interaction.owned-view-pending",
            "Interaction / Saved Owned View Pending",
            owned_pending,
            CARD_VIEWPORT,
        ),
        (
            "interaction.owned-indeterminate",
            "Interaction / Uncertain Owned Save",
            owned_uncertain,
            CARD_VIEWPORT,
        ),
        (
            "interaction.owned-answered",
            "Interaction / Answered Owned Decision",
            owned_answered,
            CARD_VIEWPORT,
        ),
        (
            "interaction.owned-small",
            "Interaction / Owned Form Resize Limit",
            owned_small,
            Viewport::new(27, 11, 1, 1),
        ),
    ] {
        stories.push(Story {id:StoryId::new(id),title,category:Category::Interactions,description:"Fixed owned facts and production authoring overlay; no filesystem or live services.",viewport,build,owns:&[],shows:&[AssetId(SceneFirstAsset::QuestBoardParchment)]});
    }
    stories
}

fn table_populated(context: StoryContext) -> StoryFixture {
    StoryFixture::SceneApplication(Box::new(fixtures::war_table_fixture(
        context,
        fixtures::WarTableState::Populated,
    )))
}
fn table_narrow(context: StoryContext) -> StoryFixture {
    StoryFixture::SceneApplication(Box::new(fixtures::war_table_fixture(
        context,
        fixtures::WarTableState::Narrow,
    )))
}
fn table_empty(context: StoryContext) -> StoryFixture {
    StoryFixture::SceneApplication(Box::new(fixtures::war_table_fixture(
        context,
        fixtures::WarTableState::Empty,
    )))
}
fn table_offline(context: StoryContext) -> StoryFixture {
    StoryFixture::SceneApplication(Box::new(fixtures::war_table_fixture(
        context,
        fixtures::WarTableState::Offline,
    )))
}
fn table_cyclic(context: StoryContext) -> StoryFixture {
    StoryFixture::SceneApplication(Box::new(fixtures::war_table_fixture(
        context,
        fixtures::WarTableState::Cyclic,
    )))
}
fn table_large(context: StoryContext) -> StoryFixture {
    StoryFixture::SceneApplication(Box::new(fixtures::war_table_fixture(
        context,
        fixtures::WarTableState::Large,
    )))
}
fn table_stories() -> Vec<Story> {
    let variants: [(&str, &str, StoryBuilder); 6] = [
        (
            "interaction.table",
            "Interaction / War Table",
            table_populated,
        ),
        (
            "interaction.table-narrow",
            "Interaction / Narrow War Table",
            table_narrow,
        ),
        (
            "interaction.table-empty",
            "Interaction / Empty War Table",
            table_empty,
        ),
        (
            "interaction.table-offline",
            "Interaction / Offline War Table",
            table_offline,
        ),
        (
            "interaction.table-cyclic",
            "Interaction / Cyclic War Table",
            table_cyclic,
        ),
        (
            "interaction.table-large",
            "Interaction / Large War Table",
            table_large,
        ),
    ];
    variants.into_iter().enumerate().map(|(index,(id,title,build))| Story {
        id:StoryId::new(id),title,category:Category::Interactions,
        description:"Source-owned dependency facts through the static production RGB table and selected parchment.",
        viewport:if index==1 {NARROW_VIEWPORT}else{CARD_VIEWPORT},build,
        owns:if index==0 {&[AssetId(SceneFirstAsset::WarTableMap)]} else {&[]},
        shows:if index==0 {&[]} else {&[AssetId(SceneFirstAsset::WarTableMap)]},
    }).collect()
}

fn owned_actions(context: StoryContext) -> StoryFixture {
    StoryFixture::SceneApplication(Box::new(fixtures::owned_authoring_fixture(
        context,
        fixtures::OwnedAuthoringState::Actions,
    )))
}

fn owned_empty(context: StoryContext) -> StoryFixture {
    StoryFixture::SceneApplication(Box::new(fixtures::owned_authoring_fixture(
        context,
        fixtures::OwnedAuthoringState::EmptyCampaign,
    )))
}

fn owned_answer(context: StoryContext) -> StoryFixture {
    StoryFixture::SceneApplication(Box::new(fixtures::owned_authoring_fixture(
        context,
        fixtures::OwnedAuthoringState::Answer,
    )))
}

fn owned_narrow(context: StoryContext) -> StoryFixture {
    StoryFixture::SceneApplication(Box::new(fixtures::owned_authoring_fixture(
        context,
        fixtures::OwnedAuthoringState::Narrow,
    )))
}

fn owned_pending(context: StoryContext) -> StoryFixture {
    StoryFixture::SceneApplication(Box::new(fixtures::owned_authoring_fixture(
        context,
        fixtures::OwnedAuthoringState::ViewPending,
    )))
}

fn owned_uncertain(context: StoryContext) -> StoryFixture {
    StoryFixture::SceneApplication(Box::new(fixtures::owned_authoring_fixture(
        context,
        fixtures::OwnedAuthoringState::Indeterminate,
    )))
}

fn owned_answered(context: StoryContext) -> StoryFixture {
    StoryFixture::SceneApplication(Box::new(fixtures::owned_authoring_fixture(
        context,
        fixtures::OwnedAuthoringState::Answered,
    )))
}

fn owned_small(context: StoryContext) -> StoryFixture {
    StoryFixture::SceneApplication(Box::new(fixtures::owned_authoring_fixture(
        context,
        fixtures::OwnedAuthoringState::Small,
    )))
}
