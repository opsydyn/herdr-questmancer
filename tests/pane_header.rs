use questmancer::{
    domain::{DomainState, Timestamp},
    herdr::protocol::{SessionSnapshotResult, SuccessResponse},
    pane_header::PaneHeaderProjection,
};

fn fixture() -> DomainState {
    let response: SuccessResponse<SessionSnapshotResult> =
        serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
    DomainState::from_snapshot(&response.result.snapshot, Timestamp::from_millis(1000))
}

#[test]
fn unqualified_adventurers_cannot_publish_a_pane_header() {
    let mut domain = fixture();
    domain.agents.values_mut().next().unwrap().capture_identity =
        questmancer::domain::CaptureIdentity::Unqualified;
    assert!(PaneHeaderProjection::from_domain(&domain, None, true).is_empty());
}

fn qualified() -> DomainState {
    let mut domain = fixture();
    let agent = domain.agents.values_mut().next().unwrap();
    agent.name = "Ari".into();
    agent.persona.class = questmancer::domain::AdventurerClass::Pathseeker;
    agent.presence = questmancer::domain::Presence::Working;
    agent.capture_identity = questmancer::domain::CaptureIdentity::Session {
        terminal: "terminal-1".into(),
        session: questmancer::herdr::protocol::AgentSessionInfo {
            source: "test:pi".into(),
            agent: "pi".into(),
            kind: "id".into(),
            value: "session-a".into(),
        },
    };
    domain
}

#[test]
fn uses_live_name_class_condition_and_authored_native_pixels() {
    let domain = qualified();
    let header = PaneHeaderProjection::from_domain(&domain, None, true).remove(0);
    assert_eq!(header.label, "Ari · Pathseeker · working");
    assert_eq!(header.rgba.len(), 8 * 12 * 4);
    assert!(header.rgba.chunks_exact(4).any(|p| p[3] == 255));
    assert!(header.rgba.chunks_exact(4).any(|p| p[3] == 0));
    let sprite = questmancer::scene::assets::adventurer::adventurer_roster_frame(
        &domain.agents.values().next().unwrap().persona,
    );
    let expected: Vec<u8> = sprite
        .pixels()
        .iter()
        .flat_map(|p| p.map_or([0; 4], |c| [c.r, c.g, c.b, 255]))
        .collect();
    assert_eq!(header.rgba, expected);
}

#[test]
fn disconnected_managed_departed_and_ambiguous_panes_are_omitted() {
    let mut domain = qualified();
    let pane = domain.agents.values().next().unwrap().pane_id.clone();
    assert!(PaneHeaderProjection::from_domain(&domain, None, false).is_empty());
    assert!(PaneHeaderProjection::from_domain(&domain, Some(&pane), true).is_empty());
    let mut duplicate = domain.agents.values().next().unwrap().clone();
    duplicate.key = questmancer::domain::AgentKey::new("other");
    domain.agents.insert(duplicate.key.clone(), duplicate);
    assert!(PaneHeaderProjection::from_domain(&domain, None, true).is_empty());
    domain
        .agents
        .remove(&questmancer::domain::AgentKey::new("other"));
    domain.agents.values_mut().next().unwrap().presence = questmancer::domain::Presence::Exited;
    assert!(PaneHeaderProjection::from_domain(&domain, None, true).is_empty());
}

#[test]
fn identity_replacement_changes_payload_even_when_name_and_presence_match() {
    let mut domain = qualified();
    let before = PaneHeaderProjection::from_domain(&domain, None, true);
    let questmancer::domain::CaptureIdentity::Session { session, .. } =
        &mut domain.agents.values_mut().next().unwrap().capture_identity
    else {
        panic!()
    };
    session.value = "replacement".into();
    let after = PaneHeaderProjection::from_domain(&domain, None, true);
    assert_eq!(before[0].label, after[0].label);
    assert_ne!(before, after);
}
