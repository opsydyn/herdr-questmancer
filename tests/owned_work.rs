use questmancer::{
    app::{Model, View},
    domain::Timestamp,
    owned_work::{
        Action, Command, CommandError, CommandId, OwnedState, OwnedStore, StoreError, transition,
    },
    persistence::{PersistenceWorker, WorkerPaths, load_work_cache},
    sources::local_work::LocalWorkSource,
    work::{ArtifactId, ArtifactKind, CampaignId, ObligationId, QuestId, QuestKind, SourceId},
};

fn command(source: &SourceId, revision: u64, action: Action) -> Command {
    Command {
        id: CommandId::new(&format!("command-{revision}")),
        source: source.clone(),
        expected_revision: revision,
        actor: "Questmancer".into(),
        at: Timestamp::from_millis(1000 + i64::try_from(revision).unwrap()),
        action,
    }
}

fn tempdir() -> tempfile::TempDir {
    tempfile::tempdir_in(std::fs::canonicalize(std::env::temp_dir()).unwrap()).unwrap()
}
fn request(source: &SourceId, revision: u64) -> Command {
    command(
        source,
        revision,
        Action::RequestDecision {
            id: ObligationId::new(source, "decision"),
            campaign: CampaignId::new(source, "campaign"),
            quest: None,
            incarnation: "request-1".into(),
            prompt: "Keep compatibility?".into(),
            context: None,
        },
    )
}
fn answer(source: &SourceId, revision: u64) -> Command {
    command(
        source,
        revision,
        Action::AnswerDecision {
            id: ObligationId::new(source, "decision"),
            incarnation: "request-1".into(),
            answer: "Yes, for one release.".into(),
        },
    )
}
fn state_with_request() -> OwnedState {
    let s = SourceId::new("owned:test");
    let state = transition(&OwnedState::new(s.clone()).unwrap(), &create_campaign(&s))
        .unwrap()
        .state;
    transition(&state, &request(&s, 1)).unwrap().state
}

#[test]
fn retries_rejections_and_terminal_incarnations_preserve_authority() {
    let state = state_with_request();
    let source = state.source();
    let cmd = answer(source, state.revision());
    let accepted = transition(&state, &cmd).unwrap();
    let retried = transition(&accepted.state, &cmd).unwrap();
    assert!(retried.replayed);
    assert_eq!(retried.state, accepted.state);
    assert_eq!(retried.receipt, accepted.receipt);
    let mut changed = cmd.clone();
    changed.actor = "Another human".into();
    assert_eq!(
        transition(&accepted.state, &changed),
        Err(CommandError::RetryConflict)
    );
    assert_eq!(
        transition(&accepted.state, &answer(source, 3)),
        Err(CommandError::Conflict)
    );
    assert_eq!(
        transition(&accepted.state, &request(source, 3)),
        Err(CommandError::Conflict)
    );
    let mut wrong = answer(source, 2);
    wrong.id = CommandId::new("wrong-incarnation");
    if let Action::AnswerDecision { incarnation, .. } = &mut wrong.action {
        *incarnation = "older".into();
    }
    assert_eq!(transition(&state, &wrong), Err(CommandError::Reference));
    let mut foreign = cmd.clone();
    foreign.source = SourceId::new("projects:other");
    assert_eq!(
        transition(&state, &foreign),
        Err(CommandError::ForeignSource)
    );
    let mut target = cmd.clone();
    if let Action::AnswerDecision { id, .. } = &mut target.action {
        *id = ObligationId::new(&SourceId::new("owned:other"), "decision");
    }
    assert_eq!(
        transition(&state, &target),
        Err(CommandError::ForeignSource)
    );
    let mut obsolete = cmd;
    obsolete.expected_revision = 0;
    assert_eq!(transition(&state, &obsolete), Err(CommandError::Stale));
    assert!(
        state
            .answer(&ObligationId::new(source, "decision"))
            .is_none()
    );
}

#[test]
fn command_text_and_cross_campaign_references_are_checked_before_writes() {
    let source = SourceId::new("owned:test");
    let state = OwnedState::new(source.clone()).unwrap();
    for name in [" ".into(), "bad\u{001b}text".into(), "x".repeat(8193)] {
        let mut cmd = create_campaign(&source);
        if let Action::CreateCampaign { name: value, .. } = &mut cmd.action {
            *value = name;
        }
        assert_eq!(transition(&state, &cmd), Err(CommandError::Invalid));
    }
    let state = transition(&state, &create_campaign(&source)).unwrap().state;
    let bad = command(
        &source,
        1,
        Action::CreateQuest {
            id: QuestId::new(&source, "quest"),
            campaign: CampaignId::new(&source, "missing"),
            title: "Work".into(),
            description: None,
            kind: QuestKind::Ship,
        },
    );
    assert_eq!(transition(&state, &bad), Err(CommandError::Reference));
    let mut good = bad.clone();
    if let Action::CreateQuest { campaign, .. } = &mut good.action {
        *campaign = CampaignId::new(&source, "campaign");
    }
    let state = transition(&state, &good).unwrap().state;
    let other = command(
        &source,
        2,
        Action::CreateCampaign {
            id: CampaignId::new(&source, "other"),
            name: "Other".into(),
            objective: None,
        },
    );
    let state = transition(&state, &other).unwrap().state;
    let mut bad = request(&source, 3);
    if let Action::RequestDecision {
        campaign, quest, ..
    } = &mut bad.action
    {
        *campaign = CampaignId::new(&source, "other");
        *quest = Some(QuestId::new(&source, "quest"));
    }
    assert_eq!(transition(&state, &bad), Err(CommandError::Reference));
}

#[test]
fn evicted_receipt_cannot_reexecute_an_old_command() {
    let source = SourceId::new("owned:retention");
    let first = create_campaign(&source);
    let mut state = transition(&OwnedState::new(source.clone()).unwrap(), &first)
        .unwrap()
        .state;
    for n in 1..=128 {
        state = transition(
            &state,
            &command(
                &source,
                n,
                Action::CreateCampaign {
                    id: CampaignId::new(&source, &format!("campaign-{n}")),
                    name: format!("Campaign {n}"),
                    objective: None,
                },
            ),
        )
        .unwrap()
        .state;
    }
    assert!(state.receipt(&first.id).is_none());
    assert_eq!(transition(&state, &first), Err(CommandError::Stale));
    assert_eq!(state.inventory().campaigns.len(), 129);
}

#[test]
fn decision_request_attribution_survives_receipt_eviction_and_store_reopening() {
    let dir = tempdir();
    let path = dir.path().join("owned-work.json");
    let source = SourceId::new("owned:test");
    let mut store = OwnedStore::create(&path, source.clone()).unwrap();
    store.execute(&create_campaign(&source)).unwrap();
    let mut cmd = request(&source, 1);
    cmd.actor = "Alice".into();
    store.execute(&cmd).unwrap();
    for n in 2..130 {
        store
            .execute(&command(
                &source,
                n,
                Action::RegisterArtifact {
                    id: ArtifactId::new(&source, &format!("a-{n}")),
                    campaign: CampaignId::new(&source, "campaign"),
                    quest: None,
                    kind: ArtifactKind::Report,
                    title: "Report".into(),
                    uri: None,
                },
            ))
            .unwrap();
    }
    assert!(store.state().receipt(&cmd.id).is_none());
    drop(store);
    let store = OwnedStore::open(&path, &source).unwrap();
    assert_eq!(
        store
            .state()
            .decision_detail(&ObligationId::new(&source, "decision"))
            .unwrap()
            .requested_by,
        "Alice"
    );
    assert_eq!(
        store.state().inventory().obligations[&ObligationId::new(&source, "decision")].state,
        questmancer::work::ObligationState::Open
    );
}

#[test]
fn writer_lock_and_selected_identity_protect_existing_records() {
    let dir = tempdir();
    let path = dir.path().join("owned-work.json");
    let source = SourceId::new("owned:test");
    let mut store = OwnedStore::create(&path, source.clone()).unwrap();
    store.execute(&create_campaign(&source)).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert!(matches!(
        OwnedStore::open(&path, &source),
        Err(StoreError::Busy)
    ));
    assert!(matches!(
        OwnedStore::create(&path, source.clone()),
        Err(StoreError::Busy)
    ));
    drop(store);
    assert!(matches!(
        OwnedStore::open(&path, &SourceId::new("owned:wrong")),
        Err(StoreError::Selection)
    ));
    assert!(matches!(
        OwnedStore::create(&path, source.clone()),
        Err(StoreError::Selection)
    ));
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert!(OwnedStore::open(&path, &source).is_ok());
    let missing = dir.path().join("missing.json");
    assert!(matches!(
        OwnedStore::open(&missing, &source),
        Err(StoreError::Selection)
    ));
    assert!(!missing.exists());
}

#[test]
fn writer_lock_excludes_an_independent_process_and_releases_on_drop() {
    let dir = tempdir();
    let path = dir.path().join("owned-work.json");
    let source = SourceId::new("owned:process-lock");
    let store = OwnedStore::create(&path, source).unwrap();
    let probe = |expected| {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "owned_store_child_lock_probe", "--nocapture"])
            .env("QUESTMANCER_OWNED_LOCK_TEST_PATH", &path)
            .env("QUESTMANCER_OWNED_LOCK_TEST_EXPECTED", expected)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    };
    probe("busy");
    drop(store);
    probe("open");
}

#[test]
fn owned_store_child_lock_probe() {
    let Some(path) = std::env::var_os("QUESTMANCER_OWNED_LOCK_TEST_PATH") else {
        return;
    };
    let result = OwnedStore::open(
        std::path::Path::new(&path),
        &SourceId::new("owned:process-lock"),
    );
    if std::env::var("QUESTMANCER_OWNED_LOCK_TEST_EXPECTED").unwrap() == "busy" {
        assert!(matches!(result, Err(StoreError::Busy)));
    } else {
        assert!(result.is_ok(), "{result:?}");
    }
}

#[test]
fn invalid_future_duplicate_and_inconsistent_stores_are_preserved() {
    let dir = tempdir();
    let path = dir.path().join("owned-work.json");
    let source = SourceId::new("owned:test");
    let mut store = OwnedStore::create(&path, source.clone()).unwrap();
    store.execute(&create_campaign(&source)).unwrap();
    let original = std::fs::read_to_string(&path).unwrap();
    drop(store);
    let mut version: serde_json::Value = serde_json::from_str(&original).unwrap();
    version["version"] = 999.into();
    let mut wrong: serde_json::Value = serde_json::from_str(&original).unwrap();
    wrong["state"]["revision"] = 99.into();
    let duplicate = original.replacen("\"revision\":1", "\"revision\":1,\"revision\":1", 1);
    for contents in [
        "malformed".into(),
        version.to_string(),
        wrong.to_string(),
        duplicate,
    ] {
        std::fs::write(&path, &contents).unwrap();
        assert!(matches!(
            OwnedStore::open(&path, &source),
            Err(StoreError::Protected(_))
        ));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), contents);
    }
    let file = std::fs::File::create(&path).unwrap();
    file.set_len(16 * 1024 * 1024 + 1).unwrap();
    drop(file);
    assert!(matches!(
        OwnedStore::open(&path, &source),
        Err(StoreError::Protected(_))
    ));
    assert_eq!(
        std::fs::metadata(&path).unwrap().len(),
        16 * 1024 * 1024 + 1
    );
}

#[test]
fn replaced_store_lock_or_root_and_symlink_selections_fail_without_writing() {
    for replacement in ["store", "lock", "root"] {
        let dir = tempdir();
        let root = dir.path().join("home");
        std::fs::create_dir(&root).unwrap();
        let path = root.join("owned-work.json");
        let source = SourceId::new("owned:test");
        let mut store = OwnedStore::create(&path, source.clone()).unwrap();
        let original = std::fs::read(&path).unwrap();
        match replacement {
            "store" => {
                std::fs::rename(&path, root.join("previous.json")).unwrap();
                std::fs::write(&path, &original).unwrap();
            }
            "lock" => {
                std::fs::rename(root.join("owned-work.json.lock"), root.join("old.lock")).unwrap();
                std::fs::write(root.join("owned-work.json.lock"), "").unwrap();
            }
            _ => {
                std::fs::rename(&root, dir.path().join("old-root")).unwrap();
                std::fs::create_dir(&root).unwrap();
                std::fs::write(&path, &original).unwrap();
            }
        }
        assert!(store.execute(&create_campaign(&source)).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), original);
    }
    let dir = tempdir();
    let source = SourceId::new("owned:test");
    let target = dir.path().join("target.json");
    std::fs::write(&target, "untouched").unwrap();
    let link = dir.path().join("link.json");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    assert!(OwnedStore::create(&link, source.clone()).is_err());
    assert!(OwnedStore::open(&link, &source).is_err());
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "untouched");
    let root_link = dir.path().join("alias");
    std::os::unix::fs::symlink(dir.path(), &root_link).unwrap();
    assert!(OwnedStore::create(&root_link.join("new.json"), source).is_err());
    assert!(!dir.path().join("new.json").exists());
}

#[test]
fn rejected_and_identical_retried_commands_do_not_publish_again() {
    let dir = tempdir();
    let path = dir.path().join("owned-work.json");
    let source = SourceId::new("owned:test");
    let mut store = OwnedStore::create(&path, source.clone()).unwrap();
    let cmd = create_campaign(&source);
    let receipt = store.execute(&cmd).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(store.execute(&cmd).unwrap(), receipt);
    let mut bad = cmd;
    bad.actor = "Changed".into();
    assert!(matches!(
        store.execute(&bad),
        Err(StoreError::Command(CommandError::RetryConflict))
    ));
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(store.state().revision(), 1);
}

#[tokio::test]
async fn cache_failure_does_not_undo_answer_and_complete_reconciliation_recovers_it() {
    let dir = tempdir();
    let path = dir.path().join("owned-work.json");
    let source = SourceId::new("owned:test");
    let mut store = OwnedStore::create(&path, source.clone()).unwrap();
    store.execute(&create_campaign(&source)).unwrap();
    store.execute(&request(&source, 1)).unwrap();
    let paths = WorkerPaths::new(Some(dir.path().join("state.json")), None);
    let (client, _diagnostics, worker) = PersistenceWorker::start(paths.clone());
    let mut model = Model::new(View::Guild);
    let local = LocalWorkSource::new(store.state(), model.work()).unwrap();
    questmancer::work::runtime::accept(
        &client,
        &mut model,
        local
            .baseline(store.state(), Timestamp::from_millis(1))
            .unwrap(),
    )
    .await
    .unwrap();
    let previous = std::fs::read(paths.work_path().unwrap()).unwrap();
    store.execute(&answer(&source, 2)).unwrap();
    std::fs::write(paths.work_path().unwrap(), "{\"version\":999}").unwrap();
    assert!(
        questmancer::work::runtime::accept(
            &client,
            &mut model,
            local
                .baseline(store.state(), Timestamp::from_millis(2))
                .unwrap()
        )
        .await
        .is_err()
    );
    assert_eq!(model.work().open_decisions().len(), 1);
    assert_eq!(
        model.work().sources[&source].freshness,
        questmancer::work::Freshness::Reconcile
    );
    assert!(
        store
            .state()
            .answer(&ObligationId::new(&source, "decision"))
            .is_some()
    );
    assert_eq!(
        std::fs::read_to_string(paths.work_path().unwrap()).unwrap(),
        "{\"version\":999}",
        "protected cache is retained"
    );
    // Explicit test-owner repair, not automatic production clobbering.
    std::fs::write(paths.work_path().unwrap(), previous).unwrap();
    questmancer::work::runtime::accept(
        &client,
        &mut model,
        local
            .baseline(store.state(), Timestamp::from_millis(3))
            .unwrap(),
    )
    .await
    .unwrap();
    assert!(model.work().open_decisions().is_empty());
    assert_eq!(
        model.work().sources[&source].freshness,
        questmancer::work::Freshness::Current
    );
    client.shutdown().await.unwrap();
    worker.await.unwrap();
    drop(store);
    let store = OwnedStore::open(&path, &source).unwrap();
    assert_eq!(
        store
            .state()
            .answer(&ObligationId::new(&source, "decision"))
            .unwrap()
            .text,
        "Yes, for one release."
    );
}

#[test]
fn older_store_and_generation_overflow_cannot_become_current() {
    let state = state_with_request();
    let local = LocalWorkSource::new(&state, &questmancer::work::Projection::default()).unwrap();
    let mut projected = questmancer::work::reduce(
        questmancer::work::Projection::default(),
        local
            .baseline(&state, Timestamp::from_millis(1))
            .unwrap()
            .message,
    )
    .unwrap();
    let old = OwnedState::new(state.source().clone()).unwrap();
    assert!(matches!(
        LocalWorkSource::new(&old, &projected),
        Err(questmancer::sources::local_work::LocalSourceError::OlderStore)
    ));
    projected
        .sources
        .get_mut(state.source())
        .unwrap()
        .generation = u64::MAX;
    assert!(matches!(
        LocalWorkSource::new(&state, &projected),
        Err(questmancer::sources::local_work::LocalSourceError::Exhausted)
    ));
}

#[tokio::test]
async fn owned_complete_baselines_preserve_external_facts_intent_and_projects_checkpoint() {
    use questmancer::sources::herdr_projects::{Registration, Selection, capture, map_snapshot};
    let dir = tempdir();
    let upstream = dir.path().join("upstream");
    std::fs::create_dir(&upstream).unwrap();
    std::fs::create_dir(upstream.join(".state")).unwrap();
    std::fs::create_dir(upstream.join("threads")).unwrap();
    std::fs::write(
        upstream.join("PROJECT.md"),
        "+++\nname = 'External campaign'\n+++\n",
    )
    .unwrap();
    std::fs::write(
        upstream.join(".state/project.json"),
        "{\"status\":\"active\"}",
    )
    .unwrap();
    std::fs::write(upstream.join(".state/lock"), "").unwrap();
    std::fs::write(upstream.join("threads/t-0001.toml"), "id = 't-0001'\ntitle = 'External quest'\nstatus = 'open'\nkind = 'checkout'\ncreated = '2026-09-28T10:00:00Z'\n").unwrap();
    let external = SourceId::new("projects:synthetic");
    let reg = Registration::retained_records(external.clone(), "campaign").unwrap();
    let snapshot = capture(
        &Selection::new(external.clone(), "campaign", upstream).unwrap(),
        Timestamp::from_millis(1),
    )
    .unwrap();
    let paths = WorkerPaths::new(Some(dir.path().join("state.json")), None);
    let (client, _diagnostics, worker) = PersistenceWorker::start(paths.clone());
    let mut model = Model::new(View::Guild);
    let mut fixtures = questmancer::work::Projection::default();
    for msg in questmancer::work::fixture::FixtureSource::authentication(Timestamp::from_millis(1))
    {
        fixtures = questmancer::work::reduce(fixtures, msg).unwrap();
    }
    let obligation = fixtures.open_decisions()[0].clone();
    fixtures.acknowledge(&obligation, Some(Timestamp::from_millis(10000)));
    client.publish_work(fixtures.clone()).await.unwrap();
    model.replace_work(fixtures);
    let proposal = map_snapshot(&reg, 1, &snapshot, model.work(), None)
        .unwrap()
        .unwrap();
    let checkpoint = questmancer::work::runtime::accept_proposal(&client, &mut model, proposal)
        .await
        .unwrap();
    let prior = model.work().clone();
    let state = state_with_request();
    let local = LocalWorkSource::new(&state, model.work()).unwrap();
    questmancer::work::runtime::accept(
        &client,
        &mut model,
        local.baseline(&state, Timestamp::from_millis(2)).unwrap(),
    )
    .await
    .unwrap();
    let cached = load_work_cache(&paths.work_path().unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(cached.herdr_projects(&external), Some(&checkpoint));
    for (id, facts) in prior.sources {
        assert_eq!(model.work().sources[&id], facts);
    }
    assert_eq!(model.work().intent, prior.intent);
    assert_eq!(
        model.work().sources[state.source()].inventory,
        *state.inventory()
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(
            &std::fs::read(paths.work_path().unwrap()).unwrap()
        )
        .unwrap()["version"],
        2
    );
    client.shutdown().await.unwrap();
    worker.await.unwrap();
}

#[test]
fn owned_facts_reach_existing_board_and_table_without_domain_mutation() {
    use questmancer::{interaction::reduce_action, ui::input::Action as Input};
    let source = SourceId::new("owned:test");
    let state = transition(
        &OwnedState::new(source.clone()).unwrap(),
        &create_campaign(&source),
    )
    .unwrap()
    .state;
    let state = transition(
        &state,
        &command(
            &source,
            1,
            Action::CreateQuest {
                id: QuestId::new(&source, "quest"),
                campaign: CampaignId::new(&source, "campaign"),
                title: "Migrate API auth".into(),
                description: None,
                kind: QuestKind::Ship,
            },
        ),
    )
    .unwrap()
    .state;
    let mut cmd = request(&source, 2);
    if let Action::RequestDecision { quest, .. } = &mut cmd.action {
        *quest = Some(QuestId::new(&source, "quest"));
    }
    let state = transition(&state, &cmd).unwrap().state;
    let mut model = Model::new(View::Guild);
    let local = LocalWorkSource::new(&state, model.work()).unwrap();
    model.replace_work(
        questmancer::work::reduce(
            model.work().clone(),
            local
                .baseline(&state, Timestamp::from_millis(1))
                .unwrap()
                .message,
        )
        .unwrap(),
    );
    let prior = model.work().clone();
    let _ = reduce_action(&mut model, Input::NextUrgent);
    let render = |m: &Model, w, h| {
        let mut terminal =
            ratatui::Terminal::new(ratatui::backend::TestBackend::new(w, h)).unwrap();
        terminal
            .draw(|frame| {
                questmancer::ui::scene_overlays::render_scene_overlays(
                    frame,
                    m,
                    &questmancer::scene::presentation::ScenePresentation::from_model(m),
                    None,
                );
            })
            .unwrap();
        terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>()
    };
    let board = render(&model, 120, 38);
    assert!(board.contains("QUEST BOARD"));
    assert!(board.contains("Keep compatibility?"));
    let _ = reduce_action(&mut model, Input::OpenWarTable);
    let graph = questmancer::work::graph::CampaignGraph::from_model(&model).unwrap();
    assert_eq!(
        graph.nodes[&QuestId::new(&source, "quest")].human_requests,
        1
    );
    assert!(render(&model, 120, 38).contains("WAR TABLE"));
    for (w, h) in [(48, 28), (28, 12), (12, 6)] {
        let _ = render(&model, w, h);
    }
    assert_eq!(model.work(), &prior);
}

proptest::proptest! {
    #[test]
    fn command_sequences_preserve_revision_identity_answers_and_retry_effects(steps in proptest::collection::vec(0_u8..8, 1..30)) {
        use proptest::prelude::*;
        let mut state = state_with_request();
        let source = state.source().clone();
        let mut last: Option<Command> = None;
        let mut open: Option<ObligationId> = Some(ObligationId::new(&source, "decision"));
        for (n, step) in steps.into_iter().enumerate() {
            let before = state.clone();
            let revision = state.revision();
            let cmd = match step {
                0 => create_campaign(&source),
                1 => command(&source, revision, Action::RequestDecision { id: ObligationId::new(&source, &format!("d-{n}")), campaign: CampaignId::new(&source, "campaign"), quest: None, incarnation: format!("i-{n}"), prompt: "Choose".into(), context: None }),
                2 => {
                    let id = open.clone().unwrap_or_else(|| ObligationId::new(&source, "missing"));
                    let incarnation = state.inventory().obligations.get(&id).map_or("missing".into(), |o| o.incarnation.clone());
                    command(&source, revision, Action::AnswerDecision { id, incarnation, answer: "Accepted".into() })
                },
                3 => { let mut c = answer(&source, revision); c.source = SourceId::new("owned:foreign"); c },
                4 => last.clone().unwrap_or_else(|| create_campaign(&source)),
                5 => { let mut c = last.clone().unwrap_or_else(|| create_campaign(&source)); c.actor = "Different".into(); c },
                6 => command(&source, revision, Action::RegisterArtifact { id: ArtifactId::new(&source, &format!("a-{n}")), campaign: CampaignId::new(&source, "campaign"), quest: None, kind: ArtifactKind::Report, title: "Report".into(), uri: None }),
                _ => { state = serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap(); prop_assert_eq!(&state, &before); state.validate().unwrap(); continue; }
            };
            if let Ok(next) = transition(&state, &cmd) {
                    prop_assert_eq!(next.state.revision(), revision + u64::from(!next.replayed));
                    if next.replayed { prop_assert_eq!(&next.state, &before); }
                    else { match &cmd.action {
                        Action::RequestDecision { id, .. } => open = Some(id.clone()),
                        Action::AnswerDecision { id, .. } => { prop_assert_eq!(next.state.answer(id).unwrap().text.as_str(), "Accepted"); open = None; },
                        _ => (),
                    } }
                    let again = transition(&next.state, &cmd).unwrap();
                    prop_assert!(again.replayed); prop_assert_eq!(&again.state, &next.state); prop_assert_eq!(&again.receipt, &next.receipt);
                    last = Some(cmd); state = next.state;
            } else { prop_assert_eq!(&state, &before); }
            prop_assert_eq!(state.source(), &source);
            state.validate().unwrap();
            let projected = questmancer::work::reduce(questmancer::work::Projection::default(), LocalWorkSource::new(&state, &questmancer::work::Projection::default()).unwrap().baseline(&state, Timestamp::from_millis(1)).unwrap().message).unwrap();
            for o in state.inventory().obligations.values() {
                if state.answer(&o.id).is_some() { prop_assert!(!projected.open_decisions().contains(&o.id)); }
            }
            state = serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
        }
    }
}

fn create_campaign(source: &SourceId) -> Command {
    command(
        source,
        0,
        Action::CreateCampaign {
            id: CampaignId::new(source, "campaign"),
            name: "Replace authentication".into(),
            objective: Some("Preserve access".into()),
        },
    )
}

#[tokio::test]
// Keep the complete create/answer/artifact/restart transaction in one test.
#[allow(clippy::too_many_lines)]
async fn owned_records_answers_and_projection_survive_restart() {
    let dir = tempfile::tempdir_in(std::fs::canonicalize(std::env::temp_dir()).unwrap()).unwrap();
    let path = dir.path().join("owned-work.json");
    let source = SourceId::new("owned:test");
    let campaign = CampaignId::new(&source, "campaign");
    let quest = QuestId::new(&source, "quest");
    let decision = ObligationId::new(&source, "decision");
    let mut store = OwnedStore::create(&path, source.clone()).unwrap();
    store.execute(&create_campaign(&source)).unwrap();
    store
        .execute(&command(
            &source,
            1,
            Action::CreateQuest {
                id: quest.clone(),
                campaign: campaign.clone(),
                title: "Migrate API auth".into(),
                description: None,
                kind: QuestKind::Ship,
            },
        ))
        .unwrap();
    store
        .execute(&command(
            &source,
            2,
            Action::RequestDecision {
                id: decision.clone(),
                campaign: campaign.clone(),
                quest: Some(quest.clone()),
                incarnation: "request-1".into(),
                prompt: "Keep the old token endpoint?".into(),
                context: None,
            },
        ))
        .unwrap();

    let paths = WorkerPaths::new(Some(dir.path().join("state.json")), None);
    let (client, _diagnostics, worker) = PersistenceWorker::start(paths.clone());
    let mut model = Model::new(View::Guild);
    let local = LocalWorkSource::new(store.state(), model.work()).unwrap();
    questmancer::work::runtime::accept(
        &client,
        &mut model,
        local
            .baseline(store.state(), Timestamp::from_millis(2000))
            .unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(model.work().open_decisions(), vec![decision.clone()]);
    model.replace_work({
        let mut p = model.work().clone();
        p.acknowledge(&decision, None);
        p.acknowledge(&decision, Some(Timestamp::from_millis(9000)));
        p
    });
    client.publish_work(model.work().clone()).await.unwrap();
    assert_eq!(model.work().open_decisions(), vec![decision.clone()]);

    store
        .execute(&command(
            &source,
            3,
            Action::AnswerDecision {
                id: decision.clone(),
                incarnation: "request-1".into(),
                answer: "Keep it for one release.".into(),
            },
        ))
        .unwrap();
    store
        .execute(&command(
            &source,
            4,
            Action::RegisterArtifact {
                id: ArtifactId::new(&source, "artifact"),
                campaign,
                quest: Some(quest),
                kind: ArtifactKind::PullRequest,
                title: "PR #481".into(),
                uri: None,
            },
        ))
        .unwrap();
    questmancer::work::runtime::accept(
        &client,
        &mut model,
        local
            .baseline(store.state(), Timestamp::from_millis(3000))
            .unwrap(),
    )
    .await
    .unwrap();
    assert!(model.work().open_decisions().is_empty());
    assert!(questmancer::work::queue(&model).is_empty());
    let expected = store.state().clone();
    drop(store);
    let store = OwnedStore::open(&path, &source).unwrap();
    assert_eq!(store.state(), &expected);
    assert_eq!(
        store.state().answer(&decision).unwrap().text,
        "Keep it for one release."
    );
    let cached = load_work_cache(&paths.work_path().unwrap())
        .await
        .unwrap()
        .unwrap();
    model.replace_work(cached.projection);
    let local = LocalWorkSource::new(store.state(), model.work()).unwrap();
    questmancer::work::runtime::accept(
        &client,
        &mut model,
        local
            .baseline(store.state(), Timestamp::from_millis(4000))
            .unwrap(),
    )
    .await
    .unwrap();
    assert!(model.work().open_decisions().is_empty());
    assert_eq!(model.work().sources[&source].inventory.artifacts.len(), 1);
    assert!(model.work().sources[&source].bindings.is_empty());
    client.shutdown().await.unwrap();
    worker.await.unwrap();
}
