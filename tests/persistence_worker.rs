use std::{task::Poll, time::Duration};

use futures_util::{StreamExt, stream::FuturesUnordered};
use questmancer::{
    app::{Model, Motion, View},
    command::DeskCommand,
    config::PersistencePaths,
    domain::{AgentPersona, EventId, GuestbookEntry, GuestbookEvent, PersonaKey, Timestamp},
    herdr::environment::HerdrEnvironment,
    herdr::{
        protocol::{SessionSnapshot, SessionSnapshotResult, SuccessResponse},
        supervisor::ConnectionUpdate,
    },
    interaction::reduce_action,
    persistence::{
        PersistenceWorker, WorkerPaths, load_guestbook, load_startup, load_state, publish_state,
    },
    runtime_loop::{
        RuntimeExit, apply_connection_update, dispatch_action_effects, dispatch_persistence_effects,
    },
    terminal::{RuntimeLifecycle, shutdown_persistence},
    ui::input::Action,
    update::Command,
};

fn fixed_persisted_state(view: View) -> questmancer::persistence::PersistedStateV1 {
    questmancer::persistence::PersistedStateV1::capture(&questmancer::app::Model::new(view))
}

fn snapshot() -> SessionSnapshot {
    let response: SuccessResponse<SessionSnapshotResult> =
        serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
    response.result.snapshot
}

async fn wait_for_path(path: &std::path::Path) {
    let path = path.to_owned();
    tokio::task::spawn_blocking(move || {
        let deadline = std::time::Instant::now() + Duration::from_secs(1);
        while std::time::Instant::now() < deadline {
            if path.exists() {
                return;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        panic!("timed out waiting for {}", path.display());
    })
    .await
    .unwrap();
}

async fn assert_path_remains_absent(path: &std::path::Path) {
    let path = path.to_owned();
    tokio::task::spawn_blocking(move || {
        let deadline = std::time::Instant::now() + Duration::from_secs(1);
        while std::time::Instant::now() < deadline {
            assert!(!path.exists(), "{} was published too early", path.display());
            std::thread::sleep(Duration::from_millis(1));
        }
    })
    .await
    .unwrap();
}

fn guestbook_entry(id: &str) -> GuestbookEntry {
    GuestbookEntry {
        id: EventId::new(id),
        occurred_at: Timestamp::from_millis(1_000),
        agent: None,
        workspace: None,
        pane: None,
        pane_revision: 7,
        kind: GuestbookEvent::WorkCompleted,
        summary: format!("entry {id}"),
    }
}

#[tokio::test(start_paused = true)]
async fn runtime_dispatch_durably_appends_before_staging_the_following_state() {
    let directory = tempfile::tempdir().unwrap();
    let state_path = directory.path().join("state.json");
    let guestbook_path = directory.path().join("guestbook.jsonl");
    let (mut client, _diagnostics, worker) = PersistenceWorker::start(WorkerPaths::new(
        Some(state_path.clone()),
        Some(guestbook_path.clone()),
    ));
    let entry = guestbook_entry("runtime-order");
    let model = Model::new(View::Delve);

    let errors = dispatch_persistence_effects(
        &mut client,
        &model,
        vec![
            Command::AppendGuestbook(entry.clone()),
            Command::PersistState,
        ],
    )
    .await;

    assert!(errors.is_empty());
    let replay = load_guestbook(&guestbook_path, 10).await;
    assert_eq!(
        replay.guestbook.entries().iter().collect::<Vec<_>>(),
        vec![&entry]
    );
    client.shutdown().await.unwrap();
    worker.await.unwrap();
    assert_eq!(
        load_state(&state_path).await.unwrap(),
        Some(fixed_persisted_state(View::Delve))
    );
}

#[tokio::test(start_paused = true)]
async fn bounded_runtime_shutdown_flushes_latest_state_after_prior_append() {
    let directory = tempfile::tempdir().unwrap();
    let state_path = directory.path().join("state.json");
    let guestbook_path = directory.path().join("guestbook.jsonl");
    let (mut client, _diagnostics, worker) = PersistenceWorker::start(WorkerPaths::new(
        Some(state_path.clone()),
        Some(guestbook_path.clone()),
    ));
    let entry = guestbook_entry("before-runtime-shutdown");
    let mut model = Model::new(View::Guild);

    assert!(
        dispatch_persistence_effects(
            &mut client,
            &model,
            vec![
                Command::AppendGuestbook(entry.clone()),
                Command::PersistState,
            ],
        )
        .await
        .is_empty()
    );
    model.switch_to(View::Delve);
    assert!(
        dispatch_persistence_effects(&mut client, &model, [Command::PersistState])
            .await
            .is_empty()
    );

    shutdown_persistence(&client, worker).await.unwrap();

    assert_eq!(
        load_state(&state_path).await.unwrap(),
        Some(fixed_persisted_state(View::Delve))
    );
    let replay = load_guestbook(&guestbook_path, 10).await;
    assert_eq!(
        replay.guestbook.entries().iter().collect::<Vec<_>>(),
        vec![&entry]
    );
}

#[tokio::test(start_paused = true)]
async fn bounded_runtime_shutdown_returns_filesystem_failure_after_worker_exit() {
    let directory = tempfile::tempdir().unwrap();
    let blocked_parent = directory.path().join("not-a-directory");
    std::fs::write(&blocked_parent, b"blocking file").unwrap();
    let (mut client, mut diagnostics, worker) = PersistenceWorker::start(WorkerPaths::new(
        Some(blocked_parent.join("state.json")),
        None,
    ));
    client
        .stage_state(fixed_persisted_state(View::Delve))
        .unwrap();

    let error = shutdown_persistence(&client, worker).await.unwrap_err();

    assert!(error.to_string().contains("shut down persistence writer"));
    assert_eq!(
        diagnostics.recv().await.unwrap().operation,
        "create state directory"
    );
}

#[tokio::test]
async fn malformed_state_survives_initial_snapshot_flush_while_guestbook_stays_writable() {
    let directory = tempfile::tempdir().unwrap();
    let state_path = directory.path().join("state.json");
    let guestbook_path = directory.path().join("guestbook.jsonl");
    let malformed = b"{not valid state json}";
    tokio::fs::write(&state_path, malformed).await.unwrap();
    let startup = load_startup(
        PersistencePaths::from_lookup(|name| {
            (name == "HERDR_PLUGIN_STATE_DIR").then(|| directory.path().display().to_string())
        }),
        None,
    )
    .await;
    assert_eq!(startup.diagnostics.len(), 1);
    assert_eq!(startup.diagnostics[0].operation, "parse state");

    let mut model = startup.model;
    let snapshot_effects = apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(snapshot()),
        Timestamp::from_millis(2_000),
    );
    assert!(
        snapshot_effects
            .persistence
            .contains(&Command::PersistState)
    );
    let entry = guestbook_entry("protected-state");
    let (mut client, _diagnostics, worker) = PersistenceWorker::start(startup.paths);

    assert!(
        dispatch_persistence_effects(&mut client, &model, snapshot_effects.persistence)
            .await
            .is_empty()
    );
    client.append_guestbook(entry.clone()).await.unwrap();
    shutdown_persistence(&client, worker).await.unwrap();

    assert_eq!(tokio::fs::read(&state_path).await.unwrap(), malformed);
    let replay = load_guestbook(&guestbook_path, 10).await;
    assert!(replay.diagnostics.is_empty());
    assert_eq!(
        replay.guestbook.entries().iter().collect::<Vec<_>>(),
        vec![&entry]
    );
}

#[tokio::test(start_paused = true)]
async fn offline_view_change_preserves_restored_selection_for_reconnect() {
    let directory = tempfile::tempdir().unwrap();
    let state_path = directory.path().join("state.json");
    let selected = PersonaKey::new("remembered-agent");
    let mut restored = fixed_persisted_state(View::Guild);
    restored.personas.insert(
        selected.clone(),
        AgentPersona {
            appearance: AgentPersona::appearance_for_key(&selected),
            key: selected.clone(),
            handle: "remembered".to_owned(),
        },
    );
    restored.selected_persona = Some(selected.clone());
    publish_state(&state_path, &restored).await.unwrap();
    let startup = load_startup(
        PersistencePaths::from_lookup(|name| {
            (name == "HERDR_PLUGIN_STATE_DIR").then(|| directory.path().display().to_string())
        }),
        None,
    )
    .await;
    let mut model = startup.model;
    let (mut client, _diagnostics, worker) = PersistenceWorker::start(startup.paths);

    let reduction = reduce_action(&mut model, Action::Switch(View::Delve));
    assert!(
        dispatch_persistence_effects(&mut client, &model, reduction.persistence)
            .await
            .is_empty()
    );
    shutdown_persistence(&client, worker).await.unwrap();

    let published = load_state(&state_path).await.unwrap().unwrap();
    assert_eq!(published.last_view, View::Delve);
    assert_eq!(published.selected_persona, Some(selected));
}

#[tokio::test(start_paused = true)]
async fn quit_lifecycle_stops_real_runtime_then_flushes_writer() {
    let directory = tempfile::tempdir().unwrap();
    let state_path = directory.path().join("state.json");
    let environment =
        HerdrEnvironment::new(directory.path().join("missing.sock"), "/usr/bin/herdr");
    let (mut lifecycle, _diagnostics) = RuntimeLifecycle::start(
        Some(&environment),
        WorkerPaths::new(Some(state_path.clone()), None),
    );
    lifecycle
        .connection_mut()
        .unwrap()
        .schedule([DeskCommand::RefreshSnapshot]);
    let mut model = Model::new(View::Guild);

    let reduction = reduce_action(&mut model, Action::Switch(View::Delve));
    let switched = dispatch_action_effects(lifecycle.persistence_mut(), &model, reduction).await;
    assert!(switched.persistence_errors.is_empty());
    let reduction = reduce_action(&mut model, Action::Quit);
    let quitting = dispatch_action_effects(lifecycle.persistence_mut(), &model, reduction).await;

    assert_eq!(
        lifecycle.complete(quitting.exit).await.unwrap(),
        Some(RuntimeExit::Quit)
    );
    assert_eq!(
        load_state(&state_path).await.unwrap(),
        Some(fixed_persisted_state(View::Delve))
    );
}

#[tokio::test(start_paused = true)]
async fn coalesces_state_to_the_latest_value_after_250_milliseconds() {
    let directory = tempfile::tempdir().unwrap();
    let state_path = directory.path().join("state.json");
    let paths = WorkerPaths::new(
        Some(state_path.clone()),
        Some(directory.path().join("guestbook.jsonl")),
    );
    let (mut client, _diagnostics, worker) = PersistenceWorker::start(paths);
    tokio::task::yield_now().await;
    let first = fixed_persisted_state(View::Guild);
    let mut middle = first.clone();
    middle.preferences.motion = Motion::Reduced;
    let latest = fixed_persisted_state(View::Delve);

    assert!(client.stage_state(first).unwrap());
    assert!(client.stage_state(middle).unwrap());
    assert!(client.stage_state(latest.clone()).unwrap());
    tokio::task::yield_now().await;

    tokio::time::advance(Duration::from_millis(249)).await;
    tokio::task::yield_now().await;
    assert!(!state_path.exists());

    tokio::time::advance(Duration::from_millis(1)).await;
    wait_for_path(&state_path).await;
    assert_eq!(load_state(&state_path).await.unwrap(), Some(latest));

    client.shutdown().await.unwrap();
    worker.await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn a_later_distinct_state_resets_the_debounce_deadline() {
    let directory = tempfile::tempdir().unwrap();
    let state_path = directory.path().join("state.json");
    let (mut client, _diagnostics, worker) =
        PersistenceWorker::start(WorkerPaths::new(Some(state_path.clone()), None));
    tokio::task::yield_now().await;

    assert!(
        client
            .stage_state(fixed_persisted_state(View::Guild))
            .unwrap()
    );
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_millis(249)).await;
    assert!(!state_path.exists());

    let latest = fixed_persisted_state(View::Delve);
    assert!(client.stage_state(latest.clone()).unwrap());
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_millis(249)).await;
    assert_path_remains_absent(&state_path).await;

    tokio::time::advance(Duration::from_millis(1)).await;
    wait_for_path(&state_path).await;
    assert_eq!(load_state(&state_path).await.unwrap(), Some(latest));

    client.shutdown().await.unwrap();
    worker.await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn unchanged_state_is_not_staged_or_republished() {
    let directory = tempfile::tempdir().unwrap();
    let state_path = directory.path().join("state.json");
    let paths = WorkerPaths::new(Some(state_path.clone()), None);
    let (mut client, _diagnostics, worker) = PersistenceWorker::start(paths);
    tokio::task::yield_now().await;
    let state = fixed_persisted_state(View::Guild);

    assert!(client.stage_state(state.clone()).unwrap());
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_millis(250)).await;
    wait_for_path(&state_path).await;
    let published_metadata = tokio::fs::metadata(&state_path).await.unwrap();

    assert!(!client.stage_state(state).unwrap());
    tokio::time::advance(Duration::from_millis(250)).await;
    tokio::task::yield_now().await;
    let unchanged_metadata = tokio::fs::metadata(&state_path).await.unwrap();

    assert_eq!(
        unchanged_metadata.modified().unwrap(),
        published_metadata.modified().unwrap()
    );
    assert_eq!(unchanged_metadata.len(), published_metadata.len());

    client.shutdown().await.unwrap();
    worker.await.unwrap();
}

#[tokio::test]
async fn flush_publishes_dirty_state_without_waiting_for_the_debounce() {
    let directory = tempfile::tempdir().unwrap();
    let state_path = directory.path().join("state.json");
    let (mut client, _diagnostics, worker) =
        PersistenceWorker::start(WorkerPaths::new(Some(state_path.clone()), None));
    let state = fixed_persisted_state(View::Delve);

    assert!(client.stage_state(state.clone()).unwrap());
    assert!(!state_path.exists());

    client.flush().await.unwrap();

    assert_eq!(load_state(&state_path).await.unwrap(), Some(state));
    client.shutdown().await.unwrap();
    worker.await.unwrap();
}

#[tokio::test]
async fn shutdown_publishes_dirty_state_and_exits() {
    let directory = tempfile::tempdir().unwrap();
    let state_path = directory.path().join("state.json");
    let (mut client, _diagnostics, worker) =
        PersistenceWorker::start(WorkerPaths::new(Some(state_path.clone()), None));
    let state = fixed_persisted_state(View::Guild);

    assert!(client.stage_state(state.clone()).unwrap());

    client.shutdown().await.unwrap();
    worker.await.unwrap();

    assert_eq!(load_state(&state_path).await.unwrap(), Some(state));
}

#[tokio::test]
async fn disabled_paths_are_successful_no_ops() {
    let (mut client, mut diagnostics, worker) =
        PersistenceWorker::start(WorkerPaths::new(None, None));

    assert!(
        client
            .stage_state(fixed_persisted_state(View::Delve))
            .unwrap()
    );
    client
        .append_guestbook(guestbook_entry("disabled"))
        .await
        .unwrap();
    client.flush().await.unwrap();
    client.shutdown().await.unwrap();
    worker.await.unwrap();

    assert!(diagnostics.try_recv().is_err());
}

#[tokio::test]
async fn guestbook_append_is_acknowledged_after_the_record_is_durable() {
    let directory = tempfile::tempdir().unwrap();
    let guestbook_path = directory.path().join("guestbook.jsonl");
    let (client, _diagnostics, worker) =
        PersistenceWorker::start(WorkerPaths::new(None, Some(guestbook_path.clone())));
    let entry = guestbook_entry("acknowledged");

    client.append_guestbook(entry.clone()).await.unwrap();

    let replay = load_guestbook(&guestbook_path, 10).await;
    assert!(replay.diagnostics.is_empty());
    assert_eq!(
        replay.guestbook.entries().iter().collect::<Vec<_>>(),
        vec![&entry]
    );
    client.shutdown().await.unwrap();
    worker.await.unwrap();
}

#[tokio::test]
async fn acknowledged_append_survives_restart_after_a_truncated_guestbook_tail() {
    let directory = tempfile::tempdir().unwrap();
    let guestbook_path = directory.path().join("guestbook.jsonl");
    let prior = guestbook_entry("prior-complete");
    let acknowledged = guestbook_entry("acknowledged-after-truncation");
    let mut damaged_bytes = serde_json::to_vec(&prior).unwrap();
    damaged_bytes.extend_from_slice(b"\n{\"id\":");
    tokio::fs::write(&guestbook_path, damaged_bytes)
        .await
        .unwrap();
    let (client, _diagnostics, worker) =
        PersistenceWorker::start(WorkerPaths::new(None, Some(guestbook_path.clone())));

    client.append_guestbook(acknowledged.clone()).await.unwrap();
    client.shutdown().await.unwrap();
    worker.await.unwrap();

    let replay = load_guestbook(&guestbook_path, 10).await;
    assert_eq!(
        replay.guestbook.entries().iter().collect::<Vec<_>>(),
        vec![&prior, &acknowledged]
    );
    assert_eq!(replay.diagnostics.len(), 1);
    assert_eq!(replay.diagnostics[0].line, Some(2));
}

#[tokio::test(start_paused = true)]
async fn an_expired_state_deadline_precedes_sustained_queued_appends() {
    let directory = tempfile::tempdir().unwrap();
    let state_path = directory.path().join("state.json");
    let guestbook_path = directory.path().join("guestbook.jsonl");
    let (mut client, _diagnostics, worker) = PersistenceWorker::start(WorkerPaths::new(
        Some(state_path.clone()),
        Some(guestbook_path),
    ));
    tokio::task::yield_now().await;
    let state = fixed_persisted_state(View::Delve);
    assert!(client.stage_state(state.clone()).unwrap());
    tokio::task::yield_now().await;

    let mut appends = FuturesUnordered::new();
    for index in 0..64 {
        appends.push(client.append_guestbook(guestbook_entry(&format!("queued-{index}"))));
    }
    assert!(matches!(futures_util::poll!(appends.next()), Poll::Pending));

    tokio::time::advance(Duration::from_millis(250)).await;
    appends.next().await.unwrap().unwrap();

    assert_eq!(load_state(&state_path).await.unwrap(), Some(state));
    while let Some(result) = appends.next().await {
        result.unwrap();
    }

    client.shutdown().await.unwrap();
    worker.await.unwrap();
}

#[tokio::test]
async fn flush_acknowledges_after_an_earlier_append_and_dirty_state_are_durable() {
    let directory = tempfile::tempdir().unwrap();
    let state_path = directory.path().join("state.json");
    let guestbook_path = directory.path().join("guestbook.jsonl");
    let (mut client, _diagnostics, worker) = PersistenceWorker::start(WorkerPaths::new(
        Some(state_path.clone()),
        Some(guestbook_path.clone()),
    ));
    let state = fixed_persisted_state(View::Guild);
    let entry = guestbook_entry("before-flush");
    assert!(client.stage_state(state.clone()).unwrap());

    let mut append = Box::pin(client.append_guestbook(entry.clone()));
    assert!(matches!(futures_util::poll!(&mut append), Poll::Pending));

    client.flush().await.unwrap();

    assert_eq!(load_state(&state_path).await.unwrap(), Some(state));
    let replay = load_guestbook(&guestbook_path, 10).await;
    assert!(replay.diagnostics.is_empty());
    assert_eq!(
        replay.guestbook.entries().iter().collect::<Vec<_>>(),
        vec![&entry]
    );

    append.await.unwrap();

    client.shutdown().await.unwrap();
    worker.await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn failed_state_write_is_non_fatal_and_only_a_distinct_state_retries() {
    let directory = tempfile::tempdir().unwrap();
    let blocked_parent = directory.path().join("not-a-directory");
    std::fs::write(&blocked_parent, b"blocking file").unwrap();
    let state_path = blocked_parent.join("state.json");
    let (mut client, mut diagnostics, worker) =
        PersistenceWorker::start(WorkerPaths::new(Some(state_path), None));
    tokio::task::yield_now().await;
    let first = fixed_persisted_state(View::Guild);

    assert!(client.stage_state(first.clone()).unwrap());
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_millis(250)).await;
    let first_diagnostic = diagnostics.recv().await.unwrap();
    assert_eq!(first_diagnostic.operation, "create state directory");
    assert_eq!(first_diagnostic.path, blocked_parent);
    assert!(!first_diagnostic.source_message.is_empty());

    assert!(!client.stage_state(first).unwrap());
    client.flush().await.unwrap();
    assert!(diagnostics.try_recv().is_err());

    assert!(
        client
            .stage_state(fixed_persisted_state(View::Delve))
            .unwrap()
    );
    let second_error = client.flush().await.unwrap_err();
    assert_eq!(second_error.operation, "create state directory");
    let second_diagnostic = diagnostics.recv().await.unwrap();
    assert_eq!(second_diagnostic.operation, second_error.operation);
    assert_eq!(second_diagnostic.path, second_error.path);
    assert_eq!(
        second_diagnostic.source_message,
        second_error.source_message
    );

    client.shutdown().await.unwrap();
    worker.await.unwrap();
}

#[tokio::test]
async fn diagnostic_queue_remains_bounded_under_repeated_failures() {
    let directory = tempfile::tempdir().unwrap();
    let blocked_parent = directory.path().join("not-a-directory");
    std::fs::write(&blocked_parent, b"blocking file").unwrap();
    let (mut client, diagnostics, worker) = PersistenceWorker::start(WorkerPaths::new(
        Some(blocked_parent.join("state.json")),
        None,
    ));
    let capacity = diagnostics.max_capacity();

    for schema_version in 2..u32::try_from(capacity + 10).unwrap() {
        let mut state = fixed_persisted_state(View::Guild);
        state.schema_version = schema_version;
        assert!(client.stage_state(state).unwrap());
        assert!(client.flush().await.is_err());
    }

    assert_eq!(diagnostics.len(), capacity);
    client.shutdown().await.unwrap();
    worker.await.unwrap();
}

#[tokio::test]
async fn saturated_diagnostic_queue_retains_the_latest_failure() {
    let directory = tempfile::tempdir().unwrap();
    let blocked_parent = directory.path().join("not-a-directory");
    std::fs::write(&blocked_parent, b"blocking file").unwrap();
    let guestbook_path = directory.path().join("guestbook-is-a-directory");
    std::fs::create_dir(&guestbook_path).unwrap();
    let (mut client, mut diagnostics, worker) = PersistenceWorker::start(WorkerPaths::new(
        Some(blocked_parent.join("state.json")),
        Some(guestbook_path),
    ));
    let capacity = diagnostics.max_capacity();

    for schema_version in 2..u32::try_from(capacity + 2).unwrap() {
        let mut state = fixed_persisted_state(View::Guild);
        state.schema_version = schema_version;
        assert!(client.stage_state(state).unwrap());
        assert!(client.flush().await.is_err());
    }
    let latest_error = client
        .append_guestbook(guestbook_entry("latest-diagnostic"))
        .await
        .unwrap_err();
    assert_eq!(latest_error.operation, "open guestbook");

    let mut retained = Vec::new();
    while let Ok(diagnostic) = diagnostics.try_recv() {
        retained.push(diagnostic);
    }
    assert_eq!(retained.len(), capacity);
    assert_eq!(retained.last().unwrap().operation, latest_error.operation,);

    client.shutdown().await.unwrap();
    worker.await.unwrap();
}
