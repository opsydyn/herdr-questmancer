use std::ops::ControlFlow;

use crate::{
    app::{Modal, Model},
    command::AgentCommand,
    domain::{AdventurerClass, AdventurerPersona, Agent, AgentKey, Presence},
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
    pub work_intent: Option<crate::work::Projection>,
    pub preference: Option<PreferenceChange>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreferenceChange {
    Character {
        adventurer: AgentKey,
        persona: AdventurerPersona,
    },
    PartyName(Option<String>),
}

#[must_use]
#[allow(clippy::too_many_lines)] // The central action dispatch preserves existing effect ordering.
pub fn reduce_action(model: &mut Model, action: Action) -> ActionReduction {
    let before = PersistedStateV1::capture(model);
    let mut commands = Vec::new();
    if !matches!(action, Action::Redraw | Action::None) {
        model.note_interaction();
    }
    if action == Action::Submit && matches!(model.modal(), Modal::CharacterPicker { .. }) {
        let mut reduction = finish_reduction(model, &before, ControlFlow::Continue(()), commands);
        reduction.preference =
            model
                .character_change()
                .map(|(adventurer, persona)| PreferenceChange::Character {
                    adventurer,
                    persona,
                });
        return reduction;
    }
    if action == Action::Submit
        && let Modal::PartyName { draft, .. } = model.modal()
    {
        let name = draft.trim();
        let mut reduction = finish_reduction(model, &before, ControlFlow::Continue(()), commands);
        reduction.preference = Some(PreferenceChange::PartyName(
            (!name.is_empty()).then(|| name.to_owned()),
        ));
        return reduction;
    }
    if intercept_character_picker(model, action) {
        return finish_reduction(model, &before, ControlFlow::Continue(()), commands);
    }
    if action == Action::OpenAuthoring {
        open_authoring(model);
        return finish_reduction(model, &before, ControlFlow::Continue(()), commands);
    }
    if model.modal() == &Modal::OwnedAuthoring {
        return finish_reduction(model, &before, ControlFlow::Continue(()), commands);
    }
    if intercept_war_table(model, action, &mut commands)
        || intercept_work_board(model, action, &mut commands)
        || intercept_inbox(model, action, &mut commands)
    {
        return finish_reduction(model, &before, ControlFlow::Continue(()), commands);
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
        Action::OpenCharacterPicker => {
            model.open_character_picker();
            ControlFlow::Continue(())
        }
        Action::ExplainStatus => {
            commands.extend(
                model
                    .open_status_explanation()
                    .map(AgentCommand::ExplainStatus),
            );
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
        Action::ScrollUp | Action::ScrollDown => {
            model.scroll_reading(action == Action::ScrollDown);
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
        Action::OpenWarTable
        | Action::OpenQuestBoard
        | Action::OpenChronicle
        | Action::ToggleLedger => {
            open_parchment(model, action);
            ControlFlow::Continue(())
        }
        Action::TypeCharacter(_) | Action::Backspace | Action::ClearInput => {
            edit_modal_input(model, action);
            ControlFlow::Continue(())
        }
        Action::Dismiss => {
            if matches!(model.modal(), Modal::PartyName { .. }) {
                model.cancel_party_name();
            } else {
                dismiss(model);
            }
            ControlFlow::Continue(())
        }
        Action::Submit => {
            submit_modal(model, &mut commands);
            ControlFlow::Continue(())
        }
        Action::ToggleChronicleChapter
        | Action::SelectAt { .. }
        | Action::Redraw
        | Action::OpenAuthoring
        | Action::OpenInbox
        | Action::RequestReview
        | Action::ChangeQuestLifecycle
        | Action::EditQuestPrerequisites
        | Action::EditQuestAssignment
        | Action::ImportCheck
        | Action::SettleReview
        | Action::LinkReviewAdventurer
        | Action::NameParty
        | Action::None => ControlFlow::Continue(()),
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
            commands.extend(load_output(model, pane_id));
        }
        SelectedPane::Managed => model.set_action_feedback(
            "The scrying table cannot observe the Questmancer's own managed pane.".to_owned(),
        ),
        SelectedPane::Missing => {
            model.set_action_feedback("No adventurer is selected to scry.".to_owned());
        }
    }
}

fn intercept_character_picker(model: &mut Model, action: Action) -> bool {
    if !matches!(model.modal(), Modal::CharacterPicker { .. }) {
        return false;
    }
    match action {
        Action::Dismiss => model.cancel_character_picker(),
        Action::Next => model.move_character_choice(|choice| choice + 1),
        Action::Previous => model.move_character_choice(|choice| choice.saturating_sub(1)),
        Action::First => model.move_character_choice(|_| 0),
        Action::Last => model.move_character_choice(|_| AdventurerClass::ALL.len()),
        _ => {}
    }
    true
}

/// Swallows actions aimed at the party while a reading surface is open.
///
/// The input layer already refuses to produce most of these from a key press,
/// but actions also arrive from the scene (clicks) and from callers holding an
/// `Action` directly, so the guard has to live here too. Scrying is
/// deliberately absent: `o` refreshes it, which is a documented binding.
fn intercept_reading_modal(model: &mut Model, action: Action) -> bool {
    if matches!(model.modal(), Modal::StatusExplanation(_)) {
        if action == Action::Dismiss {
            model.dismiss_modal();
        }
        return true;
    }
    if matches!(model.modal(), Modal::Chronicle) {
        match action {
            Action::Dismiss | Action::OpenChronicle => model.dismiss_modal(),
            Action::ToggleChronicleChapter => model.toggle_chronicle_chapter(),
            Action::ScrollUp | Action::ScrollDown => {
                model.scroll_reading(action == Action::ScrollDown);
            }
            _ => {}
        }
        return true;
    }
    if !matches!(model.modal(), Modal::LibrarianLedger { .. }) {
        return false;
    }
    match action {
        Action::ToggleLedger | Action::Dismiss => model.dismiss_modal(),
        Action::OpenCharacterPicker => model.open_character_picker(),
        Action::NameParty => model.open_party_name(),
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
        work_intent: model.pending_work().cloned(),
        preference: None,
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
    if let Some(pane_id) = model.uncertain_counsel_pane() {
        commands.push(AgentCommand::FocusPane(pane_id));
        model.set_action_feedback(
            "Inspect the adventurer before abandoning or reissuing this counsel.".to_owned(),
        );
        return;
    }
    if matches!(
        model.counsel_phase(),
        Some(crate::app::CounselPhase::SubmissionFailed { .. })
    ) {
        if let Some((pane_id, request)) = model.retry_counsel_submission() {
            commands.push(AgentCommand::SubmitCounsel { pane_id, request });
        }
        return;
    }
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
    // The parchment stays open. It is the only surface that can report what
    // became of the send, and it closes when the result says so.
    let Some(request) = model.begin_counsel_send(pane_id.clone()) else {
        return;
    };
    commands.push(AgentCommand::SendCounsel {
        pane_id,
        text: draft,
        request,
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
    let first = matched[0].clone();
    model.set_search_results(query.clone(), matched);
    model.select_agent(&first);
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
        commands.extend(load_output(model, pane_id));
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
        commands.extend(load_output(model, pane_id));
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
        || agent.persona.role_label().to_lowercase().contains(query)
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

/// Closes whatever is open, keeping a counsel draft rather than binning it.
fn dismiss(model: &mut Model) {
    if model.modal() == &Modal::None {
        model.dismiss_adventurer_card();
        return;
    }
    if model.abandon_uncertain_counsel() {
        model.dismiss_modal();
        model.set_action_feedback(
            "Unconfirmed counsel abandoned. Inspect before issuing it again.".to_owned(),
        );
        return;
    }
    let kept = model.keep_counsel_draft();
    model.dismiss_modal();
    if kept {
        model.set_action_feedback("Draft kept. Press r to take it up again.".to_owned());
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
            commands.extend(load_output(model, pane_id));
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
    use crate::work::{AttentionTarget, WorkSelection};
    let queue = crate::work::queue(model);
    let current = match model.work_selection() {
        Some(WorkSelection::Obligation(id)) => Some(AttentionTarget::Obligation(id.clone())),
        _ => model
            .selected_agent_key()
            .cloned()
            .map(AttentionTarget::Agent),
    };
    let index = current
        .as_ref()
        .and_then(|current| queue.iter().position(|target| target == current))
        .map_or(0, |i| (i + 1) % queue.len().max(1));
    match queue.get(index) {
        Some(AttentionTarget::Obligation(id)) => {
            if matches!(model.modal(), Modal::QuestInbox { .. }) {
                model.select_inbox_item(id.clone());
                model.open_inbox(false);
            } else {
                model.select_work(WorkSelection::Obligation(id.clone()));
            }
        }
        Some(AttentionTarget::Agent(key)) => {
            let key = key.clone();
            let before = selected_pane(model);
            model.clear_work_selection();
            if matches!(
                model.modal(),
                Modal::QuestBoard | Modal::QuestInbox { .. } | Modal::WarTable
            ) {
                model.dismiss_modal();
            }
            model.select_agent(&key);
            model.show_adventurer_card();
            let after = selected_pane(model);
            if after != before
                && let Some(pane) = after
            {
                commands.extend(load_output(model, pane));
            }
        }
        None => model.set_action_feedback("No adventurer is waiting on you.".into()),
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
        commands.extend(load_output(model, pane_id));
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
        commands.extend(load_output(model, pane_id));
    }
}

fn load_output(model: &mut Model, pane_id: crate::domain::PaneId) -> Option<AgentCommand> {
    let request = model.begin_output_request(&pane_id)?;
    Some(AgentCommand::LoadOutput {
        pane_id,
        lines: model.settings().output_preview_lines.get(),
        request,
    })
}

fn open_parchment(model: &mut Model, action: Action) {
    match action {
        Action::OpenWarTable => model.open_war_table(),
        Action::OpenQuestBoard => model.open_quest_board(),
        Action::OpenChronicle => model.open_chronicle(),
        Action::ToggleLedger => model.toggle_ledger(),
        _ => {}
    }
}
fn intercept_war_table(
    model: &mut Model,
    action: Action,
    commands: &mut Vec<AgentCommand>,
) -> bool {
    if model.modal() != &Modal::WarTable {
        return false;
    }
    match action {
        Action::Dismiss | Action::OpenWarTable => model.dismiss_modal(),
        Action::OpenQuestBoard => model.open_quest_board(),
        Action::Next => model.cycle_war_quest(true),
        Action::Previous => model.cycle_war_quest(false),
        Action::NextCampaign => model.cycle_war_campaign(true),
        Action::PreviousResult => model.cycle_war_campaign(false),
        Action::ScrollUp | Action::ScrollDown => model.scroll_reading(action == Action::ScrollDown),
        Action::NextUrgent => select_next_urgent(model, commands),
        Action::Observe => observe_work(model, commands),
        _ => {}
    }
    true
}
fn observe_work(model: &mut Model, commands: &mut Vec<AgentCommand>) {
    let pane = crate::work::view::selected_quest(model)
        .and_then(|q| crate::work::bound_agent(model, q))
        .map(|a| a.pane_id.clone());
    if let Some(pane) = pane {
        commands.push(AgentCommand::FocusPane(pane));
    } else {
        model.set_action_feedback(
            "Observe unavailable: no verified live adventurer binding.".into(),
        );
    }
}

fn intercept_work_board(
    model: &mut Model,
    action: Action,
    commands: &mut Vec<AgentCommand>,
) -> bool {
    if model.modal() != &Modal::QuestBoard {
        return false;
    }
    match action {
        Action::Dismiss | Action::OpenQuestBoard => model.dismiss_modal(),
        Action::OpenInbox => model.open_inbox(false),
        Action::RequestReview => {
            open_review_form(model, crate::owned_work::authoring::FormKind::RequestReview);
        }
        Action::ChangeQuestLifecycle => open_lifecycle_form(model),
        Action::EditQuestPrerequisites => open_prerequisite_form(model),
        Action::EditQuestAssignment => open_assignment_form(model),
        Action::OpenWarTable => model.open_war_table(),
        Action::Next => model.cycle_work(true),
        Action::Previous => model.cycle_work(false),
        Action::ScrollUp | Action::ScrollDown => {
            model.scroll_reading(action == Action::ScrollDown);
        }
        Action::AcknowledgeSummons => model.request_work_intent(false),
        Action::DeferSummons => model.request_work_intent(true),
        Action::Observe => {
            let pane = crate::work::view::selected_quest(model)
                .and_then(|q| crate::work::bound_agent(model, q))
                .map(|a| a.pane_id.clone());
            if let Some(pane) = pane {
                commands.push(AgentCommand::FocusPane(pane));
            } else {
                model.set_action_feedback(
                    "Observe unavailable: no verified live adventurer binding.".into(),
                );
            }
        }
        Action::NextUrgent => select_next_urgent(model, commands),
        _ => {}
    }
    true
}

fn intercept_inbox(model: &mut Model, action: Action, commands: &mut Vec<AgentCommand>) -> bool {
    use crate::owned_work::authoring::FormKind;
    let Modal::QuestInbox { detail } = *model.modal() else {
        return false;
    };
    match action {
        Action::Dismiss if detail => model.open_inbox(false),
        Action::Dismiss if model.inbox_is_home() => model.dismiss_modal(),
        Action::Dismiss | Action::OpenQuestBoard => model.open_quest_board(),
        Action::Switch(view) => {
            model.dismiss_modal();
            model.switch_to(view);
        }
        Action::Submit if model.work_selection().is_some() => model.open_inbox(true),
        Action::Next => model.cycle_inbox(true, false),
        Action::Previous => model.cycle_inbox(false, false),
        Action::NextResult => model.cycle_inbox(true, true),
        Action::PreviousResult => model.cycle_inbox(false, true),
        Action::ScrollUp | Action::ScrollDown if detail => {
            model.scroll_reading(action == Action::ScrollDown);
        }
        Action::ScrollUp | Action::ScrollDown => {
            model.cycle_inbox(action == Action::ScrollDown, false);
        }
        Action::NextUrgent => select_next_urgent(model, commands),
        Action::AcknowledgeSummons => model.request_inbox_intent(false),
        Action::DeferSummons => model.request_inbox_intent(true),
        Action::Observe => {
            if let Some(agent) = crate::work::inbox::observer(model) {
                commands.push(AgentCommand::FocusVerifiedPane {
                    pane_id: agent.pane_id.clone(),
                    identity: agent.capture_identity.clone(),
                });
            } else {
                model.set_action_feedback(
                    "Observe unavailable: no verified live adventurer binding.".into(),
                );
            }
        }
        Action::LinkReviewAdventurer => model.link_inbox_adventurer(),
        Action::Refresh if model.authoring().ready() => model.authoring_mut().review = None,
        Action::RequestReview => open_review_form(model, FormKind::RequestReview),
        Action::ImportCheck => open_review_form(model, FormKind::RecordCheck),
        Action::SettleReview => open_review_form(model, FormKind::SettleReview),
        _ => {}
    }
    true
}

fn open_review_form(model: &mut Model, kind: crate::owned_work::authoring::FormKind) {
    use crate::{
        owned_work::authoring::{FormKind, FormTarget},
        work::{ObligationState, inbox, view},
    };
    let target = if kind == FormKind::RequestReview {
        view::selected_quest(model)
            .and_then(|id| {
                model
                    .authoring()
                    .metadata
                    .as_ref()?
                    .inventory
                    .quests
                    .get(id)
            })
            .map(|q| FormTarget::Quest {
                id: q.id.clone(),
                title: q.title.clone(),
            })
    } else {
        inbox::owned_review(model).and_then(|(o, _)| {
            let detail = inbox::current_detail(model)?;
            if o.state != ObligationState::Open
                || kind == FormKind::SettleReview
                    && !(detail.review.check_passed() && detail.changes.is_ok())
            {
                return None;
            }
            let title = model.work().quest(o.quest.as_ref()?)?.title.clone();
            Some(FormTarget::Review {
                id: o.id.clone(),
                incarnation: o.incarnation.clone(),
                title,
                head: detail.review.target.head.clone(),
            })
        })
    };
    let Some(target) = target else {
        model.set_action_feedback(match kind {
            FormKind::RequestReview => "Select a quest in your opened local store to request a review.",
            FormKind::RecordCheck => "Select an open owned review and wait for its details to load.",
            _ => "Settlement requires the pinned change summary and a passing check for its candidate.",
        }.into());
        return;
    };
    open_contextual_owned_form(model, kind, target);
}

fn selected_owned_quest(model: &Model) -> Option<(&crate::work::Quest, u64)> {
    let metadata = model.authoring().metadata.as_ref()?;
    let crate::work::WorkSelection::Quest(id) = model.work_selection()? else {
        return None;
    };
    let quest = metadata.inventory.quests.get(id)?;
    Some((quest, metadata.revision))
}

fn open_lifecycle_form(model: &mut Model) {
    use crate::owned_work::{
        QuestLifecycleChange,
        authoring::{FormKind, FormTarget},
    };
    let target = selected_owned_quest(model).and_then(|(quest, revision)| {
        Some(FormTarget::QuestLifecycle {
            id: quest.id.clone(),
            title: quest.title.clone(),
            change: QuestLifecycleChange::for_state(quest.state)?,
            revision,
        })
    });
    if let Some(target) = target {
        open_contextual_owned_form(model, FormKind::QuestLifecycle, target);
    } else {
        model.set_action_feedback(
            "Select a quest in your opened local store to change its state.".into(),
        );
    }
}

fn open_prerequisite_form(model: &mut Model) {
    use crate::owned_work::authoring::{FormKind, FormTarget};
    let target =
        selected_owned_quest(model).map(|(quest, revision)| FormTarget::QuestPrerequisites {
            id: quest.id.clone(),
            title: quest.title.clone(),
            current: quest.dependencies.clone(),
            revision,
        });
    if let Some(target) = target {
        open_contextual_owned_form(model, FormKind::QuestPrerequisites, target);
    } else {
        model.set_action_feedback(
            "Select a quest in your opened local store to edit prerequisites.".into(),
        );
    }
}

fn open_assignment_form(model: &mut Model) {
    use crate::owned_work::authoring::{FormKind, FormTarget};
    let target = selected_owned_quest(model).map(|(quest, revision)| FormTarget::QuestAssignment {
        id: quest.id.clone(),
        title: quest.title.clone(),
        current: quest.assignment.clone(),
        revision,
    });
    if let Some(target) = target {
        open_contextual_owned_form(model, FormKind::QuestAssignment, target);
    } else {
        model.set_action_feedback(
            "Select a quest in your opened local store to edit its assignment.".into(),
        );
    }
}

fn open_contextual_owned_form(
    model: &mut Model,
    kind: crate::owned_work::authoring::FormKind,
    target: crate::owned_work::authoring::FormTarget,
) {
    use crate::owned_work::authoring::Surface;

    let from_inbox = match *model.modal() {
        Modal::QuestInbox { detail } => Some(detail),
        _ => None,
    };
    let authoring = model.authoring_mut();
    authoring.form_with_target(kind, Some(target));
    if authoring.surface == Surface::Form(kind) && authoring.feedback.is_empty() {
        authoring.return_to_inbox = from_inbox;
    } else {
        let feedback = authoring.feedback.clone();
        model.set_action_feedback(feedback);
        return;
    }
    model.open_owned_authoring();
}

fn submit_modal(model: &mut Model, commands: &mut Vec<AgentCommand>) {
    match model.modal() {
        Modal::Counsel { .. } => submit_counsel(model, commands),
        Modal::Search { .. } => submit_search(model, commands),
        Modal::None
        | Modal::LibrarianLedger { .. }
        | Modal::Scrying
        | Modal::Chronicle
        | Modal::OwnedAuthoring
        | Modal::QuestBoard
        | Modal::QuestInbox { .. }
        | Modal::CharacterPicker { .. }
        | Modal::PartyName { .. }
        | Modal::StatusExplanation(_)
        | Modal::WarTable => {}
    }
}

fn open_authoring(model: &mut Model) {
    let from_inbox = match *model.modal() {
        Modal::QuestInbox { detail } => Some(detail),
        _ => None,
    };
    let selected = crate::work::view::selected_quest(model)
        .and_then(|q| model.authoring().metadata.as_ref()?.inventory.quests.get(q))
        .map(|q| q.campaign.clone());
    let selected = selected.or_else(|| match model.work_selection() {
        Some(crate::work::WorkSelection::Obligation(id)) => model
            .authoring()
            .metadata
            .as_ref()?
            .inventory
            .obligations
            .get(id)
            .map(|o| o.campaign.clone()),
        _ => None,
    });
    if model.authoring().fields.is_empty() {
        model.authoring_mut().campaign = selected;
    }
    model.authoring_mut().menu();
    model.authoring_mut().return_to_inbox = from_inbox;
    model.open_owned_authoring();
}
