use questmancer::{
    app::{CharacterSet, ConnectionState, DisplayPreferences, Modal, Model, View},
    domain::{Campaign, DomainState, Timestamp, WorkspaceId},
    herdr::protocol::{SessionSnapshotResult, SuccessResponse},
    interaction::reduce_action,
    persistence::PersistedStateV1,
    ui::{self, goblins::sighting_for_campaign, input::Action},
};
use ratatui::{Terminal, backend::TestBackend};
use std::path::PathBuf;

fn live_model() -> Model {
    let response: SuccessResponse<SessionSnapshotResult> =
        serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
    let mut model = Model::new(View::Guild);
    model.replace_domain(DomainState::from_snapshot(
        &response.result.snapshot,
        Timestamp::from_millis(1_000),
    ));
    model.set_connection(ConnectionState::Connected);
    model.set_now(Timestamp::from_millis(1_000));
    model
}

fn submit_search(model: &mut Model, query: &str) -> questmancer::interaction::ActionReduction {
    model.open_search();
    for character in query.chars() {
        model.push_modal_character(character);
    }
    reduce_action(model, Action::Submit)
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

#[test]
fn sightings_are_deterministic_and_non_semantic() {
    let workspace = WorkspaceId::new("w1");
    let first = sighting_for_campaign(&workspace);
    let second = sighting_for_campaign(&workspace);

    assert_eq!(first, second);
}

#[test]
fn rare_sightings_are_visible_without_changing_campaign_meaning() {
    let rare = (0..4_096)
        .map(|index| WorkspaceId::new(format!("goblin-candidate-{index}")))
        .find(|workspace| sighting_for_campaign(workspace).is_some())
        .expect("the one-in-256 sighting rate yields a fixture");
    let common = (0..4_096)
        .map(|index| WorkspaceId::new(format!("ordinary-candidate-{index}")))
        .find(|workspace| sighting_for_campaign(workspace).is_none())
        .expect("a common campaign fixture exists");
    let campaign = |workspace_id: WorkspaceId| Campaign {
        workspace_id,
        label: "The Same Campaign".to_owned(),
        cwd: PathBuf::from("/same/campaign"),
        party: Vec::new(),
    };

    let mut rare_model = Model::new(View::Guild);
    rare_model.set_preferences(DisplayPreferences {
        character_set: CharacterSet::Ascii,
        ..DisplayPreferences::default()
    });
    rare_model
        .domain_mut()
        .campaigns
        .insert(rare.clone(), campaign(rare));
    let mut common_model = Model::new(View::Guild);
    common_model.set_preferences(*rare_model.preferences());
    common_model
        .domain_mut()
        .campaigns
        .insert(common.clone(), campaign(common));

    let rare_screen = render(&rare_model, 80, 24);
    let common_screen = render(&common_model, 80, 24);
    assert!(rare_screen.contains("{g}"), "{rare_screen}");
    assert!(!common_screen.contains("{g}"), "{common_screen}");
    assert_eq!(rare_model.domain().campaigns.len(), 1);
    assert_eq!(common_model.domain().campaigns.len(), 1);
}

#[test]
fn exact_incantation_releases_goblins_without_selecting_an_agent() {
    let mut model = live_model();
    let selected = model.selected_agent_key().cloned();
    let durable = PersistedStateV1::capture(&model);

    let reduction = submit_search(&mut model, "  ReLeAsE tHe GoBlInS  ");

    assert!(model.goblins().is_visible(model.now()));
    assert_eq!(model.selected_agent_key(), selected.as_ref());
    assert_eq!(model.modal(), &Modal::None);
    assert_eq!(
        model.status_message(),
        Some("The goblins deny any involvement.")
    );
    assert!(reduction.commands.is_empty());
    assert!(reduction.persistence.is_empty());
    assert_eq!(PersistedStateV1::capture(&model), durable);
}

#[test]
fn false_incantations_retain_normal_search_behavior() {
    for query in [
        "release the goblin",
        "please release the goblins",
        "release  the goblins",
        "release the goblins!",
    ] {
        let mut model = live_model();
        let selected = model.selected_agent_key().cloned();

        let reduction = submit_search(&mut model, query);

        assert!(!model.goblins().is_visible(model.now()), "query {query:?}");
        assert_eq!(
            model.selected_agent_key(),
            selected.as_ref(),
            "query {query:?}"
        );
        assert!(
            matches!(model.modal(), Modal::Search { .. }),
            "query {query:?}"
        );
        assert!(reduction.commands.is_empty(), "query {query:?}");
        assert!(reduction.persistence.is_empty(), "query {query:?}");
    }
}

#[test]
fn outbreak_ends_at_the_exact_three_second_boundary() {
    let mut model = Model::new(View::Guild);
    model.set_now(Timestamp::from_millis(1_000));
    let released_at = model.now();
    model.goblins_mut().release(released_at);

    model.set_now(Timestamp::from_millis(3_999));
    assert!(model.goblins().is_visible(model.now()));

    model.set_now(Timestamp::from_millis(4_000));
    assert!(!model.goblins().is_visible(model.now()));
}

#[test]
fn outbreak_notice_is_transient_marginalia_not_factual_history() {
    let mut model = live_model();
    let chronicle_len = model.domain().chronicle.entries().len();
    let durable = PersistedStateV1::capture(&model);
    let released_at = model.now();
    model.goblins_mut().release(released_at);

    let active = render(&model, 120, 30);
    assert!(active.contains("CREATURES DETECTED"), "{active}");
    assert_eq!(model.domain().chronicle.entries().len(), chronicle_len);
    assert_eq!(PersistedStateV1::capture(&model), durable);

    model.set_now(Timestamp::from_millis(released_at.as_millis() + 3_000));
    let settled = render(&model, 120, 30);
    assert!(!settled.contains("CREATURES DETECTED"), "{settled}");
    assert_eq!(model.domain().chronicle.entries().len(), chronicle_len);
    assert_eq!(PersistedStateV1::capture(&model), durable);
}
