use std::{task::Poll, time::Duration};

use futures_util::{StreamExt, stream::FuturesUnordered};
use herdr_webmaster::{
    app::{Motion, View},
    domain::{EventId, GuestbookEntry, GuestbookEvent, Timestamp},
    persistence::{PersistenceWorker, WorkerPaths, load_guestbook, load_state},
};

fn fixed_persisted_state(view: View) -> herdr_webmaster::persistence::PersistedStateV1 {
    herdr_webmaster::persistence::PersistedStateV1::capture(&herdr_webmaster::app::Model::new(view))
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
async fn coalesces_state_to_the_latest_value_after_250_milliseconds() {
    let directory = tempfile::tempdir().unwrap();
    let state_path = directory.path().join("state.json");
    let paths = WorkerPaths::new(
        Some(state_path.clone()),
        Some(directory.path().join("guestbook.jsonl")),
    );
    let (mut client, _diagnostics, worker) = PersistenceWorker::start(paths);
    tokio::task::yield_now().await;
    let first = fixed_persisted_state(View::Desk);
    let mut middle = first.clone();
    middle.preferences.motion = Motion::Reduced;
    let latest = fixed_persisted_state(View::Cafe);

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
            .stage_state(fixed_persisted_state(View::Desk))
            .unwrap()
    );
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_millis(249)).await;
    assert!(!state_path.exists());

    let latest = fixed_persisted_state(View::Cafe);
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
    let state = fixed_persisted_state(View::Desk);

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
    let state = fixed_persisted_state(View::Cafe);

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
    let state = fixed_persisted_state(View::Desk);

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
            .stage_state(fixed_persisted_state(View::Cafe))
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
    let state = fixed_persisted_state(View::Cafe);
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
    let state = fixed_persisted_state(View::Desk);
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
    let first = fixed_persisted_state(View::Desk);

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
            .stage_state(fixed_persisted_state(View::Cafe))
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
        let mut state = fixed_persisted_state(View::Desk);
        state.schema_version = schema_version;
        assert!(client.stage_state(state).unwrap());
        assert!(client.flush().await.is_err());
    }

    assert_eq!(diagnostics.len(), capacity);
    client.shutdown().await.unwrap();
    worker.await.unwrap();
}
