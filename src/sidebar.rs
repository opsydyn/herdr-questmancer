//! Questmancer-owned display tokens for Herdr's opt-in custom sidebar rows.
//!
//! The sidebar remains Herdr UI. This module derives small, truthful strings
//! from Questmancer's existing projection; it never changes an agent's title,
//! display name, state labels, or semantic status.

use std::collections::BTreeMap;

use crate::domain::{Campaign, DomainState, Presence};

pub const QUEST_ROLE: &str = "quest_role";
pub const QUEST_OMEN: &str = "quest_omen";
pub const QUEST_CAMPAIGN: &str = "quest_campaign";
pub const SIDEBAR_SOURCE: &str = "plugin:opsydyn.questmancer";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SidebarProjection {
    pub agents: Vec<SidebarAgentTokens>,
    pub campaigns: Vec<SidebarCampaignTokens>,
}

impl SidebarProjection {
    #[must_use]
    pub fn from_domain(domain: &DomainState) -> Self {
        let agents = domain
            .agents
            .values()
            .map(|agent| SidebarAgentTokens {
                pane_id: agent.pane_id.clone(),
                tokens: BTreeMap::from([
                    (
                        QUEST_ROLE.to_owned(),
                        format!("{:?} {:?}", agent.persona.ancestry, agent.persona.class),
                    ),
                    (QUEST_OMEN.to_owned(), omen(agent.presence).to_owned()),
                ]),
            })
            .collect();
        let campaigns = domain
            .campaigns
            .values()
            .map(|campaign| SidebarCampaignTokens {
                workspace_id: campaign.workspace_id.clone(),
                tokens: BTreeMap::from([(
                    QUEST_CAMPAIGN.to_owned(),
                    campaign_summary(campaign, domain),
                )]),
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

fn omen(presence: Presence) -> &'static str {
    match presence {
        Presence::Working => "on expedition",
        Presence::Blocked => "seeks counsel",
        Presence::Done => "returned with spoils",
        Presence::Idle => "at the hearth",
        Presence::Exited => "departed the guild",
        Presence::Unknown => "whereabouts unknown",
    }
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
