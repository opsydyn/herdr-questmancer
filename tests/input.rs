use std::time::Duration;

use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use questmancer::{
    app::{CounselPhase, Modal, View},
    domain::AgentKey,
    scene::{
        SceneActorRegion, SceneFrame, SceneInteractable, SceneInteractableRegion, SceneTarget,
        pixel::PixelRect, stage::WorldScene,
    },
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
    assert_eq!(action_for(key(KeyCode::Char('?'))), Action::ToggleLedger);
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
        interactables: Vec::new(),
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
        action_for_scene_event_in(
            &click,
            &Modal::LibrarianLedger {
                page: questmancer::ledger::LedgerPageId::Welcome,
            },
            &frame,
        ),
        Action::None
    );
}

#[test]
fn clicking_empty_world_space_dismisses_the_adventurer_card() {
    let frame = SceneFrame {
        world: WorldScene::GuildHall,
        next_frame_in: None,
        actors: Vec::new(),
        interactables: Vec::new(),
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
fn scene_targets_keep_librarian_hits_distinct_from_agents() {
    let frame = SceneFrame {
        world: WorldScene::GuildHall,
        next_frame_in: None,
        actors: vec![SceneActorRegion {
            agent: AgentKey::new("codex"),
            bounds: PixelRect::new(2, 4, 4, 6),
        }],
        interactables: vec![SceneInteractableRegion {
            kind: SceneInteractable::Librarian,
            bounds: PixelRect::new(10, 20, 6, 8),
        }],
    };

    assert_eq!(
        frame.target_at(3, 2),
        Some(SceneTarget::Agent(&AgentKey::new("codex")))
    );
    assert_eq!(
        frame.target_at(12, 10),
        Some(SceneTarget::Interactable(SceneInteractable::Librarian))
    );
    assert_eq!(
        frame.interactable_at(12, 13),
        Some(SceneInteractable::Librarian)
    );
    assert_eq!(frame.target_at(30, 20), None);
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
    assert_eq!(action_for(key(KeyCode::Tab)), Action::NextCampaign);
    assert_eq!(action_for(key(KeyCode::Char('!'))), Action::NextUrgent);
    assert_eq!(action_for(key(KeyCode::Char('c'))), Action::OpenChronicle);
    assert_eq!(action_for(key(KeyCode::Char('n'))), Action::NextResult);
    assert_eq!(action_for(key(KeyCode::Char('N'))), Action::PreviousResult);
    assert_eq!(action_for(key(KeyCode::Char('g'))), Action::First);
    assert_eq!(action_for(key(KeyCode::Char('G'))), Action::Last);
    assert_eq!(action_for(key(KeyCode::Char('/'))), Action::Search);
}

#[test]
fn counsel_modal_treats_global_shortcuts_as_composed_text() {
    let modal = Modal::Counsel {
        draft: String::new(),
        phase: CounselPhase::Drafting,
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
fn ledger_modal_accepts_only_navigation_and_dismissal_keys() {
    let modal = Modal::LibrarianLedger {
        page: questmancer::ledger::LedgerPageId::Welcome,
    };
    for code in [
        KeyCode::Char('q'),
        KeyCode::Enter,
        KeyCode::Char('/'),
        KeyCode::Char('r'),
    ] {
        assert_eq!(
            action_for_event_in(&Event::Key(key(code)), &modal),
            Action::None,
            "ledger leaked {code:?}"
        );
    }
    assert_eq!(
        action_for_event_in(&Event::Key(key(KeyCode::Esc)), &modal),
        Action::Dismiss
    );
    assert_eq!(
        action_for_event_in(&Event::Key(key(KeyCode::Char('?'))), &modal),
        Action::ToggleLedger
    );
    assert_eq!(
        action_for_event_in(&Event::Key(key(KeyCode::Right)), &modal),
        Action::Next
    );
    assert_eq!(
        action_for_event_in(&Event::Key(key(KeyCode::Left)), &modal),
        Action::Previous
    );
}

#[test]
fn scrying_modal_accepts_only_refresh_scrolling_and_dismissal() {
    for code in [KeyCode::Char('q'), KeyCode::Enter, KeyCode::Char('/')] {
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
    // Scrying fetches far more output than the parchment can show, so reading
    // it needs to be possible without leaking the party-moving keys.
    for (code, expected) in [
        (KeyCode::Char('j'), Action::ScrollDown),
        (KeyCode::Down, Action::ScrollDown),
        (KeyCode::Char('k'), Action::ScrollUp),
        (KeyCode::Up, Action::ScrollUp),
    ] {
        assert_eq!(
            action_for_event_in(&Event::Key(key(code)), &Modal::Scrying),
            expected,
            "{code:?} must scroll inside Scrying"
        );
    }
}
