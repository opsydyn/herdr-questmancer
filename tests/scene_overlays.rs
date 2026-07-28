use std::{collections::BTreeMap, path::PathBuf};

use questmancer::{
    app::{ConnectionState, Model, OutputPreview, View},
    domain::{
        AdventurerPersona, Agent, AgentKey, Campaign, Chronicle, DomainState, GuildAttention,
        PaneId, PersonaKey, Presence, TabId, Timestamp, WorkspaceId,
    },
    interaction::reduce_action,
    scene::{
        SceneActorRegion, SceneFrame, pixel::PixelRect, presentation::ScenePresentation,
        stage::WorldScene,
    },
    ui::{
        input::Action,
        scene_overlays::{render_scene_identity_labels, render_scene_overlays},
    },
};
use ratatui::{Terminal, backend::TestBackend, widgets::Paragraph};

fn model() -> Model {
    let key = AgentKey::new("codex");
    let workspace_id = WorkspaceId::new("questmancer");
    let mut agents = BTreeMap::new();
    agents.insert(
        key.clone(),
        Agent {
            key: key.clone(),
            pane_id: PaneId::new("w1:p1"),
            workspace_id: workspace_id.clone(),
            tab_id: TabId::new("w1:t1"),
            name: "codex".to_owned(),
            custom_status: None,
            presence: Presence::Working,
            presence_since: Timestamp::from_millis(1_000),
            attention: GuildAttention::Clear,
            focused: false,
            pane_revision: 1,
            persona: AdventurerPersona::for_key(PersonaKey::new("overlay-codex")),
        },
    );
    let mut campaigns = BTreeMap::new();
    campaigns.insert(
        workspace_id.clone(),
        Campaign {
            workspace_id,
            label: "Questmancer".to_owned(),
            cwd: PathBuf::from("/tmp/questmancer"),
            party: vec![key.clone()],
        },
    );
    let mut model = Model::new(View::Guild);
    model.set_connection(ConnectionState::Connected);
    model.set_now(Timestamp::from_millis(1_000));
    model.replace_domain(DomainState {
        campaigns,
        agents,
        selected_agent: Some(key),
        chronicle: Chronicle::default(),
    });
    model
}

fn render(model: &Model, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| {
            frame.render_widget(Paragraph::new("WORLD REMAINS"), frame.area());
            render_scene_overlays(frame, model, &ScenePresentation::from_model(model), None);
        })
        .unwrap();
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

#[test]
fn counsel_uses_a_centered_parchment_without_replacing_the_world() {
    let mut model = model();
    let _ = reduce_action(&mut model, Action::Counsel);
    for character in "use jsonb".chars() {
        let _ = reduce_action(&mut model, Action::TypeCharacter(character));
    }

    let screen = render(&model, 120, 36);

    assert!(screen.contains("WORLD REMAINS"));
    assert!(screen.contains("ISSUE COUNSEL"));
    assert!(screen.contains("use jsonb"));
    assert!(screen.contains("Enter send"));
    assert!(screen.contains("Esc cancel"));
}

#[test]
fn search_and_scrying_are_contextual_overlays() {
    let mut model = model();
    let _ = reduce_action(&mut model, Action::Search);
    assert!(render(&model, 120, 36).contains("SEARCH THE GUILD"));

    model.dismiss_modal();
    model.set_output_preview(Some(OutputPreview {
        pane_id: PaneId::new("w1:p1"),
        revision: 1,
        text: "cargo test passed".to_owned(),
        loading: false,
        error: None,
    }));
    let _ = reduce_action(&mut model, Action::Refresh);
    let screen = render(&model, 120, 36);
    assert!(screen.contains("SCRYING"));
    assert!(screen.contains("cargo test passed"));
}

#[test]
fn librarian_ledger_is_the_single_responsive_help_surface() {
    let mut model = model();
    let _ = reduce_action(&mut model, Action::ToggleLedger);

    let wide = render(&model, 120, 36);
    assert!(wide.contains("LIBRARIAN'S LEDGER"));
    assert!(wide.contains("Welcome to the Guild"));
    assert!(wide.contains("Page 1 / 4"));
    assert!(wide.contains("Esc/? close"));

    let compact = render(&model, 64, 24);
    assert!(compact.contains("LIBRARIAN'S LEDGER"));
    assert!(compact.contains("Welcome to the Guild"));
}

#[test]
fn selected_adventurer_card_exposes_fantasy_and_system_identity() {
    let mut model = model();
    model.show_adventurer_card();
    model.set_now(Timestamp::from_millis(421_000));
    model
        .domain_mut()
        .agents
        .get_mut(&AgentKey::new("codex"))
        .unwrap()
        .custom_status = Some("Indexing the forgotten library".to_owned());
    let persona_name = model.selected_agent().unwrap().persona.name.clone();

    let screen = render(&model, 120, 36);

    assert!(screen.contains("ADVENTURER"));
    assert!(screen.contains(&persona_name));
    assert!(screen.contains("Agent: codex"));
    assert!(screen.contains("Campaign: Questmancer"));
    assert!(screen.contains("Working · 7m"));
    assert!(screen.contains("Indexing the forgotten library"));
}

#[test]
fn selected_adventurer_does_not_force_the_card_open() {
    let model = model();

    let screen = render(&model, 120, 36);

    assert!(!screen.contains("ADVENTURER"));
    assert!(model.selected_agent().is_some());
}

#[test]
fn every_visible_sprite_has_an_agent_state_nameplate() {
    let mut model = model();
    model.set_now(Timestamp::from_millis(421_000));
    let scene = SceneFrame {
        world: WorldScene::GuildHall,
        next_frame_in: None,
        actors: vec![SceneActorRegion {
            agent: AgentKey::new("codex"),
            bounds: PixelRect::new(10, 20, 8, 14),
        }],
        interactables: Vec::new(),
    };
    let backend = TestBackend::new(120, 36);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| render_scene_identity_labels(frame, &model, &scene))
        .unwrap();
    let buffer = terminal.backend().buffer();
    let screen = (0..36)
        .map(|y| {
            (0..120)
                .map(|x| buffer.cell((x, y)).unwrap().symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(screen.contains("codex · WORKING 7m"));
}

#[test]
fn a_nameplate_never_paints_through_another_adventurer() {
    let mut model = model();
    let mut other = model.selected_agent().cloned().unwrap();
    other.key = AgentKey::new("other");
    other.name = "other".to_owned();
    model.domain_mut().agents.insert(other.key.clone(), other);
    model.set_now(Timestamp::from_millis(421_000));
    let scene = SceneFrame {
        world: WorldScene::GuildHall,
        next_frame_in: None,
        actors: vec![
            SceneActorRegion {
                agent: AgentKey::new("codex"),
                bounds: PixelRect::new(20, 20, 16, 24),
            },
            SceneActorRegion {
                agent: AgentKey::new("other"),
                bounds: PixelRect::new(20, 18, 16, 24),
            },
        ],
        interactables: Vec::new(),
    };
    let backend = TestBackend::new(120, 36);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| render_scene_identity_labels(frame, &model, &scene))
        .unwrap();
    let buffer = terminal.backend().buffer();
    let row = |y| {
        (0..120)
            .map(|x| buffer.cell((x, y)).unwrap().symbol())
            .collect::<String>()
    };

    assert!(
        !row(9).contains("codex"),
        "the preferred label row belongs to the neighbouring adventurer"
    );
    assert!(
        row(22).contains("codex · WORKING 7m"),
        "the selected nameplate should use the first clear lane below the actor"
    );
}

#[test]
fn command_ribbon_expires_after_three_seconds() {
    let mut model = model();
    let _ = reduce_action(&mut model, Action::Next);
    assert!(model.command_ribbon_visible());
    assert!(render(&model, 120, 36).contains("[1] Guild"));

    model.set_now(Timestamp::from_millis(4_001));
    assert!(!model.command_ribbon_visible());
    assert!(!render(&model, 120, 36).contains("[1] Guild"));
}

#[test]
fn overlays_are_safe_at_zero_and_minimum_viewports() {
    let mut model = model();
    let _ = reduce_action(&mut model, Action::ToggleLedger);
    for (width, height) in [(0, 0), (1, 1), (40, 18), (80, 24)] {
        let _ = render(&model, width, height);
    }
}
