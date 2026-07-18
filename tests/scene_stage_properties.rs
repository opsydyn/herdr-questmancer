#[allow(dead_code)]
mod support;

use std::collections::BTreeSet;

use proptest::prelude::*;
use questmancer::{
    app::{Model, View},
    domain::Presence,
    scene::{pixel::PixelSize, snapshot::SceneSnapshot, stage::ScenePlan},
};

proptest! {
    #[test]
    fn chosen_world_places_every_non_exited_agent_exactly_once(
        snapshot in support::strategies::scene_snapshot()
    ) {
        let plan = ScenePlan::project(&snapshot, PixelSize::new(160, 90));
        let actor_keys = plan
            .actors
            .iter()
            .map(|actor| actor.agent.clone())
            .collect::<Vec<_>>();
        let unique_actor_keys = actor_keys.iter().collect::<BTreeSet<_>>();
        let expected_count = snapshot
            .agents
            .iter()
            .filter(|agent| agent.presence != Presence::Exited)
            .count();

        prop_assert_eq!(actor_keys.len(), expected_count);
        prop_assert_eq!(unique_actor_keys.len(), actor_keys.len());
        for agent in &snapshot.agents {
            let occurrences = actor_keys.iter().filter(|key| *key == &agent.key).count();
            if agent.presence == Presence::Exited {
                prop_assert_eq!(occurrences, 0);
            } else {
                prop_assert_eq!(occurrences, 1);
            }
        }
    }

    #[test]
    fn input_vector_order_does_not_change_the_plan(
        snapshot in support::strategies::scene_snapshot(),
        viewport in (0_u16..400, 0_u16..240),
    ) {
        let viewport = PixelSize::new(viewport.0, viewport.1);
        let expected = ScenePlan::project(&snapshot, viewport);
        let mut reordered = snapshot;
        reordered.agents.reverse();
        reordered.campaigns.reverse();

        prop_assert_eq!(ScenePlan::project(&reordered, viewport), expected);
    }

    #[test]
    fn fixed_snapshot_viewport_and_time_are_deterministic_and_never_panic(
        snapshot in support::strategies::scene_snapshot(),
        width in 0_u16..400,
        height in 0_u16..240,
    ) {
        let viewport = PixelSize::new(width, height);
        let first = ScenePlan::project(&snapshot, viewport);
        let second = ScenePlan::project(&snapshot, viewport);

        prop_assert_eq!(first, second);
    }

    #[test]
    fn changing_legacy_selection_cannot_change_the_snapshot(
        domain in support::domain_state()
    ) {
        let mut first = Model::new(View::Guild);
        *first.domain_mut() = domain.clone();
        first.domain_mut().selected_agent = None;

        let mut second = Model::new(View::Delve);
        *second.domain_mut() = domain;
        second.domain_mut().selected_agent = second.domain().agents.keys().next_back().cloned();

        prop_assert_eq!(
            SceneSnapshot::from_model(&first),
            SceneSnapshot::from_model(&second)
        );
    }

    #[test]
    fn snapshot_and_stage_projection_do_not_persist_camera_or_station_state(
        domain in support::domain_state(),
        width in 0_u16..400,
        height in 0_u16..240,
    ) {
        let mut model = Model::new(View::Guild);
        *model.domain_mut() = domain;
        let before = model.domain().clone();

        let snapshot = SceneSnapshot::from_model(&model);
        let _plan = ScenePlan::project(&snapshot, PixelSize::new(width, height));

        prop_assert_eq!(model.domain(), &before);
    }
}
