use questmancer::{
    app::CharacterSet,
    domain::{AgentKey, DomainState, GuildAttention, GuildSummons, PaneId, Presence, Timestamp},
    herdr::protocol::{SessionSnapshotResult, SuccessResponse},
    sidebar::{
        QUEST_CAMPAIGN, QUEST_CONDITION, QUEST_HOARD, QUEST_OMEN, QUEST_PARTY, QUEST_ROLE,
        QUEST_SIGIL, QUEST_TRINKET, QUEST_VIGIL, SidebarProjection,
    },
};

const NOW: Timestamp = Timestamp::from_millis(1_000);

fn project(domain: &DomainState) -> SidebarProjection {
    SidebarProjection::from_domain(domain, NOW, CharacterSet::Unicode)
}

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

    let projection = project(&domain);

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

    let projection = project(&domain);

    assert_eq!(
        projection.agents[0].tokens.get(QUEST_OMEN),
        Some(&"departed the guild".to_owned())
    );
    assert_eq!(
        projection.campaigns[0].tokens.get(QUEST_CAMPAIGN),
        Some(&"no adventurers".to_owned())
    );
}

/// The sidebar is a character sheet, so a reader should be able to tell an
/// adventurer's class, condition and standing without opening the card.
#[test]
fn marginalia_projects_a_character_sheet_for_each_adventurer() {
    let mut domain = fixture_domain();
    let agent = domain.agents.values_mut().next().unwrap();
    agent.presence = Presence::Working;

    let projection = project(&domain);
    let tokens = &projection.agents[0].tokens;

    assert_eq!(
        tokens.get(QUEST_CONDITION),
        Some(&"Concentrating".to_owned())
    );
    // Every class has a sigil, and none of them is blank.
    assert!(tokens.get(QUEST_SIGIL).is_some_and(|s| !s.is_empty()));
    assert!(tokens.get(QUEST_TRINKET).is_some_and(|s| !s.is_empty()));
    // Nothing has been returned in the fixture, so the bag is honestly empty.
    assert_eq!(tokens.get(QUEST_HOARD), Some(&"◈ empty".to_owned()));
    // Only a blocked adventurer keeps a vigil.
    assert_eq!(tokens.get(QUEST_VIGIL), Some(&String::new()));
}

/// The vigil is an exhaustion track for a summons nobody has answered, so it
/// has to deepen as the wait lengthens rather than sit at one value.
#[test]
fn an_unanswered_summons_deepens_its_vigil_over_time() {
    let mut domain = fixture_domain();
    let agent = domain.agents.values_mut().next().unwrap();
    agent.presence = Presence::Blocked;
    agent.presence_since = Timestamp::from_millis(0);

    let fresh = SidebarProjection::from_domain(
        &domain,
        Timestamp::from_millis(10_000),
        CharacterSet::Unicode,
    );
    let stale = SidebarProjection::from_domain(
        &domain,
        Timestamp::from_millis(2 * 60 * 60 * 1_000),
        CharacterSet::Unicode,
    );

    let fresh_vigil = fresh.agents[0].tokens.get(QUEST_VIGIL).unwrap();
    let stale_vigil = stale.agents[0].tokens.get(QUEST_VIGIL).unwrap();
    assert_eq!(fresh_vigil.chars().count(), 6);
    assert_eq!(stale_vigil.chars().count(), 6);
    assert!(
        stale_vigil.matches('●').count() > fresh_vigil.matches('●').count(),
        "a longer wait must read as a deeper vigil: {fresh_vigil} then {stale_vigil}"
    );
}

/// A terminal without Unicode still has to get a readable sheet, not boxes.
#[test]
fn the_ascii_character_set_keeps_every_token_readable() {
    let mut domain = fixture_domain();
    let agent = domain.agents.values_mut().next().unwrap();
    agent.presence = Presence::Blocked;

    let projection = SidebarProjection::from_domain(&domain, NOW, CharacterSet::Ascii);
    let tokens = &projection.agents[0].tokens;

    for token in [QUEST_SIGIL, QUEST_HOARD, QUEST_VIGIL] {
        let value = tokens.get(token).expect("token is published");
        assert!(
            value.is_ascii(),
            "{token} left non-ASCII output in ASCII mode: {value}"
        );
    }
    assert!(
        projection.campaigns[0]
            .tokens
            .get(QUEST_PARTY)
            .is_some_and(|party| party.is_ascii())
    );
}

/// The rank token Herdr sorts by and the `!` jump inside Questmancer must
/// agree, or the sidebar order contradicts the key. Both read one definition
/// in the domain; this proves they still line up end to end.
#[test]
fn the_rank_token_orders_agents_the_way_the_urgency_jump_does() {
    use questmancer::{
        app::{Model, View},
        sidebar::QUEST_RANK,
    };

    let mut model = Model::new(View::Guild);
    let mut domain = fixture_domain();
    let template = domain.agents.values().next().unwrap().clone();
    domain.agents.clear();
    for (name, presence, attention) in [
        ("aa-quiet", Presence::Working, GuildAttention::Clear),
        (
            "bb-unanswered",
            Presence::Blocked,
            GuildAttention::unread(GuildSummons::CounselRequested, Timestamp::from_millis(10)),
        ),
        (
            "cc-seen",
            Presence::Blocked,
            GuildAttention::Read {
                summons: GuildSummons::CounselRequested,
                since: Timestamp::from_millis(20),
            },
        ),
        (
            "dd-quieter",
            Presence::Done,
            GuildAttention::unread(GuildSummons::SpoilsReturned, Timestamp::from_millis(30)),
        ),
    ] {
        let mut agent = template.clone();
        agent.key = AgentKey::new(name);
        agent.pane_id = PaneId::new(format!("w1:{name}"));
        agent.presence = presence;
        agent.attention = attention;
        domain.agents.insert(agent.key.clone(), agent);
    }
    model.replace_domain(domain.clone());
    model.set_now(Timestamp::from_millis(1_000));

    let projection = SidebarProjection::from_domain(
        &domain,
        Timestamp::from_millis(1_000),
        CharacterSet::Unicode,
    );
    let rank_of = |name: &str| {
        let key = AgentKey::new(name);
        let pane = domain.agents[&key].pane_id.clone();
        projection
            .agents
            .iter()
            .find(|entry| entry.pane_id == pane)
            .and_then(|entry| entry.tokens.get(QUEST_RANK))
            .cloned()
            .unwrap_or_default()
    };

    assert_eq!(rank_of("bb-unanswered"), "0");
    assert_eq!(rank_of("cc-seen"), "1");
    assert_eq!(rank_of("dd-quieter"), "2");
    assert_eq!(rank_of("aa-quiet"), "3", "nothing wanted sorts last");

    // Ascending string order over the ranks must reproduce the jump's order.
    let waiting = model.adventurers_awaiting_a_human();
    let mut by_rank = ["bb-unanswered", "cc-seen", "dd-quieter"]
        .map(|name| (rank_of(name), name))
        .to_vec();
    by_rank.sort();
    assert_eq!(
        by_rank.iter().map(|(_, name)| *name).collect::<Vec<_>>(),
        waiting.iter().map(AgentKey::as_str).collect::<Vec<_>>(),
        "sorting by the published token must match the urgency jump"
    );
}

/// One digit on purpose: Herdr compares custom tokens as strings, and a single
/// digit orders identically whether that comparison is lexicographic or
/// numeric. A wider value would depend on which, and we cannot see which.
#[test]
fn the_rank_token_is_always_one_digit() {
    use questmancer::sidebar::QUEST_RANK;

    let mut domain = fixture_domain();
    let template = domain.agents.values().next().unwrap().clone();
    domain.agents.clear();
    for (index, presence) in [
        Presence::Working,
        Presence::Blocked,
        Presence::Done,
        Presence::Idle,
        Presence::Exited,
        Presence::Unknown,
    ]
    .into_iter()
    .enumerate()
    {
        let mut agent = template.clone();
        agent.key = AgentKey::new(format!("agent-{index}"));
        agent.pane_id = PaneId::new(format!("w1:p{index}"));
        agent.presence = presence;
        domain.agents.insert(agent.key.clone(), agent);
    }

    let projection = SidebarProjection::from_domain(
        &domain,
        Timestamp::from_millis(1_000),
        CharacterSet::Unicode,
    );
    for entry in &projection.agents {
        let rank = entry.tokens.get(QUEST_RANK).expect("every agent is ranked");
        assert_eq!(rank.len(), 1, "rank {rank:?} is not a single digit");
        assert!(rank.chars().all(|c| c.is_ascii_digit()));
    }
}
