use herdr_webmaster::{
    domain::{
        Attention, AttentionReason, DomainState, PaneId, Presence, SiteStatus, Timestamp,
        WorkspaceId,
    },
    herdr::protocol::{SessionSnapshot, SessionSnapshotResult, SuccessResponse},
};

fn snapshot() -> SessionSnapshot {
    let response: SuccessResponse<SessionSnapshotResult> =
        serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
    response.result.snapshot
}

#[test]
fn snapshot_normalizes_sites_agents_attention_and_personas() {
    let state = DomainState::from_snapshot(&snapshot(), Timestamp::from_millis(10_000));
    let site = state.sites.get(&WorkspaceId::new("w1")).unwrap();
    let agent = state.agents.get(&site.agents[0]).unwrap();

    assert_eq!(site.label, "webmaster");
    assert_eq!(site.cwd.to_string_lossy(), "/tmp/herdr-webmaster");
    assert_eq!(site.status(&state.agents), SiteStatus::NeedsWebmaster);
    assert_eq!(agent.name, "Codex");
    assert_eq!(agent.custom_status.as_deref(), Some("which schema?"));
    assert_eq!(agent.presence, Presence::Blocked);
    assert_eq!(agent.pane_revision, 7);
    assert_eq!(
        agent.attention,
        Attention::unseen(AttentionReason::NeedsInput, Timestamp::from_millis(10_000))
    );
    assert!(agent.persona.key.as_str().starts_with("persona-"));
}

#[test]
fn managed_pane_is_excluded_from_snapshot_normalization() {
    let mut snapshot = snapshot();
    let mut managed = snapshot.agents[0].clone();
    managed.pane_id = "w2:p3".to_owned();
    managed.workspace_id = "w2".to_owned();
    managed.name = Some("webmaster-smoke".to_owned());
    managed.agent_session = Some(herdr_webmaster::herdr::protocol::AgentSessionInfo {
        source: "manual-test".to_owned(),
        agent: "webmaster-smoke".to_owned(),
        kind: "session".to_owned(),
        value: "unique-managed-pane".to_owned(),
    });
    snapshot.agents.push(managed);
    let mut workspace = snapshot.workspaces[0].clone();
    workspace.workspace_id = "w2".to_owned();
    workspace.label = "webmaster-test".to_owned();
    snapshot.workspaces.push(workspace);

    let unfiltered = DomainState::from_snapshot(&snapshot, Timestamp::from_millis(10_000));
    assert_eq!(unfiltered.agents.len(), 2);
    assert!(unfiltered.sites.contains_key(&WorkspaceId::new("w2")));

    let state = DomainState::from_snapshot_excluding(
        &snapshot,
        Timestamp::from_millis(10_000),
        Some(&PaneId::new("w2:p3")),
    );

    assert!(state.agent_key_for_pane(&PaneId::new("w2:p3")).is_none());
    assert!(state.sites.values().all(|site| {
        !site
            .agents
            .iter()
            .any(|key| state.agents[key].pane_id == PaneId::new("w2:p3"))
    }));
    assert_ne!(
        state
            .selected_agent
            .as_ref()
            .map(|key| &state.agents[key].pane_id),
        Some(&PaneId::new("w2:p3"))
    );
}

#[test]
fn site_status_uses_the_required_priority() {
    let mut state = DomainState::from_snapshot(&snapshot(), Timestamp::from_millis(1));
    let site = state.sites.values().next().unwrap().clone();
    let key = site.agents[0].clone();
    state.agents.get_mut(&key).unwrap().presence = Presence::Working;
    state.agents.get_mut(&key).unwrap().attention = Attention::Clear;
    assert_eq!(site.status(&state.agents), SiteStatus::Updating);

    state.agents.get_mut(&key).unwrap().presence = Presence::Done;
    state.agents.get_mut(&key).unwrap().attention =
        Attention::unseen(AttentionReason::WorkCompleted, Timestamp::from_millis(2));
    assert_eq!(site.status(&state.agents), SiteStatus::UpdateReady);

    state.agents.get_mut(&key).unwrap().presence = Presence::Blocked;
    assert_eq!(site.status(&state.agents), SiteStatus::NeedsWebmaster);

    state.agents.get_mut(&key).unwrap().presence = Presence::Idle;
    state.agents.get_mut(&key).unwrap().attention = Attention::Clear;
    assert_eq!(site.status(&state.agents), SiteStatus::Online);

    state.agents.get_mut(&key).unwrap().presence = Presence::Exited;
    assert_eq!(site.status(&state.agents), SiteStatus::Offline);
}

#[test]
fn empty_site_is_offline() {
    let mut state = DomainState::from_snapshot(&snapshot(), Timestamp::from_millis(1));
    let site = state.sites.values_mut().next().unwrap();
    site.agents.clear();

    assert_eq!(site.status(&state.agents), SiteStatus::Offline);
}

#[test]
fn agent_keys_and_personas_stay_stable_across_pane_changes_with_a_session() {
    let first = DomainState::from_snapshot(&snapshot(), Timestamp::from_millis(1));
    let mut moved = snapshot();
    moved.agents[0].pane_id = "w1:p9".to_owned();
    moved.panes[0].pane_id = "w1:p9".to_owned();
    let second = DomainState::from_snapshot(&moved, Timestamp::from_millis(2));

    let first_agent = first.agents.values().next().unwrap();
    let second_agent = second.agents.values().next().unwrap();
    assert_eq!(first_agent.key, second_agent.key);
    assert_eq!(first_agent.persona, second_agent.persona);
}
