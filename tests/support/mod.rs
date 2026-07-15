use std::collections::BTreeMap;

use questmancer::{
    domain::{
        AgentKey, Attention, AttentionReason, DomainState, Guestbook, PaneId, Presence, Site,
        TabId, Timestamp, WorkspaceId,
    },
    herdr::protocol::{SessionSnapshotResult, SuccessResponse},
    persistence::{PersistedStateV1, parse_state, publish_state},
};

pub(crate) mod strategies;

#[allow(unused_imports)]
pub(crate) use strategies::{
    agent, agent_identity, agent_status, attention, domain_state, domain_with_one_agent,
    guestbook_event, pane_id, persisted_state, status_event, timestamp, topology_events,
};

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

#[allow(dead_code)]
pub(crate) async fn assert_atomic_publication(first: PersistedStateV1, second: PersistedStateV1) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("state.json");
    publish_state(&path, &first).await.unwrap();

    let writer_path = path.clone();
    let writer_first = first.clone();
    let writer_second = second.clone();
    let writer = tokio::spawn(async move {
        for index in 0..4 {
            let state = if index % 2 == 0 {
                &writer_second
            } else {
                &writer_first
            };
            publish_state(&writer_path, state).await.unwrap();
            tokio::task::yield_now().await;
        }
    });

    let reader_path = path.clone();
    let reader = tokio::spawn(async move {
        for _ in 0..32 {
            let bytes = tokio::fs::read(&reader_path).await.unwrap();
            let observed = parse_state(&reader_path, &bytes).unwrap();
            assert!(observed == first || observed == second);
            tokio::task::yield_now().await;
        }
    });

    writer.await.unwrap();
    reader.await.unwrap();
}
