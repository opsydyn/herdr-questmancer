#[allow(dead_code)]
mod support;

use std::collections::BTreeMap;

use proptest::prelude::*;
use questmancer::{
    app::{Model, View},
    domain::{
        AdventurerPersona, AgentKey, Campaign, CampaignStatus, ChronicleEntry, ChronicleEvent,
        DomainState, GuildSummons, PersonaKey, Presence, WorkspaceId,
    },
    ui::cafe_scene::layout_bays,
    update::{AppEvent, Command, update},
};
use ratatui::layout::Rect;

proptest! {
    #[test]
    fn every_generated_agent_is_owned_by_exactly_one_visible_bay(
        workspaces in prop::collection::vec(support::strategies::workspace_id(), 0..=12),
        agents_per_workspace in prop::collection::vec(0usize..=4, 0..=12),
    ) {
        let mut campaigns = BTreeMap::new();
        let mut agents = BTreeMap::new();
        let template = support::fixture_domain().agents.values().next().unwrap().clone();
        for (workspace_index, workspace_id) in workspaces.into_iter().enumerate() {
            let count = agents_per_workspace.get(workspace_index).copied().unwrap_or_default();
            let mut keys = Vec::with_capacity(count);
            for agent_index in 0..count {
                let mut agent = template.clone();
                agent.key = AgentKey::new(format!("agent-{workspace_index}-{agent_index}"));
                agent.pane_id = questmancer::domain::PaneId::new(format!("pane-{workspace_index}-{agent_index}"));
                agent.workspace_id = workspace_id.clone();
                keys.push(agent.key.clone());
                agents.insert(agent.key.clone(), agent);
            }
            campaigns.entry(workspace_id.clone()).or_insert_with(|| Campaign {
                workspace_id,
                label: "site".to_owned(),
                cwd: "/tmp".into(),
                party: Vec::new(),
            }).party.extend(keys);
        }

        for (width, height) in [(240, 120), (80, 24), (60, 18), (1, 1), (0, 0)] {
            let bays = layout_bays(&campaigns, &agents, Rect::new(0, 0, width, height), None);
            let mut ownership = BTreeMap::<AgentKey, usize>::new();
            for bay in &bays {
                for key in &bay.agent_keys {
                    *ownership.entry(key.clone()).or_default() += 1;
                }
            }
            for count in ownership.values() {
                prop_assert_eq!(*count, 1);
            }
            if width == 0 || height == 0 {
                prop_assert!(bays.is_empty());
                prop_assert!(ownership.is_empty());
            }
            if (width, height) == (240, 120) {
                prop_assert_eq!(ownership.len(), agents.len());
            } else {
                prop_assert!(ownership.len() <= agents.len());
            }
        }
    }

    #[test]
    fn managed_pane_is_absent_from_the_cafe_model_and_rendered_surface(
        managed_pane in support::pane_id(),
    ) {
        let response: questmancer::herdr::protocol::SuccessResponse<questmancer::herdr::protocol::SessionSnapshotResult> =
            serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
        let mut snapshot = response.result.snapshot;
        let mut managed = snapshot.agents[0].clone();
        managed.pane_id = managed_pane.to_string();
        managed.name = Some("webmaster-managed-pane".to_owned());
        snapshot.agents.push(managed);
        let state = DomainState::from_snapshot_excluding(
            &snapshot,
            questmancer::domain::Timestamp::from_millis(1_000),
            Some(&managed_pane),
        );
        prop_assert!(state.agent_key_for_pane(&managed_pane).is_none());
        prop_assert!(state.agents.values().all(|agent| agent.name != "webmaster-managed-pane"));

        let mut model = Model::new(View::Delve);
        model.replace_domain(state);
        let backend = ratatui::backend::TestBackend::new(120, 30);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        terminal.draw(|frame| questmancer::ui::render(frame, &model)).unwrap();
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
        let first_persona = AdventurerPersona::for_agent(&agent, workspace_root.as_deref());
        let second_persona = AdventurerPersona::for_agent(&agent, workspace_root.as_deref());

        prop_assert_eq!(&first_key, &second_key);
        prop_assert_eq!(&first_persona, &second_persona);
        prop_assert_eq!(
            first_persona.appearance,
            AdventurerPersona::appearance_for_key(&first_key),
        );
        prop_assert!(!first_persona.name.trim().is_empty());
        prop_assert!(first_persona.name.len() <= 64);
        prop_assert!(!first_persona.epithet.as_str().trim().is_empty());
        prop_assert!(first_persona.epithet.as_str().len() <= 64);

        let has_named_workspace_identity = workspace_root.is_some()
            && (agent.name.is_some() || agent.agent.is_some() || agent.display_agent.is_some());
        if agent.agent_session.is_some() || has_named_workspace_identity {
            let mut moved = agent.clone();
            moved.pane_id.push_str("-moved");
            prop_assert_eq!(
                AdventurerPersona::for_agent(&moved, workspace_root.as_deref()),
                first_persona,
            );
        }
    }

    #[test]
    fn marking_attention_read_is_idempotent(attention in support::attention()) {
        let once = attention.clone().mark_read();
        let twice = once.clone().mark_read();

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
        stale_seed in any::<u64>(),
        status in support::agent_status(),
    ) {
        let current = state.agents.values().next().unwrap().pane_revision;
        let stale_revision = stale_seed % current;
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
    fn campaign_status_priority_is_independent_of_agent_insertion_order(
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
        let forward_campaign = Campaign {
            workspace_id: WorkspaceId::new("site"),
            label: "site".to_owned(),
            cwd: "/tmp/site".into(),
            party: forward_keys,
        };
        let reverse_campaign = Campaign {
            party: reverse_keys,
            ..forward_campaign.clone()
        };
        let expected = expected_campaign_status(&forward_agents);

        prop_assert_eq!(forward_campaign.status(&forward_agents), expected);
        prop_assert_eq!(reverse_campaign.status(&reverse_agents), expected);
    }

    #[test]
    fn chronicle_event_ids_are_stable_for_equal_identity_inputs(
        occurred_at in support::timestamp(),
        pane in prop::option::of(support::pane_id()),
        pane_revision in any::<u64>(),
        event in support::chronicle_event(),
        first_summary in ".{0,32}",
        second_summary in ".{0,32}",
    ) {
        let first = ChronicleEntry::new(
            occurred_at,
            Some("agent-first".into()),
            Some("workspace-first".into()),
            pane.clone(),
            pane_revision,
            event,
            first_summary,
        );
        let second = ChronicleEntry::new(
            occurred_at,
            Some("agent-second".into()),
            Some("workspace-second".into()),
            pane,
            pane_revision,
            event,
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
                let Command::AppendChronicle(entry) = &commands[0] else {
                    prop_assert!(false, "real pane exit must append Chronicle history");
                    return Ok(());
                };
                prop_assert_eq!(entry.event, ChronicleEvent::AdventurerDeparted);
                prop_assert_eq!(entry.pane.as_ref(), Some(pane_id));
                prop_assert_eq!(entry.pane_revision, *revision);
                prop_assert_eq!(&commands[1], &Command::PersistState);
            }
        }
        AppEvent::WorkspaceClosed(workspace_id) => {
            if state.campaigns.contains_key(workspace_id) {
                prop_assert_eq!(commands, [Command::PersistState]);
            } else {
                prop_assert!(commands.is_empty());
            }
        }
        AppEvent::AgentStatusChanged { .. } | AppEvent::MarkRead(_) => {
            prop_assert!(false, "topology strategy emitted a non-topology event");
        }
    }
    Ok(())
}

fn expected_campaign_status(
    agents: &BTreeMap<questmancer::domain::AgentKey, questmancer::domain::Agent>,
) -> CampaignStatus {
    if agents
        .values()
        .any(|agent| agent.presence == Presence::Blocked)
    {
        CampaignStatus::CounselRequired
    } else if agents.values().any(|agent| {
        agent.attention.is_unread()
            && agent.attention.summons() == Some(GuildSummons::SpoilsReturned)
    }) {
        CampaignStatus::SpoilsAwaitingInspection
    } else if agents
        .values()
        .any(|agent| agent.presence == Presence::Working)
    {
        CampaignStatus::ExpeditionActive
    } else if agents
        .values()
        .any(|agent| agent.presence != Presence::Exited)
    {
        CampaignStatus::PartyAtRest
    } else {
        CampaignStatus::Abandoned
    }
}
