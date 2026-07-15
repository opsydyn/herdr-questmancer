use serde::Deserialize;

use crate::{
    app::ConnectionState,
    domain::{DomainState, PaneId, Timestamp, WorkspaceId},
    update::AppEvent,
};

use super::{
    protocol::{AgentStatus, WireEvent},
    supervisor::ConnectionUpdate,
};

#[derive(Clone, Debug, PartialEq)]
pub enum AdapterAction {
    Apply(Box<AppEvent>),
    SetConnection(ConnectionState),
    RequestSnapshot,
    Diagnostic(String),
}

#[must_use]
pub fn adapt_update(
    update: ConnectionUpdate,
    state: &DomainState,
    observed_at: Timestamp,
) -> Vec<AdapterAction> {
    match update {
        ConnectionUpdate::Connected(snapshot) => vec![
            AdapterAction::SetConnection(ConnectionState::Connected),
            AdapterAction::Apply(Box::new(AppEvent::SnapshotReplaced {
                snapshot,
                observed_at,
            })),
        ],
        ConnectionUpdate::Event(event) => adapt_wire_event(event, state, observed_at),
        ConnectionUpdate::Disconnected(message) => vec![
            AdapterAction::SetConnection(ConnectionState::Offline),
            AdapterAction::Diagnostic(message),
        ],
        ConnectionUpdate::Reconnecting { attempt, .. } => vec![AdapterAction::SetConnection(
            ConnectionState::Reconnecting { attempt },
        )],
        ConnectionUpdate::Resyncing => vec![
            AdapterAction::SetConnection(ConnectionState::Connecting),
            AdapterAction::RequestSnapshot,
        ],
        ConnectionUpdate::Incompatible { expected, actual } => vec![AdapterAction::SetConnection(
            ConnectionState::Incompatible { expected, actual },
        )],
    }
}

fn adapt_wire_event(
    event: WireEvent,
    state: &DomainState,
    observed_at: Timestamp,
) -> Vec<AdapterAction> {
    match event.event.as_str() {
        "pane.agent_status_changed" | "pane_agent_status_changed" => {
            adapt_agent_status(event, state, observed_at)
        }
        "workspace_closed" | "workspace.closed" => adapt_workspace_closed(event),
        "pane_exited" | "pane.exited" => adapt_pane_exited(event, state, observed_at),
        "workspace_created"
        | "workspace_updated"
        | "workspace_renamed"
        | "workspace_moved"
        | "worktree_created"
        | "worktree_opened"
        | "worktree_removed"
        | "tab_created"
        | "tab_closed"
        | "tab_renamed"
        | "tab_moved"
        | "pane_created"
        | "pane_closed"
        | "pane_moved"
        | "pane_agent_detected"
        | "layout_updated" => vec![AdapterAction::RequestSnapshot],
        "workspace_focused" | "tab_focused" | "pane_focused" => Vec::new(),
        unknown => vec![AdapterAction::Diagnostic(format!(
            "ignored unknown Herdr event {unknown:?}"
        ))],
    }
}

fn adapt_agent_status(
    event: WireEvent,
    state: &DomainState,
    observed_at: Timestamp,
) -> Vec<AdapterAction> {
    let Ok(data) = serde_json::from_value::<AgentStatusData>(event.data) else {
        return vec![AdapterAction::RequestSnapshot];
    };
    let pane_id = PaneId::new(data.pane_id);
    let Some(agent_key) = state.agent_key_for_pane(&pane_id) else {
        return vec![AdapterAction::RequestSnapshot];
    };
    let revision = data
        .revision
        .unwrap_or_else(|| state.agents[agent_key].pane_revision.saturating_add(1));
    vec![AdapterAction::Apply(Box::new(
        AppEvent::AgentStatusChanged {
            pane_id,
            status: data.agent_status,
            custom_status: data.custom_status,
            revision,
            occurred_at: observed_at,
        },
    ))]
}

fn adapt_workspace_closed(event: WireEvent) -> Vec<AdapterAction> {
    let Ok(data) = serde_json::from_value::<WorkspaceData>(event.data) else {
        return vec![AdapterAction::RequestSnapshot];
    };
    vec![AdapterAction::Apply(Box::new(AppEvent::WorkspaceClosed(
        WorkspaceId::new(data.workspace_id),
    )))]
}

fn adapt_pane_exited(
    event: WireEvent,
    state: &DomainState,
    observed_at: Timestamp,
) -> Vec<AdapterAction> {
    let Ok(data) = serde_json::from_value::<PaneData>(event.data) else {
        return vec![AdapterAction::RequestSnapshot];
    };
    let pane_id = PaneId::new(data.pane_id);
    let Some(agent_key) = state.agent_key_for_pane(&pane_id) else {
        return vec![AdapterAction::RequestSnapshot];
    };
    vec![AdapterAction::Apply(Box::new(AppEvent::PaneExited {
        pane_id,
        revision: state.agents[agent_key].pane_revision.saturating_add(1),
        occurred_at: observed_at,
    }))]
}

#[derive(Debug, Deserialize)]
struct AgentStatusData {
    pane_id: String,
    agent_status: AgentStatus,
    #[serde(default)]
    custom_status: Option<String>,
    #[serde(default)]
    revision: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct WorkspaceData {
    workspace_id: String,
}

#[derive(Debug, Deserialize)]
struct PaneData {
    pane_id: String,
}
