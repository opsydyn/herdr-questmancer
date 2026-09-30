use questmancer::{
    app::{Model, View},
    domain::Timestamp,
    work::{self, QuestId, QuestState, SourceId},
};

fn model() -> Model {
    let mut model = Model::new(View::Guild);
    model.set_now(Timestamp::from_millis(100));
    let mut work = work::Projection::default();
    for event in work::fixture::FixtureSource::authentication(model.now()) {
        work = work::reduce(work, event).unwrap();
    }
    model.replace_work(work);
    model
}

#[test]
fn unknown_and_closed_use_neutral_map_markers_without_working_or_success_cues() {
    use questmancer::scene::{
        pixel::{PixelSize, Rgb, RgbBuffer},
        render::war_table::paint,
        war_table::TablePlan,
    };
    fn painted(state: QuestState) -> RgbBuffer {
        let mut model = model();
        let source = SourceId::new("fixture-auth");
        let id = QuestId::new(&source, "api");
        let mut projection = model.work().clone();
        projection
            .sources
            .get_mut(&source)
            .unwrap()
            .inventory
            .quests
            .get_mut(&id)
            .unwrap()
            .state = state;
        model.replace_work(projection);
        model.open_war_table();
        let graph = work::graph::CampaignGraph::from_model(&model).unwrap();
        let plan = TablePlan::project(Some(&graph), PixelSize::new(120, 76));
        let mut buffer = RgbBuffer::filled(120, 76, Rgb::BLACK);
        paint(Some(&graph), &plan, &mut buffer);
        buffer
    }
    assert_eq!(painted(QuestState::Unknown), painted(QuestState::Planned));
    assert_eq!(painted(QuestState::Closed), painted(QuestState::Cancelled));
    assert_ne!(painted(QuestState::Unknown), painted(QuestState::Active));
    assert_ne!(painted(QuestState::Closed), painted(QuestState::Completed));
}

#[test]
fn table_projects_dependencies_without_turning_attention_into_work_state() {
    let model = model();
    let before = model.work().clone();
    let graph = work::graph::CampaignGraph::from_model(&model).unwrap();
    let source = SourceId::new("fixture-auth");
    let web = QuestId::new(&source, "web");
    let tests = QuestId::new(&source, "tests");
    assert!(graph.edges.contains(&(web.clone(), tests.clone())));
    assert_eq!(graph.nodes[&web].state, QuestState::Active);
    assert_eq!(graph.nodes[&web].human_requests, 1);
    assert_eq!(graph.nodes[&tests].layer, Some(1));
    assert_eq!(model.work(), &before);
}

#[test]
fn cycles_missing_and_outside_dependencies_are_attributed_without_readiness() {
    use work::{CampaignId, CampaignState, WorkCampaign};
    let mut model = model();
    let source = SourceId::new("fixture-auth");
    let api = QuestId::new(&source, "api");
    let web = QuestId::new(&source, "web");
    let tests = QuestId::new(&source, "tests");
    let outside = QuestId::new(&source, "other");
    let missing = QuestId::new(&source, "missing");
    let other = CampaignId::new(&source, "other");
    let mut state = model.work().clone();
    let inventory = &mut state.sources.get_mut(&source).unwrap().inventory;
    inventory.campaigns.insert(
        other.clone(),
        WorkCampaign {
            id: other.clone(),
            revision: 1,
            name: "Other campaign".into(),
            objective: None,
            state: CampaignState::Active,
        },
    );
    let mut q = inventory.quests[&api].clone();
    q.id = outside.clone();
    q.campaign = other;
    q.dependencies.clear();
    inventory.quests.insert(outside.clone(), q);
    inventory.quests.get_mut(&api).unwrap().dependencies = vec![outside.clone(), missing.clone()];
    inventory.quests.get_mut(&web).unwrap().dependencies = vec![tests.clone()];
    model.replace_work(state);
    model.open_war_table();
    let graph = work::graph::CampaignGraph::from_model(&model).unwrap();
    assert!(graph.has_cycle());
    assert!(graph.nodes[&web].cyclic);
    assert!(graph.nodes[&tests].cyclic);
    assert!(!graph.nodes[&api].cyclic);
    assert_eq!(graph.nodes[&web].layer, None);
    assert_eq!(graph.nodes[&api].outside_dependencies, vec![outside]);
    assert_eq!(graph.nodes[&api].missing_dependencies, vec![missing]);
    assert!(!model.work().dependencies_ready(&api));
    assert!(!model.work().dependencies_ready(&tests));
    assert_eq!(
        questmancer::scene::war_table::TablePlan::project(
            Some(&graph),
            questmancer::scene::pixel::PixelSize::new(120, 76)
        )
        .mode,
        questmancer::scene::war_table::MapMode::Inventory
    );
}

#[test]
fn navigation_reaches_every_quest_and_source_qualified_campaign_and_repairs_removal() {
    use questmancer::{app::Modal, interaction::reduce_action, ui::input::Action};
    use work::{CampaignId, CampaignState, Inventory, Message, WorkCampaign};
    let mut model = model();
    let source = SourceId::new("other-home");
    let campaign = CampaignId::new(&source, "auth");
    let mut inventory = Inventory::default();
    inventory.campaigns.insert(
        campaign.clone(),
        WorkCampaign {
            id: campaign.clone(),
            revision: 1,
            name: "Replace Authentication Layer".into(),
            objective: None,
            state: CampaignState::Active,
        },
    );
    let state = work::reduce(
        model.work().clone(),
        Message::Baseline {
            source,
            generation: 1,
            captured_at: model.now(),
            cursor: 0,
            inventory,
            complete: true,
            bindings: std::collections::BTreeMap::default(),
        },
    )
    .unwrap();
    model.replace_work(state);
    let before = model.work().clone();
    let action = reduce_action(&mut model, Action::OpenWarTable);
    assert!(
        action.commands.is_empty() && action.persistence.is_empty() && action.work_intent.is_none()
    );
    let ids = work::graph::CampaignGraph::from_model(&model)
        .unwrap()
        .nodes
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    for id in &ids {
        assert_eq!(work::view::selected_quest(&model), Some(id));
        let action = reduce_action(&mut model, Action::Next);
        assert!(action.commands.is_empty());
    }
    assert_eq!(model.work(), &before);
    let _ = reduce_action(&mut model, Action::NextCampaign);
    assert_eq!(model.war_campaign(), Some(&campaign));
    assert!(model.work_selection().is_none());
    assert!(
        work::graph::details(&model)
            .iter()
            .any(|s| s.contains("No quests are supplied"))
    );
    let _ = reduce_action(&mut model, Action::PreviousResult);
    let selected = work::view::selected_quest(&model).unwrap().clone();
    let mut state = model.work().clone();
    let inventory = &mut state
        .sources
        .get_mut(&SourceId::new("fixture-auth"))
        .unwrap()
        .inventory;
    inventory.quests.remove(&selected);
    inventory
        .obligations
        .retain(|_, o| o.quest.as_ref() != Some(&selected));
    model.replace_work(state);
    assert_ne!(work::view::selected_quest(&model), Some(&selected));
    assert_eq!(model.modal(), &Modal::WarTable);
    let _ = reduce_action(&mut model, Action::OpenQuestBoard);
    assert_eq!(model.modal(), &Modal::QuestBoard);
    let _ = reduce_action(&mut model, Action::OpenWarTable);
    assert_eq!(model.modal(), &Modal::WarTable);
    assert!(
        reduce_action(&mut model, Action::Observe)
            .commands
            .is_empty()
    );
    let _ = reduce_action(&mut model, Action::NextUrgent);
    assert_eq!(model.modal(), &Modal::QuestBoard);
}

#[test]
fn deferred_request_remains_a_fact_without_becoming_an_alert() {
    let mut model = model();
    let source = SourceId::new("fixture-auth");
    let id = work::ObligationId::new(&source, "compatibility");
    let mut state = model.work().clone();
    state.acknowledge(&id, Some(Timestamp::from_millis(9999)));
    state.restore_cached();
    model.replace_work(state);
    model.open_war_table();
    let graph = work::graph::CampaignGraph::from_model(&model).unwrap();
    let node = &graph.nodes[&QuestId::new(&source, "web")];
    assert_eq!(node.human_requests, 1);
    assert_eq!(node.eligible_requests, 0);
    assert!(work::queue(&model).is_empty());
    assert_eq!(graph.freshness, work::Freshness::Cached);
}

#[test]
fn table_renders_deterministically_without_domain_mutation_or_animation() {
    use questmancer::{
        app::Motion,
        scene::{
            pixel::{PixelSize, Rgb, RgbBuffer},
            presentation::ScenePresentation,
            render_scene_for_world,
            snapshot::SceneSnapshot,
            war_table::TablePlan,
        },
    };
    let mut model = model();
    model.open_war_table();
    let before = model.work().clone();
    for motion in [Motion::Full, Motion::Reduced, Motion::None] {
        let mut preferences = *model.preferences();
        preferences.motion = motion;
        model.set_preferences(preferences);
        let snapshot = SceneSnapshot::from_model(&model);
        let presentation = ScenePresentation::from_model(&model);
        for (width, height) in [(1, 1), (20, 14), (48, 56), (80, 24), (120, 76)] {
            let viewport = PixelSize::new(width, height);
            let mut a = RgbBuffer::filled(0, 0, Rgb::BLACK);
            let mut b = a.clone();
            let first = render_scene_for_world(&snapshot, &presentation, viewport, &mut a);
            let second = render_scene_for_world(&snapshot, &presentation, viewport, &mut b);
            assert_eq!(a, b);
            assert_eq!(first, second);
            assert!(first.next_frame_in.is_none());
            assert!(first.actors.is_empty());
            assert!(first.interactables.is_empty());
            let plan = TablePlan::project(presentation.war_table.as_ref(), viewport);
            if let Some(map) = plan.map {
                for card in &plan.cards {
                    assert!(
                        card.bounds.x >= map.x
                            && card.bounds.y >= map.y
                            && card.bounds.x + i32::from(card.bounds.width)
                                <= map.x + i32::from(map.width)
                            && card.bounds.y + i32::from(card.bounds.height)
                                <= map.y + i32::from(map.height)
                    );
                }
            }
        }
    }
    assert_eq!(model.work(), &before);
}

#[test]
fn narrow_details_scroll_to_literal_requests_and_large_inventory_keeps_selection() {
    use questmancer::{
        interaction::reduce_action,
        scene::{
            pixel::PixelSize,
            war_table::{MapMode, TablePlan},
        },
        ui::input::Action,
    };
    let mut model = model();
    model.select_work(work::WorkSelection::Quest(QuestId::new(
        &SourceId::new("fixture-auth"),
        "web",
    )));
    model.open_war_table();
    model.set_work_viewport_size(48, 28);
    assert_eq!(
        TablePlan::project(
            work::graph::CampaignGraph::from_model(&model).as_ref(),
            PixelSize::new(48, 56)
        )
        .mode,
        MapMode::DetailsOnly
    );
    let lines = work::graph::details(&model).join("\n");
    assert!(lines.contains("Should the old token endpoint remain for backwards compatibility?"));
    assert!(lines.contains("Settled only by: A matching source resolution of compatibility-1."));
    for _ in 0..150 {
        let _ = reduce_action(&mut model, Action::ScrollDown);
    }
    let text = render_text(&model, 48, 28);
    assert!(text.contains("compatibility-1."), "{text}");
    for width in 1..=125 {
        for height in [1, 6, 12, 28] {
            let _ = render_text(&model, width, height);
        }
    }
}

fn render_text(model: &Model, width: u16, height: u16) -> String {
    let mut terminal =
        ratatui::Terminal::new(ratatui::backend::TestBackend::new(width, height)).unwrap();
    terminal
        .draw(|frame| {
            questmancer::ui::scene_overlays::render_scene_overlays(
                frame,
                model,
                &questmancer::scene::presentation::ScenePresentation::from_model(model),
                None,
            );
        })
        .unwrap();
    terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect()
}

#[cfg(feature = "storybook")]
#[test]
fn large_inventory_pages_every_selected_quest_without_hiding_dependency_facts() {
    use questmancer::{
        scene::{
            pixel::PixelSize,
            war_table::{MapMode, TablePlan},
        },
        storybook::fixtures::{StoryContext, WarTableState, war_table_fixture},
    };
    let mut model = war_table_fixture(StoryContext::fixed(), WarTableState::Large);
    let before = model.work().clone();
    for _ in 0..100 {
        let graph = work::graph::CampaignGraph::from_model(&model).unwrap();
        assert_eq!(graph.nodes.len(), 100);
        let plan = TablePlan::project(Some(&graph), PixelSize::new(120, 76));
        assert_eq!(plan.mode, MapMode::Inventory);
        assert!(
            plan.cards
                .iter()
                .any(|p| Some(&p.id) == graph.selected.as_ref())
        );
        assert!(plan.cards.len() <= 6);
        model.cycle_war_quest(true);
    }
    assert_eq!(model.work(), &before);
}

#[test]
fn table_input_is_contextual_and_ignores_agent_editing_keys() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use questmancer::{
        app::Modal,
        interaction::reduce_action,
        ui::input::{Action, action_for, action_for_event_in},
    };
    // Global opening does not steal printable text from a counsel draft.
    assert_eq!(
        action_for(KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE)),
        Action::OpenWarTable
    );
    assert_eq!(
        action_for_event_in(
            &crossterm::event::Event::Key(KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE)),
            &Modal::Counsel {
                draft: "kept".into(),
                phase: questmancer::app::CounselPhase::default()
            }
        ),
        Action::TypeCharacter('w')
    );
    let mut model = model();
    let _ = reduce_action(&mut model, Action::OpenWarTable);
    for (code, action) in [
        (KeyCode::Char('j'), Action::Next),
        (KeyCode::Tab, Action::NextCampaign),
        (KeyCode::BackTab, Action::PreviousResult),
        (KeyCode::Down, Action::ScrollDown),
        (KeyCode::Char('b'), Action::OpenQuestBoard),
        (KeyCode::Char('r'), Action::None),
        (KeyCode::Char('s'), Action::None),
    ] {
        assert_eq!(
            action_for_event_in(
                &crossterm::event::Event::Key(KeyEvent::new(code, KeyModifiers::NONE)),
                &Modal::WarTable
            ),
            action
        );
    }
    let event = crossterm::event::Event::Mouse(crossterm::event::MouseEvent {
        kind: crossterm::event::MouseEventKind::ScrollDown,
        column: 1,
        row: 1,
        modifiers: KeyModifiers::NONE,
    });
    assert_eq!(
        action_for_event_in(&event, &Modal::WarTable),
        Action::ScrollDown
    );
}

proptest::proptest! {
    #[test]
    fn generated_dag_layers_keep_every_prerequisite_before_its_dependent(flags in proptest::collection::vec(proptest::bool::ANY,1..35)) {
        let source=SourceId::new("generated");let campaign=work::CampaignId::new(&source,"campaign");
        let mut inventory=work::Inventory::default();inventory.campaigns.insert(campaign.clone(),work::WorkCampaign{id:campaign.clone(),revision:1,name:"Generated".into(),objective:None,state:work::CampaignState::Active});
        for (i, chained) in flags.iter().enumerate() {
            let id=QuestId::new(&source,&format!("quest-{i:03}"));
            inventory.quests.insert(id.clone(),work::Quest{id,campaign:campaign.clone(),revision:1,title:format!("Quest {i}"),description:None,kind:work::QuestKind::Ship,state:QuestState::Planned,assignment:None,dependencies:if i==0 {vec![]} else {vec![QuestId::new(&source,&format!("quest-{:03}",if *chained {i-1}else{0}))]}});
        }
        let state=work::reduce(work::Projection::default(),work::Message::Baseline{source,generation:1,captured_at:Timestamp::from_millis(1),cursor:0,inventory,complete:true,bindings:std::collections::BTreeMap::default()}).unwrap();
        let graph=work::graph::CampaignGraph::project(&state,&campaign,None,Timestamp::from_millis(1)).unwrap();
        proptest::prop_assert!(!graph.has_cycle());proptest::prop_assert_eq!(graph.nodes.len(),flags.len());
        for (from,to) in &graph.edges {proptest::prop_assert!(graph.nodes[from].layer.unwrap()<graph.nodes[to].layer.unwrap());}
        proptest::prop_assert!(state.sources.values().all(|s|s.inventory.quests.values().all(|q|q.state==QuestState::Planned)));
    }
}

#[cfg(feature = "storybook")]
#[test]
fn production_table_review_exports_when_requested() {
    use questmancer::{
        app::ColorMode,
        scene::{
            pixel::{PixelSize, Rgb, RgbBuffer},
            presentation::ScenePresentation,
            render_scene_for_world,
            snapshot::SceneSnapshot,
        },
        storybook::fixtures::{StoryContext, WarTableState, war_table_fixture},
        ui::scene_adapter::flush_rgb,
    };
    fn export_table(model: &Model, path: &std::path::Path, width: u16, height: u16) {
        let mut terminal =
            ratatui::Terminal::new(ratatui::backend::TestBackend::new(width, height)).unwrap();
        let presentation = ScenePresentation::from_model(model);
        let snapshot = SceneSnapshot::from_model(model);
        let mut rgb = RgbBuffer::filled(0, 0, Rgb::BLACK);
        terminal
            .draw(|frame| {
                let scene = render_scene_for_world(
                    &snapshot,
                    &presentation,
                    PixelSize::new(width, height * 2),
                    &mut rgb,
                );
                assert!(scene.next_frame_in.is_none());
                flush_rgb(
                    frame.buffer_mut(),
                    ratatui::layout::Rect::new(0, 0, width, height),
                    &rgb,
                    Rgb::BLACK,
                    ColorMode::Xterm256,
                );
                questmancer::ui::scene_overlays::render_scene_overlays(
                    frame,
                    model,
                    &presentation,
                    None,
                );
            })
            .unwrap();
        let cells=terminal.backend().buffer().content.iter().map(|c|serde_json::json!({"s":c.symbol(),"fg":format!("{:?}",c.fg),"bg":format!("{:?}",c.bg)})).collect::<Vec<_>>();
        std::fs::write(
            path,
            serde_json::to_vec_pretty(
                &serde_json::json!({"width":width,"height":height,"cells":cells}),
            )
            .unwrap(),
        )
        .unwrap();
    }
    let Some(path) = std::env::var_os("QUESTMANCER_TABLE_EXPORT") else {
        return;
    };
    let path = std::path::PathBuf::from(path);
    std::fs::create_dir_all(&path).unwrap();
    let variants = [
        ("populated", WarTableState::Populated, 120, 38),
        ("narrow", WarTableState::Narrow, 48, 28),
        ("empty", WarTableState::Empty, 120, 38),
        ("offline", WarTableState::Offline, 120, 38),
        ("cyclic", WarTableState::Cyclic, 120, 38),
        ("large", WarTableState::Large, 120, 38),
    ];
    for (name, state, width, height) in variants {
        let mut model = war_table_fixture(StoryContext::fixed(), state);
        model.set_work_viewport_size(width, height);
        export_table(&model, &path.join(format!("{name}.json")), width, height);
        if name == "large" {
            for _ in 0..96 {
                model.cycle_war_quest(true);
            }
            export_table(&model, &path.join("large-later-page.json"), width, height);
        }
        if name == "populated" {
            let mut state = model.work().clone();
            state.restore_cached();
            model.replace_work(state);
            export_table(&model, &path.join("cached.json"), width, height);
        }
    }
}

#[test]
fn text_parchment_clears_rgb_half_blocks_before_writing() {
    use questmancer::{
        app::ColorMode,
        scene::{
            pixel::{PixelSize, Rgb, RgbBuffer},
            presentation::ScenePresentation,
            render_scene_for_world,
            snapshot::SceneSnapshot,
            war_table::TablePlan,
        },
        ui::scene_adapter::flush_rgb,
    };
    let mut model = model();
    model.open_war_table();
    let presentation = ScenePresentation::from_model(&model);
    let plan = TablePlan::project(presentation.war_table.as_ref(), PixelSize::new(120, 76));
    let mut pixels = RgbBuffer::filled(0, 0, Rgb::BLACK);
    let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(120, 38)).unwrap();
    terminal
        .draw(|frame| {
            let _ = render_scene_for_world(
                &SceneSnapshot::from_model(&model),
                &presentation,
                PixelSize::new(120, 76),
                &mut pixels,
            );
            flush_rgb(
                frame.buffer_mut(),
                ratatui::layout::Rect::new(0, 0, 120, 38),
                &pixels,
                Rgb::BLACK,
                ColorMode::Xterm256,
            );
            questmancer::ui::scene_overlays::render_scene_overlays(
                frame,
                &model,
                &presentation,
                None,
            );
        })
        .unwrap();
    let rect = plan.details;
    for y in
        u16::try_from(rect.y / 2).unwrap()..u16::try_from(rect.y / 2).unwrap() + rect.height / 2
    {
        for x in u16::try_from(rect.x).unwrap()..u16::try_from(rect.x).unwrap() + rect.width {
            assert_ne!(
                terminal.backend().buffer()[(x, y)].symbol(),
                "▀",
                "parchment retained a recoloured RGB half-block"
            );
        }
    }
}
