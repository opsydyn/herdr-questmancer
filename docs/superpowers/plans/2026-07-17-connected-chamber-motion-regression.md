# Connected Chamber and Motion Regression Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restore complete persona and motion rendering in normal production connected Delves without weakening crop-honest chamber projection.

**Architecture:** Keep the eight-row chamber presentation contract unchanged and raise only the production authored-chamber maximum from six to eight rows. Existing partition clamping keeps constrained cells short and on the honest textual fallback; Storybook motion stories then expose their existing production animation frames at their declared reference viewport.

**Tech Stack:** Rust, Ratatui `TestBackend`/`Buffer`, Cargo integration tests, Proptest, shell workflow checks.

## Global Constraints

- Complete compact chambers use exactly one name row, six persona rows, and one state row.
- Departed and chambers shorter than eight rows report no persona art.
- The Delve renderer consumes production structural projection and does not recompute layout.
- Motion tests derive reference dimensions from each story's `Viewport`.
- Do not change story viewports, IDs, order, ownership totals, dependencies, or terminal lifecycle.

---

### Task 1: Capture connected chamber and story reference regressions

**Files:**
- Modify: `tests/delve_scene.rs`
- Modify: `tests/delve_rendering.rs`
- Modify: `tests/storybook_rendering.rs`

**Interfaces:**
- Consumes: `layout_delves(...) -> Vec<CampaignDelve>`, `ui::render(&mut Frame, &Model)`, `Story.viewport: Viewport`.
- Produces: regression tests that fail while authored chambers remain capped at six rows.

- [ ] **Step 1: Add a feature-off structural connected-layout test**

Add a two-agent campaign using the existing `support::fixture_domain()` agent template, lay it out in `Rect::new(0, 0, 120, 30)`, and assert that both chamber anchors have `height == 8` while remaining inside the Delve rectangle.

```rust
#[test]
fn connected_layout_allocates_complete_compact_chambers_when_room_allows() {
    let workspace = WorkspaceId::new("connected");
    let template = support::fixture_domain().agents.into_values().next().unwrap();
    let agents = ["a1", "a2"]
        .into_iter()
        .map(|id| {
            let mut agent = template.clone();
            agent.key = AgentKey::new(id);
            agent.workspace_id = workspace.clone();
            (agent.key.clone(), agent)
        })
        .collect::<BTreeMap<_, _>>();
    let campaign = Campaign {
        workspace_id: workspace.clone(),
        label: "Connected".to_owned(),
        cwd: "/tmp/connected".into(),
        party: agents.keys().cloned().collect(),
    };
    let delves = layout_delves(
        &BTreeMap::from([(workspace, campaign)]),
        &agents,
        Rect::new(0, 0, 120, 30),
        None,
    );

    assert_eq!(delves.len(), 1);
    assert_eq!(delves[0].chambers.len(), 2);
    assert!(delves[0].chambers.iter().all(|chamber| chamber.height == 8));
}
```

- [ ] **Step 2: Add a feature-off production buffer test**

Add a `render_buffer` helper beside the existing string renderer. Render the representative 120x30 connected model with one Working agent's `Footwear::Boots` versus `Footwear::Sabatons`, then `HairShape::Shaved` versus `HairShape::Quiff`; require full `Buffer` inequality for both comparisons.

```rust
fn render_buffer(model: &Model, width: u16, height: u16) -> ratatui::buffer::Buffer {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|frame| ui::render(frame, model)).unwrap();
    terminal.backend().buffer().clone()
}

#[test]
fn connected_persona_art_preserves_top_and_bottom_rows() {
    let mut baseline = three_agent_model();
    let key = AgentKey::new("agent-a");
    baseline.domain_mut().agents.get_mut(&key).unwrap().persona.appearance.footwear =
        Footwear::Boots;
    let mut different_footwear = baseline.clone();
    different_footwear.domain_mut().agents.get_mut(&key).unwrap().persona.appearance.footwear =
        Footwear::Sabatons;
    assert_ne!(
        render_buffer(&baseline, 120, 30),
        render_buffer(&different_footwear, 120, 30),
    );

    baseline.domain_mut().agents.get_mut(&key).unwrap().persona.appearance.hair = HairShape::Shaved;
    let mut different_hair = baseline.clone();
    different_hair.domain_mut().agents.get_mut(&key).unwrap().persona.appearance.hair =
        HairShape::Quiff;
    assert_ne!(
        render_buffer(&baseline, 120, 30),
        render_buffer(&different_hair, 120, 30),
    );
}
```

- [ ] **Step 3: Make the motion story test use declared reference viewports**

For each motion story, find its `Story`, build its application model, and render with `story.viewport.reference_width` and `story.viewport.reference_height`. Assert the three viewports are equal before asserting the three buffers are pairwise distinct.

```rust
let render = |id: &str| {
    let story = catalogue().iter().find(|story| story.id.as_str() == id).unwrap();
    let model = compatibility_model(id);
    let viewport = story.viewport;
    (
        viewport,
        storybook_ui::render_application_buffer(
            &model,
            viewport.reference_width,
            viewport.reference_height,
        ),
    )
};
let (full_viewport, full) = render("compat.motion-full");
let (reduced_viewport, reduced) = render("compat.motion-reduced");
let (none_viewport, none) = render("compat.motion-none");
assert_eq!(full_viewport, reduced_viewport);
assert_eq!(reduced_viewport, none_viewport);
assert_ne!(full, reduced);
assert_ne!(reduced, none);
assert_ne!(full, none);
```

- [ ] **Step 4: Run RED tests and record exact failures**

Run:

```bash
cargo test --test delve_scene connected_layout_allocates_complete_compact_chambers_when_room_allows -- --exact
cargo test --test delve_rendering connected_persona_art_preserves_top_and_bottom_rows -- --exact
cargo test --features storybook --test storybook_rendering motion_story_production_buffers_are_pairwise_distinct -- --exact
```

Expected: the structural assertion observes height six, persona-only buffer changes are invisible, and Full/Reduced/None buffers at 130x36 are equal.

### Task 2: Raise the production authored chamber cap

**Files:**
- Modify: `src/ui/delve_scene.rs:158`
- Test: `tests/delve_scene.rs`
- Test: `tests/delve_rendering.rs`
- Test: `tests/storybook_rendering.rs`

**Interfaces:**
- Consumes: `authored_chambers(variant, count, delve) -> Vec<ChamberAnchor>`.
- Produces: eight-row anchors whenever `height / rows >= 8`; constrained partitions continue returning their actual smaller height.

- [ ] **Step 1: Implement the minimal cap change**

```rust
let chamber_height = (height / rows).clamp(1, 8);
```

- [ ] **Step 2: Run focused GREEN tests**

Run the three exact RED commands, then:

```bash
cargo test --test delve_scene
cargo test --test delve_rendering
cargo test --test delve_widgets
cargo test --features storybook --test render_projection
cargo test --features storybook --test storybook_catalogue
cargo test --features storybook --test storybook_fixtures
cargo test --features storybook --test storybook_rendering
```

Expected: all pass, including existing 14x7 Text, 14x8 CompactScene, complete top/bottom persona rows, and Departed projection tests.

- [ ] **Step 3: Commit implementation and tests**

```bash
git add src/ui/delve_scene.rs tests/delve_scene.rs tests/delve_rendering.rs tests/storybook_rendering.rs
git commit -m "fix: restore connected Delve persona motion"
```

### Task 3: Verify and document the third wave

**Files:**
- Modify: `.superpowers/sdd/questmancer-storybook-final-fix-report.md`

**Interfaces:**
- Consumes: final implementation commit and verification output.
- Produces: auditable third-wave RED/GREEN, matrix, PTY, and crop-honesty self-review evidence.

- [ ] **Step 1: Run the complete automated matrix**

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
PROPTEST_CASES=1024 cargo test --features storybook --test storybook_properties
cargo test --all-targets
bash tests/scripts.sh
bash -n tests/scripts.sh herdr/install.sh herdr/run.sh herdr/control.sh
cargo build --release
git diff --check
```

Expected: every command exits zero; property tests report 4 passed and scripts report 20 passed.

- [ ] **Step 2: Run the Herdr-free real PTY smoke**

```bash
env -u HERDR_BIN_PATH -u HERDR_PANE_ID -u HERDR_PLUGIN_ID \
  -u HERDR_PLUGIN_STATE_DIR -u HERDR_PLUGIN_ROOT \
  -u HERDR_PLUGIN_CONFIG_DIR -u HERDR_SOCKET_PATH \
  cargo run --features storybook --bin questmancer-storybook
```

Inspect a connected Delve and a motion compatibility story, open/close help, return, and quit. Record validation, visible complete chambers/motion output, exit status, and terminal restoration sequences.

- [ ] **Step 3: Append the third-wave report and self-review**

Record both RED failures, focused counts, full matrix, PTY actions, implementation hash, and explicit review of every chamber height class: Hidden at zero, Text below eight, CompactScene from 14x8, Full from 28x10, and Departed persona None at every size. Confirm motion render dimensions came from each declared story viewport.

- [ ] **Step 4: Commit the report and confirm clean state**

```bash
git add -f .superpowers/sdd/questmancer-storybook-final-fix-report.md
git commit -m "docs: record connected chamber regression verification"
git status --porcelain
git diff --check 834c98a..HEAD
```

Expected: status is empty and diff-check exits zero.
