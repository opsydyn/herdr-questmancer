#![cfg(feature = "storybook")]

use std::collections::HashSet;

use questmancer::app::{Model, View};
use questmancer::{
    app::{CharacterSet, ColorMode, DisplayPreferences, Motion},
    domain::{
        AccentTone, AdventurerClass, AdventurerPersona, AdventuringGear, Ancestry, BodyProportions,
        FaceDetail, Footwear, Garb, GuildAttention, HairShape, HairTone, HeadShape, Keepsake,
        Legwear, PersonaKey, Presence, SkinTone,
    },
    storybook::{
        AssetId, CompatibilityAsset, SceneAsset, WidgetAsset, asset_inventory,
        assets::{LandmarkAsset, RoomCameraAsset, TruthfulStationAsset},
        catalogue::{
            Category, Story, StoryId, Viewport, catalogue, validate_catalogue, validate_coverage,
        },
        fixtures::{AtlasContent, StoryContext, StoryFixture},
    },
    ui::{delve_scene::DelveVariant, goblins::GoblinSighting, theatre::TheatrePose},
};
use ratatui::layout::Rect;

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
fn production_and_storybook_asset_families_expose_exhaustive_collections() {
    let production_count = AdventurerClass::ALL.len()
        + AdventuringGear::ALL.len()
        + Ancestry::ALL.len()
        + BodyProportions::ALL.len()
        + HeadShape::ALL.len()
        + SkinTone::ALL.len()
        + HairShape::ALL.len()
        + HairTone::ALL.len()
        + FaceDetail::ALL.len()
        + Garb::ALL.len()
        + Legwear::ALL.len()
        + Footwear::ALL.len()
        + Keepsake::ALL.len()
        + AccentTone::ALL.len()
        + questmancer::ui::pixel::ColorRole::ALL.len()
        + TheatrePose::ALL.len()
        + DelveVariant::ALL.len()
        + GoblinSighting::ALL.len();
    let storybook_count = WidgetAsset::ALL.len()
        + SceneAsset::ALL.len()
        + CompatibilityAsset::ALL.len()
        + LandmarkAsset::ALL.len()
        + TruthfulStationAsset::ALL.len()
        + RoomCameraAsset::ALL.len()
        + 1;
    assert_eq!(asset_inventory().len(), production_count + storybook_count);
}

#[test]
fn production_catalogue_owns_every_authored_asset_once() {
    let report = validate_catalogue().unwrap();
    assert_eq!(asset_inventory().len(), 178);
    assert_eq!(report.owned(), asset_inventory().len());
    assert!(report.missing().is_empty());
    assert!(report.duplicates().is_empty());
    assert!(report.unknown().is_empty());
}

#[test]
fn authored_inventory_includes_every_great_room_asset_family() {
    let labels = asset_inventory()
        .into_iter()
        .map(AssetId::label)
        .collect::<HashSet<_>>();

    for expected in [
        "landmark: guild door",
        "landmark: quest wall",
        "landmark: campaign table",
        "landmark: counsel bell",
        "landmark: hearth",
        "landmark: chronicle lectern",
        "landmark: scrying alcove",
        "landmark: spoils vault",
        "truthful station: campaign token",
        "truthful station: counsel projection",
        "truthful station: hearth adventurer",
        "truthful station: spoils adventurer",
        "room camera: whole room",
        "room camera: cropped room",
        "room camera: landmark",
    ] {
        assert!(
            labels.contains(expected),
            "missing authored asset {expected}"
        );
    }
}

#[test]
fn catalogue_has_fixed_great_room_review_stories() {
    let ids = catalogue()
        .iter()
        .map(|story| story.id.as_str())
        .collect::<HashSet<_>>();

    for expected in [
        "atlas.great-room-landmarks",
        "atlas.truthful-stations",
        "scenes.guild-one-campaign",
        "scenes.guild-reviewr-unavailable",
        "scenes.guild-scrying-failed",
        "scenes.guild-cropped-room",
        "scenes.guild-landmark-camera",
    ] {
        assert!(ids.contains(expected), "missing fixed story {expected}");
    }
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
                    | AssetId::Landmark(_)
                    | AssetId::TruthfulStation(_)
                    | AssetId::RoomCamera(_)
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
    for story in catalogue().iter().filter(|story| {
        story.category == Category::AssetAtlas
            && !story.id.as_str().starts_with("atlas.great-room")
            && !story.id.as_str().starts_with("atlas.truthful-stations")
            && !story.id.as_str().starts_with("atlas.camera-")
    }) {
        let mut expected = if story.id.as_str() == "atlas.palette-roles" {
            HashSet::new()
        } else if story.id.as_str() == "atlas.poses" {
            persona_assets(&pose).into_iter().collect()
        } else {
            persona_assets(&profile).into_iter().collect()
        };
        expected.retain(|asset| !story.owns.contains(asset));
        assert_eq!(
            story
                .shows
                .iter()
                .copied()
                .collect::<std::collections::HashSet<_>>(),
            expected,
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
            "atlas.great-room-landmarks",
            "atlas.truthful-stations",
            "atlas.camera-whole-room",
            "atlas.camera-cropped-room",
            "atlas.camera-landmark",
            "widgets.adventurer-cards",
            "widgets.chambers",
            "widgets.guild-regions",
            "widgets.counsel",
            "widgets.search",
            "widgets.help",
            "scenes.guild-empty",
            "scenes.guild-populated",
            "scenes.guild-one-campaign",
            "scenes.guild-mixed-attention",
            "scenes.guild-disconnected",
            "scenes.guild-reconnecting",
            "scenes.guild-reviewr-unavailable",
            "scenes.guild-scrying-failed",
            "scenes.guild-cropped-room",
            "scenes.guild-landmark-camera",
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
    assert_eq!(ids.len(), 54);
}

#[test]
fn all_four_categories_are_populated_in_the_fixed_order() {
    let counts = Category::ALL.map(|category| {
        catalogue()
            .iter()
            .filter(|story| story.category == category)
            .count()
    });
    assert_eq!(counts, [20, 6, 22, 6]);
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
            "scenes.guild-one-campaign",
            vec![AssetId::Scene(SceneAsset::GuildOneCampaign)],
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
            "scenes.guild-reviewr-unavailable",
            vec![AssetId::Scene(SceneAsset::GuildReviewrUnavailable)],
        ),
        (
            "scenes.guild-scrying-failed",
            vec![AssetId::Scene(SceneAsset::GuildScryingFailed)],
        ),
        (
            "scenes.guild-cropped-room",
            vec![AssetId::Scene(SceneAsset::GuildCroppedRoom)],
        ),
        (
            "scenes.guild-landmark-camera",
            vec![AssetId::Scene(SceneAsset::GuildLandmarkCamera)],
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

    assert_eq!(expected.len(), 34);
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
    let story = catalogue()
        .iter()
        .find(|story| story.id.as_str() == "widgets.adventurer-cards")
        .unwrap();
    let StoryFixture::AssetAtlas(atlas) = (story.build)(&StoryContext::fixed()) else {
        panic!("widgets.adventurer-cards must be an asset atlas");
    };
    assert_eq!(atlas.tiles.len(), 2);
    assert_eq!(
        atlas
            .tiles
            .iter()
            .map(|tile| (tile.preferred_width, tile.preferred_height))
            .collect::<Vec<_>>(),
        [(36, 21), (30, 12)]
    );
    assert!(
        atlas
            .tiles
            .iter()
            .all(|tile| matches!(tile.content, AtlasContent::AdventurerCard { .. }))
    );
}

#[test]
fn chamber_atlas_is_the_complete_pose_by_production_layout_matrix() {
    let story = catalogue()
        .iter()
        .find(|story| story.id.as_str() == "widgets.chambers")
        .unwrap();
    let StoryFixture::AssetAtlas(atlas) = (story.build)(&StoryContext::fixed()) else {
        panic!("widgets.chambers must be an asset atlas");
    };
    let expected = [
        (TheatrePose::Delving, "delving"),
        (TheatrePose::SeekingCounsel, "seeking counsel"),
        (TheatrePose::SpoilsUnopened, "spoils unopened"),
        (TheatrePose::VictoryRecorded, "victory recorded"),
        (TheatrePose::Resting, "resting"),
        (TheatrePose::Departed, "departed"),
        (TheatrePose::Unknown, "unknown"),
    ];

    assert_eq!(atlas.tiles.len(), expected.len() * 2);
    for (index, (pose, pose_label)) in expected.into_iter().enumerate() {
        for (layout_index, (layout, dimensions, selected)) in
            [("Full", (30, 12), true), ("Compact", (26, 9), false)]
                .into_iter()
                .enumerate()
        {
            let tile = &atlas.tiles[index * 2 + layout_index];
            assert_eq!(tile.label, format!("{layout} chamber - {pose_label}"));
            assert_eq!((tile.preferred_width, tile.preferred_height), dimensions);
            let AtlasContent::Chamber {
                theatre,
                selected: actual_selected,
                ..
            } = &tile.content
            else {
                panic!("{} bypasses the production chamber widget path", tile.label);
            };
            assert_eq!(theatre.pose, pose, "{}", tile.label);
            assert_eq!(*actual_selected, selected, "{}", tile.label);
        }
    }
}

#[test]
fn great_room_atlases_only_embed_production_application_renderers() {
    for story_id in [
        "atlas.great-room-landmarks",
        "atlas.truthful-stations",
        "atlas.camera-whole-room",
        "atlas.camera-cropped-room",
        "atlas.camera-landmark",
    ] {
        let story = catalogue()
            .iter()
            .find(|story| story.id.as_str() == story_id)
            .unwrap();
        let StoryFixture::AssetAtlas(atlas) = (story.build)(&StoryContext::fixed()) else {
            panic!("{story_id} must be an asset atlas");
        };
        assert!(!atlas.tiles.is_empty(), "{story_id}");
        assert!(
            atlas
                .tiles
                .iter()
                .all(|tile| matches!(tile.content, AtlasContent::Application { .. })),
            "{story_id} bypasses the production application renderer"
        );
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
fn compatibility_stories_use_clean_connected_great_room_baselines() {
    for story in catalogue()
        .iter()
        .filter(|story| story.category == Category::Compatibility)
    {
        let StoryFixture::Application(model) = (story.build)(&StoryContext::fixed()) else {
            panic!("{} must use the application renderer", story.id.as_str());
        };
        assert_eq!(model.view(), View::Guild, "{}", story.id.as_str());
        assert!(model.domain().agents.values().all(|agent| {
            agent.attention == GuildAttention::Clear && agent.custom_status.is_none()
        }));
        let motion_story = story.id.as_str().starts_with("compat.motion-");
        if motion_story {
            assert!(
                model
                    .domain()
                    .agents
                    .values()
                    .any(|agent| agent.presence == Presence::Working)
            );
            assert!(
                model
                    .domain()
                    .agents
                    .values()
                    .any(|agent| agent.presence == Presence::Idle)
            );
        } else {
            let presences = model
                .domain()
                .agents
                .values()
                .map(|agent| agent.presence)
                .collect::<Vec<_>>();
            assert!(
                [
                    Presence::Working,
                    Presence::Blocked,
                    Presence::Done,
                    Presence::Idle,
                ]
                .into_iter()
                .all(|presence| presences.contains(&presence)),
                "{}",
                story.id.as_str()
            );
        }
        assert!(model.output_preview().is_none(), "{}", story.id.as_str());
        assert!(
            model.domain().chronicle.entries().is_empty(),
            "{}",
            story.id.as_str()
        );
    }
}

#[test]
fn every_non_atlas_application_story_uses_the_production_fixture_bridge() {
    for story in catalogue() {
        let fixture = (story.build)(&StoryContext::fixed());
        let is_widget_atlas = matches!(
            story.id.as_str(),
            "widgets.adventurer-cards" | "widgets.chambers" | "widgets.guild-regions"
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
        connected
            .domain()
            .agents
            .values()
            .all(|agent| agent.attention == GuildAttention::Clear && agent.custom_status.is_none())
    );
    assert!(connected.output_preview().is_none());
    assert!(connected.domain().chronicle.entries().is_empty());
    assert!(
        connected
            .selected_agent_key()
            .is_some_and(|selected| connected.domain().agents.contains_key(selected))
    );
    assert!(
        mixed
            .domain()
            .agents
            .values()
            .any(|agent| agent.presence != Presence::Working)
    );
    let mixed_domain = mixed.domain();
    assert!(mixed.output_preview().is_some_and(|preview| {
        mixed_domain
            .agents
            .values()
            .any(|agent| agent.pane_id == preview.pane_id)
    }));
    for entry in mixed_domain.chronicle.entries() {
        assert!(
            entry
                .adventurer
                .as_ref()
                .is_none_or(|key| mixed_domain.agents.contains_key(key))
        );
        assert!(
            entry
                .campaign
                .as_ref()
                .is_none_or(|key| mixed_domain.campaigns.contains_key(key))
        );
        assert!(entry.pane.as_ref().is_none_or(|pane| {
            mixed_domain
                .agents
                .values()
                .any(|agent| &agent.pane_id == pane)
        }));
    }
}

#[test]
fn application_shows_cover_representative_production_breakpoints() {
    for story in catalogue() {
        let StoryFixture::Application(model) = (story.build)(&StoryContext::fixed()) else {
            continue;
        };
        let mut observed = HashSet::new();
        for width in representative_axis(
            story.viewport.minimum_width,
            story.viewport.reference_width,
            &[79, 80, 119, 120],
        ) {
            for height in representative_axis(
                story.viewport.minimum_height,
                story.viewport.reference_height,
                &[19, 20, 23, 24, 31, 32],
            ) {
                let projection =
                    questmancer::ui::render_projection_for(&model, Rect::new(0, 0, width, height));
                observed.extend(projection_assets(&model, &projection));
            }
        }
        observed.retain(|asset| !story.owns.contains(asset));
        assert_eq!(
            story.shows.iter().copied().collect::<HashSet<_>>(),
            observed,
            "{}",
            story.id.as_str()
        );
    }
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

#[test]
fn intermediate_named_delve_branches_are_structural_and_inventoried() {
    let story = catalogue()
        .iter()
        .find(|story| story.id.as_str() == "scenes.delve-library")
        .unwrap();
    let StoryFixture::Application(model) = (story.build)(&StoryContext::fixed()) else {
        panic!("library must be an application fixture");
    };
    let reference = questmancer::ui::render_projection_for(&model, Rect::new(0, 0, 130, 36));
    let minimum = questmancer::ui::render_projection_for(&model, Rect::new(0, 0, 60, 18));
    let intermediate = questmancer::ui::render_projection_for(&model, Rect::new(0, 0, 60, 36));
    assert!(reference.visible_agents.iter().all(|agent| {
        agent.chamber == Some(questmancer::ui::ChamberPresentation::CompactScene)
    }));
    assert!(
        minimum
            .visible_agents
            .iter()
            .all(|agent| { agent.chamber == Some(questmancer::ui::ChamberPresentation::Text) })
    );
    assert!(
        intermediate
            .visible_agents
            .iter()
            .any(|agent| { agent.chamber == Some(questmancer::ui::ChamberPresentation::Full) })
    );
    assert!(
        story
            .shows
            .contains(&AssetId::Widget(WidgetAsset::ChamberCompact))
    );
    assert!(
        story
            .shows
            .contains(&AssetId::Widget(WidgetAsset::ChamberFull))
    );
}

fn projection_assets(
    model: &Model,
    projection: &questmancer::ui::RenderProjection,
) -> HashSet<AssetId> {
    use questmancer::ui::{ChamberPresentation, GuildRegion, PersonaRenderMode};

    let mut assets = HashSet::new();
    for region in &projection.guild_regions {
        assets.insert(AssetId::Widget(match region {
            GuildRegion::QuestBoard => WidgetAsset::QuestBoard,
            GuildRegion::Party => WidgetAsset::Party,
            GuildRegion::Summons => WidgetAsset::Summons,
            GuildRegion::Chronicle => WidgetAsset::Chronicle,
            GuildRegion::AdventurerProfile => WidgetAsset::AdventurerProfile,
            GuildRegion::Scrying => WidgetAsset::Scrying,
            GuildRegion::Spoils => WidgetAsset::Spoils,
        }));
    }
    if let Some(agent) = projection
        .guild_profile_agent
        .as_ref()
        .and_then(|key| model.domain().agents.get(key))
    {
        assets.insert(AssetId::Class(agent.persona.class));
        assets.insert(AssetId::Ancestry(agent.persona.ancestry));
    }
    if projection.delve_connected_scene_visible {
        assets.insert(AssetId::Scene(SceneAsset::ConnectedDelves));
    }
    assets.extend(
        projection
            .delve_variants
            .iter()
            .copied()
            .map(AssetId::DelveVariant),
    );
    for rendered in &projection.visible_agents {
        let agent = model.domain().agents.get(&rendered.key).unwrap();
        assets.insert(AssetId::Pose(rendered.pose));
        match rendered.chamber {
            Some(ChamberPresentation::Full) => {
                assets.insert(AssetId::Widget(WidgetAsset::ChamberFull));
            }
            Some(ChamberPresentation::Text | ChamberPresentation::CompactScene) => {
                assets.insert(AssetId::Widget(WidgetAsset::ChamberCompact));
            }
            Some(ChamberPresentation::Hidden) | None => {}
        }
        if rendered.persona == PersonaRenderMode::Full {
            assets.extend(persona_assets(&agent.persona));
        }
    }
    assets
}

fn persona_assets(persona: &AdventurerPersona) -> [AssetId; 14] {
    let appearance = persona.appearance;
    [
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
