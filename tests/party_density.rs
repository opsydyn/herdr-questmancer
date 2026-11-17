#![cfg(feature = "storybook")]

use questmancer::{
    app::{Motion, View},
    domain::{AgentKey, GuildAttention, GuildSummons, Presence, Timestamp},
    scene::{
        fit::{CellPixels, FitScene, FitTarget},
        pixel::PixelRect,
    },
    storybook::{
        density::{PartySize, fixture},
        fixtures::StoryContext,
    },
};
use ratatui::layout::Rect;

fn party(count: usize, view: View) -> questmancer::app::Model {
    let mut model = fixture(StoryContext::fixed(), PartySize::ThirtyTwo, view);
    let seed = model.domain().agents.values().next().unwrap().clone();
    model.domain_mut().agents.clear();
    for index in 0..count {
        let mut agent = seed.clone();
        agent.key = AgentKey::new(format!("density-{index:03}"));
        agent.presence = if index == count.saturating_sub(1) {
            Presence::Unknown
        } else {
            Presence::Idle
        };
        agent.attention = GuildAttention::Clear;
        model.domain_mut().agents.insert(agent.key.clone(), agent);
    }
    model.domain_mut().selected_agent = model.domain().agents.keys().next().cloned();
    let mut preferences = *model.preferences();
    preferences.motion = Motion::None;
    model.set_preferences(preferences);
    model
}

fn project(model: &questmancer::app::Model, width: u16, height: u16) -> FitScene {
    FitScene::project(
        model,
        Rect::new(0, 1, width, height - 3),
        CellPixels::new(8, 16).unwrap(),
    )
    .unwrap_or_else(|error| panic!("production fit lost the complete party: {error}"))
}

#[test]
fn seven_resting_adventurers_keep_full_sprites_fitted_in_both_rooms() {
    for view in [View::Guild, View::Delve] {
        let model = party(7, view);
        let scene = project(&model, 120, 34);
        assert_eq!(scene.frame.actors.len(), 7);
        assert!(
            scene
                .frame
                .actors
                .iter()
                .all(|a| a.bounds.width == 16 && a.bounds.height == 24)
        );
        assert!(
            scene
                .frame
                .actors
                .iter()
                .any(|a| a.bounds.y != scene.frame.actors[0].bounds.y)
        );
    }
}

#[test]
fn size_tiers_keep_every_actor_complete_distinct_and_clickable() {
    for view in [View::Guild, View::Delve] {
        for count in [0, 1, 7, 11, 12, 24, 32, 33, 64] {
            let model = party(count, view);
            for (width, height) in [(120, 34), (96, 28), (80, 24)] {
                let scene = project(&model, width, height);
                assert_eq!(scene.frame.actors.len(), count, "{view:?} party {count}");
                assert_eq!(
                    scene.frame.interactables.len(),
                    usize::from(view == View::Guild && count <= 32)
                );
                for (index, actor) in scene.frame.actors.iter().enumerate() {
                    assert_eq!(
                        (actor.bounds.width, actor.bounds.height),
                        if count <= 11 { (16, 24) } else { (8, 12) }
                    );
                    assert!(
                        actor.bounds.x >= 0
                            && actor.bounds.y >= 0
                            && actor.bounds.x + i32::from(actor.bounds.width) <= 160
                            && actor.bounds.y + i32::from(actor.bounds.height) <= 90
                    );
                    assert!(
                        scene
                            .frame
                            .actors
                            .iter()
                            .skip(index + 1)
                            .all(|b| !overlaps(actor.bounds, b.bounds))
                    );
                    assert!(
                        scene
                            .frame
                            .interactables
                            .iter()
                            .all(|b| !overlaps(actor.bounds, b.bounds))
                    );
                    assert!(scene.cells().any(|(x, y)| scene.target_at(x, y)
                        == Some(FitTarget::Adventurer(actor.agent.clone()))));
                }
            }
        }
    }
}

#[test]
fn exited_adventurers_release_capacity_and_parties_beyond_the_limit_are_explicit() {
    for view in [View::Guild, View::Delve] {
        let mut model = party(12, view);
        let original = project(&model, 96, 28);
        model
            .domain_mut()
            .agents
            .values_mut()
            .last()
            .unwrap()
            .presence = Presence::Exited;
        let smaller = project(&model, 96, 28);
        assert_eq!(smaller.frame.actors.len(), 11);
        assert!(smaller.frame.actors.iter().all(|a| a.bounds.width == 16));
        model
            .domain_mut()
            .agents
            .values_mut()
            .last()
            .unwrap()
            .presence = Presence::Unknown;
        assert_eq!(project(&model, 96, 28).frame.actors, original.frame.actors);
        assert!(matches!(
            FitScene::project(
                &party(65, view),
                Rect::new(0, 1, 96, 25),
                CellPixels::new(8, 16).unwrap()
            ),
            Err(questmancer::scene::fit::FitFailure::Incomplete { total: 65, .. })
        ));
    }
}

#[test]
fn full_motion_is_semantic_and_completion_is_quiet_at_three_seconds() {
    for view in [View::Guild, View::Delve] {
        for count in [7, 12, 33] {
            let mut model = party(count, view);
            let mut preferences = *model.preferences();
            preferences.motion = Motion::Full;
            model.set_preferences(preferences);
            let since = model.now();
            let agent = model.domain_mut().agents.values_mut().next().unwrap();
            agent.presence = Presence::Done;
            agent.attention = GuildAttention::unread(GuildSummons::SpoilsReturned, since);
            assert!(project(&model, 96, 28).frame.next_frame_in.is_some());
            model.set_now(Timestamp::from_millis(since.as_millis() + 2999));
            assert!(project(&model, 96, 28).frame.next_frame_in.is_some());
            model.set_now(Timestamp::from_millis(since.as_millis() + 3000));
            let quiet = project(&model, 96, 28);
            assert_eq!(quiet.frame.next_frame_in, None);
            model.set_now(Timestamp::from_millis(since.as_millis() + 8000));
            assert_eq!(project(&model, 96, 28).rgb, quiet.rgb);
        }
    }
}

#[test]
fn reduced_and_still_motion_have_no_cleanup_wake_or_time_dependent_pixels() {
    for view in [View::Guild, View::Delve] {
        for count in [7, 12, 33] {
            for motion in [Motion::Reduced, Motion::None] {
                let mut model = party(count, view);
                let mut preferences = *model.preferences();
                preferences.motion = motion;
                model.set_preferences(preferences);
                let since = model.now();
                let agent = model.domain_mut().agents.values_mut().next().unwrap();
                agent.presence = Presence::Done;
                agent.attention = GuildAttention::unread(GuildSummons::SpoilsReturned, since);
                let before = project(&model, 96, 28);
                assert_eq!(before.frame.next_frame_in, None);
                model.set_now(Timestamp::from_millis(since.as_millis() + 5000));
                let after = project(&model, 96, 28);
                assert_eq!(after.frame.next_frame_in, None);
                assert_eq!(after.rgb, before.rgb);
            }
        }
    }
}

#[test]
fn presence_changes_do_not_change_density_or_move_the_rest_of_the_party() {
    for view in [View::Guild, View::Delve] {
        for count in [7, 11, 12, 32, 33, 64] {
            let mut model = party(count, view);
            let before = project(&model, 96, 28);
            for presence in [
                Presence::Working,
                Presence::Blocked,
                Presence::Idle,
                Presence::Unknown,
                Presence::Done,
            ] {
                for agent in model.domain_mut().agents.values_mut() {
                    agent.presence = presence;
                }
                let after = project(&model, 96, 28);
                assert_eq!(after.transform, before.transform);
                assert_eq!(
                    after.frame.actors, before.frame.actors,
                    "{view:?} party {count} {presence:?}"
                );
                assert_eq!(after.frame.next_frame_in, None);
                if count > 11 {
                    let pose = match presence {
                        Presence::Working => questmancer::scene::stage::ScenePose::Working,
                        Presence::Blocked => questmancer::scene::stage::ScenePose::SeekingCounsel,
                        Presence::Idle => questmancer::scene::stage::ScenePose::Resting,
                        Presence::Done => questmancer::scene::stage::ScenePose::Settled,
                        Presence::Unknown | Presence::Exited => {
                            questmancer::scene::stage::ScenePose::Unknown
                        }
                    };
                    let cue = questmancer::scene::assets::roster::state_marker(pose);
                    for actor in &after.frame.actors {
                        for (index, colour) in cue.pixels().iter().enumerate() {
                            if let Some(colour) = colour {
                                let x = actor.bounds.x + 2 + i32::try_from(index % 5).unwrap();
                                let y = actor.bounds.y - 6 + i32::try_from(index / 5).unwrap();
                                assert_eq!(
                                    after.rgb.get(x, y),
                                    Some(*colour),
                                    "{view:?} party {count} lost {presence:?} cue"
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

fn overlaps(a: PixelRect, b: PixelRect) -> bool {
    a.x < b.x + i32::from(b.width)
        && a.x + i32::from(a.width) > b.x
        && a.y < b.y + i32::from(b.height)
        && a.y + i32::from(a.height) > b.y
}
