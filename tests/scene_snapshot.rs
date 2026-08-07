use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

use questmancer::{
    app::{ConnectionState, DisplayPreferences, Model, Motion, OutputPreview, View},
    domain::{
        AdventurerPersona, Agent, AgentKey, Campaign, Chronicle, DomainState, GuildAttention,
        GuildSummons, PaneId, PersonaKey, Presence, TabId, Timestamp, WorkspaceId,
    },
    persistence::{PersistedStateV1, STATE_SCHEMA_VERSION},
    scene::snapshot::{SceneConnection, SceneSnapshot, SceneTransition},
};

fn agent(key: &str, workspace: &str, presence: Presence, focused: bool) -> Agent {
    Agent {
        key: AgentKey::new(key),
        pane_id: PaneId::new(format!("pane-{key}")),
        workspace_id: WorkspaceId::new(workspace),
        tab_id: TabId::new(format!("tab-{key}")),
        name: format!("Agent {key}"),
        custom_status: Some(format!("status-{key}")),
        presence,
        presence_since: Timestamp::from_millis(1_000),
        attention: GuildAttention::Clear,
        focused,
        pane_revision: 7,
        persona: AdventurerPersona::for_key(PersonaKey::new(format!("persona-{key}"))),
    }
}

fn model() -> Model {
    let alpha = agent("alpha", "workspace-b", Presence::Working, true);
    let beta = agent("beta", "workspace-a", Presence::Idle, false);
    let mut agents = BTreeMap::new();
    agents.insert(alpha.key.clone(), alpha);
    agents.insert(beta.key.clone(), beta);

    let mut campaigns = BTreeMap::new();
    campaigns.insert(
        WorkspaceId::new("workspace-b"),
        Campaign {
            workspace_id: WorkspaceId::new("workspace-b"),
            label: "Beta campaign".to_owned(),
            cwd: PathBuf::from("/tmp/beta"),
            party: vec![AgentKey::new("alpha")],
        },
    );
    campaigns.insert(
        WorkspaceId::new("workspace-a"),
        Campaign {
            workspace_id: WorkspaceId::new("workspace-a"),
            label: "Alpha campaign".to_owned(),
            cwd: PathBuf::from("/tmp/alpha"),
            party: vec![AgentKey::new("beta")],
        },
    );

    let mut model = Model::new(View::Guild);
    model.set_connection(ConnectionState::Reconnecting { attempt: 4 });
    model.set_now(Timestamp::from_millis(9_000));
    model.set_preferences(DisplayPreferences {
        motion: Motion::Reduced,
        ..DisplayPreferences::default()
    });
    model.replace_domain(DomainState {
        campaigns,
        agents,
        selected_agent: Some(AgentKey::new("alpha")),
        chronicle: Chronicle::default(),
    });
    model
}

#[test]
fn snapshot_ignores_legacy_ui_persistence_and_goblin_state() {
    let baseline = model();
    let mut changed = baseline.clone();

    changed.switch_to(View::Delve);
    changed.domain_mut().selected_agent = Some(AgentKey::new("beta"));
    changed.toggle_ledger();
    changed.set_output_preview(Some(OutputPreview {
        pane_id: PaneId::new("pane-alpha"),
        revision: 99,
        text: "preview".to_owned(),
        loading: false,
        error: None,
    }));
    changed.set_action_feedback("notice".to_owned());
    changed.set_reviewr_available(true);
    changed.goblins_mut().release(Timestamp::from_millis(8_000));
    changed
        .durable_intent_mut()
        .seed(&PersistedStateV1 {
            schema_version: STATE_SCHEMA_VERSION,
            last_view: View::Delve,
            preferences: DisplayPreferences::default(),
            selected_persona: None,
            personas: BTreeMap::new(),
            seen_attention: BTreeSet::default(),
        })
        .unwrap();

    assert_eq!(
        SceneSnapshot::from_model(&baseline),
        SceneSnapshot::from_model(&changed)
    );
}

#[test]
fn snapshot_changes_for_each_herdr_reported_scene_fact() {
    let baseline = model();
    let baseline_snapshot = SceneSnapshot::from_model(&baseline);

    for mutate in [
        (|agent: &mut Agent| agent.focused = !agent.focused) as fn(&mut Agent),
        |agent| agent.presence = Presence::Blocked,
        |agent| agent.presence_since = Timestamp::from_millis(2_000),
        |agent| agent.custom_status = Some("different".to_owned()),
    ] {
        let mut changed = baseline.clone();
        mutate(
            changed
                .domain_mut()
                .agents
                .get_mut(&AgentKey::new("alpha"))
                .unwrap(),
        );
        assert_ne!(baseline_snapshot, SceneSnapshot::from_model(&changed));
    }
}

#[test]
fn snapshot_preserves_connection_details_motion_time_and_stable_ordering() {
    let mut model = model();
    model.set_connection(ConnectionState::Incompatible {
        expected: 17,
        actual: 12,
    });
    let snapshot = SceneSnapshot::from_model(&model);

    assert_eq!(
        snapshot.connection,
        SceneConnection::Incompatible {
            expected: 17,
            actual: 12,
        }
    );
    assert_eq!(snapshot.motion, Motion::Reduced);
    assert_eq!(snapshot.now, Timestamp::from_millis(9_000));
    assert_eq!(
        snapshot
            .campaigns
            .iter()
            .map(|campaign| campaign.workspace_id.as_str())
            .collect::<Vec<_>>(),
        ["workspace-a", "workspace-b"]
    );
    assert_eq!(
        snapshot
            .agents
            .iter()
            .map(|agent| agent.key.as_str())
            .collect::<Vec<_>>(),
        ["alpha", "beta"]
    );
    assert_ne!(
        snapshot.campaigns[0].variant_seed,
        snapshot.campaigns[1].variant_seed
    );
    assert_eq!(snapshot, SceneSnapshot::from_model(&model));
}

#[test]
fn snapshot_preserves_every_connection_variant() {
    let cases = [
        (ConnectionState::Offline, SceneConnection::Offline),
        (ConnectionState::Connecting, SceneConnection::Connecting),
        (ConnectionState::Connected, SceneConnection::Connected),
        (
            ConnectionState::Reconnecting { attempt: 9 },
            SceneConnection::Reconnecting { attempt: 9 },
        ),
        (
            ConnectionState::Incompatible {
                expected: 20,
                actual: 19,
            },
            SceneConnection::Incompatible {
                expected: 20,
                actual: 19,
            },
        ),
    ];

    for (connection, expected) in cases {
        let mut model = model();
        model.set_connection(connection);
        assert_eq!(SceneSnapshot::from_model(&model).connection, expected);
    }
}

#[test]
fn snapshot_collapses_attention_read_state_and_deferred_until() {
    let transition = SceneTransition {
        summons: GuildSummons::SpoilsReturned,
        since: Timestamp::from_millis(2_000),
    };
    let attention_variants = [
        GuildAttention::Unread {
            summons: transition.summons,
            since: transition.since,
        },
        GuildAttention::Read {
            summons: transition.summons,
            since: transition.since,
        },
        GuildAttention::Deferred {
            summons: transition.summons,
            since: transition.since,
            until: Timestamp::from_millis(99_000),
        },
    ];

    let snapshots = attention_variants.map(|attention| {
        let mut model = model();
        model
            .domain_mut()
            .agents
            .get_mut(&AgentKey::new("alpha"))
            .unwrap()
            .attention = attention;
        SceneSnapshot::from_model(&model)
    });

    assert_eq!(snapshots[0], snapshots[1]);
    assert_eq!(snapshots[1], snapshots[2]);
    assert_eq!(snapshots[0].agents[0].transition, Some(transition));
}
