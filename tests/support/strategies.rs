use std::{collections::BTreeMap, path::PathBuf};

use herdr_webmaster::{
    app::{CharacterSet, ColorMode, DisplayPreferences, Motion, View},
    domain::{
        Agent, AgentKey, AgentPersona, Attention, AttentionReason, DomainState, Guestbook, PaneId,
        PersonaKey, Presence, Site, TabId, Timestamp, WorkspaceId,
    },
    persistence::{AttentionEpisodeKey, PersistedStateV1, STATE_SCHEMA_VERSION},
};
use proptest::prelude::*;

fn id_text() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,11}"
}

fn display_text() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 _-]{1,20}"
}

pub(crate) fn workspace_id() -> impl Strategy<Value = WorkspaceId> {
    id_text().prop_map(WorkspaceId::new)
}

pub(crate) fn tab_id() -> impl Strategy<Value = TabId> {
    id_text().prop_map(TabId::new)
}

pub(crate) fn pane_id() -> impl Strategy<Value = PaneId> {
    id_text().prop_map(PaneId::new)
}

pub(crate) fn agent_key() -> impl Strategy<Value = AgentKey> {
    id_text().prop_map(AgentKey::new)
}

pub(crate) fn persona_key() -> impl Strategy<Value = PersonaKey> {
    id_text().prop_map(PersonaKey::new)
}

pub(crate) fn attention_reason() -> impl Strategy<Value = AttentionReason> {
    prop_oneof![
        Just(AttentionReason::NeedsInput),
        Just(AttentionReason::WorkCompleted),
        Just(AttentionReason::PaneExited),
    ]
}

pub(crate) fn persona() -> impl Strategy<Value = AgentPersona> {
    (persona_key(), display_text()).prop_map(|(key, handle)| AgentPersona {
        appearance: AgentPersona::appearance_for_key(&key),
        key,
        handle,
    })
}

fn timestamp() -> impl Strategy<Value = Timestamp> {
    any::<i64>().prop_map(Timestamp::from_millis)
}

fn presence() -> impl Strategy<Value = Presence> {
    prop_oneof![
        Just(Presence::Working),
        Just(Presence::Blocked),
        Just(Presence::Done),
        Just(Presence::Idle),
        Just(Presence::Exited),
        Just(Presence::Unknown),
    ]
}

fn attention() -> impl Strategy<Value = Attention> {
    prop_oneof![
        Just(Attention::Clear),
        (attention_reason(), timestamp())
            .prop_map(|(reason, since)| Attention::Unseen { reason, since }),
        (attention_reason(), timestamp())
            .prop_map(|(reason, since)| Attention::Seen { reason, since }),
        (attention_reason(), timestamp(), timestamp()).prop_map(|(reason, since, until)| {
            Attention::Snoozed {
                reason,
                since,
                until,
            }
        }),
    ]
}

pub(crate) fn agent() -> impl Strategy<Value = Agent> {
    (
        (agent_key(), pane_id(), workspace_id(), tab_id()),
        (
            display_text(),
            prop::option::of(display_text()),
            presence(),
            timestamp(),
            attention(),
        ),
        (any::<bool>(), any::<u64>(), persona()),
    )
        .prop_map(
            |(
                (key, pane_id, workspace_id, tab_id),
                (name, custom_status, presence, presence_since, attention),
                (focused, pane_revision, persona),
            )| Agent {
                key,
                pane_id,
                workspace_id,
                tab_id,
                name,
                custom_status,
                presence,
                presence_since,
                attention,
                focused,
                pane_revision,
                persona,
            },
        )
}

pub(crate) fn domain_state() -> impl Strategy<Value = DomainState> {
    prop::collection::vec(agent(), 0..=4)
        .prop_map(|agents| {
            agents
                .into_iter()
                .map(|agent| (agent.key.clone(), agent))
                .collect::<BTreeMap<_, _>>()
        })
        .prop_flat_map(|agents| {
            let keys = agents.keys().cloned().collect::<Vec<_>>();
            let selection = if keys.is_empty() {
                Just(None).boxed()
            } else {
                prop_oneof![Just(None), prop::sample::select(keys).prop_map(Some),].boxed()
            };
            (Just(agents), selection)
        })
        .prop_map(|(agents, selected_agent)| {
            let mut sites = BTreeMap::<WorkspaceId, Site>::new();
            for (key, agent) in &agents {
                sites
                    .entry(agent.workspace_id.clone())
                    .or_insert_with(|| Site {
                        workspace_id: agent.workspace_id.clone(),
                        label: format!("site-{}", agent.workspace_id),
                        cwd: PathBuf::from(format!("/tmp/{}", agent.workspace_id)),
                        agents: Vec::new(),
                    })
                    .agents
                    .push(key.clone());
            }
            DomainState {
                sites,
                agents,
                selected_agent,
                guestbook: Guestbook::default(),
            }
        })
}

fn view() -> impl Strategy<Value = View> {
    prop_oneof![Just(View::Desk), Just(View::Cafe)]
}

fn preferences() -> impl Strategy<Value = DisplayPreferences> {
    (
        prop_oneof![
            Just(Motion::Full),
            Just(Motion::Reduced),
            Just(Motion::None)
        ],
        prop_oneof![Just(CharacterSet::Unicode), Just(CharacterSet::Ascii)],
        prop_oneof![Just(ColorMode::Xterm256), Just(ColorMode::Ansi16)],
    )
        .prop_map(|(motion, character_set, color_mode)| DisplayPreferences {
            motion,
            character_set,
            color_mode,
        })
}

pub(crate) fn persisted_state() -> impl Strategy<Value = PersistedStateV1> {
    prop::collection::vec(persona(), 0..=4)
        .prop_map(|personas| {
            personas
                .into_iter()
                .map(|persona| (persona.key.clone(), persona))
                .collect::<BTreeMap<_, _>>()
        })
        .prop_flat_map(|personas| {
            let keys = personas.keys().cloned().collect::<Vec<_>>();
            let selected = if keys.is_empty() {
                Just(None).boxed()
            } else {
                prop::option::of(prop::sample::select(keys.clone())).boxed()
            };
            let episodes = if keys.is_empty() {
                Just(Vec::new()).boxed()
            } else {
                prop::collection::vec(
                    (prop::sample::select(keys), any::<u64>(), attention_reason()),
                    0..=6,
                )
                .boxed()
            };
            (view(), preferences(), Just(personas), selected, episodes)
        })
        .prop_map(
            |(last_view, preferences, personas, selected_persona, episodes)| PersistedStateV1 {
                schema_version: STATE_SCHEMA_VERSION,
                last_view,
                preferences,
                selected_persona,
                personas,
                seen_attention: episodes
                    .into_iter()
                    .map(|(persona, pane_revision, reason)| AttentionEpisodeKey {
                        persona,
                        pane_revision,
                        reason,
                    })
                    .collect(),
            },
        )
}
