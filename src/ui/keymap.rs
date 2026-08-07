//! The one place a key binding is described.
//!
//! The Librarian's Ledger used to carry three sentences of hand-written prose
//! summarising the controls. Every binding added after it was written went
//! undocumented — `Tab`, `!`, `c` and `n`/`N` were all reachable and none of
//! them appeared. Prose about keys drifts from keys silently, because nothing
//! fails when it does.
//!
//! So the Ledger page is generated from this table, and
//! `every_bound_key_is_documented` sweeps the real key handler and refuses any
//! action a key can produce that is missing here.

use super::input::Action;
use crate::app::View;

#[derive(Debug)]
pub struct Binding {
    pub keys: &'static str,
    pub description: &'static str,
    /// The action this key produces, so the drift guard can match it against
    /// what `action_for` actually returns.
    pub action: Action,
}

pub const BINDINGS: &[Binding] = &[
    Binding {
        keys: "1 / 2",
        description: "Enter the Guild Hall or the Delve",
        action: Action::Switch(View::Guild),
    },
    Binding {
        keys: "j / k",
        description: "Select the next or previous adventurer",
        action: Action::Next,
    },
    Binding {
        keys: "g / G",
        description: "Select the first or last adventurer",
        action: Action::First,
    },
    Binding {
        keys: "Tab",
        description: "Move to the next campaign's party",
        action: Action::NextCampaign,
    },
    Binding {
        keys: "!",
        description: "Jump to the next adventurer waiting on you",
        action: Action::NextUrgent,
    },
    Binding {
        keys: "Enter",
        description: "Observe the selected adventurer's Herdr pane",
        action: Action::Observe,
    },
    Binding {
        keys: "r",
        description: "Open the counsel parchment",
        action: Action::Counsel,
    },
    Binding {
        keys: "Space",
        description: "Acknowledge the selected summons",
        action: Action::AcknowledgeSummons,
    },
    Binding {
        keys: "o",
        description: "Scry the adventurer's recent output",
        action: Action::Refresh,
    },
    Binding {
        keys: "v",
        description: "Inspect spoils through Reviewr",
        action: Action::InspectSpoils,
    },
    Binding {
        keys: "c",
        description: "Read the Chronicle",
        action: Action::OpenChronicle,
    },
    Binding {
        keys: "/",
        description: "Search the party and campaigns",
        action: Action::Search,
    },
    Binding {
        keys: "n / N",
        description: "Walk to the next or previous search match",
        action: Action::NextResult,
    },
    Binding {
        keys: "?",
        description: "Open or close this Ledger",
        action: Action::ToggleLedger,
    },
    Binding {
        keys: "Esc",
        description: "Dismiss the open parchment",
        action: Action::Dismiss,
    },
    Binding {
        keys: "q",
        description: "Close Questmancer",
        action: Action::Quit,
    },
];

/// Actions a key can produce that get no keyring line of their own.
///
/// Two kinds, and nothing else. The reverse halves of pairs already listed —
/// `k`, `G` and `N` are documented on the `j / k`, `g / G` and `n / N` rows,
/// so a separate entry would say the same thing twice. And the text-editing
/// keys inside a parchment, which the parchment's own footer describes;
/// "every printable character types one character" is noise on a keyring.
///
/// The drift guard skips exactly this list, so anything genuinely new still
/// fails until it is either described or deliberately added here.
pub const UNLISTED: &[Action] = &[
    // Reverse halves of listed pairs.
    Action::Previous,
    Action::Last,
    Action::PreviousResult,
    // Parchment text editing.
    Action::TypeCharacter(' '),
    Action::Backspace,
    Action::ClearInput,
    Action::Submit,
    // Not key-driven at all.
    Action::Redraw,
    Action::None,
];

/// The keyring as the Ledger shows it, key column padded to line up.
#[must_use]
pub fn lines() -> Vec<String> {
    let width = BINDINGS
        .iter()
        .map(|binding| binding.keys.chars().count())
        .max()
        .unwrap_or(0);
    BINDINGS
        .iter()
        .map(|binding| {
            let keys = binding.keys;
            let padding = " ".repeat(width - keys.chars().count());
            format!("{keys}{padding}   {}", binding.description)
        })
        .collect()
}
