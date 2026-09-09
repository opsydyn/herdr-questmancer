use questmancer::{
    app::{Model, View},
    command::{AgentCommand, CommandResult},
    domain::Timestamp,
    herdr::{
        protocol::{
            AgentStatus, SessionSnapshot, SessionSnapshotResult, SuccessResponse, WireEvent,
        },
        supervisor::ConnectionUpdate,
    },
    runtime_loop::{
        RuntimeEffects, apply_command_result, apply_connection_update, request_snapshot_refresh,
    },
    update::Command,
};
use serde_json::json;

fn snapshot(status: AgentStatus, revision: u64) -> SessionSnapshot {
    let response: SuccessResponse<SessionSnapshotResult> =
        serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
    let mut snapshot = response.result.snapshot;
    snapshot.agents[0].agent_status = status;
    snapshot.agents[0].revision = revision;
    snapshot.panes[0].agent_status = status;
    snapshot.panes[0].revision = revision;
    snapshot
}

fn connected(status: AgentStatus) -> Model {
    let mut model = Model::new(View::Guild);
    model
        .domain_mut()
        .capture
        .start(questmancer::domain::CaptureRunId::new("capture-test"));
    apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(snapshot(status, 7)),
        Timestamp::from_millis(1_000),
    );
    model
}

fn refresh(model: &mut Model, snapshot: SessionSnapshot, at: i64) -> RuntimeEffects {
    let effects = request_snapshot_refresh(model);
    let request = effects
        .agent_commands
        .into_iter()
        .find_map(|command| match command {
            AgentCommand::RefreshSnapshot(request) => Some(request),
            _ => None,
        })
        .unwrap();
    apply_command_result(
        model,
        CommandResult::SnapshotLoaded {
            request,
            snapshot: Box::new(snapshot),
        },
        Timestamp::from_millis(at),
    )
}

#[test]
fn qualified_working_to_blocked_refresh_records_one_zero_xp_observation() {
    let mut model = connected(AgentStatus::Working);
    let effects = refresh(&mut model, snapshot(AgentStatus::Blocked, 8), 2_000);
    assert_eq!(
        effects
            .persistence
            .iter()
            .filter(|command| command.is_chronicle_append())
            .count(),
        1
    );
    assert_eq!(model.domain().chronicle.entries().len(), 1);
    let record = model.domain().chronicle.entries().front().unwrap();
    assert_eq!(record.summary, "Codex was observed needing counsel");
    assert_eq!(model.experience(), 0);
    let wire = serde_json::to_value(record).unwrap();
    assert_eq!(wire["record_version"], 2);
    let repeated = refresh(&mut model, snapshot(AgentStatus::Blocked, 9), 3_000);
    assert!(
        !repeated
            .persistence
            .iter()
            .any(Command::is_chronicle_append)
    );
    let matching = apply_connection_update(
        &mut model,
        ConnectionUpdate::Event(WireEvent {
            event: "pane.agent_status_changed".into(),
            data: json!({"pane_id":"w1:p1", "agent_status":"blocked", "revision":10,
            "terminal_id":"terminal-1", "agent_session":{"source":"codex","agent":"codex","kind":"id","value":"session-123"}}),
        }),
        Timestamp::from_millis(4_000),
    );
    assert!(
        !matching
            .persistence
            .iter()
            .any(Command::is_chronicle_append)
    );
    assert_eq!(model.domain().chronicle.entries().len(), 1);
    assert_eq!(model.experience(), 0);
}

fn metadata(model: &mut Model, status: AgentStatus, revision: u64, at: i64) -> RuntimeEffects {
    let status = match status {
        AgentStatus::Working => "working",
        AgentStatus::Blocked => "blocked",
        AgentStatus::Done => "done",
        AgentStatus::Idle => "idle",
        AgentStatus::Unknown => "unknown",
    };
    apply_connection_update(
        model,
        ConnectionUpdate::Event(WireEvent {
            event: "pane.agent_status_changed".into(),
            data: json!({"pane_id":"w1:p1", "agent_status":status, "revision":revision,
            "terminal_id":"terminal-1", "agent_session":{"source":"codex","agent":"codex","kind":"id","value":"session-123"}}),
        }),
        Timestamp::from_millis(at),
    )
}

#[test]
fn reconnects_and_resyncs_remain_quiet_with_capture_enabled() {
    let mut model = connected(AgentStatus::Working);
    for status in [
        AgentStatus::Blocked,
        AgentStatus::Done,
        AgentStatus::Unknown,
    ] {
        apply_connection_update(
            &mut model,
            ConnectionUpdate::Resyncing,
            Timestamp::from_millis(2_000),
        );
        let effects = apply_connection_update(
            &mut model,
            ConnectionUpdate::Connected(snapshot(status, 10)),
            Timestamp::from_millis(3_000),
        );
        assert!(!effects.persistence.iter().any(Command::is_chronicle_append));
        assert!(model.domain().chronicle.entries().is_empty());
        assert_eq!(model.experience(), 0);
    }
}

#[test]
fn snapshot_done_never_gains_a_later_metadata_reward_but_a_new_episode_can() {
    use questmancer::domain::ChronicleEvent;
    let mut model = connected(AgentStatus::Working);
    refresh(&mut model, snapshot(AgentStatus::Done, 8), 2_000);
    metadata(&mut model, AgentStatus::Done, 9, 3_000);
    assert_eq!(model.domain().chronicle.entries().len(), 1);
    assert_eq!(
        model.domain().chronicle.entries()[0].event(),
        ChronicleEvent::PresenceObserved
    );
    assert_eq!(model.experience(), 0);
    metadata(&mut model, AgentStatus::Working, 10, 4_000);
    metadata(&mut model, AgentStatus::Done, 11, 5_000);
    refresh(&mut model, snapshot(AgentStatus::Done, 12), 6_000);
    assert_eq!(model.domain().chronicle.entries().len(), 3);
    assert_eq!(
        model.domain().chronicle.entries()[2].event(),
        ChronicleEvent::SpoilsReturned
    );
    assert_eq!(model.experience(), 10);
    let ordinals = model
        .domain()
        .chronicle
        .entries()
        .iter()
        .map(|entry| entry.observation().unwrap().stamp.ordinal)
        .collect::<Vec<_>>();
    assert_eq!(ordinals, vec![1, 2, 3]);
}

#[test]
fn metadata_first_and_matching_snapshot_record_and_reward_only_once() {
    let mut model = connected(AgentStatus::Working);
    metadata(&mut model, AgentStatus::Done, 8, 2_000);
    refresh(&mut model, snapshot(AgentStatus::Done, 9), 3_000);
    assert_eq!(model.domain().chronicle.entries().len(), 1);
    assert_eq!(model.experience(), 10);
}

#[test]
fn unknown_observations_have_an_explicit_category_for_both_sources() {
    use questmancer::domain::ChronicleEvent;
    for via_snapshot in [true, false] {
        let mut model = connected(AgentStatus::Working);
        if via_snapshot {
            refresh(&mut model, snapshot(AgentStatus::Unknown, 8), 2_000);
        } else {
            metadata(&mut model, AgentStatus::Unknown, 8, 2_000);
        }
        let entry = &model.domain().chronicle.entries()[0];
        assert_eq!(entry.event(), ChronicleEvent::WhereaboutsUnknown);
        assert_ne!(entry.event(), ChronicleEvent::AdventurerJoined);
        assert_eq!(
            entry.summary,
            if via_snapshot {
                "Codex's whereabouts were observed as unknown"
            } else {
                "Codex's whereabouts became unknown"
            }
        );
        assert_eq!(model.experience(), 0);
    }
}

#[test]
fn a_new_identity_first_observed_done_records_only_membership() {
    use questmancer::domain::ChronicleEvent;
    let mut model = connected(AgentStatus::Working);
    let mut baseline = snapshot(AgentStatus::Working, 7);
    baseline.agents.clear();
    apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(baseline),
        Timestamp::from_millis(1_500),
    );
    refresh(&mut model, snapshot(AgentStatus::Done, 8), 2_000);
    let entries = model.domain().chronicle.entries();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].event(), ChronicleEvent::AdventurerObserved);
    assert_eq!(entries[0].summary, "Codex was first observed in the guild");
    metadata(&mut model, AgentStatus::Done, 9, 3_000);
    assert_eq!(model.experience(), 0);
    assert_eq!(model.domain().chronicle.entries().len(), 1);
}

#[test]
fn changed_terminal_incarnation_baselines_the_subject_without_revision_comparison() {
    let mut model = connected(AgentStatus::Working);
    let mut changed = snapshot(AgentStatus::Blocked, 1);
    changed.agents[0].terminal_id = "replacement-terminal".into();
    changed.panes[0].terminal_id = "replacement-terminal".into();
    let effects = refresh(&mut model, changed.clone(), 2_000);
    assert!(!effects.resubscribe);
    assert!(model.domain().chronicle.entries().is_empty());
    assert_eq!(model.selected_agent().unwrap().pane_revision, 1);
    // A high revision from the old terminal cannot act on its replacement.
    let old = metadata(&mut model, AgentStatus::Done, 100, 3_000);
    assert!(!old.persistence.iter().any(Command::is_chronicle_append));
    assert_eq!(model.selected_agent().unwrap().pane_revision, 1);
    assert_eq!(model.experience(), 0);
}

#[test]
fn fallback_identity_changes_are_quiet_and_never_invent_completion() {
    let mut model = connected(AgentStatus::Working);
    let mut fallback = snapshot(AgentStatus::Working, 7);
    fallback.agents[0].agent_session = None;
    apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(fallback.clone()),
        Timestamp::from_millis(1_500),
    );
    fallback.agents[0].agent_status = AgentStatus::Done;
    fallback.agents[0].revision = 8;
    fallback.panes[0].agent_status = AgentStatus::Done;
    fallback.panes[0].revision = 8;
    refresh(&mut model, fallback, 2_000);
    assert!(model.domain().chronicle.entries().is_empty());
    assert_eq!(model.experience(), 0);
}

#[test]
fn visibility_loss_and_reappearance_are_membership_observations_not_departures() {
    use questmancer::domain::ChronicleEvent;
    let mut model = connected(AgentStatus::Working);
    let mut absent = snapshot(AgentStatus::Working, 8);
    absent.agents.clear();
    refresh(&mut model, absent, 2_000);
    refresh(&mut model, snapshot(AgentStatus::Done, 9), 3_000);
    assert_eq!(
        model
            .domain()
            .chronicle
            .entries()
            .iter()
            .map(questmancer::domain::ChronicleEntry::event)
            .collect::<Vec<_>>(),
        vec![
            ChronicleEvent::AdventurerNoLongerVisible,
            ChronicleEvent::AdventurerObserved
        ]
    );
    assert_eq!(model.experience(), 0);
}

#[test]
fn timestamp_ties_and_clock_rollback_cannot_collapse_accepted_observations() {
    use std::collections::BTreeSet;
    let mut model = connected(AgentStatus::Working);
    for (status, revision, at) in [
        (AgentStatus::Blocked, 8, 2_000),
        (AgentStatus::Working, 9, 2_000),
        (AgentStatus::Blocked, 10, 1_000),
    ] {
        refresh(&mut model, snapshot(status, revision), at);
    }
    let entries = model.domain().chronicle.entries();
    assert_eq!(entries.len(), 3);
    assert_eq!(
        entries
            .iter()
            .map(|entry| &entry.id)
            .collect::<BTreeSet<_>>()
            .len(),
        3
    );
    assert_eq!(
        entries
            .iter()
            .map(|entry| entry.observation().unwrap().stamp.ordinal)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([1, 2, 3])
    );
}

#[test]
fn eviction_does_not_reset_current_fact_duplicate_guards() {
    let mut model = connected(AgentStatus::Working);
    for revision in 8..=510 {
        let status = if revision % 2 == 0 {
            AgentStatus::Blocked
        } else {
            AgentStatus::Working
        };
        refresh(
            &mut model,
            snapshot(status, revision),
            i64::try_from(revision).unwrap() * 1000,
        );
    }
    assert_eq!(model.domain().chronicle.entries().len(), 500);
    let last = model.domain().chronicle.entries().back().unwrap().clone();
    metadata(&mut model, AgentStatus::Blocked, 511, 600_000);
    assert_eq!(model.domain().chronicle.entries().len(), 500);
    assert_eq!(model.domain().chronicle.entries().back(), Some(&last));
    assert_eq!(model.experience(), 0);
}

#[test]
fn temporarily_missing_session_evidence_does_not_invent_visibility_loss() {
    let mut model = connected(AgentStatus::Working);
    let mut ambiguous = snapshot(AgentStatus::Done, 8);
    ambiguous.agents[0].agent_session = None;
    refresh(&mut model, ambiguous, 2_000);
    assert!(
        model.domain().chronicle.entries().is_empty(),
        "an unidentified occupant may still be the same adventurer"
    );
}

fn with_spare_pane(mut snapshot: SessionSnapshot) -> SessionSnapshot {
    let mut spare = snapshot.panes[0].clone();
    spare.pane_id = "w1:p2".into();
    spare.terminal_id = "spare-terminal".into();
    spare.agent_session = None;
    snapshot.panes.push(spare);
    snapshot
}

#[test]
fn moving_the_same_terminal_cannot_reset_its_revision_boundary() {
    let mut model = connected(AgentStatus::Working);
    apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(with_spare_pane(snapshot(AgentStatus::Working, 7))),
        Timestamp::from_millis(1_500),
    );
    let before = model.domain().clone();
    let mut moved = with_spare_pane(snapshot(AgentStatus::Done, 6));
    moved.agents[0].pane_id = "w1:p2".into();
    let effects = refresh(&mut model, moved, 2_000);
    assert!(
        effects
            .agent_commands
            .iter()
            .any(|command| matches!(command, AgentCommand::RefreshSnapshot(_)))
    );
    assert_eq!(model.domain(), &before);
}

#[test]
fn a_same_session_pane_move_preserves_continuity_without_membership_noise() {
    use questmancer::domain::ChronicleEvent;
    let mut model = connected(AgentStatus::Working);
    apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(with_spare_pane(snapshot(AgentStatus::Working, 7))),
        Timestamp::from_millis(1_500),
    );
    let original_key = model.selected_agent_key().cloned();
    let mut moved = with_spare_pane(snapshot(AgentStatus::Blocked, 8));
    moved.agents[0].pane_id = "w1:p2".into();
    refresh(&mut model, moved, 2_000);
    assert_eq!(model.selected_agent_key(), original_key.as_ref());
    assert_eq!(model.domain().chronicle.entries().len(), 1);
    assert_eq!(
        model.domain().chronicle.entries()[0].event(),
        ChronicleEvent::PresenceObserved
    );
}

fn finish_request(
    model: &mut Model,
    effects: RuntimeEffects,
    snapshot: SessionSnapshot,
    at: i64,
) -> RuntimeEffects {
    let request = effects
        .agent_commands
        .into_iter()
        .find_map(|command| match command {
            AgentCommand::RefreshSnapshot(request) => Some(request),
            _ => None,
        })
        .unwrap();
    apply_command_result(
        model,
        CommandResult::SnapshotLoaded {
            request,
            snapshot: Box::new(snapshot),
        },
        Timestamp::from_millis(at),
    )
}

fn close_hint(model: &mut Model, workspace: &str) -> RuntimeEffects {
    apply_connection_update(
        model,
        ConnectionUpdate::Event(WireEvent {
            event: "workspace_closed".into(),
            data: json!({"workspace_id":workspace}),
        }),
        Timestamp::from_millis(2_000),
    )
}

fn with_empty_campaign(mut snapshot: SessionSnapshot, id: &str) -> SessionSnapshot {
    let mut campaign = snapshot.workspaces[0].clone();
    campaign.workspace_id = id.into();
    campaign.label = id.into();
    campaign.pane_count = 0;
    campaign.tab_count = 0;
    snapshot.workspaces.push(campaign);
    snapshot
}

#[test]
fn campaign_close_hints_do_not_remove_live_parties_and_expire_after_confirmation() {
    let mut model = connected(AgentStatus::Working);
    let baseline = with_empty_campaign(snapshot(AgentStatus::Working, 7), "empty-campaign");
    apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(baseline.clone()),
        Timestamp::from_millis(1_500),
    );
    let requested = close_hint(&mut model, "empty-campaign");
    assert_eq!(model.domain().campaigns.len(), 2);
    assert!(model.domain().chronicle.entries().is_empty());
    finish_request(&mut model, requested, baseline, 3_000);
    assert_eq!(model.domain().campaigns.len(), 2);
    assert!(model.domain().chronicle.entries().is_empty());
    refresh(&mut model, snapshot(AgentStatus::Working, 8), 4_000);
    assert_eq!(model.domain().chronicle.entries().len(), 1);
    assert_eq!(
        model.domain().chronicle.entries()[0].summary,
        "Campaign empty-campaign was no longer visible"
    );
    assert_eq!(model.experience(), 0);
}

#[test]
fn a_confirmed_campaign_removal_absorbs_its_membership_losses_without_awards() {
    use questmancer::domain::ChronicleEvent;
    let mut model = connected(AgentStatus::Working);
    let mut baseline = with_empty_campaign(
        with_spare_pane(snapshot(AgentStatus::Working, 7)),
        "remaining",
    );
    let mut second = baseline.agents[0].clone();
    second.pane_id = "w1:p2".into();
    second.terminal_id = "spare-terminal".into();
    second.agent_session.as_mut().unwrap().value = "second-session".into();
    baseline.agents.push(second);
    apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(baseline.clone()),
        Timestamp::from_millis(1_500),
    );
    let requested = close_hint(&mut model, "w1");
    assert_eq!(model.domain().agents.len(), 2);
    baseline
        .workspaces
        .retain(|campaign| campaign.workspace_id == "remaining");
    baseline.agents.clear();
    // Subscription membership remains valid: the now-plain panes moved before
    // the old campaign's absence was observed. Pane removal would resubscribe.
    for pane in &mut baseline.panes {
        pane.workspace_id = "remaining".into();
    }
    finish_request(&mut model, requested, baseline, 3_000);
    assert_eq!(model.domain().chronicle.entries().len(), 1);
    let entry = &model.domain().chronicle.entries()[0];
    assert_eq!(entry.event(), ChronicleEvent::CampaignRemoved);
    assert_eq!(entry.summary, "Campaign webmaster closed");
    assert_eq!(entry.adventurer(), None);
    assert_eq!(model.experience(), 0);
}

#[test]
fn two_campaigns_removed_at_one_time_have_distinct_ids_and_stable_ordinals() {
    let mut model = connected(AgentStatus::Working);
    let baseline = with_empty_campaign(
        with_empty_campaign(snapshot(AgentStatus::Working, 7), "b-campaign"),
        "a-campaign",
    );
    apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(baseline),
        Timestamp::from_millis(1_500),
    );
    refresh(&mut model, snapshot(AgentStatus::Working, 8), 2_000);
    let entries = model.domain().chronicle.entries();
    assert_eq!(entries.len(), 2);
    assert_ne!(entries[0].id, entries[1].id);
    assert_eq!(entries[0].campaign().unwrap().as_str(), "a-campaign");
    assert_eq!(entries[1].campaign().unwrap().as_str(), "b-campaign");
    assert_eq!(entries[0].observation().unwrap().stamp.ordinal, 1);
    assert_eq!(entries[1].observation().unwrap().stamp.ordinal, 2);
    assert_eq!(model.experience(), 0);
}

#[test]
fn unversioned_exit_hints_cannot_claim_a_departure() {
    let mut model = connected(AgentStatus::Working);
    let effects = apply_connection_update(
        &mut model,
        ConnectionUpdate::Event(WireEvent {
            event: "pane.exited".into(),
            data: json!({"pane_id":"w1:p1","exit_code":0}),
        }),
        Timestamp::from_millis(2_000),
    );
    assert!(
        effects
            .agent_commands
            .iter()
            .any(|command| matches!(command, AgentCommand::RefreshSnapshot(_)))
    );
    assert!(model.domain().chronicle.entries().is_empty());
    assert_eq!(
        model.selected_agent().unwrap().presence,
        questmancer::domain::Presence::Working
    );
    let mut absent = snapshot(AgentStatus::Idle, 8);
    absent.agents.clear();
    finish_request(&mut model, effects, absent, 3_000);
    assert_eq!(
        model.domain().chronicle.entries()[0].event(),
        questmancer::domain::ChronicleEvent::AdventurerNoLongerVisible
    );
}

#[test]
fn corroborated_same_incarnation_departures_are_zero_xp_and_not_repeated_by_absence() {
    let mut model = connected(AgentStatus::Working);
    let effects = apply_connection_update(
        &mut model,
        ConnectionUpdate::Event(WireEvent {
            event: "pane.exited".into(),
            data: json!({"pane_id":"w1:p1", "revision":8, "terminal_id":"terminal-1", "agent_session":{"source":"codex","agent":"codex","kind":"id","value":"session-123"}}),
        }),
        Timestamp::from_millis(2_000),
    );
    assert_eq!(
        effects
            .persistence
            .iter()
            .filter(|command| command.is_chronicle_append())
            .count(),
        1
    );
    let mut absent = snapshot(AgentStatus::Idle, 9);
    absent.agents.clear();
    refresh(&mut model, absent, 3_000);
    assert_eq!(model.domain().chronicle.entries().len(), 1);
    let entry = &model.domain().chronicle.entries()[0];
    assert_eq!(
        entry.event(),
        questmancer::domain::ChronicleEvent::AdventurerDeparted
    );
    assert!(matches!(
        entry.observation().unwrap().evidence,
        questmancer::domain::ObservationEvidence::LifecycleDeparture
    ));
    assert_eq!(model.experience(), 0);
}

#[test]
fn new_record_ids_ignore_display_names_and_clock_but_include_campaign_identity() {
    use questmancer::domain::ObservationSubject;
    let mut model = connected(AgentStatus::Working);
    let baseline = with_empty_campaign(snapshot(AgentStatus::Working, 7), "removed");
    apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(baseline),
        Timestamp::from_millis(1_500),
    );
    refresh(&mut model, snapshot(AgentStatus::Working, 8), 2_000);
    let record = model.domain().chronicle.entries()[0].clone();
    let mut same = record.observation().unwrap().clone();
    let ObservationSubject::Campaign { name, .. } = &mut same.subject else {
        panic!("campaign subject")
    };
    *name = "renamed later".into();
    assert_eq!(same.event_id(), record.id);
    let ObservationSubject::Campaign { id, .. } = &mut same.subject else {
        unreachable!()
    };
    *id = questmancer::domain::WorkspaceId::new("another-campaign");
    assert_ne!(same.event_id(), record.id);
    let wire = serde_json::to_string(&record).unwrap();
    assert!(!wire.contains("session-123"));
    assert!(!wire.contains("terminal-1"));
}

#[test]
fn a_quiet_terminal_replacement_cannot_trigger_the_same_party_cat_reaction() {
    let mut model = connected(AgentStatus::Working);
    let mut replacement = snapshot(AgentStatus::Idle, 8);
    replacement.agents[0].terminal_id = "new-terminal".into();
    replacement.panes[0].terminal_id = "new-terminal".into();
    refresh(&mut model, replacement, 2_000);
    assert!(model.domain().chronicle.entries().is_empty());
    assert_eq!(model.party_rest_since(), None);
}

#[test]
fn a_new_terminal_invalidates_scrying_even_when_its_key_pane_and_revision_match() {
    use questmancer::{domain::PaneId, interaction::reduce_action, ui::input::Action};
    let mut model = connected(AgentStatus::Working);
    let old_request = reduce_action(&mut model, Action::Refresh)
        .commands
        .into_iter()
        .find_map(|command| match command {
            AgentCommand::LoadOutput { request, .. } => Some(request),
            _ => None,
        })
        .unwrap();
    apply_command_result(
        &mut model,
        CommandResult::OutputLoaded {
            request: old_request,
            pane_id: PaneId::new("w1:p1"),
            revision: 50,
            text: "old terminal output".into(),
            truncated: false,
        },
        Timestamp::from_millis(1_500),
    );
    let mut replacement = snapshot(AgentStatus::Working, 7);
    replacement.agents[0].terminal_id = "new-terminal".into();
    replacement.panes[0].terminal_id = "new-terminal".into();
    let effects = refresh(&mut model, replacement, 2_000);
    let new_request = effects
        .agent_commands
        .into_iter()
        .find_map(|command| match command {
            AgentCommand::LoadOutput { request, .. } => Some(request),
            _ => None,
        })
        .expect("changed incarnation needs its own output read");
    assert!(model.output_preview().unwrap().text.is_empty());
    apply_command_result(
        &mut model,
        CommandResult::OutputLoaded {
            request: old_request,
            pane_id: PaneId::new("w1:p1"),
            revision: 100,
            text: "late old output".into(),
            truncated: false,
        },
        Timestamp::from_millis(2_500),
    );
    assert!(model.output_preview().unwrap().text.is_empty());
    apply_command_result(
        &mut model,
        CommandResult::OutputLoaded {
            request: new_request,
            pane_id: PaneId::new("w1:p1"),
            revision: 7,
            text: "new terminal output".into(),
            truncated: false,
        },
        Timestamp::from_millis(3_000),
    );
    assert_eq!(model.output_preview().unwrap().text, "new terminal output");
}

#[test]
fn chapter_ties_follow_accepted_observation_order_instead_of_hash_order() {
    use questmancer::chronicle_chapter::{ChapterRequest, ChapterWindow};
    let mut model = connected(AgentStatus::Working);
    for revision in 8..=12 {
        let status = if revision % 2 == 0 {
            AgentStatus::Blocked
        } else {
            AgentStatus::Working
        };
        refresh(&mut model, snapshot(status, revision), 2_000);
    }
    let request = ChapterRequest {
        window: ChapterWindow::last_hour(Timestamp::from_millis(3_000)),
        adventurer: None,
    };
    let chapter = request.project(&model.domain().chronicle);
    assert_eq!(
        chapter
            .sources()
            .iter()
            .map(|entry| entry.observation().unwrap().stamp.ordinal)
            .collect::<Vec<_>>(),
        vec![5, 4, 3, 2, 1]
    );
}

#[test]
fn chapters_separate_observed_spoils_unknown_whereabouts_and_legacy_identity_events() {
    use questmancer::{
        chronicle_chapter::{ChapterRequest, ChapterWindow},
        domain::{ChronicleEntry, ChronicleEvent},
    };
    let mut model = connected(AgentStatus::Working);
    refresh(&mut model, snapshot(AgentStatus::Done, 8), 2_000);
    metadata(&mut model, AgentStatus::Unknown, 9, 3_000);
    model.domain_mut().chronicle.append(ChronicleEntry::new(
        Timestamp::from_millis(1_000),
        None,
        None,
        None,
        0,
        ChronicleEvent::AdventurerJoined,
        "Legacy whereabouts unknown",
    ));
    let request = ChapterRequest {
        window: ChapterWindow::last_hour(Timestamp::from_millis(4_000)),
        adventurer: None,
    };
    let chapter = request.project(&model.domain().chronicle);
    assert_eq!(chapter.count(ChronicleEvent::PresenceObserved), 1);
    assert_eq!(chapter.count(ChronicleEvent::WhereaboutsUnknown), 1);
    assert_eq!(chapter.count(ChronicleEvent::AdventurerJoined), 1);
    assert_eq!(chapter.count(ChronicleEvent::SpoilsReturned), 0);
    let text = chapter.lines().join("\n");
    assert!(text.contains("1 presence-observation event recorded"));
    assert!(text.contains("1 unknown-whereabouts event recorded"));
    assert!(text.contains("1 identity event recorded"));
    assert!(text.contains("Retained records only; missing history is not inferred."));
    assert!(text.contains("Observed locally: snapshot observation"));
    assert!(text.contains("Observed locally: pane metadata observation"));
    assert!(text.contains("Codex was observed with spoils reported"));
    for source in chapter.sources() {
        assert!(text.replace('\n', "").contains(source.id.as_str()));
    }
}

#[tokio::test]
async fn restart_replays_v2_without_awarding_again_and_starts_a_new_capture_run() {
    use questmancer::{
        config::PersistencePaths,
        persistence::{PersistedStateV1, load_startup},
        runtime_loop::bootstrap_model,
    };
    let mut original = connected(AgentStatus::Working);
    metadata(&mut original, AgentStatus::Done, 8, 2_000);
    assert_eq!(original.experience(), 10);
    let original_record = original.domain().chronicle.entries()[0].clone();
    let directory = tempfile::tempdir().unwrap();
    let state = serde_json::to_vec(&PersistedStateV1::capture(&original)).unwrap();
    let state_text = String::from_utf8(state.clone()).unwrap();
    for ephemeral in [
        "capture",
        "terminal-1",
        "session-123",
        "ordinal",
        "observation",
    ] {
        assert!(!state_text.contains(ephemeral));
    }
    tokio::fs::write(directory.path().join("state.json"), state)
        .await
        .unwrap();
    let mut history = serde_json::to_vec(&original_record).unwrap();
    history.push(b'\n');
    tokio::fs::write(directory.path().join("chronicle.jsonl"), &history)
        .await
        .unwrap();
    let paths = PersistencePaths::from_lookup(|name| {
        (name == "HERDR_PLUGIN_STATE_DIR").then(|| directory.path().display().to_string())
    });
    let startup = load_startup(paths, None).await;
    assert!(startup.diagnostics.is_empty());
    let mut restored = bootstrap_model(startup.model, None);
    assert_eq!(restored.experience(), 10);
    assert_eq!(restored.domain().chronicle.entries()[0], original_record);
    let baseline = apply_connection_update(
        &mut restored,
        ConnectionUpdate::Connected(snapshot(AgentStatus::Done, 8)),
        Timestamp::from_millis(3_000),
    );
    assert!(
        !baseline
            .persistence
            .iter()
            .any(Command::is_chronicle_append)
    );
    metadata(&mut restored, AgentStatus::Done, 9, 4_000);
    assert_eq!(restored.experience(), 10);
    assert_eq!(restored.domain().chronicle.entries().len(), 1);
    metadata(&mut restored, AgentStatus::Working, 10, 5_000);
    let new_record = restored.domain().chronicle.entries().back().unwrap();
    assert_eq!(new_record.observation().unwrap().stamp.ordinal, 1);
    assert_ne!(
        new_record.observation().unwrap().stamp.run,
        original_record.observation().unwrap().stamp.run
    );
    assert_ne!(new_record.id, original_record.id);
    assert_eq!(
        tokio::fs::read(directory.path().join("chronicle.jsonl"))
            .await
            .unwrap(),
        history,
        "reading and reducing never rewrite history without dispatch"
    );
}

#[test]
fn the_managed_pane_and_render_clock_never_create_observations_or_commands() {
    use questmancer::domain::PaneId;
    let mut model = connected(AgentStatus::Working);
    model.set_managed_pane_id(Some(PaneId::new("w1:p1")));
    apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(snapshot(AgentStatus::Working, 7)),
        Timestamp::from_millis(1_500),
    );
    let effects = refresh(&mut model, snapshot(AgentStatus::Done, 8), 2_000);
    assert!(effects.agent_commands.is_empty());
    assert!(!effects.persistence.iter().any(Command::is_chronicle_append));
    assert!(model.domain().agents.is_empty());
    for at in 2_000..2_100 {
        model.set_now(Timestamp::from_millis(at));
    }
    assert!(model.domain().chronicle.entries().is_empty());
    assert_eq!(model.experience(), 0);
}

#[test]
fn obsolete_snapshot_results_remain_quiet_with_capture_enabled() {
    let mut model = connected(AgentStatus::Working);
    let requested = request_snapshot_refresh(&mut model);
    apply_connection_update(
        &mut model,
        ConnectionUpdate::Disconnected("test".into()),
        Timestamp::from_millis(2_000),
    );
    apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(snapshot(AgentStatus::Working, 20)),
        Timestamp::from_millis(3_000),
    );
    let before = model.domain().clone();
    let old = finish_request(&mut model, requested, snapshot(AgentStatus::Done, 8), 4_000);
    assert_eq!(old, RuntimeEffects::default());
    assert_eq!(model.domain(), &before);
    assert_eq!(model.experience(), 0);
}

proptest::proptest! {
    #[test]
    fn arbitrary_snapshot_sequences_capture_only_real_differences_without_rewards(
        reports in proptest::collection::vec((0_u8..5, proptest::prelude::any::<i64>()), 0..32)
    ) {
        use std::collections::BTreeSet;
        use proptest::prelude::*;
        let mut model = connected(AgentStatus::Working);
        let mut prior = AgentStatus::Working;
        let mut expected = 0;
        for (index, (status, at)) in reports.into_iter().enumerate() {
            let status = [AgentStatus::Working, AgentStatus::Blocked, AgentStatus::Done, AgentStatus::Idle, AgentStatus::Unknown][usize::from(status)];
            refresh(&mut model, snapshot(status, u64::try_from(index).unwrap() + 8), at);
            if status != prior { expected += 1; }
            prior = status;
        }
        let entries = model.domain().chronicle.entries();
        prop_assert_eq!(entries.len(), expected);
        prop_assert_eq!(model.experience(), 0);
        prop_assert_eq!(entries.iter().map(|entry| &entry.id).collect::<BTreeSet<_>>().len(), expected);
        let ordinals = entries.iter().map(|entry| entry.observation().unwrap().stamp.ordinal).collect::<BTreeSet<_>>();
        prop_assert_eq!(ordinals, (1..=u64::try_from(expected).unwrap()).collect::<BTreeSet<_>>());
        let mut bytes = Vec::new();
        for entry in entries { bytes.extend(serde_json::to_vec(entry).unwrap()); bytes.push(b'\n'); }
        let replay = questmancer::persistence::replay_chronicle(std::path::Path::new("property.jsonl"), &bytes, 500);
        prop_assert!(replay.diagnostics.is_empty());
        prop_assert_eq!(&replay.chronicle, &model.domain().chronicle);
    }
}

#[test]
fn an_ambiguous_baseline_suppresses_subject_history_until_identity_is_reestablished() {
    let mut model = connected(AgentStatus::Working);
    let mut ambiguous = snapshot(AgentStatus::Working, 7);
    ambiguous.agents.push(ambiguous.agents[0].clone());
    apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(ambiguous),
        Timestamp::from_millis(1_500),
    );
    assert!(
        !model
            .selected_agent()
            .unwrap()
            .capture_identity
            .is_qualified()
    );
    assert!(
        model
            .integration_diagnostic()
            .unwrap()
            .contains("without unambiguous session identity")
    );
    refresh(&mut model, snapshot(AgentStatus::Done, 8), 2_000);
    assert!(
        model
            .selected_agent()
            .unwrap()
            .capture_identity
            .is_qualified()
    );
    assert!(model.domain().chronicle.entries().is_empty());
    refresh(&mut model, snapshot(AgentStatus::Working, 9), 3_000);
    assert_eq!(model.domain().chronicle.entries().len(), 1);
    assert_eq!(
        model.domain().chronicle.entries()[0]
            .observation()
            .unwrap()
            .stamp
            .ordinal,
        1
    );
    assert_eq!(model.experience(), 0);
}
