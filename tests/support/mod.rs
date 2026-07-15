use std::collections::BTreeMap;

use herdr_webmaster::{
    domain::{
        AgentKey, Attention, AttentionReason, DomainState, Guestbook, PaneId, Presence, Site,
        TabId, Timestamp, WorkspaceId,
    },
    herdr::protocol::{SessionSnapshotResult, SuccessResponse},
};

pub(crate) mod strategies;

pub(crate) use strategies::{domain_state, persisted_state};

pub(crate) fn fixture_domain() -> DomainState {
    let response: SuccessResponse<SessionSnapshotResult> =
        serde_json::from_str(include_str!("../fixtures/herdr/session_snapshot.json")).unwrap();
    DomainState::from_snapshot(&response.result.snapshot, Timestamp::from_millis(1_000))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LiveFacts {
    sites: BTreeMap<WorkspaceId, Site>,
    agents: BTreeMap<AgentKey, LiveAgentFacts>,
    guestbook: Guestbook,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LiveAgentFacts {
    key: AgentKey,
    pane_id: PaneId,
    workspace_id: WorkspaceId,
    tab_id: TabId,
    name: String,
    custom_status: Option<String>,
    presence: Presence,
    presence_since: Timestamp,
    attention_episode: Option<(AttentionReason, Timestamp, Option<Timestamp>)>,
    focused: bool,
    pane_revision: u64,
}

pub(crate) fn live_facts(domain: &DomainState) -> LiveFacts {
    LiveFacts {
        sites: domain.sites.clone(),
        agents: domain
            .agents
            .iter()
            .map(|(key, agent)| {
                let attention_episode = match agent.attention {
                    Attention::Clear => None,
                    Attention::Unseen { reason, since } | Attention::Seen { reason, since } => {
                        Some((reason, since, None))
                    }
                    Attention::Snoozed {
                        reason,
                        since,
                        until,
                    } => Some((reason, since, Some(until))),
                };
                (
                    key.clone(),
                    LiveAgentFacts {
                        key: agent.key.clone(),
                        pane_id: agent.pane_id.clone(),
                        workspace_id: agent.workspace_id.clone(),
                        tab_id: agent.tab_id.clone(),
                        name: agent.name.clone(),
                        custom_status: agent.custom_status.clone(),
                        presence: agent.presence,
                        presence_since: agent.presence_since,
                        attention_episode,
                        focused: agent.focused,
                        pane_revision: agent.pane_revision,
                    },
                )
            })
            .collect(),
        guestbook: domain.guestbook.clone(),
    }
}
