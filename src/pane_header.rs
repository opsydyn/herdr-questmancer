//! Pure, ephemeral annotations derived from qualified live adventurer identities.
use std::collections::BTreeMap;

use crate::{
    app::{ConnectionState, Model},
    domain::{CaptureIdentity, DomainState, PaneId, Presence},
    herdr::protocol::AgentSessionInfo,
    scene::assets::adventurer::adventurer_roster_frame,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaneHeaderProjection {
    pub pane_id: PaneId,
    pub terminal_id: String,
    pub agent_session: AgentSessionInfo,
    pub label: String,
    /// Authored, personalised 8x12 RGBA pixels. No scaling or terminal escape sequences.
    pub rgba: Vec<u8>,
}

impl PaneHeaderProjection {
    #[must_use]
    pub fn from_model(model: &Model) -> Vec<Self> {
        Self::from_domain(
            model.domain(),
            model.managed_pane_id(),
            *model.connection() == ConnectionState::Connected,
        )
    }

    #[must_use]
    pub fn from_domain(
        domain: &DomainState,
        managed: Option<&PaneId>,
        connected: bool,
    ) -> Vec<Self> {
        if !connected {
            return Vec::new();
        }
        let mut counts = BTreeMap::<&PaneId, usize>::new();
        for agent in domain.agents.values() {
            *counts.entry(&agent.pane_id).or_default() += 1;
        }
        domain
            .agents
            .values()
            .filter_map(|agent| {
                if managed == Some(&agent.pane_id)
                    || counts[&agent.pane_id] != 1
                    || agent.presence == Presence::Exited
                {
                    return None;
                }
                let CaptureIdentity::Session { terminal, session } = &agent.capture_identity else {
                    return None;
                };
                let name = agent.name.split_whitespace().collect::<Vec<_>>().join(" ");
                let name = name
                    .chars()
                    .filter(|c| !c.is_control())
                    .take(80)
                    .collect::<String>();
                if name.is_empty() {
                    return None;
                }
                let condition = match agent.presence {
                    Presence::Working => "working",
                    Presence::Blocked => "seeks counsel",
                    Presence::Done => "spoils returned",
                    Presence::Idle => "resting",
                    Presence::Unknown => "whereabouts unknown",
                    Presence::Exited => return None,
                };
                let sprite = adventurer_roster_frame(&agent.persona);
                let rgba = sprite
                    .pixels()
                    .iter()
                    .flat_map(|pixel| pixel.map_or([0, 0, 0, 0], |rgb| [rgb.r, rgb.g, rgb.b, 255]))
                    .collect();
                Some(Self {
                    pane_id: agent.pane_id.clone(),
                    terminal_id: terminal.clone(),
                    agent_session: session.clone(),
                    label: format!("{name} · {:?} · {condition}", agent.persona.class),
                    rgba,
                })
            })
            .collect()
    }
}
