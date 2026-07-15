use crate::{
    domain::{
        Attention, AttentionReason, DomainState, GuestbookEntry, GuestbookEvent, PaneId, Presence,
        Timestamp, WorkspaceId,
    },
    herdr::protocol::AgentStatus,
};

use super::{AppEvent, Command};

#[must_use]
pub fn update(mut state: DomainState, event: AppEvent) -> (DomainState, Vec<Command>) {
    let commands = match event {
        AppEvent::SnapshotReplaced {
            snapshot,
            observed_at,
            excluded_pane,
        } => replace_snapshot(&mut state, &snapshot, observed_at, excluded_pane.as_ref()),
        AppEvent::AgentStatusChanged {
            pane_id,
            status,
            custom_status,
            revision,
            occurred_at,
        } => change_status(
            &mut state,
            &pane_id,
            status,
            custom_status,
            revision,
            occurred_at,
        ),
        AppEvent::PaneExited {
            pane_id,
            revision,
            occurred_at,
        } => exit_pane(&mut state, &pane_id, revision, occurred_at),
        AppEvent::WorkspaceClosed(workspace_id) => close_workspace(&mut state, &workspace_id),
        AppEvent::MarkSeen(agent_key) => mark_seen(&mut state, &agent_key),
    };
    (state, commands)
}

fn replace_snapshot(
    state: &mut DomainState,
    snapshot: &crate::herdr::protocol::SessionSnapshot,
    observed_at: Timestamp,
    excluded_pane: Option<&PaneId>,
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
    replacement.guestbook = state.guestbook.clone();
    *state = replacement;
    vec![Command::PersistState]
}

fn change_status(
    state: &mut DomainState,
    pane_id: &PaneId,
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
    if revision < agent.pane_revision
        || (revision == agent.pane_revision && next_presence == agent.presence)
    {
        return Vec::new();
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
    let (attention, event, summary) = match next_presence {
        Presence::Working => (
            Attention::Clear,
            GuestbookEvent::WorkStarted,
            "started updating",
        ),
        Presence::Blocked => (
            Attention::unseen(AttentionReason::NeedsInput, occurred_at),
            GuestbookEvent::WebmasterNeeded,
            "contacted the webmaster",
        ),
        Presence::Done => (
            Attention::unseen(AttentionReason::WorkCompleted, occurred_at),
            GuestbookEvent::WorkCompleted,
            "published a site update",
        ),
        Presence::Idle => (
            Attention::Clear,
            GuestbookEvent::AgentBecameIdle,
            "went idle",
        ),
        Presence::Exited => (
            Attention::unseen(AttentionReason::PaneExited, occurred_at),
            GuestbookEvent::PaneExited,
            "left a broken link",
        ),
        Presence::Unknown => (
            Attention::Clear,
            GuestbookEvent::AgentDetected,
            "status unknown",
        ),
    };
    agent.attention = attention;
    append_history(state, &key, event, summary, occurred_at)
}

fn exit_pane(
    state: &mut DomainState,
    pane_id: &PaneId,
    revision: u64,
    occurred_at: Timestamp,
) -> Vec<Command> {
    let Some(key) = state.agent_key_for_pane(pane_id).cloned() else {
        return vec![Command::RequestSnapshot];
    };
    let agent = state.agents.get_mut(&key).expect("agent key came from map");
    if revision < agent.pane_revision || agent.presence == Presence::Exited {
        return Vec::new();
    }
    agent.presence = Presence::Exited;
    agent.presence_since = occurred_at;
    agent.attention = Attention::unseen(AttentionReason::PaneExited, occurred_at);
    agent.pane_revision = revision;
    append_history(
        state,
        &key,
        GuestbookEvent::PaneExited,
        "left a broken link",
        occurred_at,
    )
}

fn append_history(
    state: &mut DomainState,
    key: &crate::domain::AgentKey,
    kind: GuestbookEvent,
    summary: &str,
    occurred_at: Timestamp,
) -> Vec<Command> {
    let agent = &state.agents[key];
    let entry = GuestbookEntry::new(
        occurred_at,
        Some(key.clone()),
        Some(agent.workspace_id.clone()),
        Some(agent.pane_id.clone()),
        agent.pane_revision,
        kind,
        format!("{} {summary}", agent.name),
    );
    if state.guestbook.append(entry.clone()) {
        vec![Command::AppendGuestbook(entry), Command::PersistState]
    } else {
        Vec::new()
    }
}

fn close_workspace(state: &mut DomainState, workspace_id: &WorkspaceId) -> Vec<Command> {
    if state.sites.remove(workspace_id).is_none() {
        return Vec::new();
    }
    state
        .agents
        .retain(|_, agent| &agent.workspace_id != workspace_id);
    if state
        .selected_agent
        .as_ref()
        .is_some_and(|key| !state.agents.contains_key(key))
    {
        state.selected_agent = state.agents.keys().next().cloned();
    }
    vec![Command::PersistState]
}

fn mark_seen(state: &mut DomainState, agent_key: &crate::domain::AgentKey) -> Vec<Command> {
    let Some(agent) = state.agents.get_mut(agent_key) else {
        return Vec::new();
    };
    if !agent.attention.is_unseen() {
        return Vec::new();
    }
    agent.attention = agent.attention.clone().mark_seen();
    vec![Command::PersistState]
}
