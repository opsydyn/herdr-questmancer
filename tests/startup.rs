use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use proptest::prelude::*;
use questmancer::{
    app::{CharacterSet, ColorMode, DisplayPreferences, Motion, View},
    config::PersistencePaths,
    domain::{AdventurerPersona, ChronicleEntry, ChronicleEvent, PersonaKey, Timestamp},
    persistence::{PersistedStateV1, StartupData, effective_view, load_startup},
};
use tempfile::TempDir;

fn paths(config_dir: Option<&Path>, state_dir: Option<&Path>) -> PersistencePaths {
    PersistencePaths::from_lookup(|name| match name {
        "HERDR_PLUGIN_CONFIG_DIR" => config_dir.map(|path| path.display().to_string()),
        "HERDR_PLUGIN_STATE_DIR" => state_dir.map(|path| path.display().to_string()),
        _ => None,
    })
}

fn state(last_view: View, preferences: DisplayPreferences) -> PersistedStateV1 {
    PersistedStateV1 {
        schema_version: 1,
        last_view,
        preferences,
        selected_persona: None,
        personas: BTreeMap::new(),
        seen_attention: BTreeSet::new(),
    }
}

fn chronicle_entry(index: i64) -> ChronicleEntry {
    ChronicleEntry::new(
        Timestamp::from_millis(index),
        None,
        None,
        None,
        u64::try_from(index).unwrap(),
        ChronicleEvent::SpoilsReturned,
        format!("entry {index}"),
    )
}

async fn write_chronicle(directory: &TempDir, entries: impl IntoIterator<Item = ChronicleEntry>) {
    let mut bytes = Vec::new();
    for entry in entries {
        bytes.extend(serde_json::to_vec(&entry).unwrap());
        bytes.push(b'\n');
    }
    tokio::fs::write(directory.path().join("chronicle.jsonl"), bytes)
        .await
        .unwrap();
}

#[test]
fn view_precedence_is_explicit_then_persisted_then_configured_then_desk() {
    assert_eq!(
        effective_view(Some(View::Guild), Some(View::Delve), View::Delve),
        View::Guild
    );
    assert_eq!(
        effective_view(None, Some(View::Delve), View::Guild),
        View::Delve
    );
    assert_eq!(effective_view(None, None, View::Delve), View::Delve);
    assert_eq!(effective_view(None, None, View::Guild), View::Guild);

    for explicit in [None, Some(View::Guild), Some(View::Delve)] {
        for persisted in [None, Some(View::Guild), Some(View::Delve)] {
            for configured in [View::Guild, View::Delve] {
                let expected = explicit.or(persisted).unwrap_or(configured);
                assert_eq!(
                    effective_view(explicit, persisted, configured),
                    expected,
                    "explicit={explicit:?}, persisted={persisted:?}, configured={configured:?}"
                );
            }
        }
    }
}

#[tokio::test]
async fn absent_files_use_defaults_without_diagnostics() {
    let config = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();

    let startup = load_startup(paths(Some(config.path()), Some(state.path())), None).await;

    assert_eq!(startup.model.view(), View::Guild);
    assert_eq!(startup.model.preferences(), &DisplayPreferences::default());
    assert_eq!(startup.model.settings().output_preview_lines, 80);
    assert_eq!(
        startup.model.settings().reviewr_action,
        "persiyanov.reviewr.open"
    );
    assert!(startup.model.settings().show_elapsed_time);
    assert!(startup.model.domain().chronicle.entries().is_empty());
    assert_eq!(startup.paths.state, Some(state.path().join("state.json")));
    assert_eq!(
        startup.paths.chronicle,
        Some(state.path().join("chronicle.jsonl"))
    );
    assert!(startup.diagnostics.is_empty());
}

#[tokio::test]
async fn valid_files_restore_state_preferences_history_and_config_only_settings() {
    let config = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    tokio::fs::write(
        config.path().join("config.toml"),
        br#"
default_view = "guild"
motion = "full"
character_set = "unicode"
color_mode = "xterm256"
output_preview_lines = 123
chronicle_max_entries = 50
reviewr_action = "acme.diff.inspect"
show_elapsed_time = false
"#,
    )
    .await
    .unwrap();
    let persisted_preferences = DisplayPreferences {
        motion: Motion::None,
        character_set: CharacterSet::Ascii,
        color_mode: ColorMode::Ansi16,
    };
    let mut persisted_state = state(View::Delve, persisted_preferences);
    let persona_key = PersonaKey::new("persona-restored");
    let mut restored_persona = AdventurerPersona::for_key(persona_key.clone());
    restored_persona.name = "Restored Name".to_owned();
    persisted_state
        .personas
        .insert(persona_key.clone(), restored_persona);
    persisted_state.selected_persona = Some(persona_key.clone());
    tokio::fs::write(
        state_dir.path().join("state.json"),
        serde_json::to_vec(&persisted_state).unwrap(),
    )
    .await
    .unwrap();
    write_chronicle(&state_dir, (0..55).map(chronicle_entry)).await;

    let startup = load_startup(paths(Some(config.path()), Some(state_dir.path())), None).await;

    assert_eq!(startup.model.view(), View::Delve);
    assert_eq!(startup.model.preferences(), &persisted_preferences);
    let captured = PersistedStateV1::capture(&startup.model);
    assert_eq!(captured.personas[&persona_key].name, "Restored Name");
    assert_eq!(startup.model.settings().output_preview_lines, 123);
    assert_eq!(startup.model.settings().reviewr_action, "acme.diff.inspect");
    assert!(!startup.model.settings().show_elapsed_time);
    let entries = startup.model.domain().chronicle.entries();
    assert_eq!(entries.len(), 50);
    assert_eq!(entries.front().unwrap().summary, "entry 5");
    assert_eq!(entries.back().unwrap().summary, "entry 54");
    let state_json = serde_json::to_string(&captured).unwrap();
    assert!(!state_json.contains("output_preview_lines"));
    assert!(!state_json.contains("reviewr_action"));
    assert!(!state_json.contains("show_elapsed_time"));
    assert_eq!(
        startup.paths.state,
        Some(state_dir.path().join("state.json"))
    );
    assert!(startup.diagnostics.is_empty());
}

#[tokio::test]
async fn invalid_config_uses_safe_runtime_defaults_but_keeps_valid_state() {
    let config = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    tokio::fs::write(config.path().join("config.toml"), b"default_view = [")
        .await
        .unwrap();
    let persisted_preferences = DisplayPreferences {
        motion: Motion::Reduced,
        character_set: CharacterSet::Ascii,
        color_mode: ColorMode::Ansi16,
    };
    tokio::fs::write(
        state_dir.path().join("state.json"),
        serde_json::to_vec(&state(View::Delve, persisted_preferences)).unwrap(),
    )
    .await
    .unwrap();

    let startup = load_startup(paths(Some(config.path()), Some(state_dir.path())), None).await;

    assert_eq!(startup.model.view(), View::Delve);
    assert_eq!(startup.model.preferences(), &persisted_preferences);
    assert_eq!(startup.model.settings().output_preview_lines, 80);
    assert_eq!(startup.diagnostics.len(), 1);
    assert_eq!(startup.diagnostics[0].operation, "parse config");
    assert_eq!(
        startup.diagnostics[0].path,
        config.path().join("config.toml")
    );
}

#[tokio::test]
async fn future_state_is_ignored_without_hiding_valid_chronicle_history() {
    let state_dir = tempfile::tempdir().unwrap();
    let mut future = state(View::Delve, DisplayPreferences::default());
    future.schema_version = 2;
    tokio::fs::write(
        state_dir.path().join("state.json"),
        serde_json::to_vec(&future).unwrap(),
    )
    .await
    .unwrap();
    write_chronicle(&state_dir, [chronicle_entry(1)]).await;

    let startup = load_startup(paths(None, Some(state_dir.path())), None).await;

    assert_eq!(startup.model.view(), View::Guild);
    assert_eq!(startup.model.domain().chronicle.entries().len(), 1);
    assert_eq!(startup.diagnostics.len(), 1);
    assert_eq!(startup.diagnostics[0].operation, "validate state");
    assert_eq!(startup.paths.state, None);
    assert_eq!(
        startup.paths.chronicle,
        Some(state_dir.path().join("chronicle.jsonl"))
    );
}

#[tokio::test]
async fn unreadable_state_disables_only_state_publication() {
    let state_dir = tempfile::tempdir().unwrap();
    tokio::fs::create_dir(state_dir.path().join("state.json"))
        .await
        .unwrap();

    let startup = load_startup(paths(None, Some(state_dir.path())), None).await;

    assert_eq!(startup.paths.state, None);
    assert_eq!(
        startup.paths.chronicle,
        Some(state_dir.path().join("chronicle.jsonl"))
    );
    assert_eq!(startup.diagnostics.len(), 1);
    assert_eq!(startup.diagnostics[0].operation, "read state");
}

#[tokio::test]
async fn malformed_chronicle_records_report_diagnostics_and_preserve_valid_records() {
    let state_dir = tempfile::tempdir().unwrap();
    let mut bytes = serde_json::to_vec(&chronicle_entry(1)).unwrap();
    bytes.extend_from_slice(b"\n{not json}\n");
    tokio::fs::write(state_dir.path().join("chronicle.jsonl"), bytes)
        .await
        .unwrap();

    let startup = load_startup(paths(None, Some(state_dir.path())), None).await;

    assert_eq!(startup.model.domain().chronicle.entries().len(), 1);
    assert_eq!(startup.diagnostics.len(), 1);
    assert_eq!(startup.diagnostics[0].operation, "parse chronicle record");
    assert_eq!(startup.diagnostics[0].line, Some(2));
}

#[tokio::test]
async fn missing_config_directory_does_not_block_valid_state() {
    let root = tempfile::tempdir().unwrap();
    let missing_config = root.path().join("missing-config");
    let state_dir = tempfile::tempdir().unwrap();
    tokio::fs::write(
        state_dir.path().join("state.json"),
        serde_json::to_vec(&state(View::Delve, DisplayPreferences::default())).unwrap(),
    )
    .await
    .unwrap();

    let startup = load_startup(paths(Some(&missing_config), Some(state_dir.path())), None).await;

    assert_eq!(startup.model.view(), View::Delve);
    assert!(startup.diagnostics.is_empty());
}

#[tokio::test]
async fn missing_state_directory_does_not_block_valid_config() {
    let root = tempfile::tempdir().unwrap();
    let missing_state = root.path().join("missing-state");
    let config = tempfile::tempdir().unwrap();
    tokio::fs::write(config.path().join("config.toml"), b"default_view = 'delve'")
        .await
        .unwrap();

    let startup = load_startup(paths(Some(config.path()), Some(&missing_state)), None).await;

    assert_eq!(startup.model.view(), View::Delve);
    assert!(startup.diagnostics.is_empty());
}

#[tokio::test]
async fn plugin_disabled_startup_is_in_memory_only() {
    let startup = load_startup(PersistencePaths::default(), Some(View::Delve)).await;

    assert_eq!(startup.model.view(), View::Delve);
    assert_eq!(startup.paths.state, None);
    assert_eq!(startup.paths.chronicle, None);
    assert!(startup.diagnostics.is_empty());
}

#[tokio::test]
async fn startup_data_has_no_settings_owner_outside_the_model() {
    let startup = load_startup(PersistencePaths::default(), None).await;

    let StartupData {
        model,
        paths,
        diagnostics,
    } = startup;

    assert_eq!(model.settings().output_preview_lines, 80);
    assert_eq!(paths.state, None);
    assert!(diagnostics.is_empty());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn arbitrary_startup_files_never_panic(
        config_bytes in prop::collection::vec(any::<u8>(), 0..=512),
        state_bytes in prop::collection::vec(any::<u8>(), 0..=512),
        chronicle_bytes in prop::collection::vec(any::<u8>(), 0..=512),
    ) {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            let config = tempfile::tempdir().unwrap();
            let state = tempfile::tempdir().unwrap();
            tokio::fs::write(config.path().join("config.toml"), config_bytes)
                .await
                .unwrap();
            tokio::fs::write(state.path().join("state.json"), state_bytes)
                .await
                .unwrap();
            tokio::fs::write(state.path().join("chronicle.jsonl"), chronicle_bytes)
                .await
                .unwrap();

            let startup = load_startup(paths(Some(config.path()), Some(state.path())), None).await;

            prop_assert!(matches!(startup.model.view(), View::Guild | View::Delve));
            prop_assert!((10..=500).contains(&startup.model.settings().output_preview_lines));
            prop_assert!(!startup.model.settings().reviewr_action.trim().is_empty());
            let selection_is_live = startup.model.selected_agent_key().is_none_or(|key| {
                startup.model.domain().agents.contains_key(key)
            });
            prop_assert!(selection_is_live);
            Ok(())
        })?;
    }
}
