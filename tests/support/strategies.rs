use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
};

use herdr_webmaster::{
    app::{CharacterSet, ColorMode, DisplayPreferences, Motion, View},
    domain::{
        Agent, AgentKey, AgentPersona, Attention, AttentionReason, DomainState, Guestbook,
        GuestbookEvent, PaneId, PersonaKey, Presence, Site, TabId, Timestamp, WorkspaceId,
    },
    herdr::protocol::{AgentInfo, AgentSessionInfo, AgentStatus, SessionSnapshot, WorkspaceInfo},
    persistence::{AttentionEpisodeKey, PersistedStateV1, STATE_SCHEMA_VERSION},
    update::AppEvent,
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

pub(crate) fn timestamp() -> impl Strategy<Value = Timestamp> {
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

pub(crate) fn attention() -> impl Strategy<Value = Attention> {
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

pub(crate) fn agent_status() -> impl Strategy<Value = AgentStatus> {
    prop_oneof![
        Just(AgentStatus::Idle),
        Just(AgentStatus::Working),
        Just(AgentStatus::Blocked),
        Just(AgentStatus::Done),
        Just(AgentStatus::Unknown),
    ]
}

fn agent_session() -> impl Strategy<Value = AgentSessionInfo> {
    (
        display_text(),
        display_text(),
        display_text(),
        display_text(),
    )
        .prop_map(|(source, agent, kind, value)| AgentSessionInfo {
            source,
            agent,
            kind,
            value,
        })
}

pub(crate) fn agent_identity() -> impl Strategy<Value = (AgentInfo, Option<String>)> {
    (
        (workspace_id(), tab_id(), pane_id()),
        (
            prop::option::of(display_text()),
            prop::option::of(display_text()),
            prop::option::of(display_text()),
        ),
        prop::option::of(agent_session()),
        prop::option::of(id_text().prop_map(|root| format!("/repo/{root}"))),
        agent_status(),
        0_u64..=1_000,
    )
        .prop_map(
            |(
                (workspace_id, tab_id, pane_id),
                (name, agent, display_agent),
                agent_session,
                workspace_root,
                agent_status,
                revision,
            )| {
                (
                    AgentInfo {
                        terminal_id: format!("terminal-{pane_id}"),
                        agent_status,
                        workspace_id: workspace_id.to_string(),
                        tab_id: tab_id.to_string(),
                        pane_id: pane_id.to_string(),
                        focused: false,
                        revision,
                        agent,
                        agent_session,
                        custom_status: None,
                        cwd: None,
                        display_agent,
                        foreground_cwd: None,
                        name,
                        screen_detection_skipped: false,
                        state_labels: HashMap::new(),
                        title: None,
                    },
                    workspace_root,
                )
            },
        )
}

pub(crate) fn domain_with_one_agent() -> impl Strategy<Value = DomainState> {
    agent().prop_map(|mut agent| {
        agent.pane_revision = agent.pane_revision % 100 + 1;
        let key = agent.key.clone();
        let workspace_id = agent.workspace_id.clone();
        let mut agents = BTreeMap::new();
        agents.insert(key.clone(), agent);
        let mut sites = BTreeMap::new();
        sites.insert(
            workspace_id.clone(),
            Site {
                workspace_id,
                label: "site".to_owned(),
                cwd: PathBuf::from("/tmp/site"),
                agents: vec![key.clone()],
            },
        );
        DomainState {
            sites,
            agents,
            selected_agent: Some(key),
            guestbook: Guestbook::default(),
        }
    })
}

pub(crate) fn status_event(state: &DomainState, revision: u64, status: AgentStatus) -> AppEvent {
    let pane_id = state.agents.values().next().unwrap().pane_id.clone();
    AppEvent::AgentStatusChanged {
        pane_id,
        status,
        custom_status: None,
        revision,
        occurred_at: Timestamp::from_millis(i64::try_from(revision).unwrap_or(i64::MAX)),
    }
}

fn session_snapshot() -> impl Strategy<Value = SessionSnapshot> {
    prop::collection::vec(agent_identity().prop_map(|(agent, _)| agent), 0..=4).prop_map(|agents| {
        let mut workspaces = BTreeMap::<String, WorkspaceInfo>::new();
        for agent in &agents {
            workspaces
                .entry(agent.workspace_id.clone())
                .or_insert_with(|| WorkspaceInfo {
                    workspace_id: agent.workspace_id.clone(),
                    number: 1,
                    label: format!("site-{}", agent.workspace_id),
                    focused: false,
                    pane_count: 0,
                    tab_count: 1,
                    active_tab_id: agent.tab_id.clone(),
                    agent_status: agent.agent_status,
                    worktree: None,
                })
                .pane_count += 1;
        }
        SessionSnapshot {
            version: "0.7.3".to_owned(),
            protocol: 16,
            focused_workspace_id: agents.first().map(|agent| agent.workspace_id.clone()),
            focused_tab_id: agents.first().map(|agent| agent.tab_id.clone()),
            focused_pane_id: agents.first().map(|agent| agent.pane_id.clone()),
            workspaces: workspaces.into_values().collect(),
            tabs: Vec::new(),
            panes: Vec::new(),
            layouts: Vec::new(),
            agents,
        }
    })
}

pub(crate) fn topology_events() -> impl Strategy<Value = Vec<AppEvent>> {
    prop::collection::vec(session_snapshot(), 1..=8).prop_map(|snapshots| {
        snapshots
            .into_iter()
            .flat_map(|snapshot| {
                let mut events = vec![AppEvent::SnapshotReplaced {
                    snapshot: snapshot.clone(),
                    observed_at: Timestamp::from_millis(1_000),
                    excluded_pane: None,
                }];
                if let Some(agent) = snapshot.agents.first() {
                    let pane_exit = AppEvent::PaneExited {
                        pane_id: PaneId::new(&agent.pane_id),
                        revision: agent.revision + 1,
                        occurred_at: Timestamp::from_millis(2_000),
                    };
                    events.push(pane_exit.clone());
                    events.push(pane_exit);
                }
                if let Some(workspace) = snapshot.workspaces.first() {
                    let workspace_closed =
                        AppEvent::WorkspaceClosed(WorkspaceId::new(&workspace.workspace_id));
                    events.push(workspace_closed.clone());
                    events.push(workspace_closed);
                }
                events.push(AppEvent::PaneExited {
                    pane_id: PaneId::new("property-missing-pane"),
                    revision: 1,
                    occurred_at: Timestamp::from_millis(3_000),
                });
                events.push(AppEvent::WorkspaceClosed(WorkspaceId::new(
                    "property-missing-workspace",
                )));
                events
            })
            .collect()
    })
}

pub(crate) fn guestbook_event() -> impl Strategy<Value = GuestbookEvent> {
    prop_oneof![
        Just(GuestbookEvent::AgentDetected),
        Just(GuestbookEvent::WorkStarted),
        Just(GuestbookEvent::WebmasterNeeded),
        Just(GuestbookEvent::WorkCompleted),
        Just(GuestbookEvent::AgentBecameIdle),
        Just(GuestbookEvent::PaneExited),
        Just(GuestbookEvent::PaneClosed),
    ]
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
