use crossterm::event::{Event, KeyCode, KeyModifiers};

use super::app::Action;

pub fn action_for_event(event: &Event) -> Action {
    let Event::Key(key) = event else {
        return Action::Ignore;
    };
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Action::Quit;
    }
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Action::NextStory,
        KeyCode::Char('k') | KeyCode::Up => Action::PreviousStory,
        KeyCode::Char('l') | KeyCode::Right => Action::NextCategory,
        KeyCode::Char('h') | KeyCode::Left => Action::PreviousCategory,
        KeyCode::Enter => Action::Inspect,
        KeyCode::Char('?') => Action::ToggleHelp,
        KeyCode::Esc => Action::Escape,
        KeyCode::Char('q') => Action::Quit,
        _ => Action::Ignore,
    }
}
