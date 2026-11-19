#![cfg(feature = "storybook")]

use questmancer::{
    app::View,
    scene::{
        SceneTarget,
        fit::{CellPixels, FitScene, FitTarget},
        pixel::{Rgb, RgbBuffer},
    },
    storybook::fixtures::{FitParty, StoryContext, hall_fit_fixture},
    ui::world_fit::{WorldFit, render_application},
};
use ratatui::{Terminal, backend::TestBackend, layout::Rect};

fn draw(
    fit: &mut WorldFit,
    model: &questmancer::app::Model,
    size: (u16, u16),
) -> (questmancer::scene::SceneFrame, String, Vec<u32>) {
    let mut terminal = Terminal::new(TestBackend::new(size.0, size.1)).unwrap();
    let mut rgb = RgbBuffer::filled(0, 0, Rgb::BLACK);
    let mut scene = None;
    terminal
        .draw(|f| scene = Some(render_application(f, model, &mut rgb, None, fit)))
        .unwrap();
    let text = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect();
    (scene.unwrap(), text, fit.finish_frame())
}

#[test]
fn production_fit_uses_visible_cells_for_every_actor_and_librarian() {
    let model = hall_fit_fixture(StoryContext::fixed(), FitParty::Eleven);
    let cell = CellPixels::new(8, 16).unwrap();
    let mut graphics = WorldFit::new(Some(cell));
    for (w, h) in [(120, 34), (96, 28), (80, 24)] {
        let (frame, text, _) = draw(&mut graphics, &model, (w, h));
        assert!(text.contains("a=T,U=1"));
        assert!(text.contains("need counsel"));
        let source = FitScene::project(&model, Rect::new(0, 1, w, h - 3), cell).unwrap();
        for y in 0..h {
            for x in 0..w {
                let expected = source.target_at(x, y);
                let actual = frame.target_at(x, y).map(|t| match t {
                    SceneTarget::Agent(key) => FitTarget::Adventurer(key.clone()),
                    SceneTarget::Interactable(_) => FitTarget::Librarian,
                });
                assert_eq!(actual, expected, "cell {x},{y} at {w}x{h}");
            }
        }
        assert_eq!(frame.next_frame_in, source.frame.next_frame_in);
    }
    assert!(graphics.shutdown().len() <= 1);
    assert!(graphics.shutdown().is_empty());
}

#[test]
fn fitted_hall_keeps_counsel_visible_when_a_neighbour_occupies_the_usual_cue() {
    use questmancer::domain::Presence;
    let mut model = hall_fit_fixture(StoryContext::fixed(), FitParty::Eleven);
    for (index, agent) in model.domain_mut().agents.values_mut().enumerate() {
        agent.presence = if index == 8 {
            Presence::Blocked
        } else {
            Presence::Idle
        };
    }
    let area = Rect::new(0, 0, 96, 25);
    let cell = CellPixels::new(8, 16).unwrap();
    let blocked = FitScene::project(&model, area, cell).unwrap();
    model
        .domain_mut()
        .agents
        .values_mut()
        .nth(8)
        .unwrap()
        .presence = Presence::Idle;
    let resting = FitScene::project(&model, area, cell).unwrap();
    assert_eq!(blocked.frame.actors, resting.frame.actors);
    let target = blocked.frame.actors[8].bounds;
    let flame = questmancer::scene::assets::palette::FLAME;
    assert!((target.y - 6..target.y + 7).any(|y| {
        (target.x - 5..target.x + i32::from(target.width) + 5)
            .any(|x| blocked.rgb.get(x, y) == Some(flame) && resting.rgb.get(x, y) != Some(flame))
    }));
}

#[test]
fn production_cache_and_overlay_view_fallback_transitions_release_only_owned_images() {
    let mut model = hall_fit_fixture(StoryContext::fixed(), FitParty::Five);
    let mut graphics = WorldFit::new(CellPixels::new(8, 16));
    draw(&mut graphics, &model, (96, 28));
    assert_eq!(graphics.uploads(), 1);
    for _ in 0..3 {
        draw(&mut graphics, &model, (96, 28));
    }
    assert_eq!(graphics.uploads(), 1);
    model.open_ledger();
    let (_, text, removed) = draw(&mut graphics, &model, (96, 28));
    assert!(text.contains("LIBRARIAN'S LEDGER"));
    assert!(!text.contains("a=T,U=1"));
    assert_eq!(removed.len(), 1);
    model.dismiss_modal();
    draw(&mut graphics, &model, (96, 28));
    assert_eq!(graphics.uploads(), 2);
    model.switch_to(View::Delve);
    let (_, text, removed) = draw(&mut graphics, &model, (96, 28));
    assert!(text.contains("a=T,U=1"));
    assert_eq!(removed.len(), 1);
    assert_eq!(graphics.uploads(), 3);
    assert_eq!(graphics.shutdown().len(), 1);
}

#[test]
fn compact_rooms_keep_selected_status_action_and_counsel_count_visible() {
    for view in [View::Guild, View::Delve] {
        let mut model = hall_fit_fixture(StoryContext::fixed(), FitParty::Five);
        model.switch_to(view);
        model.set_integration_diagnostic("Partial Chronicle coverage".to_owned());
        let selected = model.selected_agent().unwrap().name.clone();
        let mut graphics = WorldFit::new(None);
        let (scene, text, _) = draw(&mut graphics, &model, (50, 18));
        assert_eq!(scene.actors.len(), 5);
        for fact in [
            selected.as_str(),
            "WORKING",
            "[Enter]",
            "[!] 1 need counsel",
            "Partial Chronicle coverage",
        ] {
            assert!(text.contains(fact), "missing {fact} in {view:?}");
        }
        assert!(scene.target_at(20, 16).is_none());
    }
}

#[test]
fn production_fallback_retains_complete_parties_when_graphics_are_unavailable() {
    for party in [FitParty::Five, FitParty::Crowded] {
        let model = hall_fit_fixture(StoryContext::fixed(), party);
        let mut graphics = WorldFit::new(None);
        let (frame, text, removed) = draw(&mut graphics, &model, (160, 48));
        assert_eq!(frame.actors.len(), model.domain().agents.len());
        assert!(!text.contains("a=T,U=1"));
        assert!(removed.is_empty());
    }
}

#[test]
fn every_parchment_and_ansi_preference_suspends_native_placeholders() {
    use questmancer::app::Model;
    let openers: [fn(&mut Model); 7] = [
        Model::open_ledger,
        Model::open_counsel,
        Model::open_search,
        Model::open_chronicle,
        Model::open_quest_board,
        Model::open_war_table,
        Model::open_scrying,
    ];
    for view in [View::Guild, View::Delve] {
        for open in openers {
            let mut model = hall_fit_fixture(StoryContext::fixed(), FitParty::Five);
            model.switch_to(view);
            let mut graphics = WorldFit::new(CellPixels::new(8, 16));
            draw(&mut graphics, &model, (96, 28));
            open(&mut model);
            let (_, text, removed) = draw(&mut graphics, &model, (96, 28));
            assert!(
                !text.contains("a=T,U=1"),
                "parchment must never overwrite image placeholders"
            );
            assert_eq!(removed.len(), 1);
        }
    }
    let mut model = hall_fit_fixture(StoryContext::fixed(), FitParty::Five);
    let mut graphics = WorldFit::new(CellPixels::new(8, 16));
    draw(&mut graphics, &model, (96, 28));
    model.cycle_color_mode();
    let (_, text, removed) = draw(&mut graphics, &model, (96, 28));
    assert!(!text.contains("a=T,U=1"));
    assert_eq!(removed.len(), 1);
}

#[test]
fn production_clicks_reuse_existing_selection_and_ledger_actions() {
    use crossterm::event::{Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
    use questmancer::{interaction::reduce_scene_action, ui::input::action_for_scene_event_in};
    let mut model = hall_fit_fixture(StoryContext::fixed(), FitParty::Five);
    let mut graphics = WorldFit::new(CellPixels::new(8, 16));
    let (frame, _, _) = draw(&mut graphics, &model, (96, 28));
    let target = model.domain().agents.keys().nth(1).unwrap().clone();
    let cell = (0..28)
        .flat_map(|y| (0..96).map(move |x| (x, y)))
        .find(|(x, y)| frame.agent_at(*x, *y) == Some(&target))
        .unwrap();
    let click = |(column, row)| {
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column,
            row,
            modifiers: KeyModifiers::NONE,
        })
    };
    let action = action_for_scene_event_in(&click(cell), model.modal(), &frame);
    let _ = reduce_scene_action(&mut model, action, &frame);
    assert_eq!(model.selected_agent_key(), Some(&target));
    assert!(model.adventurer_card_visible());
    model.dismiss_adventurer_card();
    let cell = (0..28)
        .flat_map(|y| (0..96).map(move |x| (x, y)))
        .find(|(x, y)| frame.interactable_at(*x, *y).is_some())
        .unwrap();
    let action = action_for_scene_event_in(&click(cell), model.modal(), &frame);
    let _ = reduce_scene_action(&mut model, action, &frame);
    assert!(model.ledger_page().is_some());
}

#[test]
fn animated_fit_keeps_semantic_deadlines_and_reuses_identical_frames() {
    use questmancer::{app::Motion, domain::Timestamp};
    for view in [View::Guild, View::Delve] {
        let mut model = hall_fit_fixture(StoryContext::fixed(), FitParty::Five);
        while model.preferences().motion != Motion::Full {
            model.cycle_motion();
        }
        model.switch_to(view);
        let mut graphics = WorldFit::new(CellPixels::new(8, 16));
        let mut observed = Vec::new();
        for ms in (121_000..123_000).step_by(125) {
            model.set_now(Timestamp::from_millis(ms));
            let (frame, _, ids) = draw(&mut graphics, &model, (96, 28));
            let source = FitScene::project(
                &model,
                Rect::new(0, 1, 96, 25),
                CellPixels::new(8, 16).unwrap(),
            )
            .unwrap();
            assert_eq!(frame.next_frame_in, source.frame.next_frame_in);
            observed.extend(ids);
        }
        observed.extend(graphics.shutdown());
        observed.sort_unstable();
        observed.dedup();
        assert!(
            observed.len() <= 2,
            "animation cannot allocate unbounded image identities"
        );
        assert!(
            graphics.uploads() > 1,
            "working animation must reach the native adapter"
        );
    }
}

#[test]
fn fresh_delve_work_walks_to_the_steady_seat_and_keeps_its_identity() {
    use questmancer::{
        app::{ConnectionState, Motion},
        domain::{AdventurerClass, Presence, Timestamp},
    };

    let mut model = hall_fit_fixture(StoryContext::fixed(), FitParty::Five);
    model.switch_to(View::Delve);
    while model.preferences().motion != Motion::Full {
        model.cycle_motion();
    }
    let mover = model.selected_agent_key().unwrap().clone();
    let floor = Timestamp::from_millis(130_000);
    let started = Timestamp::from_millis(130_100);
    model.set_connection_at(ConnectionState::Connected, floor);
    for agent in model.domain_mut().agents.values_mut() {
        agent.presence = Presence::Idle;
    }
    let agent = model.domain_mut().agents.get_mut(&mover).unwrap();
    agent.persona.class = AdventurerClass::RatRoyalty;
    agent.presence = Presence::Working;
    agent.presence_since = started;

    let cells = CellPixels::new(8, 16).unwrap();
    let area = Rect::new(0, 0, 96, 25);
    let sample = |model: &mut questmancer::app::Model, millis| {
        model.set_now(Timestamp::from_millis(millis));
        FitScene::project(model, area, cells).unwrap()
    };
    let first = sample(&mut model, 130_100);
    let middle = sample(&mut model, 130_725);
    let settled = sample(&mut model, 131_600);
    assert_ne!(first.frame.actors[0].bounds, middle.frame.actors[0].bounds);
    assert_ne!(
        middle.frame.actors[0].bounds,
        settled.frame.actors[0].bounds
    );
    for scene in [&first, &middle, &settled] {
        assert_eq!(scene.frame.actors[0].agent, mover);
        assert_eq!(scene.frame.actors[1..], settled.frame.actors[1..]);
        assert!((0..area.height).any(|y| {
            (0..area.width)
                .any(|x| scene.target_at(x, y) == Some(FitTarget::Adventurer(mover.clone())))
        }));
    }
    assert_eq!(
        middle.frame.next_frame_in,
        Some(std::time::Duration::from_millis(125))
    );
    assert_eq!(
        settled.frame.actors,
        sample(&mut model, 132_100).frame.actors
    );
    model.cycle_motion();
    assert_eq!(
        sample(&mut model, 130_725).frame.actors,
        settled.frame.actors
    );
    while model.preferences().motion != Motion::Full {
        model.cycle_motion();
    }
    model.set_connection_at(ConnectionState::Connected, started);
    assert_eq!(
        sample(&mut model, 130_725).frame.actors,
        settled.frame.actors
    );
}

#[test]
fn capability_geometry_and_tiny_panes_fall_back_without_forcing_a_protocol() {
    use questmancer::ui::world_fit::cell_pixels;
    assert_eq!(cell_pixels(120, 34, 960, 544), CellPixels::new(8, 16));
    for args in [
        (0, 34, 960, 544),
        (120, 0, 960, 544),
        (120, 34, 0, 0),
        (120, 34, 961, 544),
    ] {
        assert_eq!(cell_pixels(args.0, args.1, args.2, args.3), None);
    }
    let picker = ratatui_image::picker::Picker::halfblocks();
    let mut graphics = WorldFit::from_picker(Some(&picker), false);
    let model = hall_fit_fixture(StoryContext::fixed(), FitParty::Five);
    assert!(!draw(&mut graphics, &model, (96, 28)).1.contains("a=T,U=1"));
    let mut graphics = WorldFit::new(CellPixels::new(8, 16));
    draw(&mut graphics, &model, (96, 28));
    let (frame, text, removed) = draw(&mut graphics, &model, (1, 1));
    assert!(frame.actors.is_empty());
    assert!(!text.contains("a=T,U=1"));
    assert_eq!(removed.len(), 1);
}

#[test]
fn delve_fit_preserves_every_source_pixel_and_visible_mouse_target() {
    let mut model = hall_fit_fixture(StoryContext::fixed(), FitParty::Five);
    model.switch_to(View::Delve);
    let cell = CellPixels::new(8, 16).unwrap();
    let mut graphics = WorldFit::new(Some(cell));
    for (w, h) in [(120, 34), (96, 28), (80, 24)] {
        let (frame, text, _) = draw(&mut graphics, &model, (w, h));
        assert!(
            text.contains("a=T,U=1"),
            "Delve must fit through the native adapter"
        );
        assert_eq!(frame.world, questmancer::scene::stage::WorldScene::Delve);
        assert_eq!(frame.actors.len(), 5);
        assert!(
            frame.interactables.is_empty(),
            "Delve has no Librarian actor"
        );
        let source = FitScene::project(&model, Rect::new(0, 1, w, h - 3), cell).unwrap();
        for y in 0..h {
            for x in 0..w {
                assert_eq!(
                    frame.agent_at(x, y).cloned(),
                    source.target_at(x, y).map(|t| match t {
                        FitTarget::Adventurer(a) => a,
                        FitTarget::Librarian => panic!("Delve Librarian"),
                    }),
                    "cell {x},{y} at {w}x{h}"
                );
            }
        }
        let image = source.transform.image(&source.rgb).to_rgb8();
        let scale = u32::from(source.transform.scale);
        for y in 0..90 {
            for x in 0..160 {
                let pixel = source.rgb.get(x, y).unwrap();
                for (dx, dy) in [(0, 0), (scale - 1, scale - 1)] {
                    assert_eq!(
                        image
                            .get_pixel(
                                u32::try_from(x).unwrap() * scale + source.transform.padding.0 + dx,
                                u32::try_from(y).unwrap() * scale + source.transform.padding.1 + dy
                            )
                            .0,
                        [pixel.r, pixel.g, pixel.b]
                    );
                }
            }
        }
        assert_eq!(frame.next_frame_in, source.frame.next_frame_in);
    }
}

#[test]
fn crowded_delve_keeps_native_room_and_full_masters() {
    let mut model = hall_fit_fixture(StoryContext::fixed(), FitParty::Crowded);
    model.switch_to(View::Delve);
    let mut graphics = WorldFit::new(CellPixels::new(8, 16));
    for (w, h) in [(160, 48), (120, 34), (80, 24)] {
        let (frame, text, _) = draw(&mut graphics, &model, (w, h));
        assert_eq!(frame.actors.len(), 11);
        assert!(text.contains("a=T,U=1"));
        let source = FitScene::project(
            &model,
            Rect::new(0, 1, w, h - 3),
            CellPixels::new(8, 16).unwrap(),
        )
        .unwrap();
        assert!(
            source
                .frame
                .actors
                .iter()
                .all(|a| (a.bounds.width, a.bounds.height) == (16, 24))
        );
        for actor in &frame.actors {
            assert!((0..h).any(|y| (0..w).any(|x| frame.agent_at(x, y) == Some(&actor.agent))));
        }
    }
}

#[test]
fn scrying_parchment_suspends_the_delve_fit_then_resumes_without_changing_work() {
    let mut model = hall_fit_fixture(StoryContext::fixed(), FitParty::Five);
    model.switch_to(View::Delve);
    let mut graphics = WorldFit::new(CellPixels::new(8, 16));
    draw(&mut graphics, &model, (96, 28));
    let selected = model.selected_agent_key().cloned();
    model.open_scrying();
    let (_, text, removed) = draw(&mut graphics, &model, (96, 28));
    assert!(text.contains("SCRYING"));
    assert!(!text.contains("a=T,U=1"));
    assert_eq!(removed.len(), 1);
    model.dismiss_modal();
    let (_, text, _) = draw(&mut graphics, &model, (96, 28));
    assert!(text.contains("a=T,U=1"));
    assert_eq!(model.selected_agent_key(), selected.as_ref());
}

#[test]
fn delve_capacity_uses_all_five_station_types_and_ignores_exited_adventurers() {
    use questmancer::domain::Presence;
    let mut model = hall_fit_fixture(StoryContext::fixed(), FitParty::Crowded);
    model.switch_to(View::Delve);
    for (index, agent) in model.domain_mut().agents.values_mut().enumerate() {
        agent.presence = match index {
            0 | 1 => Presence::Working,
            2 | 3 => Presence::Blocked,
            4 | 5 => Presence::Done,
            6 | 7 => Presence::Idle,
            8 | 9 => Presence::Unknown,
            _ => Presence::Exited,
        };
    }
    let mut graphics = WorldFit::new(CellPixels::new(8, 16));
    let (frame, text, _) = draw(&mut graphics, &model, (96, 28));
    assert!(text.contains("a=T,U=1"));
    assert_eq!(frame.actors.len(), 10);
    assert!(frame.interactables.is_empty());
    for actor in &frame.actors {
        assert_ne!(
            model.domain().agents[&actor.agent].presence,
            Presence::Exited
        );
        assert!((0..28).any(|y| (0..96).any(|x| frame.agent_at(x, y) == Some(&actor.agent))));
    }
}

#[test]
fn five_resting_or_unknown_adventurers_keep_the_fitted_dungeon() {
    use questmancer::domain::Presence;
    let mut model = hall_fit_fixture(StoryContext::fixed(), FitParty::Five);
    model.switch_to(View::Delve);
    for (i, a) in model.domain_mut().agents.values_mut().enumerate() {
        a.presence = if i == 3 {
            Presence::Unknown
        } else {
            Presence::Idle
        };
    }
    let mut graphics = WorldFit::new(CellPixels::new(8, 16));
    for (w, h) in [(120, 34), (96, 28), (80, 24)] {
        let (frame, text, _) = draw(&mut graphics, &model, (w, h));
        assert!(
            text.contains("a=T,U=1"),
            "resting crowd must keep the fitted dungeon at {w}x{h}"
        );
        assert_eq!(frame.actors.len(), 5);
        let source = FitScene::project(
            &model,
            Rect::new(0, 1, w, h - 3),
            CellPixels::new(8, 16).unwrap(),
        )
        .unwrap();
        assert!(
            source
                .frame
                .actors
                .iter()
                .all(|a| (a.bounds.width, a.bounds.height) == (16, 24))
        );
        for actor in &frame.actors {
            assert!((0..h).any(|y| (0..w).any(|x| frame.agent_at(x, y) == Some(&actor.agent))));
        }
    }
}

#[test]
fn profile_preserves_fitted_world_scale_and_image_lease_in_both_rooms() {
    for view in [View::Guild, View::Delve] {
        let mut model = hall_fit_fixture(StoryContext::fixed(), FitParty::Five);
        model.switch_to(view);
        let mut graphics = WorldFit::new(CellPixels::new(8, 16));
        let (_, _, removed) = draw(&mut graphics, &model, (96, 28));
        assert!(removed.is_empty());
        assert_eq!(graphics.uploads(), 1);
        model.show_adventurer_card();
        let (_, text, removed) = draw(&mut graphics, &model, (96, 28));
        assert!(text.contains("ADVENTURER"));
        assert!(
            text.contains("need counsel"),
            "{view:?}: profile replaced the fitted world"
        );
        assert!(
            removed.is_empty(),
            "{view:?}: opening profile retired the world image"
        );
        assert_eq!(graphics.uploads(), 1);
        model.dismiss_adventurer_card();
        let (_, _, removed) = draw(&mut graphics, &model, (96, 28));
        assert!(removed.is_empty());
        assert_eq!(
            graphics.uploads(),
            1,
            "{view:?}: closing profile must reuse the same fitted image"
        );
    }
}

#[test]
fn character_picker_keeps_fitted_room_scale_during_preview_resize_and_cancel() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
    use questmancer::{interaction::reduce_scene_action, ui::input::Action};

    for view in [View::Guild, View::Delve] {
        let mut model = hall_fit_fixture(StoryContext::fixed(), FitParty::Five);
        model.switch_to(view);
        let original = model.selected_agent().unwrap().persona.class;
        let mut graphics = WorldFit::new(CellPixels::new(8, 16));
        let (before, _, _) = draw(&mut graphics, &model, (120, 34));
        let open = Event::Key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE));
        let action = graphics.action_for_event(&open, &model, &before);
        let _ = reduce_scene_action(&mut model, action, &before);

        let (scene, text, removed) = draw(&mut graphics, &model, (120, 34));
        assert!(text.contains("CHOOSE CHARACTER"));
        assert!(
            removed.is_empty(),
            "{view:?}: picker retired the fitted room"
        );
        assert_eq!(graphics.uploads(), 1);
        assert!(
            scene
                .actors
                .iter()
                .all(|actor| before.actors.contains(actor))
        );
        let wheel = Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 70,
            row: 12,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(
            graphics.action_for_event(&wheel, &model, &scene),
            Action::Next
        );
        let _ = reduce_scene_action(&mut model, Action::Next, &scene);
        assert!(draw(&mut graphics, &model, (120, 34)).2.is_empty());
        assert_eq!(graphics.uploads(), 1);

        let (resized, text, removed) = draw(&mut graphics, &model, (96, 28));
        assert!(
            text.contains("a=T,U=1"),
            "{view:?}: fitted resize must upload"
        );
        assert_eq!(removed.len(), 1);
        let _ = reduce_scene_action(&mut model, Action::Dismiss, &resized);
        let (restored, text, removed) = draw(&mut graphics, &model, (96, 28));
        assert!(!text.contains("CHOOSE CHARACTER"));
        assert_eq!(restored.actors.len(), before.actors.len());
        assert!(removed.is_empty());
        assert_eq!(graphics.uploads(), 2);
        assert_eq!(model.selected_agent().unwrap().persona.class, original);
    }
}

#[test]
fn automatic_small_party_keeps_its_image_and_safe_hit_map_beneath_profiles() {
    use questmancer::storybook::density::{self, PartySize};
    use questmancer::ui::scene_overlays::adventurer_card_area;
    for view in [View::Guild, View::Delve] {
        let mut model = density::fixture(StoryContext::fixed(), PartySize::ThirtyTwo, view);
        let mut graphics = WorldFit::new(CellPixels::new(8, 16));
        let (_, text, _) = draw(&mut graphics, &model, (96, 28));
        assert!(text.contains("32 adventurers / 8 need counsel"));
        model.show_adventurer_card();
        let (frame, text, removed) = draw(&mut graphics, &model, (96, 28));
        assert!(text.contains("ADVENTURER"));
        assert_eq!(graphics.uploads(), 1);
        assert!(removed.is_empty());
        let card = adventurer_card_area(Rect::new(0, 0, 96, 28), &model).unwrap();
        for y in card.y..card.bottom() {
            for x in card.x..card.right() {
                assert!(frame.target_at(x, y).is_none());
            }
        }
        model.dismiss_adventurer_card();
        assert!(draw(&mut graphics, &model, (96, 28)).2.is_empty());
        assert_eq!(graphics.uploads(), 1);
    }
}

#[test]
fn covered_profile_clicks_never_dispatch_to_hidden_adventurers() {
    use crossterm::event::{Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
    use questmancer::{
        interaction::reduce_scene_action,
        ui::{input::Action, scene_overlays::adventurer_card_area},
    };
    for view in [View::Guild, View::Delve] {
        let mut model = hall_fit_fixture(StoryContext::fixed(), FitParty::Five);
        model.switch_to(view);
        let mut graphics = WorldFit::new(CellPixels::new(8, 16));
        let (before, _, _) = draw(&mut graphics, &model, (120, 34));
        model.show_adventurer_card();
        let (scene, _, _) = draw(&mut graphics, &model, (120, 34));
        let area = adventurer_card_area(Rect::new(0, 0, 120, 34), &model).unwrap();
        let selection = model.selected_agent_key().cloned();
        for y in area.y..area.bottom() {
            for x in area.x..area.right() {
                assert!(scene.target_at(x, y).is_none());
                let click = Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Left),
                    column: x,
                    row: y,
                    modifiers: KeyModifiers::NONE,
                });
                let action = graphics.action_for_event(&click, &model, &scene);
                assert_eq!(action, Action::None);
                let result = reduce_scene_action(&mut model, action, &scene);
                assert!(result.commands.is_empty());
            }
        }
        assert_eq!(model.selected_agent_key(), selection.as_ref());
        assert!(model.adventurer_card_visible());
        assert!(
            scene.actors.len() < before.actors.len(),
            "covered actors must leave the hit map"
        );
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('j'),
            KeyModifiers::NONE,
        );
        assert_eq!(
            graphics.action_for_event(&Event::Key(key), &model, &scene),
            Action::Next
        );
    }
}

#[test]
fn profile_open_at_start_and_full_occlusion_do_not_lose_first_world_upload() {
    use questmancer::app::Motion;
    for view in [View::Guild, View::Delve] {
        let mut model = hall_fit_fixture(StoryContext::fixed(), FitParty::Five);
        model.switch_to(view);
        let mut preferences = *model.preferences();
        preferences.motion = Motion::Full;
        model.set_preferences(preferences);
        model.show_adventurer_card();
        let mut graphics = WorldFit::new(CellPixels::new(8, 16));
        let (frame, text, removed) = draw(&mut graphics, &model, (60, 14));
        assert!(text.contains("ADVENTURER"));
        assert!(
            !text.contains("a=T,U=1"),
            "fully hidden world must defer upload"
        );
        assert!(frame.actors.is_empty());
        assert_eq!(frame.next_frame_in, None);
        assert!(removed.is_empty());
        assert_eq!(graphics.uploads(), 1);
        model.dismiss_adventurer_card();
        let (frame, text, removed) = draw(&mut graphics, &model, (60, 14));
        assert!(
            text.contains("a=T,U=1"),
            "first exposure must transmit the cached world"
        );
        assert_eq!(frame.actors.len(), 5);
        assert!(frame.next_frame_in.is_some());
        assert_eq!(graphics.uploads(), 1);
        assert!(removed.is_empty());
        for _ in 0..3 {
            model.show_adventurer_card();
            let (_, _, removed) = draw(&mut graphics, &model, (60, 14));
            assert!(removed.is_empty());
            model.dismiss_adventurer_card();
            let (_, _, removed) = draw(&mut graphics, &model, (60, 14));
            assert!(removed.is_empty());
        }
        assert_eq!(graphics.uploads(), 1);
    }
}

#[test]
fn visible_world_upload_prefixes_stay_outside_profile_at_every_review_size() {
    use questmancer::ui::scene_overlays::adventurer_card_area;
    for view in [View::Guild, View::Delve] {
        for (width, height) in [
            (60, 14),
            (80, 14),
            (80, 24),
            (96, 20),
            (120, 34),
            (200, 40),
            (60, 90),
        ] {
            let mut model = hall_fit_fixture(StoryContext::fixed(), FitParty::Five);
            model.switch_to(view);
            model.show_adventurer_card();
            let mut graphics = WorldFit::new(CellPixels::new(8, 16));
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            let mut rgb = RgbBuffer::filled(0, 0, Rgb::BLACK);
            terminal
                .draw(|f| {
                    render_application(f, &model, &mut rgb, None, &mut graphics);
                })
                .unwrap();
            let card = adventurer_card_area(Rect::new(0, 0, width, height), &model).unwrap();
            for y in card.y..card.bottom() {
                for x in card.x..width {
                    let symbol = terminal.backend().buffer().cell((x, y)).unwrap().symbol();
                    assert!(
                        !symbol.contains("a=T,U=1") && !symbol.contains('\u{10eeee}'),
                        "{view:?} world prefix inside profile at {x},{y}"
                    );
                }
            }
            assert!(graphics.finish_frame().is_empty());
        }
    }
}
