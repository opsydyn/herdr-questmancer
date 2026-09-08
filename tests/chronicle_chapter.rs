use questmancer::{
    chronicle_chapter::{ChapterRequest, ChapterWindow},
    domain::{AgentKey, Chronicle, ChronicleEntry, ChronicleEvent, PaneId, Timestamp},
};

fn entry(event: ChronicleEvent, agent: &str, revision: u64, at: i64) -> ChronicleEntry {
    ChronicleEntry::new(
        Timestamp::from_millis(at),
        Some(AgentKey::new(agent)),
        None,
        Some(PaneId::new(agent)),
        revision,
        event,
        format!("{agent} {}", event.label()),
    )
}

fn request() -> ChapterRequest {
    ChapterRequest {
        window: ChapterWindow::last_hour(Timestamp::from_millis(3_600_000)),
        adventurer: None,
    }
}

#[test]
fn chapter_uses_inclusive_explicit_bounds_and_never_counts_future_events() {
    let mut chronicle = Chronicle::new(10);
    for (revision, at) in [(-1_i64), 0, 3_600_000, 3_600_001].into_iter().enumerate() {
        chronicle.append(entry(
            ChronicleEvent::SpoilsReturned,
            "Ari",
            revision as u64,
            at,
        ));
    }
    let request = request();
    let chapter = request.project(&chronicle);
    assert_eq!(chapter.count(ChronicleEvent::SpoilsReturned), 2);
    assert_eq!(
        chapter
            .sources()
            .iter()
            .map(|e| e.occurred_at.as_millis())
            .collect::<Vec<_>>(),
        [3_600_000, 0]
    );
}

#[test]
fn repeats_count_events_and_source_order_is_stable_across_arrival_order() {
    let events = [
        entry(ChronicleEvent::SpoilsReturned, "Ari", 1, 500),
        entry(ChronicleEvent::SpoilsReturned, "Ari", 2, 500),
        entry(ChronicleEvent::SpoilsReturned, "Bea", 1, 500),
    ];
    let mut forward = Chronicle::new(10);
    let mut reverse = Chronicle::new(10);
    for e in &events {
        forward.append(e.clone());
        forward.append(e.clone());
    }
    for e in events.iter().rev() {
        reverse.append(e.clone());
    }
    let request = request();
    let chapter = request.project(&forward);
    assert_eq!(chapter.sources().len(), 3);
    assert_eq!(chapter.count(ChronicleEvent::SpoilsReturned), 3);
    assert_eq!(chapter.lines(), request.project(&reverse).lines());
    let text = chapter.lines().join("\n");
    assert!(text.contains("3 spoils-return events"));
    assert!(text.contains("Ari returned with spoils"));
    assert!(text.contains("Bea returned with spoils"));
    assert!(!text.contains("3 adventurers"));
}

#[test]
fn bounded_history_and_empty_windows_make_no_claim_of_completeness() {
    let mut chronicle = Chronicle::new(1);
    chronicle.append(entry(ChronicleEvent::SpoilsReturned, "Ari", 1, 100));
    chronicle.append(entry(ChronicleEvent::CampaignClosed, "Ari", 2, 200));
    let request = request();
    let chapter = request.project(&chronicle);
    assert_eq!(chapter.count(ChronicleEvent::SpoilsReturned), 0);
    let lines = chapter.lines().join("\n");
    assert!(lines.contains("1 campaign-closure event"));
    assert!(lines.contains("Retained records only"));
    assert!(!lines.contains("success"));
    let empty = ChapterRequest {
        window: ChapterWindow::last_hour(Timestamp::from_millis(8_000_000)),
        adventurer: None,
    };
    let lines = empty.project(&chronicle).lines().join("\n");
    assert!(lines.contains("No retained events in this window."));
    assert!(!lines.contains("nothing happened"));
}

#[test]
fn every_event_type_has_a_factual_template_and_traceable_source() {
    let mut chronicle = Chronicle::new(10);
    for (revision, event) in ChronicleEvent::ALL.iter().enumerate() {
        let mut source = entry(*event, "Ari", revision as u64, 500);
        source.summary.clear();
        chronicle.append(source);
    }
    let request = request();
    let chapter = request.project(&chronicle);
    let lines = chapter.lines().join("\n");
    for event in ChronicleEvent::ALL {
        assert_eq!(chapter.count(*event), 1);
        if *event == ChronicleEvent::AdventurerJoined {
            assert!(lines.contains("identity event (summary unavailable)"));
        } else {
            assert!(lines.contains(event.label()));
        }
    }
    for source in chapter.sources() {
        assert!(lines.contains(source.id.as_str()));
    }
}

#[test]
fn timestamp_extremes_are_saturating_and_honest() {
    for at in [i64::MIN, i64::MAX] {
        let mut chronicle = Chronicle::new(1);
        chronicle.append(entry(ChronicleEvent::AdventurerJoined, "Ari", 1, at));
        let request = ChapterRequest {
            window: ChapterWindow::last_hour(Timestamp::from_millis(at)),
            adventurer: None,
        };
        assert!(request.window.from() <= request.window.through());
        let text = request.project(&chronicle).lines().join("\n");
        assert!(text.contains(&format!("epoch ms {at}")));
    }
}

#[test]
fn unknown_whereabouts_are_not_rewritten_as_an_arrival() {
    use questmancer::{
        domain::DomainState,
        herdr::protocol::{AgentStatus, SessionSnapshotResult, SuccessResponse},
        update::{AppEvent, update},
    };
    let response: SuccessResponse<SessionSnapshotResult> =
        serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
    let domain = DomainState::from_snapshot(&response.result.snapshot, Timestamp::from_millis(0));
    let (domain, _) = update(
        domain,
        AppEvent::AgentStatusChanged {
            pane_id: PaneId::new("w1:p1"),
            status: AgentStatus::Unknown,
            custom_status: None,
            revision: 8,
            occurred_at: Timestamp::from_millis(500),
        },
    );
    let request = request();
    let chapter = request.project(&domain.chronicle);
    assert_eq!(chapter.sources().len(), 1);
    let text = chapter.lines().join("\n");
    assert!(text.contains("whereabouts unknown"));
    assert!(text.contains("1 identity event"));
    assert!(!text.contains("arrival"));
    assert!(!text.contains("joined"));
}

#[test]
fn long_recorded_names_remain_complete_across_scrollable_lines() {
    use questmancer::chronicle_chapter::CHAPTER_COLUMNS;
    let mut chronicle = Chronicle::new(1);
    let mut source = entry(ChronicleEvent::SpoilsReturned, "Ari", 1, 100);
    source.summary = "冒険者 Ari 🧙 ".repeat(25);
    let summary = source.summary.clone();
    chronicle.append(source);
    let request = request();
    let lines = request.project(&chronicle).lines();
    assert!(
        lines
            .iter()
            .all(|line| ratatui::text::Line::from(line.as_str()).width()
                <= usize::from(CHAPTER_COLUMNS - 2))
    );
    let begin = lines
        .iter()
        .position(|line| line.starts_with("* 1970"))
        .unwrap()
        + 1;
    let end = lines
        .iter()
        .position(|line| line.starts_with("Source:"))
        .unwrap();
    assert_eq!(lines[begin..end].concat(), summary);
}
