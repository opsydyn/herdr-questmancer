#[allow(dead_code)]
mod support;

use std::collections::BTreeMap;

use herdr_webmaster::{
    app::{Model, View},
    domain::{
        AgentKey, AgentPersona, AttentionReason, DomainState, GuestbookEntry, GuestbookEvent,
        PersonaKey, Presence, Site, SiteStatus, WorkspaceId,
    },
    ui::cafe_scene::layout_bays,
    update::{AppEvent, Command, update},
};
use proptest::prelude::*;
use ratatui::layout::Rect;

proptest! {
    #[test]
    fn every_generated_agent_is_owned_by_exactly_one_visible_bay(
        workspaces in prop::collection::vec(support::strategies::workspace_id(), 0..=12),
        agents_per_workspace in prop::collection::vec(0usize..=4, 0..=12),
    ) {
        let mut sites = BTreeMap::new();
        let mut agents = BTreeMap::new();
        let template = support::fixture_domain().agents.values().next().unwrap().clone();
        for (workspace_index, workspace_id) in workspaces.into_iter().enumerate() {
            let count = agents_per_workspace.get(workspace_index).copied().unwrap_or_default();
            let mut keys = Vec::with_capacity(count);
            for agent_index in 0..count {
                let mut agent = template.clone();
                agent.key = AgentKey::new(format!("agent-{workspace_index}-{agent_index}"));
                agent.pane_id = herdr_webmaster::domain::PaneId::new(format!("pane-{workspace_index}-{agent_index}"));
                agent.workspace_id = workspace_id.clone();
                keys.push(agent.key.clone());
                agents.insert(agent.key.clone(), agent);
            }
            sites.entry(workspace_id.clone()).or_insert_with(|| Site {
                workspace_id,
                label: "site".to_owned(),
                cwd: "/tmp".into(),
                agents: Vec::new(),
            }).agents.extend(keys);
        }

        let bays = layout_bays(&sites, &agents, Rect::new(0, 0, 240, 120), None);
        let mut ownership = BTreeMap::<AgentKey, usize>::new();
        for bay in &bays {
            let site = &sites[&bay.workspace_id];
            for (key, _seat) in site.agents.iter().zip(&bay.seats) {
                *ownership.entry(key.clone()).or_default() += 1;
            }
        }
        for key in agents.keys() {
            prop_assert_eq!(ownership.get(key).copied().unwrap_or_default(), 1);
        }
    }

    #[test]
    fn managed_pane_is_absent_from_the_cafe_model_and_rendered_surface(
        managed_pane in support::pane_id(),
    ) {
        let response: herdr_webmaster::herdr::protocol::SuccessResponse<herdr_webmaster::herdr::protocol::SessionSnapshotResult> =
            serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
        let mut snapshot = response.result.snapshot;
        let mut managed = snapshot.agents[0].clone();
        managed.pane_id = managed_pane.to_string();
        managed.name = Some("webmaster-managed-pane".to_owned());
        snapshot.agents.push(managed);
        let state = DomainState::from_snapshot_excluding(
            &snapshot,
            herdr_webmaster::domain::Timestamp::from_millis(1_000),
            Some(&managed_pane),
        );
        prop_assert!(state.agent_key_for_pane(&managed_pane).is_none());
        prop_assert!(state.agents.values().all(|agent| agent.name != "webmaster-managed-pane"));

        let mut model = Model::new(View::Cafe);
        model.replace_domain(state);
        let backend = ratatui::backend::TestBackend::new(120, 30);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        terminal.draw(|frame| herdr_webmaster::ui::render(frame, &model)).unwrap();
        let screen = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>();
        prop_assert!(!screen.contains("webmaster-managed-pane"));
    }

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

            assert_topology_commands(&state, &event, &commands)?;
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

fn assert_topology_commands(
    state: &DomainState,
    event: &AppEvent,
    commands: &[Command],
) -> proptest::test_runner::TestCaseResult {
    match event {
        AppEvent::SnapshotReplaced { .. } => {
            prop_assert_eq!(commands, [Command::PersistState]);
        }
        AppEvent::PaneExited {
            pane_id, revision, ..
        } => {
            let Some(agent_key) = state.agent_key_for_pane(pane_id) else {
                prop_assert_eq!(commands, [Command::RequestSnapshot]);
                return Ok(());
            };
            let agent = &state.agents[agent_key];
            if *revision < agent.pane_revision || agent.presence == Presence::Exited {
                prop_assert!(commands.is_empty());
            } else {
                prop_assert_eq!(commands.len(), 2);
                let Command::AppendGuestbook(entry) = &commands[0] else {
                    prop_assert!(false, "real pane exit must append guestbook history");
                    return Ok(());
                };
                prop_assert_eq!(entry.kind, GuestbookEvent::PaneExited);
                prop_assert_eq!(entry.pane.as_ref(), Some(pane_id));
                prop_assert_eq!(entry.pane_revision, *revision);
                prop_assert_eq!(&commands[1], &Command::PersistState);
            }
        }
        AppEvent::WorkspaceClosed(workspace_id) => {
            if state.sites.contains_key(workspace_id) {
                prop_assert_eq!(commands, [Command::PersistState]);
            } else {
                prop_assert!(commands.is_empty());
            }
        }
        AppEvent::AgentStatusChanged { .. } | AppEvent::MarkSeen(_) => {
            prop_assert!(false, "topology strategy emitted a non-topology event");
        }
    }
    Ok(())
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
