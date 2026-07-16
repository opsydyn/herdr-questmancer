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

fn story(id: &'static str, category: Category) -> Story {
    Story::new(
        StoryId::new(id),
        "Navigation fixture",
        category,
        "Navigation fixture",
        Viewport::new(80, 24, 40, 12),
        build,
        &[],
        &[],
    )
}

fn navigation_catalogue() -> Vec<Story> {
    vec![
        story("atlas.zero", Category::AssetAtlas),
        story("widgets.zero", Category::Widgets),
        story("atlas.one", Category::AssetAtlas),
        story("compat.zero", Category::Compatibility),
        story("widgets.one", Category::Widgets),
        story("scenes.zero", Category::FullScenes),
        story("widgets.two", Category::Widgets),
        story("compat.one", Category::Compatibility),
    ]
}

#[test]
fn story_navigation_stays_within_an_interleaved_category_and_clamps_both_ends() {
    let stories = navigation_catalogue();
    let mut app = StorybookApp::new(&stories);
    app.select(1, &stories);

    assert_eq!(app.index_within_category(&stories), 0);
    assert_eq!(
        reduce(&mut app, Action::PreviousStory, &stories),
        Exit::Continue
    );
    assert_eq!(app.selected_index(), 1);

    reduce(&mut app, Action::NextStory, &stories);
    assert_eq!(app.selected_index(), 4);
    assert_eq!(app.index_within_category(&stories), 1);
    reduce(&mut app, Action::NextStory, &stories);
    assert_eq!(app.selected_index(), 6);
    assert_eq!(app.index_within_category(&stories), 2);
    reduce(&mut app, Action::NextStory, &stories);
    assert_eq!(app.selected_index(), 6);

    reduce(&mut app, Action::PreviousStory, &stories);
    assert_eq!(app.selected_index(), 4);
    reduce(&mut app, Action::PreviousStory, &stories);
    assert_eq!(app.selected_index(), 1);
}

#[test]
fn category_navigation_selects_first_stories_and_preserves_outer_boundary_selection() {
    let stories = navigation_catalogue();
    let mut app = StorybookApp::new(&stories);

    app.select(2, &stories);
    reduce(&mut app, Action::PreviousCategory, &stories);
    assert_eq!(app.selected_index(), 2);

    reduce(&mut app, Action::NextCategory, &stories);
    assert_eq!(app.selected_index(), 1);
    reduce(&mut app, Action::NextCategory, &stories);
    assert_eq!(app.selected_index(), 5);
    reduce(&mut app, Action::NextCategory, &stories);
    assert_eq!(app.selected_index(), 3);

    app.select(7, &stories);
    reduce(&mut app, Action::NextCategory, &stories);
    assert_eq!(app.selected_index(), 7);
    reduce(&mut app, Action::PreviousCategory, &stories);
    assert_eq!(app.selected_index(), 5);
    reduce(&mut app, Action::PreviousCategory, &stories);
    assert_eq!(app.selected_index(), 1);
    reduce(&mut app, Action::PreviousCategory, &stories);
    assert_eq!(app.selected_index(), 0);
}

#[test]
fn category_navigation_skips_empty_categories_in_both_directions() {
    let stories = vec![
        story("atlas.zero", Category::AssetAtlas),
        story("compat.zero", Category::Compatibility),
        story("atlas.one", Category::AssetAtlas),
        story("compat.one", Category::Compatibility),
    ];
    let mut app = StorybookApp::new(&stories);

    app.select(2, &stories);
    reduce(&mut app, Action::NextCategory, &stories);
    assert_eq!(app.selected_index(), 1);

    app.select(3, &stories);
    reduce(&mut app, Action::NextCategory, &stories);
    assert_eq!(app.selected_index(), 3);
    reduce(&mut app, Action::PreviousCategory, &stories);
    assert_eq!(app.selected_index(), 0);

    app.select(2, &stories);
    reduce(&mut app, Action::PreviousCategory, &stories);
    assert_eq!(app.selected_index(), 2);
}

#[test]
fn oversized_select_clamps_to_the_last_story() {
    let stories = navigation_catalogue();
    let mut app = StorybookApp::new(&stories);
    app.select(usize::MAX, &stories);
    assert_eq!(app.selected_index(), stories.len() - 1);
}

#[test]
#[should_panic(expected = "Storybook catalogue must not be empty")]
fn new_rejects_an_empty_catalogue() {
    StorybookApp::new(&[]);
}

#[test]
#[should_panic(expected = "Storybook catalogue must not be empty")]
fn select_rejects_an_empty_catalogue() {
    let stories = navigation_catalogue();
    let mut app = StorybookApp::new(&stories);
    app.select(0, &[]);
}

#[test]
fn escape_hides_help_then_leaves_inspection_then_quits_the_catalogue() {
    let stories = navigation_catalogue();
    let mut app = StorybookApp::new(&stories);
    reduce(&mut app, Action::Inspect, &stories);
    reduce(&mut app, Action::ToggleHelp, &stories);
    assert_eq!(app.mode(), Mode::Inspect);
    assert!(app.help_visible());

    assert_eq!(reduce(&mut app, Action::Escape, &stories), Exit::Continue);
    assert_eq!(app.mode(), Mode::Inspect);
    assert!(!app.help_visible());
    assert_eq!(reduce(&mut app, Action::Escape, &stories), Exit::Continue);
    assert_eq!(app.mode(), Mode::Catalogue);
    assert_eq!(reduce(&mut app, Action::Escape, &stories), Exit::Quit);
}

#[test]
fn explicit_quit_exits_from_every_mode_and_with_help_visible() {
    let stories = navigation_catalogue();

    let mut catalogue = StorybookApp::new(&stories);
    assert_eq!(reduce(&mut catalogue, Action::Quit, &stories), Exit::Quit);

    let mut inspect = StorybookApp::new(&stories);
    reduce(&mut inspect, Action::Inspect, &stories);
    assert_eq!(reduce(&mut inspect, Action::Quit, &stories), Exit::Quit);

    let mut help = StorybookApp::new(&stories);
    reduce(&mut help, Action::ToggleHelp, &stories);
    assert_eq!(reduce(&mut help, Action::Quit, &stories), Exit::Quit);
}

#[test]
fn ignore_is_a_state_preserving_continue() {
    let stories = navigation_catalogue();
    let mut app = StorybookApp::new(&stories);
    app.select(4, &stories);
    reduce(&mut app, Action::Inspect, &stories);
    reduce(&mut app, Action::ToggleHelp, &stories);
    let before = app.clone();

    assert_eq!(reduce(&mut app, Action::Ignore, &stories), Exit::Continue);
    assert_eq!(app, before);
}

#[test]
fn storybook_keys_map_completely_without_leaking_production_actions() {
    let key = |code, modifiers| Event::Key(KeyEvent::new(code, modifiers));
    for (code, action) in [
        (KeyCode::Char('j'), Action::NextStory),
        (KeyCode::Down, Action::NextStory),
        (KeyCode::Char('k'), Action::PreviousStory),
        (KeyCode::Up, Action::PreviousStory),
        (KeyCode::Char('l'), Action::NextCategory),
        (KeyCode::Right, Action::NextCategory),
        (KeyCode::Char('h'), Action::PreviousCategory),
        (KeyCode::Left, Action::PreviousCategory),
        (KeyCode::Enter, Action::Inspect),
        (KeyCode::Char('?'), Action::ToggleHelp),
        (KeyCode::Esc, Action::Escape),
        (KeyCode::Char('q'), Action::Quit),
    ] {
        assert_eq!(action_for_event(&key(code, KeyModifiers::NONE)), action);
    }

    assert_eq!(
        action_for_event(&key(KeyCode::Char('c'), KeyModifiers::CONTROL)),
        Action::Quit
    );
    assert_eq!(
        action_for_event(&key(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        )),
        Action::Quit
    );
    assert_eq!(
        action_for_event(&key(KeyCode::Char('c'), KeyModifiers::NONE)),
        Action::Ignore
    );
    assert_eq!(
        action_for_event(&key(KeyCode::Char('c'), KeyModifiers::ALT)),
        Action::Ignore
    );
    assert_eq!(
        action_for_event(&key(KeyCode::F(12), KeyModifiers::NONE)),
        Action::Ignore
    );
    assert_eq!(action_for_event(&Event::Resize(120, 40)), Action::Ignore);
}

#[test]
fn storybook_runtime_source_audit_rejects_direct_production_dependencies() {
    // Behavioral setup order and listener lifetime are covered by runtime unit
    // tests; this audit only guards direct source dependencies.
    let source = include_str!("../src/storybook/runtime.rs");
    assert!(source.contains("terminal::TerminalGuard"));

    for forbidden in [
        "config::",
        "herdr::",
        "persistence::",
        "runtime_loop::",
        "RuntimeRegistration",
        "std::env",
        "std::fs",
        "tokio::fs",
    ] {
        assert!(
            !source.contains(forbidden),
            "Storybook runtime must not import {forbidden}"
        );
    }
}
