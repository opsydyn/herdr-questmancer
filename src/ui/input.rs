use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};

use crate::{
    app::{Modal, View},
    scene::SceneFrame,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Switch(View),
    ToggleLedger,
    Dismiss,
    Redraw,
    Quit,
    Next,
    Previous,
    First,
    Last,
    SelectAt { column: u16, row: u16 },
    Observe,
    Counsel,
    AcknowledgeSummons,
    Refresh,
    InspectSpoils,
    NextCampaign,
    OpenChronicle,
    NextResult,
    PreviousResult,
    DeferSummons,
    CycleMotion,
    CycleCharacterSet,
    CycleColorMode,
    NextUrgent,
    Search,
    TypeCharacter(char),
    Submit,
    Backspace,
    ClearInput,
    None,
}

pub fn action_for_scene_event_in(event: &Event, modal: &Modal, scene: &SceneFrame) -> Action {
    if modal != &Modal::None {
        return action_for_event_in(event, modal);
    }
    if let Event::Mouse(mouse) = event
        && mouse.kind == MouseEventKind::Down(MouseButton::Left)
    {
        return scene
            .target_at(mouse.column, mouse.row)
            .map_or(Action::Dismiss, |_| Action::SelectAt {
                column: mouse.column,
                row: mouse.row,
            });
    }
    action_for_event_in(event, modal)
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
    if matches!(modal, Modal::Scrying) {
        return match key.code {
            KeyCode::Esc => Action::Dismiss,
            KeyCode::Char('o') => Action::Refresh,
            _ => Action::None,
        };
    }
    if matches!(modal, Modal::Chronicle) {
        // The Chronicle is a reading surface. Only closing it does anything,
        // so no key leaks through to move a selection you cannot see.
        return match key.code {
            KeyCode::Esc | KeyCode::Char('c') => Action::Dismiss,
            _ => Action::None,
        };
    }
    if matches!(modal, Modal::LibrarianLedger { .. }) {
        return match key.code {
            KeyCode::Esc => Action::Dismiss,
            KeyCode::Char('?') => Action::ToggleLedger,
            KeyCode::Char('j') | KeyCode::Right | KeyCode::Down => Action::Next,
            KeyCode::Char('k') | KeyCode::Left | KeyCode::Up => Action::Previous,
            KeyCode::Char('g') => Action::First,
            KeyCode::Char('G') => Action::Last,
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
        KeyCode::Char('?') => Action::ToggleLedger,
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
        KeyCode::Tab => Action::NextCampaign,
        // Deliberately not `n`: that is being kept for cycling search results.
        KeyCode::Char('!') => Action::NextUrgent,
        KeyCode::Char('c') => Action::OpenChronicle,
        KeyCode::Char('n') => Action::NextResult,
        KeyCode::Char('N') => Action::PreviousResult,
        KeyCode::Char('s') => Action::DeferSummons,
        KeyCode::Char('m') => Action::CycleMotion,
        KeyCode::Char('u') => Action::CycleCharacterSet,
        KeyCode::Char('p') => Action::CycleColorMode,
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
