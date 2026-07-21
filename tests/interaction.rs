use std::ops::ControlFlow;

use questmancer::{
    app::{GuildFocus, Modal, Model, RuntimeSettings, View},
    command::AgentCommand,
    domain::{
        AdventurerPersona, AgentKey, DomainState, GuildAttention, GuildSummons, PaneId, PersonaKey,
        Timestamp, WorkspaceId,
    },
    herdr::protocol::{SessionSnapshotResult, SuccessResponse},
    interaction::{reduce_action, reduce_scene_action},
    ledger::LedgerPageId,
    scene::{
        SceneFrame, SceneInteractable, SceneInteractableRegion, pixel::PixelRect, stage::WorldScene,
    },
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
    let mut model = Model::new(View::Guild);
    model.replace_domain(domain);
    model
}

fn live_model_with_two_distinct_personas() -> Model {
    let mut model = live_model_with_two_agents();
    let second_key = model.domain().agents.keys().next_back().unwrap().clone();
    let persona_key = PersonaKey::new("persona-z");
    let second = model.domain_mut().agents.get_mut(&second_key).unwrap();
    second.persona = AdventurerPersona::for_key(persona_key);
    "second persona".clone_into(&mut second.persona.name);
    model.replace_domain(model.domain().clone());
    model
}

fn searchable_model() -> Model {
    let mut model = live_model_with_two_agents();
    let first_key = model.domain().agents.keys().next().unwrap().clone();
    let second_key = model.domain().agents.keys().next_back().unwrap().clone();
    let first_site = model.domain().campaigns.values().next().unwrap().clone();
    {
        let domain = model.domain_mut();
        let first = domain.agents.get_mut(&first_key).unwrap();
        "Alpha".clone_into(&mut first.name);
        "one".clone_into(&mut first.persona.name);
        first.custom_status = Some("waiting".to_owned());

        let second = domain.agents.get_mut(&second_key).unwrap();
        "Beta".clone_into(&mut second.name);
        "two".clone_into(&mut second.persona.name);
        second.custom_status = Some("deploying".to_owned());
        second.workspace_id = WorkspaceId::new("w2");

        let mut second_site = first_site;
        second_site.workspace_id = WorkspaceId::new("w2");
        "Moon Base".clone_into(&mut second_site.label);
        second_site.party = vec![second_key];
        domain.campaigns.insert(WorkspaceId::new("w2"), second_site);
    }
    model
}

fn delve_twin(model: &Model) -> Model {
    let mut twin = model.clone();
    twin.switch_to(View::Delve);
    twin
}

#[test]
fn quit_is_an_explicit_typed_loop_outcome() {
    let mut model = Model::new(View::Guild);

    let reduction = reduce_action(&mut model, Action::Quit);

    assert_eq!(reduction.control, ControlFlow::Break(()));
    assert!(reduction.commands.is_empty());
}

#[test]
fn ledger_opens_pages_toggles_and_blocks_normal_model_actions() {
    let mut model = live_model_with_two_agents();
    let selected = model.selected_agent_key().cloned();

    let opened = reduce_action(&mut model, Action::ToggleLedger);
    assert!(matches!(model.modal(), Modal::LibrarianLedger { .. }));
    assert!(opened.commands.is_empty());
    assert!(opened.persistence.is_empty());

    for action in [
        Action::Next,
        Action::Switch(View::Delve),
        Action::CycleRegion,
        Action::Counsel,
        Action::Search,
    ] {
        let blocked = reduce_action(&mut model, action);
        assert_eq!(model.view(), View::Guild, "ledger leaked {action:?}");
        assert_eq!(
            model.guild_focus(),
            GuildFocus::QuestWall,
            "ledger leaked {action:?}"
        );
        assert_eq!(
            model.selected_agent_key(),
            selected.as_ref(),
            "ledger leaked {action:?}"
        );
        assert!(matches!(model.modal(), Modal::LibrarianLedger { .. }));
        assert!(blocked.commands.is_empty(), "ledger leaked {action:?}");
        assert!(blocked.persistence.is_empty(), "ledger leaked {action:?}");
    }

    let closed = reduce_action(&mut model, Action::ToggleLedger);
    assert_eq!(model.modal(), &Modal::None);
    assert!(closed.commands.is_empty());
    assert!(closed.persistence.is_empty());

    let _ = reduce_action(&mut model, Action::ToggleLedger);
    let dismissed = reduce_action(&mut model, Action::Dismiss);
    assert_eq!(model.modal(), &Modal::None);
    assert!(dismissed.commands.is_empty());
    assert!(dismissed.persistence.is_empty());
}

#[test]
fn view_switch_is_reduced_without_effect_commands() {
    let mut model = Model::new(View::Guild);

    let reduction = reduce_action(&mut model, Action::Switch(View::Delve));

    assert_eq!(model.view(), View::Delve);
    assert_eq!(reduction.control, ControlFlow::Continue(()));
    assert!(reduction.commands.is_empty());
    assert_eq!(reduction.persistence, vec![Command::PersistState]);

    let unchanged = reduce_action(&mut model, Action::Switch(View::Delve));
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
        agent.presence = questmancer::domain::Presence::Idle;
    }

    let redraw = reduce_action(&mut model, Action::Redraw);

    assert!(redraw.commands.is_empty());
    assert!(redraw.persistence.is_empty());
}

#[test]
fn narrow_landmark_cycle_is_deterministic_and_wraps_without_effects() {
    let mut model = Model::new(View::Guild);

    for expected in [
        GuildFocus::CampaignTables,
        GuildFocus::CounselBell,
        GuildFocus::Hearth,
        GuildFocus::Chronicle,
        GuildFocus::Scrying,
        GuildFocus::Spoils,
        GuildFocus::Door,
        GuildFocus::QuestWall,
    ] {
        let reduction = reduce_action(&mut model, Action::CycleRegion);
        assert_eq!(model.guild_focus(), expected);
        assert_eq!(reduction.control, ControlFlow::Continue(()));
        assert!(reduction.commands.is_empty());
        assert!(reduction.persistence.is_empty());
    }
}

#[test]
fn guild_and_delve_switching_preserves_selection_and_landmark_focus() {
    let mut model = live_model_with_two_agents();
    let _ = reduce_action(&mut model, Action::Next);
    model.set_guild_focus(GuildFocus::Scrying);
    let selected = model.selected_agent_key().cloned();

    for view in [View::Delve, View::Guild] {
        let reduction = reduce_action(&mut model, Action::Switch(view));
        assert_eq!(model.view(), view);
        assert_eq!(model.selected_agent_key(), selected.as_ref());
        assert_eq!(model.guild_focus(), GuildFocus::Scrying);
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
        vec![AgentCommand::LoadOutput {
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
        vec![AgentCommand::LoadOutput {
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
        vec![AgentCommand::LoadOutput {
            pane_id: PaneId::new("w1:p2"),
            lines: 80,
        }]
    );

    let previous = reduce_action(&mut model, Action::Previous);
    assert_eq!(
        previous.commands,
        vec![AgentCommand::LoadOutput {
            pane_id: PaneId::new("w1:p1"),
            lines: 80,
        }]
    );
}

#[test]
fn pointer_selection_never_observes_or_sends_counsel() {
    let mut model = live_model_with_two_agents();
    let target = model.domain().agents.keys().next_back().unwrap().clone();
    let scene = questmancer::scene::SceneFrame {
        world: questmancer::scene::stage::WorldScene::GuildHall,
        next_frame_in: None,
        actors: vec![questmancer::scene::SceneActorRegion {
            agent: target.clone(),
            bounds: questmancer::scene::pixel::PixelRect::new(10, 20, 8, 14),
        }],
        interactables: Vec::new(),
    };

    let reduction = reduce_scene_action(
        &mut model,
        Action::SelectAt {
            column: 12,
            row: 11,
        },
        &scene,
    );

    assert_eq!(model.selected_agent_key(), Some(&target));
    assert!(model.adventurer_card_visible());
    assert_eq!(
        reduction.commands,
        vec![AgentCommand::LoadOutput {
            pane_id: PaneId::new("w1:p2"),
            lines: 80,
        }]
    );
    assert!(!reduction.commands.iter().any(|command| matches!(
        command,
        AgentCommand::FocusPane(_) | AgentCommand::SendCounsel { .. }
    )));
}

#[test]
fn clicking_librarian_opens_fresh_ledger_without_selecting_or_commanding_an_agent() {
    let mut model = live_model_with_two_agents();
    let selected = model.selected_agent_key().cloned();
    let scene = SceneFrame {
        world: WorldScene::GuildHall,
        next_frame_in: None,
        actors: Vec::new(),
        interactables: vec![SceneInteractableRegion {
            kind: SceneInteractable::Librarian,
            bounds: PixelRect::new(10, 20, 16, 24),
        }],
    };

    let reduction = reduce_scene_action(
        &mut model,
        Action::SelectAt {
            column: 12,
            row: 11,
        },
        &scene,
    );

    assert_eq!(model.ledger_page(), Some(LedgerPageId::Welcome));
    assert_eq!(model.selected_agent_key(), selected.as_ref());
    assert!(reduction.commands.is_empty());
    assert!(reduction.persistence.is_empty());
}

#[test]
fn selected_adventurer_card_can_be_dismissed_without_clearing_selection() {
    let mut model = live_model_with_two_agents();
    let target = model.domain().agents.keys().next_back().unwrap().clone();
    let scene = questmancer::scene::SceneFrame {
        world: questmancer::scene::stage::WorldScene::GuildHall,
        next_frame_in: None,
        actors: vec![questmancer::scene::SceneActorRegion {
            agent: target.clone(),
            bounds: questmancer::scene::pixel::PixelRect::new(10, 20, 8, 14),
        }],
        interactables: Vec::new(),
    };

    let _ = reduce_scene_action(
        &mut model,
        Action::SelectAt {
            column: 12,
            row: 11,
        },
        &scene,
    );
    assert!(model.adventurer_card_visible());

    let _ = reduce_action(&mut model, Action::Dismiss);

    assert!(!model.adventurer_card_visible());
    assert_eq!(model.selected_agent_key(), Some(&target));
}

#[test]
fn clicking_the_selected_adventurer_toggles_its_card_closed() {
    let mut model = live_model_with_two_agents();
    let target = model.domain().agents.keys().next_back().unwrap().clone();
    let scene = questmancer::scene::SceneFrame {
        world: questmancer::scene::stage::WorldScene::GuildHall,
        next_frame_in: None,
        actors: vec![questmancer::scene::SceneActorRegion {
            agent: target.clone(),
            bounds: questmancer::scene::pixel::PixelRect::new(10, 20, 8, 14),
        }],
        interactables: Vec::new(),
    };
    let click = Action::SelectAt {
        column: 12,
        row: 11,
    };

    let _ = reduce_scene_action(&mut model, click, &scene);
    let _ = reduce_scene_action(&mut model, click, &scene);

    assert!(!model.adventurer_card_visible());
    assert_eq!(model.selected_agent_key(), Some(&target));
}

#[test]
fn visit_focuses_selected_pane_and_empty_selection_is_contextual() {
    let mut selected = live_model_with_two_agents();
    let visit = reduce_action(&mut selected, Action::Observe);
    assert_eq!(
        visit.commands,
        vec![AgentCommand::FocusPane(PaneId::new("w1:p1"))]
    );

    let mut empty = Model::new(View::Guild);
    let no_visit = reduce_action(&mut empty, Action::Observe);
    assert!(no_visit.commands.is_empty());
    assert_eq!(
        empty.status_message(),
        Some("No adventurer is selected to observe.")
    );
}

#[test]
fn managed_pane_selection_never_emits_effect_commands() {
    for (action, expected) in [
        (
            Action::Observe,
            "The Questmancer cannot observe its own managed pane.",
        ),
        (
            Action::Refresh,
            "The scrying table cannot observe the Questmancer's own managed pane.",
        ),
        (
            Action::Counsel,
            "Counsel cannot be issued to the Questmancer's own managed pane.",
        ),
        (
            Action::InspectSpoils,
            "The spoils cannot be inspected for the Questmancer's own managed pane.",
        ),
    ] {
        let mut model = live_model_with_two_agents();
        model.set_managed_pane_id(Some(PaneId::new("w1:p1")));
        model.set_reviewr_available(true);
        let reduction = reduce_action(&mut model, action);
        assert!(reduction.commands.is_empty());
        assert_eq!(model.status_message(), Some(expected), "action {action:?}");
    }
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
        vec![AgentCommand::LoadOutput {
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
    assert_eq!(selected.modal(), &Modal::Scrying);
    assert_eq!(
        refresh.commands,
        vec![AgentCommand::LoadOutput {
            pane_id: PaneId::new("w1:p1"),
            lines: 80,
        }]
    );

    let mut empty = Model::new(View::Guild);
    let no_refresh = reduce_action(&mut empty, Action::Refresh);
    assert!(no_refresh.commands.is_empty());
    assert_eq!(empty.modal(), &Modal::None);
    assert_eq!(
        empty.status_message(),
        Some("No adventurer is selected to scry.")
    );
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
        vec![AgentCommand::LoadOutput {
            pane_id: PaneId::new("w1:p1"),
            lines: 123,
        }]
    );

    model.set_reviewr_available(true);
    let reviewr = reduce_action(&mut model, Action::InspectSpoils);
    assert_eq!(
        reviewr.commands,
        vec![AgentCommand::InspectSpoils {
            pane_id: PaneId::new("w1:p1"),
            qualified_id: "acme.diff.inspect".to_owned(),
        }]
    );
}

#[test]
fn reviewr_opens_only_when_available_for_a_selection() {
    let mut available = live_model_with_two_agents();
    available.set_reviewr_available(true);
    let open = reduce_action(&mut available, Action::InspectSpoils);
    assert_eq!(
        open.commands,
        vec![AgentCommand::InspectSpoils {
            pane_id: PaneId::new("w1:p1"),
            qualified_id: "persiyanov.reviewr.open".to_owned(),
        }]
    );

    let mut unavailable = live_model_with_two_agents();
    let no_open = reduce_action(&mut unavailable, Action::InspectSpoils);
    assert!(no_open.commands.is_empty());
    assert_eq!(
        unavailable.status_message(),
        Some("The spoils cannot be inspected here: Reviewr is unavailable.")
    );

    let mut empty = Model::new(View::Guild);
    empty.set_reviewr_available(true);
    let no_selection = reduce_action(&mut empty, Action::InspectSpoils);
    assert!(no_selection.commands.is_empty());
    assert_eq!(
        empty.status_message(),
        Some("The spoils cannot be inspected here: no adventurer is selected.")
    );
}

#[test]
fn counsel_submit_sends_the_exact_draft_to_the_selected_pane() {
    let mut model = live_model_with_two_agents();

    let opened = reduce_action(&mut model, Action::Counsel);
    assert!(opened.commands.is_empty());
    assert_eq!(
        model.modal(),
        &Modal::Counsel {
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
        vec![AgentCommand::SendCounsel {
            pane_id: PaneId::new("w1:p1"),
            text: "  use jsonb ".to_owned(),
        }]
    );
    assert_eq!(model.modal(), &Modal::None);
}

#[test]
fn empty_counsel_stays_open_while_clear_and_cancel_are_local() {
    let mut model = live_model_with_two_agents();
    let _ = reduce_action(&mut model, Action::Counsel);
    let _ = reduce_action(&mut model, Action::TypeCharacter(' '));

    let empty = reduce_action(&mut model, Action::Submit);
    assert!(empty.commands.is_empty());
    assert_eq!(
        model.status_message(),
        Some("Counsel cannot be issued: the message is empty.")
    );
    assert!(matches!(model.modal(), Modal::Counsel { .. }));

    let _ = reduce_action(&mut model, Action::TypeCharacter('x'));
    let cleared = reduce_action(&mut model, Action::ClearInput);
    assert!(cleared.commands.is_empty());
    assert_eq!(
        model.modal(),
        &Modal::Counsel {
            draft: String::new()
        }
    );

    let cancelled = reduce_action(&mut model, Action::Dismiss);
    assert!(cancelled.commands.is_empty());
    assert_eq!(model.modal(), &Modal::None);

    let mut empty_selection = Model::new(View::Guild);
    let no_counsel = reduce_action(&mut empty_selection, Action::Counsel);
    assert!(no_counsel.commands.is_empty());
    assert_eq!(
        empty_selection.status_message(),
        Some("Counsel cannot be issued: no adventurer is selected.")
    );
}

#[test]
fn mark_seen_uses_the_domain_reducer_for_the_selected_agent() {
    let mut model = live_model_with_two_agents();
    assert!(model.selected_agent().unwrap().attention.is_unread());

    let marked = reduce_action(&mut model, Action::AcknowledgeSummons);

    assert!(marked.commands.is_empty());
    assert_eq!(marked.persistence, vec![Command::PersistState]);
    assert!(!model.selected_agent().unwrap().attention.is_unread());
    assert_eq!(model.status_message(), Some("Summons acknowledged."));

    let mut empty = Model::new(View::Guild);
    let no_mark = reduce_action(&mut empty, Action::AcknowledgeSummons);
    assert!(no_mark.commands.is_empty());
    assert!(no_mark.persistence.is_empty());
    assert_eq!(
        empty.status_message(),
        Some("No adventurer is selected to acknowledge.")
    );
}

#[test]
fn acknowledgement_claims_success_only_for_an_unread_summons() {
    let mut clear = live_model_with_two_agents();
    clear
        .domain_mut()
        .agents
        .values_mut()
        .next()
        .unwrap()
        .attention = GuildAttention::Clear;
    clear.clear_action_feedback();

    let clear_result = reduce_action(&mut clear, Action::AcknowledgeSummons);

    assert!(clear_result.persistence.is_empty());
    assert_ne!(clear.status_message(), Some("Summons acknowledged."));

    let mut read = live_model_with_two_agents();
    read.domain_mut()
        .agents
        .values_mut()
        .next()
        .unwrap()
        .attention = GuildAttention::Read {
        summons: GuildSummons::CounselRequested,
        since: Timestamp::from_millis(500),
    };
    read.clear_action_feedback();

    let read_result = reduce_action(&mut read, Action::AcknowledgeSummons);

    assert!(read_result.persistence.is_empty());
    assert_ne!(read.status_message(), Some("Summons acknowledged."));

    let mut unread = live_model_with_two_agents();
    unread.clear_action_feedback();

    let unread_result = reduce_action(&mut unread, Action::AcknowledgeSummons);

    assert_eq!(unread_result.persistence, vec![Command::PersistState]);
    assert_eq!(unread.status_message(), Some("Summons acknowledged."));
    assert!(!unread.selected_agent().unwrap().attention.is_unread());
}

#[test]
fn mark_seen_keeps_the_newly_selected_distinct_persona_selected() {
    let mut model = live_model_with_two_distinct_personas();
    model.select_last_agent();
    let selected = model.selected_agent_key().unwrap().clone();

    let marked = reduce_action(&mut model, Action::AcknowledgeSummons);

    assert!(marked.commands.is_empty());
    assert_eq!(model.selected_agent_key(), Some(&selected));
    assert!(!model.domain().agents[&selected].attention.is_unread());
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
            vec![AgentCommand::LoadOutput {
                pane_id: PaneId::new("w1:p2"),
                lines: 80,
            }],
            "query {query}"
        );
    }
}

#[test]
fn search_matches_visible_presence_class_and_ancestry_terms() {
    use questmancer::domain::{AdventurerClass, Ancestry, Presence};

    for query in [
        "delving", "working", "counsel", "blocked", "wizard", "dwarf",
    ] {
        let mut model = searchable_model();
        let first_key = model.domain().agents.keys().next().unwrap().clone();
        let second_key = model.domain().agents.keys().next_back().unwrap().clone();
        let non_target = model.domain_mut().agents.get_mut(&first_key).unwrap();
        non_target.presence = Presence::Idle;
        non_target.persona.class = AdventurerClass::Barbarian;
        non_target.persona.ancestry = Ancestry::Human;
        let target = model.domain_mut().agents.get_mut(&second_key).unwrap();
        target.presence = if matches!(query, "counsel" | "blocked") {
            Presence::Blocked
        } else {
            Presence::Working
        };
        target.persona.class = AdventurerClass::Wizard;
        target.persona.ancestry = Ancestry::Dwarf;

        let _ = reduce_action(&mut model, Action::Search);
        for character in query.chars() {
            let _ = reduce_action(&mut model, Action::TypeCharacter(character));
        }
        let submitted = reduce_action(&mut model, Action::Submit);

        assert_eq!(
            model.selected_agent_key(),
            Some(&second_key),
            "query {query}"
        );
        assert_eq!(model.modal(), &Modal::None, "query {query}");
        assert_eq!(submitted.commands.len(), 1, "query {query}");
    }

    for query in ["resting", "spoils returned", "victory recorded", "departed"] {
        let mut model = searchable_model();
        let first_key = model.domain().agents.keys().next().unwrap().clone();
        let second_key = model.domain().agents.keys().next_back().unwrap().clone();
        model
            .domain_mut()
            .agents
            .get_mut(&first_key)
            .unwrap()
            .presence = Presence::Working;
        let target = model.domain_mut().agents.get_mut(&second_key).unwrap();
        match query {
            "resting" => target.presence = Presence::Idle,
            "spoils returned" => {
                target.presence = Presence::Done;
                target.attention = GuildAttention::unread(
                    GuildSummons::SpoilsReturned,
                    Timestamp::from_millis(1_000),
                );
            }
            "victory recorded" => {
                target.presence = Presence::Done;
                target.attention = GuildAttention::Read {
                    summons: GuildSummons::SpoilsReturned,
                    since: Timestamp::from_millis(1_000),
                };
            }
            "departed" => target.presence = Presence::Exited,
            _ => unreachable!(),
        }

        let _ = reduce_action(&mut model, Action::Search);
        for character in query.chars() {
            let _ = reduce_action(&mut model, Action::TypeCharacter(character));
        }
        let submitted = reduce_action(&mut model, Action::Submit);

        assert_eq!(
            model.selected_agent_key(),
            Some(&second_key),
            "query {query}"
        );
        assert_eq!(model.modal(), &Modal::None, "query {query}");
        assert_eq!(submitted.commands.len(), 1, "query {query}");
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
    assert_eq!(
        model.status_message(),
        Some("No adventurer or campaign answers \"missing\".")
    );
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
    assert_eq!(
        model.status_message(),
        Some("Enter an adventurer or campaign to search.")
    );
}

#[test]
fn delve_selection_reuses_the_guild_commands_and_loads_once_per_change() {
    let mut guild = live_model_with_two_agents();
    let mut delve = delve_twin(&guild);

    for action in [
        Action::Next,
        Action::Next,
        Action::Previous,
        Action::First,
        Action::Last,
    ] {
        let guild_reduction = reduce_action(&mut guild, action);
        let delve_reduction = reduce_action(&mut delve, action);

        assert_eq!(delve_reduction, guild_reduction, "action {action:?}");
        assert_eq!(
            delve.selected_agent_key(),
            guild.selected_agent_key(),
            "action {action:?}"
        );
        assert!(delve_reduction.commands.len() <= 1, "action {action:?}");
    }
}

#[test]
fn delve_visit_refresh_and_optional_reviewr_reuse_typed_guild_commands() {
    for action in [Action::Observe, Action::Refresh] {
        let mut guild = live_model_with_two_agents();
        let mut delve = delve_twin(&guild);

        let guild_reduction = reduce_action(&mut guild, action);
        let delve_reduction = reduce_action(&mut delve, action);

        assert_eq!(delve_reduction, guild_reduction, "action {action:?}");
    }

    let mut guild = live_model_with_two_agents();
    guild.set_reviewr_available(true);
    let mut delve = delve_twin(&guild);
    let guild_reduction = reduce_action(&mut guild, Action::InspectSpoils);
    let delve_reduction = reduce_action(&mut delve, Action::InspectSpoils);

    assert_eq!(delve_reduction, guild_reduction);
    assert_eq!(
        delve_reduction.commands,
        vec![AgentCommand::InspectSpoils {
            pane_id: PaneId::new("w1:p1"),
            qualified_id: "persiyanov.reviewr.open".to_owned(),
        }]
    );
}

#[test]
fn delve_counsel_and_acknowledgement_reuse_the_existing_local_and_command_boundaries() {
    let mut guild = live_model_with_two_agents();
    let mut delve = delve_twin(&guild);

    for action in [
        Action::Counsel,
        Action::TypeCharacter('o'),
        Action::TypeCharacter('k'),
        Action::Submit,
    ] {
        let guild_reduction = reduce_action(&mut guild, action);
        let delve_reduction = reduce_action(&mut delve, action);
        assert_eq!(delve_reduction, guild_reduction, "action {action:?}");
    }

    let mut guild = live_model_with_two_agents();
    let mut delve = delve_twin(&guild);
    let guild_seen = reduce_action(&mut guild, Action::AcknowledgeSummons);
    let delve_seen = reduce_action(&mut delve, Action::AcknowledgeSummons);

    assert_eq!(delve_seen, guild_seen);
    assert_eq!(
        delve.selected_agent().unwrap().attention,
        guild.selected_agent().unwrap().attention
    );
    assert!(delve_seen.commands.is_empty());
}

#[test]
fn delve_search_reuses_selection_and_single_output_load_boundary() {
    let mut guild = searchable_model();
    let mut delve = delve_twin(&guild);

    for action in [
        Action::Search,
        Action::TypeCharacter('b'),
        Action::TypeCharacter('e'),
        Action::TypeCharacter('t'),
        Action::TypeCharacter('a'),
        Action::Submit,
    ] {
        let guild_reduction = reduce_action(&mut guild, action);
        let delve_reduction = reduce_action(&mut delve, action);
        assert_eq!(delve_reduction, guild_reduction, "action {action:?}");
    }

    assert_eq!(delve.selected_agent_key(), guild.selected_agent_key());
    assert_eq!(delve.modal(), &Modal::None);
}
