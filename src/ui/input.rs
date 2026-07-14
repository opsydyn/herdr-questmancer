use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::app::View;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Switch(View),
    ShowHelp,
    Dismiss,
    Redraw,
    Quit,
    None,
}

pub const fn action_for_event(event: &Event) -> Action {
    match event {
        Event::Key(key) if matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) => {
            action_for(*key)
        }
        Event::Resize(_, _) => Action::Redraw,
        _ => Action::None,
    }
}

pub const fn action_for(key: KeyEvent) -> Action {
    if matches!(key.code, KeyCode::Char('c')) && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Action::Quit;
    }

    match key.code {
        KeyCode::Char('1') | KeyCode::F(1) => Action::Switch(View::Desk),
        KeyCode::Char('2') | KeyCode::F(2) => Action::Switch(View::Cafe),
        KeyCode::Char('?') => Action::ShowHelp,
        KeyCode::Esc => Action::Dismiss,
        KeyCode::Char('q') => Action::Quit,
        _ => Action::None,
    }
}
