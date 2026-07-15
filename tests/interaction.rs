use std::ops::ControlFlow;

use herdr_webmaster::{
    app::{Modal, Model, Region, RuntimeSettings, View},
    command::DeskCommand,
    domain::{AgentKey, AgentPersona, DomainState, PaneId, PersonaKey, Timestamp, WorkspaceId},
    herdr::protocol::{SessionSnapshotResult, SuccessResponse},
    interaction::reduce_action,
    ui::input::Action,
    update::Command,
};

fn live_model_with_two_agents() -> Model {
    let response: SuccessResponse<SessionSnapshotResult> =
        serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
    let mut domain =
        DomainState::from_snapshot(&response.result.snapshot, Timestamp::from_millis(1_000));
    let mut second = domain.agents.values().next().unwrap().clone();
    second.key = AgentKey::new("agent-z");
    second.pane_id = PaneId::new("w1:p2");
    domain.agents.insert(second.key.clone(), second);
    let mut model = Model::new(View::Desk);
    model.replace_domain(domain);
    model
}

fn live_model_with_two_distinct_personas() -> Model {
    let mut model = live_model_with_two_agents();
    let second_key = model.domain().agents.keys().next_back().unwrap().clone();
    let persona_key = PersonaKey::new("persona-z");
    let second = model.domain_mut().agents.get_mut(&second_key).unwrap();
    second.persona = AgentPersona {
        appearance: AgentPersona::appearance_for_key(&persona_key),
        key: persona_key,
        handle: "second_persona".to_owned(),
    };
    model.replace_domain(model.domain().clone());
    model
}

fn searchable_model() -> Model {
    let mut model = live_model_with_two_agents();
    let first_key = model.domain().agents.keys().next().unwrap().clone();
    let second_key = model.domain().agents.keys().next_back().unwrap().clone();
    let first_site = model.domain().sites.values().next().unwrap().clone();
    {
        let domain = model.domain_mut();
        let first = domain.agents.get_mut(&first_key).unwrap();
        "Alpha".clone_into(&mut first.name);
        "one".clone_into(&mut first.persona.handle);
        first.custom_status = Some("waiting".to_owned());

        let second = domain.agents.get_mut(&second_key).unwrap();
        "Beta".clone_into(&mut second.name);
        "two".clone_into(&mut second.persona.handle);
        second.custom_status = Some("deploying".to_owned());
        second.workspace_id = WorkspaceId::new("w2");

        let mut second_site = first_site;
        second_site.workspace_id = WorkspaceId::new("w2");
        "Moon Base".clone_into(&mut second_site.label);
        second_site.agents = vec![second_key];
        domain.sites.insert(WorkspaceId::new("w2"), second_site);
    }
    model
}

fn cafe_twin(model: &Model) -> Model {
    let mut twin = model.clone();
    twin.switch_to(View::Cafe);
    twin
}

#[test]
fn quit_is_an_explicit_typed_loop_outcome() {
    let mut model = Model::new(View::Desk);

    let reduction = reduce_action(&mut model, Action::Quit);

    assert_eq!(reduction.control, ControlFlow::Break(()));
    assert!(reduction.commands.is_empty());
}

#[test]
fn view_switch_is_reduced_without_effect_commands() {
    let mut model = Model::new(View::Desk);

    let reduction = reduce_action(&mut model, Action::Switch(View::Cafe));

    assert_eq!(model.view(), View::Cafe);
    assert_eq!(reduction.control, ControlFlow::Continue(()));
    assert!(reduction.commands.is_empty());
    assert_eq!(reduction.persistence, vec![Command::PersistState]);

    let unchanged = reduce_action(&mut model, Action::Switch(View::Cafe));
    assert!(unchanged.persistence.is_empty());
}

#[test]
fn selection_changes_persist_but_noops_and_animation_redraws_do_not() {
    let mut model = live_model_with_two_distinct_personas();

    let changed = reduce_action(&mut model, Action::Last);
    assert_eq!(changed.persistence, vec![Command::PersistState]);

    let unchanged = reduce_action(&mut model, Action::Last);
    assert!(unchanged.persistence.is_empty());

    model.set_now(Timestamp::from_millis(86_400_000));
    let redraw = reduce_action(&mut model, Action::Redraw);
    assert!(redraw.persistence.is_empty());
}

#[test]
fn unchanged_idle_room_emits_no_output_load_or_persistence_effects() {
    let mut model = live_model_with_two_agents();
    for agent in model.domain_mut().agents.values_mut() {
        agent.presence = herdr_webmaster::domain::Presence::Idle;
    }

    let redraw = reduce_action(&mut model, Action::Redraw);

    assert!(redraw.commands.is_empty());
    assert!(redraw.persistence.is_empty());
}

#[test]
fn region_cycle_is_deterministic_and_wraps() {
    let mut model = Model::new(View::Desk);

    for expected in [
        Region::Inbox,
        Region::Guestbook,
        Region::Agent,
        Region::Sites,
    ] {
        let reduction = reduce_action(&mut model, Action::CycleRegion);
        assert_eq!(model.region(), expected);
        assert_eq!(reduction.control, ControlFlow::Continue(()));
        assert!(reduction.commands.is_empty());
    }
}

#[test]
fn first_and_last_select_boundaries_and_load_only_changed_selection() {
    let mut model = live_model_with_two_agents();

    let last = reduce_action(&mut model, Action::Last);
    assert_eq!(
        model.selected_agent().unwrap().pane_id,
        PaneId::new("w1:p2")
    );
    assert_eq!(
        last.commands,
        vec![DeskCommand::LoadOutput {
            pane_id: PaneId::new("w1:p2"),
            lines: 80,
        }]
    );

    let unchanged = reduce_action(&mut model, Action::Last);
    assert!(unchanged.commands.is_empty());

    let first = reduce_action(&mut model, Action::First);
    assert_eq!(
        model.selected_agent().unwrap().pane_id,
        PaneId::new("w1:p1")
    );
    assert_eq!(
        first.commands,
        vec![DeskCommand::LoadOutput {
            pane_id: PaneId::new("w1:p1"),
            lines: 80,
        }]
    );
}

#[test]
fn relative_selection_loads_one_preview_per_change() {
    let mut model = live_model_with_two_agents();

    let next = reduce_action(&mut model, Action::Next);
    assert_eq!(
        next.commands,
        vec![DeskCommand::LoadOutput {
            pane_id: PaneId::new("w1:p2"),
            lines: 80,
        }]
    );

    let previous = reduce_action(&mut model, Action::Previous);
    assert_eq!(
        previous.commands,
        vec![DeskCommand::LoadOutput {
            pane_id: PaneId::new("w1:p1"),
            lines: 80,
        }]
    );
}

#[test]
fn visit_focuses_selected_pane_and_empty_selection_is_contextual() {
    let mut selected = live_model_with_two_agents();
    let visit = reduce_action(&mut selected, Action::Visit);
    assert_eq!(
        visit.commands,
        vec![DeskCommand::FocusPane(PaneId::new("w1:p1"))]
    );

    let mut empty = Model::new(View::Desk);
    let no_visit = reduce_action(&mut empty, Action::Visit);
    assert!(no_visit.commands.is_empty());
    assert_eq!(empty.status_message(), Some("no agent selected to visit"));
}

#[test]
fn managed_pane_selection_never_emits_effect_commands() {
    let mut model = live_model_with_two_agents();
    model.set_managed_pane_id(Some(PaneId::new("w1:p1")));

    for action in [Action::Visit, Action::Refresh, Action::Reply] {
        let reduction = reduce_action(&mut model, action);
        assert!(reduction.commands.is_empty());
    }
    assert_eq!(model.status_message(), Some("no agent selected to reply"));
}

#[test]
fn navigation_does_not_load_output_for_a_managed_pane() {
    let mut model = live_model_with_two_agents();
    model.set_managed_pane_id(Some(PaneId::new("w1:p2")));

    let next = reduce_action(&mut model, Action::Next);
    assert!(next.commands.is_empty());

    let previous = reduce_action(&mut model, Action::Previous);
    assert_eq!(
        previous.commands,
        vec![DeskCommand::LoadOutput {
            pane_id: PaneId::new("w1:p1"),
            lines: 80,
        }]
    );
}

#[test]
fn navigation_with_only_the_managed_pane_has_no_output_effect() {
    let mut model = live_model_with_two_agents();
    model
        .domain_mut()
        .agents
        .retain(|_, agent| agent.pane_id == PaneId::new("w1:p1"));
    let only_agent = model.domain().agents.keys().next().cloned();
    model.domain_mut().selected_agent = only_agent;
    model.set_managed_pane_id(Some(PaneId::new("w1:p1")));

    for action in [Action::First, Action::Last, Action::Next, Action::Previous] {
        let reduction = reduce_action(&mut model, action);
        assert!(reduction.commands.is_empty());
    }
}

#[test]
fn refresh_loads_only_the_selected_output() {
    let mut selected = live_model_with_two_agents();
    let refresh = reduce_action(&mut selected, Action::Refresh);
    assert_eq!(
        refresh.commands,
        vec![DeskCommand::LoadOutput {
            pane_id: PaneId::new("w1:p1"),
            lines: 80,
        }]
    );

    let mut empty = Model::new(View::Desk);
    let no_refresh = reduce_action(&mut empty, Action::Refresh);
    assert!(no_refresh.commands.is_empty());
    assert_eq!(empty.status_message(), Some("no agent selected to refresh"));
}

#[test]
fn configured_runtime_settings_drive_output_and_reviewr_commands() {
    let mut model = live_model_with_two_agents();
    model.set_settings(RuntimeSettings {
        output_preview_lines: 123,
        reviewr_action: "acme.diff.inspect".to_owned(),
        show_elapsed_time: true,
    });

    let refresh = reduce_action(&mut model, Action::Refresh);
    assert_eq!(
        refresh.commands,
        vec![DeskCommand::LoadOutput {
            pane_id: PaneId::new("w1:p1"),
            lines: 123,
        }]
    );

    model.set_reviewr_available(true);
    let reviewr = reduce_action(&mut model, Action::Reviewr);
    assert_eq!(
        reviewr.commands,
        vec![DeskCommand::OpenReviewr {
            pane_id: PaneId::new("w1:p1"),
            qualified_id: "acme.diff.inspect".to_owned(),
        }]
    );
}

#[test]
fn reviewr_opens_only_when_available_for_a_selection() {
    let mut available = live_model_with_two_agents();
    available.set_reviewr_available(true);
    let open = reduce_action(&mut available, Action::Reviewr);
    assert_eq!(
        open.commands,
        vec![DeskCommand::OpenReviewr {
            pane_id: PaneId::new("w1:p1"),
            qualified_id: "persiyanov.reviewr.open".to_owned(),
        }]
    );

    let mut unavailable = live_model_with_two_agents();
    let no_open = reduce_action(&mut unavailable, Action::Reviewr);
    assert!(no_open.commands.is_empty());
    assert_eq!(unavailable.status_message(), Some("reviewr is unavailable"));

    let mut empty = Model::new(View::Desk);
    empty.set_reviewr_available(true);
    let no_selection = reduce_action(&mut empty, Action::Reviewr);
    assert!(no_selection.commands.is_empty());
    assert_eq!(
        empty.status_message(),
        Some("no agent selected for reviewr")
    );
}

#[test]
fn reply_submit_sends_the_exact_draft_to_the_selected_pane() {
    let mut model = live_model_with_two_agents();

    let opened = reduce_action(&mut model, Action::Reply);
    assert!(opened.commands.is_empty());
    assert_eq!(
        model.modal(),
        &Modal::Reply {
            draft: String::new()
        }
    );

    for character in "  use jsonb  ".chars() {
        let _ = reduce_action(&mut model, Action::TypeCharacter(character));
    }
    let _ = reduce_action(&mut model, Action::Backspace);
    let sent = reduce_action(&mut model, Action::Submit);

    assert_eq!(
        sent.commands,
        vec![DeskCommand::SendReply {
            pane_id: PaneId::new("w1:p1"),
            text: "  use jsonb ".to_owned(),
        }]
    );
    assert_eq!(model.modal(), &Modal::None);
}

#[test]
fn empty_reply_stays_open_while_clear_and_cancel_are_local() {
    let mut model = live_model_with_two_agents();
    let _ = reduce_action(&mut model, Action::Reply);
    let _ = reduce_action(&mut model, Action::TypeCharacter(' '));

    let empty = reduce_action(&mut model, Action::Submit);
    assert!(empty.commands.is_empty());
    assert_eq!(model.status_message(), Some("reply cannot be empty"));
    assert!(matches!(model.modal(), Modal::Reply { .. }));

    let _ = reduce_action(&mut model, Action::TypeCharacter('x'));
    let cleared = reduce_action(&mut model, Action::ClearInput);
    assert!(cleared.commands.is_empty());
    assert_eq!(
        model.modal(),
        &Modal::Reply {
            draft: String::new()
        }
    );

    let cancelled = reduce_action(&mut model, Action::Dismiss);
    assert!(cancelled.commands.is_empty());
    assert_eq!(model.modal(), &Modal::None);

    let mut empty_selection = Model::new(View::Desk);
    let no_reply = reduce_action(&mut empty_selection, Action::Reply);
    assert!(no_reply.commands.is_empty());
    assert_eq!(
        empty_selection.status_message(),
        Some("no agent selected to reply")
    );
}

#[test]
fn mark_seen_uses_the_domain_reducer_for_the_selected_agent() {
    let mut model = live_model_with_two_agents();
    assert!(model.selected_agent().unwrap().attention.is_unseen());

    let marked = reduce_action(&mut model, Action::MarkSeen);

    assert!(marked.commands.is_empty());
    assert_eq!(marked.persistence, vec![Command::PersistState]);
    assert!(!model.selected_agent().unwrap().attention.is_unseen());

    let mut empty = Model::new(View::Desk);
    let no_mark = reduce_action(&mut empty, Action::MarkSeen);
    assert!(no_mark.commands.is_empty());
    assert!(no_mark.persistence.is_empty());
    assert_eq!(
        empty.status_message(),
        Some("no agent selected to mark seen")
    );
}

#[test]
fn mark_seen_keeps_the_newly_selected_distinct_persona_selected() {
    let mut model = live_model_with_two_distinct_personas();
    model.select_last_agent();
    let selected = model.selected_agent_key().unwrap().clone();

    let marked = reduce_action(&mut model, Action::MarkSeen);

    assert!(marked.commands.is_empty());
    assert_eq!(model.selected_agent_key(), Some(&selected));
    assert!(!model.domain().agents[&selected].attention.is_unseen());
}

#[test]
fn search_modal_supports_edit_clear_and_cancel() {
    let mut model = live_model_with_two_agents();

    let opened = reduce_action(&mut model, Action::Search);
    assert!(opened.commands.is_empty());
    assert_eq!(
        model.modal(),
        &Modal::Search {
            query: String::new()
        }
    );

    let _ = reduce_action(&mut model, Action::TypeCharacter('C'));
    let _ = reduce_action(&mut model, Action::TypeCharacter('o'));
    let _ = reduce_action(&mut model, Action::Backspace);
    assert_eq!(
        model.modal(),
        &Modal::Search {
            query: "C".to_owned()
        }
    );

    let _ = reduce_action(&mut model, Action::ClearInput);
    assert_eq!(
        model.modal(),
        &Modal::Search {
            query: String::new()
        }
    );

    let _ = reduce_action(&mut model, Action::Dismiss);
    assert_eq!(model.modal(), &Modal::None);
}

#[test]
fn search_is_case_insensitive_across_agent_fields() {
    for query in ["bEtA", "TWO", "moon", "DEPLOY"] {
        let mut model = searchable_model();
        let _ = reduce_action(&mut model, Action::Search);
        for character in query.chars() {
            let _ = reduce_action(&mut model, Action::TypeCharacter(character));
        }

        let submitted = reduce_action(&mut model, Action::Submit);

        assert_eq!(
            model.selected_agent().unwrap().pane_id,
            PaneId::new("w1:p2")
        );
        assert_eq!(model.modal(), &Modal::None);
        assert_eq!(
            submitted.commands,
            vec![DeskCommand::LoadOutput {
                pane_id: PaneId::new("w1:p2"),
                lines: 80,
            }],
            "query {query}"
        );
    }
}

#[test]
fn search_no_match_stays_editable_with_visible_status() {
    let mut model = searchable_model();
    let _ = reduce_action(&mut model, Action::Search);
    for character in "missing".chars() {
        let _ = reduce_action(&mut model, Action::TypeCharacter(character));
    }

    let submitted = reduce_action(&mut model, Action::Submit);

    assert!(submitted.commands.is_empty());
    assert_eq!(
        model.modal(),
        &Modal::Search {
            query: "missing".to_owned()
        }
    );
    assert_eq!(model.status_message(), Some("no agents match \"missing\""));
}

#[test]
fn empty_search_does_not_select_the_first_agent() {
    let mut model = searchable_model();
    let _ = reduce_action(&mut model, Action::Last);
    let selected_before = model.selected_agent_key().cloned();
    let _ = reduce_action(&mut model, Action::Search);

    let submitted = reduce_action(&mut model, Action::Submit);

    assert!(submitted.commands.is_empty());
    assert_eq!(model.selected_agent_key(), selected_before.as_ref());
    assert_eq!(
        model.modal(),
        &Modal::Search {
            query: String::new()
        }
    );
    assert_eq!(model.status_message(), Some("enter a search query"));
}

#[test]
fn cafe_selection_reuses_the_desk_commands_and_loads_once_per_change() {
    let mut desk = live_model_with_two_agents();
    let mut cafe = cafe_twin(&desk);

    for action in [
        Action::Next,
        Action::Next,
        Action::Previous,
        Action::First,
        Action::Last,
    ] {
        let desk_reduction = reduce_action(&mut desk, action);
        let cafe_reduction = reduce_action(&mut cafe, action);

        assert_eq!(cafe_reduction, desk_reduction, "action {action:?}");
        assert_eq!(
            cafe.selected_agent_key(),
            desk.selected_agent_key(),
            "action {action:?}"
        );
        assert!(cafe_reduction.commands.len() <= 1, "action {action:?}");
    }
}

#[test]
fn cafe_visit_refresh_and_optional_reviewr_reuse_typed_desk_commands() {
    for action in [Action::Visit, Action::Refresh] {
        let mut desk = live_model_with_two_agents();
        let mut cafe = cafe_twin(&desk);

        let desk_reduction = reduce_action(&mut desk, action);
        let cafe_reduction = reduce_action(&mut cafe, action);

        assert_eq!(cafe_reduction, desk_reduction, "action {action:?}");
    }

    let mut desk = live_model_with_two_agents();
    desk.set_reviewr_available(true);
    let mut cafe = cafe_twin(&desk);
    let desk_reduction = reduce_action(&mut desk, Action::Reviewr);
    let cafe_reduction = reduce_action(&mut cafe, Action::Reviewr);

    assert_eq!(cafe_reduction, desk_reduction);
    assert_eq!(
        cafe_reduction.commands,
        vec![DeskCommand::OpenReviewr {
            pane_id: PaneId::new("w1:p1"),
            qualified_id: "persiyanov.reviewr.open".to_owned(),
        }]
    );
}

#[test]
fn cafe_reply_and_mark_seen_reuse_the_existing_local_and_command_boundaries() {
    let mut desk = live_model_with_two_agents();
    let mut cafe = cafe_twin(&desk);

    for action in [
        Action::Reply,
        Action::TypeCharacter('o'),
        Action::TypeCharacter('k'),
        Action::Submit,
    ] {
        let desk_reduction = reduce_action(&mut desk, action);
        let cafe_reduction = reduce_action(&mut cafe, action);
        assert_eq!(cafe_reduction, desk_reduction, "action {action:?}");
    }

    let mut desk = live_model_with_two_agents();
    let mut cafe = cafe_twin(&desk);
    let desk_seen = reduce_action(&mut desk, Action::MarkSeen);
    let cafe_seen = reduce_action(&mut cafe, Action::MarkSeen);

    assert_eq!(cafe_seen, desk_seen);
    assert_eq!(
        cafe.selected_agent().unwrap().attention,
        desk.selected_agent().unwrap().attention
    );
    assert!(cafe_seen.commands.is_empty());
}

#[test]
fn cafe_search_reuses_selection_and_single_output_load_boundary() {
    let mut desk = searchable_model();
    let mut cafe = cafe_twin(&desk);

    for action in [
        Action::Search,
        Action::TypeCharacter('b'),
        Action::TypeCharacter('e'),
        Action::TypeCharacter('t'),
        Action::TypeCharacter('a'),
        Action::Submit,
    ] {
        let desk_reduction = reduce_action(&mut desk, action);
        let cafe_reduction = reduce_action(&mut cafe, action);
        assert_eq!(cafe_reduction, desk_reduction, "action {action:?}");
    }

    assert_eq!(cafe.selected_agent_key(), desk.selected_agent_key());
    assert_eq!(cafe.modal(), &Modal::None);
}
