use crate::{
    app::{ConnectionState, Model, OutputPreview},
    command::{AgentCommand, CommandExecutor, CommandResult},
    domain::{PaneId, Timestamp},
    herdr::{
        client::HerdrClient,
        environment::HerdrEnvironment,
        event_adapter::{AdapterAction, adapt_update_excluding},
        supervisor::{Backoff, ConnectionSupervisor, ConnectionUpdate},
    },
    interaction::ActionReduction,
    persistence::{PersistedStateV1, PersistenceClient, PersistenceError},
    ui::copy::COUNSEL_ISSUED,
    update::{AppEvent, Command, update},
};
use tokio::{
    sync::{mpsc, watch},
    task::{JoinHandle, JoinSet},
};

#[derive(Debug, PartialEq)]
pub enum RuntimeEvent {
    Connection(ConnectionUpdate),
    Command(CommandResult),
    CommandTaskFailed(String),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RuntimeEffects {
    pub agent_commands: Vec<AgentCommand>,
    pub persistence: Vec<Command>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExit {
    Quit,
    Signal,
    InputClosed,
}

#[derive(Debug)]
pub struct ActionRuntimeEffects {
    pub agent_commands: Vec<AgentCommand>,
    pub persistence_errors: Vec<PersistenceError>,
    pub exit: Option<RuntimeExit>,
}

pub async fn dispatch_action_effects(
    client: &mut PersistenceClient,
    model: &Model,
    reduction: ActionReduction,
) -> ActionRuntimeEffects {
    let persistence_errors =
        dispatch_persistence_effects(client, model, reduction.persistence).await;
    ActionRuntimeEffects {
        agent_commands: reduction.commands,
        persistence_errors,
        exit: reduction.control.is_break().then_some(RuntimeExit::Quit),
    }
}

pub async fn dispatch_persistence_effects(
    client: &mut PersistenceClient,
    model: &Model,
    effects: impl IntoIterator<Item = Command>,
) -> Vec<PersistenceError> {
    let mut errors = Vec::new();
    for effect in effects {
        let result = match effect {
            Command::AppendChronicle(entry) => client.append_chronicle(entry).await,
            Command::PersistState => client
                .stage_state(PersistedStateV1::capture(model))
                .map(drop),
            Command::RequestSnapshot => continue,
        };
        if let Err(error) = result {
            errors.push(error);
        }
    }
    errors
}

#[derive(Debug)]
pub struct RuntimeConnection {
    executor: CommandExecutor,
    update_rx: mpsc::Receiver<ConnectionUpdate>,
    updates_open: bool,
    shutdown_tx: watch::Sender<bool>,
    supervisor_task: Option<JoinHandle<()>>,
    command_tasks: JoinSet<CommandResult>,
}

impl RuntimeConnection {
    pub fn start(environment: &HerdrEnvironment) -> Self {
        let client = HerdrClient::new(environment.socket_path());
        let managed_pane_id = std::env::var("HERDR_PANE_ID")
            .ok()
            .filter(|value| !value.is_empty())
            .map(PaneId::new);
        let executor = CommandExecutor::new(client.clone(), managed_pane_id);
        let supervisor = ConnectionSupervisor::new(client, Backoff::default());
        let (update_tx, update_rx) = mpsc::channel(32);
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let supervisor_task = tokio::spawn(supervisor.run(update_tx, shutdown_rx));

        Self {
            executor,
            update_rx,
            updates_open: true,
            shutdown_tx,
            supervisor_task: Some(supervisor_task),
            command_tasks: JoinSet::new(),
        }
    }

    pub fn schedule(&mut self, commands: impl IntoIterator<Item = AgentCommand>) {
        for command in commands {
            let executor = self.executor.clone();
            self.command_tasks
                .spawn(async move { executor.execute(command).await });
        }
    }

    pub async fn next_event(&mut self) -> RuntimeEvent {
        loop {
            let has_commands = !self.command_tasks.is_empty();
            if !self.updates_open && !has_commands {
                return std::future::pending().await;
            }

            tokio::select! {
                update = self.update_rx.recv(), if self.updates_open => {
                    if let Some(update) = update {
                        return RuntimeEvent::Connection(update);
                    }
                    self.updates_open = false;
                }
                completion = self.command_tasks.join_next(), if has_commands => {
                    match completion {
                        Some(Ok(result)) => return RuntimeEvent::Command(result),
                        Some(Err(error)) => {
                            return RuntimeEvent::CommandTaskFailed(error.to_string());
                        }
                        None => {}
                    }
                }
            }
        }
    }

    pub async fn shutdown(mut self) -> Result<(), tokio::task::JoinError> {
        let _ = self.shutdown_tx.send(true);
        let supervisor_task = self
            .supervisor_task
            .take()
            .expect("runtime supervisor task is owned until shutdown");
        supervisor_task.abort();
        let supervisor_error = match supervisor_task.await {
            Ok(()) => None,
            Err(error) if error.is_cancelled() => None,
            Err(error) => Some(error),
        };

        self.command_tasks.abort_all();
        let mut command_error = None;
        while let Some(result) = self.command_tasks.join_next().await {
            if let Err(error) = result
                && !error.is_cancelled()
                && command_error.is_none()
            {
                command_error = Some(error);
            }
        }
        supervisor_error.or(command_error).map_or(Ok(()), Err)
    }
}

impl Drop for RuntimeConnection {
    fn drop(&mut self) {
        let _ = self.shutdown_tx.send(true);
        if let Some(supervisor_task) = self.supervisor_task.take() {
            supervisor_task.abort();
        }
        self.command_tasks.abort_all();
    }
}

pub fn bootstrap_model(mut model: Model, environment: Option<&HerdrEnvironment>) -> Model {
    if environment.is_some() {
        model.set_connection(ConnectionState::Connecting);
        model.set_connection_diagnostic("connecting to Herdr".to_owned());
    } else {
        model.set_connection_diagnostic(
            "offline: launch from Herdr to connect to the live session".to_owned(),
        );
    }
    model
}

pub fn apply_connection_update(
    model: &mut Model,
    connection_update: ConnectionUpdate,
    observed_at: Timestamp,
) -> RuntimeEffects {
    let discover_reviewr = matches!(connection_update, ConnectionUpdate::Connected(_));
    let diagnostic_is_connection = matches!(&connection_update, ConnectionUpdate::Disconnected(_));
    let before = selected_revision(model);
    let actions = adapt_update_excluding(
        connection_update,
        model.domain(),
        observed_at,
        model.managed_pane_id(),
    );
    let mut effects = RuntimeEffects::default();

    for action in actions {
        match action {
            AdapterAction::Apply(event) => {
                apply_domain_event(model, *event, &mut effects);
            }
            AdapterAction::SetConnection(connection) => {
                let connected = connection == ConnectionState::Connected;
                model.set_connection(connection);
                if connected {
                    model.clear_connection_notice();
                }
            }
            AdapterAction::RequestSnapshot => push_unique_refresh(&mut effects.agent_commands),
            AdapterAction::Diagnostic(message) => {
                if diagnostic_is_connection {
                    model.set_connection_diagnostic(message);
                } else {
                    model.set_integration_diagnostic(message);
                }
            }
        }
    }

    let after = selected_revision(model);
    if after != before
        && let Some((pane_id, _)) = after
    {
        effects.agent_commands.push(AgentCommand::LoadOutput {
            pane_id,
            lines: model.settings().output_preview_lines,
        });
    }
    if discover_reviewr {
        effects.agent_commands.push(AgentCommand::DiscoverReviewr {
            qualified_id: model.settings().reviewr_action.clone(),
        });
    }
    effects
}

pub fn apply_command_result(
    model: &mut Model,
    result: CommandResult,
    observed_at: Timestamp,
) -> RuntimeEffects {
    let mut effects = RuntimeEffects::default();
    match result {
        CommandResult::Focused(pane_id) => {
            model.set_action_feedback(format!("observing {pane_id}"));
        }
        CommandResult::CounselSent(_) => {
            model.set_action_feedback(COUNSEL_ISSUED.to_owned());
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
                    model.set_action_feedback("output preview was truncated".to_owned());
                }
            }
        }
        CommandResult::ReviewrAvailable(available) => {
            model.set_reviewr_available(available);
        }
        CommandResult::SpoilsOpened => {
            model.set_action_feedback("Spoils inspected.".to_owned());
        }
        CommandResult::SnapshotLoaded(snapshot) => {
            apply_domain_event(
                model,
                AppEvent::SnapshotReplaced {
                    snapshot: *snapshot,
                    observed_at,
                    excluded_pane: model.managed_pane_id().cloned(),
                },
                &mut effects,
            );
        }
        CommandResult::Failed { operation, message } => {
            model.set_action_feedback(format!("{operation} failed: {message}"));
        }
    }
    effects
}

fn apply_domain_event(model: &mut Model, event: AppEvent, effects: &mut RuntimeEffects) {
    let state = model.take_domain();
    let (state, domain_commands) = update(state, event);
    model.replace_domain(state);
    for command in domain_commands {
        if command == Command::RequestSnapshot {
            push_unique_refresh(&mut effects.agent_commands);
        } else {
            effects.persistence.push(command);
        }
    }
}

fn selected_revision(model: &Model) -> Option<(PaneId, u64)> {
    model
        .selected_agent()
        .map(|agent| (agent.pane_id.clone(), agent.pane_revision))
}

fn push_unique_refresh(commands: &mut Vec<AgentCommand>) {
    if !commands.contains(&AgentCommand::RefreshSnapshot) {
        commands.push(AgentCommand::RefreshSnapshot);
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, time::Duration};

    use tokio::{sync::oneshot, time::timeout};

    use super::*;

    #[tokio::test]
    async fn shutdown_cancels_a_supervisor_blocked_on_a_saturated_update_channel() {
        let (update_tx, update_rx) = mpsc::channel(1);
        let (first_sent_tx, first_sent_rx) = oneshot::channel();
        let supervisor_task = tokio::spawn(async move {
            update_tx
                .send(ConnectionUpdate::Disconnected("first".into()))
                .await
                .unwrap();
            first_sent_tx.send(()).unwrap();
            update_tx
                .send(ConnectionUpdate::Disconnected("blocked".into()))
                .await
                .unwrap();
        });
        let (shutdown_tx, _shutdown_rx) = watch::channel(false);
        let connection = RuntimeConnection {
            executor: CommandExecutor::new(HerdrClient::new(PathBuf::from("missing.sock")), None),
            update_rx,
            updates_open: true,
            shutdown_tx,
            supervisor_task: Some(supervisor_task),
            command_tasks: JoinSet::new(),
        };
        first_sent_rx.await.unwrap();
        tokio::task::yield_now().await;

        timeout(Duration::from_millis(100), connection.shutdown())
            .await
            .expect("shutdown hung behind the saturated supervisor update channel")
            .unwrap();
    }
}
