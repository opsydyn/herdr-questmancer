# Scene-First Production Cutover Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan inline, task-by-task. Do not create worktrees or dispatch subagents for this repository.

**Goal:** Make the approved RGB Guild Hall and Delve the sole production renderer while preserving selection, search, reply, pane focus, output inspection and explicit world switching.

**Architecture:** The normal interactive runtime will render `SceneSnapshot` truth plus a separate `ScenePresentation` projection into the existing RGB buffer, flush it once through the half-block adapter, then paint contextual Ratatui overlays. Existing input reduction and typed Herdr commands remain authoritative; after parity is proven, the legacy dashboard renderer and preview-only binary are deleted.

**Tech Stack:** Rust 1.90, Ratatui 0.30, Crossterm 0.29, Tokio, BLAKE3, `proptest` 1.11, Herdr 0.7.4.

## Global Constraints

- Work inline on `main`; do not create a worktree or dispatch subagents.
- Preserve all existing uncommitted work and stage only files named by the active task.
- The RGB renderer is the only production renderer; do not add a fallback or runtime switch.
- Preserve the existing typed `Action -> ActionReduction -> AgentCommand` flow.
- Herdr state owns agent presence and pose; UI interaction may select or command an agent but must not invent state.
- Storybook remains development-only and consumes production scene components.
- Keep terminal restoration correct for normal exit, signals and panic.
- Each task uses red-green-refactor and ends with a focused commit.

---

## File structure

- `src/scene/presentation.rs` — UI-only scene projection: selected agent, chosen world and contextual overlay facts.
- `src/scene/render/interaction.rs` — paints in-world selection and short-lived command-ribbon treatments into the RGB buffer.
- `src/ui/scene_overlays.rs` — paints counsel, search, help, output and error parchments after the RGB flush.
- `src/terminal.rs` — one interactive production loop and one scene-first draw boundary.
- `src/scene/{mod.rs,stage.rs,render/mod.rs}` — explicit-world scene rendering entry point shared by production and Storybook.
- `tests/scene_interaction.rs` — selected-agent and world-switch rendering contracts.
- `tests/scene_overlays.rs` — contextual overlay rendering contracts.
- `tests/scene_runtime.rs` — production binary, runtime and packaging cutover contracts.
- `tests/scripts.sh` — release surface rejects preview and legacy renderer paths.
- `src/ui/` legacy files listed in Task 5 — deleted only after production parity is green.

### Task 1: Add the scene presentation boundary and in-world selection

**Files:**
- Create: `src/scene/presentation.rs`
- Create: `src/scene/render/interaction.rs`
- Modify: `src/scene/mod.rs`
- Modify: `src/scene/render/mod.rs`
- Modify: `src/scene/stage.rs`
- Test: `tests/scene_interaction.rs`

**Interfaces:**
- Produces: `ScenePresentation::from_model(&Model) -> ScenePresentation`
- Produces: `render_scene_for_world(&SceneSnapshot, &ScenePresentation, PixelSize, &mut RgbBuffer) -> SceneFrame`
- Produces: `paint_interaction(&ScenePresentation, &ScenePlan, PixelSize, &mut RgbBuffer)`
- Consumes: existing `SceneSnapshot`, `ScenePlan`, `WorldScene`, `AgentKey`, `View` and actor placements.

- [ ] **Step 1: Write the failing presentation tests**

```rust
#[test]
fn presentation_keeps_ui_state_outside_scene_truth() {
    let mut model = connected_model(View::Guild);
    model.select_agent(AgentKey::new("codex"));

    let snapshot = SceneSnapshot::from_model(&model);
    let presentation = ScenePresentation::from_model(&model);

    assert_eq!(presentation.world, WorldScene::GuildHall);
    assert_eq!(presentation.selected_agent, Some(AgentKey::new("codex")));
    assert_eq!(snapshot, SceneSnapshot::from_model(&model));
}

#[test]
fn explicit_world_render_marks_only_the_selected_adventurer() {
    let (snapshot, presentation) = mixed_party_with_selected("codex");
    let mut target = RgbBuffer::filled(160, 90, Rgb::BLACK);

    render_scene_for_world(
        &snapshot,
        &presentation,
        PixelSize::new(160, 90),
        &mut target,
    );

    assert_eq!(count_selection_runes(&target), 1);
}
```

- [ ] **Step 2: Run the tests and verify the missing boundary fails**

Run: `cargo test --test scene_interaction --features storybook`

Expected: FAIL because `scene::presentation` and `render_scene_for_world` do not exist.

- [ ] **Step 3: Implement the minimal presentation types and renderer entry point**

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScenePresentation {
    pub world: WorldScene,
    pub selected_agent: Option<AgentKey>,
    pub overlay: SceneOverlay,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SceneOverlay {
    None,
    Counsel,
    Search,
    Help,
    Scrying,
}

impl ScenePresentation {
    pub fn from_model(model: &Model) -> Self {
        Self {
            world: match model.view() {
                View::Guild => WorldScene::GuildHall,
                View::Delve => WorldScene::Delve,
            },
            selected_agent: model.selected_agent_key().cloned(),
            overlay: SceneOverlay::from_model(model),
        }
    }
}
```

Add `render_scene_for_world` beside `render_scene`; project with `project_for_world`, paint the world, then call `render::interaction::paint_interaction`. Draw one deterministic rune/lamplight marker around the selected actor placement without changing the sprite or domain state.

- [ ] **Step 4: Run focused scene tests**

Run: `cargo test --test scene_interaction --test scene_guild_hall --test scene_delve --features storybook`

Expected: PASS; exactly one selected actor receives the marker in either world.

- [ ] **Step 5: Commit the presentation boundary**

```bash
git add src/scene/presentation.rs src/scene/render/interaction.rs src/scene/mod.rs src/scene/render/mod.rs src/scene/stage.rs tests/scene_interaction.rs
git commit -m "feat: project interaction into pixel worlds"
```

### Task 2: Render contextual controls over the RGB world

**Files:**
- Modify: `src/app.rs`
- Modify: `src/interaction.rs`
- Create: `src/ui/scene_overlays.rs`
- Modify: `src/ui/mod.rs`
- Modify: `src/ui/theme.rs`
- Test: `tests/scene_overlays.rs`
- Test: `tests/reply.rs`
- Test: `tests/interaction.rs`

**Interfaces:**
- Consumes: `Model`, `ScenePresentation`, Ratatui `Frame`, existing counsel/search text and output preview.
- Produces: `render_scene_overlays(frame: &mut Frame<'_>, model: &Model, presentation: &ScenePresentation)`.
- Produces: `Modal::Scrying` and `Model::command_ribbon_visible() -> bool`.

- [ ] **Step 1: Write failing overlay tests with `TestBackend`**

```rust
#[test]
fn counsel_uses_a_centered_parchment_without_replacing_the_world() {
    let mut model = model_with_selected_agent();
    reduce_action(&mut model, Action::Counsel);
    let buffer = render_over_world(&model, 120, 36);

    assert!(buffer_text(&buffer).contains("ISSUE COUNSEL"));
    assert!(buffer_text(&buffer).contains("Esc cancel"));
    assert!(world_pixel_survives_outside_modal(&buffer));
}

#[test]
fn search_and_scrying_are_contextual_overlays() {
    let mut model = model_with_selected_agent();
    reduce_action(&mut model, Action::Search);
    assert!(buffer_text(&render_over_world(&model, 120, 36)).contains("SEARCH THE GUILD"));

    model.set_output_preview(Some(output_preview("cargo test passed")));
    reduce_action(&mut model, Action::Refresh);
    assert!(buffer_text(&render_over_world(&model, 120, 36)).contains("SCRYING"));
}

#[test]
fn command_ribbon_expires_without_persisting_state() {
    let mut model = model_with_selected_agent();
    model.set_now(Timestamp::from_millis(1_000));
    reduce_action(&mut model, Action::Next);
    assert!(model.command_ribbon_visible());

    model.set_now(Timestamp::from_millis(4_001));
    assert!(!model.command_ribbon_visible());
}
```

- [ ] **Step 2: Run the overlay tests and verify red**

Run: `cargo test --test scene_overlays --test reply`

Expected: FAIL because `render_scene_overlays` does not exist.

- [ ] **Step 3: Implement the scrying modal, transient ribbon and overlay dispatcher**

```rust
pub fn render_scene_overlays(
    frame: &mut Frame<'_>,
    model: &Model,
    presentation: &ScenePresentation,
) {
    match presentation.overlay {
        SceneOverlay::Counsel | SceneOverlay::Search => render_input_parchment(frame, model),
        SceneOverlay::Help => render_help_parchment(frame),
        SceneOverlay::Scrying => render_scrying_parchment(frame, model),
        SceneOverlay::None => render_context_ribbon(frame, model),
    }
}
```

Add `Modal::Scrying`; `Action::Refresh` opens it and schedules the existing
single `LoadOutput` command, while Esc dismisses it. Add
`last_interaction_at: Option<Timestamp>` to `Model`; meaningful reducer actions
set it to `model.now()`, and `command_ribbon_visible` returns true for at most
three seconds. Do not persist either field. Use `Clear` only inside the overlay
rectangle. Keep the rest of the RGB world untouched. The ribbon contains only
valid actions and remains one line high. Reuse existing copy constants; do not
duplicate command semantics.

- [ ] **Step 4: Verify overlays at normal and minimum viewports**

Run: `cargo test --test scene_overlays --test reply --test input --test interaction`

Expected: PASS at `120x36` and `80x24`; zero-sized rectangles are panic-free.

- [ ] **Step 5: Commit contextual overlays**

```bash
git add src/app.rs src/interaction.rs src/ui/scene_overlays.rs src/ui/mod.rs src/ui/theme.rs tests/scene_overlays.rs tests/reply.rs tests/interaction.rs
git commit -m "feat: add contextual pixel-world controls"
```

### Task 3: Move the normal interactive runtime onto the RGB renderer

**Files:**
- Modify: `src/terminal.rs`
- Modify: `src/main.rs`
- Modify: `src/scene/mod.rs`
- Create: `tests/scene_runtime.rs`
- Modify: `tests/runtime_loop.rs`

**Interfaces:**
- Consumes: `SceneSnapshot::from_model`, `ScenePresentation::from_model`, `render_scene_for_world`, `flush_rgb`, `render_scene_overlays`.
- Produces: `draw_scene_application(&mut Tui, &Model, &mut RgbBuffer) -> Result<SceneFrame>`.

- [ ] **Step 1: Write failing production-entry and interactive-runtime tests**

```rust
#[test]
fn normal_binary_uses_the_scene_first_runtime() {
    let main = fs::read_to_string(root().join("src/main.rs")).unwrap();
    assert!(main.contains("terminal::run(view).await"));
    assert!(!main.contains("run_scene_preview"));

    let terminal = fs::read_to_string(root().join("src/terminal.rs")).unwrap();
    assert!(terminal.contains("draw_scene_application"));
    assert!(!terminal.contains("RenderExperience::Legacy"));
}

#[test]
fn production_scene_runtime_keeps_typed_input_reduction() {
    for (key, action) in retained_scene_actions() {
        assert_eq!(action_for(key), action);
    }
}
```

- [ ] **Step 2: Run and verify the cutover tests fail**

Run: `cargo test --test scene_runtime --test input --test interaction`

Expected: FAIL because the normal loop still calls `ui::render_with_projection` and retains `RenderExperience::Legacy`.

- [ ] **Step 3: Implement the production draw boundary**

```rust
fn draw_scene_application(
    terminal: &mut Tui,
    model: &Model,
    buffer: &mut RgbBuffer,
) -> Result<SceneFrame> {
    let mut rendered = None;
    terminal.draw(|frame| {
        let area = frame.area();
        let snapshot = SceneSnapshot::from_model(model);
        let presentation = ScenePresentation::from_model(model);
        let scene_frame = render_scene_for_world(
            &snapshot,
            &presentation,
            PixelSize::new(area.width, area.height.saturating_mul(2)),
            buffer,
        );
        flush_rgb(frame.buffer_mut(), area, buffer, Rgb::BLACK);
        ui::scene_overlays::render_scene_overlays(frame, model, &presentation);
        rendered = Some(scene_frame);
    })?;
    rendered.context("scene application draw did not produce a frame")
}
```

Use this draw in live and offline loops. Keep the existing input branch unchanged: it must continue calling `action_for_event_in`, `reduce_action`, `dispatch_action_effects` and `connection.schedule`. Reset `AnimationScheduler` from `SceneFrame::next_frame_in`.

- [ ] **Step 4: Verify command and persistence parity**

Run: `cargo test --test scene_runtime --test runtime_loop --test interaction --test input --test reply`

Expected: PASS; selection schedules one lazy output load, Enter focuses, reply sends once, and search does not mutate agent truth.

- [ ] **Step 5: Commit the production runtime cutover**

```bash
git add src/terminal.rs src/main.rs src/scene/mod.rs tests/scene_runtime.rs tests/runtime_loop.rs
git commit -m "feat: run questmancer as a pixel world"
```

### Task 4: Make Storybook review the production interaction surface

**Files:**
- Modify: `src/storybook/fixtures.rs`
- Modify: `src/storybook/catalogue.rs`
- Modify: `src/storybook/ui.rs`
- Test: `tests/storybook.rs`
- Test: `tests/scene_overlays.rs`

**Interfaces:**
- Consumes: production `ScenePresentation`, `render_scene_for_world` and `render_scene_overlays`.
- Produces: fixed stories for selected adventurer, counsel, search, scrying, help and narrow overlays.

- [ ] **Step 1: Add failing Storybook coverage tests**

```rust
#[test]
fn catalogue_contains_every_production_scene_overlay_once() {
    let titles = story_titles();
    for title in [
        "Interaction / Selected Adventurer",
        "Interaction / Counsel Parchment",
        "Interaction / Search Parchment",
        "Interaction / Scrying Parchment",
        "Interaction / Help Parchment",
        "Interaction / Narrow Parchment",
    ] {
        assert_eq!(titles.iter().filter(|candidate| **candidate == title).count(), 1);
    }
}
```

- [ ] **Step 2: Run and verify missing stories fail**

Run: `cargo test --test storybook --test scene_overlays --features storybook`

Expected: FAIL listing the missing interaction stories.

- [ ] **Step 3: Add fixed fixtures using the production renderer**

Each fixture builds a normal `Model`, applies the real reducer action, derives `SceneSnapshot` and `ScenePresentation`, and renders via the production entry points. Do not reproduce overlay or selection logic inside Storybook.

- [ ] **Step 4: Run Storybook verification**

Run: `cargo test --all-targets --features storybook`

Expected: PASS; every authored interaction surface appears exactly once.

- [ ] **Step 5: Commit Storybook parity**

```bash
git add src/storybook/fixtures.rs src/storybook/catalogue.rs src/storybook/ui.rs tests/storybook.rs tests/scene_overlays.rs
git commit -m "test: review pixel-world interactions in storybook"
```

### Task 5: Delete the legacy renderer and preview-only path

**Files:**
- Delete: `src/bin/questmancer_scene_preview.rs`
- Delete: `tests/scene_live_preview.rs`
- Delete: `src/ui/delve_projection.rs`
- Delete: `src/ui/delve_scene.rs`
- Delete: `src/ui/goblins.rs`
- Delete: `src/ui/guild_room_projection.rs`
- Delete: `src/ui/persona/`
- Delete: `src/ui/pixel/`
- Delete: `src/ui/theatre.rs`
- Delete: `src/ui/views/delve.rs`
- Delete: `src/ui/views/great_room.rs`
- Delete: `src/ui/views/guild_hall.rs`
- Delete: `src/ui/views/help.rs`
- Delete: `src/ui/views/reply.rs`
- Delete: `src/ui/widgets/`
- Delete: `tests/delve_rendering.rs`
- Delete: `tests/delve_scene.rs`
- Delete: `tests/delve_widgets.rs`
- Delete: `tests/goblins.rs`
- Delete: `tests/guild_hall_rendering.rs`
- Delete: `tests/guild_room_projection.rs`
- Delete: `tests/guild_room_properties.rs`
- Delete: `tests/persona_art.rs`
- Delete: `tests/pixel.rs`
- Delete: `tests/render_projection.rs`
- Delete: `tests/rendering.rs`
- Delete: `tests/theatre.rs`
- Modify: `tests/property_domain.rs`
- Modify: `tests/runtime_loop.rs`
- Modify: `tests/storybook_catalogue.rs`
- Modify: `tests/storybook_fixtures.rs`
- Modify: `src/ui/mod.rs`
- Modify: `Cargo.toml`
- Modify: `justfile`
- Modify: `tests/scripts.sh`
- Create: `tests/no_legacy_renderer.rs`

**Interfaces:**
- Keeps: `ui::input`, `ui::scene_adapter`, `ui::scene_overlays`, shared copy and theme values still referenced by production.
- Removes: `RenderExperience`, `run_scene_preview`, `render_with_projection`, `RenderProjection` and all dashboard-only projection types.

- [ ] **Step 1: Write the failing absence test before deleting code**

```rust
#[test]
fn repository_has_one_production_renderer() {
    let forbidden = [
        "RenderExperience",
        "run_scene_preview",
        "render_with_projection",
        "questmancer-scene-preview",
        "scene-preview = []",
    ];
    for needle in forbidden {
        assert!(!tracked_source().contains(needle), "legacy path remains: {needle}");
    }
}
```

- [ ] **Step 2: Run and verify the absence test fails on current legacy symbols**

Run: `cargo test --test no_legacy_renderer`

Expected: FAIL and report at least `RenderExperience` and `questmancer-scene-preview`.

- [ ] **Step 3: Use references to determine exact safe deletions**

Run:

```bash
rg -n "render_with_projection|RenderProjection|GuildRoomRenderPath|ChamberPresentation|DelveVariant|run_scene_preview|questmancer-scene-preview" src tests Cargo.toml justfile herdr tests/scripts.sh
```

The files listed in this task are the removal boundary. Before deleting them,
move any copy constant still referenced by `src/ui/scene_overlays.rs` into that
module and any colour still referenced by `src/scene/` into
`src/scene/assets/palette.rs`. Remove legacy-only property cases from
`tests/property_domain.rs` and projection scheduler cases from
`tests/runtime_loop.rs`; retain their domain and runtime command tests.

- [ ] **Step 4: Remove preview feature, binary, legacy modules and obsolete tests**

Keep `storybook = []` as the only renderer-related Cargo feature. Remove `scene-preview = []`, the preview `[[bin]]`, `just scene-preview`, and `just scene-preview-test`. Update module declarations so only production scene interaction modules compile.

- [ ] **Step 5: Verify the repository has one renderer**

Run: `cargo test --test no_legacy_renderer && cargo check --all-targets --all-features`

Expected: PASS with no unresolved legacy imports.

- [ ] **Step 6: Commit legacy deletion separately**

```bash
git add Cargo.toml Cargo.lock justfile src/ui/mod.rs tests/no_legacy_renderer.rs tests/property_domain.rs tests/runtime_loop.rs tests/storybook_catalogue.rs tests/storybook_fixtures.rs tests/scripts.sh
git add -u src/bin/questmancer_scene_preview.rs src/ui/persona src/ui/pixel src/ui/views src/ui/widgets src/ui/delve_projection.rs src/ui/delve_scene.rs src/ui/goblins.rs src/ui/guild_room_projection.rs src/ui/theatre.rs
git add -u tests/delve_rendering.rs tests/delve_scene.rs tests/delve_widgets.rs tests/goblins.rs tests/guild_hall_rendering.rs tests/guild_room_projection.rs tests/guild_room_properties.rs tests/persona_art.rs tests/pixel.rs tests/render_projection.rs tests/rendering.rs tests/scene_live_preview.rs tests/theatre.rs
git commit -m "refactor: remove the legacy questmancer renderer"
```

### Task 6: Update plugin, documentation and release contracts

**Files:**
- Modify: `README.md`
- Modify: `PLAN.md`
- Modify: `docs/manual-test/questmancer-scene-preview.md`
- Modify: `docs/superpowers/reviews/2026-07-17-scene-first-cutover.md`
- Modify: `herdr-plugin.toml`
- Modify: `herdr/run.sh`
- Modify: `tests/scripts.sh`
- Test: `tests/cli.rs`

**Interfaces:**
- Consumes: final production command and retained actions.
- Produces: one user installation/test path and an accepted cutover ledger.

- [ ] **Step 1: Write failing release-surface assertions**

Extend `tests/scripts.sh` to require:

```bash
assert_contains herdr/run.sh 'exec "$binary" "$@"'
assert_not_contains herdr-plugin.toml 'scene-preview'
assert_not_contains herdr/run.sh 'questmancer-scene-preview'
assert_not_contains README.md 'production Questmancer pane still uses the existing UI renderer'
```

Keep all five existing plugin actions unless product review explicitly removes them: `open`, `close`, `toggle`, `guild`, `delve`.

- [ ] **Step 2: Run the script test and verify stale documentation fails**

Run: `bash tests/scripts.sh`

Expected: FAIL on the README's preview-only language.

- [ ] **Step 3: Rewrite user documentation around the production pixel world**

Document installation, `cargo build --release`, linked-plugin behaviour, open/close/toggle, `1`/`2`, selection, observe, counsel, search, scrying and Storybook. Rename the manual guide to production acceptance language or rewrite it in place with no preview-only claims.

Set the ledger header to `Status: APPROVED FOR PRODUCTION CUTOVER` and record the 2026-07-19 Guild Hall and Delve captures as visual evidence. Leave untested state transitions explicitly `NOT REVIEWED`; renderer approval does not fabricate transition evidence.

- [ ] **Step 4: Run documentation and plugin contract checks**

Run: `bash tests/scripts.sh && cargo test --test cli`

Expected: PASS; plugin still exposes the retained actions and launches the normal binary.

- [ ] **Step 5: Commit product and release documentation**

```bash
git add README.md PLAN.md docs/manual-test/questmancer-scene-preview.md docs/superpowers/reviews/2026-07-17-scene-first-cutover.md herdr-plugin.toml herdr/run.sh tests/scripts.sh tests/cli.rs
git commit -m "docs: ship the questmancer pixel world"
```

### Task 7: Full verification and guarded live acceptance

**Files:**
- Modify only if verification exposes a cutover defect.

**Interfaces:**
- Consumes: completed production renderer, overlays, runtime, Storybook and plugin packaging.
- Produces: release-grade automated evidence and a guarded live acceptance report.

- [ ] **Step 1: Run formatting and static analysis**

Run:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
git diff --check
```

Expected: all commands exit 0 with no warnings.

- [ ] **Step 2: Run complete automated verification**

Run:

```bash
cargo test --all-targets --all-features
PROPTEST_CASES=4096 cargo test --test scene_pixel_properties --test scene_stage_properties
bash tests/scripts.sh
bash -n herdr/install.sh herdr/run.sh herdr/control.sh
cargo build --release
```

Expected: all commands exit 0.

- [ ] **Step 3: Reopen the linked plugin without relinking**

Record the pre-test pane/tab baseline, close only a positively identified test-owned Questmancer pane, then invoke:

```bash
herdr plugin action invoke opsydyn.questmancer.open
herdr plugin action invoke opsydyn.questmancer.open
```

Expected: exactly one Questmancer pane using `target/release/questmancer`; no preview binary and no legacy dashboard text.

- [ ] **Step 4: Exercise retained interactions manually**

Verify `1`, `2`, `j`/`k`, arrows, Enter, `r`, `/`, `v`, Esc and `q`. Send counsel only to an approved disposable agent; never operate on an arbitrary real agent. Mark unavailable transitions `BLOCKED`, not passed.

- [ ] **Step 5: Restore the original Herdr environment**

Close only test-created panes/tabs, return focus to the original tab, do not unlink the plugin, do not stop Herdr, and confirm the pane/tab set matches the baseline.

- [ ] **Step 6: Commit verification-only repairs if required**

If no repair was required, create no empty commit. If a defect was fixed through a new failing test, stage only that test and its fix and use:

```bash
git commit -m "fix: complete scene-first production acceptance"
```

## Completion criteria

- The linked production plugin opens the approved RGB Guild Hall and Delve.
- Selection, reply, search, focus, output inspection and `1`/`2` switching work on the pixel-world surface.
- The repository contains no preview-only or legacy renderer path.
- Storybook reviews production assets, worlds and overlays.
- Automated verification passes and manual evidence distinguishes passed, blocked and unreviewed checks.
- Existing unrelated working-tree changes remain preserved.
