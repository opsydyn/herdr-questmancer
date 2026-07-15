use std::path::Path;

use proptest::prelude::*;
use questmancer::{
    domain::{Chronicle, ChronicleEntry, ChronicleEvent, EventId, Timestamp},
    persistence::{append_chronicle, load_chronicle, replay_chronicle},
};
use tempfile::tempdir;

#[derive(Clone, Debug)]
enum ReplayRecord {
    Valid(ChronicleEntry),
    Duplicate(ChronicleEntry),
    Invalid(Vec<u8>),
}

fn entry(id: &str, occurred_at: i64) -> ChronicleEntry {
    ChronicleEntry {
        id: EventId::new(id),
        occurred_at: Timestamp::from_millis(occurred_at),
        adventurer: None,
        campaign: None,
        pane: None,
        pane_revision: 0,
        event: ChronicleEvent::SpoilsReturned,
        summary: format!("entry {id}"),
    }
}

fn jsonl(entries: &[ChronicleEntry]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for entry in entries {
        bytes.extend(serde_json::to_vec(entry).unwrap());
        bytes.push(b'\n');
    }
    bytes
}

fn replay_entry() -> impl Strategy<Value = ChronicleEntry> {
    (0_u8..16, any::<i64>(), any::<u16>()).prop_map(|(id, occurred_at, pane_revision)| {
        ChronicleEntry {
            id: EventId::new(format!("event-{id}")),
            occurred_at: Timestamp::from_millis(occurred_at),
            adventurer: None,
            campaign: None,
            pane: None,
            pane_revision: u64::from(pane_revision),
            event: ChronicleEvent::SpoilsReturned,
            summary: format!("entry {id}"),
        }
    })
}

fn replay_record() -> impl Strategy<Value = ReplayRecord> {
    prop_oneof![
        4 => replay_entry().prop_map(ReplayRecord::Valid),
        2 => replay_entry().prop_map(ReplayRecord::Duplicate),
        2 => prop::collection::vec(any::<u8>().prop_filter("record bytes exclude the JSONL delimiter", |byte| *byte != b'\n'), 0..32)
            .prop_map(|mut bytes| {
                bytes.insert(0, 0xff);
                ReplayRecord::Invalid(bytes)
            }),
    ]
}

#[test]
fn replay_preserves_chronicle_order_deduplication_and_bound() {
    let latest = entry("latest", 300);
    let oldest = entry("oldest", 100);
    let middle = entry("middle", 200);
    let bytes = jsonl(&[latest.clone(), oldest, latest.clone(), middle.clone()]);

    let replay = replay_chronicle(Path::new("chronicle.jsonl"), &bytes, 2);

    assert!(replay.diagnostics.is_empty());
    assert_eq!(
        replay.chronicle.entries().iter().collect::<Vec<_>>(),
        vec![&middle, &latest]
    );
}

#[test]
fn schema_invalid_json_and_malformed_utf8_do_not_hide_valid_history() {
    let first = entry("first", 100);
    let second = entry("second", 200);
    let mut bytes = jsonl(std::slice::from_ref(&first));
    bytes.extend(b"{\"bad\":true}\n\xff\n");
    bytes.extend(jsonl(std::slice::from_ref(&second)));

    let replay = replay_chronicle(Path::new("chronicle.jsonl"), &bytes, 500);

    assert_eq!(
        replay.chronicle.entries().iter().collect::<Vec<_>>(),
        vec![&first, &second]
    );
    assert_eq!(replay.diagnostics.len(), 2);
    assert_eq!(replay.diagnostics[0].line, Some(2));
    assert_eq!(replay.diagnostics[1].line, Some(3));
}

#[test]
fn non_newline_terminated_final_record_is_rejected_as_truncated() {
    let complete_entry = entry("complete", 50);
    let final_entry = entry("final", 100);
    let mut bytes = jsonl(std::slice::from_ref(&complete_entry));
    bytes.extend(serde_json::to_vec(&final_entry).unwrap());

    let replay = replay_chronicle(Path::new("chronicle.jsonl"), &bytes, 500);

    assert_eq!(
        replay.chronicle.entries().iter().collect::<Vec<_>>(),
        vec![&complete_entry]
    );
    assert_eq!(replay.diagnostics.len(), 1);
    assert_eq!(replay.diagnostics[0].line, Some(2));
    assert!(replay.diagnostics[0].source_message.contains("truncated"));
}

#[test]
fn replay_folds_diagnostics_after_five_rejected_records() {
    let bytes = b"bad\nbad\nbad\nbad\nbad\nbad\nbad\n";

    let replay = replay_chronicle(Path::new("chronicle.jsonl"), bytes, 500);

    assert_eq!(replay.diagnostics.len(), 6);
    assert_eq!(
        replay.diagnostics[..5]
            .iter()
            .map(|diagnostic| diagnostic.line)
            .collect::<Vec<_>>(),
        vec![Some(1), Some(2), Some(3), Some(4), Some(5)]
    );
    assert_eq!(replay.diagnostics[5].line, None);
    assert!(
        replay.diagnostics[5]
            .source_message
            .contains("2 additional")
    );
}

#[tokio::test]
async fn absent_chronicle_loads_as_empty_without_diagnostics() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("missing/chronicle.jsonl");

    let replay = load_chronicle(&path, 500).await;

    assert!(replay.chronicle.entries().is_empty());
    assert!(replay.diagnostics.is_empty());
}

#[tokio::test]
async fn append_writes_one_compact_record_and_one_newline() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("chronicle.jsonl");
    let entry = entry("first", 100);

    append_chronicle(&path, &entry).await.unwrap();

    let bytes = tokio::fs::read(&path).await.unwrap();
    let mut expected = serde_json::to_vec(&entry).unwrap();
    expected.push(b'\n');
    assert_eq!(bytes, expected);
}

#[tokio::test]
async fn append_creates_missing_parent_directories() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("nested/state/chronicle.jsonl");
    let entry = entry("first", 100);

    append_chronicle(&path, &entry).await.unwrap();

    assert!(path.is_file());
}

#[tokio::test]
async fn multiple_appends_remain_in_write_order() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("chronicle.jsonl");
    let first = entry("first", 200);
    let second = entry("second", 100);

    append_chronicle(&path, &first).await.unwrap();
    append_chronicle(&path, &second).await.unwrap();

    assert_eq!(
        tokio::fs::read(&path).await.unwrap(),
        jsonl(&[first, second])
    );
}

#[tokio::test]
async fn append_failure_reports_the_chronicle_path() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("chronicle.jsonl");
    tokio::fs::create_dir(&path).await.unwrap();

    let error = append_chronicle(&path, &entry("first", 100))
        .await
        .unwrap_err();

    assert_eq!(error.operation, "open chronicle");
    assert_eq!(error.path, path);
    assert_eq!(error.line, None);
    assert!(!error.source_message.is_empty());
}

proptest! {
    #[test]
    fn arbitrary_record_interleavings_match_a_chronicle_fold(
        records in prop::collection::vec(replay_record(), 0..100),
        maximum_entries in 1_usize..100,
    ) {
        let mut bytes = Vec::new();
        let mut expected = Chronicle::new(maximum_entries);
        for record in records {
            match record {
                ReplayRecord::Valid(entry) => {
                    bytes.extend(serde_json::to_vec(&entry).unwrap());
                    bytes.push(b'\n');
                    expected.append(entry);
                }
                ReplayRecord::Duplicate(entry) => {
                    let serialized = serde_json::to_vec(&entry).unwrap();
                    bytes.extend(&serialized);
                    bytes.push(b'\n');
                    bytes.extend(serialized);
                    bytes.push(b'\n');
                    expected.append(entry.clone());
                    expected.append(entry);
                }
                ReplayRecord::Invalid(record) => {
                    bytes.extend(record);
                    bytes.push(b'\n');
                }
            }
        }

        let replay = replay_chronicle(Path::new("chronicle.jsonl"), &bytes, maximum_entries);
        let entries = replay.chronicle.entries();
        let unique_ids = entries
            .iter()
            .map(|entry| entry.id.clone())
            .collect::<std::collections::BTreeSet<_>>();

        prop_assert_eq!(unique_ids.len(), entries.len());
        prop_assert!(entries.len() <= maximum_entries);
        let chronological = entries.iter().zip(entries.iter().skip(1)).all(|(first, second)| {
            first.occurred_at <= second.occurred_at
        });
        prop_assert!(chronological);
        prop_assert_eq!(replay.chronicle, expected);
    }
}
