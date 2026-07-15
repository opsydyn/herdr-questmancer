#[allow(dead_code, unused_imports)]
mod support;

use std::fs;

use proptest::prelude::*;
use questmancer::{
    app::{Model, View},
    domain::PersonaKey,
    persistence::{AttentionEpisodeKey, PersistedStateV1, load_state, parse_state, publish_state},
};
use tempfile::tempdir;

fn valid_state() -> PersistedStateV1 {
    let mut model = Model::new(View::Delve);
    model.replace_domain(support::fixture_domain());
    model.mark_selected_attention_read();
    PersistedStateV1::capture(&model)
}

#[tokio::test]
async fn missing_state_file_loads_as_none() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("state.json");

    assert_eq!(load_state(&path).await.unwrap(), None);
}

#[tokio::test]
async fn non_missing_read_failure_is_a_path_bearing_diagnostic() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("state.json");
    let temporary_path = directory.path().join("state.json.tmp");
    let leftover = b"leftover temporary bytes";
    fs::create_dir(&path).unwrap();
    fs::write(&temporary_path, leftover).unwrap();

    let error = load_state(&path).await.unwrap_err();

    assert_eq!(error.operation, "read state");
    assert_eq!(error.path, path);
    assert_eq!(error.line, None);
    assert!(!error.source_message.is_empty());
    assert!(path.is_dir());
    assert_eq!(fs::read(temporary_path).unwrap(), leftover);
}

#[test]
fn valid_v1_bytes_parse() {
    let state = valid_state();
    let bytes = serde_json::to_vec(&state).unwrap();

    assert_eq!(parse_state("state.json".as_ref(), &bytes).unwrap(), state);
}

#[test]
fn malformed_json_reports_the_path_and_one_based_line() {
    let path = std::path::Path::new("/tmp/herdr/state.json");
    let error = parse_state(path, b"{\n  \"schema_version\": 1,\n  nope\n}").unwrap_err();

    assert_eq!(error.operation, "parse state");
    assert_eq!(error.path, path);
    assert_eq!(error.line, Some(3));
    assert!(!error.source_message.is_empty());
}

#[test]
fn unsupported_schema_version_fails_closed() {
    let mut state = valid_state();
    state.schema_version = 2;

    let error =
        parse_state("state.json".as_ref(), &serde_json::to_vec(&state).unwrap()).unwrap_err();

    assert_eq!(error.operation, "validate state");
    assert_eq!(error.line, None);
    assert!(error.source_message.contains("schema version 2"));
}

#[test]
fn mismatched_embedded_persona_key_fails_closed() {
    let mut state = valid_state();
    let map_key = state.personas.keys().next().unwrap().clone();
    state.personas.get_mut(&map_key).unwrap().key = PersonaKey::new("persona-other");

    let error =
        parse_state("state.json".as_ref(), &serde_json::to_vec(&state).unwrap()).unwrap_err();

    assert_eq!(error.operation, "validate state");
    assert!(error.source_message.contains("does not match embedded key"));
}

#[test]
fn selected_persona_missing_from_the_map_fails_closed() {
    let mut state = valid_state();
    state.selected_persona = Some(PersonaKey::new("persona-missing"));

    let error =
        parse_state("state.json".as_ref(), &serde_json::to_vec(&state).unwrap()).unwrap_err();

    assert_eq!(error.operation, "validate state");
    assert!(
        error
            .source_message
            .contains("missing from the persona map")
    );
}

#[test]
fn seen_attention_persona_missing_from_the_map_fails_closed() {
    let mut state = valid_state();
    let episode = state.seen_attention.iter().next().unwrap().clone();
    state.seen_attention.insert(AttentionEpisodeKey {
        persona: PersonaKey::new("persona-missing"),
        ..episode
    });

    let error =
        parse_state("state.json".as_ref(), &serde_json::to_vec(&state).unwrap()).unwrap_err();

    assert_eq!(error.operation, "validate state");
    assert!(
        error
            .source_message
            .contains("seen attention persona persona-missing")
    );
}

#[tokio::test]
async fn invalid_state_file_is_preserved_exactly() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("state.json");
    let temporary_path = directory.path().join("state.json.tmp");
    let invalid = b"{ definitely not state json }";
    let leftover = b"leftover temporary bytes";
    fs::write(&path, invalid).unwrap();
    fs::write(&temporary_path, leftover).unwrap();

    let error = load_state(&path).await.unwrap_err();

    assert_eq!(error.operation, "parse state");
    assert_eq!(error.path, path);
    assert_eq!(fs::read(&path).unwrap(), invalid);
    assert_eq!(fs::read(&temporary_path).unwrap(), leftover);
}

#[tokio::test]
async fn publication_writes_pretty_json_with_a_trailing_newline() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("state.json");
    let state = valid_state();

    publish_state(&path, &state).await.unwrap();

    let mut expected = serde_json::to_string_pretty(&state).unwrap();
    expected.push('\n');
    assert_eq!(fs::read_to_string(path).unwrap(), expected);
}

#[tokio::test]
async fn publication_atomically_replaces_an_existing_valid_document() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("state.json");
    let old = valid_state();
    let mut new = old.clone();
    new.last_view = View::Guild;
    fs::write(&path, serde_json::to_vec(&old).unwrap()).unwrap();

    publish_state(&path, &new).await.unwrap();

    assert_eq!(load_state(&path).await.unwrap(), Some(new));
}

#[tokio::test]
async fn publication_creates_a_missing_parent_directory() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("nested/state/state.json");
    let state = valid_state();

    publish_state(&path, &state).await.unwrap();

    assert_eq!(load_state(&path).await.unwrap(), Some(state));
}

#[tokio::test]
async fn publication_reuses_and_cleans_up_a_leftover_temporary_file() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("state.json");
    let temporary_path = directory.path().join("state.json.tmp");
    let state = valid_state();
    fs::write(&temporary_path, b"partial stale json").unwrap();

    publish_state(&path, &state).await.unwrap();

    assert_eq!(load_state(&path).await.unwrap(), Some(state));
    assert!(!temporary_path.exists());
}

#[tokio::test]
async fn temporary_file_creation_failure_retains_the_prior_destination() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("state.json");
    let temporary_path = directory.path().join("state.json.tmp");
    let old = valid_state();
    let mut new = old.clone();
    new.last_view = View::Guild;
    let old_bytes = serde_json::to_vec_pretty(&old).unwrap();
    fs::write(&path, &old_bytes).unwrap();
    fs::create_dir(&temporary_path).unwrap();

    let error = publish_state(&path, &new).await.unwrap_err();

    assert_eq!(error.operation, "create temporary state");
    assert_eq!(error.path, temporary_path);
    assert_eq!(error.line, None);
    assert!(!error.source_message.is_empty());
    assert_eq!(fs::read(path).unwrap(), old_bytes);
}

#[cfg(unix)]
#[tokio::test]
async fn read_only_parent_rename_failure_retains_the_prior_destination() {
    use std::os::unix::fs::PermissionsExt;

    let directory = tempdir().unwrap();
    let path = directory.path().join("state.json");
    let temporary_path = directory.path().join("state.json.tmp");
    let old = valid_state();
    let mut new = old.clone();
    new.last_view = View::Guild;
    let old_bytes = serde_json::to_vec_pretty(&old).unwrap();
    fs::write(&path, &old_bytes).unwrap();
    fs::write(&temporary_path, b"stale temporary bytes").unwrap();
    let original_permissions = fs::metadata(directory.path()).unwrap().permissions();
    let mut read_only = original_permissions.clone();
    read_only.set_mode(0o555);
    fs::set_permissions(directory.path(), read_only).unwrap();

    let result = publish_state(&path, &new).await;

    fs::set_permissions(directory.path(), original_permissions).unwrap();
    let error = result.unwrap_err();
    assert_eq!(error.operation, "rename state");
    assert_eq!(error.path, path);
    assert_eq!(error.line, None);
    assert!(!error.source_message.is_empty());
    assert_eq!(fs::read(path).unwrap(), old_bytes);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn arbitrary_bytes_never_parse_as_an_unvalidated_state(bytes in prop::collection::vec(any::<u8>(), 0..=512)) {
        if let Ok(state) = parse_state("state.json".as_ref(), &bytes) {
            prop_assert!(state.validate().is_ok());
        }
    }

    #[test]
    fn atomic_publication_never_exposes_partial_json(
        first in support::persisted_state(),
        second in support::persisted_state(),
    ) {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(support::assert_atomic_publication(first, second));
    }
}
