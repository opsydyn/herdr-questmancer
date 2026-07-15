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
