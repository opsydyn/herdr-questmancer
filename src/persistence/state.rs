use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    app::{DisplayPreferences, Model, View},
    domain::{AdventurerPersona, DomainState, GuildAttention, GuildSummons, PersonaKey},
};

pub const STATE_SCHEMA_VERSION: u32 = 2;
pub const PARTY_NAME_LIMIT: usize = 32;

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct AttentionEpisodeKey {
    pub persona: PersonaKey,
    pub summons: GuildSummons,
}

/// Legacy Rust type name retained for callers; captures write schema v2 and parse migrates v1.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PersistedStateV1 {
    pub schema_version: u32,
    pub last_view: View,
    pub preferences: DisplayPreferences,
    pub selected_persona: Option<PersonaKey>,
    pub personas: BTreeMap<PersonaKey, AdventurerPersona>,
    pub seen_attention: BTreeSet<AttentionEpisodeKey>,
    /// Lifetime standing remains independent of retained Chronicle history.
    #[serde(default)]
    pub experience: u64,
    #[serde(default)]
    pub owned_store: Option<crate::owned_work::StoreSelection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub party_name: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DurableIntent {
    selected_persona: Option<PersonaKey>,
    personas: BTreeMap<PersonaKey, AdventurerPersona>,
    seen_attention: BTreeSet<AttentionEpisodeKey>,
    experience: u64,
    owned_store: Option<crate::owned_work::StoreSelection>,
    party_name: Option<String>,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum StateValidationError {
    #[error("unsupported state schema version {actual}; expected {expected}")]
    UnsupportedSchemaVersion { expected: u32, actual: u32 },
    #[error("persona map key {map_key} does not match embedded key {embedded_key}")]
    PersonaKeyMismatch {
        map_key: PersonaKey,
        embedded_key: PersonaKey,
    },
    #[error("selected persona {0} is missing from the persona map")]
    SelectedPersonaMissing(PersonaKey),
    #[error("invalid owned store selection")]
    InvalidOwnedSelection,
    #[error("invalid party name")]
    InvalidPartyName,
    #[error("seen attention persona {0} is missing from the persona map")]
    SeenAttentionPersonaMissing(PersonaKey),
}

impl PersistedStateV1 {
    #[must_use]
    pub fn capture(model: &Model) -> Self {
        let intent = model.durable_intent();
        Self {
            schema_version: STATE_SCHEMA_VERSION,
            last_view: model.view(),
            preferences: *model.preferences(),
            selected_persona: model
                .selected_agent()
                .map(|agent| agent.persona.key.clone())
                .or_else(|| intent.selected_persona.clone()),
            personas: intent.personas.clone(),
            seen_attention: intent.seen_attention.clone(),
            experience: intent.experience,
            owned_store: intent.owned_store.clone(),
            party_name: intent.party_name.clone(),
        }
    }

    pub fn validate(&self) -> Result<(), StateValidationError> {
        if self.schema_version != STATE_SCHEMA_VERSION && self.schema_version != 1 {
            return Err(StateValidationError::UnsupportedSchemaVersion {
                expected: STATE_SCHEMA_VERSION,
                actual: self.schema_version,
            });
        }

        if self.party_name.as_ref().is_some_and(|name| {
            name.is_empty()
                || name.trim() != name
                || name.chars().count() > PARTY_NAME_LIMIT
                || name.chars().any(char::is_control)
        }) {
            return Err(StateValidationError::InvalidPartyName);
        }

        if (self.schema_version == 1 && self.owned_store.is_some())
            || self
                .owned_store
                .as_ref()
                .is_some_and(|s| s.validate().is_err())
        {
            return Err(StateValidationError::InvalidOwnedSelection);
        }

        for (map_key, persona) in &self.personas {
            if map_key != &persona.key {
                return Err(StateValidationError::PersonaKeyMismatch {
                    map_key: map_key.clone(),
                    embedded_key: persona.key.clone(),
                });
            }
        }

        if let Some(selected) = &self.selected_persona
            && !self.personas.contains_key(selected)
        {
            return Err(StateValidationError::SelectedPersonaMissing(
                selected.clone(),
            ));
        }

        if let Some(episode) = self
            .seen_attention
            .iter()
            .find(|episode| !self.personas.contains_key(&episode.persona))
        {
            return Err(StateValidationError::SeenAttentionPersonaMissing(
                episode.persona.clone(),
            ));
        }

        Ok(())
    }
}

impl DurableIntent {
    pub fn party_name(&self) -> Option<&str> {
        self.party_name.as_deref()
    }

    pub(crate) fn set_party_name(&mut self, name: Option<String>) {
        self.party_name = name;
    }

    pub fn owned_store(&self) -> Option<&crate::owned_work::StoreSelection> {
        self.owned_store.as_ref()
    }
    pub fn set_owned_store(&mut self, selection: Option<crate::owned_work::StoreSelection>) {
        self.owned_store = selection;
    }

    pub fn seed(&mut self, state: &PersistedStateV1) -> Result<(), StateValidationError> {
        state.validate()?;
        self.selected_persona.clone_from(&state.selected_persona);
        self.personas.clone_from(&state.personas);
        self.seen_attention.clone_from(&state.seen_attention);
        self.experience = state.experience;
        self.owned_store.clone_from(&state.owned_store);
        self.party_name.clone_from(&state.party_name);
        Ok(())
    }

    #[must_use]
    pub const fn experience(&self) -> u64 {
        self.experience
    }

    /// Adds to the guild's standing. Saturating and never subtracting: a score
    /// that can go down is not a record of what you have done.
    pub const fn earn(&mut self, experience: u64) {
        self.experience = self.experience.saturating_add(experience);
    }

    pub fn overlay(&mut self, domain: &mut DomainState) {
        for agent in domain.agents.values_mut() {
            let persona_key = agent.persona.key.clone();
            if let Some(persona) = self.personas.get(&persona_key) {
                agent.persona.clone_from(persona);
            } else {
                self.personas.insert(persona_key, agent.persona.clone());
            }
        }

        let selected_matches = self
            .selected_persona
            .as_ref()
            .map_or_else(Vec::new, |selected| {
                domain
                    .agents
                    .iter()
                    .filter(|(_, agent)| &agent.persona.key == selected)
                    .map(|(key, _)| key.clone())
                    .collect::<Vec<_>>()
            });
        if let [selected] = selected_matches.as_slice() {
            domain.selected_agent = Some(selected.clone());
        } else if domain
            .selected_agent
            .as_ref()
            .is_none_or(|key| !domain.agents.contains_key(key))
        {
            domain.selected_agent = domain.agents.keys().next().cloned();
        }

        for agent in domain.agents.values_mut() {
            let Some(summons) = agent.attention.summons() else {
                continue;
            };
            let episode = AttentionEpisodeKey {
                persona: agent.persona.key.clone(),
                summons,
            };
            if matches!(agent.attention, GuildAttention::Unread { .. })
                && self.seen_attention.contains(&episode)
            {
                agent.attention = agent.attention.clone().mark_read();
            }
        }

        let represented = domain
            .agents
            .values()
            .filter_map(attention_episode)
            .collect::<BTreeSet<_>>();
        self.seen_attention
            .retain(|episode| represented.contains(episode));
        self.seen_attention.extend(
            domain
                .agents
                .values()
                .filter(|agent| matches!(agent.attention, GuildAttention::Read { .. }))
                .filter_map(attention_episode),
        );
        self.selected_persona = domain
            .selected_agent
            .as_ref()
            .and_then(|key| domain.agents.get(key))
            .map(|agent| agent.persona.key.clone());
    }

    pub(crate) fn remember_selected(&mut self, selected: Option<PersonaKey>) {
        self.selected_persona = selected;
    }

    pub(crate) fn remember_persona(&mut self, persona: &AdventurerPersona) {
        self.personas.insert(persona.key.clone(), persona.clone());
    }
}

fn attention_episode(agent: &crate::domain::Agent) -> Option<AttentionEpisodeKey> {
    Some(AttentionEpisodeKey {
        persona: agent.persona.key.clone(),
        summons: agent.attention.summons()?,
    })
}
