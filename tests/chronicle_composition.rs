//! Socket-to-JSONL qualification. The real connection task owns both read paths;
//! only unrelated output/sidebar/review commands are omitted by this harness.
use questmancer::{
    app::{Model, View},
    command::{AgentCommand, CommandResult},
    config::PersistencePaths,
    domain::{ChronicleEvent, ObservationEvidence, Presence, Timestamp},
    herdr::{environment::HerdrEnvironment, supervisor::ConnectionUpdate},
    persistence::{
        PersistenceClient, PersistenceWorker, WorkerPaths, load_startup, replay_chronicle,
    },
    runtime_loop::{
        RuntimeConnection, RuntimeEffects, RuntimeEvent, apply_command_result,
        apply_connection_update, bootstrap_model, dispatch_persistence_effects,
        request_snapshot_refresh,
    },
};
use serde_json::{Value, json};
use std::{path::PathBuf, time::Duration};
use tempfile::TempDir;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
    time::timeout,
};

fn snapshot(status: &str, revision: u64) -> Value {
    let mut value: Value =
        serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
    let snap = &mut value["result"]["snapshot"];
    for list in ["agents", "panes"] {
        snap[list][0]["agent_status"] = json!(status);
        snap[list][0]["revision"] = json!(revision);
        snap[list][0]["custom_status"] = Value::Null;
        snap[list][0]["agent_session"] =
            json!({"source":"codex","agent":"codex","kind":"id","value":"session-123"});
    }
    value
}

async fn send(stream: &mut UnixStream, value: &Value) {
    stream
        .write_all(format!("{value}\n").as_bytes())
        .await
        .unwrap();
}

struct Request {
    stream: UnixStream,
    id: Value,
}
impl Request {
    async fn reply(mut self, mut response: Value) {
        response["id"] = self.id;
        send(&mut self.stream, &response).await;
    }
}

struct Harness {
    directory: TempDir,
    listener: UnixListener,
    connection: RuntimeConnection,
    model: Model,
    persistence: PersistenceClient,
    worker: tokio::task::JoinHandle<()>,
    now: i64,
}
impl Harness {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let socket = directory.path().join("herdr.sock");
        let listener = UnixListener::bind(&socket).unwrap();
        let environment = HerdrEnvironment::new(socket, "/usr/bin/herdr");
        let connection = RuntimeConnection::start(&environment);
        let (persistence, _, worker) = PersistenceWorker::start(WorkerPaths::new(
            Some(directory.path().join("state.json")),
            Some(directory.path().join("chronicle.jsonl")),
        ));
        Self {
            directory,
            listener,
            connection,
            model: bootstrap_model(Model::new(View::Guild), Some(&environment)),
            persistence,
            worker,
            now: 1_000,
        }
    }
    async fn request(&self, expected: &str) -> Request {
        let (mut stream, _) = timeout(Duration::from_secs(2), self.listener.accept())
            .await
            .expect("request timeout")
            .unwrap();
        let mut line = String::new();
        BufReader::new(&mut stream)
            .read_line(&mut line)
            .await
            .unwrap();
        let request: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(request["method"], expected);
        Request {
            stream,
            id: request["id"].clone(),
        }
    }
    async fn baseline(&mut self, state: &str, revision: u64) -> UnixStream {
        self.request("ping")
            .await
            .reply(serde_json::from_str(include_str!("fixtures/herdr/pong.json")).unwrap())
            .await;
        self.request("session.snapshot")
            .await
            .reply(snapshot("working", 1))
            .await;
        let mut subscription = self.request("events.subscribe").await;
        let id = subscription.id.clone();
        send(
            &mut subscription.stream,
            &json!({"id":id,"result":{"type":"subscription_started"}}),
        )
        .await;
        self.request("session.snapshot")
            .await
            .reply(snapshot(state, revision))
            .await;
        assert!(matches!(
            self.step().await,
            RuntimeEvent::Connection(ConnectionUpdate::Connected(_))
        ));
        subscription.stream
    }
    async fn effects(
        &mut self,
        effects: RuntimeEffects,
    ) -> Vec<questmancer::persistence::PersistenceError> {
        if effects.resubscribe {
            self.connection.resubscribe();
        }
        self.connection.schedule(
            effects
                .agent_commands
                .into_iter()
                .filter(|cmd| matches!(cmd, AgentCommand::RefreshSnapshot(_))),
        );
        dispatch_persistence_effects(&mut self.persistence, &self.model, effects.persistence).await
    }
    async fn step(&mut self) -> RuntimeEvent {
        let event = timeout(Duration::from_secs(2), self.connection.next_event())
            .await
            .expect("runtime event timeout");
        self.now += 1_000;
        let at = Timestamp::from_millis(self.now);
        let effects = match &event {
            RuntimeEvent::Connection(update) => {
                apply_connection_update(&mut self.model, update.clone(), at)
            }
            RuntimeEvent::Command(result) => {
                apply_command_result(&mut self.model, result.clone(), at)
            }
            RuntimeEvent::CommandTaskFailed(error) => panic!("{error}"),
        };
        assert!(self.effects(effects).await.is_empty());
        event
    }
    async fn refresh(&mut self, state: &str, revision: u64) {
        let effects = request_snapshot_refresh(&mut self.model);
        assert!(self.effects(effects).await.is_empty());
        self.request("session.snapshot")
            .await
            .reply(snapshot(state, revision))
            .await;
        assert!(matches!(
            self.step().await,
            RuntimeEvent::Command(CommandResult::SnapshotLoaded { .. })
        ));
    }
    async fn metadata(&mut self, subscription: &mut UnixStream, state: &str, revision: u64) {
        send(
            subscription,
            &json!({"event":"pane.agent_status_changed","data":{
            "pane_id":"w1:p1","workspace_id":"w1","terminal_id":"terminal-1",
            "agent_session":{"source":"codex","agent":"codex","kind":"id","value":"session-123"},
            "agent_status":state,"revision":revision}}),
        )
        .await;
        assert!(matches!(
            self.step().await,
            RuntimeEvent::Connection(ConnectionUpdate::Event(_))
        ));
    }
    fn history_path(&self) -> PathBuf {
        self.directory.path().join("chronicle.jsonl")
    }
    async fn persisted(&self) -> Vec<questmancer::domain::ChronicleEntry> {
        self.persistence.flush().await.unwrap();
        let bytes = tokio::fs::read(self.history_path())
            .await
            .unwrap_or_default();
        let replay = replay_chronicle(&self.history_path(), &bytes, 500);
        assert!(replay.diagnostics.is_empty());
        let entries = replay
            .chronicle
            .entries()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        for entry in &entries {
            assert!(entry.observation().is_some());
            let json = serde_json::to_value(entry).unwrap();
            assert_eq!(json["record_version"], 2);
            assert_eq!(
                serde_json::from_value::<questmancer::domain::ChronicleEntry>(json).unwrap(),
                *entry
            );
        }
        entries
    }
    async fn stop(self) {
        self.connection.shutdown().await.unwrap();
        self.persistence.shutdown().await.unwrap();
        self.worker.await.unwrap();
    }
}

#[tokio::test]
async fn supervisor_probe_cannot_capture_and_command_refresh_persists_qualified_evidence() {
    let mut h = Harness::new();
    let mut subscription = h.baseline("working", 7).await;
    assert!(h.persisted().await.is_empty());
    send(
        &mut subscription,
        &json!({"event":"pane_agent_detected","data":{"pane_id":"w1:p1"}}),
    )
    .await;
    // Supervisor starts its subscription-membership probe before runtime receives
    // the hint. Hold it, then let runtime start the separate qualified read.
    let probe = h.request("session.snapshot").await;
    h.step().await;
    let qualified = h.request("session.snapshot").await;
    probe.reply(snapshot("done", 100)).await;
    qualified.reply(snapshot("blocked", 8)).await;
    h.step().await;
    assert_eq!(
        h.model.selected_agent().unwrap().presence,
        Presence::Blocked
    );
    let records = h.persisted().await;
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].event(), ChronicleEvent::PresenceObserved);
    assert!(matches!(
        records[0].observation().unwrap().evidence,
        ObservationEvidence::Snapshot { .. }
    ));
    assert_eq!(h.model.experience(), 0);
    // Buffered unversioned completion text is replaced with current pane facts.
    send(&mut subscription, &json!({"event":"pane.agent_status_changed","data":{"pane_id":"w1:p1","agent_status":"done"}})).await;
    let pane = snapshot("blocked", 8)["result"]["snapshot"]["panes"][0].clone();
    h.request("pane.get")
        .await
        .reply(json!({"result":{"type":"pane_info","pane":pane}}))
        .await;
    h.step().await;
    assert_eq!(h.persisted().await, records);
    h.metadata(&mut subscription, "working", 9).await;
    h.refresh("done", 10).await;
    h.metadata(&mut subscription, "done", 10).await;
    assert_eq!(h.persisted().await.len(), 3);
    assert_eq!(
        h.model.experience(),
        0,
        "snapshot-first done must not later earn metadata XP"
    );
    let bytes = tokio::fs::read(h.history_path()).await.unwrap();
    h.connection.shutdown().await.unwrap();
    let paths = PersistencePaths::from_lookup(|name| {
        (name == "HERDR_PLUGIN_STATE_DIR").then(|| h.directory.path().display().to_string())
    });
    let startup = load_startup(paths, None).await;
    assert!(startup.diagnostics.is_empty());
    let environment =
        HerdrEnvironment::new(h.directory.path().join("herdr.sock"), "/usr/bin/herdr");
    h.model = bootstrap_model(startup.model, Some(&environment));
    h.connection = RuntimeConnection::start(&environment);
    let mut restarted = h.baseline("done", 10).await;
    h.metadata(&mut restarted, "done", 11).await;
    assert_eq!(tokio::fs::read(h.history_path()).await.unwrap(), bytes);
    assert_eq!(h.model.experience(), 0);
    h.stop().await;
}

#[tokio::test]
async fn overtaken_socket_results_trigger_bounded_resync_without_history_backfill() {
    let mut h = Harness::new();
    let mut subscription = h.baseline("working", 7).await;
    let effects = request_snapshot_refresh(&mut h.model);
    assert!(h.effects(effects).await.is_empty());
    let first = h.request("session.snapshot").await;
    h.metadata(&mut subscription, "blocked", 8).await;
    first.reply(snapshot("done", 9)).await;
    h.step().await;
    let retry = h.request("session.snapshot").await;
    h.metadata(&mut subscription, "idle", 9).await;
    let records = h.persisted().await;
    retry.reply(snapshot("done", 10)).await;
    h.step().await;
    let _replacement = h.baseline("done", 11).await;
    assert_eq!(h.persisted().await, records);
    assert_eq!(records.len(), 2);
    assert!(records.iter().all(|record| matches!(
        record.observation().unwrap().evidence,
        ObservationEvidence::PaneMetadata { .. }
    )));
    assert_eq!(h.model.selected_agent().unwrap().presence, Presence::Done);
    assert_eq!(h.model.experience(), 0);
    h.stop().await;
}

#[tokio::test]
async fn capture_append_failure_returns_the_real_worker_diagnostic() {
    let mut h = Harness::new();
    let _subscription = h.baseline("working", 7).await;
    tokio::fs::create_dir(h.history_path()).await.unwrap();
    let effects = request_snapshot_refresh(&mut h.model);
    assert!(h.effects(effects).await.is_empty());
    h.request("session.snapshot")
        .await
        .reply(snapshot("blocked", 8))
        .await;
    let RuntimeEvent::Command(result) = h.connection.next_event().await else {
        panic!("expected snapshot")
    };
    let effects = apply_command_result(&mut h.model, result, Timestamp::from_millis(5_000));
    let errors = h.effects(effects).await;
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].path, h.history_path());
    assert_eq!(
        h.model.domain().chronicle.entries().len(),
        1,
        "in-memory acceptance is not a durability acknowledgement"
    );
    assert_eq!(h.model.experience(), 0);
    assert!(h.history_path().is_dir());
    h.stop().await;
}

#[tokio::test]
async fn mixed_history_appends_keep_legacy_bytes_and_restart_does_not_reaward_spoils() {
    use questmancer::domain::ChronicleEntry;
    let mut h = Harness::new();
    let legacy = ChronicleEntry::new(
        Timestamp::from_millis(500),
        None,
        None,
        None,
        1,
        ChronicleEvent::SpoilsReturned,
        "Original legacy wording",
    );
    let legacy_bytes = format!(" {} \n", serde_json::to_string(&legacy).unwrap()).into_bytes();
    tokio::fs::write(h.history_path(), &legacy_bytes)
        .await
        .unwrap();
    let paths = || {
        PersistencePaths::from_lookup(|name| {
            (name == "HERDR_PLUGIN_STATE_DIR").then(|| h.directory.path().display().to_string())
        })
    };
    let startup = load_startup(paths(), None).await;
    assert!(startup.diagnostics.is_empty());
    h.model = bootstrap_model(startup.model, None);
    let mut subscription = h.baseline("working", 7).await;
    assert_eq!(
        h.model.experience(),
        0,
        "legacy spoils are history, not a replayed award"
    );
    h.metadata(&mut subscription, "done", 8).await;
    h.refresh("done", 8).await;
    h.persistence.flush().await.unwrap();
    let recorded = tokio::fs::read(h.history_path()).await.unwrap();
    assert!(recorded.starts_with(&legacy_bytes));
    assert_eq!(h.model.experience(), 10);
    assert_eq!(h.model.domain().chronicle.entries().len(), 2);
    h.connection.shutdown().await.unwrap();
    let paths = PersistencePaths::from_lookup(|name| {
        (name == "HERDR_PLUGIN_STATE_DIR").then(|| h.directory.path().display().to_string())
    });
    let startup = load_startup(paths, None).await;
    assert!(startup.diagnostics.is_empty());
    let environment =
        HerdrEnvironment::new(h.directory.path().join("herdr.sock"), "/usr/bin/herdr");
    h.model = bootstrap_model(startup.model, Some(&environment));
    h.connection = RuntimeConnection::start(&environment);
    let mut restarted = h.baseline("done", 9).await;
    h.metadata(&mut restarted, "done", 10).await;
    assert_eq!(h.model.experience(), 10);
    assert_eq!(tokio::fs::read(h.history_path()).await.unwrap(), recorded);
    assert_eq!(h.model.domain().chronicle.entries().front(), Some(&legacy));
    h.stop().await;
}
