use std::path::Path;

use proptest::prelude::*;
use questmancer::{
    app::{CharacterSet, ColorMode, Motion, View},
    config::{PersistencePaths, QuestmancerConfig},
};

#[test]
fn parses_a_complete_configuration() {
    let config = QuestmancerConfig::parse(
        br#"
            default_view = "delve"
            motion = "reduced"
            character_set = "ascii"
            color_mode = "ansi16"
            output_preview_lines = 120
            chronicle_max_entries = 750
            reviewr_action = "persiyanov.reviewr.open"
            show_elapsed_time = false
            future_field = "accepted"
        "#,
    )
    .unwrap();

    assert_eq!(config.default_view, View::Delve);
    assert_eq!(config.preferences.motion, Motion::Reduced);
    assert_eq!(config.preferences.character_set, CharacterSet::Ascii);
    assert_eq!(config.preferences.color_mode, ColorMode::Ansi16);
    assert_eq!(config.output_preview_lines, 120);
    assert_eq!(config.chronicle_max_entries, 750);
    assert_eq!(config.reviewr_action, "persiyanov.reviewr.open");
    assert!(!config.show_elapsed_time);
}

#[test]
fn empty_configuration_uses_complete_defaults() {
    let config = QuestmancerConfig::parse(b"").unwrap();

    assert_eq!(config.default_view, View::Guild);
    assert_eq!(config.preferences.motion, Motion::Full);
    assert_eq!(config.preferences.character_set, CharacterSet::Unicode);
    assert_eq!(config.preferences.color_mode, ColorMode::Xterm256);
    assert_eq!(config.output_preview_lines, 80);
    assert_eq!(config.chronicle_max_entries, 500);
    assert_eq!(config.reviewr_action, "persiyanov.reviewr.open");
    assert!(config.show_elapsed_time);
}

#[test]
fn accepts_every_view_value() {
    for (value, expected) in [("guild", View::Guild), ("delve", View::Delve)] {
        let config = QuestmancerConfig::parse(format!("default_view = '{value}'").as_bytes())
            .expect("accepted view");
        assert_eq!(config.default_view, expected);
    }
}

#[test]
fn accepts_every_motion_value() {
    for (value, expected) in [
        ("full", Motion::Full),
        ("reduced", Motion::Reduced),
        ("none", Motion::None),
    ] {
        let config = QuestmancerConfig::parse(format!("motion = '{value}'").as_bytes())
            .expect("accepted motion");
        assert_eq!(config.preferences.motion, expected);
    }
}

#[test]
fn accepts_every_character_set_value() {
    for (value, expected) in [
        ("unicode", CharacterSet::Unicode),
        ("ascii", CharacterSet::Ascii),
    ] {
        let config = QuestmancerConfig::parse(format!("character_set = '{value}'").as_bytes())
            .expect("accepted character set");
        assert_eq!(config.preferences.character_set, expected);
    }
}

#[test]
fn accepts_every_color_mode_value() {
    for (value, expected) in [
        ("xterm256", ColorMode::Xterm256),
        ("ansi16", ColorMode::Ansi16),
    ] {
        let config = QuestmancerConfig::parse(format!("color_mode = '{value}'").as_bytes())
            .expect("accepted color mode");
        assert_eq!(config.preferences.color_mode, expected);
    }
}

#[test]
fn accepts_output_preview_line_bounds() {
    for value in [10, 500] {
        let config = QuestmancerConfig::parse(format!("output_preview_lines = {value}").as_bytes())
            .expect("value at the bound");
        assert_eq!(config.output_preview_lines, value);
    }
}

#[test]
fn rejects_output_preview_lines_outside_bounds() {
    for value in [9, 501] {
        let error = QuestmancerConfig::parse(format!("output_preview_lines = {value}").as_bytes())
            .unwrap_err();
        assert!(error.to_string().contains("output_preview_lines"));
    }
}

#[test]
fn accepts_chronicle_entry_bounds() {
    for value in [50, 10_000] {
        let config =
            QuestmancerConfig::parse(format!("chronicle_max_entries = {value}").as_bytes())
                .expect("value at the bound");
        assert_eq!(config.chronicle_max_entries, value);
    }
}

#[test]
fn rejects_chronicle_entries_outside_bounds() {
    for value in [49, 10_001] {
        let error = QuestmancerConfig::parse(format!("chronicle_max_entries = {value}").as_bytes())
            .unwrap_err();
        assert!(error.to_string().contains("chronicle_max_entries"));
    }
}

#[test]
fn empty_reviewr_action_rejects_the_whole_file() {
    let error = QuestmancerConfig::parse(
        br#"
            default_view = "delve"
            reviewr_action = "   "
        "#,
    )
    .unwrap_err();

    assert!(error.to_string().contains("reviewr_action"));
}

#[test]
fn malformed_toml_rejects_the_whole_file() {
    assert!(QuestmancerConfig::parse(b"default_view = [").is_err());
}

#[test]
fn discovers_each_plugin_directory_independently() {
    let paths = PersistencePaths::from_lookup(|name| match name {
        "HERDR_PLUGIN_CONFIG_DIR" => Some("/tmp/config".into()),
        "HERDR_PLUGIN_STATE_DIR" => Some("/tmp/state".into()),
        _ => None,
    });

    assert_eq!(
        paths.config_path().unwrap(),
        Path::new("/tmp/config/config.toml")
    );
    assert_eq!(
        paths.state_path().unwrap(),
        Path::new("/tmp/state/state.json")
    );
    assert_eq!(
        paths.chronicle_path().unwrap(),
        Path::new("/tmp/state/chronicle.jsonl")
    );
}

#[test]
fn missing_plugin_directories_disable_only_their_own_paths() {
    let config_only = PersistencePaths::from_lookup(|name| {
        (name == "HERDR_PLUGIN_CONFIG_DIR").then(|| "/tmp/config".into())
    });
    assert_eq!(
        config_only.config_path().unwrap(),
        Path::new("/tmp/config/config.toml")
    );
    assert_eq!(config_only.state_path(), None);
    assert_eq!(config_only.chronicle_path(), None);

    let state_only = PersistencePaths::from_lookup(|name| {
        (name == "HERDR_PLUGIN_STATE_DIR").then(|| "/tmp/state".into())
    });
    assert_eq!(state_only.config_path(), None);
    assert_eq!(
        state_only.state_path().unwrap(),
        Path::new("/tmp/state/state.json")
    );
    assert_eq!(
        state_only.chronicle_path().unwrap(),
        Path::new("/tmp/state/chronicle.jsonl")
    );
}

proptest! {
    #[test]
    fn arbitrary_config_bytes_never_panic(
        bytes in proptest::collection::vec(any::<u8>(), 0..4096),
    ) {
        let _ = QuestmancerConfig::parse(&bytes);
    }
}
