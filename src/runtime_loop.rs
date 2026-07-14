use crate::{
    app::{Model, OutputPreview},
    command::{CommandResult, DeskCommand},
    domain::{PaneId, Timestamp},
    herdr::{
        event_adapter::{AdapterAction, adapt_update},
        supervisor::ConnectionUpdate,
    },
    update::{AppEvent, Command, update},
};

pub fn apply_connection_update(
    model: &mut Model,
    connection_update: ConnectionUpdate,
    observed_at: Timestamp,
) -> Vec<DeskCommand> {
    let discover_reviewr = matches!(connection_update, ConnectionUpdate::Connected(_));
    let before = selected_revision(model);
    let actions = adapt_update(connection_update, model.domain(), observed_at);
    let mut commands = Vec::new();

    for action in actions {
        match action {
            AdapterAction::Apply(event) => {
                apply_domain_event(model, *event, &mut commands);
            }
            AdapterAction::SetConnection(connection) => model.set_connection(connection),
            AdapterAction::RequestSnapshot => push_unique_refresh(&mut commands),
            AdapterAction::Diagnostic(message) => model.set_status_message(Some(message)),
        }
    }

    let after = selected_revision(model);
    if after != before
        && let Some((pane_id, _)) = after
    {
        commands.push(DeskCommand::LoadOutput { pane_id, lines: 80 });
    }
    if discover_reviewr {
        commands.push(DeskCommand::DiscoverReviewr);
    }
    commands
}

pub fn apply_command_result(model: &mut Model, result: CommandResult, observed_at: Timestamp) {
    match result {
        CommandResult::Focused(pane_id) => {
            model.set_status_message(Some(format!("visited {pane_id}")));
        }
        CommandResult::ReplySent(pane_id) => {
            model.set_status_message(Some(format!("reply sent to {pane_id}")));
        }
        CommandResult::OutputLoaded {
            pane_id,
            revision,
            text,
            truncated,
        } => {
            let belongs_to_selection = model
                .selected_agent()
                .is_none_or(|agent| agent.pane_id == pane_id);
            if belongs_to_selection {
                model.set_output_preview(Some(OutputPreview {
                    pane_id,
                    revision,
                    text,
                    loading: false,
                    error: None,
                }));
                if truncated {
                    model.set_status_message(Some("output preview was truncated".to_owned()));
                }
            }
        }
        CommandResult::ReviewrAvailable(available) => {
            model.set_reviewr_available(available);
        }
        CommandResult::ReviewrOpened => {
            model.set_status_message(Some("opened reviewr".to_owned()));
        }
        CommandResult::SnapshotLoaded(snapshot) => {
            let mut ignored = Vec::new();
            apply_domain_event(
                model,
                AppEvent::SnapshotReplaced {
                    snapshot: *snapshot,
                    observed_at,
                },
                &mut ignored,
            );
        }
        CommandResult::Failed { operation, message } => {
            model.set_status_message(Some(format!("{operation} failed: {message}")));
        }
    }
}

fn apply_domain_event(model: &mut Model, event: AppEvent, commands: &mut Vec<DeskCommand>) {
    let state = std::mem::take(model.domain_mut());
    let (state, domain_commands) = update(state, event);
    model.replace_domain(state);
    for command in domain_commands {
        if command == Command::RequestSnapshot {
            push_unique_refresh(commands);
        }
    }
}

fn selected_revision(model: &Model) -> Option<(PaneId, u64)> {
    model
        .selected_agent()
        .map(|agent| (agent.pane_id.clone(), agent.pane_revision))
}

fn push_unique_refresh(commands: &mut Vec<DeskCommand>) {
    if !commands.contains(&DeskCommand::RefreshSnapshot) {
        commands.push(DeskCommand::RefreshSnapshot);
    }
}
