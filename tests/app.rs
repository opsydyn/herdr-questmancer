use questmancer::{
    app::{ConnectionState, Modal, Model, Region, View},
    domain::{AgentKey, DomainState, PaneId, Timestamp},
    herdr::protocol::{SessionSnapshotResult, SuccessResponse},
};

fn domain_state() -> DomainState {
    let response: SuccessResponse<SessionSnapshotResult> =
        serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
    DomainState::from_snapshot(&response.result.snapshot, Timestamp::from_millis(1))
}

#[test]
fn starts_in_requested_view() {
    let model = Model::new(View::Delve);
    assert_eq!(model.view(), View::Delve);
    assert_eq!(model.connection(), &ConnectionState::Offline);
    assert!(model.domain().agents.is_empty());
    assert_eq!(model.modal(), &Modal::None);
}

#[test]
fn new_model_has_no_managed_pane() {
    let model = Model::new(View::Guild);
    assert_eq!(model.managed_pane_id(), None);
}

#[test]
fn managed_pane_round_trips_through_model() {
    let mut model = Model::new(View::Guild);
    let pane_id = PaneId::new("w2:p3");

    model.set_managed_pane_id(Some(pane_id.clone()));

    assert_eq!(model.managed_pane_id(), Some(&pane_id));
}

#[test]
fn switches_views() {
    let mut model = Model::new(View::Guild);
    model.switch_to(View::Delve);
    assert_eq!(model.view(), View::Delve);
}

#[test]
fn domain_replacement_keeps_a_valid_selection() {
    let mut model = Model::new(View::Guild);
    model.replace_domain(domain_state());

    assert!(model.selected_agent().is_some());
    assert_eq!(
        model.selected_agent(),
        model.domain().agents.values().next()
    );
}

#[test]
fn selection_movement_clamps_at_the_boundaries() {
    let mut domain = domain_state();
    let first = domain.agents.values().next().unwrap().clone();
    let mut second = first.clone();
    second.key = AgentKey::new("agent-z");
    second.pane_id = "w1:p2".into();
    domain.agents.insert(second.key.clone(), second);
    let mut model = Model::new(View::Guild);
    model.replace_domain(domain);

    model.select_previous_agent();
    let first_key = model.selected_agent().unwrap().key.clone();
    model.select_next_agent();
    assert_ne!(model.selected_agent().unwrap().key, first_key);
    model.select_next_agent();
    assert_eq!(
        model.selected_agent().unwrap().key,
        AgentKey::new("agent-z")
    );
}

#[test]
fn region_and_counsel_modal_are_explicit_app_state() {
    let mut model = Model::new(View::Guild);
    model.set_region(Region::Summons);
    model.open_counsel();
    model.push_counsel_character('h');
    model.push_counsel_character('i');

    assert_eq!(model.region(), Region::Summons);
    assert_eq!(model.modal(), &Modal::Counsel { draft: "hi".into() });
    assert_eq!(model.take_counsel(), Some("hi".into()));
    assert_eq!(model.modal(), &Modal::None);
}

#[test]
fn counsel_editing_can_backspace_clear_and_cancel() {
    let mut model = Model::new(View::Guild);
    model.open_counsel();
    model.push_counsel_character('o');
    model.push_counsel_character('k');
    model.backspace_counsel();
    assert_eq!(model.modal(), &Modal::Counsel { draft: "o".into() });

    model.clear_modal_input();
    assert_eq!(
        model.modal(),
        &Modal::Counsel {
            draft: String::new()
        }
    );
    model.dismiss_modal();
    assert_eq!(model.modal(), &Modal::None);
}
