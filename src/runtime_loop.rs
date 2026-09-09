use std::time::Duration;

use crate::{
    app::{ConnectionState, Model},
    command::{AgentCommand, CommandExecutor, CommandResult},
    domain::{AgentKey, PaneId, Timestamp},
    herdr::{
        client::HerdrClient,
        environment::HerdrEnvironment,
        event_adapter::{AdapterAction, adapt_update_excluding},
        supervisor::{Backoff, ConnectionSupervisor, ConnectionUpdate},
    },
    interaction::ActionReduction,
    persistence::{PersistedStateV1, PersistenceClient, PersistenceError},
    sidebar::SidebarProjection,
    snapshot_refresh::{LiveFacts, SnapshotCompletion, SnapshotPurpose},
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
    pub resubscribe: bool,
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
    supervisor: ConnectionSupervisor,
    update_rx: mpsc::Receiver<ConnectionUpdate>,
    updates_open: bool,
    shutdown_tx: watch::Sender<bool>,
    supervisor_task: Option<JoinHandle<()>>,
    command_tasks: JoinSet<CommandResult>,
    /// Scrying has one read in flight and at most the latest pending refresh.
    /// Its cancellation must never touch the counsel/focus command set.
    output_tasks: JoinSet<CommandResult>,
    snapshot_tasks: JoinSet<CommandResult>,
    pending_snapshot: Option<AgentCommand>,
    pending_output: Option<AgentCommand>,
    last_sidebar_projection: Option<SidebarProjection>,
    /// Whether Questmancer asked Herdr to order its agent list. Tracked so
    /// shutdown clears exactly what was set and nothing else.
    urgency_view_set: bool,
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
        let supervisor_task = tokio::spawn(supervisor.clone().run(update_tx, shutdown_rx));

        Self {
            executor,
            supervisor,
            update_rx,
            updates_open: true,
            shutdown_tx,
            supervisor_task: Some(supervisor_task),
            command_tasks: JoinSet::new(),
            output_tasks: JoinSet::new(),
            snapshot_tasks: JoinSet::new(),
            pending_snapshot: None,
            pending_output: None,
            last_sidebar_projection: None,
            urgency_view_set: false,
        }
    }

    pub fn schedule(&mut self, commands: impl IntoIterator<Item = AgentCommand>) {
        for command in commands {
            if matches!(command, AgentCommand::RefreshSnapshot(_)) {
                self.pending_snapshot = Some(command);
                self.start_pending_snapshot();
                continue;
            }
            if matches!(command, AgentCommand::LoadOutput { .. }) {
                self.pending_output = Some(command);
                self.start_pending_output();
                continue;
            }
            match &command {
                AgentCommand::PublishMarginalia(projection) => {
                    self.last_sidebar_projection = Some(projection.clone());
                }
                AgentCommand::SetUrgencyView => self.urgency_view_set = true,
                _ => {}
            }
            let executor = self.executor.clone();
            self.command_tasks
                .spawn(async move { executor.execute(command).await });
        }
    }

    fn start_pending_output(&mut self) {
        if self.output_tasks.is_empty()
            && let Some(command) = self.pending_output.take()
        {
            let executor = self.executor.clone();
            self.output_tasks
                .spawn(async move { executor.execute(command).await });
        }
    }

    fn start_pending_snapshot(&mut self) {
        if self.snapshot_tasks.is_empty()
            && let Some(command) = self.pending_snapshot.take()
        {
            let executor = self.executor.clone();
            self.snapshot_tasks
                .spawn(async move { executor.execute(command).await });
        }
    }

    fn cancel_snapshots(&mut self) {
        self.pending_snapshot = None;
        self.snapshot_tasks.abort_all();
    }

    /// Replace only our subscription task/channel. Dropping the old receiver
    /// also discards buffered updates from the superseded connection cycle.
    /// The shared Herdr server and counsel/focus/output task sets are untouched.
    pub fn resubscribe(&mut self) {
        self.cancel_snapshots();
        if let Some(task) = self.supervisor_task.take() {
            task.abort();
        }
        let (tx, rx) = mpsc::channel(32);
        self.update_rx = rx;
        self.updates_open = true;
        self.supervisor_task = Some(tokio::spawn(
            self.supervisor
                .clone()
                .run(tx, self.shutdown_tx.subscribe()),
        ));
    }

    pub async fn next_event(&mut self) -> RuntimeEvent {
        loop {
            let has_commands = !self.command_tasks.is_empty();
            let has_output = !self.output_tasks.is_empty();
            let has_snapshots = !self.snapshot_tasks.is_empty();
            if !self.updates_open && !has_commands && !has_output && !has_snapshots {
                return std::future::pending().await;
            }

            tokio::select! {
                update = self.update_rx.recv(), if self.updates_open => {
                    if let Some(update) = update {
                        if !matches!(update, ConnectionUpdate::Event(_)) {
                            self.cancel_snapshots();
                            self.pending_output = None;
                            self.output_tasks.abort_all();
                        }
                        return RuntimeEvent::Connection(update);
                    }
                    self.updates_open = false;
                }
                completion = self.snapshot_tasks.join_next(), if has_snapshots => {
                    self.start_pending_snapshot();
                    match completion {
                        Some(Ok(result)) => return RuntimeEvent::Command(result),
                        Some(Err(error)) if !error.is_cancelled() => {
                            return RuntimeEvent::CommandTaskFailed(error.to_string());
                        }
                        _ => {}
                    }
                }
                completion = self.output_tasks.join_next(), if has_output => {
                    self.start_pending_output();
                    match completion {
                        Some(Ok(result)) => return RuntimeEvent::Command(result),
                        Some(Err(error)) if !error.is_cancelled() => {
                            return RuntimeEvent::CommandTaskFailed(error.to_string());
                        }
                        _ => {}
                    }
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

        let mut command_error = None;
        for tasks in [
            &mut self.command_tasks,
            &mut self.output_tasks,
            &mut self.snapshot_tasks,
        ] {
            tasks.abort_all();
            while let Some(result) = tasks.join_next().await {
                if let Err(error) = result
                    && !error.is_cancelled()
                    && command_error.is_none()
                {
                    command_error = Some(error);
                }
            }
        }
        if let Some(projection) = self.last_sidebar_projection.take() {
            let _ = tokio::time::timeout(
                Duration::from_millis(250),
                self.executor.clear_marginalia(&projection),
            )
            .await;
        }
        if self.urgency_view_set {
            self.urgency_view_set = false;
            let _ = tokio::time::timeout(
                Duration::from_millis(250),
                self.executor.clear_urgency_view(),
            )
            .await;
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
        self.output_tasks.abort_all();
        self.snapshot_tasks.abort_all();
    }
}

pub fn bootstrap_model(mut model: Model, environment: Option<&HerdrEnvironment>) -> Model {
    let mut capture_bytes = [0_u8; 32];
    match getrandom::fill(&mut capture_bytes) {
        Ok(()) => model
            .domain_mut()
            .capture
            .start(crate::domain::CaptureRunId::new(
                blake3::hash(&capture_bytes).to_hex().to_string(),
            )),
        Err(error) => {
            model.set_integration_diagnostic(format!("Chronicle capture unavailable: {error}"));
        }
    }
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
    let publish_marginalia = matches!(&connection_update, ConnectionUpdate::Connected(_));
    let diagnostic_is_connection = matches!(&connection_update, ConnectionUpdate::Disconnected(_));
    let before_marginalia = SidebarProjection::from_domain(
        model.domain(),
        model.now(),
        model.preferences().character_set,
    );
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
                model.set_connection_at(connection, observed_at);
                if connected {
                    model.clear_connection_notice();
                }
            }
            AdapterAction::RequestSnapshot => {
                // A topology/reconciliation hint can supersede a refresh even
                // before the domain has enough data to install the new facts.
                model.snapshot_refresh.facts_changed();
                queue_snapshot_refresh(model, &mut effects);
            }
            AdapterAction::Diagnostic(message) => {
                if diagnostic_is_connection {
                    model.set_connection_diagnostic(message);
                } else {
                    model.set_integration_diagnostic(message);
                }
            }
        }
    }

    refresh_output_after_update(model, before.as_ref(), discover_reviewr, &mut effects);
    if discover_reviewr {
        effects.agent_commands.push(AgentCommand::DiscoverReviewr {
            qualified_id: model.settings().reviewr_action.clone(),
        });
        // Asked for on every fresh connection rather than once: Herdr's view
        // is transient, so a server restart or reconnect drops it.
        if model.settings().sidebar_urgency_order {
            effects.agent_commands.push(AgentCommand::SetUrgencyView);
        }
    }
    let after_marginalia = SidebarProjection::from_domain(
        model.domain(),
        model.now(),
        model.preferences().character_set,
    );
    if publish_marginalia || after_marginalia != before_marginalia {
        effects
            .agent_commands
            .push(AgentCommand::PublishMarginalia(after_marginalia));
    }
    effects
}

fn apply_counsel_result(model: &mut Model, result: CommandResult) {
    match result {
        CommandResult::CounselSent { request, .. } => {
            if model.complete_counsel(request) {
                model.set_counsel_issued(COUNSEL_ISSUED.to_owned());
            }
        }
        CommandResult::CounselTextFailed { request, message } => {
            if model.fail_counsel(request, message.clone()) {
                model.set_action_feedback(format!("Counsel was not issued: {message}"));
            }
        }
        CommandResult::CounselTextUncertain {
            request, message, ..
        } => {
            if model.mark_counsel_text_uncertain(request, message.clone()) {
                model.set_action_feedback(format!("Counsel delivery is unconfirmed: {message}"));
            }
        }
        CommandResult::CounselSubmissionFailed {
            request, message, ..
        } => {
            if model.fail_counsel_submission(request, message.clone()) {
                model.set_action_feedback(format!(
                    "Counsel was written but not submitted: {message}"
                ));
            }
        }
        _ => unreachable!("only counsel results are routed here"),
    }
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
        result @ (CommandResult::CounselSent { .. }
        | CommandResult::CounselTextFailed { .. }
        | CommandResult::CounselTextUncertain { .. }
        | CommandResult::CounselSubmissionFailed { .. }) => apply_counsel_result(model, result),
        CommandResult::OutputLoaded {
            pane_id,
            request,
            revision,
            text,
            truncated,
        } => {
            if model.complete_output_request(request, &pane_id, revision, text) && truncated {
                model.set_action_feedback("output preview was truncated".to_owned());
            }
        }
        CommandResult::OutputFailed {
            pane_id,
            request,
            message,
        } => {
            model.fail_output_request(request, &pane_id, &message);
        }
        CommandResult::ReviewrAvailable(available) => {
            model.set_reviewr_available(available);
            if available {
                model.clear_reviewr_availability_notice();
            }
        }
        CommandResult::SpoilsOpened => {
            model.set_action_feedback("Spoils inspected.".to_owned());
        }
        CommandResult::UrgencyViewFailed { message } => {
            // Never fatal: an unsorted sidebar is Herdr's normal state, and
            // Questmancer's own urgency jump is unaffected.
            model.set_integration_diagnostic(format!("sidebar urgency order failed: {message}"));
        }
        CommandResult::UrgencyViewSet | CommandResult::MarginaliaPublished => {}
        CommandResult::MarginaliaFailed { message } => {
            model.set_integration_diagnostic(format!("sidebar marginalia failed: {message}"));
        }
        CommandResult::SnapshotLoaded { request, snapshot } => {
            let valid = valid_refresh(model, &snapshot, observed_at);
            let subscribed = model.snapshot_refresh.subscriptions_match(&snapshot);
            match model.snapshot_refresh.complete(request, valid, subscribed) {
                SnapshotCompletion::Ignore => {}
                SnapshotCompletion::Retry(next) => {
                    if !valid {
                        model.set_integration_diagnostic(
                            "Chronicle snapshot withheld: conflicting or ambiguous evidence"
                                .to_owned(),
                        );
                    }
                    effects
                        .agent_commands
                        .push(AgentCommand::RefreshSnapshot(next));
                }
                SnapshotCompletion::Resubscribe => {
                    if !valid {
                        model.set_integration_diagnostic(
                            "Chronicle snapshot withheld: conflicting or ambiguous evidence"
                                .to_owned(),
                        );
                    }
                    model.set_connection_at(ConnectionState::Connecting, observed_at);
                    effects.resubscribe = true;
                }
                SnapshotCompletion::Apply => {
                    let before = selected_revision(model);
                    apply_domain_event(
                        model,
                        AppEvent::SnapshotReplaced {
                            purpose: SnapshotPurpose::Refresh(request),
                            snapshot,
                            observed_at,
                            excluded_pane: model.managed_pane_id().cloned(),
                        },
                        &mut effects,
                    );
                    refresh_output_after_update(model, before.as_ref(), false, &mut effects);
                    queue_pending_snapshot(model, &mut effects);
                }
            }
        }
        CommandResult::SnapshotFailed { request, message } => {
            if model.snapshot_refresh.fail(request) {
                model.set_action_feedback(format!("refresh snapshot failed: {message}"));
                queue_pending_snapshot(model, &mut effects);
            }
        }
        CommandResult::Failed { operation, message } => {
            model.set_action_feedback(format!("{operation} failed: {message}"));
        }
    }
    effects
}

fn apply_domain_event(model: &mut Model, event: AppEvent, effects: &mut RuntimeEffects) {
    if let AppEvent::SnapshotReplaced {
        purpose: SnapshotPurpose::Baseline,
        snapshot,
        ..
    } = &event
    {
        model.snapshot_refresh.baseline(snapshot);
    }
    let previous = crate::app::PartyActivity::from_domain(model.domain());
    let before_facts = LiveFacts::from_domain(model.domain());
    let observed_at = match &event {
        AppEvent::SnapshotReplaced { observed_at, .. } => Some(*observed_at),
        AppEvent::AgentStatusChanged { occurred_at, .. } => Some(*occurred_at),
        _ => None,
    };
    let state = model.take_domain();
    let (state, domain_commands) = update(state, event);
    model.replace_domain(state);
    if before_facts != LiveFacts::from_domain(model.domain()) {
        model.snapshot_refresh.facts_changed();
    }
    model.observe_party_activity(&previous, observed_at);
    let unqualified = model
        .domain()
        .agents
        .values()
        .filter(|agent| !agent.capture_identity.is_qualified())
        .count();
    if unqualified > 0 {
        model.set_integration_diagnostic(format!("Chronicle observations withheld for {unqualified} adventurer(s) without unambiguous session identity"));
    }
    for command in domain_commands {
        if command == Command::RequestSnapshot {
            model.snapshot_refresh.facts_changed();
            queue_snapshot_refresh(model, effects);
        } else {
            // `AppendChronicle` is emitted once per genuinely new event, so
            // this is the one place standing can be earned without paying
            // twice for the same piece of work.
            if let Command::AppendChronicle(entry) = &command {
                model.earn_experience(entry.event().experience());
            }
            effects.persistence.push(command);
        }
    }
}

fn selected_revision(
    model: &Model,
) -> Option<(AgentKey, PaneId, u64, crate::domain::CaptureIdentity)> {
    model.selected_agent().map(|agent| {
        (
            agent.key.clone(),
            agent.pane_id.clone(),
            agent.pane_revision,
            agent.capture_identity.clone(),
        )
    })
}

fn refresh_output_after_update(
    model: &mut Model,
    before: Option<&(AgentKey, PaneId, u64, crate::domain::CaptureIdentity)>,
    fresh_connection: bool,
    effects: &mut RuntimeEffects,
) {
    let after = selected_revision(model);
    if (fresh_connection || after.as_ref() != before)
        && let Some((_, pane_id, _, _)) = after
        && let Some(request) = model.begin_output_request(&pane_id)
    {
        effects.agent_commands.push(AgentCommand::LoadOutput {
            pane_id,
            lines: model.settings().output_preview_lines.get(),
            request,
        });
    }
}

/// Request a correlated refresh of current live facts. Repeated requests
/// coalesce until completion; offline requests cannot establish a baseline.
pub fn request_snapshot_refresh(model: &mut Model) -> RuntimeEffects {
    let mut effects = RuntimeEffects::default();
    queue_snapshot_refresh(model, &mut effects);
    effects
}

fn queue_snapshot_refresh(model: &mut Model, effects: &mut RuntimeEffects) {
    if let Some(request) = model.snapshot_refresh.request() {
        effects
            .agent_commands
            .push(AgentCommand::RefreshSnapshot(request));
    }
}

fn queue_pending_snapshot(model: &mut Model, effects: &mut RuntimeEffects) {
    if let Some(request) = model.snapshot_refresh.take_pending() {
        effects
            .agent_commands
            .push(AgentCommand::RefreshSnapshot(request));
    }
}

fn valid_refresh(
    model: &Model,
    snapshot: &crate::herdr::protocol::SessionSnapshot,
    at: Timestamp,
) -> bool {
    if snapshot.protocol != crate::herdr::supervisor::SUPPORTED_PROTOCOL {
        return false;
    }
    let replacement =
        crate::domain::DomainState::from_snapshot_excluding(snapshot, at, model.managed_pane_id());
    let source_count = snapshot
        .agents
        .iter()
        .filter(|agent| {
            model
                .managed_pane_id()
                .is_none_or(|pane| pane.as_str() != agent.pane_id)
        })
        .count();
    // An ambiguous projection cannot qualify an observation. Resubscription
    // can still establish a quiet baseline through the existing source path.
    if replacement.agents.len() != source_count {
        return false;
    }
    replacement.agents.iter().all(|(key, agent)| {
        model.domain().agents.get(key).is_none_or(|previous| {
            let same_revision_stream = previous
                .capture_identity
                .same_incarnation(&agent.capture_identity)
                || (previous.capture_identity == agent.capture_identity
                    && previous.pane_id == agent.pane_id);
            !same_revision_stream
                || agent.pane_revision > previous.pane_revision
                || (agent.pane_revision == previous.pane_revision
                    && agent.presence == previous.presence)
        })
    })
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, time::Duration};

    use tokio::{
        io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
        net::{UnixListener, UnixStream},
        sync::oneshot,
        time::timeout,
    };

    use super::*;
    use crate::{
        app::{CounselRequest, OutputRequest},
        domain::WorkspaceId,
        sidebar::{
            QUEST_CAMPAIGN, QUEST_OMEN, QUEST_ROLE, SidebarAgentTokens, SidebarCampaignTokens,
        },
    };

    fn test_connection(
        socket_path: PathBuf,
    ) -> (RuntimeConnection, mpsc::Sender<ConnectionUpdate>) {
        let (update_tx, update_rx) = mpsc::channel(1);
        let (shutdown_tx, _shutdown_rx) = watch::channel(false);
        (
            RuntimeConnection {
                supervisor: ConnectionSupervisor::new(
                    HerdrClient::new(&socket_path),
                    Backoff::default(),
                ),
                executor: CommandExecutor::new(HerdrClient::new(socket_path), None),
                update_rx,
                updates_open: true,
                shutdown_tx,
                supervisor_task: Some(tokio::spawn(async {})),
                command_tasks: JoinSet::new(),
                output_tasks: JoinSet::new(),
                snapshot_tasks: JoinSet::new(),
                pending_snapshot: None,
                pending_output: None,
                last_sidebar_projection: None,
                urgency_view_set: false,
            },
            update_tx,
        )
    }

    fn output_command(number: u32) -> AgentCommand {
        AgentCommand::LoadOutput {
            pane_id: PaneId::new("w1:p1"),
            lines: number,
            request: OutputRequest(u64::from(number)),
        }
    }

    async fn accept_request(listener: &UnixListener) -> (UnixStream, serde_json::Value) {
        let (mut stream, _) = timeout(Duration::from_secs(1), listener.accept())
            .await
            .unwrap()
            .unwrap();
        let mut line = String::new();
        timeout(
            Duration::from_secs(1),
            BufReader::new(&mut stream).read_line(&mut line),
        )
        .await
        .unwrap()
        .unwrap();
        (stream, serde_json::from_str(&line).unwrap())
    }

    async fn respond_to(
        stream: &mut UnixStream,
        request: &serde_json::Value,
        result: serde_json::Value,
    ) {
        let response = serde_json::json!({"id": request["id"], "result": result});
        stream
            .write_all(format!("{response}\n").as_bytes())
            .await
            .unwrap();
    }

    fn output_response(revision: u64) -> serde_json::Value {
        serde_json::json!({"type": "pane_read", "read": {
            "pane_id": "w1:p1", "workspace_id": "w1", "tab_id": "w1:t1",
            "source": "recent_unwrapped", "format": "text", "text": "output",
            "revision": revision, "truncated": false
        }})
    }

    fn snapshot_command(id: u64) -> AgentCommand {
        AgentCommand::RefreshSnapshot(crate::snapshot_refresh::SnapshotRequest {
            id: crate::snapshot_refresh::SnapshotRequestId(id),
            ..Default::default()
        })
    }

    fn fixture_result(name: &str) -> serde_json::Value {
        let text = match name {
            "pong" => include_str!("../tests/fixtures/herdr/pong.json"),
            "snapshot" => include_str!("../tests/fixtures/herdr/session_snapshot.json"),
            _ => panic!("unknown fixture"),
        };
        serde_json::from_str::<serde_json::Value>(text).unwrap()["result"].clone()
    }

    async fn assert_socket_closed(stream: &mut UnixStream) {
        let mut line = String::new();
        assert_eq!(
            timeout(
                Duration::from_secs(1),
                BufReader::new(stream).read_line(&mut line)
            )
            .await
            .unwrap()
            .unwrap(),
            0
        );
    }

    #[tokio::test]
    async fn snapshot_scheduling_keeps_one_active_and_only_the_latest_pending_read() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("herdr.sock");
        let listener = UnixListener::bind(&path).unwrap();
        let (mut connection, _tx) = test_connection(path);
        connection.schedule([snapshot_command(1)]);
        let (mut first_stream, first) = accept_request(&listener).await;
        connection.schedule((2..=100).map(snapshot_command));
        assert!(
            timeout(Duration::from_millis(50), listener.accept())
                .await
                .is_err()
        );
        respond_to(&mut first_stream, &first, fixture_result("snapshot")).await;
        assert!(
            matches!(timeout(Duration::from_secs(1), connection.next_event()).await.unwrap(),
            RuntimeEvent::Command(CommandResult::SnapshotLoaded { request, .. }) if request.id.0 == 1)
        );
        let (mut last_stream, last) = accept_request(&listener).await;
        assert_eq!(last["method"], "session.snapshot");
        respond_to(&mut last_stream, &last, fixture_result("snapshot")).await;
        assert!(
            matches!(timeout(Duration::from_secs(1), connection.next_event()).await.unwrap(),
            RuntimeEvent::Command(CommandResult::SnapshotLoaded { request, .. }) if request.id.0 == 100)
        );
        assert!(
            timeout(Duration::from_millis(50), listener.accept())
                .await
                .is_err()
        );
        connection.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn snapshot_cancellation_does_not_cancel_counsel_or_selected_output() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("herdr.sock");
        let listener = UnixListener::bind(&path).unwrap();
        let (mut connection, _tx) = test_connection(path);
        connection.schedule([snapshot_command(1)]);
        let (mut snapshot_stream, _) = accept_request(&listener).await;
        connection.schedule([snapshot_command(2), output_command(1)]);
        let (mut output_stream, output) = accept_request(&listener).await;
        assert_eq!(output["method"], "pane.read");
        connection.schedule([AgentCommand::SendCounsel {
            pane_id: PaneId::new("w1:p1"),
            text: "continue".into(),
            request: CounselRequest(21),
        }]);
        let (mut text_stream, text) = accept_request(&listener).await;
        assert_eq!(text["method"], "pane.send_text");
        connection.cancel_snapshots();
        assert_socket_closed(&mut snapshot_stream).await;
        respond_to(&mut text_stream, &text, serde_json::json!({"type":"ok"})).await;
        let (mut keys_stream, keys) = accept_request(&listener).await;
        assert_eq!(keys["method"], "pane.send_keys");
        respond_to(&mut keys_stream, &keys, serde_json::json!({"type":"ok"})).await;
        respond_to(&mut output_stream, &output, output_response(1)).await;
        let mut counsel = false;
        let mut output = false;
        for _ in 0..2 {
            match timeout(Duration::from_secs(1), connection.next_event())
                .await
                .unwrap()
            {
                RuntimeEvent::Command(CommandResult::CounselSent {
                    request: CounselRequest(21),
                    ..
                }) => counsel = true,
                RuntimeEvent::Command(CommandResult::OutputLoaded {
                    request: OutputRequest(1),
                    ..
                }) => output = true,
                event => panic!("unexpected event after snapshot cancellation: {event:?}"),
            }
        }
        assert!(counsel && output);
        assert!(
            timeout(Duration::from_millis(50), listener.accept())
                .await
                .is_err()
        );
        connection.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn resubscription_discards_the_old_channel_and_obtains_a_post_subscribe_baseline() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("herdr.sock");
        let listener = UnixListener::bind(&path).unwrap();
        let (mut connection, old_tx) = test_connection(path);
        old_tx
            .send(ConnectionUpdate::Disconnected(
                "obsolete queued update".into(),
            ))
            .await
            .unwrap();
        connection.schedule([snapshot_command(1)]);
        let (mut old_stream, _) = accept_request(&listener).await;
        connection.resubscribe();
        assert!(old_tx.is_closed());
        assert_socket_closed(&mut old_stream).await;
        for (method, fixture) in [("ping", "pong"), ("session.snapshot", "snapshot")] {
            let (mut stream, request) = accept_request(&listener).await;
            assert_eq!(request["method"], method);
            respond_to(&mut stream, &request, fixture_result(fixture)).await;
        }
        let (mut subscription, request) = accept_request(&listener).await;
        assert_eq!(request["method"], "events.subscribe");
        respond_to(
            &mut subscription,
            &request,
            serde_json::json!({"type":"subscription_started"}),
        )
        .await;
        let (mut stream, request) = accept_request(&listener).await;
        assert_eq!(request["method"], "session.snapshot");
        let mut fresh = fixture_result("snapshot");
        fresh["snapshot"]["workspaces"][0]["label"] = serde_json::json!("fresh baseline");
        respond_to(&mut stream, &request, fresh).await;
        assert!(
            matches!(timeout(Duration::from_secs(1), connection.next_event()).await.unwrap(),
            RuntimeEvent::Connection(ConnectionUpdate::Connected(snapshot)) if snapshot.workspaces[0].label == "fresh baseline")
        );
        connection.shutdown().await.unwrap();
        assert_socket_closed(&mut subscription).await;
    }

    #[tokio::test]
    async fn output_refresh_bursts_coalesce_without_blocking_counsel() {
        let directory = tempfile::tempdir().unwrap();
        let socket_path = directory.path().join("herdr.sock");
        let listener = UnixListener::bind(&socket_path).unwrap();
        let (mut connection, _update_tx) = test_connection(socket_path);
        connection.schedule([output_command(1)]);
        let (mut first_stream, first) = accept_request(&listener).await;
        assert_eq!(first["method"], "pane.read");
        assert_eq!(first["params"]["lines"], 1);

        connection.schedule((2..=100).map(output_command));
        connection.schedule([AgentCommand::SendCounsel {
            pane_id: PaneId::new("w1:p1"),
            text: "review the result".into(),
            request: CounselRequest(7),
        }]);
        let (mut text_stream, text) = accept_request(&listener).await;
        assert_eq!(
            text["method"], "pane.send_text",
            "superseded reads reached Herdr"
        );
        respond_to(&mut text_stream, &text, serde_json::json!({"type": "ok"})).await;
        let (mut keys_stream, keys) = accept_request(&listener).await;
        assert_eq!(keys["method"], "pane.send_keys");
        respond_to(&mut keys_stream, &keys, serde_json::json!({"type": "ok"})).await;
        assert!(matches!(
            timeout(Duration::from_secs(1), connection.next_event())
                .await
                .unwrap(),
            RuntimeEvent::Command(CommandResult::CounselSent {
                request: CounselRequest(7),
                ..
            })
        ));

        respond_to(&mut first_stream, &first, output_response(1)).await;
        assert!(matches!(
            timeout(Duration::from_secs(1), connection.next_event())
                .await
                .unwrap(),
            RuntimeEvent::Command(CommandResult::OutputLoaded {
                request: OutputRequest(1),
                ..
            })
        ));
        let (mut latest_stream, latest) = accept_request(&listener).await;
        assert_eq!(latest["method"], "pane.read");
        assert_eq!(latest["params"]["lines"], 100);
        respond_to(&mut latest_stream, &latest, output_response(100)).await;
        assert!(matches!(
            timeout(Duration::from_secs(1), connection.next_event())
                .await
                .unwrap(),
            RuntimeEvent::Command(CommandResult::OutputLoaded {
                request: OutputRequest(100),
                ..
            })
        ));
        assert!(
            timeout(Duration::from_millis(50), listener.accept())
                .await
                .is_err()
        );
        connection.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn disconnect_cancels_only_output_and_drops_its_pending_refresh() {
        let directory = tempfile::tempdir().unwrap();
        let socket_path = directory.path().join("herdr.sock");
        let listener = UnixListener::bind(&socket_path).unwrap();
        let (mut connection, update_tx) = test_connection(socket_path);
        connection.schedule([output_command(1)]);
        let (mut output_stream, output) = accept_request(&listener).await;
        assert_eq!(output["method"], "pane.read");
        connection.schedule([AgentCommand::SendCounsel {
            pane_id: PaneId::new("w1:p1"),
            text: "finish this check".into(),
            request: CounselRequest(8),
        }]);
        let (mut text_stream, text) = accept_request(&listener).await;
        assert_eq!(text["method"], "pane.send_text");
        connection.schedule([output_command(2)]);
        update_tx
            .send(ConnectionUpdate::Disconnected("closed".into()))
            .await
            .unwrap();
        assert!(matches!(
            timeout(Duration::from_secs(1), connection.next_event())
                .await
                .unwrap(),
            RuntimeEvent::Connection(ConnectionUpdate::Disconnected(_))
        ));

        let mut line = String::new();
        assert_eq!(
            timeout(
                Duration::from_secs(1),
                BufReader::new(&mut output_stream).read_line(&mut line)
            )
            .await
            .unwrap()
            .unwrap(),
            0,
            "the invalidated output socket must be closed"
        );
        respond_to(&mut text_stream, &text, serde_json::json!({"type": "ok"})).await;
        let (mut keys_stream, keys) = accept_request(&listener).await;
        assert_eq!(keys["method"], "pane.send_keys");
        respond_to(&mut keys_stream, &keys, serde_json::json!({"type": "ok"})).await;
        assert!(matches!(
            timeout(Duration::from_secs(1), connection.next_event())
                .await
                .unwrap(),
            RuntimeEvent::Command(CommandResult::CounselSent {
                request: CounselRequest(8),
                ..
            })
        ));
        assert!(
            timeout(Duration::from_millis(50), listener.accept())
                .await
                .is_err()
        );
        assert!(
            timeout(Duration::from_millis(50), connection.next_event())
                .await
                .is_err()
        );
        connection.shutdown().await.unwrap();
    }

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
            supervisor: ConnectionSupervisor::new(
                HerdrClient::new(PathBuf::from("missing.sock")),
                Backoff::default(),
            ),
            executor: CommandExecutor::new(HerdrClient::new(PathBuf::from("missing.sock")), None),
            update_rx,
            updates_open: true,
            shutdown_tx,
            supervisor_task: Some(supervisor_task),
            command_tasks: JoinSet::new(),
            output_tasks: JoinSet::new(),
            snapshot_tasks: JoinSet::new(),
            pending_snapshot: None,
            pending_output: None,
            last_sidebar_projection: None,
            urgency_view_set: false,
        };
        first_sent_rx.await.unwrap();
        tokio::task::yield_now().await;

        timeout(Duration::from_millis(100), connection.shutdown())
            .await
            .expect("shutdown hung behind the saturated supervisor update channel")
            .unwrap();
    }

    #[tokio::test]
    async fn shutdown_clears_last_published_marginalia() {
        let directory = tempfile::tempdir().unwrap();
        let socket_path = directory.path().join("herdr.sock");
        let listener = UnixListener::bind(&socket_path).unwrap();
        let server = tokio::spawn(async move {
            for expected_method in ["pane.report_metadata", "workspace.report_metadata"] {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut line = String::new();
                BufReader::new(&mut stream)
                    .read_line(&mut line)
                    .await
                    .unwrap();
                let request: serde_json::Value = serde_json::from_str(&line).unwrap();
                assert_eq!(request["method"], expected_method);
                assert!(
                    request["params"]["tokens"]
                        .as_object()
                        .unwrap()
                        .values()
                        .all(serde_json::Value::is_null)
                );
                let response = serde_json::json!({"id": request["id"], "result": {"type": "ok"}});
                stream
                    .write_all(serde_json::to_string(&response).unwrap().as_bytes())
                    .await
                    .unwrap();
                stream.write_all(b"\n").await.unwrap();
            }
        });
        let (_update_tx, update_rx) = mpsc::channel(1);
        let (shutdown_tx, _shutdown_rx) = watch::channel(false);
        let connection = RuntimeConnection {
            supervisor: ConnectionSupervisor::new(
                HerdrClient::new(&socket_path),
                Backoff::default(),
            ),
            executor: CommandExecutor::new(HerdrClient::new(socket_path), None),
            update_rx,
            updates_open: false,
            shutdown_tx,
            supervisor_task: Some(tokio::spawn(async {})),
            command_tasks: JoinSet::new(),
            output_tasks: JoinSet::new(),
            snapshot_tasks: JoinSet::new(),
            pending_snapshot: None,
            pending_output: None,
            urgency_view_set: false,
            last_sidebar_projection: Some(SidebarProjection {
                agents: vec![SidebarAgentTokens {
                    pane_id: PaneId::new("w1:p1"),
                    tokens: [
                        (QUEST_ROLE.into(), "Gnome Paladin".into()),
                        (QUEST_OMEN.into(), "seeks counsel".into()),
                    ]
                    .into_iter()
                    .collect(),
                }],
                campaigns: vec![SidebarCampaignTokens {
                    workspace_id: WorkspaceId::new("w1"),
                    tokens: [(QUEST_CAMPAIGN.into(), "1 adventurer · 1 summons".into())]
                        .into_iter()
                        .collect(),
                }],
            }),
        };

        connection.shutdown().await.unwrap();
        server.await.unwrap();
    }
}
