use std::ops::ControlFlow;

use questmancer::{
    app::{Modal, Model, RuntimeSettings, View},
    command::AgentCommand,
    config::OutputPreviewLines,
    domain::{
        AdventurerPersona, AgentKey, ChronicleEntry, ChronicleEvent, DomainState, GuildAttention,
        GuildSummons, PaneId, PersonaKey, Presence, Timestamp, WorkspaceId,
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
        Action::NextCampaign,
        Action::NextUrgent,
        Action::Counsel,
        Action::Search,
    ] {
        let blocked = reduce_action(&mut model, action);
        assert_eq!(model.view(), View::Guild, "ledger leaked {action:?}");
        // Both jumps set feedback when they cannot move, so silence proves the
        // ledger swallowed the key rather than the key merely finding nothing.
        assert_eq!(model.action_feedback(), None, "ledger leaked {action:?}");
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

/// `Tab` used to cycle eight "landmark" variants that no renderer, overlay or
/// command ever read; the test it replaced asserted exactly that — a
/// deterministic cycle producing no commands and no effects. It now moves the
/// selection into the next campaign's party, which is a grouping the party
/// actually has and previously had no way to traverse.
#[test]
fn tab_moves_the_selection_into_the_next_campaign() {
    let mut model = searchable_model();
    let first = model
        .selected_agent()
        .map(|agent| agent.workspace_id.clone());

    let reduction = reduce_action(&mut model, Action::NextCampaign);
    assert_eq!(reduction.control, ControlFlow::Continue(()));
    let second = model
        .selected_agent()
        .map(|agent| agent.workspace_id.clone());
    assert_ne!(first, second, "Tab must land in a different campaign");

    // And it wraps rather than stopping at the last campaign.
    let _ = reduce_action(&mut model, Action::NextCampaign);
    assert_eq!(
        model
            .selected_agent()
            .map(|agent| agent.workspace_id.clone()),
        first
    );
}

/// One campaign is not a cycle. Saying so beats pretending to move.
#[test]
fn tab_says_so_when_the_whole_party_is_on_one_campaign() {
    let mut model = live_model_with_two_agents();
    let selected = model.selected_agent_key().cloned();

    let _ = reduce_action(&mut model, Action::NextCampaign);

    assert_eq!(model.selected_agent_key(), selected.as_ref());
    assert_eq!(
        model.action_feedback(),
        Some("The party is all on one campaign.")
    );
}

#[test]
fn guild_and_delve_switching_preserves_selection() {
    let mut model = live_model_with_two_agents();
    let _ = reduce_action(&mut model, Action::Next);
    let selected = model.selected_agent_key().cloned();

    for view in [View::Delve, View::Guild] {
        let reduction = reduce_action(&mut model, Action::Switch(view));
        assert_eq!(model.view(), view);
        assert_eq!(model.selected_agent_key(), selected.as_ref());
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
        sidebar_urgency_order: false,
        output_preview_lines: OutputPreviewLines::new(123).unwrap(),
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

/// A party where the adventurer needing counsel is deliberately *not* adjacent
/// to the selection, so a test cannot pass by accidentally stepping one place.
fn model_with_a_buried_call_for_counsel() -> Model {
    let mut model = live_model_with_two_agents();
    let template = model.domain().agents.values().next().unwrap().clone();
    {
        let domain = model.domain_mut();
        domain.agents.clear();
        for (index, name) in ["aa-calm", "bb-calm", "cc-blocked", "dd-calm"]
            .into_iter()
            .enumerate()
        {
            let mut agent = template.clone();
            agent.key = AgentKey::new(name);
            agent.pane_id = PaneId::new(format!("w1:p{index}"));
            agent.presence = Presence::Working;
            agent.attention = GuildAttention::Clear;
            if name == "cc-blocked" {
                agent.presence = Presence::Blocked;
                agent.attention = GuildAttention::unread(
                    GuildSummons::CounselRequested,
                    Timestamp::from_millis(500),
                );
            }
            domain.agents.insert(agent.key.clone(), agent);
        }
        domain.selected_agent = Some(AgentKey::new("aa-calm"));
    }
    model.replace_domain(model.domain().clone());
    model.set_now(Timestamp::from_millis(10_000));
    model
}

/// Selection was sequential only, so reaching the one adventurer that needed a
/// human meant stepping past every adventurer that did not.
#[test]
fn the_urgency_jump_reaches_a_blocked_adventurer_in_one_press() {
    let mut model = model_with_a_buried_call_for_counsel();
    assert_eq!(
        model.selected_agent().map(|agent| agent.key.clone()),
        Some(AgentKey::new("aa-calm"))
    );

    let flow = reduce_action(&mut model, Action::NextUrgent);
    assert!(matches!(flow.control, ControlFlow::Continue(())));
    assert_eq!(
        model.selected_agent().map(|agent| agent.key.clone()),
        Some(AgentKey::new("cc-blocked")),
        "one press must land on the adventurer asking for counsel"
    );
}

/// An unanswered call for counsel outranks a summons somebody has already
/// seen, and within a rank the adventurer who has waited longest comes first.
#[test]
fn urgency_order_puts_the_unanswered_and_longest_waiting_first() {
    let mut model = model_with_a_buried_call_for_counsel();
    let template = model.domain().agents.values().next().unwrap().clone();
    {
        let domain = model.domain_mut();
        let mut seen = template.clone();
        seen.key = AgentKey::new("ee-seen");
        seen.pane_id = PaneId::new("w1:p9");
        seen.presence = Presence::Blocked;
        seen.attention = GuildAttention::Read {
            summons: GuildSummons::CounselRequested,
            since: Timestamp::from_millis(100),
        };
        domain.agents.insert(seen.key.clone(), seen);

        let mut older = template.clone();
        older.key = AgentKey::new("ff-older");
        older.pane_id = PaneId::new("w1:p8");
        older.presence = Presence::Blocked;
        older.attention =
            GuildAttention::unread(GuildSummons::CounselRequested, Timestamp::from_millis(50));
        domain.agents.insert(older.key.clone(), older);
    }
    model.replace_domain(model.domain().clone());

    let waiting = model.adventurers_awaiting_a_human();
    assert_eq!(
        waiting,
        vec![
            AgentKey::new("ff-older"),
            AgentKey::new("cc-blocked"),
            AgentKey::new("ee-seen"),
        ],
        "urgency must rank by what the party needs, not by name"
    );
}

/// Deferring a summons said "not now". The jump has to honour that, or
/// deferring means nothing.
#[test]
fn a_deferred_summons_is_skipped_until_it_expires() {
    let mut model = model_with_a_buried_call_for_counsel();
    {
        let domain = model.domain_mut();
        let blocked = domain.agents.get_mut(&AgentKey::new("cc-blocked")).unwrap();
        blocked.presence = Presence::Working;
        blocked.attention = GuildAttention::Deferred {
            summons: GuildSummons::CounselRequested,
            since: Timestamp::from_millis(500),
            until: Timestamp::from_millis(60_000),
        };
    }
    model.replace_domain(model.domain().clone());

    assert!(model.adventurers_awaiting_a_human().is_empty());
    let _ = reduce_action(&mut model, Action::NextUrgent);
    assert_eq!(
        model.selected_agent().map(|agent| agent.key.clone()),
        Some(AgentKey::new("aa-calm")),
        "with nobody waiting the selection must not wander"
    );
    assert_eq!(
        model.action_feedback(),
        Some("No adventurer is waiting on you.")
    );

    // Once the snooze expires the same adventurer is waiting again.
    model.set_now(Timestamp::from_millis(90_000));
    assert_eq!(
        model.adventurers_awaiting_a_human(),
        vec![AgentKey::new("cc-blocked")]
    );
}

/// With several waiting, repeated presses walk them all and come back round.
#[test]
fn repeated_urgency_jumps_cycle_every_waiting_adventurer() {
    let mut model = model_with_a_buried_call_for_counsel();
    {
        let domain = model.domain_mut();
        let second = domain.agents.get_mut(&AgentKey::new("dd-calm")).unwrap();
        second.presence = Presence::Blocked;
        second.attention =
            GuildAttention::unread(GuildSummons::CounselRequested, Timestamp::from_millis(900));
    }
    model.replace_domain(model.domain().clone());

    let mut visited = Vec::new();
    for _ in 0..3 {
        let _ = reduce_action(&mut model, Action::NextUrgent);
        visited.push(
            model
                .selected_agent()
                .map(|agent| agent.key.clone())
                .unwrap(),
        );
    }
    assert_eq!(
        visited,
        vec![
            AgentKey::new("cc-blocked"),
            AgentKey::new("dd-calm"),
            AgentKey::new("cc-blocked"),
        ],
        "the jump must cycle rather than stick on the first"
    );
}

/// Seven Chronicle event types were recorded, persisted to `chronicle.jsonl`
/// and replayed on startup. Exactly one of them reached a human — returned
/// spoils, as a count in a sidebar token. The other six were written and read
/// by nothing.
#[test]
fn the_chronicle_view_shows_what_the_guild_recorded() {
    let mut model = live_model_with_two_agents();
    let first = model.domain().agents.keys().next().unwrap().clone();
    {
        let domain = model.domain_mut();
        domain.selected_agent = None;
        for (millis, event, summary) in [
            (1_000, ChronicleEvent::AdventurerJoined, "aria joined"),
            (2_000, ChronicleEvent::CounselRequested, "aria asked"),
            (3_000, ChronicleEvent::SpoilsReturned, "aria returned"),
        ] {
            domain.chronicle.append(ChronicleEntry::new(
                Timestamp::from_millis(millis),
                Some(first.clone()),
                None,
                None,
                0,
                event,
                summary,
            ));
        }
    }
    model.replace_domain(model.domain().clone());
    model.set_now(Timestamp::from_millis(10_000));

    let _ = reduce_action(&mut model, Action::OpenChronicle);
    assert_eq!(model.modal(), &Modal::Chronicle);

    let shown = model.chronicle_entries(10);
    assert_eq!(shown.len(), 3, "every recorded event must be reachable");
    assert_eq!(
        shown.first().map(|entry| entry.summary.as_str()),
        Some("aria returned"),
        "the Chronicle reads newest first"
    );

    // Every event type carries guild voice, so no entry renders as a bare
    // enum name or an empty line.
    for event in ChronicleEvent::ALL {
        assert!(!event.label().is_empty(), "{event:?} has no label");
    }
}

/// With an adventurer selected the Chronicle answers "what has this one been
/// doing", which is the question the Hall is usually asked.
#[test]
fn the_chronicle_scopes_to_the_selected_adventurer() {
    let mut model = live_model_with_two_agents();
    let first = model.domain().agents.keys().next().unwrap().clone();
    let second = model.domain().agents.keys().next_back().unwrap().clone();
    {
        let domain = model.domain_mut();
        for (millis, who, summary) in [
            (1_000, first.clone(), "first did a thing"),
            (2_000, second.clone(), "second did a thing"),
        ] {
            domain.chronicle.append(ChronicleEntry::new(
                Timestamp::from_millis(millis),
                Some(who),
                None,
                None,
                0,
                ChronicleEvent::DelveBegan,
                summary,
            ));
        }
        domain.selected_agent = Some(second.clone());
    }
    model.replace_domain(model.domain().clone());

    let scoped = model.chronicle_entries(10);
    assert_eq!(scoped.len(), 1);
    assert_eq!(scoped[0].summary, "second did a thing");

    model.domain_mut().selected_agent = None;
    assert_eq!(
        model.chronicle_entries(10).len(),
        2,
        "with nothing selected the Chronicle covers the whole guild"
    );
}

/// The Chronicle is a reading surface: keys must not act on a party you are
/// not looking at.
#[test]
fn the_chronicle_swallows_keys_that_would_move_the_party() {
    let mut model = live_model_with_two_agents();
    let _ = reduce_action(&mut model, Action::OpenChronicle);
    let selected = model.selected_agent_key().cloned();

    for action in [Action::Next, Action::NextUrgent, Action::NextCampaign] {
        let blocked = reduce_action(&mut model, action);
        assert_eq!(model.selected_agent_key(), selected.as_ref());
        assert!(blocked.commands.is_empty());
    }
    assert_eq!(model.modal(), &Modal::Chronicle);

    let _ = reduce_action(&mut model, Action::Dismiss);
    assert_eq!(model.modal(), &Modal::None);
}

/// Search used to `find_map` the first hit and drop the rest: a query matching
/// three adventurers picked one silently and never admitted the others were
/// there.
#[test]
fn search_keeps_every_match_and_n_walks_them() {
    let mut model = searchable_model();
    // Give all three the same searchable fragment.
    {
        let domain = model.domain_mut();
        for (index, agent) in domain.agents.values_mut().enumerate() {
            agent.name = format!("scout-{index}");
        }
        domain.selected_agent = None;
    }
    model.replace_domain(model.domain().clone());
    let total = model.domain().agents.len();
    assert!(total >= 2, "fixture needs several matches");

    let _ = reduce_action(&mut model, Action::Search);
    for character in "scout".chars() {
        let _ = reduce_action(&mut model, Action::TypeCharacter(character));
    }
    let _ = reduce_action(&mut model, Action::Submit);

    assert_eq!(model.search_results().len(), total, "every match is kept");
    assert!(
        model
            .action_feedback()
            .is_some_and(|feedback| feedback.starts_with(&format!("1/{total}"))),
        "the count of matches must be reported, got {:?}",
        model.action_feedback()
    );

    let mut visited = vec![model.selected_agent_key().cloned().unwrap()];
    for _ in 1..total {
        let _ = reduce_action(&mut model, Action::NextResult);
        visited.push(model.selected_agent_key().cloned().unwrap());
    }
    let unique = visited.iter().collect::<std::collections::HashSet<_>>();
    assert_eq!(
        unique.len(),
        total,
        "n must reach every match, not repeat one"
    );

    // And it wraps back to the first.
    let _ = reduce_action(&mut model, Action::NextResult);
    assert_eq!(model.selected_agent_key(), visited.first());
}

/// `N` walks the same set backwards.
#[test]
fn shift_n_walks_the_matches_in_reverse() {
    let mut model = searchable_model();
    {
        let domain = model.domain_mut();
        for (index, agent) in domain.agents.values_mut().enumerate() {
            agent.name = format!("scout-{index}");
        }
        domain.selected_agent = None;
    }
    model.replace_domain(model.domain().clone());

    let _ = reduce_action(&mut model, Action::Search);
    for character in "scout".chars() {
        let _ = reduce_action(&mut model, Action::TypeCharacter(character));
    }
    let _ = reduce_action(&mut model, Action::Submit);
    let first = model.selected_agent_key().cloned().unwrap();

    let _ = reduce_action(&mut model, Action::NextResult);
    let second = model.selected_agent_key().cloned().unwrap();
    assert_ne!(first, second);

    let _ = reduce_action(&mut model, Action::PreviousResult);
    assert_eq!(model.selected_agent_key(), Some(&first));
}

/// Walking with no search behind it says so instead of moving the selection.
#[test]
fn cycling_without_a_search_explains_itself() {
    let mut model = live_model_with_two_agents();
    let selected = model.selected_agent_key().cloned();

    let _ = reduce_action(&mut model, Action::NextResult);

    assert_eq!(model.selected_agent_key(), selected.as_ref());
    assert_eq!(
        model.action_feedback(),
        Some("No search to walk. Press / to search.")
    );
}

/// A result set outlives the party it described. Cycling must not land on an
/// adventurer who has left.
#[test]
fn search_results_drop_adventurers_who_have_gone() {
    let mut model = searchable_model();
    {
        let domain = model.domain_mut();
        for (index, agent) in domain.agents.values_mut().enumerate() {
            agent.name = format!("scout-{index}");
        }
        domain.selected_agent = None;
    }
    model.replace_domain(model.domain().clone());

    let _ = reduce_action(&mut model, Action::Search);
    for character in "scout".chars() {
        let _ = reduce_action(&mut model, Action::TypeCharacter(character));
    }
    let _ = reduce_action(&mut model, Action::Submit);
    let before = model.search_results().len();

    let departed = model.domain().agents.keys().next_back().unwrap().clone();
    model.domain_mut().agents.remove(&departed);

    let after = model.search_results();
    assert_eq!(after.len(), before - 1);
    assert!(
        !after.contains(&departed),
        "a departed adventurer stays out"
    );
}

/// Motion, glyphs and colour depth were configuration-file only: changing any
/// of them meant editing a file and restarting. Reduced motion in particular
/// is a poor thing to gate behind a restart.
#[test]
fn display_preferences_can_be_changed_while_running_and_persist() {
    let mut model = live_model_with_two_agents();
    let before = *model.preferences();

    let motion = reduce_action(&mut model, Action::CycleMotion);
    assert_ne!(model.preferences().motion, before.motion);
    assert_eq!(model.action_feedback(), Some("Motion: reduced."));
    assert!(
        !motion.persistence.is_empty(),
        "a runtime preference change must be written down, or it dies with \
         the session"
    );

    let _ = reduce_action(&mut model, Action::CycleCharacterSet);
    assert_ne!(model.preferences().character_set, before.character_set);
    assert_eq!(model.action_feedback(), Some("Glyphs: ASCII."));

    let _ = reduce_action(&mut model, Action::CycleColorMode);
    assert_ne!(model.preferences().color_mode, before.color_mode);
    assert_eq!(model.action_feedback(), Some("Colour: 16 colours."));
}

/// Motion has three settings and returns to where it started.
#[test]
fn motion_cycles_through_every_setting() {
    let mut model = live_model_with_two_agents();
    let start = model.preferences().motion;

    let mut seen = Vec::new();
    for _ in 0..3 {
        let _ = reduce_action(&mut model, Action::CycleMotion);
        seen.push(model.preferences().motion);
    }

    assert_eq!(seen.len(), 3);
    assert!(
        seen[0] != seen[1] && seen[1] != seen[2] && seen[0] != seen[2],
        "each step must reach a different setting, got {seen:?}"
    );
    assert_eq!(model.preferences().motion, start, "and come back round");
}

/// `GuildAttention::Deferred` shipped from the start with no way to reach it.
/// The reducer could mark a summons read, the urgency jump was already written
/// to skip deferred summons, and no control could put an adventurer into the
/// state at all.
#[test]
fn setting_a_summons_aside_takes_it_out_of_the_urgency_cycle() {
    let mut model = model_with_a_buried_call_for_counsel();
    assert_eq!(
        model.adventurers_awaiting_a_human(),
        vec![AgentKey::new("cc-blocked")]
    );

    let _ = reduce_action(&mut model, Action::NextUrgent);
    assert_eq!(
        model.selected_agent_key(),
        Some(&AgentKey::new("cc-blocked"))
    );

    let set_aside = reduce_action(&mut model, Action::DeferSummons);
    assert!(
        model.action_feedback().is_some_and(|f| f.contains("aside")),
        "setting aside must say so, got {:?}",
        model.action_feedback()
    );
    // Deliberately session-scoped. Acknowledging a summons is written to
    // durable state and survives a restart; setting one aside is not, because
    // the summons still genuinely needs answering and reopening Questmancer is
    // a reasonable moment to be reminded. See `Model::SNOOZE`.
    assert!(
        set_aside.persistence.is_empty(),
        "a snooze must not outlive the session"
    );

    assert!(
        model.adventurers_awaiting_a_human().is_empty(),
        "a set-aside summons must leave the urgency cycle"
    );
    let _ = reduce_action(&mut model, Action::NextUrgent);
    assert_eq!(
        model.action_feedback(),
        Some("No adventurer is waiting on you."),
    );
}

/// Setting aside says "not now", not "handled": the summons and the moment it
/// arrived both survive, so the Hall still shows counsel is wanted.
#[test]
fn setting_aside_keeps_the_summons_rather_than_clearing_it() {
    let mut model = model_with_a_buried_call_for_counsel();
    let key = AgentKey::new("cc-blocked");
    let before = model.domain().agents.get(&key).unwrap().attention.clone();

    model.select_agent(&key);
    let _ = reduce_action(&mut model, Action::DeferSummons);

    let after = &model.domain().agents.get(&key).unwrap().attention;
    assert_eq!(after.summons(), before.summons(), "the summons survives");
    assert_eq!(after.since(), before.since(), "so does when it arrived");
    assert!(after.is_deferred_at(model.now()));
    assert_eq!(
        model.domain().agents.get(&key).unwrap().presence,
        Presence::Blocked,
        "the adventurer still needs counsel; only the queue changed"
    );
}

/// The snooze expires on its own.
#[test]
fn a_set_aside_summons_returns_when_its_time_is_up() {
    let mut model = model_with_a_buried_call_for_counsel();
    model.select_agent(&AgentKey::new("cc-blocked"));
    let _ = reduce_action(&mut model, Action::DeferSummons);
    assert!(model.adventurers_awaiting_a_human().is_empty());

    let later = Timestamp::from_millis(
        model.now().as_millis() + i64::try_from(Model::SNOOZE.as_millis()).unwrap() + 1,
    );
    model.set_now(later);

    assert_eq!(
        model.adventurers_awaiting_a_human(),
        vec![AgentKey::new("cc-blocked")],
        "the summons must come back when the snooze runs out"
    );
}

/// An adventurer with nothing to answer for cannot be snoozed.
#[test]
fn setting_aside_nothing_says_so() {
    let mut model = model_with_a_buried_call_for_counsel();
    model.select_agent(&AgentKey::new("aa-calm"));

    let _ = reduce_action(&mut model, Action::DeferSummons);

    assert_eq!(
        model.action_feedback(),
        Some("That adventurer has no summons to set aside.")
    );
}

/// `Esc` used to bin a counsel draft outright, so a slip mid-sentence cost the
/// whole message.
#[test]
fn a_counsel_draft_survives_dismissing_the_parchment() {
    let mut model = live_model_with_two_agents();
    let _ = reduce_action(&mut model, Action::Counsel);
    for character in "check the migration".chars() {
        let _ = reduce_action(&mut model, Action::TypeCharacter(character));
    }

    let _ = reduce_action(&mut model, Action::Dismiss);
    assert_eq!(model.modal(), &Modal::None);
    assert_eq!(
        model.action_feedback(),
        Some("Draft kept. Press r to take it up again."),
        "keeping the draft is worth saying, or the user retypes it anyway"
    );

    let _ = reduce_action(&mut model, Action::Counsel);
    assert_eq!(
        model.modal(),
        &Modal::Counsel {
            draft: "check the migration".to_owned()
        }
    );
}

/// Drafts belong to the adventurer they were written for. Restoring somebody
/// else's half-written counsel would be worse than losing it.
#[test]
fn counsel_drafts_do_not_follow_the_selection_to_another_adventurer() {
    let mut model = live_model_with_two_distinct_personas();
    let first = model.domain().agents.keys().next().unwrap().clone();
    let second = model.domain().agents.keys().next_back().unwrap().clone();

    model.select_agent(&first);
    let _ = reduce_action(&mut model, Action::Counsel);
    for character in "for the first".chars() {
        let _ = reduce_action(&mut model, Action::TypeCharacter(character));
    }
    let _ = reduce_action(&mut model, Action::Dismiss);

    model.select_agent(&second);
    let _ = reduce_action(&mut model, Action::Counsel);
    assert_eq!(
        model.modal(),
        &Modal::Counsel {
            draft: String::new()
        },
        "the second adventurer starts from a blank parchment"
    );

    let _ = reduce_action(&mut model, Action::Dismiss);
    model.select_agent(&first);
    let _ = reduce_action(&mut model, Action::Counsel);
    assert_eq!(
        model.modal(),
        &Modal::Counsel {
            draft: "for the first".to_owned()
        },
        "and the first adventurer's draft is still waiting"
    );
}

/// Once sent there is nothing to resume.
#[test]
fn a_sent_counsel_leaves_no_draft_behind() {
    let mut model = live_model_with_two_agents();
    let _ = reduce_action(&mut model, Action::Counsel);
    for character in "ship it".chars() {
        let _ = reduce_action(&mut model, Action::TypeCharacter(character));
    }
    let sent = reduce_action(&mut model, Action::Submit);
    assert!(!sent.commands.is_empty(), "counsel must actually be sent");

    let _ = reduce_action(&mut model, Action::Counsel);
    assert_eq!(
        model.modal(),
        &Modal::Counsel {
            draft: String::new()
        }
    );
}

/// An empty parchment is not a draft, and saying "draft kept" for nothing
/// would train the user to ignore the message.
#[test]
fn dismissing_an_empty_parchment_says_nothing() {
    let mut model = live_model_with_two_agents();
    let _ = reduce_action(&mut model, Action::Counsel);
    let _ = reduce_action(&mut model, Action::Dismiss);
    assert_eq!(model.action_feedback(), None);
}
