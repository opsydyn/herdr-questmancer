use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use questmancer::{
    app::{Modal, Model, View},
    domain::Timestamp,
    interaction::{ActionReduction, reduce_action},
    owned_work::{
        Action, CheckReceipt, CheckResult, Command, CommandId, OwnedRuntime, OwnedStore,
        StoreSelection, authoring,
    },
    persistence::{PersistenceClient, PersistenceWorker, WorkerPaths, load_work_cache},
    runtime_loop::dispatch_action_effects,
    ui::input::action_for_event_in,
    work::{CampaignId, ObligationState, QuestId, QuestKind, QuestState, SourceId},
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
