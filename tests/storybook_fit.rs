#![cfg(feature = "storybook")]

use questmancer::storybook::{
    fit::{CellPixels, FitScene, FitTarget, FitTransform},
    fixtures::{FitParty, StoryContext, hall_fit_fixture},
};
use ratatui::layout::Rect;

#[test]
fn integer_fit_matches_literal_pane_geometry_and_rejects_invalid_axes() {
    let cell = CellPixels::new(8, 16).unwrap();
    let plan = FitTransform::new(Rect::new(7, 3, 96, 28), cell).unwrap();
    assert_eq!(plan.scale, 4);
    assert_eq!(plan.area, Rect::new(15, 5, 80, 23));
    assert_eq!(plan.padding, (0, 4));
    assert_eq!(CellPixels::new(0, 16), None);
    assert_eq!(CellPixels::new(8, 0), None);
    assert_eq!(FitTransform::new(Rect::new(0, 0, 1, 1), cell), None);
    assert_eq!(
        FitTransform::new(
            Rect::new(0, 0, 80, 24),
            CellPixels {
                width: 0,
                height: 16
            }
        ),
        None
    );
    assert_eq!(plan.source_at_cell(14, 5), None);
    assert_eq!(plan.source_at_cell(95, 5), None);
}

#[test]
fn every_authored_source_pixel_survives_at_each_review_size() {
    let model = hall_fit_fixture(StoryContext::fixed(), FitParty::Five);
    for (width, height) in [(120, 34), (96, 28), (80, 24)] {
        let fit = FitScene::project(
            &model,
            Rect::new(0, 0, width, height),
            CellPixels::new(8, 16).unwrap(),
        )
        .unwrap();
        assert_eq!(fit.frame.actors.len(), 5);
        assert_eq!(fit.frame.interactables.len(), 1);
        let image = fit.transform.image(&fit.rgb).to_rgb8();
        let scale = u32::from(fit.transform.scale);
        for y in 0..90 {
            for x in 0..160 {
                let expected = fit.rgb.pixels()[y * 160 + x];
                for (dx, dy) in [(0, 0), (scale - 1, scale - 1)] {
                    assert_eq!(
                        image
                            .get_pixel(
                                u32::try_from(x).unwrap() * scale + fit.transform.padding.0 + dx,
                                u32::try_from(y).unwrap() * scale + fit.transform.padding.1 + dy
                            )
                            .0,
                        [expected.r, expected.g, expected.b]
                    );
                }
            }
        }
    }
}

#[test]
fn distributed_party_and_librarian_are_clickable_after_offset_and_resize() {
    for party in [FitParty::Five, FitParty::Eleven] {
        let model = hall_fit_fixture(StoryContext::fixed(), party);
        for (width, height) in [(120, 34), (96, 28), (80, 24)] {
            let fit = FitScene::project(
                &model,
                Rect::new(11, 7, width, height),
                CellPixels::new(8, 16).unwrap(),
            )
            .unwrap();
            assert_eq!(fit.frame.actors.len(), model.domain().agents.len());
            for actor in &fit.frame.actors {
                assert!(
                    fit.cells().any(|(x, y)| fit.target_at(x, y)
                        == Some(FitTarget::Adventurer(actor.agent.clone()))),
                    "actor {} is unreachable at {width}x{height}",
                    actor.agent
                );
            }
            assert!(
                fit.cells()
                    .any(|(x, y)| fit.target_at(x, y) == Some(FitTarget::Librarian))
            );
            assert_eq!(fit.target_at(0, 0), None);
        }
    }
}

#[test]
fn crowded_station_capacity_keeps_every_adventurer_in_the_fitted_room() {
    let model = hall_fit_fixture(StoryContext::fixed(), FitParty::Crowded);
    let fit = FitScene::project(
        &model,
        Rect::new(0, 0, 120, 34),
        CellPixels::new(8, 16).unwrap(),
    )
    .unwrap();
    assert_eq!(fit.frame.actors.len(), 11);
    assert!(
        fit.frame
            .actors
            .iter()
            .all(|a| (a.bounds.width, a.bounds.height) == (16, 24))
    );
}

#[test]
fn fitted_hall_has_explicit_storybook_fixtures_without_replacing_production_worlds() {
    let stories = questmancer::storybook::catalogue::catalogue();
    for id in [
        "world.guild-fit",
        "world.guild-fit-eleven",
        "world.guild-fit-crowded",
    ] {
        assert!(stories.iter().any(|s| s.id.as_str() == id), "missing {id}");
    }
    assert!(stories.iter().any(|s| s.id.as_str() == "world.guild"));
    assert!(stories.iter().any(|s| s.id.as_str() == "world.delve"));
}

#[test]
fn delve_fit_stories_cover_native_and_crowded_dungeon_without_changing_source_facts() {
    use questmancer::{app::View, storybook::fixtures::StoryFixture};
    let catalogue = questmancer::storybook::catalogue::catalogue();
    for (id, total) in [
        ("world.delve-fit", 5),
        ("world.delve-fit-crowded", 11),
        ("world.delve-fit-resting", 5),
        ("world.delve-fit-profile", 5),
    ] {
        let story = catalogue
            .iter()
            .find(|s| s.id.as_str() == id)
            .expect("missing Delve fit story");
        let fixture = (story.build)(StoryContext::fixed());
        let StoryFixture::WorldFit(model) = fixture else {
            panic!("fit fixture required")
        };
        assert_eq!(model.view(), View::Delve);
        assert_eq!(model.domain().agents.len(), total);
    }
}
