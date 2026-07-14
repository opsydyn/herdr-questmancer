use herdr_webmaster::{
    app::{ConnectionState, Modal, Model, Region, View},
    domain::{AgentKey, DomainState, Timestamp},
    herdr::protocol::{SessionSnapshotResult, SuccessResponse},
};

fn domain_state() -> DomainState {
    let response: SuccessResponse<SessionSnapshotResult> =
        serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
    DomainState::from_snapshot(&response.result.snapshot, Timestamp::from_millis(1))
}

#[test]
fn starts_in_requested_view() {
    let model = Model::new(View::Cafe);
    assert_eq!(model.view(), View::Cafe);
    assert_eq!(model.connection(), &ConnectionState::Offline);
    assert!(model.domain().agents.is_empty());
    assert_eq!(model.modal(), &Modal::None);
}

#[test]
fn switches_views() {
    let mut model = Model::new(View::Desk);
    model.switch_to(View::Cafe);
    assert_eq!(model.view(), View::Cafe);
}

#[test]
fn domain_replacement_keeps_a_valid_selection() {
    let mut model = Model::new(View::Desk);
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
    let mut model = Model::new(View::Desk);
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
fn region_and_reply_modal_are_explicit_app_state() {
    let mut model = Model::new(View::Desk);
    model.set_region(Region::Inbox);
    model.open_reply();
    model.push_reply_character('h');
    model.push_reply_character('i');

    assert_eq!(model.region(), Region::Inbox);
    assert_eq!(model.modal(), &Modal::Reply { draft: "hi".into() });
    assert_eq!(model.take_reply(), Some("hi".into()));
    assert_eq!(model.modal(), &Modal::None);
}
