#![cfg(feature = "storybook")]

use questmancer::storybook::catalogue::{Category, Story, StoryId, Viewport, validate_coverage};
use questmancer::{
    app::{Model, View},
    storybook::{
        AssetId, WidgetAsset, asset_inventory,
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
    let error = validate_coverage(
        &[BOARD],
        &[story("one", &[BOARD, PARTY]), story("two", &[BOARD])],
    )
    .unwrap_err();
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
