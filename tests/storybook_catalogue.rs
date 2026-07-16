#![cfg(feature = "storybook")]

use questmancer::storybook::catalogue::{Category, Story, StoryId, Viewport, validate_coverage};
use questmancer::{
    app::{Model, View},
    storybook::{
        AssetId, WidgetAsset, asset_inventory,
        catalogue::catalogue,
        fixtures::{StoryContext, StoryFixture},
    },
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

    let report = validate_coverage(&atlas_inventory, catalogue()).unwrap();
    assert_eq!(report.owned(), atlas_inventory.len());
    assert!(catalogue().iter().all(|story| story.shows.is_empty()));
}

#[test]
fn atlas_catalogue_uses_canonical_ids_and_viewports() {
    let ids = catalogue()
        .iter()
        .map(|story| story.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        ids,
        [
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
        ]
    );
    assert!(catalogue().iter().all(|story| {
        story.category == Category::AssetAtlas && story.viewport == Viewport::new(120, 36, 60, 18)
    }));
}
