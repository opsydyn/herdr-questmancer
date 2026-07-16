use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::app::{Modal, View};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Switch(View),
    ShowHelp,
    Dismiss,
    Redraw,
    Quit,
    Next,
    Previous,
    First,
    Last,
    Observe,
    Counsel,
    AcknowledgeSummons,
    Refresh,
    InspectSpoils,
    CycleRegion,
    Search,
    TypeCharacter(char),
    Submit,
    Backspace,
    ClearInput,
    None,
}

pub fn action_for_event(event: &Event) -> Action {
    action_for_event_in(event, &Modal::None)
}

pub fn action_for_event_in(event: &Event, modal: &Modal) -> Action {
    match event {
        Event::Key(key) if matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) => {
            action_for_in(*key, modal)
        }
        Event::Resize(_, _) => Action::Redraw,
        _ => Action::None,
    }
}

pub fn action_for(key: KeyEvent) -> Action {
    action_for_in(key, &Modal::None)
}

fn action_for_in(key: KeyEvent, modal: &Modal) -> Action {
    if matches!(modal, Modal::Help) {
        return match key.code {
            KeyCode::Esc => Action::Dismiss,
            KeyCode::Char('?') => Action::ShowHelp,
            _ => Action::None,
        };
    }
    if matches!(modal, Modal::Counsel { .. } | Modal::Search { .. }) {
        return modal_action(key);
    }
    if matches!(key.code, KeyCode::Char('c')) && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Action::Quit;
    }

    match key.code {
        KeyCode::Char('1') | KeyCode::F(1) => Action::Switch(View::Guild),
        KeyCode::Char('2') | KeyCode::F(2) => Action::Switch(View::Delve),
        KeyCode::Char('?') => Action::ShowHelp,
        KeyCode::Esc => Action::Dismiss,
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('j') | KeyCode::Down => Action::Next,
        KeyCode::Char('k') | KeyCode::Up => Action::Previous,
        KeyCode::Char('g') => Action::First,
        KeyCode::Char('G') => Action::Last,
        KeyCode::Enter => Action::Observe,
        KeyCode::Char('r') => Action::Counsel,
        KeyCode::Char(' ') => Action::AcknowledgeSummons,
        KeyCode::Char('o') => Action::Refresh,
        KeyCode::Char('v') => Action::InspectSpoils,
        KeyCode::Tab => Action::CycleRegion,
        KeyCode::Char('/') => Action::Search,
        _ => Action::None,
    }
}

fn modal_action(key: KeyEvent) -> Action {
    if matches!(key.code, KeyCode::Char('u')) && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Action::ClearInput;
    }
    match key.code {
        KeyCode::Esc => Action::Dismiss,
        KeyCode::Enter => Action::Submit,
        KeyCode::Backspace => Action::Backspace,
        KeyCode::Char(character) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::TypeCharacter(character)
        }
        _ => Action::None,
    }
}
