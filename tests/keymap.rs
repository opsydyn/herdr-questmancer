use std::mem::discriminant;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use questmancer::{
    ledger::{LedgerPageId, page_body},
    ui::{
        input::action_for,
        keymap::{BINDINGS, UNLISTED, lines},
    },
};

/// Every key a user can press, as far as the keyring is concerned.
///
/// Function keys and modifiers are covered by the plain keys they alias, so
/// sweeping printable ASCII plus the named keys reaches every branch of
/// `action_for` that a keyring would ever list.
fn candidate_keys() -> Vec<KeyEvent> {
    let printable =
        (0x20..0x7f_u8).map(|byte| KeyEvent::new(KeyCode::Char(byte as char), KeyModifiers::NONE));
    let named = [
        KeyCode::Enter,
        KeyCode::Esc,
        KeyCode::Tab,
        KeyCode::Backspace,
        KeyCode::Up,
        KeyCode::Down,
        KeyCode::Left,
        KeyCode::Right,
        KeyCode::F(1),
        KeyCode::F(2),
    ]
    .into_iter()
    .map(|code| KeyEvent::new(code, KeyModifiers::NONE));
    printable.chain(named).collect()
}

/// The Ledger used to describe the controls in three hand-written sentences.
/// By the time anyone looked, four bindings were reachable and undocumented —
/// `Tab`, `!`, `c` and `n`/`N`. Prose about keys drifts from keys in silence,
/// because nothing fails when it does. This is what fails.
#[test]
fn every_bound_key_is_documented() {
    let documented = BINDINGS
        .iter()
        .map(|binding| discriminant(&binding.action))
        .chain(UNLISTED.iter().map(discriminant))
        .collect::<Vec<_>>();

    for key in candidate_keys() {
        let action = action_for(key);
        assert!(
            documented.contains(&discriminant(&action)),
            "{:?} produces {action:?}, which no keyring entry describes. Add it \
             to ui::keymap::BINDINGS, or to UNLISTED with a reason.",
            key.code
        );
    }
}

/// The reverse drift: an entry describing a key that no longer does anything.
#[test]
fn every_documented_binding_is_still_reachable() {
    let reachable = candidate_keys()
        .into_iter()
        .map(|key| discriminant(&action_for(key)))
        .collect::<Vec<_>>();

    for binding in BINDINGS {
        assert!(
            reachable.contains(&discriminant(&binding.action)),
            "the keyring lists {} for {:?}, but no key produces it any more",
            binding.keys,
            binding.action
        );
    }
}

/// The Ledger page is the table, not a retyping of it.
#[test]
fn the_ledger_keyring_page_is_generated_from_the_bindings() {
    let body = page_body(LedgerPageId::QuestmancersTools);
    assert_eq!(body, lines(), "the keyring page must render the table");
    assert_eq!(body.len(), BINDINGS.len());

    for binding in BINDINGS {
        assert!(
            body.iter()
                .any(|line| line.contains(binding.keys) && line.contains(binding.description)),
            "{} is missing from the rendered keyring",
            binding.keys
        );
    }
}

/// Other Ledger pages keep their authored prose.
#[test]
fn authored_ledger_pages_are_untouched() {
    for id in [
        LedgerPageId::Welcome,
        LedgerPageId::ReadingTheParty,
        LedgerPageId::SafeChronicle,
    ] {
        let body = page_body(id);
        assert!(!body.is_empty(), "{id:?} lost its text");
    }
}
