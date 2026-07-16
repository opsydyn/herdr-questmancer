#![cfg(feature = "storybook")]

use proptest::prelude::*;
use questmancer::storybook::{
    app::{Action, StorybookApp, reduce},
    catalogue::catalogue,
    fixtures::StoryContext,
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
