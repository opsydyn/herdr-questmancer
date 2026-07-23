use questmancer::{
    domain::{DomainState, GuildAttention, GuildSummons, Presence, Timestamp},
    herdr::protocol::{SessionSnapshotResult, SuccessResponse},
    sidebar::{QUEST_CAMPAIGN, QUEST_OMEN, QUEST_ROLE, SidebarProjection},
};

fn fixture_domain() -> DomainState {
    let response: SuccessResponse<SessionSnapshotResult> =
        serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
    DomainState::from_snapshot(&response.result.snapshot, Timestamp::from_millis(1_000))
}

#[test]
fn marginalia_projects_roles_truthful_omens_and_campaign_summons() {
    let mut domain = fixture_domain();
    let agent = domain.agents.values_mut().next().unwrap();
    agent.presence = Presence::Blocked;
    agent.attention = GuildAttention::unread(
        GuildSummons::CounselRequested,
        Timestamp::from_millis(1_000),
    );
    let expected_role = format!("{:?} {:?}", agent.persona.ancestry, agent.persona.class);

    let projection = SidebarProjection::from_domain(&domain);

    assert_eq!(projection.agents.len(), 1);
    assert_eq!(
        projection.agents[0].tokens.get(QUEST_ROLE),
        Some(&expected_role)
    );
    assert_eq!(
        projection.agents[0].tokens.get(QUEST_OMEN),
        Some(&"seeks counsel".to_owned())
    );
    assert_eq!(projection.campaigns.len(), 1);
    assert_eq!(
        projection.campaigns[0].tokens.get(QUEST_CAMPAIGN),
        Some(&"1 adventurer · 1 summons".to_owned())
    );
}

#[test]
fn marginalia_keeps_departed_adventurers_truthful_and_out_of_the_party_count() {
    let mut domain = fixture_domain();
    let agent = domain.agents.values_mut().next().unwrap();
    agent.presence = Presence::Exited;

    let projection = SidebarProjection::from_domain(&domain);

    assert_eq!(
        projection.agents[0].tokens.get(QUEST_OMEN),
        Some(&"departed the guild".to_owned())
    );
    assert_eq!(
        projection.campaigns[0].tokens.get(QUEST_CAMPAIGN),
        Some(&"no adventurers".to_owned())
    );
}
