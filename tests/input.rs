use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use herdr_webmaster::{
    app::{Modal, View},
    ui::input::{Action, action_for, action_for_event, action_for_event_in},
};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

#[test]
fn number_and_function_keys_switch_views() {
    assert_eq!(
        action_for(key(KeyCode::Char('1'))),
        Action::Switch(View::Desk)
    );
    assert_eq!(action_for(key(KeyCode::F(1))), Action::Switch(View::Desk));
    assert_eq!(
        action_for(key(KeyCode::Char('2'))),
        Action::Switch(View::Cafe)
    );
    assert_eq!(action_for(key(KeyCode::F(2))), Action::Switch(View::Cafe));
}

#[test]
fn global_keys_map_to_explicit_actions() {
    assert_eq!(action_for(key(KeyCode::Char('?'))), Action::ShowHelp);
    assert_eq!(action_for(key(KeyCode::Esc)), Action::Dismiss);
    assert_eq!(action_for(key(KeyCode::Char('q'))), Action::Quit);
    assert_eq!(
        action_for(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
        Action::Quit
    );
}

#[test]
fn unrelated_keys_are_ignored() {
    assert_eq!(action_for(key(KeyCode::Char('x'))), Action::None);
}

#[test]
fn resize_requests_a_redraw() {
    assert_eq!(action_for_event(&Event::Resize(100, 40)), Action::Redraw);
}

#[test]
fn desk_navigation_and_actions_have_explicit_keys() {
    assert_eq!(action_for(key(KeyCode::Char('j'))), Action::Next);
    assert_eq!(action_for(key(KeyCode::Down)), Action::Next);
    assert_eq!(action_for(key(KeyCode::Char('k'))), Action::Previous);
    assert_eq!(action_for(key(KeyCode::Enter)), Action::Visit);
    assert_eq!(action_for(key(KeyCode::Char('r'))), Action::Reply);
    assert_eq!(action_for(key(KeyCode::Char(' '))), Action::MarkSeen);
    assert_eq!(action_for(key(KeyCode::Char('o'))), Action::Refresh);
    assert_eq!(action_for(key(KeyCode::Char('v'))), Action::Reviewr);
    assert_eq!(action_for(key(KeyCode::Tab)), Action::CycleRegion);
}

#[test]
fn reply_modal_treats_global_shortcuts_as_composed_text() {
    let modal = Modal::Reply {
        draft: String::new(),
    };

    assert_eq!(
        action_for_event_in(&Event::Key(key(KeyCode::Char('q'))), &modal),
        Action::TypeCharacter('q')
    );
    assert_eq!(
        action_for_event_in(&Event::Key(key(KeyCode::Enter)), &modal),
        Action::Submit
    );
    assert_eq!(
        action_for_event_in(&Event::Key(key(KeyCode::Backspace)), &modal),
        Action::Backspace
    );
    assert_eq!(
        action_for_event_in(
            &Event::Key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL)),
            &modal
        ),
        Action::ClearInput
    );
    assert_eq!(
        action_for_event_in(&Event::Key(key(KeyCode::Esc)), &modal),
        Action::Dismiss
    );
}
