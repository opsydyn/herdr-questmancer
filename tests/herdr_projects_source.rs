use questmancer::{
    domain::Timestamp,
    sources::herdr_projects::{Coverage, Selection, ThreadStatus, capture, decode_snapshot},
    work::SourceId,
};
use std::{collections::BTreeMap, fs, path::Path};

fn fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir_in(fs::canonicalize(std::env::temp_dir()).unwrap()).unwrap();
    fs::create_dir(dir.path().join(".state")).unwrap();
    fs::create_dir(dir.path().join("threads")).unwrap();
    fs::write(dir.path().join("PROJECT.md"), "+++\nname = 'Authentication'\ngoal = 'Preserve compatibility'\n+++\nPrivate standing instructions\n").unwrap();
    fs::write(
        dir.path().join(".state/project.json"),
        r#"{"status":"active","former_slugs":[]}"#,
    )
    .unwrap();
    fs::write(dir.path().join(".state/lock"), "").unwrap();
    fs::write(dir.path().join("threads/t-0001.toml"), "id = 't-0001'\ntitle = 'Migrate API auth'\nstatus = 'resolved'\nkind = 'worktree'\ncreated = '2026-09-28T10:00:00Z'\nresolved_reason = 'manual'\nlast_group = 'resolved'\nagent = 'codex'\nprofile = 'codex-fast'\n").unwrap();
    dir
}
fn selection(path: &Path) -> Selection {
    Selection::new(
        SourceId::new("herdr-projects:test-home"),
        "auth",
        path.to_owned(),
    )
    .unwrap()
}

#[test]
fn registered_mapper_rejects_changed_creation_and_partial_omission_before_reduction() {
    use questmancer::sources::herdr_projects::{MappingError, Registration, map_snapshot};
    use questmancer::work::Projection;
    let dir = fixture();
    let registration =
        Registration::retained_records(SourceId::new("herdr-projects:test-home"), "auth").unwrap();
    let mut snapshot = capture(&selection(dir.path()), Timestamp::from_millis(100)).unwrap();
    let first = map_snapshot(&registration, 1, &snapshot, &Projection::default(), None)
        .unwrap()
        .unwrap();
    let accepted = first.projection().clone();
    let checkpoint = first.checkpoint().clone();
    snapshot.threads.get_mut("t-0001").unwrap().created = "2026-09-29T10:00:00Z".into();
    assert_eq!(
        map_snapshot(&registration, 1, &snapshot, &accepted, Some(&checkpoint)),
        Err(MappingError::Identity)
    );
    snapshot.threads.clear();
    assert_eq!(
        map_snapshot(&registration, 1, &snapshot, &accepted, Some(&checkpoint)),
        Err(MappingError::MissingThread)
    );
    assert_eq!(first.projection(), &accepted);
    assert_eq!(first.checkpoint(), &checkpoint);
}

#[tokio::test]
async fn mapped_projection_and_identity_checkpoint_restart_as_one_v2_cache() {
    use questmancer::{
        persistence::{load_work_cache, publish_work, publish_work_proposal},
        sources::herdr_projects::{Registration, map_snapshot},
        work::{Freshness, Projection},
    };
    let dir = fixture();
    let registration =
        Registration::retained_records(SourceId::new("herdr-projects:test-home"), "auth").unwrap();
    let snapshot = capture(&selection(dir.path()), Timestamp::from_millis(100)).unwrap();
    let proposal = map_snapshot(&registration, 1, &snapshot, &Projection::default(), None)
        .unwrap()
        .unwrap();
    let cache_dir = tempfile::tempdir().unwrap();
    let path = cache_dir.path().join("work-state.json");
    publish_work_proposal(&path, &proposal).await.unwrap();
    let bytes = fs::read(&path).unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()["version"],
        2
    );
    let cached = load_work_cache(&path).await.unwrap().unwrap();
    assert_eq!(
        cached.herdr_projects(registration.source()),
        Some(proposal.checkpoint())
    );
    assert_eq!(
        cached.projection.sources[registration.source()].freshness,
        Freshness::Cached
    );
    assert!(!publish_work(&path, &cached.projection).await.unwrap());
    assert_eq!(fs::read(&path).unwrap(), bytes);
    let restarted = map_snapshot(
        &registration,
        2,
        &snapshot,
        &cached.projection,
        cached.herdr_projects(registration.source()),
    )
    .unwrap()
    .unwrap();
    publish_work_proposal(&path, &restarted).await.unwrap();
    let cache = load_work_cache(&path).await.unwrap().unwrap();
    assert_eq!(
        cache.projection.sources[registration.source()].generation,
        2
    );
    assert_eq!(
        cache.herdr_projects(registration.source()),
        Some(restarted.checkpoint())
    );
}

#[test]
fn mapper_preserves_identity_on_reopen_and_equivalent_creation_and_ignores_probe_times() {
    use questmancer::{
        sources::herdr_projects::{Registration, map_snapshot},
        work::{Projection, QuestKind, QuestState},
    };
    let dir = fixture();
    let registration =
        Registration::retained_records(SourceId::new("herdr-projects:test-home"), "auth").unwrap();
    let mut snapshot = capture(&selection(dir.path()), Timestamp::from_millis(100)).unwrap();
    let proposal = map_snapshot(&registration, 1, &snapshot, &Projection::default(), None)
        .unwrap()
        .unwrap();
    let quest = proposal.projection().sources[registration.source()]
        .inventory
        .quests
        .values()
        .next()
        .unwrap();
    assert_eq!(quest.state, QuestState::Closed);
    assert_eq!(quest.kind, QuestKind::Unspecified);
    snapshot.captured_at = Timestamp::from_millis(200);
    snapshot.threads.get_mut("t-0001").unwrap().created = "2026-09-28T11:00:00+01:00".into();
    snapshot.threads.get_mut("t-0001").unwrap().updated = "2026-09-29T12:00:00Z".into();
    assert!(
        map_snapshot(
            &registration,
            1,
            &snapshot,
            proposal.projection(),
            Some(proposal.checkpoint())
        )
        .unwrap()
        .is_none()
    );
    let thread = snapshot.threads.get_mut("t-0001").unwrap();
    thread.status = ThreadStatus::Open;
    thread.resolved_reason.clear();
    let next = map_snapshot(
        &registration,
        1,
        &snapshot,
        proposal.projection(),
        Some(proposal.checkpoint()),
    )
    .unwrap()
    .unwrap();
    let reopened = &next.projection().sources[registration.source()]
        .inventory
        .quests[&quest.id];
    assert_eq!(reopened.state, QuestState::Unknown);
    assert_eq!(reopened.revision, 2);
    assert!(
        next.projection().sources[registration.source()]
            .inventory
            .obligations
            .is_empty()
    );
    assert!(
        next.projection().sources[registration.source()]
            .bindings
            .is_empty()
    );
}

#[test]
fn mapper_revokes_reference_slots_and_never_promotes_group_hash_or_ack_to_validation() {
    use questmancer::{
        sources::herdr_projects::{
            Diagnostic, HomeReport, RecordedGroup, Registration, map_snapshot,
        },
        work::{ArtifactKind, Projection},
    };
    let dir = fixture();
    let registration =
        Registration::retained_records(SourceId::new("herdr-projects:test-home"), "auth").unwrap();
    let mut snapshot = capture(&selection(dir.path()), Timestamp::from_millis(100)).unwrap();
    let thread = snapshot.threads.get_mut("t-0001").unwrap();
    thread.report_hash = "a".repeat(64);
    thread.acked_report_hash = thread.report_hash.clone();
    thread.home_report = Some(HomeReport {
        relative_path: "threads/t-0001.md".into(),
        sha256: thread.report_hash.clone(),
        matches_reported_hash: true,
    });
    thread.recorded_group = Some(RecordedGroup::ReadyForReview);
    thread.pr = "https://github.com/owner/repo/pull/481".into();
    let proposal = map_snapshot(&registration, 1, &snapshot, &Projection::default(), None)
        .unwrap()
        .unwrap();
    let inventory = &proposal.projection().sources[registration.source()].inventory;
    assert_eq!(inventory.artifacts.len(), 2);
    assert!(inventory.obligations.is_empty());
    for a in inventory.artifacts.values() {
        assert!(a.validation.is_none());
        assert!(a.head_revision.is_none());
    }
    assert!(
        inventory
            .artifacts
            .values()
            .find(|a| a.kind == ArtifactKind::PullRequest)
            .unwrap()
            .uri
            .is_some()
    );
    let thread = snapshot.threads.get_mut("t-0001").unwrap();
    thread.home_report = None;
    thread.pr.clear();
    snapshot
        .diagnostics
        .push(Diagnostic::ReportMissing("t-0001".into()));
    let next = map_snapshot(
        &registration,
        1,
        &snapshot,
        proposal.projection(),
        Some(proposal.checkpoint()),
    )
    .unwrap()
    .unwrap();
    for a in next.projection().sources[registration.source()]
        .inventory
        .artifacts
        .values()
    {
        assert!(a.uri.is_none());
        assert!(a.title.contains("unavailable"));
        assert_eq!(a.revision, 2);
    }
    assert!(
        next.projection().sources[registration.source()]
            .inventory
            .obligations
            .is_empty()
    );
}

#[test]
fn mapper_requires_locked_registered_context_and_checked_observer_order() {
    use questmancer::{
        sources::herdr_projects::{
            Coherence, Diagnostic, MappingError, Registration, map_snapshot,
        },
        work::Projection,
    };
    let dir = fixture();
    let registration =
        Registration::retained_records(SourceId::new("herdr-projects:test-home"), "auth").unwrap();
    let mut snapshot = capture(&selection(dir.path()), Timestamp::from_millis(100)).unwrap();
    snapshot.coherence = Coherence::UnlockedObservation;
    snapshot.diagnostics.push(Diagnostic::MissingRecordLock);
    assert_eq!(
        map_snapshot(&registration, 1, &snapshot, &Projection::default(), None),
        Err(MappingError::Unlocked)
    );
    snapshot.coherence = Coherence::CooperativeRecordLock;
    snapshot.diagnostics.pop();
    let proposal = map_snapshot(&registration, 1, &snapshot, &Projection::default(), None)
        .unwrap()
        .unwrap();
    assert_eq!(
        map_snapshot(&registration, 1, &snapshot, proposal.projection(), None),
        Err(MappingError::Identity)
    );
    assert!(MappingError::Identity.requires_rebind());
    assert!(!MappingError::MissingThread.requires_rebind());
    let mut accepted = proposal.projection().clone();
    accepted
        .sources
        .get_mut(registration.source())
        .unwrap()
        .cursor = u64::MAX;
    snapshot.project.name = "changed".into();
    assert_eq!(
        map_snapshot(
            &registration,
            1,
            &snapshot,
            &accepted,
            Some(proposal.checkpoint())
        ),
        Err(MappingError::Exhausted)
    );
    accepted
        .sources
        .get_mut(registration.source())
        .unwrap()
        .generation = u64::MAX;
    assert_eq!(
        registration.next_generation(&accepted),
        Err(MappingError::Exhausted)
    );
    assert_eq!(
        map_snapshot(
            &registration,
            1,
            &snapshot,
            &accepted,
            Some(proposal.checkpoint())
        ),
        Err(MappingError::Stale)
    );
}

#[test]
fn changed_report_bytes_advance_the_reference_slot_and_unsafe_pr_paths_have_no_uri() {
    use questmancer::{
        sources::herdr_projects::{HomeReport, Registration, map_snapshot},
        work::{ArtifactKind, Projection},
    };
    let dir = fixture();
    let registration =
        Registration::retained_records(SourceId::new("herdr-projects:test-home"), "auth").unwrap();
    let mut snapshot = capture(&selection(dir.path()), Timestamp::from_millis(100)).unwrap();
    let thread = snapshot.threads.get_mut("t-0001").unwrap();
    thread.report_hash = "a".repeat(64);
    thread.home_report = Some(HomeReport {
        relative_path: "threads/t-0001.md".into(),
        sha256: thread.report_hash.clone(),
        matches_reported_hash: true,
    });
    let proposal = map_snapshot(&registration, 1, &snapshot, &Projection::default(), None)
        .unwrap()
        .unwrap();
    let thread = snapshot.threads.get_mut("t-0001").unwrap();
    thread.report_hash = "b".repeat(64);
    thread.home_report.as_mut().unwrap().sha256 = thread.report_hash.clone();
    thread.pr = "https://github.com/../repo/pull/481".into();
    let next = map_snapshot(
        &registration,
        1,
        &snapshot,
        proposal.projection(),
        Some(proposal.checkpoint()),
    )
    .unwrap()
    .unwrap();
    let inventory = &next.projection().sources[registration.source()].inventory;
    assert_eq!(
        inventory
            .artifacts
            .values()
            .find(|a| a.kind == ArtifactKind::Report)
            .unwrap()
            .revision,
        2
    );
    assert!(
        inventory
            .artifacts
            .values()
            .find(|a| a.kind == ArtifactKind::PullRequest)
            .unwrap()
            .uri
            .is_none()
    );
}

#[tokio::test]
async fn mapped_consumer_retains_facts_on_omission_and_failed_publication_then_recovers() {
    use questmancer::{
        app::{Model, View},
        persistence::{PersistenceWorker, WorkerPaths, load_work_cache},
        sources::herdr_projects::Registration,
        work::{Freshness, runtime::accept_snapshot},
    };
    let dir = fixture();
    let cache = tempfile::tempdir().unwrap();
    let path = cache.path().join("work-state.json");
    let reg =
        Registration::retained_records(SourceId::new("herdr-projects:test-home"), "auth").unwrap();
    let mut snapshot = capture(&selection(dir.path()), Timestamp::from_millis(100)).unwrap();
    let (client, _, worker) = PersistenceWorker::start(WorkerPaths::new(
        Some(cache.path().join("state.json")),
        None,
    ));
    let mut model = Model::new(View::Guild);
    let checkpoint = accept_snapshot(&client, &mut model, &reg, 1, &snapshot, None)
        .await
        .unwrap();
    let original = model.work().sources[reg.source()].inventory.clone();
    let prior_bytes = fs::read(&path).unwrap();
    let mut omitted = snapshot.clone();
    omitted.threads.clear();
    assert!(
        accept_snapshot(&client, &mut model, &reg, 1, &omitted, Some(&checkpoint))
            .await
            .is_err()
    );
    assert_eq!(
        model.work().sources[reg.source()].freshness,
        Freshness::Reconcile
    );
    assert_eq!(model.work().sources[reg.source()].inventory, original);
    assert_eq!(fs::read(&path).unwrap(), prior_bytes);
    accept_snapshot(&client, &mut model, &reg, 1, &snapshot, Some(&checkpoint))
        .await
        .unwrap();
    assert_eq!(
        model.work().sources[reg.source()].freshness,
        Freshness::Current
    );
    let prior_bytes = fs::read(&path).unwrap();
    fs::create_dir(cache.path().join("work-state.json.tmp")).unwrap();
    snapshot.threads.get_mut("t-0001").unwrap().title = "Unpublished new title".into();
    assert!(
        accept_snapshot(&client, &mut model, &reg, 1, &snapshot, Some(&checkpoint))
            .await
            .is_err()
    );
    assert_eq!(model.work().sources[reg.source()].inventory, original);
    assert_eq!(fs::read(&path).unwrap(), prior_bytes);
    fs::remove_dir(cache.path().join("work-state.json.tmp")).unwrap();
    let next = accept_snapshot(&client, &mut model, &reg, 1, &snapshot, Some(&checkpoint))
        .await
        .unwrap();
    client.shutdown().await.unwrap();
    worker.await.unwrap();
    let restored = load_work_cache(&path).await.unwrap().unwrap();
    assert_eq!(restored.herdr_projects(reg.source()), Some(&next));
    assert_eq!(
        restored.projection.sources[reg.source()].freshness,
        Freshness::Cached
    );
    assert_eq!(
        restored.projection.sources[reg.source()]
            .inventory
            .quests
            .values()
            .next()
            .unwrap()
            .title,
        "Unpublished new title"
    );
}

#[tokio::test]
async fn v2_cache_protects_invalid_future_and_duplicate_identity_evidence() {
    use questmancer::{
        persistence::{load_work_cache, publish_work_proposal},
        sources::herdr_projects::{Registration, map_snapshot},
        work::Projection,
    };
    let dir = fixture();
    let cache = tempfile::tempdir().unwrap();
    let path = cache.path().join("work-state.json");
    let registration =
        Registration::retained_records(SourceId::new("herdr-projects:test-home"), "auth").unwrap();
    let snapshot = capture(&selection(dir.path()), Timestamp::from_millis(100)).unwrap();
    let proposal = map_snapshot(&registration, 1, &snapshot, &Projection::default(), None)
        .unwrap()
        .unwrap();
    publish_work_proposal(&path, &proposal).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let mut future = value.clone();
    future["version"] = 3.into();
    let mut wrong = value.clone();
    wrong["checkpoints"][registration.source().0.as_str()]["checkpoint"]["threads"] =
        serde_json::json!({});
    let compact = serde_json::to_string(&value).unwrap();
    let evidence =
        &value["checkpoints"][registration.source().0.as_str()]["checkpoint"]["threads"]["t-0001"];
    let duplicate = compact.replacen(
        "\"t-0001\":",
        &format!("\"t-0001\":{evidence},\"t-0001\":"),
        1,
    );
    for bad in [
        serde_json::to_string(&future).unwrap(),
        serde_json::to_string(&wrong).unwrap(),
        duplicate,
    ] {
        fs::write(&path, &bad).unwrap();
        assert!(load_work_cache(&path).await.is_err());
        assert!(publish_work_proposal(&path, &proposal).await.is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), bad);
    }
}

#[tokio::test]
async fn v1_migration_and_ordinary_intent_writes_preserve_registered_checkpoints() {
    use questmancer::{
        persistence::{load_work_cache, publish_work, publish_work_proposal},
        sources::herdr_projects::{Registration, map_snapshot},
        work::{self, LocalIntent, Projection},
    };
    let cache = tempfile::tempdir().unwrap();
    let path = cache.path().join("work-state.json");
    let mut legacy = Projection::default();
    for m in work::fixture::FixtureSource::authentication(Timestamp::from_millis(100)) {
        legacy = work::reduce(legacy, m).unwrap();
    }
    let obligation = legacy
        .sources
        .values()
        .flat_map(|snapshot| snapshot.inventory.obligations.values())
        .next()
        .unwrap();
    legacy.intent.insert(
        obligation.id.clone(),
        LocalIntent {
            incarnation: obligation.incarnation.clone(),
            seen: true,
            deferred_until: Some(Timestamp::from_millis(200)),
        },
    );
    fs::write(
        &path,
        serde_json::to_vec(&serde_json::json!({"version":1,"projection":legacy})).unwrap(),
    )
    .unwrap();
    let loaded = load_work_cache(&path).await.unwrap().unwrap();
    assert!(loaded.checkpoints.is_empty());
    assert_eq!(loaded.projection.intent, legacy.intent);
    assert!(publish_work(&path, &loaded.projection).await.unwrap());
    let dir = fixture();
    let registration =
        Registration::retained_records(SourceId::new("herdr-projects:test-home"), "auth").unwrap();
    let snapshot = capture(&selection(dir.path()), Timestamp::from_millis(100)).unwrap();
    let proposal = map_snapshot(&registration, 1, &snapshot, &loaded.projection, None)
        .unwrap()
        .unwrap();
    publish_work_proposal(&path, &proposal).await.unwrap();
    let mut ordinary = load_work_cache(&path).await.unwrap().unwrap().projection;
    ordinary.intent.values_mut().next().unwrap().seen = false;
    publish_work(&path, &ordinary).await.unwrap();
    let saved = load_work_cache(&path).await.unwrap().unwrap();
    assert_eq!(
        saved.herdr_projects(registration.source()),
        Some(proposal.checkpoint())
    );
    assert_eq!(saved.projection.intent, ordinary.intent);
    let before = fs::read(&path).unwrap();
    ordinary
        .sources
        .get_mut(registration.source())
        .unwrap()
        .inventory
        .quests
        .values_mut()
        .next()
        .unwrap()
        .title = "impostor".into();
    assert!(publish_work(&path, &ordinary).await.is_err());
    assert_eq!(fs::read(&path).unwrap(), before);
}
fn bytes(path: &Path) -> BTreeMap<String, Vec<u8>> {
    fn collect(root: &Path, current: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
        for entry in fs::read_dir(current).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                collect(root, &entry.path(), files);
            } else {
                files.insert(
                    entry
                        .path()
                        .strip_prefix(root)
                        .unwrap()
                        .to_string_lossy()
                        .into_owned(),
                    fs::read(entry.path()).unwrap(),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    collect(path, path, &mut files);
    files
}

#[test]
fn mapped_context_shows_literal_source_values_without_rust_option_syntax() {
    use questmancer::{
        sources::herdr_projects::{Registration, map_snapshot},
        work::Projection,
    };
    let snapshot =
        decode_snapshot(include_bytes!("fixtures/herdr_projects/source-v1.json")).unwrap();
    let registration =
        Registration::retained_records(snapshot.source.clone(), &snapshot.campaign).unwrap();
    let proposal = map_snapshot(&registration, 1, &snapshot, &Projection::default(), None)
        .unwrap()
        .unwrap();
    let text = proposal.projection().sources[registration.source()]
        .inventory
        .quests
        .values()
        .next()
        .unwrap()
        .description
        .as_ref()
        .unwrap();
    assert!(!text.contains("Some("));
    assert!(text.contains("Source thread status: resolved"));
    assert!(text.contains("Placement: worktree"));
}

#[test]
fn obsolete_generation_is_rejected_before_an_old_checkpoint_can_poison_fresh_work() {
    use questmancer::{
        sources::herdr_projects::{MappingError, Registration, map_snapshot},
        work::Projection,
    };
    let old_snapshot =
        decode_snapshot(include_bytes!("fixtures/herdr_projects/source-v1.json")).unwrap();
    let registration =
        Registration::retained_records(old_snapshot.source.clone(), &old_snapshot.campaign)
            .unwrap();
    let first = map_snapshot(
        &registration,
        1,
        &old_snapshot,
        &Projection::default(),
        None,
    )
    .unwrap()
    .unwrap();
    let mut newer = old_snapshot.clone();
    let mut thread = newer.threads["t-0001"].clone();
    thread.id = "t-0002".into();
    newer.threads.insert(thread.id.clone(), thread);
    let current = map_snapshot(
        &registration,
        2,
        &newer,
        first.projection(),
        Some(first.checkpoint()),
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        map_snapshot(
            &registration,
            1,
            &old_snapshot,
            current.projection(),
            Some(first.checkpoint())
        ),
        Err(MappingError::Stale)
    );
    let mut unlocked = old_snapshot;
    unlocked.coherence = questmancer::sources::herdr_projects::Coherence::UnlockedObservation;
    unlocked
        .diagnostics
        .push(questmancer::sources::herdr_projects::Diagnostic::MissingRecordLock);
    assert_eq!(
        map_snapshot(
            &registration,
            1,
            &unlocked,
            current.projection(),
            Some(first.checkpoint())
        ),
        Err(MappingError::Stale)
    );
}

#[tokio::test]
async fn duplicate_prepared_proposal_does_not_downgrade_acknowledged_current_facts() {
    use questmancer::{
        app::{Model, View},
        persistence::{PersistenceWorker, WorkerPaths},
        sources::herdr_projects::{Registration, map_snapshot},
        work::{Projection, runtime::accept_proposal},
    };
    let snapshot =
        decode_snapshot(include_bytes!("fixtures/herdr_projects/source-v1.json")).unwrap();
    let registration =
        Registration::retained_records(snapshot.source.clone(), &snapshot.campaign).unwrap();
    let proposal = map_snapshot(&registration, 1, &snapshot, &Projection::default(), None)
        .unwrap()
        .unwrap();
    let cache = tempfile::tempdir().unwrap();
    let (client, _, worker) = PersistenceWorker::start(WorkerPaths::new(
        Some(cache.path().join("state.json")),
        None,
    ));
    let mut model = Model::new(View::Guild);
    accept_proposal(&client, &mut model, proposal.clone())
        .await
        .unwrap();
    let accepted = model.work().clone();
    assert!(
        accept_proposal(&client, &mut model, proposal)
            .await
            .is_err()
    );
    assert_eq!(model.work(), &accepted);
    client.shutdown().await.unwrap();
    worker.await.unwrap();
}

#[tokio::test]
async fn stale_prepared_proposal_cannot_overwrite_a_newer_atomic_observation() {
    use questmancer::{
        persistence::publish_work_proposal,
        sources::herdr_projects::{Registration, map_snapshot},
        work::Projection,
    };
    let mut snapshot =
        decode_snapshot(include_bytes!("fixtures/herdr_projects/source-v1.json")).unwrap();
    let registration =
        Registration::retained_records(snapshot.source.clone(), &snapshot.campaign).unwrap();
    let first = map_snapshot(&registration, 1, &snapshot, &Projection::default(), None)
        .unwrap()
        .unwrap();
    let cache = tempfile::tempdir().unwrap();
    let path = cache.path().join("work-state.json");
    publish_work_proposal(&path, &first).await.unwrap();
    snapshot.project.name = "Newer accepted campaign".into();
    let newer = map_snapshot(
        &registration,
        1,
        &snapshot,
        first.projection(),
        Some(first.checkpoint()),
    )
    .unwrap()
    .unwrap();
    snapshot.project.name = "Other proposal from old base".into();
    let obsolete = map_snapshot(
        &registration,
        1,
        &snapshot,
        first.projection(),
        Some(first.checkpoint()),
    )
    .unwrap()
    .unwrap();
    publish_work_proposal(&path, &newer).await.unwrap();
    let bytes = fs::read(&path).unwrap();
    assert!(publish_work_proposal(&path, &obsolete).await.is_err());
    assert_eq!(fs::read(&path).unwrap(), bytes);
}

proptest::proptest! {
    #[test]
    fn registered_identity_isolation_and_repeated_observations_are_inert(seed in 0_u32..100_000) {
        use questmancer::{sources::herdr_projects::{Registration,map_snapshot},work::Projection};
        let mut snapshot=decode_snapshot(include_bytes!("fixtures/herdr_projects/source-v1.json")).unwrap();
        snapshot.source=SourceId::new(&format!("registered-{seed}:a"));
        let registration=Registration::retained_records(snapshot.source.clone(),&snapshot.campaign).unwrap();
        let first=map_snapshot(&registration,1,&snapshot,&Projection::default(),None).unwrap().unwrap();
        snapshot.captured_at=Timestamp::from_millis(i64::from(seed)+101);
        proptest::prop_assert!(map_snapshot(&registration,1,&snapshot,first.projection(),Some(first.checkpoint())).unwrap().is_none());
        snapshot.source=SourceId::new(&format!("registered-{seed}:b"));
        let other=Registration::retained_records(snapshot.source.clone(),&snapshot.campaign).unwrap();
        let second=map_snapshot(&other,1,&snapshot,first.projection(),None).unwrap().unwrap();
        let a=&second.projection().sources[registration.source()].inventory;
        let b=&second.projection().sources[other.source()].inventory;
        proptest::prop_assert_ne!(a.quests.keys().next(),b.quests.keys().next());
        proptest::prop_assert_eq!(second.projection().sources.len(),2);
    }
    #[test]
    fn partial_thread_omission_cannot_remove_or_close_known_work(seed in 0_u32..100_000) {
        use questmancer::{sources::herdr_projects::{MappingError,Registration,map_snapshot},work::Projection};
        let mut snapshot=decode_snapshot(include_bytes!("fixtures/herdr_projects/source-v1.json")).unwrap();
        snapshot.threads.get_mut("t-0001").unwrap().title=format!("Retained quest {seed}");
        let registration=Registration::retained_records(snapshot.source.clone(),&snapshot.campaign).unwrap();
        let first=map_snapshot(&registration,1,&snapshot,&Projection::default(),None).unwrap().unwrap();
        let before=first.projection().clone();
        snapshot.threads.clear();
        proptest::prop_assert_eq!(map_snapshot(&registration,1,&snapshot,&before,Some(first.checkpoint())),Err(MappingError::MissingThread));
        proptest::prop_assert_eq!(first.projection(),&before);
    }
}

#[cfg(feature = "storybook")]
#[test]
fn registered_mapping_production_surfaces_export_when_requested() {
    use questmancer::{
        app::{ColorMode, Model, View},
        scene::{
            pixel::{PixelSize, Rgb, RgbBuffer},
            presentation::ScenePresentation,
            render_scene_for_world,
            snapshot::SceneSnapshot,
        },
        sources::herdr_projects::{ProjectStatus, RecordedGroup, Registration, map_snapshot},
        ui::scene_adapter::flush_rgb,
        work::{Projection, QuestId, WorkSelection},
    };
    let Some(path) = std::env::var_os("QUESTMANCER_PROJECTS_EXPORT") else {
        return;
    };
    let path = std::path::PathBuf::from(path);
    fs::create_dir_all(&path).unwrap();
    let mut snapshot =
        decode_snapshot(include_bytes!("fixtures/herdr_projects/source-v1.json")).unwrap();
    snapshot.project.status = ProjectStatus::Archived;
    let mut open = snapshot.threads["t-0001"].clone();
    open.id = "t-0002".into();
    open.title = "Investigate compatibility".into();
    open.status = ThreadStatus::Open;
    open.resolved_reason.clear();
    open.recorded_group = Some(RecordedGroup::Working);
    let mut failed = open.clone();
    failed.id = "t-0003".into();
    failed.title = "Launch integration validation".into();
    failed.status = ThreadStatus::Failed;
    failed.error = "Source launch failed; no human request was supplied.".into();
    failed.recorded_group = Some(RecordedGroup::WaitingOnYou);
    snapshot.threads.insert(open.id.clone(), open);
    snapshot.threads.insert(failed.id.clone(), failed);
    let registration =
        Registration::retained_records(snapshot.source.clone(), &snapshot.campaign).unwrap();
    let proposal = map_snapshot(&registration, 1, &snapshot, &Projection::default(), None)
        .unwrap()
        .unwrap();
    for (name, selected, table, width, height) in [
        ("closed-table", "t-0001", true, 120, 38),
        ("unknown-table", "t-0002", true, 120, 38),
        ("board", "t-0002", false, 120, 38),
        ("narrow-board", "t-0002", false, 48, 28),
    ] {
        let mut model = Model::new(View::Guild);
        model.set_now(Timestamp::from_millis(100));
        model.replace_work(proposal.projection().clone());
        model.select_work(WorkSelection::Quest(QuestId::new(
            registration.source(),
            &format!("{}.thread.{selected}", snapshot.campaign),
        )));
        if table {
            model.open_war_table();
        }
        model.set_work_viewport_size(width, height);
        let presentation = ScenePresentation::from_model(&model);
        let mut rgb = RgbBuffer::filled(0, 0, Rgb::BLACK);
        let mut terminal =
            ratatui::Terminal::new(ratatui::backend::TestBackend::new(width, height)).unwrap();
        terminal
            .draw(|frame| {
                let _ = render_scene_for_world(
                    &SceneSnapshot::from_model(&model),
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
                    &model,
                    &presentation,
                    None,
                );
            })
            .unwrap();
        let cells=terminal.backend().buffer().content.iter().map(|c|serde_json::json!({"s":c.symbol(),"fg":format!("{:?}",c.fg),"bg":format!("{:?}",c.bg)})).collect::<Vec<_>>();
        fs::write(
            path.join(format!("{name}.json")),
            serde_json::to_vec_pretty(
                &serde_json::json!({"width":width,"height":height,"cells":cells}),
            )
            .unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn snapshot_is_versioned_literal_partial_and_does_not_write_to_the_source() {
    let dir = fixture();
    let before = bytes(dir.path());
    let snapshot = capture(&selection(dir.path()), Timestamp::from_millis(100)).unwrap();
    assert_eq!(snapshot.version, 1);
    assert_eq!(snapshot.coverage, Coverage::Partial);
    assert_eq!(snapshot.project.name, "Authentication");
    assert_eq!(snapshot.threads["t-0001"].status, ThreadStatus::Resolved);
    assert_eq!(snapshot.threads["t-0001"].resolved_reason, "manual");
    assert!(!snapshot.capabilities.structured_obligations.is_supported());
    assert!(!snapshot.capabilities.live_bindings.is_supported());
    assert!(!snapshot.capabilities.durable_identity.is_supported());
    let json = serde_json::to_vec(&snapshot).unwrap();
    assert!(!String::from_utf8_lossy(&json).contains("Private standing instructions"));
    assert_eq!(decode_snapshot(&json).unwrap(), snapshot);
    assert_eq!(bytes(dir.path()), before);
}

#[test]
fn missing_lock_is_partial_and_never_creates_source_files() {
    use questmancer::sources::herdr_projects::{Coherence, Diagnostic};
    let dir = fixture();
    fs::remove_file(dir.path().join(".state/lock")).unwrap();
    let snapshot = capture(&selection(dir.path()), Timestamp::from_millis(100)).unwrap();
    assert_eq!(snapshot.coherence, Coherence::UnlockedObservation);
    assert!(
        snapshot
            .diagnostics
            .contains(&Diagnostic::MissingRecordLock)
    );
    assert!(!dir.path().join(".state/lock").exists());
}

#[test]
fn malformed_or_unknown_required_records_reject_the_whole_capture() {
    use questmancer::sources::herdr_projects::ReadError;
    let dir = fixture();
    let path = dir.path().join("threads/t-0001.toml");
    let valid = fs::read_to_string(&path).unwrap();
    for bad in [
        "not valid TOML".to_owned(),
        valid.replace("'resolved'", "'future-state'"),
        valid.replace("'worktree'", "'review'"),
        valid.replace("id = 't-0001'", "id = 't-0002'"),
        valid.replace("2026-09-28T10:00:00Z", "not-a-time"),
        valid.replace("last_group = 'resolved'", "last_group = 'new-group'"),
    ] {
        fs::write(&path, bad).unwrap();
        assert!(matches!(
            capture(&selection(dir.path()), Timestamp::from_millis(100)),
            Err(ReadError::Invalid { .. })
        ));
    }
    fs::write(&path, valid).unwrap();
    fs::write(dir.path().join("threads/t-0002.toml"), "broken").unwrap();
    assert!(capture(&selection(dir.path()), Timestamp::from_millis(100)).is_err());
    fs::remove_file(dir.path().join("threads/t-0002.toml")).unwrap();
    fs::write(dir.path().join(".state/project.json"), "{}").unwrap();
    assert!(capture(&selection(dir.path()), Timestamp::from_millis(100)).is_err());
}

#[test]
fn symlinked_roots_ancestors_records_and_reports_are_not_followed() {
    use std::os::unix::fs::symlink;
    let dir = fixture();
    let links = tempfile::tempdir_in(fs::canonicalize(std::env::temp_dir()).unwrap()).unwrap();
    symlink(dir.path(), links.path().join("project")).unwrap();
    assert!(
        capture(
            &selection(&links.path().join("project")),
            Timestamp::from_millis(100)
        )
        .is_err()
    );
    for name in [
        "PROJECT.md",
        ".state/project.json",
        ".state/lock",
        "threads/t-0001.toml",
    ] {
        let path = dir.path().join(name);
        let content = fs::read(&path).unwrap();
        fs::remove_file(&path).unwrap();
        symlink(links.path().join("outside"), &path).unwrap();
        assert!(
            capture(&selection(dir.path()), Timestamp::from_millis(100)).is_err(),
            "{name}"
        );
        fs::remove_file(path).unwrap();
        fs::write(dir.path().join(name), content).unwrap();
    }
    symlink(
        links.path().join("outside"),
        dir.path().join("threads/t-0001.md"),
    )
    .unwrap();
    assert!(capture(&selection(dir.path()), Timestamp::from_millis(100)).is_err());
    assert!(!links.path().join("outside").exists());
}

#[test]
fn selection_rejects_identity_and_path_escapes() {
    for (source, campaign, path) in [
        ("", "auth", "/private/tmp"),
        ("good", "../auth", "/private/tmp"),
        ("other/home", "auth", "/private/tmp"),
        ("good", "auth", "relative"),
    ] {
        assert!(Selection::new(SourceId::new(source), campaign, path.into()).is_err());
    }
    let dir = fixture();
    let path = dir.path().join("threads/../");
    assert!(capture(&selection(&path), Timestamp::from_millis(100)).is_err());
}

#[test]
fn reports_are_only_home_references_with_matching_hash_evidence() {
    use questmancer::sources::herdr_projects::Diagnostic;
    use sha2::{Digest, Sha256};
    let dir = fixture();
    let path = dir.path().join("threads/t-0001.toml");
    let report = b"## Report\nThe API migration needs review.\n";
    let hash = format!("{:x}", Sha256::digest(report));
    let raw = fs::read_to_string(&path).unwrap();
    fs::write(&path, format!("{raw}report_hash = '{hash}'\nthread_dir = '/outside'\npane_id = 'w99:p1'\nagent_args = ['never export these']\n")).unwrap();
    let missing = capture(&selection(dir.path()), Timestamp::from_millis(100)).unwrap();
    assert!(
        missing
            .diagnostics
            .contains(&Diagnostic::ReportMissing("t-0001".into()))
    );
    fs::write(dir.path().join("threads/t-0001.md"), report).unwrap();
    let matched = capture(&selection(dir.path()), Timestamp::from_millis(100)).unwrap();
    assert!(
        matched.threads["t-0001"]
            .home_report
            .as_ref()
            .unwrap()
            .matches_reported_hash
    );
    let json = serde_json::to_string(&matched).unwrap();
    for private in [
        "/outside",
        "w99:p1",
        "never export these",
        "The API migration needs review.",
    ] {
        assert!(!json.contains(private));
    }
    fs::write(
        dir.path().join("threads/t-0001.md"),
        "New report, ticker has not recorded it yet",
    )
    .unwrap();
    let changed = capture(&selection(dir.path()), Timestamp::from_millis(100)).unwrap();
    assert!(
        changed
            .diagnostics
            .contains(&Diagnostic::ReportHashMismatch("t-0001".into()))
    );
    assert!(
        !changed.threads["t-0001"]
            .home_report
            .as_ref()
            .unwrap()
            .matches_reported_hash
    );
}

#[test]
fn decoder_rejects_future_versions_false_capabilities_and_duplicate_thread_keys() {
    let dir = fixture();
    let snapshot = capture(&selection(dir.path()), Timestamp::from_millis(100)).unwrap();
    let valid = serde_json::to_value(snapshot).unwrap();
    for mutate in [
        "version",
        "format",
        "capability",
        "identity",
        "extra",
        "diagnostics",
    ] {
        let mut invalid = valid.clone();
        match mutate {
            "version" => invalid["version"] = 2.into(),
            "format" => invalid["format"] = "future".into(),
            "capability" => invalid["capabilities"]["structured_obligations"] = true.into(),
            "identity" => invalid["threads"]["t-0001"]["id"] = "t-0002".into(),
            "extra" => invalid["command"] = "do something".into(),
            _ => invalid["diagnostics"] = serde_json::json!([]),
        }
        assert!(
            decode_snapshot(&serde_json::to_vec(&invalid).unwrap()).is_err(),
            "{mutate}"
        );
    }
    let thread = serde_json::to_string(&valid["threads"]["t-0001"]).unwrap();
    let mut header = valid.clone();
    header["threads"] = serde_json::json!({});
    let duplicate = serde_json::to_string(&header).unwrap().replace(
        "\"threads\":{}",
        &format!("\"threads\":{{\"t-0001\":{thread},\"t-0001\":{thread}}}"),
    );
    assert!(decode_snapshot(duplicate.as_bytes()).is_err());
}

#[test]
fn file_inventory_and_snapshot_sizes_are_bounded() {
    use questmancer::sources::herdr_projects::{MAX_SNAPSHOT_BYTES, MAX_THREADS, ReadError};
    let dir = fixture();
    fs::write(dir.path().join("PROJECT.md"), vec![b'x'; 256 * 1024 + 1]).unwrap();
    assert!(matches!(
        capture(&selection(dir.path()), Timestamp::from_millis(100)),
        Err(ReadError::Limit)
    ));
    assert!(matches!(
        decode_snapshot(&vec![b' '; MAX_SNAPSHOT_BYTES + 1]),
        Err(ReadError::Limit)
    ));
    let dir = fixture();
    let raw = fs::read(dir.path().join("threads/t-0001.toml")).unwrap();
    for n in 2..=MAX_THREADS + 1 {
        fs::write(dir.path().join(format!("threads/t-{n:04}.toml")), &raw).unwrap();
    }
    assert!(matches!(
        capture(&selection(dir.path()), Timestamp::from_millis(100)),
        Err(ReadError::Limit)
    ));
}

#[test]
fn captures_at_one_ten_and_one_hundred_are_deterministic_and_do_not_invent_work() {
    use questmancer::sources::herdr_projects::{Placement, RecordedGroup};
    for count in [1, 10, 100] {
        let dir = fixture();
        for n in 1..=count {
            fs::write(dir.path().join(format!("threads/t-{n:04}.toml")), format!("id = 't-{n:04}'\ntitle = 'Thread {n}'\nstatus = 'open'\nkind = 'checkout'\ncreated = '2026-09-28T10:00:00Z'\nlast_group = 'waiting-on-you'\n")).unwrap();
        }
        let a = capture(&selection(dir.path()), Timestamp::from_millis(100)).unwrap();
        let b = capture(&selection(dir.path()), Timestamp::from_millis(100)).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.threads.len(), count);
        assert_eq!(a.threads["t-0001"].placement, Placement::Checkout);
        assert_eq!(
            a.threads["t-0001"].recorded_group,
            Some(RecordedGroup::WaitingOnYou)
        );
        assert!(
            !a.capabilities.dependencies.is_supported()
                && !a.capabilities.structured_obligations.is_supported()
        );
    }
}

#[test]
fn source_writer_helper_process() {
    use std::{io::Write, time::Duration};
    let Ok(path) = std::env::var("QUESTMANCER_SOURCE_WRITER_TEST") else {
        return;
    };
    let root = Path::new(&path);
    let lock = fs::OpenOptions::new()
        .write(true)
        .open(root.join(".state/lock"))
        .unwrap();
    lock.lock().unwrap();
    fs::write(root.join("writer-ready"), "ready").unwrap();
    let started = std::time::Instant::now();
    while !root.join("writer-go").exists() {
        assert!(started.elapsed() < Duration::from_secs(10));
        std::thread::sleep(Duration::from_millis(1));
    }
    lock.unlock().unwrap();
    for n in 1..=40 {
        lock.lock().unwrap();
        let mut project = fs::File::create(root.join("project.tmp")).unwrap();
        write!(project, "+++\nname = 'Pair {n}'\ngoal = ''\n+++\n").unwrap();
        fs::rename(root.join("project.tmp"), root.join("PROJECT.md")).unwrap();
        std::thread::sleep(Duration::from_millis(1));
        fs::write(root.join("thread.tmp"), format!("id = 't-0001'\ntitle = 'Pair {n}'\nstatus = 'open'\nkind = 'tab'\ncreated = '2026-09-28T10:00:00Z'\n")).unwrap();
        fs::rename(root.join("thread.tmp"), root.join("threads/t-0001.toml")).unwrap();
        lock.unlock().unwrap();
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn cooperative_writer_process_cannot_publish_a_mixed_record_pair() {
    use questmancer::sources::herdr_projects::ReadError;
    use std::{
        process::Command,
        time::{Duration, Instant},
    };
    let dir = fixture();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "source_writer_helper_process"])
        .env("QUESTMANCER_SOURCE_WRITER_TEST", dir.path())
        .stdout(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let start = Instant::now();
    while !dir.path().join("writer-ready").exists() {
        if start.elapsed() > Duration::from_secs(10) {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("writer did not become ready");
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(matches!(
        capture(&selection(dir.path()), Timestamp::from_millis(100)),
        Err(ReadError::Busy)
    ));
    fs::write(dir.path().join("writer-go"), "go").unwrap();
    let mut pairs = 0;
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success());
            break;
        }
        if start.elapsed() > Duration::from_secs(10) {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("writer did not finish");
        }
        match capture(&selection(dir.path()), Timestamp::from_millis(100)) {
            Ok(snapshot) if snapshot.project.name.starts_with("Pair ") => {
                assert_eq!(snapshot.project.name, snapshot.threads["t-0001"].title);
                pairs += 1;
            }
            Ok(_) | Err(ReadError::Busy) => {}
            Err(error) => panic!("unexpected capture failure: {error}"),
        }
    }
    assert!(pairs > 0);
    let final_snapshot = capture(&selection(dir.path()), Timestamp::from_millis(100)).unwrap();
    assert_eq!(final_snapshot.project.name, "Pair 40");
}

#[test]
fn multiline_context_is_literal_and_terminal_controls_are_rejected() {
    let dir = fixture();
    fs::write(
        dir.path().join("PROJECT.md"),
        "+++\nname = 'Authentication'\ngoal = '''First line\nSecond line'''\n+++\n",
    )
    .unwrap();
    let path = dir.path().join("threads/t-0001.toml");
    let raw = fs::read_to_string(&path).unwrap();
    fs::write(
        &path,
        format!("{raw}error = '''First error\nSecond error'''\n"),
    )
    .unwrap();
    let snapshot = capture(&selection(dir.path()), Timestamp::from_millis(100)).unwrap();
    assert_eq!(snapshot.project.goal, "First line\nSecond line");
    assert_eq!(
        snapshot.threads["t-0001"].error,
        "First error\nSecond error"
    );
    let mut bad = serde_json::to_value(snapshot).unwrap();
    bad["project"]["goal"] = "Escape \u{1b}[31m".into();
    assert!(decode_snapshot(&serde_json::to_vec(&bad).unwrap()).is_err());
}

#[test]
fn missing_native_identity_cannot_be_upgraded_by_retention_or_record_reuse() {
    let dir = fixture();
    let before = capture(&selection(dir.path()), Timestamp::from_millis(100)).unwrap();
    let path = dir.path().join("threads/t-0001.toml");
    fs::remove_file(&path).unwrap();
    let absent = capture(&selection(dir.path()), Timestamp::from_millis(200)).unwrap();
    assert!(absent.threads.is_empty());
    assert_eq!(absent.coverage, Coverage::Partial);
    fs::write(&path, "id = 't-0001'\ntitle = 'Reused number'\nstatus = 'open'\nkind = 'adopted'\ncreated = '2026-09-28T10:00:00Z'\n").unwrap();
    let reused = capture(&selection(dir.path()), Timestamp::from_millis(300)).unwrap();
    assert_eq!(
        before.threads["t-0001"].created,
        reused.threads["t-0001"].created
    );
    assert!(
        !before.capabilities.durable_identity.is_supported()
            && !reused.capabilities.durable_identity.is_supported()
    );
}

fn canonical_snapshot() -> &'static questmancer::sources::herdr_projects::Snapshot {
    static SNAPSHOT: std::sync::OnceLock<questmancer::sources::herdr_projects::Snapshot> =
        std::sync::OnceLock::new();
    SNAPSHOT.get_or_init(|| {
        let dir = fixture();
        capture(&selection(dir.path()), Timestamp::from_millis(100)).unwrap()
    })
}
proptest::proptest! {
    #[test]
    fn duplicate_identity_never_selects_a_later_closure(number in 1000_u32..1_000_000) {
        let mut snapshot = canonical_snapshot().clone();
        let mut record = snapshot.threads.remove("t-0001").unwrap();
        let id = format!("t-{number}");
        record.id.clone_from(&id);
        record.status = ThreadStatus::Open;
        snapshot.threads.insert(id.clone(), record.clone());
        let mut valid = serde_json::to_value(&snapshot).unwrap();
        proptest::prop_assert!(decode_snapshot(&serde_json::to_vec(&valid).unwrap()).is_ok());
        let open = serde_json::to_string(&record).unwrap();
        record.status = ThreadStatus::Resolved;
        let closed = serde_json::to_string(&record).unwrap();
        valid["threads"] = serde_json::json!({});
        let duplicate = serde_json::to_string(&valid).unwrap().replace("\"threads\":{}", &format!("\"threads\":{{\"{id}\":{open},\"{id}\":{closed}}}"));
        proptest::prop_assert!(decode_snapshot(duplicate.as_bytes()).is_err());
    }
}

#[test]
fn frozen_snapshot_example_decodes_through_the_contract() {
    let snapshot = canonical_snapshot();
    if let Ok(path) = std::env::var("QUESTMANCER_HERDR_PROJECTS_EXPORT") {
        fs::write(path, serde_json::to_vec_pretty(snapshot).unwrap()).unwrap();
    }
    assert_eq!(
        decode_snapshot(&serde_json::to_vec(snapshot).unwrap()).unwrap(),
        *snapshot
    );
}
