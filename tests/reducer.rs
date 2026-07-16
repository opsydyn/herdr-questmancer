use questmancer::{
    domain::{
        ChronicleEvent, DomainState, GuildAttention, GuildSummons, PaneId, Presence, Timestamp,
        WorkspaceId,
    },
    herdr::protocol::{AgentStatus, SessionSnapshot, SessionSnapshotResult, SuccessResponse},
    update::{AppEvent, Command, update},
};

fn snapshot() -> SessionSnapshot {
    let response: SuccessResponse<SessionSnapshotResult> =
        serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
    response.result.snapshot
}

fn state() -> DomainState {
    DomainState::from_snapshot(&snapshot(), Timestamp::from_millis(1_000))
}

fn status(status: AgentStatus, revision: u64, at: i64) -> AppEvent {
    AppEvent::AgentStatusChanged {
        pane_id: PaneId::new("w1:p1"),
        status,
        custom_status: None,
        revision,
        occurred_at: Timestamp::from_millis(at),
    }
}

fn transition_to_status(target_status: AgentStatus) -> DomainState {
    let (initial, revision) = if target_status == AgentStatus::Blocked {
        (update(state(), status(AgentStatus::Working, 8, 2_000)).0, 9)
    } else {
        (state(), 8)
    };

    update(initial, status(target_status, revision, 3_000)).0
}

fn assert_latest_chronicle_entry(
    state: &DomainState,
    expected_event: ChronicleEvent,
    expected_summary: &str,
) {
    let entry = state.chronicle.entries().back().unwrap();
    assert_eq!(entry.event, expected_event);
    assert_eq!(entry.summary, expected_summary);
}

#[test]
fn working_status_records_delve_began_summary() {
    assert_latest_chronicle_entry(
        &transition_to_status(AgentStatus::Working),
        ChronicleEvent::DelveBegan,
        "Codex began a delve",
    );
}

#[test]
fn blocked_status_records_counsel_requested_summary() {
    assert_latest_chronicle_entry(
        &transition_to_status(AgentStatus::Blocked),
        ChronicleEvent::CounselRequested,
        "Codex requested counsel",
    );
}

#[test]
fn done_status_records_spoils_returned_summary() {
    assert_latest_chronicle_entry(
        &transition_to_status(AgentStatus::Done),
        ChronicleEvent::SpoilsReturned,
        "Codex returned with spoils",
    );
}

#[test]
fn idle_status_records_adventurer_rested_summary() {
    assert_latest_chronicle_entry(
        &transition_to_status(AgentStatus::Idle),
        ChronicleEvent::AdventurerRested,
        "Codex made camp",
    );
}

#[test]
fn unknown_status_records_adventurer_joined_summary() {
    assert_latest_chronicle_entry(
        &transition_to_status(AgentStatus::Unknown),
        ChronicleEvent::AdventurerJoined,
        "Codex whereabouts unknown",
    );
}

#[test]
fn working_to_blocked_creates_unread_counsel_summons() {
    let (working, _) = update(state(), status(AgentStatus::Working, 8, 2_000));

    let (blocked, commands) = update(working, status(AgentStatus::Blocked, 9, 3_000));
    let agent = blocked.agents.values().next().unwrap();

    assert_eq!(agent.presence, Presence::Blocked);
    assert_eq!(
        agent.attention,
        GuildAttention::unread(
            GuildSummons::CounselRequested,
            Timestamp::from_millis(3_000),
        )
    );
    assert!(commands.iter().any(Command::is_chronicle_append));
    assert_eq!(blocked.chronicle.entries().len(), 2);
    assert_eq!(
        blocked.chronicle.entries().back().unwrap().event,
        ChronicleEvent::CounselRequested
    );
}

#[test]
fn blocked_to_done_replaces_attention_with_completion() {
    let (done, _) = update(state(), status(AgentStatus::Done, 8, 2_000));
    let agent = done.agents.values().next().unwrap();

    assert_eq!(agent.presence, Presence::Done);
    assert_eq!(
        agent.attention.summons(),
        Some(GuildSummons::SpoilsReturned)
    );
    assert!(agent.attention.is_unread());
}

#[test]
fn blocked_to_idle_clears_attention() {
    let (idle, _) = update(state(), status(AgentStatus::Idle, 8, 2_000));
    let agent = idle.agents.values().next().unwrap();

    assert_eq!(agent.presence, Presence::Idle);
    assert_eq!(agent.attention, GuildAttention::Clear);
}

#[test]
fn marking_done_attention_seen_is_local_and_persisted() {
    let (done, _) = update(state(), status(AgentStatus::Done, 8, 2_000));
    let key = done.agents.keys().next().unwrap().clone();

    let (seen, commands) = update(done, AppEvent::MarkRead(key.clone()));

    assert!(matches!(
        seen.agents[&key].attention,
        GuildAttention::Read {
            summons: GuildSummons::SpoilsReturned,
            ..
        }
    ));
    assert_eq!(commands, vec![Command::PersistState]);
}

#[test]
fn pane_exit_becomes_attention_and_history() {
    let (exited, commands) = update(
        state(),
        AppEvent::PaneExited {
            pane_id: PaneId::new("w1:p1"),
            revision: 8,
            occurred_at: Timestamp::from_millis(2_000),
        },
    );
    let agent = exited.agents.values().next().unwrap();

    assert_eq!(agent.presence, Presence::Exited);
    assert_eq!(
        agent.attention.summons(),
        Some(GuildSummons::AdventurerDeparted)
    );
    assert!(commands.iter().any(Command::is_chronicle_append));
    assert_latest_chronicle_entry(
        &exited,
        ChronicleEvent::AdventurerDeparted,
        "Codex departed the guild",
    );
}

#[test]
fn workspace_close_removes_its_campaign_and_agents() {
    let (closed, commands) = update(state(), AppEvent::WorkspaceClosed(WorkspaceId::new("w1")));

    assert!(closed.campaigns.is_empty());
    assert!(closed.agents.is_empty());
    assert_eq!(commands, vec![Command::PersistState]);
}

#[test]
fn duplicate_or_stale_status_is_ignored() {
    let initial = state();
    let (duplicate, commands) = update(initial.clone(), status(AgentStatus::Blocked, 7, 2_000));
    assert_eq!(duplicate, initial);
    assert!(commands.is_empty());

    let (stale, commands) = update(initial.clone(), status(AgentStatus::Done, 6, 3_000));
    assert_eq!(stale, initial);
    assert!(commands.is_empty());
}

#[test]
fn unknown_pane_requests_a_fresh_snapshot() {
    let event = AppEvent::AgentStatusChanged {
        pane_id: PaneId::new("w9:p9"),
        status: AgentStatus::Blocked,
        custom_status: None,
        revision: 1,
        occurred_at: Timestamp::from_millis(2_000),
    };

    let (unchanged, commands) = update(state(), event);

    assert_eq!(commands, vec![Command::RequestSnapshot]);
    assert_eq!(unchanged.agents.len(), 1);
}

#[test]
fn snapshot_replacement_preserves_seen_attention_and_persona() {
    let initial = state();
    let key = initial.agents.keys().next().unwrap().clone();
    let persona = initial.agents[&key].persona.clone();
    let (seen, _) = update(initial, AppEvent::MarkRead(key.clone()));
    let mut replacement = snapshot();
    replacement.agents[0].pane_id = "w1:p9".to_owned();

    let (replaced, commands) = update(
        seen,
        AppEvent::SnapshotReplaced {
            snapshot: replacement,
            observed_at: Timestamp::from_millis(5_000),
            excluded_pane: None,
        },
    );

    assert!(matches!(
        replaced.agents[&key].attention,
        GuildAttention::Read { .. }
    ));
    assert_eq!(replaced.agents[&key].persona, persona);
    assert_eq!(replaced.agents[&key].pane_id, PaneId::new("w1:p9"));
    assert_eq!(commands, vec![Command::PersistState]);
}

#[test]
fn snapshot_replacement_excludes_managed_pane_and_preserves_selection() {
    let initial = state();
    let key = initial.agents.keys().next().unwrap().clone();
    let mut replacement = snapshot();
    replacement.agents.push({
        let mut managed = replacement.agents[0].clone();
        managed.pane_id = "w2:p3".to_owned();
        managed.workspace_id = "w2".to_owned();
        managed
    });

    let (replaced, _) = update(
        initial,
        AppEvent::SnapshotReplaced {
            snapshot: replacement,
            observed_at: Timestamp::from_millis(5_000),
            excluded_pane: Some(PaneId::new("w2:p3")),
        },
    );

    assert_eq!(replaced.selected_agent, Some(key));
    assert!(replaced.agent_key_for_pane(&PaneId::new("w2:p3")).is_none());
}
