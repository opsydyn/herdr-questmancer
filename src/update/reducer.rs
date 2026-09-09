use crate::{
    domain::{
        CaptureIdentity, DomainState, GuildAttention, GuildSummons, ObservationEvidence,
        ObservationSubject, ObservedPresence, PaneId, Presence, Timestamp, WorkspaceId,
    },
    herdr::protocol::AgentStatus,
};

use super::{AppEvent, Command};

#[must_use]
pub fn update(mut state: DomainState, event: AppEvent) -> (DomainState, Vec<Command>) {
    let commands = match event {
        AppEvent::SnapshotReplaced {
            purpose,
            snapshot,
            observed_at,
            excluded_pane,
        } => replace_snapshot(
            &mut state,
            &snapshot,
            observed_at,
            excluded_pane.as_ref(),
            purpose,
        ),
        AppEvent::AgentStatusChanged {
            identity,
            pane_id,
            status,
            custom_status,
            revision,
            occurred_at,
        } => change_status(
            &mut state,
            &pane_id,
            &identity,
            status,
            custom_status,
            revision,
            occurred_at,
        ),
        AppEvent::PaneExited {
            identity,
            pane_id,
            revision,
            occurred_at,
        } => exit_pane(&mut state, &pane_id, &identity, revision, occurred_at),
        AppEvent::WorkspaceCloseHint(workspace_id) => close_workspace(&mut state, &workspace_id),
        AppEvent::MarkRead(agent_key) => mark_read(&mut state, &agent_key),
        AppEvent::DeferSummons { agent_key, until } => defer_summons(&mut state, &agent_key, until),
    };
    (state, commands)
}

fn replace_snapshot(
    state: &mut DomainState,
    snapshot: &crate::herdr::protocol::SessionSnapshot,
    observed_at: Timestamp,
    excluded_pane: Option<&PaneId>,
    purpose: crate::snapshot_refresh::SnapshotPurpose,
) -> Vec<Command> {
    let mut replacement =
        DomainState::from_snapshot_excluding(snapshot, observed_at, excluded_pane);
    for (key, agent) in &mut replacement.agents {
        if let Some(previous) = state.agents.get(key) {
            agent.persona = previous.persona.clone();
            if agent.presence == previous.presence {
                agent.attention = previous.attention.clone();
                agent.presence_since = previous.presence_since;
            }
        }
    }
    if state
        .selected_agent
        .as_ref()
        .is_some_and(|key| replacement.agents.contains_key(key))
    {
        replacement.selected_agent.clone_from(&state.selected_agent);
    }
    replacement.chronicle = state.chronicle.clone();
    replacement.capture = state.capture.clone();
    let mut commands = match purpose {
        crate::snapshot_refresh::SnapshotPurpose::Baseline => Vec::new(),
        crate::snapshot_refresh::SnapshotPurpose::Refresh(request) => {
            super::capture::capture_snapshot(state, &mut replacement, request, observed_at)
        }
    };
    replacement.capture.clear_hints();
    *state = replacement;
    commands.push(Command::PersistState);
    commands
}

fn change_status(
    state: &mut DomainState,
    pane_id: &PaneId,
    identity: &CaptureIdentity,
    status: AgentStatus,
    custom_status: Option<String>,
    revision: u64,
    occurred_at: Timestamp,
) -> Vec<Command> {
    let Some(key) = state.agent_key_for_pane(pane_id).cloned() else {
        return vec![Command::RequestSnapshot];
    };
    let next_presence = Presence::from(status);
    let agent = state.agents.get_mut(&key).expect("agent key came from map");
    if &agent.capture_identity != identity {
        return vec![Command::RequestSnapshot];
    }
    if revision < agent.pane_revision
        || (revision == agent.pane_revision && next_presence == agent.presence)
    {
        return Vec::new();
    }
    if revision == agent.pane_revision {
        // Equal revision and different status is conflicting evidence, not a
        // newer transition. Let a qualified snapshot resolve it.
        return vec![Command::RequestSnapshot];
    }
    if next_presence == agent.presence {
        agent.pane_revision = revision;
        agent.custom_status = custom_status;
        return vec![Command::PersistState];
    }

    agent.presence = next_presence;
    agent.presence_since = occurred_at;
    agent.pane_revision = revision;
    agent.custom_status = custom_status;
    agent.attention = match next_presence {
        Presence::Blocked => GuildAttention::unread(GuildSummons::CounselRequested, occurred_at),
        Presence::Done => GuildAttention::unread(GuildSummons::SpoilsReturned, occurred_at),
        Presence::Exited => GuildAttention::unread(GuildSummons::AdventurerDeparted, occurred_at),
        Presence::Working | Presence::Idle | Presence::Unknown => GuildAttention::Clear,
    };
    let subject = ObservationSubject::adventurer(agent);
    let mut commands = Vec::new();
    if let Some(subject) = subject
        && let Some(presence) = ObservedPresence::from_presence(next_presence)
        && let Some(command) = super::capture::append_observation(
            state,
            subject,
            ObservationEvidence::PaneMetadata { presence },
            occurred_at,
        )
    {
        commands.push(command);
    }
    commands.push(Command::PersistState);
    commands
}

fn exit_pane(
    state: &mut DomainState,
    pane_id: &PaneId,
    identity: &CaptureIdentity,
    revision: u64,
    occurred_at: Timestamp,
) -> Vec<Command> {
    let Some(key) = state.agent_key_for_pane(pane_id).cloned() else {
        return vec![Command::RequestSnapshot];
    };
    let agent = state.agents.get_mut(&key).expect("agent key came from map");
    if !agent.capture_identity.same_incarnation(identity) {
        return vec![Command::RequestSnapshot];
    }
    if revision <= agent.pane_revision || agent.presence == Presence::Exited {
        return Vec::new();
    }
    agent.presence = Presence::Exited;
    agent.presence_since = occurred_at;
    agent.attention = GuildAttention::unread(GuildSummons::AdventurerDeparted, occurred_at);
    agent.pane_revision = revision;
    let subject = ObservationSubject::adventurer(agent).expect("exit identity was qualified");
    let mut commands = Vec::new();
    if let Some(command) = super::capture::append_observation(
        state,
        subject,
        ObservationEvidence::LifecycleDeparture,
        occurred_at,
    ) {
        commands.push(command);
    }
    commands.push(Command::PersistState);
    commands
}

fn close_workspace(state: &mut DomainState, workspace_id: &WorkspaceId) -> Vec<Command> {
    if !state.campaigns.contains_key(workspace_id) {
        return Vec::new();
    }
    state.capture.note_close(workspace_id.clone());
    vec![Command::RequestSnapshot]
}

/// Sets an adventurer's summons aside until a chosen moment.
///
/// Deferring is only meaningful where a summons exists; an adventurer with
/// nothing to answer for cannot be snoozed, and saying so beats writing a
/// deferral nobody asked for.
fn defer_summons(
    state: &mut DomainState,
    agent_key: &crate::domain::AgentKey,
    until: Timestamp,
) -> Vec<Command> {
    let Some(agent) = state.agents.get_mut(agent_key) else {
        return Vec::new();
    };
    if agent.attention.summons().is_none() {
        return Vec::new();
    }
    agent.attention = agent.attention.clone().defer_until(until);
    vec![Command::PersistState]
}

fn mark_read(state: &mut DomainState, agent_key: &crate::domain::AgentKey) -> Vec<Command> {
    let Some(agent) = state.agents.get_mut(agent_key) else {
        return Vec::new();
    };
    if !agent.attention.is_unread() {
        return Vec::new();
    }
    agent.attention = agent.attention.clone().mark_read();
    vec![Command::PersistState]
}
