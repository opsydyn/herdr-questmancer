#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum LedgerPageId {
    Welcome,
    ReadingTheParty,
    QuestmancersTools,
    SafeChronicle,
}

impl LedgerPageId {
    pub const ALL: [Self; 4] = [
        Self::Welcome,
        Self::ReadingTheParty,
        Self::QuestmancersTools,
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
            title: "Questmancer's Tools",
            body: &[
                "Use j/k, arrows or g/G to select; Enter observes the selected adventurer.",
                "Use r for counsel, o for scrying, / to search, Space to acknowledge summons and v for optional Reviewr spoils.",
                "Keys 1 and 2 move between the Guild Hall and Delve. Esc closes the current parchment.",
            ],
        },
        LedgerPageId::SafeChronicle => LedgerPage {
            id,
            title: "Keeping a Safe Chronicle",
            body: &[
                "Questmancer stays local. Herdr owns topology and live agent facts; Questmancer stores only small durable intent and its Chronicle.",
                "The managed Questmancer pane is never an adventurer and cannot receive focus, counsel, output or Reviewr commands.",
                "Guarded tests use disposable panes and fresh IDs. Herdr 0.7.4 cannot synthesize an explicit done transition.",
            ],
        },
    }
}
