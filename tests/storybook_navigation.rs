#![cfg(feature = "storybook")]

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use questmancer::{
    app::{Model, View},
    storybook::{
        app::{Action, Exit, Mode, StorybookApp, reduce},
        catalogue::{Category, Story, StoryId, Viewport},
        fixtures::{StoryContext, StoryFixture},
        input::action_for_event,
    },
};

#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "the StoryBuilder contract intentionally accepts a borrowed context"
)]
fn build(_: &StoryContext) -> StoryFixture {
    StoryFixture::Application(Model::new(View::Guild))
}

fn navigation_catalogue() -> Vec<Story> {
    Category::ALL
        .into_iter()
        .enumerate()
        .map(|(index, category)| {
            Story::new(
                StoryId::new(["atlas", "widgets", "scenes", "compat"][index]),
                "Navigation fixture",
                category,
                "Navigation fixture",
                Viewport::new(80, 24, 40, 12),
                build,
                &[],
                &[],
            )
        })
        .collect()
}

#[test]
fn story_and_category_navigation_clamp() {
    let stories = navigation_catalogue();
    let mut app = StorybookApp::new(&stories);
    assert_eq!(
        reduce(&mut app, Action::PreviousStory, &stories),
        Exit::Continue
    );
    assert_eq!(app.selected_index(), 0);
    reduce(&mut app, Action::NextCategory, &stories);
    assert_eq!(app.selected_story(&stories).category, Category::Widgets);
    assert_eq!(app.index_within_category(&stories), 0);
}

#[test]
fn inspect_and_escape_return_to_the_catalogue_before_quitting() {
    let stories = navigation_catalogue();
    let mut app = StorybookApp::new(&stories);
    reduce(&mut app, Action::Inspect, &stories);
    assert_eq!(app.mode(), Mode::Inspect);
    assert_eq!(reduce(&mut app, Action::Escape, &stories), Exit::Continue);
    assert_eq!(app.mode(), Mode::Catalogue);
    assert_eq!(reduce(&mut app, Action::Escape, &stories), Exit::Quit);
}

#[test]
fn keys_map_without_leaking_production_actions() {
    let key = |code| Event::Key(KeyEvent::new(code, KeyModifiers::NONE));
    assert_eq!(
        action_for_event(&key(KeyCode::Char('j'))),
        Action::NextStory
    );
    assert_eq!(action_for_event(&key(KeyCode::Enter)), Action::Inspect);
    assert_eq!(
        action_for_event(&key(KeyCode::Char('?'))),
        Action::ToggleHelp
    );
    let ctrl_c = Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
    assert_eq!(action_for_event(&ctrl_c), Action::Quit);
}
