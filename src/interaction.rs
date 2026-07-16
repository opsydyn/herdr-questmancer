use std::ops::ControlFlow;

use crate::{
    app::{Modal, Model},
    command::AgentCommand,
    persistence::PersistedStateV1,
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
    let control = match action {
        Action::Quit => ControlFlow::Break(()),
        Action::Switch(view) => {
            model.switch_to(view);
            ControlFlow::Continue(())
        }
        Action::CycleRegion => {
            model.cycle_region();
            ControlFlow::Continue(())
        }
        Action::First => {
            select_agent(model, Model::select_first_agent, &mut commands);
            ControlFlow::Continue(())
        }
        Action::Last => {
            select_agent(model, Model::select_last_agent, &mut commands);
            ControlFlow::Continue(())
        }
        Action::Next => {
            select_agent(model, Model::select_next_agent, &mut commands);
            ControlFlow::Continue(())
        }
        Action::Previous => {
            select_agent(model, Model::select_previous_agent, &mut commands);
            ControlFlow::Continue(())
        }
        Action::Visit => {
            match selected_pane_state(model) {
                SelectedPane::Available(pane_id) => {
                    commands.push(AgentCommand::FocusPane(pane_id));
                }
                SelectedPane::Managed => model.set_status_message(Some(
                    "The Questmancer cannot observe its own managed pane.".to_owned(),
                )),
                SelectedPane::Missing => model
                    .set_status_message(Some("No adventurer is selected to observe.".to_owned())),
            }
            ControlFlow::Continue(())
        }
        Action::Refresh => {
            match selected_pane_state(model) {
                SelectedPane::Available(pane_id) => commands.push(load_output(model, pane_id)),
                SelectedPane::Managed => model.set_status_message(Some(
                    "The scrying table cannot observe the Questmancer's own managed pane."
                        .to_owned(),
                )),
                SelectedPane::Missing => {
                    model.set_status_message(Some("No adventurer is selected to scry.".to_owned()));
                }
            }
            ControlFlow::Continue(())
        }
        Action::Reviewr => {
            inspect_spoils(model, &mut commands);
            ControlFlow::Continue(())
        }
        Action::Counsel => {
            open_counsel(model);
            ControlFlow::Continue(())
        }
        Action::MarkSeen => {
            mark_read(model);
            ControlFlow::Continue(())
        }
        Action::Search => {
            model.open_search();
            ControlFlow::Continue(())
        }
        Action::TypeCharacter(character) => {
            model.push_modal_character(character);
            ControlFlow::Continue(())
        }
        Action::Backspace => {
            model.backspace_modal_input();
            ControlFlow::Continue(())
        }
        Action::ClearInput => {
            model.clear_modal_input();
            ControlFlow::Continue(())
        }
        Action::Dismiss => {
            model.dismiss_modal();
            ControlFlow::Continue(())
        }
        Action::Submit => {
            match model.modal() {
                Modal::Counsel { .. } => submit_counsel(model, &mut commands),
                Modal::Search { .. } => submit_search(model, &mut commands),
                Modal::None | Modal::Help => {}
            }
            ControlFlow::Continue(())
        }
        _ => ControlFlow::Continue(()),
    };
    finish_reduction(model, &before, control, commands)
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
        SelectedPane::Managed => model.set_status_message(Some(
            "The spoils cannot be inspected for the Questmancer's own managed pane.".to_owned(),
        )),
        SelectedPane::Missing => model.set_status_message(Some(
            "The spoils cannot be inspected here: no adventurer is selected.".to_owned(),
        )),
        SelectedPane::Available(_) if !model.reviewr_available() => {
            model.set_status_message(Some(
                "The spoils cannot be inspected here: Reviewr is unavailable.".to_owned(),
            ));
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
        SelectedPane::Managed => model.set_status_message(Some(
            "Counsel cannot be issued to the Questmancer's own managed pane.".to_owned(),
        )),
        SelectedPane::Missing => model.set_status_message(Some(
            "Counsel cannot be issued: no adventurer is selected.".to_owned(),
        )),
    }
}

fn mark_read(model: &mut Model) {
    let Some(agent) = model.selected_agent() else {
        model.set_status_message(Some("No adventurer is selected to acknowledge.".to_owned()));
        return;
    };
    if !agent.attention.is_unread() {
        model.set_status_message(Some("No unread summons await acknowledgement.".to_owned()));
        return;
    }
    model.mark_selected_attention_read();
    model.set_status_message(Some(SUMMONS_ACKNOWLEDGED.to_owned()));
}

fn submit_counsel(model: &mut Model, commands: &mut Vec<AgentCommand>) {
    let Some(draft) = model.counsel_draft().map(str::to_owned) else {
        return;
    };
    if draft.trim().is_empty() {
        model.set_status_message(Some(
            "Counsel cannot be issued: the message is empty.".to_owned(),
        ));
        return;
    }
    let pane_id = match selected_pane_state(model) {
        SelectedPane::Available(pane_id) => pane_id,
        SelectedPane::Managed => {
            model.set_status_message(Some(
                "Counsel cannot be issued to the Questmancer's own managed pane.".to_owned(),
            ));
            return;
        }
        SelectedPane::Missing => {
            model.set_status_message(Some(
                "Counsel cannot be issued: no adventurer is selected.".to_owned(),
            ));
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
        model.set_status_message(Some("The goblins deny any involvement.".to_owned()));
        return;
    }
    if query.is_empty() {
        model.set_status_message(Some(
            "Enter an adventurer or campaign to search.".to_owned(),
        ));
        return;
    }
    let matched = model.domain().agents.iter().find_map(|(key, agent)| {
        let site_matches = model
            .domain()
            .campaigns
            .get(&agent.workspace_id)
            .is_some_and(|campaign| campaign.label.to_lowercase().contains(&query));
        (agent.name.to_lowercase().contains(&query)
            || agent.persona.name.to_lowercase().contains(&query)
            || agent
                .persona
                .epithet
                .as_str()
                .to_lowercase()
                .contains(&query)
            || agent
                .custom_status
                .as_ref()
                .is_some_and(|status| status.to_lowercase().contains(&query))
            || site_matches)
            .then(|| key.clone())
    });

    let Some(agent_key) = matched else {
        model.set_status_message(Some(no_match(&query)));
        return;
    };
    let before = selected_pane(model);
    model.domain_mut().selected_agent = Some(agent_key);
    let after = selected_pane(model);
    model.dismiss_modal();
    model.set_status_message(None);
    if after != before
        && let Some(pane_id) = after
    {
        commands.push(load_output(model, pane_id));
    }
}

fn select_agent(model: &mut Model, select: fn(&mut Model), commands: &mut Vec<AgentCommand>) {
    let before = selected_pane(model);
    select(model);
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
