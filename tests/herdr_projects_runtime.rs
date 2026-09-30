use questmancer::{
    app::{Model, View},
    persistence::{PersistenceWorker, WorkerPaths, load_work_cache},
    sources::herdr_projects::{Producer, RuntimeSelection},
    work::{Freshness, SourceId},
};
use std::{fs, path::Path, time::Duration};

fn fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir_in(fs::canonicalize(std::env::temp_dir()).unwrap()).unwrap();
    fs::create_dir(dir.path().join(".state")).unwrap();
    fs::create_dir(dir.path().join("threads")).unwrap();
    fs::write(
        dir.path().join("PROJECT.md"),
        "+++\nname = 'Runtime campaign'\ngoal = 'Preserve facts'\n+++\n",
    )
    .unwrap();
    fs::write(
        dir.path().join(".state/project.json"),
        r#"{"status":"active"}"#,
    )
    .unwrap();
    fs::write(dir.path().join(".state/lock"), "").unwrap();
    thread(dir.path(), "Initial work", "2026-09-28T10:00:00Z");
    dir
}
fn thread(path: &Path, title: &str, created: &str) {
    fs::write(path.join("threads/t-0001.toml"),format!("id = 't-0001'\ntitle = '{title}'\nstatus = 'open'\nkind = 'checkout'\ncreated = '{created}'\nlast_group = 'waiting-on-you'\n")).unwrap();
}
fn selection(path: &Path) -> RuntimeSelection {
    RuntimeSelection::new(
        SourceId::new("projects:runtime"),
        "campaign",
        path.to_owned(),
    )
    .unwrap()
}

#[tokio::test(start_paused = true)]
async fn owned_producer_acknowledges_changes_and_restarts_from_atomic_cache() {
    let dir = fixture();
    let cache = tempfile::tempdir().unwrap();
    let paths = WorkerPaths::new(Some(cache.path().join("state.json")), None);
    let (client, _diagnostics, worker) = PersistenceWorker::start(paths.clone());
    let mut model = Model::new(View::Guild);
    let (producer, mut inbox) = Producer::start(selection(dir.path()), model.work(), None).unwrap();
    inbox
        .next_projects()
        .await
        .consume(&client, &mut model)
        .await;
    let source = SourceId::new("projects:runtime");
    assert_eq!(
        model.work().sources.get(&source).map(|s| s.freshness),
        Some(Freshness::Current),
        "{:?}",
        model.work_source_notice()
    );
    assert!(
        model.work().sources[&source]
            .inventory
            .obligations
            .is_empty()
    );
    let first = fs::read(paths.work_path().unwrap()).unwrap();
    assert!(
        !inbox
            .next_projects()
            .await
            .consume(&client, &mut model)
            .await
    );
    assert_eq!(fs::read(paths.work_path().unwrap()).unwrap(), first);
    thread(dir.path(), "Changed work", "2026-09-28T10:00:00Z");
    inbox
        .next_projects()
        .await
        .consume(&client, &mut model)
        .await;
    assert_eq!(
        model.work().sources[&source]
            .inventory
            .quests
            .values()
            .next()
            .unwrap()
            .title,
        "Changed work"
    );
    drop(inbox);
    producer.shutdown().await.unwrap();
    let restored = load_work_cache(&paths.work_path().unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        restored.projection.sources[&source].freshness,
        Freshness::Cached
    );
    model.replace_work(restored.projection.clone());
    let (producer, mut inbox) = Producer::start(
        selection(dir.path()),
        model.work(),
        restored.herdr_projects(&source).cloned(),
    )
    .unwrap();
    inbox
        .next_projects()
        .await
        .consume(&client, &mut model)
        .await;
    assert_eq!(model.work().sources[&source].generation, 2);
    assert_eq!(model.work().sources[&source].freshness, Freshness::Current);
    drop(inbox);
    producer.shutdown().await.unwrap();
    client.shutdown().await.unwrap();
    worker.await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn changed_creation_latches_even_after_the_original_bytes_return() {
    let dir = fixture();
    let cache = tempfile::tempdir().unwrap();
    let (client, _diagnostics, worker) = PersistenceWorker::start(WorkerPaths::new(
        Some(cache.path().join("state.json")),
        None,
    ));
    let mut model = Model::new(View::Guild);
    let (producer, mut inbox) = Producer::start(selection(dir.path()), model.work(), None).unwrap();
    inbox
        .next_projects()
        .await
        .consume(&client, &mut model)
        .await;
    let prior = model.work().sources[&SourceId::new("projects:runtime")]
        .inventory
        .clone();
    thread(dir.path(), "Replacement work", "2026-09-29T10:00:00Z");
    inbox
        .next_projects()
        .await
        .consume(&client, &mut model)
        .await;
    thread(dir.path(), "Initial work", "2026-09-28T10:00:00Z");
    assert!(
        tokio::time::timeout(Duration::from_secs(60), inbox.next_projects())
            .await
            .is_err()
    );
    assert_eq!(
        model.work().sources[&SourceId::new("projects:runtime")].inventory,
        prior
    );
    assert_eq!(
        model.work().sources[&SourceId::new("projects:runtime")].freshness,
        Freshness::Reconcile
    );
    drop(inbox);
    producer.shutdown().await.unwrap();
    client.shutdown().await.unwrap();
    worker.await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn root_replacement_before_first_valid_capture_cannot_auto_attach() {
    let dir = fixture();
    let cache = tempfile::tempdir().unwrap();
    fs::write(dir.path().join(".state/project.json"), "malformed").unwrap();
    let (client, _diagnostics, worker) = PersistenceWorker::start(WorkerPaths::new(
        Some(cache.path().join("state.json")),
        None,
    ));
    let mut model = Model::new(View::Guild);
    let (producer, mut inbox) = Producer::start(selection(dir.path()), model.work(), None).unwrap();
    inbox
        .next_projects()
        .await
        .consume(&client, &mut model)
        .await;
    let old = dir.path().join("old-root");
    let old_parent = tempfile::tempdir_in(dir.path().parent().unwrap()).unwrap();
    fs::rename(dir.path(), old_parent.path().join("old")).unwrap();
    fs::create_dir(dir.path()).unwrap();
    fs::create_dir(dir.path().join(".state")).unwrap();
    fs::create_dir(dir.path().join("threads")).unwrap();
    fs::write(
        dir.path().join("PROJECT.md"),
        "+++\nname = 'Replacement'\n+++\n",
    )
    .unwrap();
    fs::write(
        dir.path().join(".state/project.json"),
        r#"{"status":"active"}"#,
    )
    .unwrap();
    fs::write(dir.path().join(".state/lock"), "").unwrap();
    inbox
        .next_projects()
        .await
        .consume(&client, &mut model)
        .await;
    assert!(
        model.work().sources.is_empty(),
        "replacement must never become a campaign"
    );
    assert!(
        model
            .work_source_notice()
            .unwrap()
            .message
            .contains("rebind")
    );
    assert!(
        tokio::time::timeout(Duration::from_secs(60), inbox.next_projects())
            .await
            .is_err()
    );
    assert!(!old.exists());
    drop(inbox);
    producer.shutdown().await.unwrap();
    client.shutdown().await.unwrap();
    worker.await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn queued_capture_is_checked_against_the_lease_before_publication() {
    let dir = fixture();
    let cache = tempfile::tempdir().unwrap();
    let (client, _diagnostics, worker) = PersistenceWorker::start(WorkerPaths::new(
        Some(cache.path().join("state.json")),
        None,
    ));
    let mut model = Model::new(View::Guild);
    let (producer, mut inbox) = Producer::start(selection(dir.path()), model.work(), None).unwrap();
    let observation = inbox.next_projects().await;
    let old_parent = tempfile::tempdir_in(dir.path().parent().unwrap()).unwrap();
    fs::rename(dir.path(), old_parent.path().join("old")).unwrap();
    fs::create_dir(dir.path()).unwrap();
    observation.consume(&client, &mut model).await;
    assert!(model.work().sources.is_empty());
    assert!(
        model
            .work_source_notice()
            .unwrap()
            .message
            .contains("rebind")
    );
    drop(inbox);
    producer.shutdown().await.unwrap();
    client.shutdown().await.unwrap();
    worker.await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn startup_selection_and_lifecycle_work_without_a_herdr_connection() {
    use questmancer::{
        config::PersistencePaths, persistence::load_startup, terminal::RuntimeLifecycle,
    };
    let dir = fixture();
    let config = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    fs::write(config.path().join("config.toml"),format!("[herdr_projects]\nenabled = true\nsource_id = 'projects:runtime'\ncampaign_id = 'campaign'\nproject_path = '{}'\nformat = 'herdr-projects-0.2.34'\nidentity_policy = 'registered_retained_records'\n",dir.path().display())).unwrap();
    let paths = PersistencePaths::from_lookup(|name| match name {
        "HERDR_PLUGIN_CONFIG_DIR" => Some(config.path().display().to_string()),
        "HERDR_PLUGIN_STATE_DIR" => Some(cache.path().display().to_string()),
        _ => None,
    });
    let startup = load_startup(paths, None).await;
    assert!(startup.diagnostics.is_empty());
    let mut model = startup.model;
    let (mut lifecycle, _diagnostics) = RuntimeLifecycle::start(None, startup.paths);
    lifecycle.start_projects(startup.projects, &startup.work_cache, &mut model);
    let observation = lifecycle
        .projects_inbox_mut()
        .as_mut()
        .unwrap()
        .next_projects()
        .await;
    observation
        .consume(lifecycle.persistence_mut(), &mut model)
        .await;
    assert_eq!(
        model.work().sources[&SourceId::new("projects:runtime")].freshness,
        Freshness::Current
    );
    assert_eq!(model.experience(), 0);
    assert!(model.domain().chronicle.entries().is_empty());
    lifecycle.complete(None).await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn obsolete_leased_capture_cannot_downgrade_a_new_generation() {
    let dir = fixture();
    let cache = tempfile::tempdir().unwrap();
    let paths = WorkerPaths::new(Some(cache.path().join("state.json")), None);
    let (client, _diagnostics, worker) = PersistenceWorker::start(paths.clone());
    let mut model = Model::new(View::Guild);
    let (old, mut old_inbox) = Producer::start(selection(dir.path()), model.work(), None).unwrap();
    old_inbox
        .next_projects()
        .await
        .consume(&client, &mut model)
        .await;
    let pending = old_inbox.next_projects().await;
    let cached = load_work_cache(&paths.work_path().unwrap())
        .await
        .unwrap()
        .unwrap();
    let source = SourceId::new("projects:runtime");
    let (new, mut new_inbox) = Producer::start(
        selection(dir.path()),
        model.work(),
        cached.herdr_projects(&source).cloned(),
    )
    .unwrap();
    new_inbox
        .next_projects()
        .await
        .consume(&client, &mut model)
        .await;
    let accepted = model.work().clone();
    let old_root = tempfile::tempdir_in(dir.path().parent().unwrap()).unwrap();
    fs::rename(dir.path(), old_root.path().join("old")).unwrap();
    fs::create_dir(dir.path()).unwrap();
    pending.consume(&client, &mut model).await;
    assert_eq!(model.work(), &accepted);
    assert!(model.work_source_notice().is_none());
    drop(old_inbox);
    drop(new_inbox);
    old.shutdown().await.unwrap();
    new.shutdown().await.unwrap();
    client.shutdown().await.unwrap();
    worker.await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn transient_read_lock_omission_and_publication_failures_retain_then_recover() {
    let dir = fixture();
    let cache = tempfile::tempdir().unwrap();
    let paths = WorkerPaths::new(Some(cache.path().join("state.json")), None);
    let (client, _diagnostics, worker) = PersistenceWorker::start(paths.clone());
    let mut model = Model::new(View::Guild);
    let source = SourceId::new("projects:runtime");
    let (producer, mut inbox) = Producer::start(selection(dir.path()), model.work(), None).unwrap();
    inbox
        .next_projects()
        .await
        .consume(&client, &mut model)
        .await;
    let prior = model.work().sources[&source].inventory.clone();
    let lock = fs::File::open(dir.path().join(".state/lock")).unwrap();
    lock.lock().unwrap();
    inbox
        .next_projects()
        .await
        .consume(&client, &mut model)
        .await;
    assert_eq!(
        model.work().sources[&source].freshness,
        Freshness::Unavailable
    );
    lock.unlock().unwrap();
    fs::write(dir.path().join("threads/t-0001.toml"), "malformed").unwrap();
    inbox
        .next_projects()
        .await
        .consume(&client, &mut model)
        .await;
    assert!(
        model
            .work_source_notice()
            .unwrap()
            .message
            .contains("invalid")
    );
    fs::remove_file(dir.path().join("threads/t-0001.toml")).unwrap();
    inbox
        .next_projects()
        .await
        .consume(&client, &mut model)
        .await;
    assert_eq!(
        model.work().sources[&source].freshness,
        Freshness::Reconcile
    );
    assert_eq!(model.work().sources[&source].inventory, prior);
    thread(dir.path(), "Recovered work", "2026-09-28T10:00:00Z");
    let cache_path = paths.work_path().unwrap();
    let accepted_bytes = fs::read(&cache_path).unwrap();
    fs::remove_file(&cache_path).unwrap();
    fs::create_dir(&cache_path).unwrap();
    inbox
        .next_projects()
        .await
        .consume(&client, &mut model)
        .await;
    assert_eq!(model.work().sources[&source].inventory, prior);
    fs::remove_dir(&cache_path).unwrap();
    fs::write(&cache_path, accepted_bytes).unwrap();
    inbox
        .next_projects()
        .await
        .consume(&client, &mut model)
        .await;
    assert_eq!(model.work().sources[&source].freshness, Freshness::Current);
    assert_eq!(model.work().sources[&source].generation, 2);
    assert_eq!(
        model.work().sources[&source]
            .inventory
            .quests
            .values()
            .next()
            .unwrap()
            .title,
        "Recovered work"
    );
    assert!(model.work_source_notice().is_none());
    drop(inbox);
    producer.shutdown().await.unwrap();
    client.shutdown().await.unwrap();
    worker.await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn shutdown_during_unacknowledged_delivery_does_not_publish_or_mutate_source() {
    use sha2::{Digest, Sha256};
    let dir = fixture();
    let mut bytes = vec![];
    for name in [
        "PROJECT.md",
        ".state/project.json",
        ".state/lock",
        "threads/t-0001.toml",
    ] {
        bytes.extend(fs::read(dir.path().join(name)).unwrap());
    }
    let before = Sha256::digest(&bytes);
    let (producer, mut inbox) = Producer::start(
        selection(dir.path()),
        &questmancer::work::Projection::default(),
        None,
    )
    .unwrap();
    let pending = inbox.next_projects().await;
    producer.shutdown().await.unwrap();
    drop(pending);
    drop(inbox);
    bytes.clear();
    for name in [
        "PROJECT.md",
        ".state/project.json",
        ".state/lock",
        "threads/t-0001.toml",
    ] {
        bytes.extend(fs::read(dir.path().join(name)).unwrap());
    }
    assert_eq!(Sha256::digest(&bytes), before);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 3);
}

#[tokio::test]
async fn protected_work_cache_disables_selected_source_before_any_project_read() {
    use questmancer::{config::PersistencePaths, persistence::load_startup};
    let config = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    fs::write(config.path().join("config.toml"),"[herdr_projects]\nenabled = true\nsource_id = 'projects:runtime'\ncampaign_id = 'campaign'\nproject_path = '/definitely/not/a/project'\nformat = 'herdr-projects-0.2.34'\nidentity_policy = 'registered_retained_records'\n").unwrap();
    fs::write(cache.path().join("work-state.json"), r#"{"version":999}"#).unwrap();
    let before = fs::read(cache.path().join("work-state.json")).unwrap();
    let paths = PersistencePaths::from_lookup(|name| match name {
        "HERDR_PLUGIN_CONFIG_DIR" => Some(config.path().display().to_string()),
        "HERDR_PLUGIN_STATE_DIR" => Some(cache.path().display().to_string()),
        _ => None,
    });
    let startup = load_startup(paths, None).await;
    assert!(startup.projects.is_none());
    assert_eq!(startup.diagnostics.len(), 1);
    assert!(
        startup
            .model
            .work_source_notice()
            .unwrap()
            .message
            .contains("protected")
    );
    assert_eq!(
        fs::read(cache.path().join("work-state.json")).unwrap(),
        before
    );
}

#[test]
fn synthetic_capture_cost_at_one_ten_and_one_hundred_threads() {
    use questmancer::{
        domain::Timestamp,
        sources::herdr_projects::{Selection, capture},
    };
    let mut measurements = vec![];
    for count in [1, 10, 100] {
        let dir = fixture();
        for index in 2..=count {
            fs::write(dir.path().join(format!("threads/t-{index:04}.toml")),format!("id = 't-{index:04}'\ntitle = 'Work {index}'\nstatus = 'open'\nkind = 'checkout'\ncreated = '2026-09-28T10:00:00Z'\n")).unwrap();
        }
        let selected = Selection::new(
            SourceId::new("projects:cost"),
            "campaign",
            dir.path().to_owned(),
        )
        .unwrap();
        let start = std::time::Instant::now();
        let snapshot = capture(&selected, Timestamp::from_millis(1)).unwrap();
        let elapsed = start.elapsed();
        assert_eq!(snapshot.threads.len(), count);
        measurements.push(serde_json::json!({"threads":count,"capture_micros":elapsed.as_micros(),"snapshot_bytes":serde_json::to_vec(&snapshot).unwrap().len()}));
    }
    if let Some(path) = std::env::var_os("QUESTMANCER_PROJECTS_COST") {
        fs::write(path, serde_json::to_vec_pretty(&measurements).unwrap()).unwrap();
    }
}

#[test]
fn new_attachment_error_remains_visible_while_old_cached_work_is_selected() {
    use questmancer::{
        domain::Timestamp,
        work::{Projection, fixture::FixtureSource, source::SourceNotice, view},
    };
    let mut work = Projection::default();
    for message in FixtureSource::authentication(Timestamp::from_millis(1)) {
        work = questmancer::work::reduce(work, message).unwrap();
    }
    work.restore_cached();
    let mut model = Model::new(View::Guild);
    model.replace_work(work);
    model.open_quest_board();
    model.set_work_source_notice(Some(SourceNotice {
        source: SourceId::new("projects:new-selection"),
        message: "Required source record is malformed; retrying.".into(),
    }));
    let details = view::details(&model).join("\n");
    assert!(details.contains("projects:new-selection"));
    assert!(details.contains("malformed"));
    assert!(details.contains("cached / unverified"));
}

#[test]
fn runtime_diagnostics_export_production_surfaces_when_requested() {
    use questmancer::{
        domain::Timestamp,
        sources::herdr_projects::{Registration, decode_snapshot, map_snapshot},
        work::{Projection, source::SourceNotice},
    };
    let Some(path) = std::env::var_os("QUESTMANCER_PROJECTS_RUNTIME_EXPORT") else {
        return;
    };
    let path = std::path::PathBuf::from(path);
    fs::create_dir_all(&path).unwrap();
    let snapshot =
        decode_snapshot(include_bytes!("fixtures/herdr_projects/source-v1.json")).unwrap();
    let registration =
        Registration::retained_records(snapshot.source.clone(), &snapshot.campaign).unwrap();
    let mapped = map_snapshot(&registration, 1, &snapshot, &Projection::default(), None)
        .unwrap()
        .unwrap();
    for (name, retained, table, width, height) in [
        ("first-read-failure", false, false, 48, 28),
        ("retained-unavailable", true, false, 48, 28),
        ("identity-reconcile", true, true, 120, 38),
        ("other-campaign-error", true, false, 120, 38),
    ] {
        let mut model = Model::new(View::Guild);
        model.set_now(Timestamp::from_millis(100));
        if retained {
            let mut work = mapped.projection().clone();
            if name == "other-campaign-error" {
                work.restore_cached();
            } else if table {
                work.reconcile(&snapshot.source);
            } else {
                work = questmancer::work::reduce(
                    work,
                    questmancer::work::Message::Disconnected(snapshot.source.clone()),
                )
                .unwrap();
            }
            model.replace_work(work);
            if table {
                model.open_war_table();
            } else {
                model.open_quest_board();
            }
        } else {
            model.open_quest_board();
        }
        let message = if table {
            "Source identity changed; attachment stopped. Confirm/rebind the retained project and restart; use fresh tokens for a replacement."
        } else {
            "Source writer holds the project lock; retaining last facts and retrying."
        };
        model.set_work_source_notice(Some(SourceNotice {
            source: if name == "other-campaign-error" {
                SourceId::new("projects:new-selection")
            } else {
                snapshot.source.clone()
            },
            message: message.into(),
        }));
        model.set_work_viewport_size(width, height);
        export_surface(&model, &path.join(format!("{name}.json")), width, height);
    }
}

fn export_surface(model: &Model, path: &Path, width: u16, height: u16) {
    use questmancer::{
        app::ColorMode,
        scene::{
            pixel::{PixelSize, Rgb, RgbBuffer},
            presentation::ScenePresentation,
            render_scene_for_world,
            snapshot::SceneSnapshot,
        },
        ui::scene_adapter::flush_rgb,
    };
    let presentation = ScenePresentation::from_model(model);
    let mut rgb = RgbBuffer::filled(0, 0, Rgb::BLACK);
    let mut terminal =
        ratatui::Terminal::new(ratatui::backend::TestBackend::new(width, height)).unwrap();
    terminal
        .draw(|frame| {
            let _ = render_scene_for_world(
                &SceneSnapshot::from_model(model),
                &presentation,
                PixelSize::new(width, height * 2),
                &mut rgb,
            );
            flush_rgb(
                frame.buffer_mut(),
                ratatui::layout::Rect::new(0, 0, width, height),
                &rgb,
                Rgb::BLACK,
                ColorMode::Xterm256,
            );
            questmancer::ui::scene_overlays::render_scene_overlays(
                frame,
                model,
                &presentation,
                None,
            );
        })
        .unwrap();
    let cells=terminal.backend().buffer().content.iter().map(|c|serde_json::json!({"s":c.symbol(),"fg":format!("{:?}",c.fg),"bg":format!("{:?}",c.bg)})).collect::<Vec<_>>();
    fs::write(
        path,
        serde_json::to_vec_pretty(
            &serde_json::json!({"width":width,"height":height,"cells":cells}),
        )
        .unwrap(),
    )
    .unwrap();
}
