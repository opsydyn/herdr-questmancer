#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum LedgerPageId {
    Welcome,
    ReadingTheParty,
    QuestmancersTools,
    GuildStanding,
    SafeChronicle,
}

impl LedgerPageId {
    pub const ALL: [Self; 5] = [
        Self::Welcome,
        Self::ReadingTheParty,
        Self::QuestmancersTools,
        Self::GuildStanding,
        Self::SafeChronicle,
    ];

    #[must_use]
    pub fn next(self) -> Self {
        let index = Self::ALL
            .iter()
            .position(|candidate| *candidate == self)
            .unwrap_or(0);
        Self::ALL[index.saturating_add(1).min(Self::ALL.len() - 1)]
    }

    #[must_use]
    pub fn previous(self) -> Self {
        let index = Self::ALL
            .iter()
            .position(|candidate| *candidate == self)
            .unwrap_or(0);
        Self::ALL[index.saturating_sub(1)]
    }

    #[must_use]
    pub fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|candidate| *candidate == self)
            .unwrap_or(0)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LedgerPage {
    pub id: LedgerPageId,
    pub title: &'static str,
    pub body: &'static [&'static str],
}

#[must_use]
pub const fn page(id: LedgerPageId) -> LedgerPage {
    match id {
        LedgerPageId::Welcome => LedgerPage {
            id,
            title: "Welcome to the Guild",
            body: &[
                "You are the Questmancer. Herdr workspaces become campaigns and coding agents become adventurers.",
                "The Guild Hall keeps the whole party visible. The Delve shows active work in the dungeon.",
                "Questmancer projects Herdr facts into this scene; it does not manually control an adventurer's state.",
            ],
        },
        LedgerPageId::ReadingTheParty => LedgerPage {
            id,
            title: "Reading the Party",
            body: &[
                "Working adventurers are carrying out a commission. Needs counsel means an adventurer is waiting for you.",
                "Completed marks observed spoils. Resting is idle, and Unknown remains unknown.",
                "The guild never invents a successful ending that Herdr did not report.",
            ],
        },
        LedgerPageId::QuestmancersTools => LedgerPage {
            id,
            title: "Questmancer's Keyring",
            // Rendered from `ui::keymap::BINDINGS`; see `page_body`. The three
            // sentences that used to live here were already missing four
            // bindings by the time anyone noticed.
            body: &[],
        },
        LedgerPageId::GuildStanding => LedgerPage {
            id,
            title: "The Guild's Standing",
            // Filled from the live score by `page_body`.
            body: &[],
        },
        LedgerPageId::SafeChronicle => LedgerPage {
            id,
            title: "Keeping a Safe Chronicle",
            body: &[
                "Questmancer stays local. Herdr owns topology and live agent facts; Questmancer stores only small durable intent and its Chronicle.",
                "The managed Questmancer pane is never an adventurer and cannot receive focus, counsel, output or Reviewr commands.",
                "Guarded tests use disposable panes and fresh IDs. Herdr 0.8.0 cannot synthesize an explicit done transition.",
            ],
        },
    }
}

/// The page's text, generated where the page is a view of something else.
///
/// The keyring is built from the real binding table rather than retyped, so a
/// new key cannot ship undocumented.
#[must_use]
pub fn page_body(id: LedgerPageId) -> Vec<String> {
    match id {
        LedgerPageId::QuestmancersTools => crate::ui::keymap::lines(),
        // Standing needs the live score, so it is rendered by
        // `standing_page_body` where the model is in reach.
        LedgerPageId::GuildStanding => Vec::new(),
        other => page(other)
            .body
            .iter()
            .map(|line| (*line).to_owned())
            .collect(),
    }
}

/// The standing page, which needs the guild's live experience.
///
/// Kept beside the score rather than in the page table because it is the one
/// page whose text changes with use.
#[must_use]
pub fn standing_page_body(experience: u64) -> Vec<String> {
    let mut lines = crate::rank::ledger_lines(experience);
    lines.push(String::new());
    lines.push("Standing is earned by work the Chronicle recorded:".to_owned());
    lines.push("spoils returned, and campaigns closed.".to_owned());
    lines.push(String::new());
    lines.push("It is one score for this Questmancer, not one per".to_owned());
    lines.push("adventurer: parties change, the guild endures. It".to_owned());
    lines.push("unlocks nothing and gates nothing.".to_owned());
    lines
}
