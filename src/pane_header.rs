//! Pure projection for the compact viewport line inside an adventurer pane.

use crate::{domain::PaneId, sidebar::SidebarProjection};

pub const PANE_HEADER_LAYER: &str = "questmancer-pane-header";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaneHeaderProjection {
    pub pane_id: PaneId,
    pub label: String,
}

impl PaneHeaderProjection {
    #[must_use]
    pub fn from_sidebar(
        sidebar: &SidebarProjection,
        managed_pane_id: Option<&PaneId>,
    ) -> Vec<Self> {
        sidebar
            .agents
            .iter()
            .filter(|agent| managed_pane_id != Some(&agent.pane_id))
            .filter_map(|agent| {
                let role = agent.tokens.get("quest_role")?;
                let condition = agent.tokens.get("quest_condition")?;
                let name = agent.tokens.get("quest_epithet")?;
                Some(Self {
                    pane_id: agent.pane_id.clone(),
                    label: format!("⌖ {name} · {role} · {condition}"),
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::sidebar::SidebarAgentTokens;

    #[test]
    fn excludes_managed_pane_and_formats_truthful_fields() {
        let managed = PaneId::new("quest-pane");
        let sidebar = SidebarProjection {
            agents: vec![
                SidebarAgentTokens {
                    pane_id: managed.clone(),
                    tokens: BTreeMap::from([
                        ("quest_epithet".into(), "Guild UI".into()),
                        ("quest_role".into(), "Mage Human".into()),
                        ("quest_condition".into(), "Resting".into()),
                    ]),
                },
                SidebarAgentTokens {
                    pane_id: PaneId::new("agent-pane"),
                    tokens: BTreeMap::from([
                        ("quest_epithet".into(), "Ari".into()),
                        ("quest_role".into(), "Pathseeker Human".into()),
                        ("quest_condition".into(), "Concentrating".into()),
                    ]),
                },
            ],
            campaigns: Vec::new(),
        };
        assert_eq!(
            PaneHeaderProjection::from_sidebar(&sidebar, Some(&managed)),
            vec![PaneHeaderProjection {
                pane_id: PaneId::new("agent-pane"),
                label: "⌖ Ari · Pathseeker Human · Concentrating".into(),
            }]
        );
    }
}
