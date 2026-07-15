use questmancer::{
    domain::{Attention, AttentionReason, DomainState, PaneId, Presence, Timestamp, WorkspaceId},
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

#[test]
fn working_to_blocked_creates_unseen_webmaster_attention() {
    let (working, _) = update(state(), status(AgentStatus::Working, 8, 2_000));

    let (blocked, commands) = update(working, status(AgentStatus::Blocked, 9, 3_000));
    let agent = blocked.agents.values().next().unwrap();

    assert_eq!(agent.presence, Presence::Blocked);
    assert_eq!(
        agent.attention,
        Attention::unseen(AttentionReason::NeedsInput, Timestamp::from_millis(3_000))
    );
    assert!(commands.iter().any(Command::is_guestbook_append));
    assert_eq!(blocked.guestbook.entries().len(), 2);
}

#[test]
fn blocked_to_done_replaces_attention_with_completion() {
    let (done, _) = update(state(), status(AgentStatus::Done, 8, 2_000));
    let agent = done.agents.values().next().unwrap();

    assert_eq!(agent.presence, Presence::Done);
    assert_eq!(
        agent.attention.reason(),
        Some(AttentionReason::WorkCompleted)
    );
    assert!(agent.attention.is_unseen());
}

#[test]
fn blocked_to_idle_clears_attention() {
    let (idle, _) = update(state(), status(AgentStatus::Idle, 8, 2_000));
    let agent = idle.agents.values().next().unwrap();

    assert_eq!(agent.presence, Presence::Idle);
    assert_eq!(agent.attention, Attention::Clear);
}

#[test]
fn marking_done_attention_seen_is_local_and_persisted() {
    let (done, _) = update(state(), status(AgentStatus::Done, 8, 2_000));
    let key = done.agents.keys().next().unwrap().clone();

    let (seen, commands) = update(done, AppEvent::MarkSeen(key.clone()));

    assert!(matches!(
        seen.agents[&key].attention,
        Attention::Seen {
            reason: AttentionReason::WorkCompleted,
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
    assert_eq!(agent.attention.reason(), Some(AttentionReason::PaneExited));
    assert!(commands.iter().any(Command::is_guestbook_append));
}

#[test]
fn workspace_close_removes_its_site_and_agents() {
    let (closed, commands) = update(state(), AppEvent::WorkspaceClosed(WorkspaceId::new("w1")));

    assert!(closed.sites.is_empty());
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
    let (seen, _) = update(initial, AppEvent::MarkSeen(key.clone()));
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
        Attention::Seen { .. }
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
