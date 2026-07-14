use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use herdr_webmaster::{
    app::View,
    ui::input::{Action, action_for},
};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

#[test]
fn number_and_function_keys_switch_views() {
    assert_eq!(action_for(key(KeyCode::Char('1'))), Action::Switch(View::Desk));
    assert_eq!(action_for(key(KeyCode::F(1))), Action::Switch(View::Desk));
    assert_eq!(action_for(key(KeyCode::Char('2'))), Action::Switch(View::Cafe));
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
