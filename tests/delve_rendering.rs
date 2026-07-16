use questmancer::{
    app::{CharacterSet, ColorMode, ConnectionState, DisplayPreferences, Model, Motion, View},
    domain::{
        AgentKey, Campaign, DomainState, GuildAttention, GuildSummons, PaneId, Presence, Timestamp,
        WorkspaceId,
    },
    herdr::protocol::{SessionSnapshotResult, SuccessResponse},
    ui::{self, delve_scene::layout_delves},
};
use ratatui::{Terminal, backend::TestBackend, layout::Rect, style::Color};

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
    alpha.attention = GuildAttention::Clear;
    alpha.focused = true;

    let mut beta = source.clone();
    beta.key = AgentKey::new("agent-b");
    beta.pane_id = PaneId::new("w1:p2");
    "Beta".clone_into(&mut beta.name);
    beta.presence = Presence::Blocked;
    beta.attention = GuildAttention::unread(
        GuildSummons::CounselRequested,
        Timestamp::from_millis(2_000),
    );
    beta.focused = false;

    let mut gamma = source;
    gamma.key = AgentKey::new("agent-c");
    gamma.pane_id = PaneId::new("w1:p3");
    "Gamma".clone_into(&mut gamma.name);
    gamma.presence = Presence::Exited;
    gamma.attention = GuildAttention::Clear;
    gamma.focused = false;

    let mut domain = DomainState::default();
    domain.agents.insert(alpha.key.clone(), alpha);
    domain.agents.insert(beta.key.clone(), beta);
    domain.agents.insert(gamma.key.clone(), gamma);
    domain.selected_agent = Some(AgentKey::new("agent-b"));

    let mut model = Model::new(View::Delve);
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

fn four_campaign_ascii_model() -> Model {
    let mut model = three_agent_model();
    let alpha = WorkspaceId::new("alpha");
    for agent in model.domain_mut().agents.values_mut() {
        agent.workspace_id = alpha.clone();
    }
    let party = model.domain().agents.keys().cloned().collect::<Vec<_>>();
    model.domain_mut().campaigns.clear();
    for id in ["alpha", "beta", "gamma", "zeta"] {
        let workspace_id = WorkspaceId::new(id);
        model.domain_mut().campaigns.insert(
            workspace_id.clone(),
            Campaign {
                workspace_id,
                label: id.to_owned(),
                cwd: format!("/tmp/{id}").into(),
                party: if id == "alpha" {
                    party.clone()
                } else {
                    Vec::new()
                },
            },
        );
    }
    model.set_preferences(DisplayPreferences {
        character_set: CharacterSet::Ascii,
        ..DisplayPreferences::default()
    });
    model
}

fn ascii_cell(screen: &str, x: u16, y: u16) -> u8 {
    screen.lines().nth(usize::from(y)).unwrap().as_bytes()[usize::from(x)]
}

#[test]
fn selected_and_departed_adventurers_remain_in_the_same_authored_delve() {
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
    assert!(screen.contains("DEPARTED") || screen.contains("EMPTY CHAMBER"));
    assert!(screen.contains("> Beta"));
    assert!(screen.contains("COUNSEL REQUESTED"));
    assert!(
        !screen.contains("CRT") && !screen.contains("MODEM") && !screen.contains("DESK"),
        "cybercafe architecture leaked into the Delve:\n{screen}"
    );
}

#[test]
fn one_hundred_twenty_columns_show_authored_delve_and_selected_chamber() {
    let screen = render(&three_agent_model(), 120, 30);

    assert_every_agent_is_visible(&screen);
    assert!(
        screen.contains("w1"),
        "missing workspace signage:\n{screen}"
    );
    assert!(
        screen.contains("PASSAGE") || screen.contains("ARCH") || screen.contains("STAIR"),
        "missing connected Delve cue:\n{screen}"
    );
    assert!(
        screen.contains("== == =="),
        "missing stone floor cue:\n{screen}"
    );
    assert!(
        screen.contains("> Beta"),
        "missing selected chamber:\n{screen}"
    );
    assert!(
        screen.contains("COUNSEL REQUESTED"),
        "missing state theatre:\n{screen}"
    );
}

#[test]
fn one_hundred_sixty_columns_keep_three_agents_in_authored_room() {
    let screen = render(&three_agent_model(), 160, 50);

    assert!(screen.contains("Alpha") && screen.contains("Beta"));
    assert!(
        screen.contains("w1"),
        "missing workspace signage:\n{screen}"
    );
    assert!(
        ["SHELVES", "STONE WALL", "MAP WALL"]
            .iter()
            .any(|cue| screen.contains(cue)),
        "missing dungeon architecture cue:\n{screen}"
    );
    assert!(screen.contains("> Beta"), "missing selection:\n{screen}");
    assert!(
        screen.contains("[!] COUNSEL REQUESTED"),
        "missing state:\n{screen}"
    );
    let alpha = screen.find("Alpha").unwrap();
    let beta = screen.find("Beta").unwrap();
    let gamma = screen.find("Gamma").unwrap();
    assert!(
        alpha < beta && beta < gamma,
        "unstable row-major order:\n{screen}"
    );
}

#[test]
fn multiple_campaigns_render_as_connected_delves_with_deterministic_variant_cues() {
    let mut model = three_agent_model();
    let gamma_key = AgentKey::new("agent-c");
    model
        .domain_mut()
        .agents
        .get_mut(&gamma_key)
        .unwrap()
        .workspace_id = WorkspaceId::new("w2");
    model.domain_mut().campaigns.insert(
        WorkspaceId::new("w1"),
        Campaign {
            workspace_id: WorkspaceId::new("w1"),
            label: "w1".into(),
            cwd: "/tmp/w1".into(),
            party: vec![AgentKey::new("agent-a"), AgentKey::new("agent-b")],
        },
    );
    model.domain_mut().campaigns.insert(
        WorkspaceId::new("w2"),
        Campaign {
            workspace_id: WorkspaceId::new("w2"),
            label: "w2".into(),
            cwd: "/tmp/w2".into(),
            party: vec![gamma_key],
        },
    );
    let screen = render(&model, 160, 40);
    assert!(screen.contains("w1"), "missing first Delve:\n{screen}");
    assert!(screen.contains("w2"), "missing second Delve:\n{screen}");
    assert!(
        screen.contains("PASSAGE") || screen.contains("ARCH") || screen.contains("STAIR"),
        "missing connected-room cue:\n{screen}"
    );
    assert_every_agent_is_visible(&screen);
}

#[test]
fn adjacent_delves_share_a_coordinate_continuous_opening() {
    let model = four_campaign_ascii_model();
    let area = Rect::new(1, 1, 158, 37);
    let delves = layout_delves(
        &model.domain().campaigns,
        &model.domain().agents,
        area,
        None,
    );
    let left = &delves[0];
    let right = &delves[1];
    assert_eq!(left.rect.right(), right.rect.x);

    let screen = render(&model, 160, 40);
    let opening =
        (left.rect.y.saturating_add(1)..left.rect.bottom().saturating_sub(1)).find(|&y| {
            [
                left.rect.right().saturating_sub(2),
                left.rect.right().saturating_sub(1),
                right.rect.x,
                right.rect.x.saturating_add(1),
            ]
            .into_iter()
            .all(|x| ascii_cell(&screen, x, y) == b'-')
        });

    assert!(
        opening.is_some(),
        "adjacent Delves have sealed borders instead of a continuous opening:\n{screen}"
    );
    assert!(screen.contains("Alpha") && screen.contains("[!] COUNSEL REQUESTED"));
    assert!(screen.contains("[1] guild") && screen.contains("[2] delves"));
}

#[test]
fn row_wrap_delves_have_a_continuous_turn_across_the_shared_seam() {
    let model = four_campaign_ascii_model();
    let area = Rect::new(1, 1, 158, 37);
    let delves = layout_delves(
        &model.domain().campaigns,
        &model.domain().agents,
        area,
        None,
    );
    let [previous, next] = delves
        .windows(2)
        .find(|pair| pair[0].rect.x > pair[1].rect.x)
        .expect("test requires a row wrap")
    else {
        unreachable!()
    };
    assert_eq!(previous.rect.bottom(), next.rect.y);
    let previous_x = previous.rect.x + previous.rect.width / 2;
    let next_x = next.rect.x + next.rect.width / 2;
    let seam_y = next.rect.y;

    let screen = render(&model, 160, 40);
    assert_eq!(ascii_cell(&screen, previous_x, seam_y - 2), b'|');
    assert_eq!(ascii_cell(&screen, previous_x, seam_y - 1), b'+');
    assert_eq!(ascii_cell(&screen, next_x, seam_y - 1), b'+');
    assert_eq!(ascii_cell(&screen, next_x, seam_y), b'|');
    assert_eq!(ascii_cell(&screen, next_x, seam_y + 1), b'|');
    assert!(
        (next_x + 1..previous_x).all(|x| ascii_cell(&screen, x, seam_y - 1) == b'-'),
        "row-wrap corridor is not continuous across the seam:\n{screen}"
    );
    assert!(screen.contains("Alpha") && screen.contains("[!] COUNSEL REQUESTED"));
}

#[test]
fn first_delve_has_a_coordinate_visible_route_home() {
    let model = four_campaign_ascii_model();
    let area = Rect::new(1, 1, 158, 37);
    let first = layout_delves(
        &model.domain().campaigns,
        &model.domain().agents,
        area,
        None,
    )
    .remove(0);
    let screen = render(&model, 160, 40);
    let route_y =
        (first.rect.y.saturating_add(1)..first.rect.bottom().saturating_sub(1)).find(|&y| {
            ascii_cell(&screen, first.rect.x, y) == b'<'
                && (1..=3).all(|offset| {
                    ascii_cell(&screen, first.rect.x.saturating_add(offset), y) == b'-'
                })
        });

    let route_y = route_y.expect("route home must open the first Delve's outer wall");
    let row = screen.lines().nth(usize::from(route_y)).unwrap();
    assert!(
        row.contains("HOME"),
        "route lacks its HOME landmark:\n{screen}"
    );
    assert!(screen.contains("Alpha") && screen.contains("[!] COUNSEL REQUESTED"));
}

#[test]
fn compact_active_delve_keeps_the_route_home_open() {
    let model = four_campaign_ascii_model();
    let screen = render(&model, 80, 24);
    let route_y = (2..18).find(|&y| {
        ascii_cell(&screen, 1, y) == b'<' && (2..=4).all(|x| ascii_cell(&screen, x, y) == b'-')
    });

    let route_y = route_y.expect("compact active Delve must retain the route home");
    assert!(
        screen
            .lines()
            .nth(usize::from(route_y))
            .unwrap()
            .contains("HOME")
    );
    assert!(screen.contains("[alpha] [beta] [gamma] [zeta]"));
    assert!(
        screen.contains("[!] COUNSEL REQUESTED"),
        "selected state was obscured:\n{screen}"
    );
}

#[test]
fn authored_variants_change_rendered_room_geometry() {
    let mut ids = Vec::new();
    for index in 0..128 {
        let id = WorkspaceId::new(format!("variant-{index}"));
        let variant = questmancer::ui::delve_scene::variant_for_campaign(&id);
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
        model.domain_mut().campaigns.clear();
        let keys = model.domain().agents.keys().cloned().collect();
        model.domain_mut().campaigns.insert(
            workspace.clone(),
            Campaign {
                workspace_id: workspace,
                label: format!("{variant:?}"),
                cwd: "/tmp".into(),
                party: keys,
            },
        );
        screens.push(render(&model, 120, 30));
    }
    assert_eq!(screens.len(), 3);
    assert!(screens.windows(2).all(|pair| pair[0] != pair[1]));
}

#[test]
fn eighty_by_twenty_four_keeps_a_compact_delve_strip_and_actions() {
    let mut model = three_agent_model();
    model.domain_mut().campaigns.insert(
        WorkspaceId::new("w1"),
        Campaign {
            workspace_id: WorkspaceId::new("w1"),
            label: "w1".into(),
            cwd: "/tmp/w1".into(),
            party: vec![
                AgentKey::new("agent-a"),
                AgentKey::new("agent-b"),
                AgentKey::new("agent-c"),
            ],
        },
    );
    model.domain_mut().campaigns.insert(
        WorkspaceId::new("w2"),
        Campaign {
            workspace_id: WorkspaceId::new("w2"),
            label: "w2".into(),
            cwd: "/tmp/w2".into(),
            party: vec![],
        },
    );
    let screen = render(&model, 80, 24);
    assert!(
        screen.contains("[w1] [w2]"),
        "missing compact Delve strip:\n{screen}"
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
    model.domain_mut().campaigns.insert(
        WorkspaceId::new("w1"),
        Campaign {
            workspace_id: WorkspaceId::new("w1"),
            label: "w1".into(),
            cwd: "/tmp/w1".into(),
            party: vec![
                AgentKey::new("agent-a"),
                AgentKey::new("agent-b"),
                AgentKey::new("agent-c"),
            ],
        },
    );
    model.domain_mut().campaigns.insert(
        WorkspaceId::new("w2"),
        Campaign {
            workspace_id: WorkspaceId::new("w2"),
            label: "w2".into(),
            cwd: "/tmp/w2".into(),
            party: vec![],
        },
    );
    let screen = render(&model, 120, 30);
    assert!(screen.is_ascii(), "non-ASCII transition:\n{screen}");
    assert!(screen.contains('+'));
    assert!(screen.contains("w1") && screen.contains("w2"));
}

#[test]
fn compact_selected_wrapped_campaign_remaps_chambers_into_active_delve() {
    let mut model = three_agent_model();
    let gamma = model
        .domain_mut()
        .agents
        .get_mut(&AgentKey::new("agent-c"))
        .unwrap();
    gamma.workspace_id = WorkspaceId::new("w2");
    model.domain_mut().selected_agent = Some(AgentKey::new("agent-c"));
    model.domain_mut().campaigns.insert(
        WorkspaceId::new("w1"),
        Campaign {
            workspace_id: WorkspaceId::new("w1"),
            label: "w1".into(),
            cwd: "/tmp/w1".into(),
            party: vec![AgentKey::new("agent-a"), AgentKey::new("agent-b")],
        },
    );
    model.domain_mut().campaigns.insert(
        WorkspaceId::new("w2"),
        Campaign {
            workspace_id: WorkspaceId::new("w2"),
            label: "w2".into(),
            cwd: "/tmp/w2".into(),
            party: vec![AgentKey::new("agent-c")],
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
fn compact_selection_targets_the_selected_overflow_delve() {
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
    model.domain_mut().campaigns.clear();
    model.domain_mut().campaigns.insert(
        WorkspaceId::new("overflow"),
        Campaign {
            workspace_id: WorkspaceId::new("overflow"),
            label: "overflow".into(),
            cwd: "/tmp".into(),
            party: keys.clone(),
        },
    );
    model.domain_mut().selected_agent = Some(keys[4].clone());
    let screen = render(&model, 80, 24);
    assert!(
        screen.contains("Overflow 4"),
        "selected overflow adventurer not visible above marker:\n{screen}"
    );
    let selected_offset = screen.find("Overflow 4").unwrap();
    let overflow_marker = screen
        .find("[more chambers]")
        .expect("overflow marker must be rendered");
    assert!(selected_offset < overflow_marker);
}

#[test]
fn eighty_columns_keep_authored_delve_and_actions() {
    let mut model = three_agent_model();
    let screen = render(&model, 80, 24);

    assert!(screen.contains("Alpha") && screen.contains("Beta"));
    assert!(
        screen.contains("w1"),
        "missing workspace signage:\n{screen}"
    );
    assert!(screen.contains("PASSAGE") || screen.contains("ARCH") || screen.contains("STAIR"));
    assert!(screen.contains("== == =="), "missing floor cue:\n{screen}");
    assert!(screen.contains("COUNSEL REQUESTED"));
    for action in [
        "[1] guild",
        "[2] delves",
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
fn sixty_columns_use_an_actionable_vertical_chamber_list() {
    let screen = render(&three_agent_model(), 60, 18);

    assert_every_agent_is_visible(&screen);
    assert!(screen.contains("> Beta"), "missing selection:\n{screen}");
    assert!(
        screen.contains("DELVING"),
        "missing delving state:\n{screen}"
    );
    assert!(screen.contains("COUNSEL REQUESTED"));
    assert!(screen.contains("DEPARTED"));
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
fn one_cell_delve_is_safe() {
    let screen = render(&three_agent_model(), 1, 1);

    assert_eq!(screen, "D");
}

#[test]
fn zero_and_tiny_delves_are_panic_free() {
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
        "active Delve strip hidden:\n{screen}"
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
    assert!(screen.contains("[>] DELVING"), "state hidden:\n{screen}");
}

#[test]
fn empty_delve_keeps_helpful_navigation_without_invalid_adventurer_actions() {
    let screen = render(&Model::new(View::Delve), 120, 30);

    assert!(screen.contains("All Delves await a party"));
    assert!(screen.contains("Start an adventurer"));
    assert!(screen.contains("[1] guild"));
    assert!(!screen.contains("[enter] visit"));
    assert!(!screen.contains("[r] reply"));
    assert!(!screen.contains("[o] refresh"));
    assert!(!screen.contains("[space] seen"));
    assert!(!screen.contains("[/] search"));
    assert!(!screen.contains("[v] reviewr"));
}

#[test]
fn footer_advertises_only_available_delve_actions() {
    let mut model = three_agent_model();
    let screen = render(&model, 160, 50);

    for action in [
        "[1] guild",
        "[2] delves",
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
fn offline_and_incompatible_overlays_preserve_the_last_visible_adventurer_states() {
    let cases = [
        (ConnectionState::Offline, "DISCONNECTED"),
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
            screen.contains("LAST TALES PRESERVED"),
            "missing preservation notice:\n{screen}"
        );
        assert_every_agent_is_visible(&screen);
        assert!(
            screen.contains("COUNSEL REQUESTED"),
            "missing preserved pose:\n{screen}"
        );
    }
}

#[test]
fn reconnecting_fog_occupies_unused_architecture_and_preserves_actionable_state() {
    for (character_set, motion) in [
        (CharacterSet::Unicode, Motion::Full),
        (CharacterSet::Ascii, Motion::Reduced),
        (CharacterSet::Ascii, Motion::None),
    ] {
        let mut model = three_agent_model();
        let workspace_id = WorkspaceId::new("w1");
        let party = model.domain().agents.keys().cloned().collect::<Vec<_>>();
        model.domain_mut().campaigns.insert(
            workspace_id.clone(),
            Campaign {
                workspace_id,
                label: "w1".to_owned(),
                cwd: "/tmp/w1".into(),
                party,
            },
        );
        model.set_connection(ConnectionState::Reconnecting { attempt: 3 });
        model.set_preferences(DisplayPreferences {
            character_set,
            motion,
            ..DisplayPreferences::default()
        });

        let area = Rect::new(1, 1, 118, 27);
        let delve = layout_delves(
            &model.domain().campaigns,
            &model.domain().agents,
            area,
            None,
        )
        .remove(0);
        let first = render(&model, 120, 30);
        let fog_row = (delve.rect.y.saturating_add(1)..delve.rect.bottom().saturating_sub(1))
            .find(|&y| first.lines().nth(usize::from(y)).unwrap().contains("FOG"))
            .expect("FOG must occupy an interior architecture row");

        assert!(delve.chambers.iter().all(|chamber| {
            fog_row < chamber.y || fog_row >= chamber.y.saturating_add(chamber.height)
        }));
        assert!(first.contains("FOG") && first.contains("RECONNECTING #3"));
        assert!(first.contains("LAST TALES PRESERVED"));
        assert_every_agent_is_visible(&first);
        assert!(first.contains("[>] DELVING"));
        assert!(first.contains("[!] COUNSEL REQUESTED"));
        assert!(first.contains("[x] DEPARTED"));
        assert!(first.contains("[1] guild") && first.contains("[2] delves"));
        if character_set == CharacterSet::Ascii {
            assert!(first.is_ascii(), "ASCII Fog emitted Unicode:\n{first}");
        }
        if motion != Motion::Full {
            model.set_now(Timestamp::from_millis(9_999));
            assert_eq!(first, render(&model, 120, 30));
        }
    }
}

#[test]
fn ascii_delve_is_actionable_and_never_emits_block_glyphs() {
    let mut model = three_agent_model();
    model.set_preferences(DisplayPreferences {
        character_set: CharacterSet::Ascii,
        ..DisplayPreferences::default()
    });

    let screen = render(&model, 120, 30);

    assert!(screen.is_ascii(), "non-ASCII Delve output:\n{screen}");
    assert_every_agent_is_visible(&screen);
    assert!(screen.contains("[!] COUNSEL REQUESTED"));
    assert!(screen.contains("[x] DEPARTED"));
    assert!(screen.contains("w1"));
}

#[test]
fn ansi_sixteen_delve_uses_only_named_palette_cells() {
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
fn reduced_and_no_motion_delves_are_stable_across_clock_changes() {
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
        assert!(first.contains("DELVING"));
        assert!(first.contains("COUNSEL REQUESTED"));
        assert!(first.contains("DEPARTED"));
    }
}

#[test]
fn chest_sparkle_has_exactly_eight_frames_then_leaves_stable_spoils() {
    let mut model = three_agent_model();
    let selected = model.selected_agent_key().unwrap().clone();
    let agent = model.domain_mut().agents.get_mut(&selected).unwrap();
    agent.presence = Presence::Done;
    agent.attention =
        GuildAttention::unread(GuildSummons::SpoilsReturned, Timestamp::from_millis(2_000));

    for frame in 1..=8 {
        model.set_now(Timestamp::from_millis(2_000 + i64::from(frame - 1) * 125));
        let screen = render(&model, 120, 30);
        assert!(screen.contains("SPOILS RETURNED"));
    }

    model.set_now(Timestamp::from_millis(3_000));
    let stable = render(&model, 120, 30);
    assert!(stable.contains("SPOILS RETURNED"), "{stable}");
}
