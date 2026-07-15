use questmancer::domain::{
    AgentKey, Guestbook, GuestbookEntry, GuestbookEvent, PaneId, Timestamp, WorkspaceId,
};

fn entry(kind: GuestbookEvent, pane: &str, revision: u64, at: i64) -> GuestbookEntry {
    GuestbookEntry::new(
        Timestamp::from_millis(at),
        Some(AgentKey::new("agent-1")),
        Some(WorkspaceId::new("w1")),
        Some(PaneId::new(pane)),
        revision,
        kind,
        "site changed",
    )
}

#[test]
fn event_id_is_stable_for_the_documented_identity_fields() {
    let first = entry(GuestbookEvent::WorkCompleted, "w1:p1", 7, 1_000);
    let second = entry(GuestbookEvent::WorkCompleted, "w1:p1", 7, 1_000);
    let different_revision = entry(GuestbookEvent::WorkCompleted, "w1:p1", 8, 1_000);

    assert_eq!(first.id, second.id);
    assert_ne!(first.id, different_revision.id);
}

#[test]
fn duplicate_entries_are_rejected() {
    let mut guestbook = Guestbook::new(10);
    let entry = entry(GuestbookEvent::WebmasterNeeded, "w1:p1", 2, 500);

    assert!(guestbook.append(entry.clone()));
    assert!(!guestbook.append(entry));
    assert_eq!(guestbook.entries().len(), 1);
}

#[test]
fn entries_remain_chronological_and_evict_the_oldest() {
    let mut guestbook = Guestbook::new(2);
    guestbook.append(entry(GuestbookEvent::WorkStarted, "w1:p1", 1, 100));
    guestbook.append(entry(GuestbookEvent::WebmasterNeeded, "w1:p1", 2, 200));
    guestbook.append(entry(GuestbookEvent::WorkCompleted, "w1:p1", 3, 300));

    let entries = guestbook.entries();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].occurred_at, Timestamp::from_millis(200));
    assert_eq!(entries[1].occurred_at, Timestamp::from_millis(300));
}
