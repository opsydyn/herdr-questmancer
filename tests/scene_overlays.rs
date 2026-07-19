use std::{collections::BTreeMap, path::PathBuf};

use questmancer::{
    app::{ConnectionState, Model, OutputPreview, View},
    domain::{
        AdventurerPersona, Agent, AgentKey, Campaign, Chronicle, DomainState, GuildAttention,
        PaneId, PersonaKey, Presence, TabId, Timestamp, WorkspaceId,
    },
    interaction::reduce_action,
    scene::presentation::ScenePresentation,
    ui::{input::Action, scene_overlays::render_scene_overlays},
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
            render_scene_overlays(frame, model, &ScenePresentation::from_model(model));
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
    let _ = reduce_action(&mut model, Action::ShowHelp);
    for (width, height) in [(0, 0), (1, 1), (40, 18), (80, 24)] {
        let _ = render(&model, width, height);
    }
}
