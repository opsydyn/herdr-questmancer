use crate::{
    app::{ConnectionState, Model, OutputPreview, View},
    command::{CommandExecutor, CommandResult, DeskCommand},
    domain::{PaneId, Timestamp},
    herdr::{
        client::HerdrClient,
        environment::HerdrEnvironment,
        event_adapter::{AdapterAction, adapt_update},
        supervisor::{Backoff, ConnectionSupervisor, ConnectionUpdate},
    },
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
        let executor = CommandExecutor::new(client.clone());
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

    pub fn schedule(&mut self, commands: impl IntoIterator<Item = DeskCommand>) {
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
        self.command_tasks.abort_all();
        let supervisor_result = self
            .supervisor_task
            .take()
            .expect("runtime supervisor task is owned until shutdown")
            .await;
        let mut command_error = None;
        while let Some(result) = self.command_tasks.join_next().await {
            if let Err(error) = result
                && !error.is_cancelled()
                && command_error.is_none()
            {
                command_error = Some(error);
            }
        }
        supervisor_result.and(command_error.map_or(Ok(()), Err))
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

pub fn bootstrap_model(view: View, environment: Option<&HerdrEnvironment>) -> Model {
    let mut model = Model::new(view);
    if environment.is_some() {
        model.set_connection(ConnectionState::Connecting);
        model.set_status_message(Some("connecting to Herdr".to_owned()));
    } else {
        model.set_status_message(Some(
            "offline: launch from Herdr to connect to the live session".to_owned(),
        ));
    }
    model
}

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
