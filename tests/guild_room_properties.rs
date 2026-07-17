#[allow(dead_code)]
mod support;

use std::collections::BTreeMap;

use proptest::prelude::*;
use questmancer::{
    app::{Model, View},
    domain::{AgentKey, DomainState, GuildAttention, GuildSummons, Presence, WorkspaceId},
    ui::guild_room_projection::{
        AdventurerRepresentation, GuildLandmark, GuildRoomProjection, project,
    },
};
use ratatui::layout::Rect;

fn model_with(domain: DomainState) -> Model {
    let mut model = Model::new(View::Guild);
    model.replace_domain(domain);
    model
}

fn represented_agent(representation: &AdventurerRepresentation) -> &AgentKey {
    match representation {
        AdventurerRepresentation::Physical { agent, .. }
        | AdventurerRepresentation::Token { agent, .. }
        | AdventurerRepresentation::Projection { agent, .. } => agent,
    }
}

fn stable_campaign_identity(
    projection: &GuildRoomProjection,
) -> Vec<(WorkspaceId, String, u64, Rect)> {
    projection
        .campaigns
        .iter()
        .map(|campaign| {
            (
                campaign.workspace_id.clone(),
                campaign.label.clone(),
                campaign.seal,
                campaign.area,
            )
        })
        .collect()
}

proptest! {
    #[test]
    fn every_non_exited_adventurer_appears_exactly_once(
        domain in support::strategies::guild_room_domain(),
        area in support::strategies::safe_rect(),
    ) {
        let model = model_with(domain.clone());
        let projection = project(&model, area);
        let mut counts = BTreeMap::<AgentKey, usize>::new();
        for representation in &projection.adventurers {
            *counts.entry(represented_agent(representation).clone()).or_default() += 1;
        }
        let expected_slots = domain
            .agents
            .values()
            .filter(|agent| agent.presence != Presence::Exited)
            .map(|agent| agent.key.clone())
            .collect::<Vec<_>>();
        let projected_slots = projection
            .adventurers
            .iter()
            .map(represented_agent)
            .cloned()
            .collect::<Vec<_>>();

        for agent in domain.agents.values() {
            let expected = usize::from(agent.presence != Presence::Exited);
            prop_assert_eq!(counts.get(&agent.key).copied().unwrap_or_default(), expected);
        }
        prop_assert_eq!(
            counts.len(),
            domain
                .agents
                .values()
                .filter(|agent| agent.presence != Presence::Exited)
                .count()
        );
        prop_assert_eq!(projected_slots, expected_slots);
    }

    #[test]
    fn exited_adventurers_never_appear(
        mut domain in support::strategies::guild_room_domain(),
        area in support::strategies::safe_rect(),
    ) {
        for agent in domain.agents.values_mut() {
            agent.presence = Presence::Exited;
        }
        let projection = project(&model_with(domain), area);
        prop_assert!(projection.adventurers.is_empty());
    }

    #[test]
    fn every_representation_uses_an_allowed_truthful_station(
        domain in support::strategies::guild_room_domain(),
        area in support::strategies::safe_rect(),
    ) {
        let projection = project(&model_with(domain.clone()), area);
        for representation in &projection.adventurers {
            match representation {
                AdventurerRepresentation::Physical { agent, station } => {
                    let source = &domain.agents[agent];
                    let allowed = matches!(
                        (source.presence, &source.attention, station),
                        (Presence::Idle, _, GuildLandmark::Hearth)
                            | (
                                Presence::Done,
                                GuildAttention::Unread {
                                    summons: GuildSummons::SpoilsReturned,
                                    ..
                                },
                                GuildLandmark::Spoils
                            )
                    );
                    prop_assert!(allowed);
                    prop_assert!(projection
                        .landmarks
                        .iter()
                        .any(|landmark| &landmark.landmark == station));
                }
                AdventurerRepresentation::Projection { agent, station } => {
                    prop_assert_eq!(domain.agents[agent].presence, Presence::Blocked);
                    prop_assert_eq!(station, &GuildLandmark::CounselBell);
                    prop_assert!(projection
                        .landmarks
                        .iter()
                        .any(|landmark| &landmark.landmark == station));
                }
                AdventurerRepresentation::Token { agent, table } => {
                    let source = &domain.agents[agent];
                    let allowed = matches!(
                        (source.presence, &source.attention),
                        (Presence::Working | Presence::Unknown, _)
                            | (
                                Presence::Done,
                                GuildAttention::Clear
                                    | GuildAttention::Read { .. }
                                    | GuildAttention::Deferred { .. }
                                    | GuildAttention::Unread {
                                        summons: GuildSummons::CounselRequested
                                            | GuildSummons::AdventurerDeparted,
                                        ..
                                    }
                            )
                    );
                    prop_assert!(allowed);
                    prop_assert_eq!(table, &source.workspace_id);
                    prop_assert!(projection
                        .campaigns
                        .iter()
                        .any(|campaign| &campaign.workspace_id == table));
                }
            }
        }
    }

    #[test]
    fn all_projected_zones_are_contained_and_non_overlapping(
        domain in support::strategies::guild_room_domain(),
        area in support::strategies::safe_rect(),
    ) {
        let projection = project(&model_with(domain), area);
        let zones = projection
            .landmarks
            .iter()
            .map(|landmark| landmark.area)
            .chain(projection.campaigns.iter().map(|campaign| campaign.area))
            .collect::<Vec<_>>();
        prop_assert!(projection
            .landmarks
            .iter()
            .all(|landmark| !matches!(landmark.landmark, GuildLandmark::CampaignTable(_))));

        for zone in &zones {
            prop_assert!(zone.x >= area.x);
            prop_assert!(zone.y >= area.y);
            prop_assert!(zone.right() <= area.right());
            prop_assert!(zone.bottom() <= area.bottom());
        }
        for (index, left) in zones.iter().enumerate() {
            for right in zones.iter().skip(index + 1) {
                prop_assert!(!rectangles_overlap(*left, *right));
            }
        }
    }

    #[test]
    fn identical_input_produces_identical_projection(
        domain in support::strategies::guild_room_domain(),
        area in support::strategies::safe_rect(),
    ) {
        let model = model_with(domain);
        prop_assert_eq!(project(&model, area), project(&model, area));
    }

    #[test]
    fn selection_never_changes_stable_campaign_table_identity(
        domain in support::strategies::guild_room_domain(),
        area in support::strategies::safe_rect(),
    ) {
        let mut first = model_with(domain);
        let mut second = first.clone();
        let keys = first.domain().agents.keys().cloned().collect::<Vec<_>>();
        first.domain_mut().selected_agent = keys.first().cloned();
        second.domain_mut().selected_agent = keys.last().cloned();

        prop_assert_eq!(
            stable_campaign_identity(&project(&first, area)),
            stable_campaign_identity(&project(&second, area)),
        );
    }

    #[test]
    fn live_focus_changes_only_illumination_not_location_or_geometry(
        domain in support::strategies::guild_room_domain(),
        area in support::strategies::safe_rect(),
    ) {
        let mut calm_domain = domain.clone();
        for agent in calm_domain.agents.values_mut() {
            agent.focused = false;
        }
        let mut focused_domain = domain;
        for agent in focused_domain.agents.values_mut() {
            agent.focused = true;
        }
        let calm = project(&model_with(calm_domain), area);
        let focused = project(&model_with(focused_domain), area);

        prop_assert_eq!(&calm.adventurers, &focused.adventurers);
        prop_assert_eq!(stable_campaign_identity(&calm), stable_campaign_identity(&focused));
        prop_assert_eq!(
            calm.landmarks
                .iter()
                .map(|landmark| (&landmark.landmark, landmark.area))
                .collect::<Vec<_>>(),
            focused
                .landmarks
                .iter()
                .map(|landmark| (&landmark.landmark, landmark.area))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn arbitrary_projection_input_never_panics(
        domain in support::strategies::guild_room_domain(),
        area in support::strategies::safe_rect(),
    ) {
        let projection = project(&model_with(domain), area);
        prop_assert_eq!(projection.landmarks.len(), 7);
    }
}

fn rectangles_overlap(left: Rect, right: Rect) -> bool {
    left.x < right.right()
        && right.x < left.right()
        && left.y < right.bottom()
        && right.y < left.bottom()
}
