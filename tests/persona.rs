use std::collections::HashSet;

use herdr_webmaster::{
    domain::{AgentPersona, PersonaKey},
    herdr::protocol::{AgentInfo, SessionSnapshotResult, SuccessResponse},
};

fn fixture_agent() -> AgentInfo {
    let response: SuccessResponse<SessionSnapshotResult> =
        serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
    response.result.snapshot.agents[0].clone()
}

#[test]
fn native_agent_session_is_the_strongest_persona_identity() {
    let original = fixture_agent();
    let mut moved = original.clone();
    moved.workspace_id = "w9".to_owned();
    moved.pane_id = "w9:p4".to_owned();

    assert_eq!(
        PersonaKey::for_agent(&original, Some("/repo/one")),
        PersonaKey::for_agent(&moved, Some("/repo/two"))
    );
}

#[test]
fn workspace_root_and_agent_name_survive_pane_changes() {
    let mut first = fixture_agent();
    first.agent_session = None;
    let mut second = first.clone();
    second.workspace_id = "new-workspace".to_owned();
    second.pane_id = "new-workspace:p8".to_owned();

    assert_eq!(
        PersonaKey::for_agent(&first, Some("/repo/shared")),
        PersonaKey::for_agent(&second, Some("/repo/shared"))
    );
}

#[test]
fn workspace_and_pane_are_the_last_identity_fallback() {
    let mut first = fixture_agent();
    first.agent_session = None;
    first.name = None;
    first.agent = None;
    first.display_agent = None;
    let mut second = first.clone();
    second.pane_id = "w1:p9".to_owned();

    assert_ne!(
        PersonaKey::for_agent(&first, None),
        PersonaKey::for_agent(&second, None)
    );
}

#[test]
fn persona_generation_is_stable_and_keeps_handle_separate_from_appearance() {
    let agent = fixture_agent();
    let first = AgentPersona::for_agent(&agent, Some("/tmp/herdr-webmaster"));
    let second = AgentPersona::for_agent(&agent, Some("/tmp/herdr-webmaster"));

    assert_eq!(first, second);
    assert!(first.handle.contains("codex"));
    assert_eq!(
        first.appearance,
        AgentPersona::appearance_for_key(&first.key)
    );
}

#[test]
fn generated_personas_have_meaningful_trait_diversity() {
    let personas = (0..24)
        .map(|index| {
            let key = PersonaKey::new(format!("persona-{index}"));
            AgentPersona::appearance_for_key(&key)
        })
        .collect::<HashSet<_>>();

    assert!(personas.len() >= 20);
    assert!(
        personas
            .iter()
            .map(|persona| persona.hair)
            .collect::<HashSet<_>>()
            .len()
            >= 5
    );
    assert!(
        personas
            .iter()
            .map(|persona| persona.accessory)
            .collect::<HashSet<_>>()
            .len()
            >= 5
    );
}
