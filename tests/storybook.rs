#![cfg(feature = "storybook")]

use questmancer::storybook::{
    app::StorybookApp,
    catalogue::{catalogue, validate_catalogue},
    fixtures::StoryContext,
    ui,
};
use ratatui::{Terminal, backend::TestBackend};

#[test]
fn catalogue_contains_every_production_scene_interaction_once() {
    let titles = catalogue()
        .iter()
        .map(|story| story.title)
        .collect::<Vec<_>>();

    for title in [
        "Interaction / Selected Adventurer",
        "Interaction / Counsel Parchment",
        "Interaction / Search Parchment",
        "Interaction / Scrying Parchment",
        "Interaction / Help Parchment",
        "Interaction / Narrow Parchment",
    ] {
        assert_eq!(
            titles
                .iter()
                .filter(|candidate| **candidate == title)
                .count(),
            1,
            "missing or duplicated Storybook interaction: {title}"
        );
    }
}

#[test]
fn every_production_story_owns_one_asset_exactly_once() {
    let report = validate_catalogue().expect("production Storybook coverage is complete");
    assert_eq!(report.owned(), catalogue().len());
    assert!(catalogue().iter().all(|story| story.owns.len() == 1));
}

#[test]
fn every_production_story_renders_at_its_minimum_viewport() {
    let stories = catalogue();
    for (index, story) in stories.iter().enumerate() {
        let mut app = StorybookApp::new(stories);
        app.select(index, stories);
        let backend = TestBackend::new(
            story.viewport.minimum_width,
            story.viewport.minimum_height.saturating_add(2),
        );
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| ui::render(frame, &app, stories, &StoryContext::fixed()))
            .unwrap();
    }
}
