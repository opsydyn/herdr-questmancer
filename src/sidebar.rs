//! Questmancer-owned display tokens for Herdr's opt-in custom sidebar rows.
//!
//! The sidebar remains Herdr UI. This module derives small, truthful strings
//! from Questmancer's existing projection; it never changes an agent's title,
//! display name, state labels, or semantic status.
//!
//! The vocabulary is a character sheet, but every value is a fact Questmancer
//! already holds: the sigil is the adventurer's class, the condition is its
//! presence, the vigil is how long a summons has gone unanswered, and the
//! hoard counts spoils actually recorded in the Chronicle. Nothing here is
//! invented flavour dressed as data.

use std::collections::BTreeMap;

use crate::app::CharacterSet;
use crate::domain::{
    AdventurerClass, AgentKey, Campaign, Chronicle, ChronicleEvent, DomainState, Presence,
    Timestamp,
};

pub const QUEST_SIGIL: &str = "quest_sigil";
pub const QUEST_ROLE: &str = "quest_role";
pub const QUEST_EPITHET: &str = "quest_epithet";
pub const QUEST_CONDITION: &str = "quest_condition";
pub const QUEST_OMEN: &str = "quest_omen";
pub const QUEST_TRINKET: &str = "quest_trinket";
pub const QUEST_VIGIL: &str = "quest_vigil";
pub const QUEST_HOARD: &str = "quest_hoard";
pub const QUEST_CAMPAIGN: &str = "quest_campaign";
pub const QUEST_PARTY: &str = "quest_party";

/// Every token Questmancer publishes, for the guard over our documented
/// sidebar configurations. A published example naming a token we do not
/// report would be as broken as one Herdr cannot parse.
pub const ALL_QUEST_TOKENS: &[&str] = &[
    QUEST_SIGIL,
    QUEST_ROLE,
    QUEST_EPITHET,
    QUEST_CONDITION,
    QUEST_OMEN,
    QUEST_TRINKET,
    QUEST_VIGIL,
    QUEST_HOARD,
    QUEST_CAMPAIGN,
    QUEST_PARTY,
];
pub const SIDEBAR_SOURCE: &str = "plugin:opsydyn.questmancer";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SidebarProjection {
    pub agents: Vec<SidebarAgentTokens>,
    pub campaigns: Vec<SidebarCampaignTokens>,
}

impl SidebarProjection {
    #[must_use]
    pub fn from_domain(domain: &DomainState, now: Timestamp, character_set: CharacterSet) -> Self {
        let agents = domain
            .agents
            .values()
            .map(|agent| SidebarAgentTokens {
                pane_id: agent.pane_id.clone(),
                tokens: BTreeMap::from([
                    (
                        QUEST_SIGIL.to_owned(),
                        sigil(agent.persona.class, character_set).to_owned(),
                    ),
                    (
                        QUEST_ROLE.to_owned(),
                        format!("{:?} {:?}", agent.persona.ancestry, agent.persona.class),
                    ),
                    (
                        QUEST_EPITHET.to_owned(),
                        agent.persona.epithet.as_str().to_owned(),
                    ),
                    (
                        QUEST_CONDITION.to_owned(),
                        condition(agent.presence).to_owned(),
                    ),
                    (QUEST_OMEN.to_owned(), omen(agent.presence).to_owned()),
                    (
                        QUEST_TRINKET.to_owned(),
                        trinket(agent.persona.appearance.keepsake, character_set),
                    ),
                    (
                        QUEST_VIGIL.to_owned(),
                        vigil(agent.presence, agent.presence_since, now, character_set),
                    ),
                    (
                        QUEST_HOARD.to_owned(),
                        hoard(&domain.chronicle, Some(&agent.key), character_set),
                    ),
                ]),
            })
            .collect();
        let campaigns = domain
            .campaigns
            .values()
            .map(|campaign| SidebarCampaignTokens {
                workspace_id: campaign.workspace_id.clone(),
                tokens: BTreeMap::from([
                    (
                        QUEST_CAMPAIGN.to_owned(),
                        campaign_summary(campaign, domain),
                    ),
                    (
                        QUEST_PARTY.to_owned(),
                        party_sigils(campaign, domain, character_set),
                    ),
                    (
                        QUEST_HOARD.to_owned(),
                        hoard(&domain.chronicle, None, character_set),
                    ),
                ]),
            })
            .collect();

        Self { agents, campaigns }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SidebarAgentTokens {
    pub pane_id: crate::domain::PaneId,
    pub tokens: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SidebarCampaignTokens {
    pub workspace_id: crate::domain::WorkspaceId,
    pub tokens: BTreeMap<String, String>,
}

/// One glyph per class. Terminals that cannot render the symbols get a
/// two-letter abbreviation instead, so the row never collapses to a box.
const fn sigil(class: AdventurerClass, character_set: CharacterSet) -> &'static str {
    match (class, character_set) {
        (AdventurerClass::Barbarian, CharacterSet::Unicode) => "⚔",
        (AdventurerClass::Barbarian, CharacterSet::Ascii) => "Ba",
        (AdventurerClass::Bard, CharacterSet::Unicode) => "♪",
        (AdventurerClass::Bard, CharacterSet::Ascii) => "Bd",
        (AdventurerClass::Cleric, CharacterSet::Unicode) => "✚",
        (AdventurerClass::Cleric, CharacterSet::Ascii) => "Cl",
        (AdventurerClass::Druid, CharacterSet::Unicode) => "❧",
        (AdventurerClass::Druid, CharacterSet::Ascii) => "Dr",
        (AdventurerClass::Paladin, CharacterSet::Unicode) => "✜",
        (AdventurerClass::Paladin, CharacterSet::Ascii) => "Pa",
        (AdventurerClass::Ranger, CharacterSet::Unicode) => "↟",
        (AdventurerClass::Ranger, CharacterSet::Ascii) => "Rg",
        (AdventurerClass::Rogue, CharacterSet::Unicode) => "✦",
        (AdventurerClass::Rogue, CharacterSet::Ascii) => "Ro",
        (AdventurerClass::Wizard, CharacterSet::Unicode) => "✧",
        (AdventurerClass::Wizard, CharacterSet::Ascii) => "Wz",
        (AdventurerClass::Artificer, CharacterSet::Unicode) => "⚙",
        (AdventurerClass::Artificer, CharacterSet::Ascii) => "Ar",
        (AdventurerClass::Runewright, CharacterSet::Unicode) => "ᚱ",
        (AdventurerClass::Runewright, CharacterSet::Ascii) => "Rw",
        (AdventurerClass::Testmender, CharacterSet::Unicode) => "✎",
        (AdventurerClass::Testmender, CharacterSet::Ascii) => "Tm",
        (AdventurerClass::Pathseeker, CharacterSet::Unicode) => "⌖",
        (AdventurerClass::Pathseeker, CharacterSet::Ascii) => "Ps",
        (AdventurerClass::Mage, CharacterSet::Unicode) => "☠",
        (AdventurerClass::Mage, CharacterSet::Ascii) => "Mg",
        (AdventurerClass::Sorcerer, CharacterSet::Unicode) => "☼",
        (AdventurerClass::Sorcerer, CharacterSet::Ascii) => "So",
    }
}

/// Presence in condition vocabulary. This is a second name for the same fact
/// Herdr's own `state_text` reports; it never replaces it.
const fn condition(presence: Presence) -> &'static str {
    match presence {
        Presence::Working => "Concentrating",
        Presence::Blocked => "Restrained",
        Presence::Done => "Triumphant",
        Presence::Idle => "Resting",
        Presence::Exited => "Departed",
        Presence::Unknown => "Blinded",
    }
}

const fn omen(presence: Presence) -> &'static str {
    match presence {
        Presence::Working => "on expedition",
        Presence::Blocked => "seeks counsel",
        Presence::Done => "returned with spoils",
        Presence::Idle => "at the hearth",
        Presence::Exited => "departed the guild",
        Presence::Unknown => "whereabouts unknown",
    }
}

/// The keepsake this adventurer carries, labelled by a glyph.
///
/// Herdr's sidebar rows hold tokens and nothing else — there is no literal
/// text element — so a row cannot supply its own "Trinket:" label. Anything a
/// value needs in order to be self-explanatory has to travel inside the token,
/// the way `hoard` already carries its own count wording.
fn trinket(keepsake: crate::domain::Keepsake, character_set: CharacterSet) -> String {
    let glyph = match character_set {
        CharacterSet::Unicode => "❖",
        CharacterSet::Ascii => "~",
    };
    // `PressedLeaf` reads better as two words on a character sheet.
    let raw = format!("{keepsake:?}");
    let mut spaced = String::with_capacity(raw.len() + 2);
    for (index, character) in raw.char_indices() {
        if index > 0 && character.is_uppercase() {
            spaced.push(' ');
        }
        spaced.push(character);
    }
    format!("{glyph} {spaced}")
}

/// An exhaustion track for a summons nobody has answered yet.
///
/// Only a blocked adventurer keeps a vigil, so every other state returns an
/// empty string and the row simply carries nothing. The levels are the six of
/// the tabletop track, filled as the wait lengthens.
fn vigil(
    presence: Presence,
    since: Timestamp,
    now: Timestamp,
    character_set: CharacterSet,
) -> String {
    if presence != Presence::Blocked {
        return String::new();
    }
    let minutes = since.elapsed_until(now).as_secs() / 60;
    let level = match minutes {
        0 => 1,
        1..=4 => 2,
        5..=14 => 3,
        15..=29 => 4,
        30..=59 => 5,
        _ => 6,
    };
    let (filled, empty) = match character_set {
        CharacterSet::Unicode => ('●', '○'),
        CharacterSet::Ascii => ('#', '-'),
    };
    let mut track = String::with_capacity(6);
    for step in 1..=6 {
        track.push(if step <= level { filled } else { empty });
    }
    track
}

/// The bag of holding: spoils this adventurer has actually brought back, or
/// the guild's total when no adventurer is given.
///
/// The Chronicle is bounded, so this counts what the current Chronicle still
/// remembers rather than all history, and the wording says so.
fn hoard(
    chronicle: &Chronicle,
    adventurer: Option<&AgentKey>,
    character_set: CharacterSet,
) -> String {
    let count = chronicle
        .entries()
        .iter()
        .filter(|entry| entry.event == ChronicleEvent::SpoilsReturned)
        .filter(|entry| match adventurer {
            Some(key) => entry.adventurer.as_ref() == Some(key),
            None => true,
        })
        .count();
    let glyph = match character_set {
        CharacterSet::Unicode => "◈",
        CharacterSet::Ascii => "*",
    };
    match count {
        0 => format!("{glyph} empty"),
        1 => format!("{glyph} 1 spoil"),
        count => format!("{glyph} {count} spoils"),
    }
}

/// The party's marching order as class sigils, so a campaign's composition
/// reads at a glance without a row per adventurer.
fn party_sigils(campaign: &Campaign, domain: &DomainState, character_set: CharacterSet) -> String {
    let mut sigils = String::new();
    for key in &campaign.party {
        let Some(agent) = domain.agents.get(key) else {
            continue;
        };
        if agent.presence == Presence::Exited {
            continue;
        }
        if !sigils.is_empty() && character_set == CharacterSet::Ascii {
            sigils.push(' ');
        }
        sigils.push_str(sigil(agent.persona.class, character_set));
    }
    sigils
}

fn campaign_summary(campaign: &Campaign, domain: &DomainState) -> String {
    let party = campaign
        .party
        .iter()
        .filter_map(|key| domain.agents.get(key))
        .filter(|agent| agent.presence != Presence::Exited)
        .count();
    let summons = campaign
        .party
        .iter()
        .filter_map(|key| domain.agents.get(key))
        .filter(|agent| agent.presence == Presence::Blocked)
        .count();

    let party = match party {
        0 => "no adventurers".to_owned(),
        1 => "1 adventurer".to_owned(),
        count => format!("{count} adventurers"),
    };
    match summons {
        0 => party,
        1 => format!("{party} · 1 summons"),
        count => format!("{party} · {count} summons"),
    }
}
