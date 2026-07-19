use std::{collections::BTreeMap, path::PathBuf};

use questmancer::{
    app::{ConnectionState, Model, View},
    domain::{
        AdventurerPersona, Agent, AgentKey, Campaign, Chronicle, DomainState, GuildAttention,
        PaneId, PersonaKey, Presence, TabId, Timestamp, WorkspaceId,
    },
    scene::{
        assets::palette::SELECTION_RUNE,
        pixel::{PixelSize, Rgb, RgbBuffer},
        presentation::{SceneOverlay, ScenePresentation},
        render_scene_for_world,
        snapshot::SceneSnapshot,
        stage::WorldScene,
    },
};

fn model(view: View) -> Model {
    let key = AgentKey::new("codex");
    let workspace_id = WorkspaceId::new("questmancer");
    let agent = Agent {
        key: key.clone(),
        pane_id: PaneId::new("w1:p1"),
        workspace_id: workspace_id.clone(),
        tab_id: TabId::new("w1:t1"),
        name: "codex".to_owned(),
        custom_status: None,
        presence: Presence::Idle,
        presence_since: Timestamp::from_millis(1_000),
        attention: GuildAttention::Clear,
        focused: false,
        pane_revision: 1,
        persona: AdventurerPersona::for_key(PersonaKey::new("scene-selected-codex")),
    };
    let campaign = Campaign {
        workspace_id: workspace_id.clone(),
        label: "Questmancer".to_owned(),
        cwd: PathBuf::from("/tmp/questmancer"),
        party: vec![key.clone()],
    };
    let mut agents = BTreeMap::new();
    agents.insert(key.clone(), agent);
    let mut campaigns = BTreeMap::new();
    campaigns.insert(workspace_id, campaign);

    let mut model = Model::new(view);
    model.set_connection(ConnectionState::Connected);
    model.set_now(Timestamp::from_millis(5_000));
    model.replace_domain(DomainState {
        campaigns,
        agents,
        selected_agent: Some(key),
        chronicle: Chronicle::default(),
    });
    model
}

#[test]
fn presentation_keeps_ui_state_outside_scene_truth() {
    let model = model(View::Guild);

    let snapshot = SceneSnapshot::from_model(&model);
    let presentation = ScenePresentation::from_model(&model);

    assert_eq!(presentation.world, WorldScene::GuildHall);
    assert_eq!(presentation.selected_agent, Some(AgentKey::new("codex")));
    assert_eq!(presentation.overlay, SceneOverlay::None);
    assert_eq!(snapshot, SceneSnapshot::from_model(&model));
}

#[test]
fn explicit_world_render_marks_only_the_selected_adventurer() {
    let model = model(View::Guild);
    let snapshot = SceneSnapshot::from_model(&model);
    let presentation = ScenePresentation::from_model(&model);
    let mut target = RgbBuffer::filled(160, 90, Rgb::BLACK);

    let frame = render_scene_for_world(
        &snapshot,
        &presentation,
        PixelSize::new(160, 90),
        &mut target,
    );

    assert_eq!(frame.world, WorldScene::GuildHall);
    assert_eq!(
        target
            .pixels()
            .iter()
            .filter(|pixel| **pixel == SELECTION_RUNE)
            .count(),
        4
    );
}

#[test]
fn model_view_explicitly_selects_the_rendered_world() {
    for (view, world) in [
        (View::Guild, WorldScene::GuildHall),
        (View::Delve, WorldScene::Delve),
    ] {
        let model = model(view);
        let snapshot = SceneSnapshot::from_model(&model);
        let presentation = ScenePresentation::from_model(&model);
        let mut target = RgbBuffer::filled(160, 90, Rgb::BLACK);

        let frame = render_scene_for_world(
            &snapshot,
            &presentation,
            PixelSize::new(160, 90),
            &mut target,
        );

        assert_eq!(frame.world, world);
    }
}
