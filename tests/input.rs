use std::time::Duration;

use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use questmancer::{
    app::{Modal, View},
    domain::AgentKey,
    scene::{SceneActorRegion, SceneFrame, pixel::PixelRect, stage::WorldScene},
    ui::input::{
        Action, action_for, action_for_event, action_for_event_in, action_for_scene_event_in,
    },
};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

#[test]
fn number_and_function_keys_switch_views() {
    assert_eq!(
        action_for(key(KeyCode::Char('1'))),
        Action::Switch(View::Guild)
    );
    assert_eq!(action_for(key(KeyCode::F(1))), Action::Switch(View::Guild));
    assert_eq!(
        action_for(key(KeyCode::Char('2'))),
        Action::Switch(View::Delve)
    );
    assert_eq!(action_for(key(KeyCode::F(2))), Action::Switch(View::Delve));
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
fn clicking_a_rendered_adventurer_selects_only_that_agent() {
    let frame = SceneFrame {
        world: WorldScene::GuildHall,
        next_frame_in: Some(Duration::from_millis(100)),
        actors: vec![SceneActorRegion {
            agent: AgentKey::new("codex"),
            bounds: PixelRect::new(10, 20, 8, 14),
        }],
    };
    let click = Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 12,
        row: 11,
        modifiers: KeyModifiers::NONE,
    });

    assert_eq!(
        action_for_scene_event_in(&click, &Modal::None, &frame),
        Action::SelectAt {
            column: 12,
            row: 11
        }
    );
    assert_eq!(
        action_for_scene_event_in(&click, &Modal::Help, &frame),
        Action::None
    );
}

#[test]
fn clicking_empty_world_space_dismisses_the_adventurer_card() {
    let frame = SceneFrame {
        world: WorldScene::GuildHall,
        next_frame_in: None,
        actors: Vec::new(),
    };
    let click = Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 12,
        row: 11,
        modifiers: KeyModifiers::NONE,
    });

    assert_eq!(
        action_for_scene_event_in(&click, &Modal::None, &frame),
        Action::Dismiss
    );
}

#[test]
fn guild_hall_navigation_and_actions_have_explicit_keys() {
    assert_eq!(action_for(key(KeyCode::Char('j'))), Action::Next);
    assert_eq!(action_for(key(KeyCode::Down)), Action::Next);
    assert_eq!(action_for(key(KeyCode::Char('k'))), Action::Previous);
    assert_eq!(action_for(key(KeyCode::Enter)), Action::Observe);
    assert_eq!(action_for(key(KeyCode::Char('r'))), Action::Counsel);
    assert_eq!(
        action_for(key(KeyCode::Char(' '))),
        Action::AcknowledgeSummons
    );
    assert_eq!(action_for(key(KeyCode::Char('o'))), Action::Refresh);
    assert_eq!(action_for(key(KeyCode::Char('v'))), Action::InspectSpoils);
    assert_eq!(action_for(key(KeyCode::Tab)), Action::CycleRegion);
    assert_eq!(action_for(key(KeyCode::Char('g'))), Action::First);
    assert_eq!(action_for(key(KeyCode::Char('G'))), Action::Last);
    assert_eq!(action_for(key(KeyCode::Char('/'))), Action::Search);
}

#[test]
fn counsel_modal_treats_global_shortcuts_as_composed_text() {
    let modal = Modal::Counsel {
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

#[test]
fn help_modal_accepts_only_dismissal_keys() {
    for code in [
        KeyCode::Char('q'),
        KeyCode::Char('j'),
        KeyCode::Enter,
        KeyCode::Char('/'),
        KeyCode::Char('r'),
    ] {
        assert_eq!(
            action_for_event_in(&Event::Key(key(code)), &Modal::Help),
            Action::None,
            "help leaked {code:?}"
        );
    }
    assert_eq!(
        action_for_event_in(&Event::Key(key(KeyCode::Esc)), &Modal::Help),
        Action::Dismiss
    );
    assert_eq!(
        action_for_event_in(&Event::Key(key(KeyCode::Char('?'))), &Modal::Help),
        Action::ShowHelp
    );
}

#[test]
fn scrying_modal_accepts_only_refresh_and_dismissal() {
    for code in [
        KeyCode::Char('q'),
        KeyCode::Char('j'),
        KeyCode::Enter,
        KeyCode::Char('/'),
    ] {
        assert_eq!(
            action_for_event_in(&Event::Key(key(code)), &Modal::Scrying),
            Action::None,
            "scrying leaked {code:?}"
        );
    }
    assert_eq!(
        action_for_event_in(&Event::Key(key(KeyCode::Char('o'))), &Modal::Scrying),
        Action::Refresh
    );
    assert_eq!(
        action_for_event_in(&Event::Key(key(KeyCode::Esc)), &Modal::Scrying),
        Action::Dismiss
    );
}
