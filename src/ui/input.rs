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
    OpenQuestBoard,
    OpenInbox,
    RequestReview,
    ChangeQuestLifecycle,
    EditQuestPrerequisites,
    EditQuestAssignment,
    ImportCheck,
    SettleReview,
    LinkReviewAdventurer,
    OpenAuthoring,
    OpenWarTable,
    OpenCharacterPicker,
    NameParty,
    ExplainStatus,
    ToggleChronicleChapter,
    NextResult,
    PreviousResult,
    DeferSummons,
    ScrollUp,
    ScrollDown,
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
    if let (Event::Mouse(mouse), Modal::CharacterPicker { .. }) = (event, modal) {
        return match mouse.kind {
            MouseEventKind::ScrollDown => Action::Next,
            MouseEventKind::ScrollUp => Action::Previous,
            _ => Action::None,
        };
    }
    // The wheel is the gesture people reach for on a wall of text, and it was
    // the one mouse event Questmancer ignored entirely.
    if let Event::Mouse(mouse) = event
        && matches!(
            modal,
            Modal::Scrying
                | Modal::Chronicle
                | Modal::QuestBoard
                | Modal::QuestInbox { .. }
                | Modal::WarTable
        )
    {
        return match mouse.kind {
            MouseEventKind::ScrollDown => Action::ScrollDown,
            MouseEventKind::ScrollUp => Action::ScrollUp,
            _ => Action::None,
        };
    }
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
    if matches!(modal, Modal::StatusExplanation(_)) {
        return if key.code == KeyCode::Esc {
            Action::Dismiss
        } else {
            Action::None
        };
    }
    if matches!(modal, Modal::CharacterPicker { .. }) {
        return character_action(key);
    }
    if matches!(modal, Modal::WarTable) {
        return match key.code {
            KeyCode::Esc | KeyCode::Char('w') => Action::Dismiss,
            KeyCode::Char('b') => Action::OpenQuestBoard,
            KeyCode::Char('!') => Action::NextUrgent,
            KeyCode::Char('j') => Action::Next,
            KeyCode::Char('k') => Action::Previous,
            KeyCode::Tab => Action::NextCampaign,
            KeyCode::BackTab => Action::PreviousResult,
            KeyCode::Down => Action::ScrollDown,
            KeyCode::Up => Action::ScrollUp,
            KeyCode::Enter => Action::Observe,
            _ => Action::None,
        };
    }
    if matches!(modal, Modal::QuestBoard) {
        return match key.code {
            KeyCode::Esc | KeyCode::Char('b') => Action::Dismiss,
            KeyCode::Char('a') => Action::OpenAuthoring,
            KeyCode::Char('i') => Action::OpenInbox,
            KeyCode::Char('v') if key.kind == KeyEventKind::Press => Action::RequestReview,
            KeyCode::Char('t') if key.kind == KeyEventKind::Press => Action::ChangeQuestLifecycle,
            KeyCode::Char('p') if key.kind == KeyEventKind::Press => Action::EditQuestPrerequisites,
            KeyCode::Char('y') if key.kind == KeyEventKind::Press => Action::EditQuestAssignment,
            KeyCode::Char('w') => Action::OpenWarTable,
            KeyCode::Char('!') => Action::NextUrgent,
            KeyCode::Char('j') | KeyCode::Tab => Action::Next,
            KeyCode::Char('k') | KeyCode::BackTab => Action::Previous,
            KeyCode::Down => Action::ScrollDown,
            KeyCode::Up => Action::ScrollUp,
            KeyCode::Enter => Action::Observe,
            KeyCode::Char(' ') => Action::AcknowledgeSummons,
            KeyCode::Char('s') => Action::DeferSummons,
            _ => Action::None,
        };
    }
    if let Modal::QuestInbox { detail } = modal {
        return inbox_action(key, *detail);
    }
    if matches!(modal, Modal::Scrying) {
        return match key.code {
            KeyCode::Esc => Action::Dismiss,
            KeyCode::Char('o') => Action::Refresh,
            KeyCode::Char('j') | KeyCode::Down => Action::ScrollDown,
            KeyCode::Char('k') | KeyCode::Up => Action::ScrollUp,
            _ => Action::None,
        };
    }
    if matches!(modal, Modal::Chronicle) {
        // The Chronicle is a reading surface. Only closing it does anything,
        // so no key leaks through to move a selection you cannot see.
        return match key.code {
            KeyCode::Esc | KeyCode::Char('c') => Action::Dismiss,
            KeyCode::Tab => Action::ToggleChronicleChapter,
            KeyCode::Char('j') | KeyCode::Down => Action::ScrollDown,
            KeyCode::Char('k') | KeyCode::Up => Action::ScrollUp,
            _ => Action::None,
        };
    }
    if matches!(modal, Modal::LibrarianLedger { .. }) {
        return ledger_action(key);
    }
    if matches!(
        modal,
        Modal::Counsel { .. } | Modal::Search { .. } | Modal::PartyName { .. }
    ) {
        return modal_action(key);
    }
    room_action(key)
}

fn inbox_action(key: KeyEvent, detail: bool) -> Action {
    match key.code {
        KeyCode::Char('1' | '2') | KeyCode::F(1 | 2) => room_action(key),
        KeyCode::Esc | KeyCode::Char('i') => Action::Dismiss,
        KeyCode::Char('b') => Action::OpenQuestBoard,
        KeyCode::Char('!') => Action::NextUrgent,
        KeyCode::Char('j') if detail => Action::ScrollDown,
        KeyCode::Char('k') if detail => Action::ScrollUp,
        KeyCode::Tab if detail => Action::NextResult,
        KeyCode::BackTab if detail => Action::PreviousResult,
        KeyCode::Char('j') | KeyCode::Tab => Action::Next,
        KeyCode::Char('k') | KeyCode::BackTab => Action::Previous,
        KeyCode::Down => Action::ScrollDown,
        KeyCode::Up => Action::ScrollUp,
        KeyCode::Enter if !detail && key.kind == KeyEventKind::Press => Action::Submit,
        KeyCode::Char('o') if key.kind == KeyEventKind::Press => Action::Observe,
        KeyCode::Char('g') if detail && key.kind == KeyEventKind::Press => Action::Refresh,
        KeyCode::Char('a') if key.kind == KeyEventKind::Press => Action::OpenAuthoring,
        KeyCode::Char('v') if key.kind == KeyEventKind::Press => Action::RequestReview,
        KeyCode::Char('c') if detail && key.kind == KeyEventKind::Press => Action::ImportCheck,
        KeyCode::Char('x') if detail && key.kind == KeyEventKind::Press => Action::SettleReview,
        KeyCode::Char('l') if detail && key.kind == KeyEventKind::Press => {
            Action::LinkReviewAdventurer
        }
        KeyCode::Char(' ') => Action::AcknowledgeSummons,
        KeyCode::Char('s') if key.kind == KeyEventKind::Press => Action::DeferSummons,
        _ => Action::None,
    }
}

fn room_action(key: KeyEvent) -> Action {
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
        KeyCode::Char('b') => Action::OpenQuestBoard,
        KeyCode::Char('w') => Action::OpenWarTable,
        KeyCode::Char('e') if key.kind == KeyEventKind::Press => Action::OpenCharacterPicker,
        KeyCode::Char('i') if key.kind == KeyEventKind::Press => Action::ExplainStatus,
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

fn ledger_action(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::Dismiss,
        KeyCode::Char('?') => Action::ToggleLedger,
        KeyCode::Char('e') if key.kind == KeyEventKind::Press => Action::OpenCharacterPicker,
        KeyCode::Char('p') if key.kind == KeyEventKind::Press => Action::NameParty,
        KeyCode::Char('j') | KeyCode::Right | KeyCode::Down => Action::Next,
        KeyCode::Char('k') | KeyCode::Left | KeyCode::Up => Action::Previous,
        KeyCode::Char('g') => Action::First,
        KeyCode::Char('G') => Action::Last,
        _ => Action::None,
    }
}

fn character_action(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::Dismiss,
        KeyCode::Char('j') | KeyCode::Down => Action::Next,
        KeyCode::Char('k') | KeyCode::Up => Action::Previous,
        KeyCode::Home => Action::First,
        KeyCode::End => Action::Last,
        KeyCode::Enter if key.kind == KeyEventKind::Press => Action::Submit,
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
