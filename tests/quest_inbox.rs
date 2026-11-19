use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use questmancer::{
    app::{ConnectionState, Modal, Model, View},
    command::AgentCommand,
    domain::{DomainState, Timestamp},
    herdr::{
        protocol::{AgentSessionInfo, SessionSnapshotResult, SuccessResponse},
        supervisor::ConnectionUpdate,
    },
    interaction::{ActionReduction, reduce_action},
    owned_work::{
        Action, CheckReceipt, CheckResult, Command, CommandId, OwnedRuntime, OwnedStore,
        StoreSelection, authoring,
    },
    persistence::{PersistenceClient, PersistenceWorker, WorkerPaths, load_work_cache},
    runtime_loop::{apply_connection_update, dispatch_action_effects},
    ui::input::action_for_event_in,
    work::{CampaignId, ObligationState, QuestId, QuestKind, QuestState, SourceId, WorkSelection},
};
use std::{path::Path, process::Command as Process, time::Duration};

fn git(repo: &Path, arguments: &[&str]) -> String {
    let result = Process::new("git")
        .current_dir(repo)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .args([
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.test",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=/dev/null",
        ])
        .args(arguments)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    String::from_utf8(result.stdout).unwrap().trim().to_owned()
}

fn repo(path: &Path) -> (String, String) {
    std::fs::create_dir(path).unwrap();
    git(path, &["init", "--quiet"]);
    git(path, &["commit", "--allow-empty", "-m", "Base"]);
    let base = git(path, &["rev-parse", "HEAD"]);
    std::fs::write(path.join("keyboard.txt"), "press i\n").unwrap();
    git(path, &["add", "keyboard.txt"]);
    git(path, &["commit", "-m", "Add shortcut"]);
    (base, git(path, &["rev-parse", "HEAD"]))
}

fn seed(selection: &StoreSelection) -> QuestId {
    let campaign = CampaignId::new(&selection.source, "guild");
    let quest = QuestId::new(&selection.source, "inbox");
    let mut store = OwnedStore::create(&selection.path, selection.source.clone()).unwrap();
    for (revision, action) in [
        Action::CreateCampaign {
            id: campaign.clone(),
            name: "Guild".into(),
            objective: None,
        },
        Action::CreateQuest {
            id: quest.clone(),
            campaign,
            title: "Make the Inbox reachable".into(),
            description: None,
            kind: QuestKind::Ship,
        },
    ]
    .into_iter()
    .enumerate()
    {
        store
            .execute(&Command {
                id: CommandId::new(&format!("seed-{revision}")),
                source: selection.source.clone(),
                expected_revision: revision as u64,
                actor: "Questmancer".into(),
                at: Timestamp::from_millis(1),
                action,
            })
            .unwrap();
    }
    assert_eq!(version(&selection.path), 1);
    quest
}

fn version(path: &Path) -> u64 {
    serde_json::from_slice::<serde_json::Value>(&std::fs::read(path).unwrap()).unwrap()["version"]
        .as_u64()
        .unwrap()
}

fn key(model: &mut Model, code: KeyCode) -> Option<ActionReduction> {
    let event = Event::Key(KeyEvent::new(code, KeyModifiers::NONE));
    if authoring::handle_event(model, &event) {
        return None;
    }
    let action = action_for_event_in(&event, model.modal());
    Some(reduce_action(model, action))
}

fn paste(model: &mut Model, text: &str) {
    assert!(authoring::handle_event(model, &Event::Paste(text.into())));
}

async fn apply(runtime: &mut OwnedRuntime, client: &mut PersistenceClient, model: &mut Model) {
    runtime.dispatch(model);
    tokio::time::timeout(Duration::from_secs(5), runtime.consume_next(client, model))
        .await
        .unwrap();
}

fn render(model: &Model, width: u16, height: u16) -> String {
    let backend = ratatui::backend::TestBackend::new(width, height);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| questmancer::ui::work_board::render(frame, model))
        .unwrap();
    terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect()
}

fn open_review(model: &mut Model) {
    let _ = key(model, KeyCode::Char('b'));
    let _ = key(model, KeyCode::Char('i'));
    assert_eq!(model.modal(), &Modal::QuestInbox { detail: false });
    let _ = key(model, KeyCode::Enter);
    assert_eq!(model.modal(), &Modal::QuestInbox { detail: true });
}

fn association_snapshot() -> questmancer::herdr::protocol::SessionSnapshot {
    let mut response: SuccessResponse<SessionSnapshotResult> =
        serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
    response.result.snapshot.agents[0].agent_session = Some(AgentSessionInfo {
        source: "test".into(),
        agent: "codex".into(),
        kind: "session".into(),
        value: "private-test-session".into(),
    });
    response.result.snapshot
}

#[tokio::test]
async fn board_quest_lifecycle_requires_confirmation_and_survives_restart() {
    let directory =
        tempfile::tempdir_in(std::fs::canonicalize(std::env::temp_dir()).unwrap()).unwrap();
    let selection = StoreSelection::new(
        directory.path().join("owned-work.json"),
        SourceId::new("owned:lifecycle"),
    )
    .unwrap();
    let quest = seed(&selection);
    let paths = WorkerPaths::new(Some(directory.path().join("state.json")), None);
    let (mut client, _, worker) = PersistenceWorker::start(paths.clone());
    let mut model = Model::new(View::Guild);
    let mut runtime = OwnedRuntime::start(&paths, &mut model).unwrap();
    model
        .authoring_mut()
        .request_setup(selection.clone(), false);
    apply(&mut runtime, &mut client, &mut model).await;
    for (from, to) in [
        (QuestState::Planned, QuestState::Active),
        (QuestState::Active, QuestState::Completed),
        (QuestState::Completed, QuestState::Active),
    ] {
        model.select_work(WorkSelection::Quest(quest.clone()));
        let original = std::fs::read(&selection.path).unwrap();
        let _ = key(&mut model, KeyCode::Char('t'));
        assert_eq!(model.modal(), &Modal::OwnedAuthoring);
        let preview = render(&model, 60, 24);
        assert!(preview.contains(&format!("{from:?} → {to:?}")), "{preview}");
        let _ = key(&mut model, KeyCode::Tab);
        let _ = key(&mut model, KeyCode::Enter);
        assert_eq!(std::fs::read(&selection.path).unwrap(), original);
        assert!(model.authoring().ready());
        let _ = key(&mut model, KeyCode::BackTab);
        let _ = key(&mut model, KeyCode::Down);
        let _ = key(&mut model, KeyCode::Tab);
        let _ = key(&mut model, KeyCode::Enter);
        apply(&mut runtime, &mut client, &mut model).await;
        assert!(model.authoring().ready(), "{}", model.authoring().feedback);
        assert_eq!(version(&selection.path), 3);
        assert_eq!(model.work().quest(&quest).unwrap().state, to);
        let _ = key(&mut model, KeyCode::Esc);
        assert_eq!(model.modal(), &Modal::QuestBoard);
        assert!(render(&model, 60, 24).contains("t: state"));
        assert!(render(&model, 90, 28).contains(&format!("{to:?}")));
        let _ = key(&mut model, KeyCode::Char('w'));
        let graph = questmancer::work::graph::CampaignGraph::from_model(&model).unwrap();
        assert_eq!(graph.nodes[&quest].state, to);
    }
    runtime.shutdown(&mut client, &mut model).await.unwrap();
    client.shutdown().await.unwrap();
    worker.await.unwrap();
    let reopened = OwnedStore::open(&selection.path, &selection.source).unwrap();
    assert_eq!(
        reopened.state().inventory().quests[&quest].state,
        QuestState::Active
    );
    let cache = load_work_cache(&paths.work_path().unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        cache.projection.quest(&quest).unwrap().state,
        QuestState::Active
    );
}

#[tokio::test]
async fn board_edits_and_reopens_owned_prerequisites() {
    let directory =
        tempfile::tempdir_in(std::fs::canonicalize(std::env::temp_dir()).unwrap()).unwrap();
    let source = SourceId::new("owned:promo");
    let selection =
        StoreSelection::new(directory.path().join("owned-work.json"), source.clone()).unwrap();
    let original = include_bytes!("fixtures/owned-work/pre-lifecycle-v2.json");
    std::fs::write(&selection.path, original).unwrap();
    let dependent = QuestId::new(&source, "picker");
    let prerequisite = QuestId::new(&source, "web");
    let edge = (prerequisite.clone(), dependent.clone());
    let paths = WorkerPaths::new(Some(directory.path().join("state.json")), None);
    let (mut client, _, worker) = PersistenceWorker::start(paths.clone());
    let mut model = Model::new(View::Guild);
    let mut runtime = OwnedRuntime::start(&paths, &mut model).unwrap();
    model
        .authoring_mut()
        .request_setup(selection.clone(), false);
    apply(&mut runtime, &mut client, &mut model).await;
    assert_eq!(std::fs::read(&selection.path).unwrap(), original);

    for (operation, present) in [("Add", true), ("Remove", false)] {
        model.select_work(WorkSelection::Quest(dependent.clone()));
        let original = std::fs::read(&selection.path).unwrap();
        let _ = key(&mut model, KeyCode::Char('p'));
        assert_eq!(model.modal(), &Modal::OwnedAuthoring);
        let _ = key(&mut model, KeyCode::Tab);
        let _ = key(&mut model, KeyCode::Enter);
        assert_eq!(std::fs::read(&selection.path).unwrap(), original);
        assert!(model.authoring().ready());
        let _ = key(&mut model, KeyCode::BackTab);
        let _ = key(&mut model, KeyCode::Down);
        let preview = render(&model, 60, 20);
        assert!(
            preview.contains(&format!("{operation} Web migration")),
            "{preview}"
        );
        let _ = key(&mut model, KeyCode::Tab);
        let _ = key(&mut model, KeyCode::Enter);
        apply(&mut runtime, &mut client, &mut model).await;
        assert!(model.authoring().ready(), "{}", model.authoring().feedback);
        assert_eq!(version(&selection.path), 4);
        let _ = key(&mut model, KeyCode::Esc);
        assert_eq!(model.modal(), &Modal::QuestBoard);
        assert_eq!(
            render(&model, 90, 28).contains("Dependency: Web migration | Planned"),
            present
        );
        let _ = key(&mut model, KeyCode::Char('w'));
        let graph = questmancer::work::graph::CampaignGraph::from_model(&model).unwrap();
        assert_eq!(graph.edges.contains(&edge), present);
        assert!(
            !graph
                .edges
                .contains(&(dependent.clone(), prerequisite.clone()))
        );
    }
    runtime.shutdown(&mut client, &mut model).await.unwrap();
    client.shutdown().await.unwrap();
    worker.await.unwrap();
    let reopened = OwnedStore::open(&selection.path, &source).unwrap();
    let quest = &reopened.state().inventory().quests[&dependent];
    assert!(quest.dependencies.is_empty());
    assert_eq!(quest.state, QuestState::Planned);
    let cache = load_work_cache(&paths.work_path().unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(cache.projection.quest(&dependent), Some(quest));
}

#[tokio::test]
async fn board_records_replaces_and_clears_a_local_assignment() {
    let directory =
        tempfile::tempdir_in(std::fs::canonicalize(std::env::temp_dir()).unwrap()).unwrap();
    let source = SourceId::new("owned:assignment");
    let selection =
        StoreSelection::new(directory.path().join("owned-work.json"), source.clone()).unwrap();
    let quest = seed(&selection);
    let paths = WorkerPaths::new(Some(directory.path().join("state.json")), None);
    let (mut client, _, worker) = PersistenceWorker::start(paths.clone());
    let mut model = Model::new(View::Guild);
    let response: SuccessResponse<SessionSnapshotResult> =
        serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
    let mut domain =
        DomainState::from_snapshot(&response.result.snapshot, Timestamp::from_millis(1));
    let adventurer = domain.agents.values_mut().next().unwrap();
    "Release reviewer".clone_into(&mut adventurer.name);
    "Release reviewer".clone_into(&mut adventurer.persona.name);
    model.replace_domain(domain);
    model.set_connection(ConnectionState::Connected);
    let mut runtime = OwnedRuntime::start(&paths, &mut model).unwrap();
    model
        .authoring_mut()
        .request_setup(selection.clone(), false);
    apply(&mut runtime, &mut client, &mut model).await;

    let mut previous = "Unassigned";
    for name in [Some("Release reviewer"), Some("Mira"), None] {
        model.select_work(WorkSelection::Quest(quest.clone()));
        assert!(render(&model, 90, 28).contains("y: assign"));
        let original = std::fs::read(&selection.path).unwrap();
        let _ = key(&mut model, KeyCode::Char('y'));
        assert_eq!(model.modal(), &Modal::OwnedAuthoring);
        let form = render(&model, 60, 20);
        assert!(
            form.contains(&format!("Current assignee: {previous}")),
            "{form}"
        );
        for code in [KeyCode::Tab, KeyCode::Tab, KeyCode::Enter] {
            let _ = key(&mut model, code);
        }
        assert!(model.authoring().ready());
        assert_eq!(std::fs::read(&selection.path).unwrap(), original);
        for code in [KeyCode::BackTab, KeyCode::BackTab, KeyCode::Down] {
            let _ = key(&mut model, code);
        }
        if name.is_none() {
            let _ = key(&mut model, KeyCode::Down);
        }
        let _ = key(&mut model, KeyCode::Tab);
        if let Some(name) = name {
            paste(&mut model, name);
        }
        let _ = key(&mut model, KeyCode::Tab);
        let _ = key(&mut model, KeyCode::Enter);
        apply(&mut runtime, &mut client, &mut model).await;
        assert!(model.authoring().ready(), "{}", model.authoring().feedback);
        assert_eq!(version(&selection.path), 5);
        let _ = key(&mut model, KeyCode::Esc);
        assert_eq!(model.modal(), &Modal::QuestBoard);
        let board = render(&model, 90, 28);
        previous = name.unwrap_or("Unassigned");
        assert!(
            board.contains(&format!("Recorded assignee: {previous}")),
            "{board}"
        );
        assert!(board.contains("Observe unavailable"), "{board}");
        assert!(questmancer::work::bound_agent(&model, &quest).is_none());
        assert!(key(&mut model, KeyCode::Enter).unwrap().commands.is_empty());
        let _ = key(&mut model, KeyCode::Char('w'));
        let graph = questmancer::work::graph::CampaignGraph::from_model(&model).unwrap();
        assert_eq!(graph.nodes[&quest].assignment.as_deref(), name);
        assert_eq!(graph.nodes[&quest].state, QuestState::Planned);
    }
    runtime.shutdown(&mut client, &mut model).await.unwrap();
    client.shutdown().await.unwrap();
    worker.await.unwrap();
    let reopened = OwnedStore::open(&selection.path, &source).unwrap();
    let persisted = &reopened.state().inventory().quests[&quest];
    assert!(persisted.assignment.is_none());
    assert_eq!(persisted.revision, 5);
    let cache = load_work_cache(&paths.work_path().unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(cache.projection.quest(&quest), Some(persisted));
}

#[tokio::test]
async fn owned_quest_association_enables_verified_observe_without_writing_a_binding() {
    let directory =
        tempfile::tempdir_in(std::fs::canonicalize(std::env::temp_dir()).unwrap()).unwrap();
    let source = SourceId::new("owned:promo");
    let selection =
        StoreSelection::new(directory.path().join("owned-work.json"), source.clone()).unwrap();
    let original = include_bytes!("fixtures/owned-work/pre-lifecycle-v2.json");
    std::fs::write(&selection.path, original).unwrap();
    let quest = QuestId::new(&source, "picker");
    let paths = WorkerPaths::new(Some(directory.path().join("state.json")), None);
    let (mut client, _, worker) = PersistenceWorker::start(paths.clone());
    let mut model = Model::new(View::Guild);
    let mut runtime = OwnedRuntime::start(&paths, &mut model).unwrap();
    model
        .authoring_mut()
        .request_setup(selection.clone(), false);
    apply(&mut runtime, &mut client, &mut model).await;
    let _ = apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(association_snapshot()),
        Timestamp::from_millis(2),
    );
    let agent = model.selected_agent().unwrap().clone();
    let expected = AgentCommand::FocusVerifiedPane {
        pane_id: agent.pane_id.clone(),
        identity: agent.capture_identity.clone(),
    };
    model.select_work(WorkSelection::Quest(quest.clone()));
    let cache_path = paths.work_path().unwrap();
    let cached = std::fs::read(&cache_path).unwrap();
    assert!(key(&mut model, KeyCode::Enter).unwrap().commands.is_empty());
    let linked = key(&mut model, KeyCode::Char('l')).unwrap();
    assert!(linked.commands.is_empty() && linked.persistence.is_empty());
    assert!(render(&model, 120, 30).contains("Live association"));
    assert_eq!(
        key(&mut model, KeyCode::Enter).unwrap().commands,
        std::slice::from_ref(&expected)
    );
    model.select_work(WorkSelection::Quest(quest.clone()));
    let _ = key(&mut model, KeyCode::Char('i'));
    assert_eq!(
        questmancer::work::view::selected_quest(&model),
        Some(&quest)
    );
    assert_eq!(
        key(&mut model, KeyCode::Char('o')).unwrap().commands,
        std::slice::from_ref(&expected)
    );
    model.select_work(WorkSelection::Quest(quest.clone()));
    let _ = key(&mut model, KeyCode::Char('w'));
    assert_eq!(
        key(&mut model, KeyCode::Enter).unwrap().commands,
        std::slice::from_ref(&expected)
    );
    assert_eq!(std::fs::read(&selection.path).unwrap(), original);
    assert_eq!(std::fs::read(&cache_path).unwrap(), cached);

    // A new acknowledged owned revision preserves this connection's link.
    model.select_work(WorkSelection::Quest(quest.clone()));
    let _ = key(&mut model, KeyCode::Char('y'));
    for code in [KeyCode::Down, KeyCode::Tab] {
        let _ = key(&mut model, code);
    }
    paste(&mut model, "Release reviewer");
    for code in [KeyCode::Tab, KeyCode::Enter] {
        let _ = key(&mut model, code);
    }
    apply(&mut runtime, &mut client, &mut model).await;
    let _ = key(&mut model, KeyCode::Esc);
    assert_eq!(
        key(&mut model, KeyCode::Enter).unwrap().commands,
        [expected]
    );
    let saved = std::fs::read_to_string(&selection.path).unwrap();
    let cached = std::fs::read_to_string(&cache_path).unwrap();
    assert!(!saved.contains("private-test-session") && !cached.contains("private-test-session"));
    assert!(
        load_work_cache(&cache_path)
            .await
            .unwrap()
            .unwrap()
            .projection
            .sources[&source]
            .bindings
            .is_empty()
    );

    let _ = key(&mut model, KeyCode::Char('L'));
    assert!(key(&mut model, KeyCode::Enter).unwrap().commands.is_empty());
    runtime.shutdown(&mut client, &mut model).await.unwrap();
    client.shutdown().await.unwrap();
    worker.await.unwrap();
}

#[test]
fn owned_quest_association_requires_current_unambiguous_evidence_and_never_revives() {
    use questmancer::{
        domain::{AgentKey, CaptureIdentity, PaneId, Presence},
        sources::local_work::LocalWorkSource,
        work::{Freshness, Projection, reduce},
    };
    let directory =
        tempfile::tempdir_in(std::fs::canonicalize(std::env::temp_dir()).unwrap()).unwrap();
    let source = SourceId::new("owned:association");
    let selection =
        StoreSelection::new(directory.path().join("owned-work.json"), source.clone()).unwrap();
    let quest = seed(&selection);
    let store = OwnedStore::open(&selection.path, &source).unwrap();
    let projection = Projection::default();
    let baseline = LocalWorkSource::new(store.state(), &projection)
        .unwrap()
        .baseline(store.state(), Timestamp::from_millis(1))
        .unwrap();
    let work = reduce(projection, baseline.message).unwrap();
    let mut model = Model::new(View::Guild);
    model.authoring_mut().selection = Some(selection);
    model.replace_work(work.clone());
    let _ = apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(association_snapshot()),
        Timestamp::from_millis(2),
    );
    model.select_work(WorkSelection::Quest(quest));
    let original = model.domain().clone();
    let candidate = model.selected_agent().unwrap().key.clone();
    let can_observe = |model: &mut Model| !key(model, KeyCode::Enter).unwrap().commands.is_empty();
    let link = |model: &mut Model| {
        let _ = key(model, KeyCode::Char('l'));
    };
    for (invalid_domain, link_allowed) in [
        ("same pane, different terminal", true),
        ("moved pane", true),
        ("unqualified report", false),
        ("exited", false),
        ("ambiguous pane", false),
        ("missing agent", false),
    ] {
        link(&mut model);
        assert!(can_observe(&mut model));
        let mut changed = original.clone();
        let agent = changed.agents.get_mut(&candidate).unwrap();
        match invalid_domain {
            "same pane, different terminal" => {
                let CaptureIdentity::Session { terminal, .. } = &mut agent.capture_identity else {
                    panic!("fixture must provide complete session evidence");
                };
                *terminal = "replacement-terminal".into();
            }
            "moved pane" => agent.pane_id = PaneId::new("moved:pane"),
            "unqualified report" => agent.capture_identity = CaptureIdentity::Unqualified,
            "exited" => agent.presence = Presence::Exited,
            "ambiguous pane" => {
                let mut another = agent.clone();
                another.key = AgentKey::new("ambiguous-agent");
                changed.agents.insert(another.key.clone(), another);
            }
            "missing agent" => {
                changed.agents.remove(&candidate);
            }
            _ => unreachable!("all invalid domain scenarios are declared"),
        }
        model.replace_domain(changed);
        assert!(!can_observe(&mut model), "{invalid_domain}");
        if !link_allowed {
            link(&mut model);
            assert!(!can_observe(&mut model), "{invalid_domain}");
        }
        model.replace_domain(original.clone());
        assert!(
            !can_observe(&mut model),
            "{invalid_domain} revived an old link"
        );
    }
    link(&mut model);
    let pane = model.selected_agent().unwrap().pane_id.clone();
    model.set_managed_pane_id(Some(pane));
    link(&mut model);
    assert!(!can_observe(&mut model));
    model.set_managed_pane_id(None);
    assert!(!can_observe(&mut model));
    link(&mut model);
    let mut cached = work.clone();
    cached.sources.get_mut(&source).unwrap().freshness = Freshness::Cached;
    model.replace_work(cached);
    link(&mut model);
    assert!(!can_observe(&mut model));
    model.replace_work(work);
    assert!(!can_observe(&mut model));
    link(&mut model);
    model.set_connection(ConnectionState::Offline);
    link(&mut model);
    assert!(!can_observe(&mut model));
    model.set_connection(ConnectionState::Connected);
    assert!(!can_observe(&mut model));
}

#[tokio::test]
#[allow(clippy::too_many_lines)] // End-to-end Board input, Git evidence, owner and cache reopen.
async fn board_inbox_rejects_stale_evidence_and_reopens_defer_and_settlement() {
    let directory =
        tempfile::tempdir_in(std::fs::canonicalize(std::env::temp_dir()).unwrap()).unwrap();
    let repository = directory.path().join("checkout");
    let (base, head) = repo(&repository);
    let selection = StoreSelection::new(
        directory.path().join("owned-work.json"),
        SourceId::new("owned:inbox"),
    )
    .unwrap();
    let quest = seed(&selection);
    let paths = WorkerPaths::new(Some(directory.path().join("state.json")), None);
    let (mut client, _, worker) = PersistenceWorker::start(paths.clone());
    let mut model = Model::new(View::Guild);
    model.set_now(Timestamp::from_millis(10_000));
    let mut runtime = OwnedRuntime::start(&paths, &mut model).unwrap();
    model
        .authoring_mut()
        .request_setup(selection.clone(), false);
    apply(&mut runtime, &mut client, &mut model).await;
    let _ = key(&mut model, KeyCode::Char('b'));
    assert!(render(&model, 45, 18).contains("i: Inbox"));
    let _ = key(&mut model, KeyCode::Char('v'));
    assert_eq!(
        model.authoring().surface,
        authoring::Surface::Form(authoring::FormKind::RequestReview)
    );
    for value in [
        "Keyboard can reach the Inbox",
        repository.to_str().unwrap(),
        &base,
        &head,
        "keyboard check",
        "review operator",
    ] {
        paste(&mut model, value);
        let _ = key(&mut model, KeyCode::Tab);
    }
    let _ = key(&mut model, KeyCode::Down);
    let _ = key(&mut model, KeyCode::Tab);
    paste(&mut model, "Pi adds the new keyboard shortcut.");
    let _ = key(&mut model, KeyCode::Tab);
    let _ = key(&mut model, KeyCode::Enter);
    apply(&mut runtime, &mut client, &mut model).await;
    assert!(model.authoring().ready(), "{}", model.authoring().feedback);
    assert_eq!(version(&selection.path), 2);
    let _ = key(&mut model, KeyCode::Esc);
    let _ = key(&mut model, KeyCode::Char('i'));
    assert!(render(&model, 80, 24).contains("1 quest due"));
    let _ = key(&mut model, KeyCode::Enter);
    apply(&mut runtime, &mut client, &mut model).await;
    assert!(render(&model, 100, 30).contains("A keyboard.txt"));
    assert!(render(&model, 100, 30).contains("Pi report"));
    assert!(
        key(&mut model, KeyCode::Char('o'))
            .unwrap()
            .commands
            .is_empty()
    );
    let _ = key(&mut model, KeyCode::Char('x'));
    assert_eq!(model.modal(), &Modal::QuestInbox { detail: true });

    let receipt = |head: String| {
        serde_json::to_string(&CheckReceipt {
            repository: repository.clone(),
            head,
            recipe: "keyboard check".into(),
            runner: "review operator".into(),
            captured_at: Timestamp::from_millis(9_000),
            result: CheckResult::Passed,
        })
        .unwrap()
    };
    let _ = key(&mut model, KeyCode::Char('c'));
    paste(&mut model, &receipt(base));
    let _ = key(&mut model, KeyCode::Tab);
    let _ = key(&mut model, KeyCode::Enter);
    apply(&mut runtime, &mut client, &mut model).await;
    assert!(model.authoring().feedback.contains("does not match"));
    assert_eq!(model.authoring().metadata.as_ref().unwrap().revision, 3);
    let _ = key(&mut model, KeyCode::BackTab);
    assert!(authoring::handle_event(
        &mut model,
        &Event::Key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL))
    ));
    paste(&mut model, &receipt(head));
    let _ = key(&mut model, KeyCode::Tab);
    let _ = key(&mut model, KeyCode::Enter);
    apply(&mut runtime, &mut client, &mut model).await;
    let _ = key(&mut model, KeyCode::Esc);
    let seen = key(&mut model, KeyCode::Char(' ')).unwrap();
    assert!(
        dispatch_action_effects(&mut client, &mut model, seen)
            .await
            .persistence_errors
            .is_empty()
    );
    assert_eq!(model.work().eligible_obligations(model.now()).len(), 1);
    assert!(render(&model, 80, 24).contains("Seen"));
    let deferred = key(&mut model, KeyCode::Char('s')).unwrap();
    assert!(
        dispatch_action_effects(&mut client, &mut model, deferred)
            .await
            .persistence_errors
            .is_empty()
    );
    assert!(model.work().eligible_obligations(model.now()).is_empty());
    runtime.shutdown(&mut client, &mut model).await.unwrap();
    client.shutdown().await.unwrap();
    worker.await.unwrap();

    let (mut client, _, worker) = PersistenceWorker::start(paths.clone());
    let mut model = Model::new(View::Guild);
    model.set_now(Timestamp::from_millis(11_000));
    model.replace_work(
        load_work_cache(&paths.work_path().unwrap())
            .await
            .unwrap()
            .unwrap()
            .projection,
    );
    let mut runtime = OwnedRuntime::start(&paths, &mut model).unwrap();
    model
        .authoring_mut()
        .request_setup(selection.clone(), false);
    apply(&mut runtime, &mut client, &mut model).await;
    open_review(&mut model);
    apply(&mut runtime, &mut client, &mut model).await;
    assert!(render(&model, 40, 16).contains("Deferred to"));
    assert!(model.work().eligible_obligations(model.now()).is_empty());
    model.set_now(Timestamp::from_millis(10_000).plus(Model::SNOOZE));
    assert_eq!(model.work().eligible_obligations(model.now()).len(), 1);
    let _ = key(&mut model, KeyCode::Char('x'));
    assert_eq!(
        model.authoring().surface,
        authoring::Surface::Form(authoring::FormKind::SettleReview)
    );
    let _ = key(&mut model, KeyCode::Down);
    let _ = key(&mut model, KeyCode::Tab);
    let cache_path = paths.work_path().unwrap();
    let good_cache = std::fs::read(&cache_path).unwrap();
    std::fs::write(&cache_path, b"{\"version\":999}").unwrap();
    let _ = key(&mut model, KeyCode::Enter);
    apply(&mut runtime, &mut client, &mut model).await;
    assert_eq!(model.authoring().phase, authoring::Phase::ViewPending);
    let committed = std::fs::read(&selection.path).unwrap();
    assert_eq!(model.work().eligible_obligations(model.now()).len(), 1);
    std::fs::write(&cache_path, good_cache).unwrap();
    let _ = key(&mut model, KeyCode::Esc);
    let _ = key(&mut model, KeyCode::Char('a'));
    for _ in 0..10 {
        let _ = key(&mut model, KeyCode::Down);
    }
    let _ = key(&mut model, KeyCode::Enter);
    apply(&mut runtime, &mut client, &mut model).await;
    assert!(model.work().eligible_obligations(model.now()).is_empty());
    assert_eq!(std::fs::read(&selection.path).unwrap(), committed);
    runtime.shutdown(&mut client, &mut model).await.unwrap();
    client.shutdown().await.unwrap();
    worker.await.unwrap();
    let reopened = OwnedStore::open(&selection.path, &selection.source).unwrap();
    let state = reopened.checked_state().unwrap();
    let obligation = state.inventory().obligations.values().next().unwrap();
    assert_eq!(obligation.state, ObligationState::Resolved);
    assert_eq!(
        state
            .review(&obligation.id)
            .unwrap()
            .settlement
            .as_ref()
            .unwrap()
            .actor,
        "Questmancer"
    );
    assert_eq!(state.inventory().quests[&quest].state, QuestState::Planned);
}
