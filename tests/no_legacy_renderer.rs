use std::{fs, path::PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn tracked_renderer_surface() -> String {
    ["Cargo.toml", "justfile", "src/terminal.rs", "src/ui/mod.rs"]
        .into_iter()
        .map(|path| fs::read_to_string(root().join(path)).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn repository_has_one_production_renderer() {
    let source = tracked_renderer_surface();
    for forbidden in [
        "RenderExperience",
        "run_scene_preview",
        "render_with_projection",
        "questmancer-scene-preview",
        "scene-preview = []",
    ] {
        assert!(
            !source.contains(forbidden),
            "legacy path remains: {forbidden}"
        );
    }
}
