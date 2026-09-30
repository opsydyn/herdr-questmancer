use questmancer::{domain::Timestamp, work::*};
use std::collections::BTreeMap;
fn baseline(source: &SourceId) -> Message {
    let campaign = CampaignId::new(source, "auth");
    let id = ObligationId::new(source, "decision");
    let mut inventory = Inventory::default();
    inventory.campaigns.insert(
        campaign.clone(),
        WorkCampaign {
            id: campaign.clone(),
            revision: 1,
            name: "Auth".into(),
            objective: None,
            state: CampaignState::Active,
        },
    );
    inventory.obligations.insert(
        id.clone(),
        Obligation {
            id,
            campaign,
            quest: None,
            revision: 1,
            incarnation: "request-1".into(),
            kind: ObligationKind::Decision,
            responsible: Responsible::Human,
            state: ObligationState::Open,
            prompt: "Keep the endpoint?".into(),
            context: None,
            action: "Answer through source".into(),
            resolution: "Matching source close".into(),
            requested_at: Timestamp::from_millis(100),
        },
    );
    Message::Baseline {
        source: source.clone(),
        generation: 1,
        captured_at: Timestamp::from_millis(100),
        cursor: 0,
        inventory,
        complete: true,
        bindings: BTreeMap::new(),
    }
}
#[test]
fn qualified_ids_and_exit_preserve_obligations() {
    let a = SourceId::new("home-a");
    let b = SourceId::new("home-b");
    let state = reduce(Projection::default(), baseline(&a)).unwrap();
    let state = reduce(state, baseline(&b)).unwrap();
    assert_eq!(state.sources.len(), 2);
    let state = reduce(
        state,
        Message::BindingRemoved {
            source: a.clone(),
            quest: QuestId::new(&a, "web"),
        },
    )
    .unwrap();
    assert_eq!(
        state.sources[&a].inventory.obligations[&ObligationId::new(&a, "decision")].state,
        ObligationState::Open
    );
    assert_ne!(
        ObligationId::new(&a, "decision"),
        ObligationId::new(&b, "decision")
    );
}

#[test]
fn partial_baseline_preserves_references_and_complete_cannot_roll_back() {
    let source = SourceId::new("source");
    let initial = reduce(Projection::default(), baseline(&source)).unwrap();
    let mut inventory = Inventory::default();
    let mut obligation = initial
        .obligation(&ObligationId::new(&source, "decision"))
        .unwrap()
        .clone();
    obligation.revision = 2;
    obligation.context = Some("Progress is unrelated".into());
    inventory
        .obligations
        .insert(obligation.id.clone(), obligation);
    let updated = reduce(
        initial.clone(),
        Message::Baseline {
            source: source.clone(),
            generation: 1,
            captured_at: Timestamp::from_millis(100),
            cursor: 1,
            inventory,
            complete: false,
            bindings: BTreeMap::new(),
        },
    )
    .expect("partial coverage retains campaign");
    assert_eq!(updated.sources[&source].inventory.campaigns.len(), 1);
    assert_eq!(reduce(updated.clone(), baseline(&source)).unwrap(), updated);
    let mut old = baseline(&source);
    if let Message::Baseline { cursor, .. } = &mut old {
        *cursor = 2;
    }
    assert_eq!(reduce(updated, old), Err(WorkError::Reconcile));
}

#[test]
fn duplicate_payload_conflict_and_exact_resolution() {
    let source = SourceId::new("source");
    let id = ObligationId::new(&source, "decision");
    let state = reduce(Projection::default(), baseline(&source)).unwrap();
    let update = Message::Update {
        source: source.clone(),
        generation: 1,
        occurred_at: Timestamp::from_millis(101),
        cursor: 1,
        event: "one".into(),
        changes: vec![],
    };
    let state = reduce(state, update.clone()).unwrap();
    assert_eq!(reduce(state.clone(), update).unwrap(), state);
    let conflicting = Message::Update {
        source: source.clone(),
        generation: 1,
        occurred_at: Timestamp::from_millis(101),
        cursor: 1,
        event: "one".into(),
        changes: vec![Change::Resolve {
            id: id.clone(),
            incarnation: "request-1".into(),
            revision: 2,
        }],
    };
    assert_eq!(reduce(state.clone(), conflicting), Err(WorkError::Conflict));
    let state = reduce(
        state,
        Message::Update {
            source: source.clone(),
            generation: 1,
            occurred_at: Timestamp::from_millis(101),
            cursor: 2,
            event: "two".into(),
            changes: vec![Change::Resolve {
                id: id.clone(),
                incarnation: "wrong".into(),
                revision: 2,
            }],
        },
    )
    .unwrap();
    assert_eq!(state.obligation(&id).unwrap().state, ObligationState::Open);
    let state = reduce(
        state,
        Message::Update {
            source,
            generation: 1,
            occurred_at: Timestamp::from_millis(101),
            cursor: 3,
            event: "three".into(),
            changes: vec![Change::Resolve {
                id: id.clone(),
                incarnation: "request-1".into(),
                revision: 2,
            }],
        },
    )
    .unwrap();
    assert!(
        state
            .eligible_obligations(Timestamp::from_millis(100))
            .is_empty()
    );
}

#[tokio::test]
async fn restart_preserves_intent_as_unverified_without_live_bindings() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("work-state.json");
    let source = SourceId::new("source");
    let id = ObligationId::new(&source, "decision");
    let mut state = reduce(Projection::default(), baseline(&source)).unwrap();
    state.acknowledge(&id, Some(Timestamp::from_millis(999)));
    assert!(
        questmancer::persistence::publish_work(&path, &state)
            .await
            .unwrap()
    );
    let restored = questmancer::persistence::load_work(&path)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(restored.sources[&source].freshness, Freshness::Cached);
    assert_eq!(restored.intent, state.intent);
    assert!(
        restored
            .eligible_obligations(Timestamp::from_millis(998))
            .is_empty()
    );
    assert_eq!(
        restored.eligible_obligations(Timestamp::from_millis(1000)),
        vec![id]
    );
    assert!(
        !questmancer::persistence::publish_work(&path, &restored)
            .await
            .unwrap()
    );
}

#[test]
fn fixture_disconnect_retains_open_work_and_reconciliation_requires_baseline() {
    let stream = fixture::FixtureSource::authentication(Timestamp::from_millis(100));
    assert!(!stream.is_empty());
    let mut p = Projection::default();
    for message in stream {
        p = reduce(p, message).unwrap();
    }
    assert_eq!(p.sources.values().next().unwrap().inventory.quests.len(), 4);
    let source = p.sources.keys().next().unwrap().clone();
    p = reduce(p, Message::Disconnected(source.clone())).unwrap();
    assert_eq!(p.eligible_obligations(Timestamp::from_millis(200)).len(), 1);
    assert_eq!(
        reduce(
            p.clone(),
            Message::Update {
                source: source.clone(),
                generation: 1,
                occurred_at: Timestamp::from_millis(101),
                cursor: 2,
                event: "late".into(),
                changes: vec![]
            }
        ),
        Err(WorkError::Reconcile)
    );
    p = reduce(
        p.clone(),
        Message::Baseline {
            source: source.clone(),
            generation: 1,
            captured_at: Timestamp::from_millis(100),
            cursor: 1,
            inventory: p.sources[&source].inventory.clone(),
            complete: true,
            bindings: BTreeMap::new(),
        },
    )
    .unwrap();
    assert_eq!(p.sources[&source].freshness, Freshness::Current);
}

#[tokio::test]
async fn composed_source_progress_exit_restart_and_matching_resolution() {
    use questmancer::{
        app::{Model, View},
        interaction::reduce_action,
        persistence::{PersistenceWorker, WorkerPaths},
        runtime_loop::dispatch_action_effects,
        ui::input::Action,
    };
    let dir = tempfile::tempdir().unwrap();
    let paths = WorkerPaths::new(
        Some(dir.path().join("state.json")),
        Some(dir.path().join("chronicle.jsonl")),
    );
    let (mut client, _, task) = PersistenceWorker::start(paths.clone());
    let source = SourceId::new("fixture-auth");
    let id = ObligationId::new(&source, "compatibility");
    let now = Timestamp::from_millis(121_000);
    let (mut model, bindings) = composed_work_seed();
    model.set_now(now);
    let (sender, mut inbox) = source::channel(source.clone(), 4);
    for mut message in fixture::FixtureSource::authentication(now) {
        if let Message::Baseline {
            bindings: target, ..
        } = &mut message
        {
            target.clone_from(&bindings);
        }
        sender
            .send(source::Envelope {
                version: 1,
                message,
            })
            .await
            .unwrap();
    }
    for _ in 0..2 {
        runtime::accept(&client, &mut model, inbox.next().await)
            .await
            .unwrap();
    }
    composed_worker_exit(&mut model);
    runtime::accept(
        &client,
        &mut model,
        source::Envelope {
            version: 1,
            message: Message::BindingRemoved {
                source: source.clone(),
                quest: QuestId::new(&source, "web"),
            },
        },
    )
    .await
    .unwrap();
    assert_eq!(
        model.work().obligation(&id).unwrap().state,
        ObligationState::Open
    );
    assert!(bound_agent(&model, &QuestId::new(&source, "web")).is_none());
    model.select_work(WorkSelection::Obligation(id.clone()));
    let reduction = reduce_action(&mut model, Action::DeferSummons);
    assert!(
        dispatch_action_effects(&mut client, &mut model, reduction)
            .await
            .persistence_errors
            .is_empty()
    );
    drop(sender);
    runtime::accept(&client, &mut model, inbox.next().await)
        .await
        .unwrap();
    client.flush().await.unwrap();
    client.shutdown().await.unwrap();
    task.await.unwrap();
    let restored = questmancer::persistence::load_work(&dir.path().join("work-state.json"))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(restored.sources[&source].freshness, Freshness::Cached);
    assert!(restored.intent[&id].deferred_until.is_some());
    assert_eq!(restored.open_decisions(), vec![id.clone()]);
    let (client, _, task) = PersistenceWorker::start(paths);
    model = Model::new(View::Guild);
    model.set_now(now);
    model.replace_work(restored);
    assert!(model.domain().agents.is_empty());
    reconcile_composed_source(&client, &mut model, &source, now).await;
    assert_resolved_work(&model, &source, &id);
    assert_eq!(model.experience(), 0);
    assert!(!dir.path().join("state.json").exists());
    assert!(!dir.path().join("chronicle.jsonl").exists());
    client.shutdown().await.unwrap();
    task.await.unwrap();
}

fn composed_work_seed() -> (questmancer::app::Model, BTreeMap<QuestId, LiveBinding>) {
    #[cfg(feature = "storybook")]
    {
        let mut model = questmancer::storybook::fixtures::quest_board_fixture(
            questmancer::storybook::fixtures::StoryContext::fixed(),
            questmancer::storybook::fixtures::WorkBoardState::Populated,
        );
        let bindings = model.work().sources[&SourceId::new("fixture-auth")]
            .bindings
            .clone();
        model.replace_work(Projection::default());
        (model, bindings)
    }
    #[cfg(not(feature = "storybook"))]
    {
        (
            questmancer::app::Model::new(questmancer::app::View::Guild),
            BTreeMap::new(),
        )
    }
}
fn composed_worker_exit(model: &mut questmancer::app::Model) {
    #[cfg(feature = "storybook")]
    {
        let agent = bound_agent(model, &QuestId::new(&SourceId::new("fixture-auth"), "web"))
            .expect("explicit fixture binding")
            .clone();
        let (domain, _legacy_effects) = questmancer::update::update(
            model.domain().clone(),
            questmancer::update::AppEvent::PaneExited {
                identity: agent.capture_identity,
                pane_id: agent.pane_id,
                revision: agent.pane_revision + 1,
                occurred_at: model.now(),
            },
        );
        model.replace_domain(domain);
        assert_eq!(
            model.domain().agents[&agent.key].presence,
            questmancer::domain::Presence::Exited
        );
    }
    #[cfg(not(feature = "storybook"))]
    {
        assert!(model.domain().agents.is_empty());
    }
}

#[test]
fn urgency_reaches_work_without_live_agent_and_seen_never_resolves() {
    use questmancer::{
        app::{Modal, Model, View},
        interaction::reduce_action,
        ui::input::Action,
    };
    let mut model = Model::new(View::Guild);
    model.set_now(Timestamp::from_millis(100));
    let mut state = Projection::default();
    for event in fixture::FixtureSource::authentication(model.now()) {
        state = reduce(state, event).unwrap();
    }
    model.replace_work(state);
    let effects = reduce_action(&mut model, Action::NextUrgent);
    assert_eq!(model.modal(), &Modal::QuestBoard);
    assert!(effects.commands.is_empty());
    let effects = reduce_action(&mut model, Action::AcknowledgeSummons);
    assert!(effects.work_intent.is_some());
    assert!(
        model.work().intent.is_empty(),
        "intent is pending until storage acknowledgement"
    );
    model.settle_work_intent(effects.work_intent);
    assert_eq!(model.work().eligible_obligations(model.now()).len(), 1);
    assert!(
        reduce_action(&mut model, Action::Observe)
            .commands
            .is_empty()
    );
}

fn render_board(model: &questmancer::app::Model, width: u16, height: u16) -> String {
    let mut terminal =
        ratatui::Terminal::new(ratatui::backend::TestBackend::new(width, height)).unwrap();
    terminal
        .draw(|frame| {
            questmancer::ui::scene_overlays::render_scene_overlays(
                frame,
                model,
                &questmancer::scene::presentation::ScenePresentation::from_model(model),
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
}
#[test]
fn production_board_displays_literal_reason_freshness_and_unavailable_observe() {
    use questmancer::{
        app::{Model, View},
        interaction::reduce_action,
        ui::input::Action,
    };
    let mut model = Model::new(View::Guild);
    model.set_now(Timestamp::from_millis(100));
    let mut p = Projection::default();
    for event in fixture::FixtureSource::authentication(model.now()) {
        p = reduce(p, event).unwrap();
    }
    p.restore_cached();
    model.replace_work(p);
    let _ = reduce_action(&mut model, Action::NextUrgent);
    let text = render_board(&model, 120, 38);
    assert!(text.contains("QUEST BOARD"));
    assert!(text.contains("unverified"));
    assert!(text.contains("Should the old token endpoint"));
    assert!(text.contains("Observe unavailable"));
}

#[test]
fn closed_incarnation_intent_never_hides_new_request_and_dependencies_stay_source_owned() {
    let source = SourceId::new("fixture-auth");
    let id = ObligationId::new(&source, "compatibility");
    let mut p = Projection::default();
    for e in fixture::FixtureSource::authentication(Timestamp::from_millis(100)) {
        p = reduce(p, e).unwrap();
    }
    p.acknowledge(&id, Some(Timestamp::from_millis(10_000)));
    let mut newer = p.obligation(&id).unwrap().clone();
    newer.revision = 2;
    newer.incarnation = "second".into();
    p = reduce(
        p,
        Message::Update {
            source: source.clone(),
            generation: 1,
            occurred_at: Timestamp::from_millis(101),
            cursor: 2,
            event: "new".into(),
            changes: vec![Change::Obligation(newer)],
        },
    )
    .unwrap();
    p = reduce(
        p,
        Message::Update {
            source: source.clone(),
            generation: 1,
            occurred_at: Timestamp::from_millis(101),
            cursor: 3,
            event: "old-close".into(),
            changes: vec![Change::Resolve {
                id: id.clone(),
                incarnation: "compatibility-1".into(),
                revision: 99,
            }],
        },
    )
    .unwrap();
    assert_eq!(
        p.eligible_obligations(Timestamp::from_millis(101)),
        vec![id]
    );
    let tests = QuestId::new(&source, "tests");
    assert!(!p.dependencies_ready(&tests));
    let mut changes = Vec::new();
    for local in ["api", "web"] {
        let mut q = p.quest(&QuestId::new(&source, local)).unwrap().clone();
        q.revision = 3;
        q.state = QuestState::Completed;
        changes.push(Change::Quest(q));
    }
    p = reduce(
        p,
        Message::Update {
            source: source.clone(),
            generation: 1,
            occurred_at: Timestamp::from_millis(101),
            cursor: 4,
            event: "dependencies".into(),
            changes,
        },
    )
    .unwrap();
    assert!(p.dependencies_ready(&tests));
    assert_eq!(p.quest(&tests).unwrap().state, QuestState::Planned);
    let mut cyclic = p.quest(&tests).unwrap().clone();
    cyclic.revision = 2;
    cyclic.dependencies = vec![tests.clone()];
    p = reduce(
        p,
        Message::Update {
            source,
            generation: 1,
            occurred_at: Timestamp::from_millis(101),
            cursor: 5,
            event: "cycle".into(),
            changes: vec![Change::Quest(cyclic)],
        },
    )
    .unwrap();
    assert!(!p.dependencies_ready(&tests));
}

#[tokio::test]
async fn corrupt_future_and_failed_publication_protect_prior_state() {
    use questmancer::persistence::{PersistenceWorker, WorkerPaths, load_work, publish_work};
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("work-state.json");
    let source = SourceId::new("source");
    let original = reduce(Projection::default(), baseline(&source)).unwrap();
    for bytes in [
        b"{broken".as_slice(),
        b"{\"version\":3,\"projection\":{\"sources\":{},\"intent\":{}}}".as_slice(),
    ] {
        tokio::fs::write(&path, bytes).await.unwrap();
        assert!(load_work(&path).await.is_err());
        assert!(publish_work(&path, &original).await.is_err());
        assert_eq!(tokio::fs::read(&path).await.unwrap(), bytes);
    }
    tokio::fs::remove_file(&path).await.unwrap();
    publish_work(&path, &original).await.unwrap();
    let bytes = tokio::fs::read(&path).await.unwrap();
    tokio::fs::create_dir(dir.path().join("work-state.json.tmp"))
        .await
        .unwrap();
    let (client, _, task) =
        PersistenceWorker::start(WorkerPaths::new(Some(dir.path().join("state.json")), None));
    let mut model = questmancer::app::Model::new(questmancer::app::View::Guild);
    model.replace_work(original.clone());
    let update = Message::Update {
        source: source.clone(),
        generation: 1,
        occurred_at: Timestamp::from_millis(101),
        cursor: 1,
        event: "close".into(),
        changes: vec![Change::Resolve {
            id: ObligationId::new(&source, "decision"),
            incarnation: "request-1".into(),
            revision: 2,
        }],
    };
    assert!(
        runtime::accept(
            &client,
            &mut model,
            source::Envelope {
                version: 1,
                message: update
            }
        )
        .await
        .is_err()
    );
    assert_eq!(model.work().sources[&source].cursor, 0);
    assert_eq!(
        model.work().sources[&source].freshness,
        Freshness::Reconcile
    );
    assert_eq!(
        model
            .work()
            .eligible_obligations(Timestamp::from_millis(200))
            .len(),
        1
    );
    assert_eq!(tokio::fs::read(&path).await.unwrap(), bytes);
    client.shutdown().await.unwrap();
    task.await.unwrap();
}

proptest::proptest! {
    #[test]
    fn unrelated_updates_and_duplicate_delivery_keep_obligation_open(updates in 1u64..60) {
        let source=SourceId::new("source");let mut p=reduce(Projection::default(),baseline(&source)).unwrap();
        for cursor in 1..=updates {
            let event=Message::Update{source:source.clone(),generation:1,occurred_at:Timestamp::from_millis(101),cursor,event:format!("progress-{cursor}"),changes:vec![]};
            p=reduce(p,event.clone()).unwrap();
            proptest::prop_assert_eq!(reduce(p.clone(),event).unwrap(),p.clone());
            proptest::prop_assert_eq!(p.eligible_obligations(Timestamp::from_millis(200)).len(),1);
        }
    }
}

#[test]
fn source_tombstone_retains_supersession_without_success() {
    let source = SourceId::new("source");
    let id = ObligationId::new(&source, "decision");
    let p = reduce(Projection::default(), baseline(&source)).unwrap();
    let p = reduce(
        p,
        Message::Update {
            source,
            generation: 1,
            occurred_at: Timestamp::from_millis(101),
            cursor: 1,
            event: "retired".into(),
            changes: vec![Change::RemoveObligation {
                id: id.clone(),
                incarnation: "request-1".into(),
                revision: 2,
            }],
        },
    )
    .unwrap();
    assert_eq!(
        p.obligation(&id).unwrap().state,
        ObligationState::Superseded
    );
}

#[test]
fn old_head_validation_does_not_claim_current_artifact_passed() {
    let mut p = Projection::default();
    for e in fixture::FixtureSource::authentication(Timestamp::from_millis(100)) {
        p = reduce(p, e).unwrap();
    }
    let source = SourceId::new("fixture-auth");
    let mut a = p.sources[&source]
        .inventory
        .artifacts
        .values()
        .next()
        .unwrap()
        .clone();
    a.revision = 2;
    a.head_revision = Some("new-head".into());
    a.validation = Some(Validation {
        head_revision: "old-head".into(),
        summary: "Passed".into(),
    });
    p = reduce(
        p,
        Message::Update {
            source: source.clone(),
            generation: 1,
            occurred_at: Timestamp::from_millis(101),
            cursor: 2,
            event: "new-head".into(),
            changes: vec![Change::Artifact(a)],
        },
    )
    .unwrap();
    let mut model = questmancer::app::Model::new(questmancer::app::View::Guild);
    model.replace_work(p);
    model.select_work(WorkSelection::Quest(QuestId::new(&source, "api")));
    assert!(
        !view::details(&model)
            .iter()
            .any(|line| line.contains("Validation: Passed"))
    );
}

#[tokio::test]
async fn inbox_cannot_write_another_sources_namespace() {
    let expected = SourceId::new("expected");
    let alien = SourceId::new("alien");
    let (sender, mut inbox) = source::channel(expected.clone(), 1);
    sender
        .send(source::Envelope {
            version: 1,
            message: baseline(&alien),
        })
        .await
        .unwrap();
    let delivered = inbox.next().await;
    assert_ne!(delivered.version, 1);
    assert_eq!(delivered.message, Message::Disconnected(expected));
}

#[test]
fn malformed_baseline_map_keys_are_not_normalised_into_valid_entities() {
    let source = SourceId::new("source");
    let mut message = baseline(&source);
    if let Message::Baseline { inventory, .. } = &mut message {
        let value = inventory.obligations.pop_first().unwrap().1;
        inventory
            .obligations
            .insert(ObligationId::new(&source, "wrong-key"), value);
    }
    assert_eq!(
        reduce(Projection::default(), message),
        Err(WorkError::Invalid)
    );
}

#[test]
fn deferred_decisions_remain_open_and_schedule_one_expiry_wake() {
    let source = SourceId::new("source");
    let mut p = reduce(Projection::default(), baseline(&source)).unwrap();
    let id = ObligationId::new(&source, "decision");
    p.acknowledge(&id, Some(Timestamp::from_millis(1000)));
    assert_eq!(p.open_decisions(), vec![id]);
    assert_eq!(
        p.next_attention_in(Timestamp::from_millis(100)),
        Some(std::time::Duration::from_millis(900))
    );
    assert_eq!(p.next_attention_in(Timestamp::from_millis(1000)), None);
}

#[test]
fn complete_coverage_cannot_be_undone_by_an_old_entity_revision() {
    let source = SourceId::new("source");
    let mut p = reduce(Projection::default(), baseline(&source)).unwrap();
    let old = p
        .obligation(&ObligationId::new(&source, "decision"))
        .unwrap()
        .clone();
    let mut inventory = p.sources[&source].inventory.clone();
    inventory.obligations.clear();
    p = reduce(
        p,
        Message::Baseline {
            source: source.clone(),
            generation: 1,
            captured_at: Timestamp::from_millis(200),
            cursor: 1,
            inventory,
            complete: true,
            bindings: BTreeMap::new(),
        },
    )
    .unwrap();
    p = reduce(
        p,
        Message::Update {
            source,
            generation: 1,
            occurred_at: Timestamp::from_millis(201),
            cursor: 2,
            event: "old-request".into(),
            changes: vec![Change::Obligation(old)],
        },
    )
    .unwrap();
    assert!(p.open_decisions().is_empty());
}

#[cfg(feature = "storybook")]
#[test]
fn explicit_session_binding_controls_observe_and_duplicate_collapse() {
    use questmancer::{
        domain::{CaptureIdentity, GuildAttention, GuildSummons, Presence, Timestamp},
        interaction::reduce_action,
        storybook::fixtures::{StoryContext, WorkBoardState, quest_board_fixture},
        ui::input::Action,
    };
    let mut model = quest_board_fixture(StoryContext::fixed(), WorkBoardState::Populated);
    assert_eq!(reduce_action(&mut model, Action::Observe).commands.len(), 1);
    let pane = bound_agent(&model, &QuestId::new(&SourceId::new("fixture-auth"), "web"))
        .unwrap()
        .pane_id
        .clone();
    model.set_managed_pane_id(Some(pane));
    assert!(
        reduce_action(&mut model, Action::Observe)
            .commands
            .is_empty()
    );
    model.set_managed_pane_id(None);
    let source = SourceId::new("fixture-auth");
    let quest = QuestId::new(&source, "web");
    let id = ObligationId::new(&source, "compatibility");
    let key = model.work().sources[&source].bindings[&quest].agent.clone();
    // Correlation is explicit; text resemblance has no effect.
    let episode = Timestamp::from_millis(111_000);
    let agent = model.domain_mut().agents.get_mut(&key).unwrap();
    agent.presence = Presence::Blocked;
    agent.attention = GuildAttention::unread(GuildSummons::CounselRequested, episode);
    let mut p = model.work().clone();
    p.sources
        .get_mut(&source)
        .unwrap()
        .inventory
        .obligations
        .get_mut(&id)
        .unwrap()
        .kind = ObligationKind::BlockedAction;
    model.replace_work(p.clone());
    assert!(queue(&model).contains(&AttentionTarget::Agent(key.clone())));
    p.sources
        .get_mut(&source)
        .unwrap()
        .bindings
        .get_mut(&quest)
        .unwrap()
        .counsel_since = Some(episode);
    model.replace_work(p);
    assert!(!queue(&model).contains(&AttentionTarget::Agent(key.clone())));
    model
        .domain_mut()
        .agents
        .get_mut(&key)
        .unwrap()
        .capture_identity = CaptureIdentity::from_parts("replacement-terminal", None);
    assert!(
        reduce_action(&mut model, Action::Observe)
            .commands
            .is_empty()
    );
    assert!(queue(&model).contains(&AttentionTarget::Agent(key)));
}

#[tokio::test]
async fn local_intent_is_committed_only_after_worker_acknowledgement() {
    use questmancer::{
        app::{Model, View},
        interaction::reduce_action,
        persistence::{PersistenceWorker, WorkerPaths, load_work},
        runtime_loop::dispatch_action_effects,
        ui::input::Action,
    };
    let dir = tempfile::tempdir().unwrap();
    let source = SourceId::new("source");
    let id = ObligationId::new(&source, "decision");
    let (mut client, _, task) =
        PersistenceWorker::start(WorkerPaths::new(Some(dir.path().join("state.json")), None));
    let mut model = Model::new(View::Guild);
    model.set_now(Timestamp::from_millis(100));
    model.replace_work(reduce(Projection::default(), baseline(&source)).unwrap());
    let _ = reduce_action(&mut model, Action::NextUrgent);
    let action = reduce_action(&mut model, Action::DeferSummons);
    assert!(model.work().intent.is_empty());
    assert!(
        dispatch_action_effects(&mut client, &mut model, action)
            .await
            .persistence_errors
            .is_empty()
    );
    assert!(model.work().intent.contains_key(&id));
    let loaded = load_work(&dir.path().join("work-state.json"))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(model.work().intent, loaded.intent);
    let bytes = tokio::fs::read(dir.path().join("work-state.json"))
        .await
        .unwrap();
    tokio::fs::create_dir(dir.path().join("work-state.json.tmp"))
        .await
        .unwrap();
    let action = reduce_action(&mut model, Action::AcknowledgeSummons);
    assert_eq!(
        dispatch_action_effects(&mut client, &mut model, action)
            .await
            .persistence_errors
            .len(),
        1
    );
    assert!(!model.work().intent[&id].seen);
    assert!(model.pending_work().is_none());
    assert_eq!(
        tokio::fs::read(dir.path().join("work-state.json"))
            .await
            .unwrap(),
        bytes
    );
    client.shutdown().await.unwrap();
    task.await.unwrap();
}

#[test]
fn narrow_details_are_reachable_and_inventory_is_keyboard_complete() {
    use questmancer::{
        app::{Model, View},
        interaction::reduce_action,
        ui::input::Action,
    };
    let mut model = Model::new(View::Guild);
    let mut p = Projection::default();
    for e in fixture::FixtureSource::authentication(Timestamp::from_millis(100)) {
        p = reduce(p, e).unwrap();
    }
    model.replace_work(p);
    model.set_board_viewport_width(28);
    let _ = reduce_action(&mut model, Action::NextUrgent);
    let all = view::items(&model);
    let mut visited = Vec::new();
    for _ in 0..all.len() {
        visited.push(model.work_selection().unwrap().clone());
        let _ = reduce_action(&mut model, Action::Next);
    }
    assert!(all.iter().all(|s| visited.contains(s)));
    for _ in 0..200 {
        let _ = reduce_action(&mut model, Action::ScrollDown);
    }
    let text = render_board(&model, 28, 24);
    assert!(text.contains("binding."));
    for width in 1..=32 {
        let _ = render_board(&model, width, 24);
    }
}

#[cfg(feature = "storybook")]
#[test]
fn production_board_review_exports_when_requested() {
    use questmancer::{
        scene::{
            pixel::{PixelSize, Rgb, RgbBuffer},
            presentation::ScenePresentation,
            render_scene_for_world,
            snapshot::SceneSnapshot,
        },
        storybook::fixtures::{StoryContext, WorkBoardState, quest_board_fixture},
        ui::{scene_adapter::flush_rgb, scene_overlays::render_scene_overlays},
    };
    let Ok(directory) = std::env::var("QUESTMANCER_BOARD_EXPORT") else {
        return;
    };
    std::fs::create_dir_all(&directory).unwrap();
    for (name, state, width, height) in [
        ("populated", WorkBoardState::Populated, 120, 36),
        ("cached", WorkBoardState::Populated, 120, 36),
        ("artifact", WorkBoardState::Populated, 120, 36),
        ("empty", WorkBoardState::Empty, 120, 36),
        ("offline", WorkBoardState::Offline, 120, 36),
        ("deferred", WorkBoardState::Deferred, 120, 36),
        ("resolved", WorkBoardState::Resolved, 120, 36),
        ("narrow", WorkBoardState::Narrow, 48, 28),
        ("worker-exited", WorkBoardState::WorkerExited, 120, 36),
    ] {
        let mut model = quest_board_fixture(StoryContext::fixed(), state);
        if name == "cached" {
            let mut p = model.work().clone();
            p.restore_cached();
            model.replace_work(p);
        }
        if name == "artifact" {
            model.select_work(WorkSelection::Quest(QuestId::new(
                &SourceId::new("fixture-auth"),
                "api",
            )));
        }
        let snapshot = SceneSnapshot::from_model(&model);
        let presentation = ScenePresentation::from_model(&model);
        let mut pixels = RgbBuffer::filled(width, height * 2, Rgb::BLACK);
        let mut terminal =
            ratatui::Terminal::new(ratatui::backend::TestBackend::new(width, height)).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                let _ = render_scene_for_world(
                    &snapshot,
                    &presentation,
                    PixelSize::new(width, height * 2),
                    &mut pixels,
                );
                flush_rgb(
                    frame.buffer_mut(),
                    area,
                    &pixels,
                    Rgb::BLACK,
                    model.preferences().color_mode,
                );
                render_scene_overlays(frame, &model, &presentation, None);
            })
            .unwrap();
        let cells=terminal.backend().buffer().content.iter().map(|c|serde_json::json!({"s":c.symbol(),"fg":format!("{:?}",c.fg),"bg":format!("{:?}",c.bg)})).collect::<Vec<_>>();
        std::fs::write(
            std::path::Path::new(&directory).join(format!("{name}.json")),
            serde_json::to_vec(&serde_json::json!({"width":width,"height":height,"cells":cells}))
                .unwrap(),
        )
        .unwrap();
    }
}

#[cfg(feature = "storybook")]
#[tokio::test]
async fn live_identity_is_omitted_and_startup_has_no_synthetic_source() {
    use questmancer::{
        config::PersistencePaths,
        persistence::{load_startup, publish_work},
        storybook::fixtures::{StoryContext, WorkBoardState, quest_board_fixture},
    };
    let dir = tempfile::tempdir().unwrap();
    let model = quest_board_fixture(StoryContext::fixed(), WorkBoardState::Populated);
    let state_dir = dir.path().join("state");
    let config_dir = dir.path().join("config");
    let startup = load_startup(
        PersistencePaths::from_lookup(|name| match name {
            "HERDR_PLUGIN_CONFIG_DIR" => Some(config_dir.display().to_string()),
            "HERDR_PLUGIN_STATE_DIR" => Some(state_dir.display().to_string()),
            _ => None,
        }),
        None,
    )
    .await;
    assert!(startup.model.work().sources.is_empty());
    publish_work(&state_dir.join("work-state.json"), model.work())
        .await
        .unwrap();
    let text = tokio::fs::read_to_string(state_dir.join("work-state.json"))
        .await
        .unwrap();
    assert!(!text.contains("fixture-terminal"));
    assert!(!text.contains("bindings"));
    assert!(!text.contains("freshness"));
    let startup = load_startup(
        PersistencePaths::from_lookup(|name| match name {
            "HERDR_PLUGIN_CONFIG_DIR" => Some(config_dir.display().to_string()),
            "HERDR_PLUGIN_STATE_DIR" => Some(state_dir.display().to_string()),
            _ => None,
        }),
        None,
    )
    .await;
    assert!(startup.diagnostics.is_empty());
    assert_eq!(
        startup
            .model
            .work()
            .sources
            .values()
            .next()
            .unwrap()
            .freshness,
        Freshness::Cached
    );
    assert!(
        startup
            .model
            .work()
            .sources
            .values()
            .all(|s| s.bindings.is_empty())
    );
}

#[cfg(feature = "storybook")]
#[test]
fn measure_work_attention_and_scene_capacity_at_one_ten_and_one_hundred() {
    use questmancer::{
        app::{Model, View},
        domain::{AgentKey, GuildAttention, PaneId, Presence},
        scene::{
            pixel::{PixelSize, Rgb, RgbBuffer},
            presentation::ScenePresentation,
            render_scene_for_world,
            snapshot::SceneSnapshot,
        },
        storybook::fixtures::{StoryContext, guild_world_fixture},
    };
    let party = guild_world_fixture(StoryContext::fixed());
    let seed = party.domain().agents.values().next().unwrap().clone();
    for count in [1, 10, 100] {
        let source = SourceId::new("load");
        let mut p = reduce(Projection::default(), baseline(&source)).unwrap();
        let example = p.sources[&source]
            .inventory
            .obligations
            .values()
            .next()
            .unwrap()
            .clone();
        p.sources
            .get_mut(&source)
            .unwrap()
            .inventory
            .obligations
            .clear();
        for i in 0..count {
            let mut o = example.clone();
            o.id = ObligationId::new(&source, &format!("request-{i:03}"));
            p.sources
                .get_mut(&source)
                .unwrap()
                .inventory
                .obligations
                .insert(o.id.clone(), o);
        }
        let mut model = Model::new(View::Guild);
        model.set_now(Timestamp::from_millis(200));
        model.replace_work(p);
        let mut campaign = party.domain().campaigns[&seed.workspace_id].clone();
        campaign.party.clear();
        model
            .domain_mut()
            .campaigns
            .insert(seed.workspace_id.clone(), campaign);
        for i in 0..count {
            let mut a = seed.clone();
            a.key = AgentKey::new(format!("load-{i}"));
            a.pane_id = PaneId::new(format!("load:{i}"));
            a.presence = Presence::Working;
            a.attention = GuildAttention::Clear;
            model
                .domain_mut()
                .campaigns
                .get_mut(&a.workspace_id)
                .unwrap()
                .party
                .push(a.key.clone());
            model.domain_mut().agents.insert(a.key.clone(), a);
        }
        model.open_quest_board();
        let start = std::time::Instant::now();
        for _ in 0..100 {
            assert_eq!(queue(&model).len(), count);
            let _ = render_board(&model, 120, 36);
        }
        let micros = start.elapsed().as_micros() / 100;
        let snapshot = SceneSnapshot::from_model(&model);
        let presentation = ScenePresentation::from_model(&model);
        let mut pixels = RgbBuffer::filled(160, 90, Rgb::BLACK);
        let scene = render_scene_for_world(
            &snapshot,
            &presentation,
            PixelSize::new(160, 90),
            &mut pixels,
        );
        assert!(scene.actors.len() <= 11);
        assert!(scene.actors.len() <= count);
        eprintln!(
            "M1 load: {count} agents / {count} obligations, queue + overlay average {micros} us, {} visible actor hit regions",
            scene.actors.len()
        );
    }
}

#[test]
fn an_unknown_close_is_a_tombstone_for_late_old_requests() {
    let source = SourceId::new("source");
    let mut first = baseline(&source);
    let mut old = None;
    if let Message::Baseline { inventory, .. } = &mut first {
        old = inventory.obligations.pop_first().map(|(_, o)| o);
    }
    let mut p = reduce(Projection::default(), first).unwrap();
    let old = old.unwrap();
    p = reduce(
        p,
        Message::Update {
            source: source.clone(),
            generation: 1,
            occurred_at: Timestamp::from_millis(101),
            cursor: 1,
            event: "close-unknown".into(),
            changes: vec![Change::Resolve {
                id: old.id.clone(),
                incarnation: old.incarnation.clone(),
                revision: 2,
            }],
        },
    )
    .unwrap();
    p = reduce(
        p,
        Message::Update {
            source,
            generation: 1,
            occurred_at: Timestamp::from_millis(102),
            cursor: 2,
            event: "late-request".into(),
            changes: vec![Change::Obligation(old)],
        },
    )
    .unwrap();
    assert!(p.open_decisions().is_empty());
}

#[test]
fn fixture_replay_is_equivalent_and_supervisor_waits_do_not_alert() {
    let events = fixture::FixtureSource::authentication(Timestamp::from_millis(100));
    let replay = |events: Vec<Message>| {
        events
            .into_iter()
            .try_fold(Projection::default(), reduce)
            .unwrap()
    };
    let expected = replay(events.clone());
    assert_eq!(replay(events.clone()), expected);
    let mut duplicate = expected.clone();
    for event in events {
        duplicate = reduce(duplicate, event).unwrap();
    }
    assert_eq!(duplicate, expected);
    let source = SourceId::new("fixture-auth");
    let id = ObligationId::new(&source, "compatibility");
    let mut o = duplicate.obligation(&id).unwrap().clone();
    o.responsible = Responsible::Supervisor;
    o.revision = 2;
    duplicate = reduce(
        duplicate,
        Message::Update {
            source,
            generation: 1,
            occurred_at: Timestamp::from_millis(102),
            cursor: 2,
            event: "supervisor-owned".into(),
            changes: vec![Change::Obligation(o)],
        },
    )
    .unwrap();
    assert_eq!(duplicate.open_decisions().len(), 1);
    assert!(
        duplicate
            .eligible_obligations(Timestamp::from_millis(200))
            .is_empty()
    );
}

async fn reconcile_composed_source(
    client: &questmancer::persistence::PersistenceClient,
    model: &mut questmancer::app::Model,
    source: &SourceId,
    now: Timestamp,
) {
    let baseline = Message::Baseline {
        source: source.clone(),
        generation: 1,
        captured_at: now,
        cursor: 1,
        inventory: model.work().sources[source].inventory.clone(),
        complete: true,
        bindings: BTreeMap::new(),
    };
    runtime::accept(
        client,
        model,
        source::Envelope {
            version: 1,
            message: baseline,
        },
    )
    .await
    .unwrap();
    runtime::accept(
        client,
        model,
        source::Envelope {
            version: 1,
            message: fixture::FixtureSource::resolution(now),
        },
    )
    .await
    .unwrap();
}

fn assert_resolved_work(model: &questmancer::app::Model, source: &SourceId, id: &ObligationId) {
    assert!(model.work().open_decisions().is_empty());
    assert_eq!(
        model.work().obligation(id).unwrap().state,
        ObligationState::Resolved
    );
    assert_eq!(
        model
            .work()
            .quest(&QuestId::new(source, "web"))
            .unwrap()
            .state,
        QuestState::Active
    );
}

#[test]
fn stale_generations_and_gaps_cannot_change_source_facts() {
    let source = SourceId::new("source");
    let mut p = reduce(Projection::default(), baseline(&source)).unwrap();
    let mut newer = baseline(&source);
    if let Message::Baseline { generation, .. } = &mut newer {
        *generation = 2;
    }
    p = reduce(p, newer).unwrap();
    assert_eq!(reduce(p.clone(), baseline(&source)).unwrap(), p);
    for (generation, cursor, expected) in [
        (1, 1, None),
        (2, 2, Some(WorkError::Reconcile)),
        (3, 1, Some(WorkError::Reconcile)),
    ] {
        let update = Message::Update {
            source: source.clone(),
            generation,
            occurred_at: Timestamp::from_millis(200),
            cursor,
            event: "late".into(),
            changes: vec![Change::Resolve {
                id: ObligationId::new(&source, "decision"),
                incarnation: "request-1".into(),
                revision: 2,
            }],
        };
        if let Some(error) = expected {
            assert_eq!(reduce(p.clone(), update), Err(error));
        } else {
            assert_eq!(reduce(p.clone(), update).unwrap(), p);
        }
    }
}

#[test]
fn attention_ranks_human_requests_then_age_and_identity() {
    let source = SourceId::new("source");
    let mut p = reduce(Projection::default(), baseline(&source)).unwrap();
    let original = p.sources[&source]
        .inventory
        .obligations
        .values()
        .next()
        .unwrap()
        .clone();
    p.sources
        .get_mut(&source)
        .unwrap()
        .inventory
        .obligations
        .clear();
    for (local, kind, responsible) in [
        (
            "z-blocked",
            ObligationKind::BlockedAction,
            Responsible::Human,
        ),
        ("a-review", ObligationKind::Review, Responsible::Human),
        ("z-decision", ObligationKind::Decision, Responsible::Human),
        ("a-decision", ObligationKind::Decision, Responsible::Human),
        (
            "recovery",
            ObligationKind::ExecutionAction,
            Responsible::Human,
        ),
        ("external", ObligationKind::Decision, Responsible::External),
    ] {
        let mut o = original.clone();
        o.id = ObligationId::new(&source, local);
        o.kind = kind;
        o.responsible = responsible;
        p.sources
            .get_mut(&source)
            .unwrap()
            .inventory
            .obligations
            .insert(o.id.clone(), o);
    }
    let expected = [
        "z-blocked",
        "a-decision",
        "z-decision",
        "recovery",
        "a-review",
    ]
    .map(|local| ObligationId::new(&source, local));
    assert_eq!(
        p.eligible_obligations(Timestamp::from_millis(200)),
        expected
    );
}

#[test]
fn missing_dependencies_and_completed_cycles_remain_unready() {
    let source = SourceId::new("fixture-auth");
    let id = QuestId::new(&source, "tests");
    let mut p = Projection::default();
    for e in fixture::FixtureSource::authentication(Timestamp::from_millis(100)) {
        p = reduce(p, e).unwrap();
    }
    let mut q = p.quest(&id).unwrap().clone();
    q.revision = 2;
    q.dependencies = vec![QuestId::new(&source, "missing")];
    p = reduce(
        p,
        Message::Update {
            source: source.clone(),
            generation: 1,
            occurred_at: Timestamp::from_millis(200),
            cursor: 2,
            event: "missing".into(),
            changes: vec![Change::Quest(q.clone())],
        },
    )
    .unwrap();
    assert!(!p.dependencies_ready(&id));
    q.revision = 3;
    q.state = QuestState::Completed;
    q.dependencies = vec![id.clone()];
    p = reduce(
        p,
        Message::Update {
            source,
            generation: 1,
            occurred_at: Timestamp::from_millis(201),
            cursor: 3,
            event: "cycle-completed".into(),
            changes: vec![Change::Quest(q)],
        },
    )
    .unwrap();
    assert!(!p.dependencies_ready(&id));
}

#[test]
fn board_inventory_wraps_complete_titles_and_keeps_selection_visible() {
    use questmancer::{
        app::{Model, View},
        interaction::reduce_action,
        ui::input::Action,
    };
    let mut model = Model::new(View::Guild);
    let mut p = Projection::default();
    for message in fixture::FixtureSource::authentication(Timestamp::from_millis(100)) {
        p = reduce(p, message).unwrap();
    }
    model.replace_work(p);
    let _ = reduce_action(&mut model, Action::NextUrgent);
    let labels = view::items(&model)
        .iter()
        .map(|item| view::label(&model, item))
        .collect::<Vec<_>>();
    let text = board_inventory_text(&model, 120, 38);
    for label in &labels {
        assert!(
            text.contains(label),
            "inventory title was cut: {label}; {text}"
        );
    }
    // Wrapped row heights, rather than item counts, must drive list scrolling.
    for _ in 0..labels.len() {
        let selected = view::label(&model, model.work_selection().unwrap());
        let text = board_inventory_text(&model, 80, 12);
        assert!(
            text.contains(&selected),
            "selected title was lost: {selected}; {text}"
        );
        let _ = reduce_action(&mut model, Action::Next);
    }
}

fn board_inventory_text(model: &questmancer::app::Model, width: u16, height: u16) -> String {
    let mut terminal =
        ratatui::Terminal::new(ratatui::backend::TestBackend::new(width, height)).unwrap();
    terminal
        .draw(|frame| questmancer::ui::work_board::render(frame, model))
        .unwrap();
    let buffer = terminal.backend().buffer();
    let board_width = width.saturating_sub(4).min(110);
    let board_height = height.saturating_sub(2).min(36);
    let left = (width - board_width) / 2 + 1;
    let top = (height - board_height) / 2 + 2;
    let rows = (top..top + board_height.saturating_sub(5)).map(|y| {
        (left + 2..left + 30)
            .map(|x| buffer[(x, y)].symbol())
            .collect::<String>()
            .trim()
            .to_owned()
    });
    rows.collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
