#[allow(dead_code)]
mod support;

use proptest::prelude::*;
use questmancer::{
    app::{Model, View},
    domain::{AgentKey, GuildAttention, PersonaKey, Timestamp},
    persistence::{AttentionEpisodeKey, DurableIntent, PersistedStateV1},
};

fn captured_state() -> PersistedStateV1 {
    let mut model = Model::new(View::Delve);
    model.replace_domain(support::fixture_domain());
    model.mark_selected_attention_read();
    PersistedStateV1::capture(&model)
}

#[test]
fn capture_contains_only_durable_intent() {
    let mut model = Model::new(View::Delve);
    model.replace_domain(support::fixture_domain());
    let agent = model.selected_agent().unwrap();
    let expected_persona = agent.persona.key.clone();
    let expected_revision = agent.pane_revision;
    let expected_summons = agent.attention.summons().unwrap();
    model.mark_selected_attention_read();

    let state = PersistedStateV1::capture(&model);

    assert_eq!(state.schema_version, 1);
    assert_eq!(state.selected_persona, Some(expected_persona.clone()));
    assert_eq!(state.personas[&expected_persona].key, expected_persona);
    assert!(state.seen_attention.contains(&AttentionEpisodeKey {
        persona: expected_persona,
        pane_revision: expected_revision,
        summons: expected_summons,
    }));
}

#[test]
fn validation_rejects_a_mismatched_embedded_persona_key() {
    let mut state = captured_state();
    let map_key = state.personas.keys().next().unwrap().clone();
    state.personas.get_mut(&map_key).unwrap().key = PersonaKey::new("persona-other");

    assert!(state.validate().is_err());
}

#[test]
fn validation_rejects_a_selected_persona_missing_from_the_map() {
    let mut state = captured_state();
    state.selected_persona = Some(PersonaKey::new("persona-missing"));

    assert!(state.validate().is_err());
}

#[test]
fn validation_rejects_a_seen_episode_referencing_a_missing_persona() {
    let mut state = captured_state();
    let episode = state.seen_attention.iter().next().unwrap().clone();
    state.seen_attention.insert(AttentionEpisodeKey {
        persona: PersonaKey::new("persona-missing"),
        ..episode
    });

    assert!(state.validate().is_err());
}

#[test]
fn failed_seed_is_atomic_and_cannot_cross_persona_identities() {
    let mut valid = captured_state();
    let persona_key = valid.personas.keys().next().unwrap().clone();
    valid.personas.get_mut(&persona_key).unwrap().handle = "known_safe".to_owned();
    let mut model = Model::new(View::Guild);
    model.durable_intent_mut().seed(&valid).unwrap();
    let before = model.durable_intent().clone();
    let mut invalid = valid;
    let embedded = invalid.personas.get_mut(&persona_key).unwrap();
    embedded.key = PersonaKey::new("persona-crossed");
    embedded.handle = "wrong_identity".to_owned();

    let result = model.durable_intent_mut().seed(&invalid);

    assert!(result.is_err());
    assert_eq!(model.durable_intent(), &before);
    model.replace_domain(support::fixture_domain());
    let persona = &model.selected_agent().unwrap().persona;
    assert_eq!(persona.key, persona_key);
    assert_eq!(persona.handle, "known_safe");
}

#[test]
fn validation_rejects_an_unsupported_schema_version() {
    let mut state = captured_state();
    state.schema_version = 2;

    assert!(state.validate().is_err());
}

#[test]
fn overlay_restores_matching_persona_selection_and_seen_episode() {
    let mut state = captured_state();
    let selected = state.selected_persona.clone().unwrap();
    state.personas.get_mut(&selected).unwrap().handle = "authored_handle".to_owned();
    let domain = support::fixture_domain();
    let before = support::live_facts(&domain);
    let mut model = Model::new(View::Guild);
    model.durable_intent_mut().seed(&state).unwrap();

    model.replace_domain(domain);

    let selected_agent = model.selected_agent().unwrap();
    assert_eq!(selected_agent.persona.key, selected);
    assert_eq!(selected_agent.persona.handle, "authored_handle");
    assert!(matches!(
        selected_agent.attention,
        GuildAttention::Read { .. }
    ));
    assert_eq!(support::live_facts(model.domain()), before);
}

#[test]
fn overlay_prunes_seen_episode_when_revision_no_longer_matches() {
    let state = captured_state();
    let stale_episode = state.seen_attention.iter().next().unwrap().clone();
    let mut domain = support::fixture_domain();
    domain.agents.values_mut().next().unwrap().pane_revision += 1;
    let mut model = Model::new(View::Guild);
    model.durable_intent_mut().seed(&state).unwrap();

    model.replace_domain(domain);

    assert!(model.selected_agent().unwrap().attention.is_unread());
    assert!(
        !PersistedStateV1::capture(&model)
            .seen_attention
            .contains(&stale_episode)
    );
}

#[test]
fn overlay_retains_learned_personas_that_are_not_live() {
    let mut state = captured_state();
    let mut historical = state.personas.values().next().unwrap().clone();
    historical.key = PersonaKey::new("persona-historical");
    state
        .personas
        .insert(historical.key.clone(), historical.clone());
    let mut model = Model::new(View::Guild);
    model.durable_intent_mut().seed(&state).unwrap();

    model.replace_domain(support::fixture_domain());

    assert_eq!(
        PersistedStateV1::capture(&model).personas[&historical.key],
        historical
    );
}

#[test]
fn overlay_keeps_a_valid_snapshot_selection_when_persona_is_ambiguous() {
    let state = captured_state();
    let mut domain = support::fixture_domain();
    let mut duplicate = domain.agents.values().next().unwrap().clone();
    duplicate.key = AgentKey::new("agent-duplicate");
    duplicate.pane_id = "w1:p2".into();
    domain
        .agents
        .insert(duplicate.key.clone(), duplicate.clone());
    domain.selected_agent = Some(duplicate.key.clone());
    let mut model = Model::new(View::Guild);
    model.durable_intent_mut().seed(&state).unwrap();

    model.replace_domain(domain);

    assert_eq!(model.selected_agent_key(), Some(&duplicate.key));
}

#[test]
fn overlay_marks_only_unread_attention_as_read() {
    let state = captured_state();
    let episode = state.seen_attention.iter().next().unwrap();
    let mut domain = support::fixture_domain();
    domain.agents.values_mut().next().unwrap().attention = GuildAttention::Deferred {
        summons: episode.summons,
        since: Timestamp::from_millis(1_000),
        until: Timestamp::from_millis(2_000),
    };
    let mut model = Model::new(View::Guild);
    model.durable_intent_mut().seed(&state).unwrap();

    model.replace_domain(domain);

    assert!(matches!(
        model.selected_agent().unwrap().attention,
        GuildAttention::Deferred { .. }
    ));
}

proptest! {
    #[test]
    fn valid_state_json_round_trips(state in support::persisted_state()) {
        let bytes = serde_json::to_vec(&state).unwrap();
        let decoded: PersistedStateV1 = serde_json::from_slice(&bytes).unwrap();
        prop_assert_eq!(decoded, state);
    }

    #[test]
    fn overlay_cannot_replace_live_facts(
        mut domain in support::domain_state(),
        state in support::persisted_state(),
    ) {
        let before = support::live_facts(&domain);
        let mut intent = DurableIntent::default();
        prop_assert!(intent.seed(&state).is_ok());
        intent.overlay(&mut domain);
        prop_assert_eq!(support::live_facts(&domain), before);
        prop_assert!(domain.selected_agent.as_ref().is_none_or(|key| domain.agents.contains_key(key)));
    }
}
