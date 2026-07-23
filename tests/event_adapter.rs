use questmancer::{
    app::ConnectionState,
    domain::{DomainState, PaneId, Timestamp, WorkspaceId},
    herdr::{
        event_adapter::{AdapterAction, adapt_update, adapt_update_excluding},
        protocol::{
            AgentStatus, SessionSnapshot, SessionSnapshotResult, SuccessResponse, WireEvent,
        },
        supervisor::ConnectionUpdate,
    },
    update::AppEvent,
};
use serde_json::json;

fn snapshot() -> SessionSnapshot {
    let response: SuccessResponse<SessionSnapshotResult> =
        serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
    response.result.snapshot
}

fn state() -> DomainState {
    DomainState::from_snapshot(&snapshot(), Timestamp::from_millis(1_000))
}

#[test]
fn connected_update_sets_connection_and_replaces_snapshot() {
    let actions = adapt_update(
        ConnectionUpdate::Connected(snapshot()),
        &DomainState::default(),
        Timestamp::from_millis(2_000),
    );

    assert_eq!(
        actions[0],
        AdapterAction::SetConnection(ConnectionState::Connected)
    );
    assert!(matches!(
        &actions[1],
        AdapterAction::Apply(event)
            if matches!(event.as_ref(), AppEvent::SnapshotReplaced { observed_at, .. }
                if *observed_at == Timestamp::from_millis(2_000))
    ));
}

#[test]
fn connected_update_carries_managed_pane_exclusion() {
    let managed = PaneId::new("w2:p3");
    let actions = adapt_update_excluding(
        ConnectionUpdate::Connected(snapshot()),
        &DomainState::default(),
        Timestamp::from_millis(2_000),
        Some(&managed),
    );
    assert!(matches!(
        &actions[1],
        AdapterAction::Apply(event)
            if matches!(event.as_ref(), AppEvent::SnapshotReplaced {
                excluded_pane: Some(pane), ..
            } if pane == &managed)
    ));
}

#[test]
fn dotted_agent_status_becomes_a_typed_transition() {
    let update = ConnectionUpdate::Event(WireEvent {
        event: "pane.agent_status_changed".into(),
        data: json!({
            "pane_id": "w1:p1",
            "workspace_id": "w1",
            "agent_status": "done",
            "custom_status": "ready to publish"
        }),
    });

    let actions = adapt_update(update, &state(), Timestamp::from_millis(3_000));

    assert_eq!(actions.len(), 1);
    assert!(matches!(
        &actions[0],
        AdapterAction::Apply(event) if matches!(event.as_ref(), AppEvent::AgentStatusChanged {
            pane_id,
            status: AgentStatus::Done,
            custom_status: Some(message),
            revision: 8,
            occurred_at,
        } if *pane_id == PaneId::new("w1:p1")
            && message == "ready to publish"
            && *occurred_at == Timestamp::from_millis(3_000))
    ));
}

#[test]
fn revisionless_duplicate_status_and_custom_status_are_inert() {
    let actions = adapt_update(
        ConnectionUpdate::Event(WireEvent {
            event: "pane.agent_status_changed".into(),
            data: json!({
                "pane_id": "w1:p1",
                "workspace_id": "w1",
                "agent_status": "blocked",
                "custom_status": "which schema?"
            }),
        }),
        &state(),
        Timestamp::from_millis(3_000),
    );

    assert!(actions.is_empty());
}

#[test]
fn explicit_stale_revision_is_preserved_for_the_domain_reducer() {
    let actions = adapt_update(
        ConnectionUpdate::Event(WireEvent {
            event: "pane.agent_status_changed".into(),
            data: json!({
                "pane_id": "w1:p1",
                "workspace_id": "w1",
                "agent_status": "done",
                "revision": 6
            }),
        }),
        &state(),
        Timestamp::from_millis(3_000),
    );

    assert!(matches!(
        &actions[0],
        AdapterAction::Apply(event) if matches!(event.as_ref(), AppEvent::AgentStatusChanged {
            revision: 6,
            ..
        })
    ));
}

#[test]
fn lifecycle_workspace_close_becomes_a_domain_event() {
    let actions = adapt_update(
        ConnectionUpdate::Event(WireEvent {
            event: "workspace_closed".into(),
            data: json!({"type": "workspace_closed", "workspace_id": "w1"}),
        }),
        &state(),
        Timestamp::from_millis(2_000),
    );

    assert_eq!(
        actions,
        vec![AdapterAction::Apply(Box::new(AppEvent::WorkspaceClosed(
            WorkspaceId::new("w1")
        )))]
    );
}

#[test]
fn pane_exit_uses_the_current_revision_boundary() {
    let actions = adapt_update(
        ConnectionUpdate::Event(WireEvent {
            event: "pane_exited".into(),
            data: json!({"type": "pane_exited", "pane_id": "w1:p1", "exit_code": 1}),
        }),
        &state(),
        Timestamp::from_millis(4_000),
    );

    assert!(matches!(
        &actions[0],
        AdapterAction::Apply(event) if matches!(event.as_ref(), AppEvent::PaneExited {
            pane_id,
            revision: 8,
            ..
        } if *pane_id == PaneId::new("w1:p1"))
    ));
}

#[test]
fn incomplete_topology_event_requests_a_snapshot() {
    let actions = adapt_update(
        ConnectionUpdate::Event(WireEvent {
            event: "pane_created".into(),
            data: json!({"type": "pane_created"}),
        }),
        &state(),
        Timestamp::from_millis(2_000),
    );

    assert_eq!(actions, vec![AdapterAction::RequestSnapshot]);
}

#[test]
fn unknown_event_is_a_non_blocking_diagnostic() {
    let actions = adapt_update(
        ConnectionUpdate::Event(WireEvent {
            event: "future_event".into(),
            data: json!({"answer": 42}),
        }),
        &state(),
        Timestamp::from_millis(2_000),
    );

    assert!(matches!(
        &actions[0],
        AdapterAction::Diagnostic(message) if message.contains("future_event")
    ));
}

#[test]
fn metadata_updates_are_inert_display_refreshes() {
    for event in ["pane.updated", "workspace.metadata_updated"] {
        let actions = adapt_update(
            ConnectionUpdate::Event(WireEvent {
                event: event.into(),
                data: json!({"source": "plugin:opsydyn.questmancer"}),
            }),
            &state(),
            Timestamp::from_millis(2_000),
        );

        assert!(actions.is_empty(), "{event} should not perturb the model");
    }
}

#[test]
fn reconnect_updates_remain_app_state_not_domain_state() {
    let actions = adapt_update(
        ConnectionUpdate::Reconnecting {
            attempt: 3,
            delay: std::time::Duration::from_secs(1),
        },
        &state(),
        Timestamp::from_millis(2_000),
    );

    assert_eq!(
        actions,
        vec![AdapterAction::SetConnection(
            ConnectionState::Reconnecting { attempt: 3 }
        )]
    );
}

#[test]
fn managed_status_and_exit_events_are_ignored() {
    let managed = PaneId::new("w2:p3");
    for event in [
        WireEvent {
            event: "pane.agent_status_changed".into(),
            data: json!({"pane_id": "w2:p3", "agent_status": "blocked"}),
        },
        WireEvent {
            event: "pane_exited".into(),
            data: json!({"pane_id": "w2:p3"}),
        },
    ] {
        assert!(
            adapt_update_excluding(
                ConnectionUpdate::Event(event),
                &state(),
                Timestamp::from_millis(2_000),
                Some(&managed),
            )
            .is_empty()
        );
    }
}

#[test]
fn unknown_non_managed_pane_still_requests_snapshot() {
    let managed = PaneId::new("w2:p3");
    let actions = adapt_update_excluding(
        ConnectionUpdate::Event(WireEvent {
            event: "pane.agent_status_changed".into(),
            data: json!({"pane_id": "w9:p9", "agent_status": "blocked"}),
        }),
        &state(),
        Timestamp::from_millis(2_000),
        Some(&managed),
    );
    assert_eq!(actions, vec![AdapterAction::RequestSnapshot]);
}
