use std::{fs, path::PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn normal_binary_uses_the_interactive_scene_first_runtime() {
    let main = fs::read_to_string(root().join("src/main.rs")).unwrap();
    assert!(main.contains("terminal::run(view).await"));
    assert!(!main.contains("run_scene_preview"));

    let terminal = fs::read_to_string(root().join("src/terminal.rs")).unwrap();
    assert!(terminal.contains("draw_scene_application"));
    assert!(!terminal.contains("RenderExperience::Legacy"));
    assert!(terminal.contains("ui::input::action_for_scene_event_in"));
    assert!(terminal.contains("reduce_scene_action"));
    assert!(terminal.contains("dispatch_action_effects"));
}

#[test]
fn production_draw_uses_explicit_world_and_contextual_overlays() {
    let terminal = fs::read_to_string(root().join("src/terminal.rs")).unwrap();
    for required in [
        "ScenePresentation::from_model",
        "render_scene_for_world",
        "flush_rgb",
        "render_scene_identity_labels",
        "render_scene_overlays",
    ] {
        assert!(
            terminal.contains(required),
            "production draw lacks {required}"
        );
    }
}
