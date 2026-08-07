use std::ops::ControlFlow;

use crate::{
    app::{Modal, Model},
    command::AgentCommand,
    domain::{Agent, Presence},
    persistence::PersistedStateV1,
    scene::SceneFrame,
    ui::{
        copy::{SUMMONS_ACKNOWLEDGED, no_match},
        input::Action,
    },
    update::Command,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionReduction {
    pub control: ControlFlow<(), ()>,
    pub commands: Vec<AgentCommand>,
    pub persistence: Vec<Command>,
}

#[must_use]
pub fn reduce_action(model: &mut Model, action: Action) -> ActionReduction {
    let before = PersistedStateV1::capture(model);
    let mut commands = Vec::new();
    if !matches!(action, Action::Redraw | Action::None) {
        model.note_interaction();
    }
    if intercept_reading_modal(model, action) {
        return finish_reduction(model, &before, ControlFlow::Continue(()), commands);
    }
    let control = match action {
        Action::Quit => ControlFlow::Break(()),
        Action::Switch(view) => {
            model.switch_to(view);
            ControlFlow::Continue(())
        }
        Action::NextCampaign => {
            select_next_campaign(model, &mut commands);
            ControlFlow::Continue(())
        }
        Action::First | Action::Last | Action::Next | Action::Previous => {
            select_sequentially(model, action, &mut commands);
            ControlFlow::Continue(())
        }
        Action::NextUrgent => {
            select_next_urgent(model, &mut commands);
            ControlFlow::Continue(())
        }
        Action::Observe => {
            observe_selected(model, &mut commands);
            ControlFlow::Continue(())
        }
        Action::Refresh => {
            refresh_selected(model, &mut commands);
            ControlFlow::Continue(())
        }
        Action::InspectSpoils => {
            inspect_spoils(model, &mut commands);
            ControlFlow::Continue(())
        }
        Action::Counsel => {
            open_counsel(model);
            ControlFlow::Continue(())
        }
        Action::AcknowledgeSummons => {
            mark_read(model);
            ControlFlow::Continue(())
        }
        Action::DeferSummons => {
            defer_summons(model);
            ControlFlow::Continue(())
        }
        Action::Search => {
            model.open_search();
            ControlFlow::Continue(())
        }
        Action::NextResult | Action::PreviousResult => {
            cycle_search(model, action == Action::NextResult, &mut commands);
            ControlFlow::Continue(())
        }
        Action::CycleMotion | Action::CycleCharacterSet | Action::CycleColorMode => {
            cycle_display_preference(model, action);
            ControlFlow::Continue(())
        }
        Action::OpenChronicle => {
            model.open_chronicle();
            ControlFlow::Continue(())
        }
        Action::ToggleLedger => {
            model.toggle_ledger();
            ControlFlow::Continue(())
        }
        Action::TypeCharacter(_) | Action::Backspace | Action::ClearInput => {
            edit_modal_input(model, action);
            ControlFlow::Continue(())
        }
        Action::Dismiss => {
            if model.modal() == &Modal::None {
                model.dismiss_adventurer_card();
            } else {
                model.dismiss_modal();
            }
            ControlFlow::Continue(())
        }
        Action::Submit => {
            match model.modal() {
                Modal::Counsel { .. } => submit_counsel(model, &mut commands),
                Modal::Search { .. } => submit_search(model, &mut commands),
                Modal::None | Modal::LibrarianLedger { .. } | Modal::Scrying | Modal::Chronicle => {
                }
            }
            ControlFlow::Continue(())
        }
        Action::SelectAt { .. } | Action::Redraw | Action::None => ControlFlow::Continue(()),
    };
    finish_reduction(model, &before, control, commands)
}

#[must_use]
pub fn reduce_scene_action(
    model: &mut Model,
    action: Action,
    scene: &SceneFrame,
) -> ActionReduction {
    let Action::SelectAt { column, row } = action else {
        return reduce_action(model, action);
    };
    let before = PersistedStateV1::capture(model);
    model.note_interaction();
    let mut commands = Vec::new();
    match scene.target_at(column, row) {
        Some(crate::scene::SceneTarget::Agent(agent)) => {
            let agent = agent.clone();
            if model.selected_agent_key() == Some(&agent) && model.adventurer_card_visible() {
                model.dismiss_adventurer_card();
            } else {
                select_agent_key(model, &agent, &mut commands);
                model.show_adventurer_card();
            }
        }
        Some(crate::scene::SceneTarget::Interactable(
            crate::scene::SceneInteractable::Librarian,
        )) => model.open_ledger(),
        None => model.dismiss_adventurer_card(),
    }
    finish_reduction(model, &before, ControlFlow::Continue(()), commands)
}

fn observe_selected(model: &mut Model, commands: &mut Vec<AgentCommand>) {
    match selected_pane_state(model) {
        SelectedPane::Available(pane_id) => commands.push(AgentCommand::FocusPane(pane_id)),
        SelectedPane::Managed => model
            .set_action_feedback("The Questmancer cannot observe its own managed pane.".to_owned()),
        SelectedPane::Missing => {
            model.set_action_feedback("No adventurer is selected to observe.".to_owned());
        }
    }
}

fn refresh_selected(model: &mut Model, commands: &mut Vec<AgentCommand>) {
    match selected_pane_state(model) {
        SelectedPane::Available(pane_id) => {
            model.open_scrying();
            commands.push(load_output(model, pane_id));
        }
        SelectedPane::Managed => model.set_action_feedback(
            "The scrying table cannot observe the Questmancer's own managed pane.".to_owned(),
        ),
        SelectedPane::Missing => {
            model.set_action_feedback("No adventurer is selected to scry.".to_owned());
        }
    }
}

/// Swallows actions aimed at the party while a reading surface is open.
///
/// The input layer already refuses to produce most of these from a key press,
/// but actions also arrive from the scene (clicks) and from callers holding an
/// `Action` directly, so the guard has to live here too. Scrying is
/// deliberately absent: `o` refreshes it, which is a documented binding.
fn intercept_reading_modal(model: &mut Model, action: Action) -> bool {
    if matches!(model.modal(), Modal::Chronicle) {
        if matches!(action, Action::Dismiss | Action::OpenChronicle) {
            model.dismiss_modal();
        }
        return true;
    }
    if !matches!(model.modal(), Modal::LibrarianLedger { .. }) {
        return false;
    }
    match action {
        Action::ToggleLedger | Action::Dismiss => model.dismiss_modal(),
        Action::Next => model.next_ledger_page(),
        Action::Previous => model.previous_ledger_page(),
        Action::First => model.first_ledger_page(),
        Action::Last => model.last_ledger_page(),
        _ => {}
    }
    true
}

fn finish_reduction(
    model: &Model,
    before: &PersistedStateV1,
    control: ControlFlow<(), ()>,
    commands: Vec<AgentCommand>,
) -> ActionReduction {
    let persistence = (PersistedStateV1::capture(model) != *before)
        .then_some(Command::PersistState)
        .into_iter()
        .collect();
    ActionReduction {
        control,
        commands,
        persistence,
    }
}

enum SelectedPane {
    Missing,
    Managed,
    Available(crate::domain::PaneId),
}

fn selected_pane_state(model: &Model) -> SelectedPane {
    let Some(agent) = model.selected_agent() else {
        return SelectedPane::Missing;
    };
    if model.managed_pane_id() == Some(&agent.pane_id) {
        SelectedPane::Managed
    } else {
        SelectedPane::Available(agent.pane_id.clone())
    }
}

fn selected_pane(model: &Model) -> Option<crate::domain::PaneId> {
    match selected_pane_state(model) {
        SelectedPane::Available(pane_id) => Some(pane_id),
        SelectedPane::Missing | SelectedPane::Managed => None,
    }
}

fn inspect_spoils(model: &mut Model, commands: &mut Vec<AgentCommand>) {
    match selected_pane_state(model) {
        SelectedPane::Managed => model.set_action_feedback(
            "The spoils cannot be inspected for the Questmancer's own managed pane.".to_owned(),
        ),
        SelectedPane::Missing => model.set_action_feedback(
            "The spoils cannot be inspected here: no adventurer is selected.".to_owned(),
        ),
        SelectedPane::Available(_) if !model.reviewr_available() => {
            model.set_reviewr_availability_diagnostic(
                "The spoils cannot be inspected here: Reviewr is unavailable.".to_owned(),
            );
        }
        SelectedPane::Available(pane_id) => commands.push(AgentCommand::InspectSpoils {
            pane_id,
            qualified_id: model.settings().reviewr_action.clone(),
        }),
    }
}

fn open_counsel(model: &mut Model) {
    match selected_pane_state(model) {
        SelectedPane::Available(_) => model.open_counsel(),
        SelectedPane::Managed => model.set_action_feedback(
            "Counsel cannot be issued to the Questmancer's own managed pane.".to_owned(),
        ),
        SelectedPane::Missing => model
            .set_action_feedback("Counsel cannot be issued: no adventurer is selected.".to_owned()),
    }
}

fn mark_read(model: &mut Model) {
    let Some(agent) = model.selected_agent() else {
        model.set_action_feedback("No adventurer is selected to acknowledge.".to_owned());
        return;
    };
    if !agent.attention.is_unread() {
        model.set_action_feedback("No unread summons await acknowledgement.".to_owned());
        return;
    }
    model.mark_selected_attention_read();
    model.set_action_feedback(SUMMONS_ACKNOWLEDGED.to_owned());
}

fn submit_counsel(model: &mut Model, commands: &mut Vec<AgentCommand>) {
    let Some(draft) = model.counsel_draft().map(str::to_owned) else {
        return;
    };
    if draft.trim().is_empty() {
        model.set_action_feedback("Counsel cannot be issued: the message is empty.".to_owned());
        return;
    }
    let pane_id = match selected_pane_state(model) {
        SelectedPane::Available(pane_id) => pane_id,
        SelectedPane::Managed => {
            model.set_action_feedback(
                "Counsel cannot be issued to the Questmancer's own managed pane.".to_owned(),
            );
            return;
        }
        SelectedPane::Missing => {
            model.set_action_feedback(
                "Counsel cannot be issued: no adventurer is selected.".to_owned(),
            );
            return;
        }
    };
    model.dismiss_modal();
    commands.push(AgentCommand::SendCounsel {
        pane_id,
        text: draft,
    });
}

fn submit_search(model: &mut Model, commands: &mut Vec<AgentCommand>) {
    let Modal::Search { query } = model.modal() else {
        return;
    };
    let query = query.trim().to_lowercase();
    if query == "release the goblins" {
        let now = model.now();
        model.goblins_mut().release(now);
        model.dismiss_modal();
        model.set_action_feedback("The goblins deny any involvement.".to_owned());
        return;
    }
    if query.is_empty() {
        model.set_action_feedback("Enter an adventurer or campaign to search.".to_owned());
        return;
    }
    // Every match, not just the first. A query hitting three adventurers used
    // to pick one silently and never admit the other two existed.
    let matched = model
        .domain()
        .agents
        .iter()
        .filter(|(_, agent)| {
            let site_matches = model
                .domain()
                .campaigns
                .get(&agent.workspace_id)
                .is_some_and(|campaign| campaign.label.to_lowercase().contains(&query));
            agent_matches_search(agent, &query) || site_matches
        })
        .map(|(key, _)| key.clone())
        .collect::<Vec<_>>();

    if matched.is_empty() {
        model.set_action_feedback(no_match(&query));
        return;
    }
    let total = matched.len();
    let before = selected_pane(model);
    model.set_search_results(query.clone(), matched);
    model.domain_mut().selected_agent = None;
    model.cycle_search_result(true);
    model.show_adventurer_card();
    let after = selected_pane(model);
    model.dismiss_modal();
    if total == 1 {
        model.clear_action_feedback();
    } else {
        model.set_action_feedback(format!("1/{total} matching \"{query}\" · n/N for the rest"));
    }
    if after != before
        && let Some(pane_id) = after
    {
        commands.push(load_output(model, pane_id));
    }
}

/// Walks the matches a search found.
fn cycle_search(model: &mut Model, forward: bool, commands: &mut Vec<AgentCommand>) {
    let before = selected_pane(model);
    let Some((position, total)) = model.cycle_search_result(forward) else {
        model.set_action_feedback(if model.search_query().is_empty() {
            "No search to walk. Press / to search.".to_owned()
        } else {
            format!(
                "Nothing still matches \"{}\".",
                model.search_query().to_owned()
            )
        });
        return;
    };
    model.show_adventurer_card();
    let query = model.search_query().to_owned();
    model.set_action_feedback(format!("{position}/{total} matching \"{query}\""));
    let after = selected_pane(model);
    if after != before
        && let Some(pane_id) = after
    {
        commands.push(load_output(model, pane_id));
    }
}

fn agent_matches_search(agent: &Agent, query: &str) -> bool {
    agent.name.to_lowercase().contains(query)
        || agent.persona.name.to_lowercase().contains(query)
        || agent
            .persona
            .epithet
            .as_str()
            .to_lowercase()
            .contains(query)
        || agent
            .custom_status
            .as_ref()
            .is_some_and(|status| status.to_lowercase().contains(query))
        || format!("{:?}", agent.persona.class)
            .to_lowercase()
            .contains(query)
        || format!("{:?}", agent.persona.ancestry)
            .to_lowercase()
            .contains(query)
        || visible_presence_terms(agent)
            .iter()
            .any(|term| term.contains(query))
}

fn visible_presence_terms(agent: &Agent) -> &'static [&'static str] {
    match agent.presence {
        Presence::Working => &["working", "delving"],
        Presence::Blocked => &["blocked", "counsel", "counsel requested"],
        Presence::Done if agent.attention.is_unread() => {
            &["completed", "spoils", "spoils returned"]
        }
        Presence::Done => &["completed", "victory", "victory recorded"],
        Presence::Idle => &["resting"],
        Presence::Exited => &["departed"],
        Presence::Unknown => &["unknown"],
    }
}

/// Sets the selected summons aside, or explains why it cannot.
fn defer_summons(model: &mut Model) {
    if model.defer_selected_summons() {
        let minutes = Model::SNOOZE.as_secs() / 60;
        model.set_action_feedback(format!("Set aside for {minutes} minutes."));
    } else {
        model.set_action_feedback("That adventurer has no summons to set aside.".to_owned());
    }
}

/// The four plain selection moves, which differ only in where they land.
fn select_sequentially(model: &mut Model, action: Action, commands: &mut Vec<AgentCommand>) {
    let step: fn(&mut Model) = match action {
        Action::First => Model::select_first_agent,
        Action::Last => Model::select_last_agent,
        Action::Next => Model::select_next_agent,
        Action::Previous => Model::select_previous_agent,
        _ => return,
    };
    select_agent(model, step, commands);
}

/// Cycles one display preference and reports where it landed.
///
/// Reporting matters more than usual here: the change to glyphs or colour
/// depth may be subtle on a given terminal, and a toggle you cannot confirm is
/// a toggle you press twice.
fn cycle_display_preference(model: &mut Model, action: Action) {
    let notice = match action {
        Action::CycleMotion => format!("Motion: {}.", model.cycle_motion()),
        Action::CycleCharacterSet => format!("Glyphs: {}.", model.cycle_character_set()),
        Action::CycleColorMode => format!("Colour: {}.", model.cycle_color_mode()),
        _ => return,
    };
    model.set_action_feedback(notice);
}

/// Applies the three text-editing actions a parchment accepts.
fn edit_modal_input(model: &mut Model, action: Action) {
    match action {
        Action::TypeCharacter(character) => model.push_modal_character(character),
        Action::Backspace => model.backspace_modal_input(),
        Action::ClearInput => model.clear_modal_input(),
        _ => {}
    }
}

/// Moves the selection into the next campaign's party.
///
/// Says so plainly when there is only one campaign, rather than appearing to
/// cycle a list of one.
fn select_next_campaign(model: &mut Model, commands: &mut Vec<AgentCommand>) {
    let before = selected_pane(model);
    if model.select_next_campaign() {
        if selected_pane(model).is_some() {
            model.show_adventurer_card();
        }
        let after = selected_pane(model);
        if after != before
            && let Some(pane_id) = after
        {
            commands.push(load_output(model, pane_id));
        }
    } else {
        model.set_action_feedback("The party is all on one campaign.".to_owned());
    }
}

/// Jumps to the next adventurer waiting on a human, or says nobody is.
///
/// The silent case matters: moving the selection somewhere arbitrary when
/// nothing is urgent would teach the key to be untrustworthy.
fn select_next_urgent(model: &mut Model, commands: &mut Vec<AgentCommand>) {
    let before = selected_pane(model);
    if model.select_next_agent_awaiting_a_human() {
        if selected_pane(model).is_some() {
            model.show_adventurer_card();
        }
        let after = selected_pane(model);
        if after != before
            && let Some(pane_id) = after
        {
            commands.push(load_output(model, pane_id));
        }
    } else {
        model.set_action_feedback("No adventurer is waiting on you.".to_owned());
    }
}

fn select_agent(model: &mut Model, select: fn(&mut Model), commands: &mut Vec<AgentCommand>) {
    let before = selected_pane(model);
    select(model);
    let after = selected_pane(model);
    if after.is_some() {
        model.show_adventurer_card();
    }
    if after != before
        && let Some(pane_id) = after
    {
        commands.push(load_output(model, pane_id));
    }
}

fn select_agent_key(
    model: &mut Model,
    agent: &crate::domain::AgentKey,
    commands: &mut Vec<AgentCommand>,
) {
    let before = selected_pane(model);
    model.select_agent(agent);
    let after = selected_pane(model);
    if after != before
        && let Some(pane_id) = after
    {
        commands.push(load_output(model, pane_id));
    }
}

fn load_output(model: &Model, pane_id: crate::domain::PaneId) -> AgentCommand {
    AgentCommand::LoadOutput {
        pane_id,
        lines: model.settings().output_preview_lines,
    }
}
