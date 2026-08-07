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
        description: "Guild Hall / Delve",
        action: Action::Switch(View::Guild),
    },
    Binding {
        keys: "j / k",
        description: "Select next / previous",
        action: Action::Next,
    },
    Binding {
        keys: "g / G",
        description: "Select first / last",
        action: Action::First,
    },
    Binding {
        keys: "Tab",
        description: "Next campaign's party",
        action: Action::NextCampaign,
    },
    Binding {
        keys: "!",
        description: "Jump to who is waiting",
        action: Action::NextUrgent,
    },
    Binding {
        keys: "Enter",
        description: "Observe the Herdr pane",
        action: Action::Observe,
    },
    Binding {
        keys: "r",
        description: "Send counsel",
        action: Action::Counsel,
    },
    Binding {
        keys: "Space",
        description: "Acknowledge the summons",
        action: Action::AcknowledgeSummons,
    },
    Binding {
        keys: "s",
        description: "Set the summons aside",
        action: Action::DeferSummons,
    },
    Binding {
        keys: "o",
        description: "Scry recent output",
        action: Action::Refresh,
    },
    Binding {
        keys: "v",
        description: "Inspect spoils in Reviewr",
        action: Action::InspectSpoils,
    },
    Binding {
        keys: "c",
        description: "Read the Chronicle",
        action: Action::OpenChronicle,
    },
    Binding {
        keys: "/",
        description: "Search party and campaigns",
        action: Action::Search,
    },
    Binding {
        keys: "n / N",
        description: "Next / previous match",
        action: Action::NextResult,
    },
    Binding {
        keys: "m",
        description: "Motion: full/reduced/still",
        action: Action::CycleMotion,
    },
    Binding {
        keys: "u",
        description: "Unicode / ASCII glyphs",
        action: Action::CycleCharacterSet,
    },
    Binding {
        keys: "p",
        description: "Truecolour / 16 colours",
        action: Action::CycleColorMode,
    },
    Binding {
        keys: "j / k, wheel",
        description: "Scroll an open parchment",
        action: Action::ScrollDown,
    },
    Binding {
        keys: "?",
        description: "Open or close this Ledger",
        action: Action::ToggleLedger,
    },
    Binding {
        keys: "Esc",
        description: "Dismiss the parchment",
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
    Action::ScrollUp,
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

/// The keyring in two columns, for a Ledger wide enough to hold them.
///
/// The list outgrew a single column: at twenty-one bindings it needed more
/// rows than a thirty-row terminal could give the page, and the entries at the
/// bottom — `Esc` and `q` — fell off. A keyring that hides keys is the same
/// failure as prose that omits them.
#[must_use]
pub fn paired_lines() -> Vec<String> {
    let single = lines();
    let width = single
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);
    let half = single.len().div_ceil(2);
    (0..half)
        .map(|index| {
            let left = &single[index];
            single.get(index + half).map_or_else(
                || left.clone(),
                |right| {
                    let padding = " ".repeat(width - left.chars().count() + 3);
                    format!("{left}{padding}{right}")
                },
            )
        })
        .collect()
}
