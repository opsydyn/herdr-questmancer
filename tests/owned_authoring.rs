use questmancer::{
    app::{Model, View},
    owned_work::{OwnedRuntime, StoreSelection},
    persistence::{PersistenceWorker, WorkerPaths, load_state},
    work::SourceId,
};

proptest::proptest! {
    #[test]
    fn submitted_draft_is_immutable_under_repeat_paste_and_resize(text in "[a-zA-Z界]{1,60}", repeats in 0usize..64) {
        use questmancer::owned_work::authoring::{FormKind,Phase,Metadata,handle_event};
        use crossterm::event::{Event,KeyEvent,KeyCode,KeyModifiers,KeyEventKind};
        let mut model=Model::new(View::Guild);model.open_owned_authoring();
        model.authoring_mut().phase=Phase::Ready;
        model.authoring_mut().metadata=Some(Metadata {source:SourceId::new("owned:test"),revision:0,inventory:questmancer::work::Inventory::default()});
        model.authoring_mut().form(FormKind::Campaign);
        handle_event(&mut model,&Event::Paste(text.clone()));model.authoring_mut().focus=2;
        handle_event(&mut model,&Event::Key(KeyEvent::new(KeyCode::Enter,KeyModifiers::NONE)));
        for _ in 0..repeats {
            handle_event(&mut model,&Event::Key(KeyEvent::new_with_kind(KeyCode::Enter,KeyModifiers::NONE,KeyEventKind::Repeat)));
            handle_event(&mut model,&Event::Paste("other input".into()));model.set_work_viewport_size(20,8);
            handle_event(&mut model,&Event::Resize(20,8));model.set_work_viewport_size(100,40);
        }
        proptest::prop_assert_eq!(&model.authoring().fields[0].text,&text);
        proptest::prop_assert_eq!(model.authoring().phase,Phase::Saving);
    }
}

#[test]
fn long_answer_rows_wrap_without_losing_content_at_form_minimum() {
    use questmancer::owned_work::authoring::{Authoring, FormKind, Metadata, Phase};
    let mut author = Authoring::default();
    author.phase = Phase::Ready;
    author.metadata = Some(Metadata {
        source: SourceId::new("owned:test"),
        revision: 0,
        inventory: questmancer::work::Inventory::default(),
    });
    author.form(FormKind::Campaign);
    author.edit(&"a".repeat(120));
    let (rows, anchor) = questmancer::ui::owned_authoring::rows(&author, 22);
    assert!(
        rows.iter()
            .all(|r| ratatui::text::Line::raw(r).width() <= 22)
    );
    assert!(rows.concat().contains(&"a".repeat(120)));
    assert!(anchor > 0);
}

#[test]
fn migration_preserves_v1_intent_and_refuses_a_smuggled_pin() {
    let mut state = questmancer::persistence::PersistedStateV1::capture(&Model::new(View::Delve));
    state.experience = 42;
    state.schema_version = 1;
    let bytes = serde_json::to_vec(&state).unwrap();
    let migrated = questmancer::persistence::parse_state("state.json".as_ref(), &bytes).unwrap();
    assert_eq!(migrated.schema_version, 2);
    assert_eq!(migrated.experience, 42);
    assert_eq!(migrated.last_view, View::Delve);
    state.owned_store = Some(
        StoreSelection::new("/tmp/owned-work.json".into(), SourceId::new("owned:test")).unwrap(),
    );
    assert!(
        questmancer::persistence::parse_state(
            "state.json".as_ref(),
            &serde_json::to_vec(&state).unwrap()
        )
        .is_err()
    );
}

#[tokio::test]
async fn protected_intent_stops_creation_before_it_touches_owned_files() {
    let dir = tempfile::tempdir_in(std::fs::canonicalize(std::env::temp_dir()).unwrap()).unwrap();
    let paths = WorkerPaths::new(Some(dir.path().join("state.json")), None);
    let bytes = b"{\"schema_version\":999}";
    std::fs::write(paths.state.as_ref().unwrap(), bytes).unwrap();
    let (mut client, _, writer) = PersistenceWorker::start(paths.clone());
    let mut model = Model::new(View::Guild);
    let selection = StoreSelection::new(
        dir.path().join("owned-work.json"),
        SourceId::new("owned:test"),
    )
    .unwrap();
    let mut runtime = OwnedRuntime::start(&paths, &mut model).unwrap();
    model.authoring_mut().request_setup(selection.clone(), true);
    runtime.dispatch(&mut model);
    runtime.consume_next(&mut client, &mut model).await;
    assert!(!selection.path.exists());
    assert!(!dir.path().join("owned-work.json.lock").exists());
    assert_eq!(std::fs::read(paths.state.as_ref().unwrap()).unwrap(), bytes);
    runtime.shutdown(&mut client, &mut model).await.unwrap();
    client.shutdown().await.unwrap();
    writer.await.unwrap();
}

#[tokio::test]
async fn remembering_failure_retries_only_the_same_selection() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use questmancer::owned_work::authoring::{Phase, Surface, handle_event};
    let dir = tempfile::tempdir_in(std::fs::canonicalize(std::env::temp_dir()).unwrap()).unwrap();
    let paths = WorkerPaths::new(Some(dir.path().join("state.json")), None);
    let (mut client, _, writer) = PersistenceWorker::start(paths.clone());
    let mut model = Model::new(View::Guild);
    let selection = StoreSelection::new(
        dir.path().join("owned-work.json"),
        SourceId::new("owned:test"),
    )
    .unwrap();
    let mut runtime = OwnedRuntime::start(&paths, &mut model).unwrap();
    model.authoring_mut().request_setup(selection.clone(), true);
    runtime.dispatch(&mut model);
    runtime.wait_result().await;
    let original = std::fs::read(&selection.path).unwrap();
    std::fs::create_dir(paths.state.as_ref().unwrap()).unwrap();
    runtime.consume_next(&mut client, &mut model).await;
    assert_eq!(model.authoring().phase, Phase::RememberPending);
    assert_eq!(model.authoring().selection, Some(selection.clone()));
    std::fs::remove_dir(paths.state.as_ref().unwrap()).unwrap();
    model.open_owned_authoring();
    model.authoring_mut().surface = Surface::Menu;
    model.authoring_mut().cursor = 10;
    handle_event(
        &mut model,
        &Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
    );
    runtime.dispatch(&mut model);
    runtime.consume_next(&mut client, &mut model).await;
    assert!(model.authoring().ready());
    assert_eq!(std::fs::read(&selection.path).unwrap(), original);
    runtime.shutdown(&mut client, &mut model).await.unwrap();
    client.shutdown().await.unwrap();
    writer.await.unwrap();
}

#[test]
fn paste_repeat_and_resize_preserve_literal_draft_and_single_submission() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    use questmancer::owned_work::authoring::{FormKind, Metadata, Phase, handle_event};
    let mut model = Model::new(View::Guild);
    model.open_owned_authoring();
    model.authoring_mut().phase = Phase::Ready;
    model.authoring_mut().metadata = Some(Metadata {
        source: SourceId::new("owned:test"),
        revision: 0,
        inventory: questmancer::work::Inventory::default(),
    });
    model.authoring_mut().form(FormKind::Campaign);
    handle_event(&mut model, &Event::Paste("!r界campaign".into()));
    assert_eq!(model.authoring().fields[0].text, "!r界campaign");
    handle_event(&mut model, &Event::Paste("x".repeat(8193)));
    assert_eq!(model.authoring().fields[0].text, "!r界campaign");
    model.authoring_mut().focus = 2;
    model.set_work_viewport_size(27, 11);
    let enter = Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    handle_event(&mut model, &enter);
    assert_eq!(model.authoring().phase, Phase::Ready);
    model.set_work_viewport_size(80, 30);
    let repeat = Event::Key(KeyEvent::new_with_kind(
        KeyCode::Enter,
        KeyModifiers::NONE,
        KeyEventKind::Repeat,
    ));
    handle_event(&mut model, &repeat);
    assert_eq!(model.authoring().phase, Phase::Ready);
    handle_event(&mut model, &enter);
    assert_eq!(model.authoring().phase, Phase::Saving);
    handle_event(&mut model, &Event::Paste("new text".into()));
    handle_event(&mut model, &repeat);
    assert_eq!(model.authoring().fields[0].text, "!r界campaign");
}

#[test]
fn checked_detail_boundary_rejects_identical_but_replaced_file() {
    let dir = tempfile::tempdir_in(std::fs::canonicalize(std::env::temp_dir()).unwrap()).unwrap();
    let path = dir.path().join("owned-work.json");
    let source = SourceId::new("owned:test");
    let store = questmancer::owned_work::OwnedStore::create(&path, source).unwrap();
    let replacement = dir.path().join("replacement");
    std::fs::write(&replacement, std::fs::read(&path).unwrap()).unwrap();
    std::fs::rename(replacement, path).unwrap();
    assert!(store.checked_state().is_err());
}

#[tokio::test]
#[allow(clippy::too_many_lines)] // Composed commit/view fault recovery.
async fn cache_failure_after_answer_is_saved_and_view_retry_cannot_duplicate_it() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use questmancer::owned_work::authoring::{FormKind, Phase, Surface, handle_event};
    use questmancer::{
        domain::Timestamp,
        owned_work::{Action, Command, CommandId, OwnedStore},
        work::{CampaignId, ObligationId},
    };
    let dir = tempfile::tempdir_in(std::fs::canonicalize(std::env::temp_dir()).unwrap()).unwrap();
    let selection = StoreSelection::new(
        dir.path().join("owned-work.json"),
        SourceId::new("owned:test"),
    )
    .unwrap();
    let campaign = CampaignId::new(&selection.source, "campaign");
    let id = ObligationId::new(&selection.source, "decision");
    let mut store = OwnedStore::create(&selection.path, selection.source.clone()).unwrap();
    for (revision, action) in [
        Action::CreateCampaign {
            id: campaign.clone(),
            name: "Policy".into(),
            objective: None,
        },
        Action::RequestDecision {
            id: id.clone(),
            campaign: campaign.clone(),
            quest: None,
            incarnation: "request".into(),
            prompt: "Keep compatibility?".into(),
            context: None,
        },
    ]
    .into_iter()
    .enumerate()
    {
        store
            .execute(&Command {
                id: CommandId::new(&format!("c{revision}")),
                source: selection.source.clone(),
                expected_revision: u64::try_from(revision).unwrap(),
                actor: "Questmancer".into(),
                at: Timestamp::from_millis(100),
                action,
            })
            .unwrap();
    }
    drop(store);
    let paths = WorkerPaths::new(Some(dir.path().join("state.json")), None);
    let cache = paths.work_path().unwrap();
    let (mut client, _, writer) = PersistenceWorker::start(paths.clone());
    let mut model = Model::new(View::Guild);
    let mut runtime = OwnedRuntime::start(&paths, &mut model).unwrap();
    model
        .authoring_mut()
        .request_setup(selection.clone(), false);
    runtime.dispatch(&mut model);
    runtime.consume_next(&mut client, &mut model).await;
    model.authoring_mut().campaign = Some(campaign);
    model.authoring_mut().read(id.clone(), "request".into());
    runtime.dispatch(&mut model);
    runtime.consume_next(&mut client, &mut model).await;
    model.open_owned_authoring();
    model.authoring_mut().form(FormKind::Answer);
    model.authoring_mut().fields[0].text = "Yes, for one release.".into();
    model.authoring_mut().focus = 1;
    let old_cache = std::fs::read(&cache).unwrap();
    std::fs::write(&cache, b"{\"version\":999}").unwrap();
    let enter = Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    handle_event(&mut model, &enter);
    runtime.dispatch(&mut model);
    runtime.consume_next(&mut client, &mut model).await;
    assert_eq!(model.authoring().phase, Phase::ViewPending);
    let committed = std::fs::read(&selection.path).unwrap();
    assert!(model.authoring().feedback.contains("saved"));
    handle_event(&mut model, &enter);
    assert_eq!(model.authoring().phase, Phase::ViewPending);
    std::fs::write(&cache, old_cache).unwrap();
    model.authoring_mut().menu();
    model.authoring_mut().cursor = 10;
    handle_event(&mut model, &enter);
    runtime.dispatch(&mut model);
    runtime.consume_next(&mut client, &mut model).await;
    runtime.dispatch(&mut model);
    runtime.consume_next(&mut client, &mut model).await;
    assert_eq!(std::fs::read(&selection.path).unwrap(), committed);
    assert!(questmancer::work::queue(&model).is_empty());
    assert_eq!(model.authoring().surface, Surface::Detail);
    assert_eq!(
        model
            .authoring()
            .detail
            .as_ref()
            .unwrap()
            .answer
            .as_ref()
            .unwrap()
            .text,
        "Yes, for one release."
    );
    runtime.shutdown(&mut client, &mut model).await.unwrap();
    client.shutdown().await.unwrap();
    writer.await.unwrap();
}

#[tokio::test]
async fn terminal_lifecycle_owns_and_joins_offline_authoring() {
    let dir = tempfile::tempdir_in(std::fs::canonicalize(std::env::temp_dir()).unwrap()).unwrap();
    let paths = WorkerPaths::new(Some(dir.path().join("state.json")), None);
    let mut model = Model::new(View::Guild);
    let (mut lifecycle, _) = questmancer::terminal::RuntimeLifecycle::start(None, paths);
    lifecycle.start_owned(&mut model).unwrap();
    let selection = StoreSelection::new(
        dir.path().join("owned-work.json"),
        SourceId::new("owned:test"),
    )
    .unwrap();
    model.authoring_mut().request_setup(selection.clone(), true);
    lifecycle.dispatch_owned(&mut model);
    lifecycle.consume_owned(&mut model).await;
    assert!(model.authoring().ready());
    lifecycle.finish_owned(&mut model).await.unwrap();
    lifecycle.complete(None).await.unwrap();
    let store =
        questmancer::owned_work::OwnedStore::open(&selection.path, &selection.source).unwrap();
    assert_eq!(store.state().revision(), 0);
}

#[tokio::test]
async fn explicit_setup_remembers_owner_and_restart_never_recreates_missing_store() {
    let dir = tempfile::tempdir_in(std::fs::canonicalize(std::env::temp_dir()).unwrap()).unwrap();
    let paths = WorkerPaths::new(Some(dir.path().join("state.json")), None);
    let selection = StoreSelection::new(
        dir.path().join("owned-work.json"),
        SourceId::new("owned:test"),
    )
    .unwrap();
    let (mut client, _, writer) = PersistenceWorker::start(paths.clone());
    let mut model = Model::new(View::Guild);
    let mut runtime = OwnedRuntime::start(&paths, &mut model).unwrap();
    assert!(!selection.path.exists());
    model.authoring_mut().request_setup(selection.clone(), true);
    runtime.dispatch(&mut model);
    runtime.consume_next(&mut client, &mut model).await;
    assert!(model.authoring().ready());
    assert_eq!(
        load_state(paths.state.as_ref().unwrap())
            .await
            .unwrap()
            .unwrap()
            .owned_store,
        Some(selection.clone())
    );
    runtime.shutdown(&mut client, &mut model).await.unwrap();
    client.shutdown().await.unwrap();
    writer.await.unwrap();
    std::fs::remove_file(&selection.path).unwrap();
    let (mut client, _, writer) = PersistenceWorker::start(paths.clone());
    let mut restored = Model::new(View::Guild);
    restored
        .durable_intent_mut()
        .set_owned_store(Some(selection.clone()));
    let mut runtime = OwnedRuntime::start(&paths, &mut restored).unwrap();
    runtime.dispatch(&mut restored);
    runtime.consume_next(&mut client, &mut restored).await;
    assert!(!restored.authoring().ready());
    assert!(!selection.path.exists());
    runtime.shutdown(&mut client, &mut restored).await.unwrap();
    client.shutdown().await.unwrap();
    writer.await.unwrap();
}

#[tokio::test]
#[allow(clippy::too_many_lines)] // Composed input/restart proof, with one owner and writer.
async fn production_input_creates_campaign_and_request_answers_and_recovers_literal_detail() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use questmancer::owned_work::authoring::{FormKind, Surface, handle_event};
    let dir = tempfile::tempdir_in(std::fs::canonicalize(std::env::temp_dir()).unwrap()).unwrap();
    let paths = WorkerPaths::new(Some(dir.path().join("state.json")), None);
    let (mut client, _, writer) = PersistenceWorker::start(paths.clone());
    let mut model = Model::new(View::Guild);
    let mut runtime = OwnedRuntime::start(&paths, &mut model).unwrap();
    model.open_quest_board();
    let key = |code| Event::Key(KeyEvent::new(code, KeyModifiers::NONE));
    let action =
        questmancer::ui::input::action_for_event_in(&key(KeyCode::Char('a')), model.modal());
    let _ = questmancer::interaction::reduce_action(&mut model, action);
    assert_eq!(model.modal(), &questmancer::app::Modal::OwnedAuthoring);
    model.authoring_mut().form(FormKind::CreateStore);
    handle_event(&mut model, &key(KeyCode::Tab));
    handle_event(&mut model, &key(KeyCode::Enter));
    runtime.dispatch(&mut model);
    runtime.consume_next(&mut client, &mut model).await;
    assert!(model.authoring().ready());
    for (kind, text, extras) in [
        (
            FormKind::Campaign,
            "Replace authentication layer",
            vec![(1, "A durable objective")],
        ),
        (FormKind::Quest, "Migrate API auth", vec![(1, "Ship")]),
        (
            FormKind::Request,
            "Keep compatibility?",
            vec![(1, "Need a policy")],
        ),
    ] {
        model.authoring_mut().form(kind);
        handle_event(&mut model, &Event::Paste(text.into()));
        for (index, value) in extras {
            let f = &mut model.authoring_mut().fields[index];
            if f.choices.is_empty() {
                f.text = value.into();
            } else {
                f.choice = f.choices.iter().position(|c| c.0 == value).unwrap();
            }
        }
        for _ in 0..model.authoring().fields.len() {
            handle_event(&mut model, &key(KeyCode::Tab));
        }
        handle_event(&mut model, &key(KeyCode::Enter));
        runtime.dispatch(&mut model);
        runtime.consume_next(&mut client, &mut model).await;
        assert!(model.authoring().ready(), "{}", model.authoring().feedback);
    }
    let obligation = model
        .authoring()
        .metadata
        .as_ref()
        .unwrap()
        .inventory
        .obligations
        .values()
        .next()
        .unwrap()
        .clone();
    model
        .authoring_mut()
        .read(obligation.id.clone(), obligation.incarnation.clone());
    runtime.dispatch(&mut model);
    runtime.consume_next(&mut client, &mut model).await;
    model.authoring_mut().form(FormKind::Answer);
    handle_event(
        &mut model,
        &Event::Paste("Yes, for one release.\nDocument its removal.".into()),
    );
    handle_event(&mut model, &key(KeyCode::Tab));
    handle_event(&mut model, &key(KeyCode::Enter));
    runtime.dispatch(&mut model);
    runtime.consume_next(&mut client, &mut model).await;
    runtime.dispatch(&mut model);
    runtime.consume_next(&mut client, &mut model).await;
    assert_eq!(model.authoring().surface, Surface::Detail);
    assert_eq!(
        model
            .authoring()
            .detail
            .as_ref()
            .unwrap()
            .answer
            .as_ref()
            .unwrap()
            .text,
        "Yes, for one release.\nDocument its removal."
    );
    assert!(questmancer::work::queue(&model).is_empty());
    model.authoring_mut().form(FormKind::Artifact);
    model.authoring_mut().fields[0].text = "Migration report".into();
    model.authoring_mut().fields[1].choice = 4;
    model.authoring_mut().focus = model.authoring().fields.len();
    handle_event(&mut model, &key(KeyCode::Enter));
    runtime.dispatch(&mut model);
    runtime.consume_next(&mut client, &mut model).await;
    assert_eq!(
        model
            .authoring()
            .metadata
            .as_ref()
            .unwrap()
            .inventory
            .artifacts
            .len(),
        1
    );
    runtime.shutdown(&mut client, &mut model).await.unwrap();
    client.shutdown().await.unwrap();
    writer.await.unwrap();
    let startup = questmancer::persistence::load_startup(
        questmancer::config::PersistencePaths::from_lookup(|name| {
            (name == "HERDR_PLUGIN_STATE_DIR").then(|| dir.path().display().to_string())
        }),
        None,
    )
    .await;
    let mut model = startup.model;
    let (mut client, _, writer) = PersistenceWorker::start(startup.paths.clone());
    let mut runtime = OwnedRuntime::start(&startup.paths, &mut model).unwrap();
    runtime.dispatch(&mut model);
    runtime.consume_next(&mut client, &mut model).await;
    assert!(model.authoring().ready());
    model
        .authoring_mut()
        .read(obligation.id, obligation.incarnation);
    runtime.dispatch(&mut model);
    runtime.consume_next(&mut client, &mut model).await;
    assert!(model.authoring().detail.as_ref().unwrap().answer.is_some());
    runtime.shutdown(&mut client, &mut model).await.unwrap();
    client.shutdown().await.unwrap();
    writer.await.unwrap();
}

#[test]
fn wheel_can_read_context_after_editing_without_changing_the_draft() {
    use crossterm::event::{Event, KeyModifiers, MouseEvent, MouseEventKind};
    use questmancer::owned_work::authoring::{FormKind, Phase, handle_event};
    let mut model = Model::new(View::Guild);
    model.open_owned_authoring();
    model.authoring_mut().phase = Phase::Ready;
    model.authoring_mut().metadata = Some(questmancer::owned_work::authoring::Metadata {
        source: SourceId::new("owned:test"),
        revision: 0,
        inventory: questmancer::work::Inventory::default(),
    });
    model.authoring_mut().form(FormKind::Campaign);
    model.authoring_mut().edit("Retained draft");
    assert!(model.authoring().follow_focus);
    handle_event(
        &mut model,
        &Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 1,
            row: 1,
            modifiers: KeyModifiers::NONE,
        }),
    );
    assert!(!model.authoring().follow_focus);
    assert_eq!(model.authoring().fields[0].text, "Retained draft");
}
