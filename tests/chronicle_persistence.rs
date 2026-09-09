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
    questmancer::domain::LegacyChronicleEntry {
        id: EventId::new(id),
        occurred_at: Timestamp::from_millis(occurred_at),
        adventurer: None,
        campaign: None,
        pane: None,
        pane_revision: 0,
        event: ChronicleEvent::SpoilsReturned,
        summary: format!("entry {id}"),
    }
    .into()
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
        questmancer::domain::LegacyChronicleEntry {
            id: EventId::new(format!("event-{id}")),
            occurred_at: Timestamp::from_millis(occurred_at),
            adventurer: None,
            campaign: None,
            pane: None,
            pane_revision: u64::from(pane_revision),
            event: ChronicleEvent::SpoilsReturned,
            summary: format!("entry {id}"),
        }
        .into()
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

fn observation(ordinal: u64, at: i64) -> ChronicleEntry {
    use questmancer::{
        domain::{
            CaptureRunId, CapturedObservation, ObservationEvidence, ObservationStamp,
            ObservationSubject, RemovalEvidence, SnapshotObservation, WorkspaceId,
        },
        snapshot_refresh::{
            ConnectionEpoch, LiveFactGeneration, SnapshotRequest, SnapshotRequestId,
        },
    };
    ChronicleEntry::observed(
        Timestamp::from_millis(at),
        CapturedObservation {
            stamp: ObservationStamp {
                run: CaptureRunId::new("replay-fixture"),
                epoch: ConnectionEpoch(1),
                ordinal,
            },
            subject: ObservationSubject::Campaign {
                id: WorkspaceId::new("campaign-a"),
                name: "Archive".into(),
            },
            evidence: ObservationEvidence::Snapshot {
                request: SnapshotRequest {
                    epoch: ConnectionEpoch(1),
                    id: SnapshotRequestId(1),
                    generation: LiveFactGeneration(1),
                },
                observation: SnapshotObservation::CampaignRemoved {
                    evidence: RemovalEvidence::NoLongerVisible,
                },
            },
        },
    )
    .unwrap()
}

#[test]
fn mixed_v1_v2_replay_preserves_legacy_ids_wording_and_observation_evidence() {
    // Use an independently formed flat legacy record: do not infer its meaning
    // from the old joined category or reinterpret its summary on replay.
    let legacy_wire = serde_json::json!({"id":"old-joined-id","occurred_at":100,"adventurer":null,"campaign":null,"pane":null,"pane_revision":0,"event":"adventurer_joined","summary":"whereabouts unknown"});
    let legacy: ChronicleEntry = serde_json::from_value(legacy_wire).unwrap();
    let observed = observation(1, 200);
    let bytes = jsonl(&[
        legacy.clone(),
        observed.clone(),
        legacy.clone(),
        observed.clone(),
    ]);
    let replay = replay_chronicle(Path::new("chronicle.jsonl"), &bytes, 500);
    assert!(replay.diagnostics.is_empty());
    assert_eq!(
        replay
            .chronicle
            .entries()
            .iter()
            .cloned()
            .collect::<Vec<_>>(),
        vec![legacy, observed]
    );
    assert_eq!(replay.chronicle.entries()[0].id.as_str(), "old-joined-id");
    assert_eq!(replay.chronicle.entries()[0].summary, "whereabouts unknown");
    assert_eq!(
        replay.chronicle.entries()[0].event(),
        ChronicleEvent::AdventurerJoined
    );
    assert!(replay.chronicle.entries()[1].observation().is_some());
    assert_eq!(replay.chronicle.entries()[1].event().experience(), 0);
}

#[test]
fn a_v2_envelope_cannot_silently_decode_as_a_reward_bearing_v1_record() {
    let value = serde_json::to_value(observation(1, 100)).unwrap();
    assert_eq!(value["record_version"], 2);
    assert!(value.get("occurred_at").is_none());
    assert!(value.get("event").is_none());
    assert!(serde_json::from_value::<questmancer::domain::LegacyChronicleEntry>(value).is_err());
}

#[test]
fn unknown_v2_versions_payloads_and_invalid_source_context_are_bounded_diagnostics() {
    let valid = observation(1, 100);
    let mut bytes = jsonl(std::slice::from_ref(&valid));
    let baseline = serde_json::to_value(&valid).unwrap();
    let mut rejected = Vec::new();
    let mut value = baseline.clone();
    value["record_version"] = 3.into();
    rejected.push(value);
    let mut value = baseline.clone();
    value["observation"]["evidence"]["observation"]["kind"] = "future_observation".into();
    rejected.push(value);
    let mut value = baseline.clone();
    value["observation"]["evidence"]["source"] = "future_source".into();
    rejected.push(value);
    let mut value = baseline.clone();
    value["observation"]["stamp"]["epoch"] = 0.into();
    rejected.push(value);
    let mut value = baseline.clone();
    value["observation"]["stamp"]["ordinal"] = 0.into();
    rejected.push(value);
    let mut value = baseline.clone();
    value["observation"]["evidence"]["request"]["epoch"] = 2.into();
    rejected.push(value);
    let mut value = baseline;
    value["id"] = "recomputed-from-the-clock".into();
    rejected.push(value);
    for value in rejected {
        bytes.extend(serde_json::to_vec(&value).unwrap());
        bytes.push(b'\n');
    }
    let last = observation(2, 200);
    bytes.extend(jsonl(std::slice::from_ref(&last)));
    bytes.extend(b"{\"record_version\":2");
    let replay = replay_chronicle(Path::new("chronicle.jsonl"), &bytes, 500);
    assert_eq!(
        replay
            .chronicle
            .entries()
            .iter()
            .cloned()
            .collect::<Vec<_>>(),
        vec![valid, last]
    );
    assert_eq!(replay.diagnostics.len(), 6);
    assert_eq!(replay.diagnostics[0].line, Some(2));
    assert!(
        replay.diagnostics[5]
            .source_message
            .contains("3 additional rejected")
    );
}

#[tokio::test]
async fn appending_v2_after_legacy_records_does_not_rewrite_the_existing_bytes() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("chronicle.jsonl");
    let old_bytes = b"{\"id\":\"old\",\"occurred_at\":50,\"adventurer\":null,\"campaign\":null,\"pane\":null,\"pane_revision\":0,\"event\":\"adventurer_joined\",\"summary\":\"whereabouts unknown\"}\n{\"record_version\":99}\n";
    tokio::fs::write(&path, old_bytes).await.unwrap();
    let observed = observation(1, 100);
    append_chronicle(&path, &observed).await.unwrap();
    let after = tokio::fs::read(&path).await.unwrap();
    assert!(after.starts_with(old_bytes));
    let replay = load_chronicle(&path, 500).await;
    assert_eq!(replay.chronicle.entries().len(), 2);
    assert_eq!(replay.chronicle.entries()[1], observed);
    assert_eq!(replay.diagnostics.len(), 1);
    assert_eq!(tokio::fs::read(&path).await.unwrap(), after);
}

#[test]
fn immutable_v2_identity_survives_timestamp_and_summary_changes() {
    let original = observation(1, 100);
    let at_other_time = observation(1, -10_000);
    assert_eq!(original.id, at_other_time.id);
    let mut renamed = original.clone();
    renamed.summary = "A captured summary remains independent of identity".into();
    let decoded: ChronicleEntry =
        serde_json::from_slice(&serde_json::to_vec(&renamed).unwrap()).unwrap();
    assert_eq!(decoded.id, original.id);
    assert_eq!(decoded.summary, renamed.summary);
    assert_eq!(decoded.observation(), original.observation());
}

#[test]
fn new_observation_categories_cannot_be_written_or_read_as_flat_legacy_records() {
    let fake_legacy = ChronicleEntry::new(
        Timestamp::from_millis(1),
        None,
        None,
        None,
        0,
        ChronicleEvent::PresenceObserved,
        "unqualified",
    );
    assert!(serde_json::to_vec(&fake_legacy).is_err());
    let flat = br#"{"id":"new","occurred_at":1,"adventurer":null,"campaign":null,"pane":null,"pane_revision":0,"event":"presence_observed","summary":"unqualified"}"#;
    assert!(serde_json::from_slice::<ChronicleEntry>(flat).is_err());
}
