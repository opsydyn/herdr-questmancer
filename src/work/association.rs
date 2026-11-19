//! Ephemeral Herdr occupant evidence for explicit user associations.

use crate::{
    app::{ConnectionState, Model},
    domain::{Agent, AgentKey, CaptureIdentity, DomainState, PaneId, Presence},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdventurerLink {
    adventurer: AgentKey,
    pane: PaneId,
    identity: CaptureIdentity,
}

impl AdventurerLink {
    pub(crate) fn matches(&self, pane: &PaneId, identity: &CaptureIdentity) -> bool {
        &self.pane == pane && &self.identity == identity
    }

    pub(crate) fn selected(model: &Model) -> Option<Self> {
        let agent = model.selected_agent()?;
        if model.connection() != &ConnectionState::Connected
            || !agent.capture_identity.is_qualified()
        {
            return None;
        }
        let link = Self {
            adventurer: agent.key.clone(),
            pane: agent.pane_id.clone(),
            identity: agent.capture_identity.clone(),
        };
        link.resolve(model.domain(), model.managed_pane_id())?;
        Some(link)
    }

    pub(crate) fn resolve<'a>(
        &self,
        domain: &'a DomainState,
        managed_pane: Option<&PaneId>,
    ) -> Option<&'a Agent> {
        let agent = domain.agents.get(&self.adventurer)?;
        (managed_pane != Some(&agent.pane_id)
            && agent.presence != Presence::Exited
            && agent.pane_id == self.pane
            && agent.capture_identity == self.identity
            && !domain
                .agents
                .values()
                .any(|other| other.key != self.adventurer && other.pane_id == self.pane))
        .then_some(agent)
    }
}
