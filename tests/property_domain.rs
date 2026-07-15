#[allow(dead_code)]
mod support;

use std::collections::BTreeMap;

use herdr_webmaster::{
    domain::{
        AgentPersona, AttentionReason, GuestbookEntry, PersonaKey, Presence, Site, SiteStatus,
        WorkspaceId,
    },
    update::update,
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn persona_key_and_appearance_are_deterministic(
        (agent, workspace_root) in support::agent_identity(),
    ) {
        let first_key = PersonaKey::for_agent(&agent, workspace_root.as_deref());
        let second_key = PersonaKey::for_agent(&agent, workspace_root.as_deref());
        let first_persona = AgentPersona::for_agent(&agent, workspace_root.as_deref());
        let second_persona = AgentPersona::for_agent(&agent, workspace_root.as_deref());

        prop_assert_eq!(&first_key, &second_key);
        prop_assert_eq!(&first_persona, &second_persona);
        prop_assert_eq!(
            first_persona.appearance,
            AgentPersona::appearance_for_key(&first_key),
        );
    }

    #[test]
    fn marking_attention_seen_is_idempotent(attention in support::attention()) {
        let once = attention.clone().mark_seen();
        let twice = once.clone().mark_seen();

        prop_assert_eq!(once, twice);
    }

    #[test]
    fn duplicate_semantic_events_are_idempotent(
        state in support::domain_with_one_agent(),
        status in support::agent_status(),
        revision_delta in 0_u64..100,
    ) {
        let current_revision = state.agents.values().next().unwrap().pane_revision;
        let event = support::status_event(
            &state,
            current_revision + revision_delta,
            status,
        );

        let (once, first_commands) = update(state.clone(), event.clone());
        let (same_once, same_first_commands) = update(state, event.clone());
        prop_assert_eq!(&same_once, &once);
        prop_assert_eq!(&same_first_commands, &first_commands);

        let (twice, duplicate_commands) = update(once.clone(), event);
        prop_assert_eq!(twice, once);
        prop_assert!(duplicate_commands.is_empty());
    }

    #[test]
    fn stale_revisions_never_regress_state(
        state in support::domain_with_one_agent(),
        stale_revision in 0_u64..100,
        status in support::agent_status(),
    ) {
        let current = state.agents.values().next().unwrap().pane_revision;
        prop_assume!(stale_revision < current);
        let event = support::status_event(&state, stale_revision, status);

        let (next, commands) = update(state.clone(), event);

        prop_assert_eq!(next, state);
        prop_assert!(commands.is_empty());
    }

    #[test]
    fn arbitrary_topology_changes_keep_selection_valid(
        mut state in support::domain_state(),
        events in support::topology_events(),
    ) {
        for event in events {
            let (next, commands) = update(state.clone(), event.clone());
            let (same_next, same_commands) = update(state, event);

            prop_assert_eq!(&same_next, &next);
            prop_assert_eq!(&same_commands, &commands);
            prop_assert!(next
                .selected_agent
                .as_ref()
                .is_none_or(|key| next.agents.contains_key(key)));
            state = next;
        }
    }

    #[test]
    fn site_status_priority_is_independent_of_agent_insertion_order(
        generated_agents in prop::collection::vec(support::agent(), 0..=8),
    ) {
        let mut forward_agents = BTreeMap::new();
        for agent in &generated_agents {
            forward_agents.insert(agent.key.clone(), agent.clone());
        }
        let mut reverse_agents = BTreeMap::new();
        for agent in forward_agents.values().rev() {
            reverse_agents.insert(agent.key.clone(), agent.clone());
        }

        let forward_keys = forward_agents.keys().cloned().collect::<Vec<_>>();
        let mut reverse_keys = forward_keys.clone();
        reverse_keys.reverse();
        let forward_site = Site {
            workspace_id: WorkspaceId::new("site"),
            label: "site".to_owned(),
            cwd: "/tmp/site".into(),
            agents: forward_keys,
        };
        let reverse_site = Site {
            agents: reverse_keys,
            ..forward_site.clone()
        };
        let expected = expected_site_status(&forward_agents);

        prop_assert_eq!(forward_site.status(&forward_agents), expected);
        prop_assert_eq!(reverse_site.status(&reverse_agents), expected);
    }

    #[test]
    fn guestbook_event_ids_are_stable_for_equal_identity_inputs(
        occurred_at in support::timestamp(),
        pane in prop::option::of(support::pane_id()),
        pane_revision in any::<u64>(),
        kind in support::guestbook_event(),
        first_summary in ".{0,32}",
        second_summary in ".{0,32}",
    ) {
        let first = GuestbookEntry::new(
            occurred_at,
            Some("agent-first".into()),
            Some("workspace-first".into()),
            pane.clone(),
            pane_revision,
            kind,
            first_summary,
        );
        let second = GuestbookEntry::new(
            occurred_at,
            Some("agent-second".into()),
            Some("workspace-second".into()),
            pane,
            pane_revision,
            kind,
            second_summary,
        );

        prop_assert_eq!(first.id, second.id);
    }
}

fn expected_site_status(
    agents: &BTreeMap<herdr_webmaster::domain::AgentKey, herdr_webmaster::domain::Agent>,
) -> SiteStatus {
    if agents
        .values()
        .any(|agent| agent.presence == Presence::Blocked)
    {
        SiteStatus::NeedsWebmaster
    } else if agents.values().any(|agent| {
        agent.attention.is_unseen()
            && agent.attention.reason() == Some(AttentionReason::WorkCompleted)
    }) {
        SiteStatus::UpdateReady
    } else if agents
        .values()
        .any(|agent| agent.presence == Presence::Working)
    {
        SiteStatus::Updating
    } else if agents
        .values()
        .any(|agent| agent.presence != Presence::Exited)
    {
        SiteStatus::Online
    } else {
        SiteStatus::Offline
    }
}
