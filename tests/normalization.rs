use questmancer::{
    domain::{
        CampaignStatus, DomainState, GuildAttention, GuildSummons, PaneId, Presence, Timestamp,
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
fn snapshot_normalizes_campaigns_agents_attention_and_personas() {
    let state = DomainState::from_snapshot(&snapshot(), Timestamp::from_millis(10_000));
    let campaign = state.campaigns.get(&WorkspaceId::new("w1")).unwrap();
    let agent = state.agents.get(&campaign.party[0]).unwrap();

    assert_eq!(campaign.label, "webmaster");
    assert_eq!(campaign.cwd.to_string_lossy(), "/tmp/herdr-questmancer");
    assert_eq!(
        campaign.status(&state.agents),
        CampaignStatus::CounselRequired
    );
    assert_eq!(agent.name, "Codex");
    assert_eq!(agent.custom_status.as_deref(), Some("which schema?"));
    assert_eq!(agent.presence, Presence::Blocked);
    assert_eq!(agent.pane_revision, 7);
    assert_eq!(
        agent.attention,
        GuildAttention::unread(
            GuildSummons::CounselRequested,
            Timestamp::from_millis(10_000),
        )
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
    managed.agent_session = Some(questmancer::herdr::protocol::AgentSessionInfo {
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
    assert!(unfiltered.campaigns.contains_key(&WorkspaceId::new("w2")));

    let state = DomainState::from_snapshot_excluding(
        &snapshot,
        Timestamp::from_millis(10_000),
        Some(&PaneId::new("w2:p3")),
    );

    assert!(state.agent_key_for_pane(&PaneId::new("w2:p3")).is_none());
    assert!(state.campaigns.values().all(|campaign| {
        !campaign
            .party
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
fn campaign_status_uses_the_required_priority() {
    let mut state = DomainState::from_snapshot(&snapshot(), Timestamp::from_millis(1));
    let campaign = state.campaigns.values().next().unwrap().clone();
    let key = campaign.party[0].clone();
    state.agents.get_mut(&key).unwrap().presence = Presence::Working;
    state.agents.get_mut(&key).unwrap().attention = GuildAttention::Clear;
    assert_eq!(
        campaign.status(&state.agents),
        CampaignStatus::ExpeditionActive
    );

    state.agents.get_mut(&key).unwrap().presence = Presence::Done;
    state.agents.get_mut(&key).unwrap().attention =
        GuildAttention::unread(GuildSummons::SpoilsReturned, Timestamp::from_millis(2));
    assert_eq!(
        campaign.status(&state.agents),
        CampaignStatus::SpoilsAwaitingInspection
    );

    state.agents.get_mut(&key).unwrap().presence = Presence::Blocked;
    assert_eq!(
        campaign.status(&state.agents),
        CampaignStatus::CounselRequired
    );

    state.agents.get_mut(&key).unwrap().presence = Presence::Idle;
    state.agents.get_mut(&key).unwrap().attention = GuildAttention::Clear;
    assert_eq!(campaign.status(&state.agents), CampaignStatus::PartyAtRest);

    state.agents.get_mut(&key).unwrap().presence = Presence::Exited;
    assert_eq!(campaign.status(&state.agents), CampaignStatus::Abandoned);
}

#[test]
fn empty_campaign_is_abandoned() {
    let mut state = DomainState::from_snapshot(&snapshot(), Timestamp::from_millis(1));
    let campaign = state.campaigns.values_mut().next().unwrap();
    campaign.party.clear();

    assert_eq!(campaign.status(&state.agents), CampaignStatus::Abandoned);
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
