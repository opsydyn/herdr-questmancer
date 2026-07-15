use std::{collections::BTreeMap, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::herdr::protocol::{SessionSnapshot, WorkspaceInfo};

use super::{Agent, AgentKey, Guestbook, PaneId, Site, Timestamp, WorkspaceId};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct DomainState {
    pub sites: BTreeMap<WorkspaceId, Site>,
    pub agents: BTreeMap<AgentKey, Agent>,
    pub selected_agent: Option<AgentKey>,
    pub guestbook: Guestbook,
}

impl DomainState {
    #[must_use]
    pub fn from_snapshot(snapshot: &SessionSnapshot, observed_at: Timestamp) -> Self {
        Self::from_snapshot_excluding(snapshot, observed_at, None)
    }

    #[must_use]
    pub fn from_snapshot_excluding(
        snapshot: &SessionSnapshot,
        observed_at: Timestamp,
        excluded_pane: Option<&PaneId>,
    ) -> Self {
        let mut state = Self::default();
        let workspace_by_id = snapshot
            .workspaces
            .iter()
            .map(|workspace| (workspace.workspace_id.as_str(), workspace))
            .collect::<BTreeMap<_, _>>();

        for source in &snapshot.agents {
            if excluded_pane.is_some_and(|pane_id| pane_id.as_str() == source.pane_id) {
                continue;
            }
            let workspace = workspace_by_id.get(source.workspace_id.as_str()).copied();
            let root = workspace.and_then(workspace_root);
            let agent = Agent::from_snapshot(source, root, observed_at);
            state.agents.insert(agent.key.clone(), agent);
        }

        for workspace in &snapshot.workspaces {
            let workspace_id = WorkspaceId::new(&workspace.workspace_id);
            let mut agent_keys = state
                .agents
                .values()
                .filter(|agent| agent.workspace_id == workspace_id)
                .map(|agent| agent.key.clone())
                .collect::<Vec<_>>();
            agent_keys.sort();
            state.sites.insert(
                workspace_id.clone(),
                Site {
                    workspace_id,
                    label: workspace.label.clone(),
                    cwd: site_cwd(workspace),
                    agents: agent_keys,
                },
            );
        }

        state.selected_agent = snapshot
            .focused_pane_id
            .as_deref()
            .and_then(|pane_id| state.agent_key_for_pane(&PaneId::new(pane_id)))
            .cloned()
            .or_else(|| state.agents.keys().next().cloned());
        state
    }

    #[must_use]
    pub fn agent_key_for_pane(&self, pane_id: &PaneId) -> Option<&AgentKey> {
        self.agents
            .iter()
            .find_map(|(key, agent)| (&agent.pane_id == pane_id).then_some(key))
    }
}

fn workspace_root(workspace: &WorkspaceInfo) -> Option<&str> {
    workspace
        .worktree
        .as_ref()
        .map(|worktree| worktree.repo_root.as_str())
}

fn site_cwd(workspace: &WorkspaceInfo) -> PathBuf {
    workspace
        .worktree
        .as_ref()
        .map(|worktree| PathBuf::from(&worktree.checkout_path))
        .unwrap_or_default()
}
