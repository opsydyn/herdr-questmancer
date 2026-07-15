use std::collections::{BTreeSet, HashSet};

use questmancer::{
    domain::{AdventurerClass, AdventurerPersona, Ancestry, PersonaKey},
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
fn persona_generation_is_stable_and_independent_of_pane_moves() {
    let original = fixture_agent();
    let mut moved_agent = original.clone();
    moved_agent.pane_id = "w1:p9".to_owned();
    let first = AdventurerPersona::for_agent(&original, Some("/repo"));
    let moved = AdventurerPersona::for_agent(&moved_agent, Some("/repo"));

    assert_eq!(first, moved);
    assert!(!first.name.trim().is_empty());
    assert!(!first.epithet.as_str().trim().is_empty());
}

#[test]
fn generated_personas_have_meaningful_trait_diversity() {
    let personas = (0..24)
        .map(|index| {
            let key = PersonaKey::new(format!("persona-{index}"));
            AdventurerPersona::appearance_for_key(&key)
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
            .map(|persona| persona.keepsake)
            .collect::<HashSet<_>>()
            .len()
            >= 5
    );
}

#[test]
fn classic_and_questmancer_classes_are_reachable() {
    let classes = (0..4096)
        .map(|index| AdventurerPersona::for_key(PersonaKey::new(format!("persona-{index}"))).class)
        .collect::<BTreeSet<_>>();

    assert!(classes.contains(&AdventurerClass::Wizard));
    assert!(classes.contains(&AdventurerClass::Rogue));
    assert!(classes.contains(&AdventurerClass::Cleric));
    assert!(classes.contains(&AdventurerClass::Runewright));
    assert!(classes.contains(&AdventurerClass::Testmender));
}

#[test]
fn goblins_are_possible_but_rare() {
    let goblins = (0..16_384)
        .filter(|index| {
            AdventurerPersona::for_key(PersonaKey::new(format!("persona-{index}"))).ancestry
                == Ancestry::Goblin
        })
        .count();

    assert!(goblins > 0);
    assert!(goblins < 256);
}
