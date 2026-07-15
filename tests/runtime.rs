use std::fs;

use questmancer::{app::View, persistence::effective_view, runtime::RuntimeRegistration};
use serde_json::Value;
use tempfile::tempdir;

#[test]
fn registration_writes_the_resolved_effective_view_and_cleans_it_on_drop() {
    let directory = tempdir().expect("temporary state directory");
    let runtime_path = directory.path().join("runtime.json");

    let resolved = effective_view(None, Some(View::Delve), View::Guild);
    let registration =
        RuntimeRegistration::register(directory.path(), "w1:p2", resolved).expect("register");
    let state: Value = serde_json::from_slice(&fs::read(&runtime_path).expect("runtime state"))
        .expect("valid runtime JSON");

    assert_eq!(state["pane_id"], "w1:p2");
    assert_eq!(state["initial_view"], "delve");
    assert!(state["pid"].as_u64().is_some_and(|pid| pid > 0));

    drop(registration);
    assert!(!runtime_path.exists());
}

#[test]
fn registration_does_not_delete_newer_pane_state() {
    let directory = tempdir().expect("temporary state directory");
    let runtime_path = directory.path().join("runtime.json");
    let registration =
        RuntimeRegistration::register(directory.path(), "old-pane", View::Guild).expect("register");

    fs::write(&runtime_path, r#"{"pane_id":"new-pane"}"#).expect("replace runtime state");
    drop(registration);

    assert!(runtime_path.exists());
}
