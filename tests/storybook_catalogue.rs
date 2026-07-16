#![cfg(feature = "storybook")]

use questmancer::app::{Model, View};
use questmancer::{
    app::{CharacterSet, ColorMode, DisplayPreferences, Motion},
    domain::{AdventurerPersona, PersonaKey, Presence},
    storybook::{
        AssetId, CompatibilityAsset, SceneAsset, WidgetAsset, asset_inventory,
        catalogue::{
            Category, Story, StoryId, Viewport, catalogue, validate_catalogue, validate_coverage,
        },
        fixtures::{AtlasContent, StoryContext, StoryFixture},
    },
    ui::{delve_scene::DelveVariant, goblins::GoblinSighting},
};

#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "the StoryBuilder contract intentionally accepts a borrowed context"
)]
fn build(_: &StoryContext) -> StoryFixture {
    StoryFixture::Application(Model::new(View::Guild))
}

fn story(id: &'static str, owns: &'static [AssetId]) -> Story {
    Story::new(
        StoryId::new(id),
        id,
        Category::Widgets,
        "coverage fixture",
        Viewport::new(80, 24, 40, 12),
        build,
        owns,
        &[],
    )
}

#[test]
fn coverage_accepts_exactly_one_owner_per_asset() {
    const BOARD: AssetId = AssetId::Widget(WidgetAsset::QuestBoard);
    let report = validate_coverage(&[BOARD], &[story("board", &[BOARD])]).unwrap();
    assert_eq!(report.owned(), 1);
    assert!(report.missing().is_empty());
    assert!(report.duplicates().is_empty());
}

#[test]
fn coverage_rejects_missing_duplicate_and_unknown_ownership() {
    const BOARD: AssetId = AssetId::Widget(WidgetAsset::QuestBoard);
    const PARTY: AssetId = AssetId::Widget(WidgetAsset::Party);
    const SUMMONS: AssetId = AssetId::Widget(WidgetAsset::Summons);
    let error = validate_coverage(
        &[BOARD, SUMMONS],
        &[story("one", &[BOARD, PARTY]), story("two", &[BOARD])],
    )
    .unwrap_err();
    assert_eq!(error.missing(), &[SUMMONS]);
    assert_eq!(error.duplicates(), &[BOARD]);
    assert_eq!(error.unknown(), &[PARTY]);
}

#[test]
fn authored_inventory_contains_no_duplicate_identifiers() {
    let inventory = asset_inventory();
    let unique = inventory
        .iter()
        .copied()
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(inventory.len(), unique.len());
}

#[test]
fn production_catalogue_owns_every_authored_asset_once() {
    let report = validate_catalogue().unwrap();
    assert_eq!(asset_inventory().len(), 158);
    assert_eq!(report.owned(), asset_inventory().len());
    assert!(report.missing().is_empty());
    assert!(report.duplicates().is_empty());
    assert!(report.unknown().is_empty());
}

#[test]
fn story_ids_and_order_are_stable() {
    let ids = catalogue()
        .iter()
        .map(|story| story.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(ids.first(), Some(&"atlas.classes"));
    assert_eq!(ids.last(), Some(&"compat.motion-none"));
    assert_eq!(
        ids.len(),
        ids.iter().collect::<std::collections::HashSet<_>>().len()
    );
}

#[test]
fn atlas_catalogue_owns_every_atlas_asset_exactly_once() {
    let atlas_inventory = asset_inventory()
        .into_iter()
        .filter(|asset| {
            matches!(
                asset,
                AssetId::Class(_)
                    | AssetId::Gear(_)
                    | AssetId::Ancestry(_)
                    | AssetId::BodyProportions(_)
                    | AssetId::HeadShape(_)
                    | AssetId::SkinTone(_)
                    | AssetId::HairShape(_)
                    | AssetId::HairTone(_)
                    | AssetId::FaceDetail(_)
                    | AssetId::Garb(_)
                    | AssetId::Legwear(_)
                    | AssetId::Footwear(_)
                    | AssetId::Keepsake(_)
                    | AssetId::AccentTone(_)
                    | AssetId::ColorRole(_)
                    | AssetId::Pose(_)
            )
        })
        .collect::<Vec<_>>();

    let atlas_stories = catalogue()
        .iter()
        .filter(|story| story.category == Category::AssetAtlas)
        .cloned()
        .collect::<Vec<_>>();
    let report = validate_coverage(&atlas_inventory, &atlas_stories).unwrap();
    assert_eq!(report.owned(), atlas_inventory.len());
}

#[test]
fn atlas_stories_enumerate_their_reused_visible_persona_assets() {
    let profile = AdventurerPersona::for_key(PersonaKey::new("storybook-atlas"));
    let pose = AdventurerPersona::for_key(PersonaKey::new("storybook-pose-atlas"));
    let profile_class = AssetId::Class(profile.class);
    let profile_ancestry = AssetId::Ancestry(profile.ancestry);
    let pose_class = AssetId::Class(pose.class);
    let pose_ancestry = AssetId::Ancestry(pose.ancestry);

    for story in catalogue()
        .iter()
        .filter(|story| story.category == Category::AssetAtlas)
    {
        let expected = match story.id.as_str() {
            "atlas.classes" => vec![profile_ancestry],
            "atlas.ancestries" => vec![profile_class],
            "atlas.palette-roles" => vec![],
            "atlas.poses" => vec![pose_class, pose_ancestry],
            _ => vec![profile_class, profile_ancestry],
        };
        assert_eq!(
            story
                .shows
                .iter()
                .copied()
                .collect::<std::collections::HashSet<_>>(),
            expected.into_iter().collect(),
            "{}",
            story.id.as_str()
        );
    }
}

#[test]
fn catalogue_uses_every_canonical_id_in_exact_order() {
    let ids = catalogue()
        .iter()
        .map(|story| story.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        ids,
        vec![
            "atlas.classes",
            "atlas.ancestries",
            "atlas.body-proportions",
            "atlas.head-shapes",
            "atlas.skin-tones",
            "atlas.hair-shapes",
            "atlas.hair-tones",
            "atlas.face-details",
            "atlas.garb",
            "atlas.legwear",
            "atlas.footwear",
            "atlas.keepsakes",
            "atlas.accent-tones",
            "atlas.palette-roles",
            "atlas.poses",
            "widgets.adventurer-cards",
            "widgets.chambers",
            "widgets.guild-regions",
            "widgets.counsel",
            "widgets.search",
            "widgets.help",
            "scenes.guild-empty",
            "scenes.guild-populated",
            "scenes.guild-mixed-attention",
            "scenes.guild-disconnected",
            "scenes.guild-reconnecting",
            "scenes.delve-library",
            "scenes.delve-undercroft",
            "scenes.delve-watchtower",
            "scenes.connected-delves",
            "scenes.mixed-state-delve",
            "scenes.narrow-guild",
            "scenes.narrow-delve",
            "goblins.chest-eyes",
            "goblins.chronicle-hand",
            "goblins.rafters-scroll",
            "goblins.stolen-biscuit",
            "goblins.outbreak",
            "compat.unicode-xterm256",
            "compat.unicode-ansi16",
            "compat.ascii-ansi16",
            "compat.motion-full",
            "compat.motion-reduced",
            "compat.motion-none",
        ]
    );
    assert_eq!(ids.len(), 44);
}

#[test]
fn all_four_categories_are_populated_in_the_fixed_order() {
    let counts = Category::ALL.map(|category| {
        catalogue()
            .iter()
            .filter(|story| story.category == category)
            .count()
    });
    assert_eq!(counts, [15, 6, 17, 6]);
    assert!(catalogue()[..15].iter().all(|story| {
        story.category == Category::AssetAtlas && story.viewport == Viewport::new(120, 36, 60, 18)
    }));
}

#[test]
fn owns_and_shows_are_known_disjoint_and_internally_unique() {
    let inventory = asset_inventory()
        .into_iter()
        .collect::<std::collections::HashSet<_>>();
    for story in catalogue() {
        let owns = story
            .owns
            .iter()
            .copied()
            .collect::<std::collections::HashSet<_>>();
        let shows = story
            .shows
            .iter()
            .copied()
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(owns.len(), story.owns.len(), "{} owns", story.id.as_str());
        assert_eq!(
            shows.len(),
            story.shows.len(),
            "{} shows",
            story.id.as_str()
        );
        assert!(owns.is_disjoint(&shows), "{}", story.id.as_str());
        assert!(
            owns.union(&shows).all(|asset| inventory.contains(asset)),
            "{}",
            story.id.as_str()
        );
        for shown in story.shows {
            assert_eq!(
                catalogue()
                    .iter()
                    .filter(|candidate| candidate.owns.contains(shown))
                    .count(),
                1,
                "{} shows {}",
                story.id.as_str(),
                shown.label()
            );
        }
    }
}

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "the canonical ownership table intentionally names every non-atlas story"
)]
fn every_non_atlas_story_has_its_exact_canonical_ownership() {
    let expected = [
        (
            "widgets.adventurer-cards",
            vec![
                AssetId::Widget(WidgetAsset::AdventurerCardFull),
                AssetId::Widget(WidgetAsset::AdventurerCardCompact),
            ],
        ),
        (
            "widgets.chambers",
            vec![
                AssetId::Widget(WidgetAsset::ChamberFull),
                AssetId::Widget(WidgetAsset::ChamberCompact),
            ],
        ),
        (
            "widgets.guild-regions",
            vec![
                AssetId::Widget(WidgetAsset::QuestBoard),
                AssetId::Widget(WidgetAsset::Party),
                AssetId::Widget(WidgetAsset::Summons),
                AssetId::Widget(WidgetAsset::Chronicle),
                AssetId::Widget(WidgetAsset::AdventurerProfile),
                AssetId::Widget(WidgetAsset::Scrying),
                AssetId::Widget(WidgetAsset::Spoils),
            ],
        ),
        (
            "widgets.counsel",
            vec![AssetId::Widget(WidgetAsset::Counsel)],
        ),
        ("widgets.search", vec![AssetId::Widget(WidgetAsset::Search)]),
        ("widgets.help", vec![AssetId::Widget(WidgetAsset::Help)]),
        (
            "scenes.guild-empty",
            vec![AssetId::Scene(SceneAsset::GuildEmpty)],
        ),
        (
            "scenes.guild-populated",
            vec![AssetId::Scene(SceneAsset::GuildPopulated)],
        ),
        (
            "scenes.guild-mixed-attention",
            vec![AssetId::Scene(SceneAsset::GuildMixedAttention)],
        ),
        (
            "scenes.guild-disconnected",
            vec![AssetId::Scene(SceneAsset::GuildDisconnected)],
        ),
        (
            "scenes.guild-reconnecting",
            vec![AssetId::Scene(SceneAsset::GuildReconnecting)],
        ),
        (
            "scenes.delve-library",
            vec![AssetId::DelveVariant(DelveVariant::ForgottenLibrary)],
        ),
        (
            "scenes.delve-undercroft",
            vec![AssetId::DelveVariant(DelveVariant::MossyUndercroft)],
        ),
        (
            "scenes.delve-watchtower",
            vec![AssetId::DelveVariant(DelveVariant::OldWatchtower)],
        ),
        (
            "scenes.connected-delves",
            vec![AssetId::Scene(SceneAsset::ConnectedDelves)],
        ),
        (
            "scenes.mixed-state-delve",
            vec![AssetId::Scene(SceneAsset::MixedStateDelve)],
        ),
        (
            "scenes.narrow-guild",
            vec![AssetId::Scene(SceneAsset::NarrowGuild)],
        ),
        (
            "scenes.narrow-delve",
            vec![AssetId::Scene(SceneAsset::NarrowDelve)],
        ),
        (
            "goblins.chest-eyes",
            vec![AssetId::GoblinSighting(GoblinSighting::ChestEyes)],
        ),
        (
            "goblins.chronicle-hand",
            vec![AssetId::GoblinSighting(GoblinSighting::ChronicleHand)],
        ),
        (
            "goblins.rafters-scroll",
            vec![AssetId::GoblinSighting(GoblinSighting::RaftersScroll)],
        ),
        (
            "goblins.stolen-biscuit",
            vec![AssetId::GoblinSighting(GoblinSighting::StolenBiscuit)],
        ),
        ("goblins.outbreak", vec![AssetId::GoblinOutbreak]),
        (
            "compat.unicode-xterm256",
            vec![AssetId::Compatibility(CompatibilityAsset::UnicodeXterm256)],
        ),
        (
            "compat.unicode-ansi16",
            vec![AssetId::Compatibility(CompatibilityAsset::UnicodeAnsi16)],
        ),
        (
            "compat.ascii-ansi16",
            vec![AssetId::Compatibility(CompatibilityAsset::AsciiAnsi16)],
        ),
        (
            "compat.motion-full",
            vec![AssetId::Compatibility(CompatibilityAsset::MotionFull)],
        ),
        (
            "compat.motion-reduced",
            vec![AssetId::Compatibility(CompatibilityAsset::MotionReduced)],
        ),
        (
            "compat.motion-none",
            vec![AssetId::Compatibility(CompatibilityAsset::MotionNone)],
        ),
    ];

    assert_eq!(expected.len(), 29);
    for (story_id, owns) in expected {
        let story = catalogue()
            .iter()
            .find(|story| story.id.as_str() == story_id)
            .unwrap();
        assert_eq!(story.owns, owns, "{story_id}");
    }
}

#[test]
fn widget_atlases_cross_the_production_layout_thresholds_after_borders() {
    let expected = [
        ("widgets.adventurer-cards", [(36, 21), (30, 12)]),
        ("widgets.chambers", [(30, 12), (26, 9)]),
    ];
    for (story_id, dimensions) in expected {
        let story = catalogue()
            .iter()
            .find(|story| story.id.as_str() == story_id)
            .unwrap();
        let StoryFixture::AssetAtlas(atlas) = (story.build)(&StoryContext::fixed()) else {
            panic!("{story_id} must be an asset atlas");
        };
        assert_eq!(atlas.tiles.len(), 2, "{story_id}");
        assert_eq!(
            atlas
                .tiles
                .iter()
                .map(|tile| (tile.preferred_width, tile.preferred_height))
                .collect::<Vec<_>>(),
            dimensions,
            "{story_id}"
        );
        assert!(atlas.tiles.iter().all(|tile| matches!(
            tile.content,
            AtlasContent::AdventurerCard { .. } | AtlasContent::Chamber { .. }
        )));
    }
}

#[test]
fn compatibility_stories_apply_the_exact_fixed_preferences() {
    let expected = [
        (
            "compat.unicode-xterm256",
            Motion::Full,
            CharacterSet::Unicode,
            ColorMode::Xterm256,
        ),
        (
            "compat.unicode-ansi16",
            Motion::Full,
            CharacterSet::Unicode,
            ColorMode::Ansi16,
        ),
        (
            "compat.ascii-ansi16",
            Motion::Full,
            CharacterSet::Ascii,
            ColorMode::Ansi16,
        ),
        (
            "compat.motion-full",
            Motion::Full,
            CharacterSet::Unicode,
            ColorMode::Xterm256,
        ),
        (
            "compat.motion-reduced",
            Motion::Reduced,
            CharacterSet::Unicode,
            ColorMode::Xterm256,
        ),
        (
            "compat.motion-none",
            Motion::None,
            CharacterSet::Unicode,
            ColorMode::Xterm256,
        ),
    ];
    for (story_id, motion, character_set, color_mode) in expected {
        let story = catalogue()
            .iter()
            .find(|story| story.id.as_str() == story_id)
            .unwrap();
        let StoryFixture::Application(model) = (story.build)(&StoryContext::fixed()) else {
            panic!("{story_id} must use the application renderer");
        };
        assert_eq!(
            model.preferences(),
            &DisplayPreferences {
                motion,
                character_set,
                color_mode
            },
            "{story_id}"
        );
    }
}

#[test]
fn every_non_atlas_application_story_uses_the_production_fixture_bridge() {
    for story in catalogue() {
        let fixture = (story.build)(&StoryContext::fixed());
        let is_widget_atlas = matches!(
            story.id.as_str(),
            "widgets.adventurer-cards" | "widgets.chambers"
        );
        if story.category == Category::AssetAtlas || is_widget_atlas {
            assert!(
                matches!(fixture, StoryFixture::AssetAtlas(_)),
                "{}",
                story.id.as_str()
            );
        } else {
            assert!(
                matches!(fixture, StoryFixture::Application(_)),
                "{}",
                story.id.as_str()
            );
        }
    }
}

#[test]
fn connected_and_mixed_state_delves_use_distinct_canonical_models() {
    let connected_story = catalogue()
        .iter()
        .find(|story| story.id.as_str() == "scenes.connected-delves")
        .unwrap();
    let mixed_story = catalogue()
        .iter()
        .find(|story| story.id.as_str() == "scenes.mixed-state-delve")
        .unwrap();
    let StoryFixture::Application(connected) = (connected_story.build)(&StoryContext::fixed())
    else {
        panic!("connected Delves must be an application fixture");
    };
    let StoryFixture::Application(mixed) = (mixed_story.build)(&StoryContext::fixed()) else {
        panic!("mixed-state Delve must be an application fixture");
    };

    assert_ne!(connected, mixed);
    assert!(
        connected
            .domain()
            .agents
            .values()
            .all(|agent| agent.presence == Presence::Working)
    );
    assert!(
        mixed
            .domain()
            .agents
            .values()
            .any(|agent| agent.presence != Presence::Working)
    );
}

#[test]
fn named_delve_fixtures_do_not_retain_cross_campaign_evidence() {
    for story_id in [
        "scenes.delve-library",
        "scenes.delve-undercroft",
        "scenes.delve-watchtower",
    ] {
        let story = catalogue()
            .iter()
            .find(|story| story.id.as_str() == story_id)
            .unwrap();
        let StoryFixture::Application(model) = (story.build)(&StoryContext::fixed()) else {
            panic!("{story_id} must be an application fixture");
        };
        assert_eq!(model.domain().campaigns.len(), 1, "{story_id}");
        assert!(model.output_preview().is_none(), "{story_id}");
        assert!(model.domain().chronicle.entries().is_empty(), "{story_id}");
    }
}
