use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
};

use proptest::prelude::*;
use questmancer::{
    app::{CharacterSet, ColorMode, DisplayPreferences, Motion, View},
    domain::{
        AdventurerPersona, Agent, AgentKey, Campaign, Chronicle, ChronicleEvent, DomainState,
        GuildAttention, GuildSummons, PaneId, PersonaKey, Presence, TabId, Timestamp, WorkspaceId,
    },
    herdr::protocol::{AgentInfo, AgentSessionInfo, AgentStatus, SessionSnapshot, WorkspaceInfo},
    persistence::{AttentionEpisodeKey, PersistedStateV1, STATE_SCHEMA_VERSION},
    update::AppEvent,
};
use ratatui::layout::Rect;

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

pub(crate) fn guild_summons() -> impl Strategy<Value = GuildSummons> {
    prop_oneof![
        Just(GuildSummons::CounselRequested),
        Just(GuildSummons::SpoilsReturned),
        Just(GuildSummons::AdventurerDeparted),
    ]
}

pub(crate) fn persona() -> impl Strategy<Value = AdventurerPersona> {
    persona_key().prop_map(AdventurerPersona::for_key)
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

pub(crate) fn safe_rect() -> impl Strategy<Value = Rect> {
    (any::<u16>(), any::<u16>())
        .prop_flat_map(|(x, y)| (Just(x), Just(y), 0..=u16::MAX - x, 0..=u16::MAX - y))
        .prop_map(|(x, y, width, height)| Rect::new(x, y, width, height))
}

pub(crate) fn attention() -> impl Strategy<Value = GuildAttention> {
    prop_oneof![
        Just(GuildAttention::Clear),
        (guild_summons(), timestamp())
            .prop_map(|(summons, since)| GuildAttention::Unread { summons, since }),
        (guild_summons(), timestamp())
            .prop_map(|(summons, since)| GuildAttention::Read { summons, since }),
        (guild_summons(), timestamp(), timestamp()).prop_map(|(summons, since, until)| {
            GuildAttention::Deferred {
                summons,
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
        let mut campaigns = BTreeMap::new();
        campaigns.insert(
            workspace_id.clone(),
            Campaign {
                workspace_id,
                label: "site".to_owned(),
                cwd: PathBuf::from("/tmp/site"),
                party: vec![key.clone()],
            },
        );
        DomainState {
            campaigns,
            agents,
            selected_agent: Some(key),
            chronicle: Chronicle::default(),
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

pub(crate) fn chronicle_event() -> impl Strategy<Value = ChronicleEvent> {
    prop_oneof![
        Just(ChronicleEvent::AdventurerJoined),
        Just(ChronicleEvent::DelveBegan),
        Just(ChronicleEvent::CounselRequested),
        Just(ChronicleEvent::SpoilsReturned),
        Just(ChronicleEvent::AdventurerRested),
        Just(ChronicleEvent::AdventurerDeparted),
        Just(ChronicleEvent::CampaignClosed),
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
            let mut campaigns = BTreeMap::<WorkspaceId, Campaign>::new();
            for (key, agent) in &agents {
                campaigns
                    .entry(agent.workspace_id.clone())
                    .or_insert_with(|| Campaign {
                        workspace_id: agent.workspace_id.clone(),
                        label: format!("site-{}", agent.workspace_id),
                        cwd: PathBuf::from(format!("/tmp/{}", agent.workspace_id)),
                        party: Vec::new(),
                    })
                    .party
                    .push(key.clone());
            }
            DomainState {
                campaigns,
                agents,
                selected_agent,
                chronicle: Chronicle::default(),
            }
        })
}

pub(crate) fn guild_room_domain() -> impl Strategy<Value = DomainState> {
    prop::collection::vec(workspace_id(), 0..=6)
        .prop_flat_map(|workspace_ids| {
            let generated_agents = if workspace_ids.is_empty() {
                Just(Vec::<Agent>::new()).boxed()
            } else {
                prop::collection::vec(
                    (agent(), prop::sample::select(workspace_ids.clone())).prop_map(
                        |(mut agent, workspace_id)| {
                            agent.workspace_id = workspace_id;
                            agent
                        },
                    ),
                    0..=8,
                )
                .boxed()
            };
            (Just(workspace_ids), generated_agents)
        })
        .prop_map(|(workspace_ids, generated_agents)| {
            let agents = generated_agents
                .into_iter()
                .map(|agent| (agent.key.clone(), agent))
                .collect::<BTreeMap<_, _>>();
            let mut campaigns = workspace_ids
                .into_iter()
                .map(|workspace_id| {
                    (
                        workspace_id.clone(),
                        Campaign {
                            label: format!("site-{workspace_id}"),
                            cwd: PathBuf::from(format!("/tmp/{workspace_id}")),
                            workspace_id,
                            party: Vec::new(),
                        },
                    )
                })
                .collect::<BTreeMap<_, _>>();
            for (key, agent) in &agents {
                campaigns
                    .get_mut(&agent.workspace_id)
                    .expect("generated agents choose an existing campaign")
                    .party
                    .push(key.clone());
            }
            (campaigns, agents)
        })
        .prop_flat_map(|(campaigns, agents)| {
            let keys = agents.keys().cloned().collect::<Vec<_>>();
            let selection = if keys.is_empty() {
                Just(None).boxed()
            } else {
                prop_oneof![Just(None), prop::sample::select(keys).prop_map(Some)].boxed()
            };
            (Just(campaigns), Just(agents), selection)
        })
        .prop_map(|(campaigns, agents, selected_agent)| DomainState {
            campaigns,
            agents,
            selected_agent,
            chronicle: Chronicle::default(),
        })
}

fn view() -> impl Strategy<Value = View> {
    prop_oneof![Just(View::Guild), Just(View::Delve)]
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
                prop::collection::vec((prop::sample::select(keys), guild_summons()), 0..=6).boxed()
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
                    .map(|(persona, summons)| AttentionEpisodeKey { persona, summons })
                    .collect(),
            },
        )
}
