#![cfg(feature = "storybook")]

use proptest::prelude::*;
use questmancer::storybook::{
    AssetId, SceneFirstAsset,
    app::{Action, StorybookApp, reduce},
    catalogue::catalogue,
    fixtures::{StoryContext, StoryFixture},
    ui,
};
use ratatui::{Terminal, backend::TestBackend};

proptest! {
    #[test]
    fn every_story_renders_for_any_terminal_size(
        width in 1_u16..180,
        height in 1_u16..60,
        story_index in any::<usize>(),
    ) {
        let stories = catalogue();
        let mut app = StorybookApp::new(stories);
        app.select(story_index % stories.len(), stories);
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| ui::render(frame, &app, stories, &StoryContext::fixed()))
            .unwrap();
    }

    #[test]
    fn arbitrary_navigation_keeps_selection_valid(
        actions in prop::collection::vec(0_u8..8, 0..200),
    ) {
        let stories = catalogue();
        let mut app = StorybookApp::new(stories);
        for value in actions {
            let action = match value {
                0 => Action::NextStory,
                1 => Action::PreviousStory,
                2 => Action::NextCategory,
                3 => Action::PreviousCategory,
                4 => Action::Inspect,
                5 => Action::ToggleHelp,
                6 => Action::Escape,
                _ => Action::Ignore,
            };
            let _ = reduce(&mut app, action, stories);
            prop_assert!(app.selected_index() < stories.len());
        }
    }
}

#[test]
fn every_story_renders_at_zero_width() {
    render_every_story_at(0, 24);
}

#[test]
fn every_story_renders_at_zero_height() {
    render_every_story_at(80, 0);
}

#[test]
fn scene_first_stories_render_at_their_fixed_review_sizes() {
    let stories = catalogue();
    let scene_first_stories = stories
        .iter()
        .enumerate()
        .filter(|(_, story)| {
            story
                .owns
                .iter()
                .any(|asset| matches!(asset, AssetId::SceneFirst(_)))
        })
        .collect::<Vec<_>>();
    assert_eq!(scene_first_stories.len(), SceneFirstAsset::ALL.len());
    for (index, story) in scene_first_stories {
        for (width, height) in [
            (story.viewport.minimum_width, story.viewport.minimum_height),
            (
                story.viewport.reference_width,
                story.viewport.reference_height,
            ),
        ] {
            let mut app = StorybookApp::new(stories);
            app.select(index, stories);
            let backend = TestBackend::new(width, height);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal
                .draw(|frame| ui::render(frame, &app, stories, &StoryContext::fixed()))
                .unwrap();
        }
    }
}

#[test]
fn scene_first_compatibility_stories_cover_motion_and_the_minimum_viewport() {
    let stories = catalogue();
    for (id, expected_motion, expected_minimum) in [
        (
            "scenes.scene-first-motion-full",
            questmancer::app::Motion::Full,
            (80, 24),
        ),
        (
            "scenes.scene-first-motion-reduced",
            questmancer::app::Motion::Reduced,
            (80, 24),
        ),
        (
            "scenes.scene-first-motion-none",
            questmancer::app::Motion::None,
            (80, 24),
        ),
        (
            "scenes.scene-first-minimum-viewport",
            questmancer::app::Motion::None,
            (40, 18),
        ),
    ] {
        let story = stories
            .iter()
            .find(|story| story.id.as_str() == id)
            .unwrap_or_else(|| panic!("missing {id}"));
        assert_eq!(
            (story.viewport.minimum_width, story.viewport.minimum_height),
            expected_minimum
        );
        let StoryFixture::PixelScene(fixture) = (story.build)(&StoryContext::fixed()) else {
            panic!("{id} must use the scene-first RGB renderer");
        };
        assert_eq!(fixture.snapshot.motion, expected_motion);
    }
}

fn render_every_story_at(width: u16, height: u16) {
    let stories = catalogue();
    let context = StoryContext::fixed();
    for index in 0..stories.len() {
        let mut app = StorybookApp::new(stories);
        app.select(index, stories);
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| ui::render(frame, &app, stories, &context))
            .unwrap();
    }
}
