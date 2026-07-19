#[allow(dead_code)]
mod support;

use std::collections::{BTreeSet, HashSet};

use proptest::prelude::*;
use questmancer::{
    app::{Model, Motion, View},
    domain::Presence,
    scene::{
        pixel::{PixelSize, Rgb, RgbBuffer},
        render_scene,
        snapshot::{SceneConnection, SceneSnapshot},
        stage::{ScenePlan, WorldScene},
    },
};

fn focused_crop_origin(focus: (i32, i32), viewport: PixelSize) -> (i32, i32) {
    let width = i32::from(viewport.width);
    let height = i32::from(viewport.height);
    (
        (focus.0 - width / 2).clamp(0, 160 - width),
        (focus.1 - height / 2).clamp(0, 90 - height),
    )
}

fn known_environment_palette(world: WorldScene, canonical: &RgbBuffer) -> HashSet<Rgb> {
    canonical
        .pixels()
        .iter()
        .copied()
        .enumerate()
        .filter_map(|(index, pixel)| {
            let x = i32::try_from(index % 160).expect("canonical x fits i32");
            let y = i32::try_from(index / 160).expect("canonical y fits i32");
            let first_delve_actor = (60..68).contains(&x) && (25..41).contains(&y);
            (world != WorldScene::Delve || !first_delve_actor).then_some(pixel)
        })
        .collect()
}

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

    #[test]
    fn both_world_renderers_accept_arbitrary_viewports_without_resizing_or_leaking_clear_pixels(
        mut source in support::strategies::scene_snapshot(),
        width in 0_u16..400,
        height in 0_u16..240,
    ) {
        prop_assume!(!source.agents.is_empty());
        let viewport = PixelSize::new(width, height);
        source.motion = Motion::None;

        let mut guild = source.clone();
        guild.connection = SceneConnection::Offline;

        let mut delve = source;
        delve.connection = SceneConnection::Connected;
        for agent in &mut delve.agents {
            agent.presence = Presence::Exited;
            agent.transition = None;
            agent.focused = false;
        }
        delve.agents[0].presence = Presence::Working;
        delve.agents[0].focused = true;

        for (expected_world, snapshot) in [
            (WorldScene::GuildHall, guild),
            (WorldScene::Delve, delve),
        ] {
            let mut environment_snapshot = snapshot.clone();
            if expected_world == WorldScene::GuildHall {
                for agent in &mut environment_snapshot.agents {
                    agent.presence = Presence::Exited;
                    agent.transition = None;
                    agent.focused = false;
                }
            }
            let mut canonical = RgbBuffer::filled(0, 0, Rgb::BLACK);
            let canonical_frame = render_scene(
                &environment_snapshot,
                PixelSize::new(160, 90),
                &mut canonical,
            );
            prop_assert_eq!(canonical_frame.world, expected_world);
            let environment_palette = known_environment_palette(expected_world, &canonical);

            let sentinel = Rgb::new(255, 0, 255);
            let mut target = RgbBuffer::filled(1, 1, sentinel);
            let frame = render_scene(&snapshot, viewport, &mut target);
            prop_assert_eq!(frame.world, expected_world);
            prop_assert_eq!(target.size(), viewport);
            if width > 0 && height > 0 {
                prop_assert!(!target.pixels().is_empty());
                prop_assert!(target.pixels().iter().all(|pixel| *pixel != sentinel));
                prop_assert!(
                    target.pixels().iter().any(|pixel| environment_palette.contains(pixel)),
                    "{expected_world:?} omitted the known environment/material palette; first pixel: {:?}",
                    target.get(0, 0)
                );
            }
        }
    }

    #[test]
    fn arbitrary_focused_crops_remain_inside_the_authored_world(
        mut source in support::strategies::scene_snapshot(),
        width in 1_u16..120,
        height in 1_u16..72,
    ) {
        prop_assume!(!source.agents.is_empty());
        source.motion = Motion::None;

        let mut guild = source.clone();
        guild.connection = SceneConnection::Connected;
        for agent in &mut guild.agents {
            agent.presence = Presence::Exited;
            agent.transition = None;
            agent.focused = false;
        }
        guild.agents[0].presence = Presence::Blocked;
        guild.agents[0].focused = true;

        let mut delve = source;
        delve.connection = SceneConnection::Connected;
        for agent in &mut delve.agents {
            agent.presence = Presence::Exited;
            agent.transition = None;
            agent.focused = false;
        }
        delve.agents[0].presence = Presence::Working;
        delve.agents[0].focused = true;

        let viewport = PixelSize::new(width, height);
        for (expected_world, focus, snapshot) in [
            (WorldScene::GuildHall, (124, 48), guild),
            (WorldScene::Delve, (81, 47), delve),
        ] {
            let mut canonical = RgbBuffer::filled(0, 0, Rgb::BLACK);
            let full_frame = render_scene(
                &snapshot,
                PixelSize::new(160, 90),
                &mut canonical,
            );
            prop_assert_eq!(full_frame.world, expected_world);

            let mut crop = RgbBuffer::filled(0, 0, Rgb::BLACK);
            let crop_frame = render_scene(&snapshot, viewport, &mut crop);
            prop_assert_eq!(crop_frame.world, expected_world);
            let (origin_x, origin_y) = focused_crop_origin(focus, viewport);
            prop_assert!(origin_x >= 0);
            prop_assert!(origin_y >= 0);
            prop_assert!(origin_x + i32::from(width) <= 160);
            prop_assert!(origin_y + i32::from(height) <= 90);
            for y in 0..i32::from(height) {
                for x in 0..i32::from(width) {
                    prop_assert_eq!(crop.get(x, y), canonical.get(x + origin_x, y + origin_y));
                }
            }
        }
    }
}
