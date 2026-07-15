use herdr_webmaster::{
    app::{CharacterSet, ColorMode, ConnectionState, DisplayPreferences, Model, Motion, View},
    domain::{
        AgentKey, Attention, AttentionReason, DomainState, PaneId, Presence, Site, Timestamp,
        WorkspaceId,
    },
    herdr::protocol::{SessionSnapshotResult, SuccessResponse},
    ui,
};
use ratatui::{Terminal, backend::TestBackend, style::Color};

fn three_agent_model() -> Model {
    let response: SuccessResponse<SessionSnapshotResult> =
        serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
    let source =
        DomainState::from_snapshot(&response.result.snapshot, Timestamp::from_millis(1_000))
            .agents
            .into_values()
            .next()
            .unwrap();

    let mut alpha = source.clone();
    alpha.key = AgentKey::new("agent-a");
    alpha.pane_id = PaneId::new("w1:p1");
    "Alpha".clone_into(&mut alpha.name);
    alpha.presence = Presence::Working;
    alpha.attention = Attention::Clear;
    alpha.focused = true;

    let mut beta = source.clone();
    beta.key = AgentKey::new("agent-b");
    beta.pane_id = PaneId::new("w1:p2");
    "Beta".clone_into(&mut beta.name);
    beta.presence = Presence::Blocked;
    beta.attention = Attention::unseen(AttentionReason::NeedsInput, Timestamp::from_millis(2_000));
    beta.focused = false;

    let mut gamma = source;
    gamma.key = AgentKey::new("agent-c");
    gamma.pane_id = PaneId::new("w1:p3");
    "Gamma".clone_into(&mut gamma.name);
    gamma.presence = Presence::Exited;
    gamma.attention = Attention::Clear;
    gamma.focused = false;

    let mut domain = DomainState::default();
    domain.agents.insert(alpha.key.clone(), alpha);
    domain.agents.insert(beta.key.clone(), beta);
    domain.agents.insert(gamma.key.clone(), gamma);
    domain.selected_agent = Some(AgentKey::new("agent-b"));

    let mut model = Model::new(View::Cafe);
    model.replace_domain(domain);
    model.set_connection(ConnectionState::Connected);
    model.set_now(Timestamp::from_millis(2_500));
    model
}

fn render(model: &Model, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|frame| ui::render(frame, model)).unwrap();
    let buffer = terminal.backend().buffer();
    (0..height)
        .map(|y| {
            (0..width)
                .map(|x| buffer.cell((x, y)).unwrap().symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_colours(model: &Model, width: u16, height: u16) -> Vec<(Color, Color)> {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|frame| ui::render(frame, model)).unwrap();
    terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| (cell.fg, cell.bg))
        .collect()
}

fn assert_every_agent_is_visible(screen: &str) {
    for name in ["Alpha", "Beta", "Gamma"] {
        assert!(screen.contains(name), "missing {name}:\n{screen}");
    }
}

#[test]
fn selected_and_disconnected_workstations_remain_in_the_same_authored_scene() {
    let mut model = three_agent_model();
    model
        .domain_mut()
        .agents
        .get_mut(&AgentKey::new("agent-a"))
        .unwrap()
        .presence = Presence::Exited;
    model
        .domain_mut()
        .agents
        .get_mut(&AgentKey::new("agent-a"))
        .unwrap()
        .focused = false;
    model.domain_mut().selected_agent = Some(AgentKey::new("agent-b"));

    let screen = render(&model, 120, 30);

    assert!(screen.contains("Alpha"));
    assert!(screen.contains("BROKEN") || screen.contains("EMPTY CHAIR"));
    assert!(screen.contains("> Beta"));
    assert!(screen.contains("HELP!"));
    assert!(
        !screen.contains("CAFE WALL / 56K CABLE RUN"),
        "nested webmaster output leaked into the cafe:\n{screen}"
    );
}

#[test]
fn one_hundred_twenty_columns_show_authored_bay_and_selected_workstation() {
    let screen = render(&three_agent_model(), 120, 30);

    assert_every_agent_is_visible(&screen);
    assert!(
        screen.contains("w1"),
        "missing workspace signage:\n{screen}"
    );
    assert!(screen.contains("AISLE"), "missing aisle cue:\n{screen}");
    assert!(screen.contains("== == =="), "missing floor cue:\n{screen}");
    assert!(
        screen.contains("> Beta"),
        "missing selected workstation:\n{screen}"
    );
    assert!(screen.contains("HELP!"), "missing state theatre:\n{screen}");
}

#[test]
fn one_hundred_sixty_columns_keep_three_agents_in_authored_room() {
    let screen = render(&three_agent_model(), 160, 50);

    assert!(screen.contains("Alpha") && screen.contains("Beta"));
    assert!(
        screen.contains("w1"),
        "missing workspace signage:\n{screen}"
    );
    assert!(screen.contains("CRT"), "missing furniture cue:\n{screen}");
    assert!(screen.contains("> Beta"), "missing selection:\n{screen}");
    assert!(screen.contains("[!] HELP!"), "missing state:\n{screen}");
    let alpha = screen.find("Alpha").unwrap();
    let beta = screen.find("Beta").unwrap();
    let gamma = screen.find("Gamma").unwrap();
    assert!(
        alpha < beta && beta < gamma,
        "unstable row-major order:\n{screen}"
    );
}

#[test]
fn multiple_workspaces_render_as_connected_bays_with_deterministic_variant_cues() {
    let mut model = three_agent_model();
    let gamma_key = AgentKey::new("agent-c");
    model
        .domain_mut()
        .agents
        .get_mut(&gamma_key)
        .unwrap()
        .workspace_id = WorkspaceId::new("w2");
    model.domain_mut().sites.insert(
        WorkspaceId::new("w1"),
        Site {
            workspace_id: WorkspaceId::new("w1"),
            label: "w1".into(),
            cwd: "/tmp/w1".into(),
            agents: vec![AgentKey::new("agent-a"), AgentKey::new("agent-b")],
        },
    );
    model.domain_mut().sites.insert(
        WorkspaceId::new("w2"),
        Site {
            workspace_id: WorkspaceId::new("w2"),
            label: "w2".into(),
            cwd: "/tmp/w2".into(),
            agents: vec![gamma_key],
        },
    );
    let screen = render(&model, 160, 40);
    assert!(screen.contains("w1"), "missing first bay:\n{screen}");
    assert!(screen.contains("w2"), "missing second bay:\n{screen}");
    assert!(
        screen.contains("AISLE"),
        "missing connected-room cue:\n{screen}"
    );
    assert_every_agent_is_visible(&screen);
}

#[test]
fn authored_variants_change_rendered_room_geometry() {
    let mut ids = Vec::new();
    for index in 0..128 {
        let id = WorkspaceId::new(format!("variant-{index}"));
        let variant = herdr_webmaster::ui::cafe_scene::variant_for_workspace(&id);
        if !ids.iter().any(|(known, _)| *known == variant) {
            ids.push((variant, id));
        }
        if ids.len() == 3 {
            break;
        }
    }
    let mut screens = Vec::new();
    for (variant, workspace) in ids {
        let mut model = three_agent_model();
        for agent in model.domain_mut().agents.values_mut() {
            agent.workspace_id = workspace.clone();
        }
        model.domain_mut().sites.clear();
        let keys = model.domain().agents.keys().cloned().collect();
        model.domain_mut().sites.insert(
            workspace.clone(),
            Site {
                workspace_id: workspace,
                label: format!("{variant:?}"),
                cwd: "/tmp".into(),
                agents: keys,
            },
        );
        screens.push(render(&model, 120, 30));
    }
    assert_eq!(screens.len(), 3);
    assert!(screens.windows(2).all(|pair| pair[0] != pair[1]));
}

#[test]
fn eighty_by_twenty_four_keeps_a_compact_bay_strip_and_actions() {
    let mut model = three_agent_model();
    model.domain_mut().sites.insert(
        WorkspaceId::new("w1"),
        Site {
            workspace_id: WorkspaceId::new("w1"),
            label: "w1".into(),
            cwd: "/tmp/w1".into(),
            agents: vec![
                AgentKey::new("agent-a"),
                AgentKey::new("agent-b"),
                AgentKey::new("agent-c"),
            ],
        },
    );
    model.domain_mut().sites.insert(
        WorkspaceId::new("w2"),
        Site {
            workspace_id: WorkspaceId::new("w2"),
            label: "w2".into(),
            cwd: "/tmp/w2".into(),
            agents: vec![],
        },
    );
    let screen = render(&model, 80, 24);
    assert!(
        screen.contains("[w1] [w2]"),
        "missing compact bay strip:\n{screen}"
    );
    assert!(screen.contains("[j/k] navigate"));
    assert!(screen.contains("[enter] visit"));
}

#[test]
fn ascii_multi_workspace_transitions_remain_ascii_safe() {
    let mut model = three_agent_model();
    model.set_preferences(DisplayPreferences {
        character_set: CharacterSet::Ascii,
        ..DisplayPreferences::default()
    });
    model.domain_mut().sites.insert(
        WorkspaceId::new("w1"),
        Site {
            workspace_id: WorkspaceId::new("w1"),
            label: "w1".into(),
            cwd: "/tmp/w1".into(),
            agents: vec![
                AgentKey::new("agent-a"),
                AgentKey::new("agent-b"),
                AgentKey::new("agent-c"),
            ],
        },
    );
    model.domain_mut().sites.insert(
        WorkspaceId::new("w2"),
        Site {
            workspace_id: WorkspaceId::new("w2"),
            label: "w2".into(),
            cwd: "/tmp/w2".into(),
            agents: vec![],
        },
    );
    let screen = render(&model, 120, 30);
    assert!(screen.is_ascii(), "non-ASCII transition:\n{screen}");
    assert!(screen.contains('+'));
    assert!(screen.contains("w1") && screen.contains("w2"));
}

#[test]
fn compact_selected_wrapped_workspace_remaps_seats_into_active_scene() {
    let mut model = three_agent_model();
    let gamma = model
        .domain_mut()
        .agents
        .get_mut(&AgentKey::new("agent-c"))
        .unwrap();
    gamma.workspace_id = WorkspaceId::new("w2");
    model.domain_mut().selected_agent = Some(AgentKey::new("agent-c"));
    model.domain_mut().sites.insert(
        WorkspaceId::new("w1"),
        Site {
            workspace_id: WorkspaceId::new("w1"),
            label: "w1".into(),
            cwd: "/tmp/w1".into(),
            agents: vec![AgentKey::new("agent-a"), AgentKey::new("agent-b")],
        },
    );
    model.domain_mut().sites.insert(
        WorkspaceId::new("w2"),
        Site {
            workspace_id: WorkspaceId::new("w2"),
            label: "w2".into(),
            cwd: "/tmp/w2".into(),
            agents: vec![AgentKey::new("agent-c")],
        },
    );
    let screen = render(&model, 80, 24);
    assert!(
        screen.contains("Gamma"),
        "selected wrapped agent missing:\n{screen}"
    );
    assert!(screen.contains("[w1] [w2]"));
    assert!(
        !screen.contains("Gamma\n")
            || screen.find("Gamma").unwrap() < screen.find("[w1] [w2]").unwrap()
    );
}

#[test]
fn compact_selection_targets_the_selected_overflow_bay() {
    let mut model = three_agent_model();
    let template = model.domain().agents.values().next().unwrap().clone();
    let mut keys = Vec::new();
    for index in 0..5 {
        let mut agent = template.clone();
        agent.key = AgentKey::new(format!("overflow-{index}"));
        agent.name = format!("Overflow {index}");
        keys.push(agent.key.clone());
        model.domain_mut().agents.insert(agent.key.clone(), agent);
    }
    model.domain_mut().sites.clear();
    model.domain_mut().sites.insert(
        WorkspaceId::new("overflow"),
        Site {
            workspace_id: WorkspaceId::new("overflow"),
            label: "overflow".into(),
            cwd: "/tmp".into(),
            agents: keys.clone(),
        },
    );
    model.domain_mut().selected_agent = Some(keys[4].clone());
    let screen = render(&model, 80, 24);
    assert!(
        screen.contains("Overflow 4"),
        "selected overflow bay not visible above strip:\n{screen}"
    );
    assert!(screen.find("Overflow 4").unwrap() < screen.find("[overflow]").unwrap());
}

#[test]
fn eighty_columns_keep_authored_bay_and_actions() {
    let mut model = three_agent_model();
    let screen = render(&model, 80, 24);

    assert!(screen.contains("Alpha") && screen.contains("Beta"));
    assert!(
        screen.contains("w1"),
        "missing workspace signage:\n{screen}"
    );
    assert!(screen.contains("AISLE"), "missing aisle cue:\n{screen}");
    assert!(screen.contains("== == =="), "missing floor cue:\n{screen}");
    assert!(screen.contains("HELP!"), "missing blocked state:\n{screen}");
    for action in [
        "[1] desk",
        "[2] cafe",
        "[j/k] navigate",
        "[enter] visit",
        "[r] reply",
        "[o] refresh",
        "[space] seen",
        "[/] search",
    ] {
        assert!(screen.contains(action), "missing {action}:\n{screen}");
    }
    assert!(!screen.contains("[v] reviewr"));

    model.set_reviewr_available(true);
    let with_reviewr = render(&model, 80, 24);
    assert!(with_reviewr.contains("[v] reviewr"));
}

#[test]
fn sixty_columns_use_an_actionable_vertical_workstation_list() {
    let screen = render(&three_agent_model(), 60, 18);

    assert_every_agent_is_visible(&screen);
    assert!(screen.contains("> Beta"), "missing selection:\n{screen}");
    assert!(
        screen.contains("BUILDING"),
        "missing working state:\n{screen}"
    );
    assert!(screen.contains("HELP!"), "missing blocked state:\n{screen}");
    assert!(screen.contains("BROKEN"), "missing exited state:\n{screen}");
    for action in [
        "[enter] visit",
        "[r] reply",
        "[o] refresh",
        "[space] seen",
        "[/] search",
    ] {
        assert!(screen.contains(action), "missing {action}:\n{screen}");
    }
}

#[test]
fn one_cell_cafe_is_safe() {
    let screen = render(&three_agent_model(), 1, 1);

    assert_eq!(screen, "C");
}

#[test]
fn zero_and_tiny_cafes_are_panic_free() {
    let model = three_agent_model();

    for (width, height) in [(0, 0), (0, 1), (1, 0), (2, 2), (3, 2), (3, 3)] {
        let _ = render(&model, width, height);
    }
}

#[test]
fn a_dense_agent_map_never_renders_cells_below_the_small_grid() {
    let mut model = three_agent_model();
    let source = model.domain().agents.values().next().unwrap().clone();
    for index in 4..=60 {
        let mut agent = source.clone();
        agent.key = AgentKey::new(format!("agent-z-{index:02}"));
        agent.pane_id = PaneId::new(format!("w1:p{index}"));
        agent.name = format!("Agent {index:02}");
        model.domain_mut().agents.insert(agent.key.clone(), agent);
    }

    let screen = render(&model, 80, 24);

    assert!(screen.contains("[w1]"));
}

#[test]
fn dense_grid_pages_to_keep_a_late_selection_visible() {
    let mut model = three_agent_model();
    let source = model.domain().agents.values().next().unwrap().clone();
    for index in 4..=60 {
        let mut agent = source.clone();
        agent.key = AgentKey::new(format!("agent-z-{index:02}"));
        agent.pane_id = PaneId::new(format!("w1:p{index}"));
        agent.name = format!("Agent {index:02}");
        model.domain_mut().agents.insert(agent.key.clone(), agent);
    }
    model.domain_mut().selected_agent = Some(AgentKey::new("agent-z-60"));

    let screen = render(&model, 80, 24);

    assert!(
        screen.contains("[w1]"),
        "active bay strip hidden:\n{screen}"
    );
    assert!(
        screen.contains("[j/k] navigate"),
        "navigation hidden:\n{screen}"
    );
}

#[test]
fn compact_dense_list_pages_to_keep_a_late_selection_visible() {
    let mut model = three_agent_model();
    let source = model.domain().agents.values().next().unwrap().clone();
    for index in 4..=60 {
        let mut agent = source.clone();
        agent.key = AgentKey::new(format!("agent-z-{index:02}"));
        agent.pane_id = PaneId::new(format!("w1:p{index}"));
        agent.name = format!("Agent {index:02}");
        model.domain_mut().agents.insert(agent.key.clone(), agent);
    }
    model.domain_mut().selected_agent = Some(AgentKey::new("agent-z-60"));

    let screen = render(&model, 60, 18);

    assert!(
        screen.contains("> Agent 60"),
        "late selection hidden:\n{screen}"
    );
    assert!(screen.contains("[>] BUILDING"), "state hidden:\n{screen}");
}

#[test]
fn empty_cafe_keeps_helpful_navigation_without_invalid_agent_actions() {
    let screen = render(&Model::new(View::Cafe), 120, 30);

    assert!(screen.contains("All workstations are free"));
    assert!(screen.contains("Start an agent"));
    assert!(screen.contains("[1] desk"));
    assert!(!screen.contains("[enter] visit"));
    assert!(!screen.contains("[r] reply"));
    assert!(!screen.contains("[o] refresh"));
    assert!(!screen.contains("[space] seen"));
    assert!(!screen.contains("[/] search"));
    assert!(!screen.contains("[v] reviewr"));
}

#[test]
fn footer_advertises_only_available_cafe_actions() {
    let mut model = three_agent_model();
    let screen = render(&model, 160, 50);

    for action in [
        "[1] desk",
        "[2] cafe",
        "[j/k] navigate",
        "[enter] visit",
        "[r] reply",
        "[o] refresh",
        "[space] seen",
        "[/] search",
    ] {
        assert!(screen.contains(action), "missing {action}:\n{screen}");
    }
    assert!(!screen.contains("[v] reviewr"));

    model.set_reviewr_available(true);
    let with_reviewr = render(&model, 160, 50);
    assert!(with_reviewr.contains("[v] reviewr"));
}

#[test]
fn connection_overlays_preserve_the_last_visible_agent_poses() {
    let cases = [
        (ConnectionState::Offline, "DISCONNECTED"),
        (
            ConnectionState::Reconnecting { attempt: 3 },
            "RECONNECTING #3",
        ),
        (
            ConnectionState::Incompatible {
                expected: 1,
                actual: 9,
            },
            "INCOMPATIBLE PROTOCOL 9 - NEED 1",
        ),
    ];

    for (connection, label) in cases {
        let mut model = three_agent_model();
        model.set_connection(connection);
        let screen = render(&model, 120, 30);

        assert!(screen.contains(label), "missing {label}:\n{screen}");
        assert!(
            screen.contains("LAST POSES PRESERVED"),
            "missing preservation notice:\n{screen}"
        );
        assert_every_agent_is_visible(&screen);
        assert!(
            screen.contains("HELP!"),
            "missing preserved pose:\n{screen}"
        );
    }
}

#[test]
fn ascii_cafe_is_actionable_and_never_emits_block_glyphs() {
    let mut model = three_agent_model();
    model.set_preferences(DisplayPreferences {
        character_set: CharacterSet::Ascii,
        ..DisplayPreferences::default()
    });

    let screen = render(&model, 120, 30);

    assert!(screen.is_ascii(), "non-ASCII cafe output:\n{screen}");
    assert_every_agent_is_visible(&screen);
    assert!(screen.contains("[!] HELP!"));
    assert!(screen.contains("[x] BROKEN"));
    assert!(screen.contains("w1"));
}

#[test]
fn ansi_sixteen_cafe_uses_only_named_palette_cells() {
    let mut model = three_agent_model();
    model.set_preferences(DisplayPreferences {
        color_mode: ColorMode::Ansi16,
        ..DisplayPreferences::default()
    });

    let colours = render_colours(&model, 120, 30);

    assert!(colours.iter().all(|(foreground, background)| !matches!(
        foreground,
        Color::Indexed(_) | Color::Rgb(_, _, _)
    ) && !matches!(
        background,
        Color::Indexed(_) | Color::Rgb(_, _, _)
    )));
    assert!(
        colours
            .iter()
            .any(|(foreground, _)| *foreground == Color::LightGreen)
    );
}

#[test]
fn reduced_and_no_motion_cafes_are_stable_across_clock_changes() {
    for motion in [Motion::Reduced, Motion::None] {
        let mut model = three_agent_model();
        model.set_preferences(DisplayPreferences {
            motion,
            ..DisplayPreferences::default()
        });
        let first = render(&model, 120, 30);
        model.set_now(Timestamp::from_millis(9_999));
        let later = render(&model, 120, 30);

        assert_eq!(first, later, "motion {motion:?} changed with the clock");
        assert!(first.contains("BUILDING"));
        assert!(first.contains("HELP!"));
        assert!(first.contains("BROKEN"));
    }
}

#[test]
fn done_confetti_has_exactly_eight_frames_then_leaves_a_stable_update_badge() {
    let mut model = three_agent_model();
    let selected = model.selected_agent_key().unwrap().clone();
    let agent = model.domain_mut().agents.get_mut(&selected).unwrap();
    agent.presence = Presence::Done;
    agent.attention = Attention::unseen(
        AttentionReason::WorkCompleted,
        Timestamp::from_millis(2_000),
    );

    for frame in 1..=8 {
        model.set_now(Timestamp::from_millis(2_000 + i64::from(frame - 1) * 125));
        let screen = render(&model, 120, 30);
        assert!(screen.contains("UPDATE"));
    }

    model.set_now(Timestamp::from_millis(3_000));
    let stable = render(&model, 120, 30);
    assert!(stable.contains("UPDATE"), "{stable}");
}
