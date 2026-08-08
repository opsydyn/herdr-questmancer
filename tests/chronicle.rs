use questmancer::domain::{
    AgentKey, Chronicle, ChronicleEntry, ChronicleEvent, PaneId, Timestamp, WorkspaceId,
};

fn entry(event: ChronicleEvent, pane: &str, revision: u64, at: i64) -> ChronicleEntry {
    ChronicleEntry::new(
        Timestamp::from_millis(at),
        Some(AgentKey::new("agent-1")),
        Some(WorkspaceId::new("w1")),
        Some(PaneId::new(pane)),
        revision,
        event,
        "site changed",
    )
}

#[test]
fn event_id_is_stable_for_the_documented_identity_fields() {
    let first = entry(ChronicleEvent::SpoilsReturned, "w1:p1", 7, 1_000);
    let second = entry(ChronicleEvent::SpoilsReturned, "w1:p1", 7, 1_000);
    let different_revision = entry(ChronicleEvent::SpoilsReturned, "w1:p1", 8, 1_000);

    assert_eq!(first.id, second.id);
    assert_ne!(first.id, different_revision.id);
}

#[test]
fn duplicate_entries_are_rejected() {
    let mut chronicle = Chronicle::new(10);
    let entry = entry(ChronicleEvent::CounselRequested, "w1:p1", 2, 500);

    assert!(chronicle.append(entry.clone()));
    assert!(!chronicle.append(entry));
    assert_eq!(chronicle.entries().len(), 1);
}

#[test]
fn entries_remain_chronological_and_evict_the_oldest() {
    let mut chronicle = Chronicle::new(2);
    chronicle.append(entry(ChronicleEvent::DelveBegan, "w1:p1", 1, 100));
    chronicle.append(entry(ChronicleEvent::CounselRequested, "w1:p1", 2, 200));
    chronicle.append(entry(ChronicleEvent::SpoilsReturned, "w1:p1", 3, 300));

    let entries = chronicle.entries();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].occurred_at, Timestamp::from_millis(200));
    assert_eq!(entries[1].occurred_at, Timestamp::from_millis(300));
}

/// Standing is earned where the Chronicle decides an event is new, so the
/// dedup that protects the history is the same dedup that protects the score.
/// The same returned spoils cannot be paid for twice.
#[test]
fn a_repeated_event_is_worth_standing_only_once() {
    use questmancer::domain::ChronicleEvent;

    let mut chronicle = Chronicle::new(64);
    let entry = ChronicleEntry::new(
        Timestamp::from_millis(1_000),
        None,
        None,
        Some(PaneId::new("w1:p1")),
        7,
        ChronicleEvent::SpoilsReturned,
        "codex returned with spoils",
    );

    let mut earned = 0_u64;
    for _ in 0..5 {
        if chronicle.append(entry.clone()) {
            earned += entry.event.experience();
        }
    }

    assert_eq!(chronicle.entries().len(), 1, "history records it once");
    assert_eq!(
        earned,
        ChronicleEvent::SpoilsReturned.experience(),
        "and it is worth standing once"
    );
}

/// The Chronicle is a bounded ring, so standing must never be recomputed from
/// it: rolling history off would make the score fall. This proves the decay a
/// derived score would suffer, which is why the counter is stored instead.
#[test]
fn the_chronicle_forgets_but_standing_must_not() {
    use questmancer::domain::ChronicleEvent;

    let mut chronicle = Chronicle::new(3);
    for index in 0..10 {
        chronicle.append(ChronicleEntry::new(
            Timestamp::from_millis(1_000 + index),
            None,
            None,
            Some(PaneId::new(format!("w1:p{index}"))),
            u64::try_from(index).unwrap_or(0),
            ChronicleEvent::SpoilsReturned,
            "spoils",
        ));
    }

    let derived_from_history: u64 = chronicle
        .entries()
        .iter()
        .map(|entry| entry.event.experience())
        .sum();
    let actually_earned = 10 * ChronicleEvent::SpoilsReturned.experience();

    assert_eq!(chronicle.entries().len(), 3, "the ring dropped the rest");
    assert!(
        derived_from_history < actually_earned,
        "a score derived from the Chronicle would have fallen from {actually_earned} \
         to {derived_from_history}; standing is stored precisely so it cannot"
    );
}
