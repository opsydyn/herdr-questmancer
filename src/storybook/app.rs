use super::catalogue::{Category, Story};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Mode {
    #[default]
    Catalogue,
    Inspect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    NextStory,
    PreviousStory,
    NextCategory,
    PreviousCategory,
    Inspect,
    ToggleHelp,
    Escape,
    Quit,
    Ignore,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Exit {
    Continue,
    Quit,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorybookApp {
    selected: usize,
    mode: Mode,
    help_visible: bool,
}

impl StorybookApp {
    pub fn new(stories: &[Story]) -> Self {
        assert!(!stories.is_empty(), "Storybook catalogue must not be empty");
        Self {
            selected: 0,
            mode: Mode::Catalogue,
            help_visible: false,
        }
    }

    pub const fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn selected_story<'a>(&self, stories: &'a [Story]) -> &'a Story {
        &stories[self.selected]
    }

    pub fn index_within_category(&self, stories: &[Story]) -> usize {
        let category = self.selected_story(stories).category;
        stories[..self.selected]
            .iter()
            .filter(|story| story.category == category)
            .count()
    }

    pub const fn mode(&self) -> Mode {
        self.mode
    }

    pub const fn help_visible(&self) -> bool {
        self.help_visible
    }

    pub fn select(&mut self, index: usize, stories: &[Story]) {
        assert!(!stories.is_empty(), "Storybook catalogue must not be empty");
        self.selected = index.min(stories.len() - 1);
    }
}

pub fn reduce(app: &mut StorybookApp, action: Action, stories: &[Story]) -> Exit {
    match action {
        Action::NextStory => move_story(app, stories, Direction::Next),
        Action::PreviousStory => move_story(app, stories, Direction::Previous),
        Action::NextCategory => move_category(app, stories, Direction::Next),
        Action::PreviousCategory => move_category(app, stories, Direction::Previous),
        Action::Inspect => app.mode = Mode::Inspect,
        Action::ToggleHelp => app.help_visible = !app.help_visible,
        Action::Escape if app.help_visible => app.help_visible = false,
        Action::Escape if app.mode == Mode::Inspect => app.mode = Mode::Catalogue,
        Action::Escape | Action::Quit => return Exit::Quit,
        Action::Ignore => {}
    }
    Exit::Continue
}

#[derive(Clone, Copy)]
enum Direction {
    Next,
    Previous,
}

fn move_story(app: &mut StorybookApp, stories: &[Story], direction: Direction) {
    let category = app.selected_story(stories).category;
    let destination = match direction {
        Direction::Next => stories
            .iter()
            .enumerate()
            .skip(app.selected + 1)
            .find(|(_, story)| story.category == category)
            .map(|(index, _)| index),
        Direction::Previous => stories[..app.selected]
            .iter()
            .enumerate()
            .rev()
            .find(|(_, story)| story.category == category)
            .map(|(index, _)| index),
    };
    if let Some(index) = destination {
        app.select(index, stories);
    }
}

fn move_category(app: &mut StorybookApp, stories: &[Story], direction: Direction) {
    let current = Category::ALL
        .iter()
        .position(|category| *category == app.selected_story(stories).category)
        .expect("selected story category must be in Category::ALL");
    let first_story =
        |category: &Category| stories.iter().position(|story| story.category == *category);
    let destination = match direction {
        Direction::Next => Category::ALL[current + 1..].iter().find_map(first_story),
        Direction::Previous => Category::ALL[..current].iter().rev().find_map(first_story),
    };
    if let Some(index) = destination {
        app.select(index, stories);
    }
}
