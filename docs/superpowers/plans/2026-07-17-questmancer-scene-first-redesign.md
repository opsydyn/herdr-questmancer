# Questmancer Scene-First Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build an original, dense 16-bit Guild Hall and Delve through a terminal-independent RGB renderer, prove both against fixed and live Herdr state, and keep the current production UI intact until a separate cutover decision.

**Architecture:** Introduce a new `scene` boundary beside `ui`: immutable `SceneSnapshot` facts are projected into deterministic world, camera, station and animation plans, painted into a reusable opaque `RgbBuffer`, then compressed into Ratatui cells by one half-block adapter. The feature-gated Storybook and live preview consume this path first; the existing `ui::render_with_projection` path remains the production default throughout this plan.

**Tech Stack:** Rust 1.90, Ratatui 0.30, Crossterm 0.29, Tokio, BLAKE3, `proptest` 1.11, Ratatui `TestBackend`, the existing feature-gated Storybook, and Herdr 0.7.4 for the guarded live-preview pass.

**Accepted design:** `docs/superpowers/specs/2026-07-17-questmancer-scene-first-redesign-design.md`

**Approved visual reference:** `reference-art/questmancer-option-a-north-star.png`

## Global Constraints

- Preserve the user's unrelated `.gitignore` and `PLAN.md` modifications; never stage them with this work.
- Use red-green-refactor for every behavior change and make each named test fail for the intended reason before editing production code.
- Keep `questmancer` and `opsydyn.questmancer` on the current production renderer for the whole plan.
- Do not remove, hide or alter the behavior of current Guild Hall, Delve, counsel, search, Reviewr, focus, persistence or plugin actions.
- Do not add prompting, search, selection, acknowledgement, refresh, Reviewr or sprite-manipulation controls to the new scene.
- Only `q`, `Ctrl+C` and process signals may exit the feature-gated live scene preview.
- Keep the `scene` core free of Ratatui, Crossterm, terminal input, runtime-clock reads, filesystem reads and Herdr socket access.
- Derive every station, pose, camera and effect from `SceneSnapshot`; do not persist scene geometry or actor location.
- Treat current `GuildAttention::Read` and `GuildAttention::Unread` equally in the new scene. Their `summons` and `since` fields provide transition identity and age; the new renderer has no acknowledgement workflow.
- Render original assets only. Do not copy Pixtuoid sprites, furniture, room geometry, labels or palette values, and do not ship generated north-star pixels.
- Target Unicode-capable true-colour terminals in the new path. Leave ANSI-256 and ASCII support on the legacy production path until the cutover review.
- Use one logical RGB pixel per terminal column and two logical RGB pixels per terminal row, emitted with `U+2580`.
- Use signed scene coordinates for blitting so sprites can clip at every edge without saturating into the wrong location.
- Reuse buffers across frames and perform no heap allocation inside the half-block flush.
- Keep static scenes event-driven; active animation must not exceed 8 FPS.
- Do not add image protocols, sound, weather, pathfinding, an ECS, a sprite editor, procedural dungeons or a theme framework in this plan.
- Run focused tests after every green step and the complete repository gate in Task 8.

---

## File Structure

### New renderer files

- `src/scene/mod.rs` — public scene facade, world dispatch and `SceneFrame` result.
- `src/scene/pixel.rs` — terminal-independent `Rgb`, `PixelSize`, `PixelPoint`, `PixelRect` and reusable opaque `RgbBuffer`.
- `src/scene/sprite.rs` — transparent `SpriteFrame`, signed clipped blitting and mirroring.
- `src/scene/snapshot.rs` — immutable, sorted projection from the existing `Model` into render facts.
- `src/scene/stage.rs` — automatic scene choice, camera anchors, truthful stations, actor ordering and cadence.
- `src/scene/assets/mod.rs` — shared indexed-sprite constructor and complete asset exports.
- `src/scene/assets/palette.rs` — original warm Guild Hall, cool Delve and persona colour vocabulary.
- `src/scene/assets/adventurer.rs` — compact original adventurer frames and persona recolouring.
- `src/scene/assets/guild_hall.rs` — Guild Hall tiles, architecture, furniture and prop sprites.
- `src/scene/assets/delve.rs` — Delve tiles, architecture, dungeon furniture and prop sprites.
- `src/scene/render/mod.rs` — shared painter ordering and camera-to-buffer transform.
- `src/scene/render/guild_hall.rs` — full Guild Hall scene painter.
- `src/scene/render/delve.rs` — full connected Delve scene painter.
- `src/scene/render/lighting.rs` — deterministic light masks, focus emphasis and reconnect dimming.
- `src/ui/scene_adapter.rs` — the only Ratatui adapter for the scene-first framebuffer.
- `src/bin/questmancer_scene_preview.rs` — feature-gated live Herdr preview with process-exit input only.

### New tests and evidence

- `tests/scene_pixel.rs` — buffer, geometry, clipping, sprite and mirroring examples.
- `tests/scene_pixel_properties.rs` — arbitrary-size clipping and no-panic properties.
- `tests/scene_adapter.rs` — half-block cell, offset, odd-height and resize-race tests.
- `tests/scene_snapshot.rs` — model-to-snapshot allow-list and stable-order tests.
- `tests/scene_stage.rs` — automatic scene, station, camera and cadence examples.
- `tests/scene_stage_properties.rs` — exactly-once actor and deterministic projection properties.
- `tests/scene_storybook.rs` — fixed scene catalogue, renderer and no-legacy-state tests.
- `tests/scene_guild_hall.rs` — Guild Hall composition and state evidence.
- `tests/scene_delve.rs` — connected Delve composition and state evidence.
- `tests/scene_live_preview.rs` — feature-gated renderer/input/default-path contracts.
- `docs/manual-test/questmancer-scene-preview.md` — exact Storybook and guarded live Herdr procedure.
- `docs/superpowers/reviews/2026-07-17-scene-first-cutover.md` — evidence ledger and explicit cutover questions.

### Principal modified files

- `Cargo.toml`, `src/lib.rs` — expose `scene`, add the `scene-preview` feature and preview binary.
- `src/terminal.rs` — share the existing runtime with an explicit legacy or scene-first render experience.
- `src/storybook/{assets,catalogue,fixtures,ui}.rs` — own and render fixed `PixelScene` stories.
- `tests/{storybook_catalogue,storybook_fixtures,storybook_rendering,storybook_properties}.rs` — extend exhaustive fixture handling.
- `justfile`, `README.md` — add developer-only renderer commands and describe the parallel migration.

---

## Task 1: Establish the terminal-independent RGB core and half-block boundary

**Files:**

- Create: `src/scene/mod.rs`
- Create: `src/scene/pixel.rs`
- Create: `src/scene/sprite.rs`
- Create: `src/ui/scene_adapter.rs`
- Modify: `src/lib.rs`
- Modify: `src/ui/mod.rs`
- Test: `tests/scene_pixel.rs`
- Test: `tests/scene_pixel_properties.rs`
- Test: `tests/scene_adapter.rs`

**Interfaces:**

- Produces: `Rgb`, `PixelPoint`, `PixelRect`, `PixelSize`, `RgbBuffer`, `SpriteFrame`, `blit`, `blit_mirrored`, and `ui::scene_adapter::flush_rgb`.
- Consumes: only standard-library types in `src/scene`; Ratatui is consumed only by `src/ui/scene_adapter.rs`.

- [ ] **Step 1: Write failing examples for opaque pixels and reusable buffers.**

  Add tests that compile against this exact public shape:

  ```rust
  use questmancer::scene::pixel::{PixelRect, Rgb, RgbBuffer};

  #[test]
  fn buffer_clear_and_clipped_fill_are_opaque() {
      let black = Rgb::new(0, 0, 0);
      let amber = Rgb::new(214, 139, 53);
      let mut buffer = RgbBuffer::filled(3, 2, black);
      buffer.fill_rect(PixelRect::new(-1, 1, 3, 2), amber);

      assert_eq!(buffer.get(0, 0), Some(black));
      assert_eq!(buffer.get(0, 1), Some(amber));
      assert_eq!(buffer.get(1, 1), Some(amber));
      assert_eq!(buffer.get(2, 1), Some(black));
  }

  #[test]
  fn ensure_size_reuses_capacity_at_the_same_size() {
      let mut buffer = RgbBuffer::filled(8, 6, Rgb::BLACK);
      let capacity = buffer.capacity();
      buffer.ensure_size(8, 6, Rgb::new(1, 2, 3));
      assert_eq!(buffer.capacity(), capacity);
      assert!(buffer.pixels().iter().all(|pixel| *pixel == Rgb::new(1, 2, 3)));
  }
  ```

- [ ] **Step 2: Run the examples and verify the module is absent.**

  ```bash
  cargo test --test scene_pixel
  ```

  Expected: compilation fails because `questmancer::scene` does not exist.

- [ ] **Step 3: Implement the minimal opaque pixel vocabulary.**

  In `src/scene/pixel.rs`, define:

  ```rust
  #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
  pub struct Rgb { pub r: u8, pub g: u8, pub b: u8 }

  impl Rgb {
      pub const BLACK: Self = Self::new(0, 0, 0);
      pub const fn new(r: u8, g: u8, b: u8) -> Self { Self { r, g, b } }
  }

  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub struct PixelPoint { pub x: i32, pub y: i32 }

  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub struct PixelSize { pub width: u16, pub height: u16 }

  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub struct PixelRect { pub x: i32, pub y: i32, pub width: u16, pub height: u16 }

  #[derive(Clone, Debug, Eq, PartialEq)]
  pub struct RgbBuffer {
      size: PixelSize,
      pixels: Vec<Rgb>,
  }
  ```

  Give `PixelPoint`, `PixelSize` and `PixelRect` `const fn new(...)` constructors matching their field order. Implement checked `get`, clipped `put`, `clear`, `fill_rect`, `pixels`, `pixels_mut`, `capacity`, and `ensure_size`. Reject impossible `width * height` allocations with checked multiplication and a clear panic in constructors; rendering call sites only pass terminal-derived `u16` sizes.

- [ ] **Step 4: Write failing transparent sprite and signed-clipping tests.**

  Test an `Option<Rgb>` frame blitted at `(-1, -1)`, at every positive edge, fully outside the target, and mirrored. Assert transparent pixels preserve the destination and negative origins do not collapse to `(0, 0)`.

- [ ] **Step 5: Implement `SpriteFrame`, `blit` and `blit_mirrored`.**

  Use this contract in `src/scene/sprite.rs`:

  ```rust
  #[derive(Clone, Debug, Eq, PartialEq)]
  pub struct SpriteFrame {
      size: PixelSize,
      pixels: Vec<Option<Rgb>>,
  }

  impl SpriteFrame {
      pub fn from_pixels(width: u16, height: u16, pixels: Vec<Option<Rgb>>) -> Self;
      pub const fn size(&self) -> PixelSize;
      pub fn pixels(&self) -> &[Option<Rgb>];
  }

  pub fn blit(frame: &SpriteFrame, origin: PixelPoint, target: &mut RgbBuffer);
  pub fn blit_mirrored(frame: &SpriteFrame, origin: PixelPoint, target: &mut RgbBuffer);
  ```

  Compute target coordinates as `i32`, skip negative or out-of-bounds coordinates, and write only `Some(Rgb)` pixels.

- [ ] **Step 6: Add property tests for arbitrary geometry.**

  With `proptest`, generate destination sizes `0..128`, frame sizes `0..40`, origins `-80..160`, transparent/opaque pixels and mirroring. Prove `blit` and `fill_rect` never panic, never resize the target, and never change a destination pixel not covered by an opaque in-bounds source pixel.

  Run:

  ```bash
  PROPTEST_CASES=1024 cargo test --test scene_pixel_properties
  ```

- [ ] **Step 7: Write failing half-block adapter tests.**

  Cover a two-pixel red/blue column, a non-zero destination `Rect`, an odd logical height with an explicit fallback colour, a zero-sized area, a target smaller than the requested area, and a buffer wider than the target. Assert the cell symbol is `▀`, foreground is the top RGB value, and background is the bottom RGB value.

- [ ] **Step 8: Implement the Ratatui-only adapter.**

  In `src/ui/scene_adapter.rs`, add:

  ```rust
  pub fn flush_rgb(
      target: &mut ratatui::buffer::Buffer,
      area: ratatui::layout::Rect,
      source: &RgbBuffer,
      odd_row_fill: Rgb,
  );
  ```

  Iterate cells directly, use the static `"▀"` symbol, clip against both `area` and `target.area`, map top/bottom pixels to `Color::Rgb`, and perform no `String`, `Vec`, `format!` or `Text` construction inside the function.

- [ ] **Step 9: Run the complete foundation tests.**

  ```bash
  cargo test --test scene_pixel --test scene_pixel_properties --test scene_adapter
  cargo test --test pixel
  cargo clippy --all-targets --all-features -- -D warnings
  ```

  Expected: all pass; the legacy `ui::pixel` tests remain unchanged.

- [ ] **Step 10: Self-review and commit.**

  Confirm `rg -n "ratatui|crossterm|HERDR|std::fs|std::time::SystemTime" src/scene` returns no hits. Commit:

  ```bash
  git add src/lib.rs src/scene src/ui/mod.rs src/ui/scene_adapter.rs tests/scene_pixel.rs tests/scene_pixel_properties.rs tests/scene_adapter.rs
  git commit -m "feat: add terminal-independent scene pixels"
  ```

---

## Task 2: Project a narrow immutable scene snapshot and truthful stage

**Files:**

- Create: `src/scene/snapshot.rs`
- Create: `src/scene/stage.rs`
- Modify: `src/scene/mod.rs`
- Test: `tests/scene_snapshot.rs`
- Test: `tests/scene_stage.rs`
- Test: `tests/scene_stage_properties.rs`
- Modify: `tests/support/strategies.rs`

**Interfaces:**

- Consumes: `Model`, `DomainState`, `ConnectionState`, `Motion`, persona/domain identifiers and `Timestamp`.
- Produces: `SceneSnapshot::from_model`, `ScenePlan::project`, `SceneConnection`, `WorldScene`, `SceneCamera`, `ActorPlacement`, `SceneEffect`, `TruthfulStation`, `ScenePose` and `SceneCadence`.

- [ ] **Step 1: Write failing allow-list tests for `SceneSnapshot`.**

  Build two models with identical connection, campaigns, agents, personas, focus and time. Change only `DomainState::selected_agent`, `GuildFocus`, `Modal`, output preview, notices and Reviewr availability. Assert their snapshots are equal. Then change an agent's Herdr-reported `focused`, `presence`, `presence_since` and `custom_status` fields one at a time and assert the snapshots differ.

- [ ] **Step 2: Define the immutable snapshot types.**

  Add this public shape in `src/scene/snapshot.rs`:

  ```rust
  #[derive(Clone, Debug, Eq, PartialEq)]
  pub enum SceneConnection {
      Offline,
      Connecting,
      Connected,
      Reconnecting { attempt: u32 },
      Incompatible { expected: u32, actual: u32 },
  }

  #[derive(Clone, Debug, Eq, PartialEq)]
  pub struct SceneSnapshot {
      pub connection: SceneConnection,
      pub campaigns: Vec<SceneCampaign>,
      pub agents: Vec<SceneAgent>,
      pub motion: Motion,
      pub now: Timestamp,
  }

  #[derive(Clone, Debug, Eq, PartialEq)]
  pub struct SceneCampaign {
      pub workspace_id: WorkspaceId,
      pub label: String,
      pub variant_seed: u64,
  }

  #[derive(Clone, Debug, Eq, PartialEq)]
  pub struct SceneAgent {
      pub key: AgentKey,
      pub workspace_id: WorkspaceId,
      pub name: String,
      pub custom_status: Option<String>,
      pub presence: Presence,
      pub presence_since: Timestamp,
      pub transition: Option<SceneTransition>,
      pub focused: bool,
      pub persona: AdventurerPersona,
  }

  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub struct SceneTransition {
      pub summons: GuildSummons,
      pub since: Timestamp,
  }
  ```

  `SceneSnapshot::from_model(&Model)` must preserve every `ConnectionState` field in `SceneConnection`, sort campaigns by `WorkspaceId` and agents by `AgentKey`, derive `variant_seed` from BLAKE3 of the workspace identity, and map every non-clear attention variant through only `summons()` and `since()`. It must not copy the current `View`, selection, modal, output, notices, Reviewr state, durable intent or goblin input state.

- [ ] **Step 3: Run snapshot tests and confirm stable ordering.**

  ```bash
  cargo test --test scene_snapshot
  ```

- [ ] **Step 4: Write failing table tests for automatic world choice.**

  Lock this priority, which is deterministic and has no manual override:

  1. non-connected connection state -> `GuildHall`;
  2. any blocked agent -> `GuildHall`;
  3. any `SpoilsReturned` transition younger than 3,000 ms -> `GuildHall`;
  4. a focused working agent -> `Delve`;
  5. any working agent -> `Delve`;
  6. otherwise -> `GuildHall`.

  Prove boundary behavior at 2,999 ms and 3,000 ms and define `COMPLETION_THEATRE_MS: i64 = 3_000` in `stage.rs`. A transition is fresh only when `now >= since`; future timestamps do not trigger theatre.

- [ ] **Step 5: Define the stage plan and exact truthful-station mapping.**

  Use this shape:

  ```rust
  pub enum WorldScene { GuildHall, Delve }
  pub enum SceneCamera { WholeRoom, Focused { anchor: CameraAnchor } }
  pub enum CameraAnchor { Door, CampaignTable(WorkspaceId), CounselBell, Hearth, Spoils, DelveParty(WorkspaceId) }
  pub enum TruthfulStation {
      CampaignToken(WorkspaceId), CounselBell, SpoilsBench, Hearth,
      DelveActive(WorkspaceId), DelveGate(WorkspaceId), DelveExit(WorkspaceId), DelveCamp(WorkspaceId),
  }
  pub enum ScenePose { Working, SeekingCounsel, ReturningWithSpoils, Settled, Resting, Unknown }
  pub struct ActorPlacement { pub agent: AgentKey, pub station: TruthfulStation, pub pose: ScenePose, pub focused: bool }
  pub enum SceneEffect {
      FreshSpoils { agent: AgentKey, since: Timestamp },
      RecentDeparture { workspace_id: WorkspaceId, since: Timestamp },
  }
  pub enum SceneCadence { EventDriven, Fps(u8) }
  pub struct ScenePlan {
      pub world: WorldScene,
      pub camera: SceneCamera,
      pub actors: Vec<ActorPlacement>,
      pub effects: Vec<SceneEffect>,
      pub cadence: SceneCadence,
  }
  ```

  Implement `ScenePlan::project(snapshot: &SceneSnapshot, viewport: PixelSize) -> Self` for automatic production choice and a crate-private `project_for_world(snapshot, viewport, world) -> Self` that performs the complete station/effect mapping for the supplied world. The latter exists so the feature-gated Storybook can inspect a truthful Delve state without introducing a manual world field into runtime facts.

  Guild Hall mapping: working and unknown -> campaign token; blocked -> Counsel Bell; done with a fresh spoils transition -> Spoils Bench/returning; settled done -> Spoils Bench/settled; idle -> Hearth; exited -> absent. Delve mapping: working -> active passage; blocked -> sealed gate; done -> exit stair; idle -> camp; unknown -> unlit active passage; exited -> absent.

  Fresh spoils and departure transitions become separate `SceneEffect` values for at most 3,000 ms. An exited adventurer never returns as an actor; `RecentDeparture` may leave door light, dust or an empty-hook cue without drawing that adventurer.

  Camera mapping: viewports at least `120x72` logical pixels use `WholeRoom`; smaller views focus, in priority order, on blocked, fresh spoils, focused agent, then Door. The focused agent changes emphasis and crop but never station or identity.

- [ ] **Step 6: Implement deterministic cadence.**

  `Motion::None` is event-driven. `Motion::Reduced` schedules at 1 FPS only when an idle actor is visible. `Motion::Full` uses the highest visible need, capped at 8 FPS: working 6, blocked 2, fresh spoils 8, idle 1, settled/unknown none.

- [ ] **Step 7: Prove stage invariants with `proptest`.**

  Extend `tests/support/strategies.rs` with scene snapshots that have unique agent keys. Prove:

  - every non-exited agent appears exactly once in the chosen world's plan;
  - exited agents never appear;
  - reordering input vectors produces the same plan;
  - changing legacy selection is impossible at the snapshot boundary;
  - fixed snapshot, viewport and time produce equal plans;
  - camera and station coordinates are never persisted into domain state;
  - arbitrary logical viewports `0..400 x 0..240` never panic.

  Run:

  ```bash
  PROPTEST_CASES=2048 cargo test --test scene_stage_properties
  ```

- [ ] **Step 8: Run focused and legacy truth tests.**

  ```bash
  cargo test --test scene_snapshot --test scene_stage --test scene_stage_properties
  cargo test --test reducer --test theatre --test persisted_state
  ```

- [ ] **Step 9: Self-review and commit.**

  Confirm the new scene code never calls `mark_read`, never reads `selected_agent`, and never mutates `Model` or `DomainState`. Commit:

  ```bash
  git add src/scene tests/scene_snapshot.rs tests/scene_stage.rs tests/scene_stage_properties.rs tests/support/strategies.rs
  git commit -m "feat: project truthful scene snapshots"
  ```

---

## Task 3: Add a scene-first Storybook lane and original asset vocabulary

**Files:**

- Create: `src/scene/assets/mod.rs`
- Create: `src/scene/assets/palette.rs`
- Create: `src/scene/assets/adventurer.rs`
- Create: `src/scene/render/mod.rs`
- Create: `src/scene/render/lighting.rs`
- Modify: `src/scene/mod.rs`
- Modify: `src/storybook/assets.rs`
- Modify: `src/storybook/catalogue.rs`
- Modify: `src/storybook/fixtures.rs`
- Modify: `src/storybook/ui.rs`
- Test: `tests/scene_storybook.rs`
- Modify: `tests/storybook_catalogue.rs`
- Modify: `tests/storybook_fixtures.rs`
- Modify: `tests/storybook_rendering.rs`
- Modify: `tests/storybook_properties.rs`

**Interfaces:**

- Consumes: Task 1 pixel/sprite APIs and Task 2 `SceneSnapshot`/`ScenePlan`.
- Produces: `StoryFixture::PixelScene`, `SceneFirstAsset`, `IndexedSprite`, `render_scene`, `SceneFrame`, and fixed scene-first stories.

- [ ] **Step 1: Write failing Storybook fixture-dispatch tests.**

  Add `StoryFixture::PixelScene(PixelSceneFixture)` expectations. Prove rendering a pixel scene never calls `crate::ui::render`, uses the Storybook canvas dimensions converted to logical pixels, and produces `▀` cells with `Color::Rgb` foreground/background.

- [ ] **Step 2: Extend the Storybook fixture boundary.**

  Add:

  ```rust
  pub struct PixelSceneFixture {
      pub snapshot: SceneSnapshot,
      pub world_override: Option<WorldScene>,
  }

  impl PixelSceneFixture {
      pub fn automatic(snapshot: SceneSnapshot) -> Self;
      pub fn in_world(snapshot: SceneSnapshot, world: WorldScene) -> Self;
  }

  pub enum StoryFixture {
      Application(Model),
      AssetAtlas(AssetAtlas),
      PixelScene(PixelSceneFixture),
  }
  ```

  `automatic` leaves production world selection intact. `in_world` exists only for fixed Storybook inspection and never enters `SceneSnapshot` or runtime state. Add a `#[cfg(feature = "storybook")] render_scene_for_story(snapshot, world_override, viewport, target)` facade that calls `ScenePlan::project` when no override exists and the crate-private `project_for_world` when one does. The override therefore recomputes truthful stations/effects for that world; it does not merely change an enum after projection.

  Define the initial renderer result now so later runtime work consumes a stable interface:

  ```rust
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub struct SceneFrame {
      pub world: WorldScene,
      pub next_frame_in: Option<std::time::Duration>,
  }

  pub fn render_scene(
      snapshot: &SceneSnapshot,
      viewport: PixelSize,
      target: &mut RgbBuffer,
  ) -> SceneFrame;
  ```

  In `storybook/ui.rs`, render `PixelScene` by reusing one `RgbBuffer`, calling `scene::render_scene_for_story(&fixture.snapshot, fixture.world_override, PixelSize::new(area.width, area.height.saturating_mul(2)), &mut buffer)`, and then `flush_rgb(frame.buffer_mut(), area, &buffer, Rgb::BLACK)`. Do not convert the scene into a temporary Ratatui `Terminal` or `Buffer`.

- [ ] **Step 3: Define the indexed original-asset constructor.**

  Add an `IndexedSprite` helper used only to author assets:

  ```rust
  pub struct IndexedPaletteEntry { pub key: char, pub colour: Option<Rgb> }

  pub fn indexed_sprite(
      rows: &[&str],
      palette: &[IndexedPaletteEntry],
  ) -> Result<SpriteFrame, AssetError>;
  ```

  Require equal character widths, reject duplicate keys and unknown glyphs, and reserve `.` for transparency. Tests must assert exact error variants for ragged rows, duplicates and missing palette entries. Runtime render paths receive parsed `SpriteFrame` values from `OnceLock`; they do not parse assets per frame.

- [ ] **Step 4: Author the compact adventurer vocabulary.**

  Create original `8x14` logical-pixel base frames for `Working`, `SeekingCounsel`, `ReturningWithSpoils`, `Settled`, `Resting` and `Unknown`, plus two walking/working alternates. Map the existing `AdventurerPersona` appearance onto skin, hair, cloth, metal and accent palette slots. Keep eyes and face details at one-pixel scale; do not reuse the current large `chamber_sprite` geometry.

  Add tests proving all ancestry/class combinations render within `8x14`, every opaque frame has a non-empty silhouette, and persona identity changes palette/detail without changing the station contract.

- [ ] **Step 5: Establish the shared painter and calibration room.**

  `render_scene` must clear/reuse the destination, obtain `ScenePlan`, and dispatch layer order:

  ```text
  background -> floor/walls -> fixed architecture -> furniture/props
             -> y-sorted actors -> lighting -> transition effects
  ```

  The first calibration scene contains original stone, timber, rug, table, candle and two compact adventurers in a continuous `120x72` logical-pixel room. It is deliberately small enough to review the engine before the full Guild Hall art task.

- [ ] **Step 6: Add exhaustive Storybook ownership.**

  Add `AssetId::SceneFirst(SceneFirstAsset)` and a compile-time exhaustive label match. Start `SceneFirstAsset::ALL` with `CalibrationRoom` and `CompactAdventurers`. Add fixed `Scenes / RGB Calibration Room` and `Atlas / Compact Scene Adventurers` stories. Update every exhaustive test/match for the new fixture and asset variants.

- [ ] **Step 7: Prove the new lane does not depend on legacy mutable UI state.**

  Add a source audit over `src/scene` rejecting `GuildFocus`, `Modal`, `OutputPreview`, `Reviewr`, `selected_agent`, `reduce_action`, `AgentCommand`, `ratatui` and `crossterm`. Add a rendering test that changes all available legacy interaction fields and receives byte-identical RGB pixels from equal `SceneSnapshot` values.

- [ ] **Step 8: Run Storybook and visual-review the calibration room.**

  ```bash
  cargo test --features storybook --test scene_storybook
  cargo test --features storybook --test storybook_catalogue --test storybook_fixtures --test storybook_rendering --test storybook_properties
  just storybook
  ```

  In Storybook, inspect the calibration room at `160x45`, `120x36` and `80x24`. Reject the slice if it reads as widgets placed on an empty background, if adventurers dominate the room, or if text is needed to identify the table, hearth-like light or architecture.

- [ ] **Step 9: Self-review and commit.**

  Confirm the new pixels are original, all assets have one Storybook owner, `src/scene` is terminal-free, and the existing Application/AssetAtlas stories are unchanged. Commit:

  ```bash
  git add src/scene src/storybook tests/scene_storybook.rs tests/storybook_catalogue.rs tests/storybook_fixtures.rs tests/storybook_rendering.rs tests/storybook_properties.rs
  git commit -m "feat: add scene-first Storybook rendering"
  ```

---

## Task 4: Author the complete Guild Hall vertical slice

**Files:**

- Create: `src/scene/assets/guild_hall.rs`
- Create: `src/scene/render/guild_hall.rs`
- Modify: `src/scene/assets/mod.rs`
- Modify: `src/scene/render/mod.rs`
- Modify: `src/scene/render/lighting.rs`
- Modify: `src/storybook/assets.rs`
- Modify: `src/storybook/catalogue.rs`
- Modify: `src/storybook/fixtures.rs`
- Test: `tests/scene_guild_hall.rs`
- Modify: `tests/scene_storybook.rs`
- Modify: `tests/storybook_catalogue.rs`

**Interfaces:**

- Consumes: `ScenePlan` actors/stations, indexed sprites, camera transform and shared lighting.
- Produces: `render::guild_hall::paint`, all Guild Hall environment assets, and six fixed Guild Hall review stories.

- [ ] **Step 1: Write failing composition and truthful-station tests.**

  For a fixed `160x90` logical scene, prove the rendered buffer contains owned rectangles for Door, Quest Wall, campaign tables, Counsel Bell, Hearth and Spoils Bench; actors occupy their projected station bounds; exited agents have no actor pixels; and two agents never own the same final actor anchor.

- [ ] **Step 2: Lock the Guild Hall authored asset set.**

  Author original indexed sprites/tiles for:

  ```text
  ashlar wall, timber beam, plank/stone floor, rug, guild door,
  quest map wall, campaign table, chairs/benches, counsel bell,
  hearth/fireplace, spoils bench/chest, shelves/books/scrolls,
  banners, candles, mugs, dice, wax seals and small clutter
  ```

  Every large surface must have at least three material shades and deterministic detail variants. Warm palette roles are amber light, ember orange, oak brown, parchment cream, wine red and deep neutral shadow. Scene variants may change decorative pixels from workspace seed but must not imply state.

- [ ] **Step 3: Paint one continuous authored room.**

  Use a canonical `160x90` logical-pixel composition with architecture touching every edge. Wider buffers extend wall/floor material deterministically before centring the canonical room; smaller buffers use `SceneCamera::Focused` crops without scaling. Do not subdivide the scene into Ratatui rectangles or draw borders around stations.

- [ ] **Step 4: Paint truthful actors and one-shot completion theatre.**

  Draw tokens on campaign tables for working/unknown, a projected compact actor at the bell for blocked, returned actors at the spoils bench for done, and resting actors near the hearth for idle. A fresh spoils transition may animate chest light/confetti for at most 3,000 ms; settled done remains visually complete without an endless effect.

- [ ] **Step 5: Paint connection truth through the room.**

  Connected uses normal lighting. Connecting/reconnecting lowers global light and lights the guild door. Offline closes/darkens the door. Incompatible adds one small factual diagnostic overlay after the scene; it does not replace the room with a panel.

- [ ] **Step 6: Add fixed Guild Hall stories and ownership.**

  Add `SceneFirstAsset` variants and stories for:

  ```text
  Guild Hall Empty
  Guild Hall Mixed Party
  Guild Hall Counsel Requested
  Guild Hall Spoils Returned
  Guild Hall Reconnecting
  Guild Hall Minimum Viewport
  ```

  Use `SceneSnapshot` fixtures directly. The Storybook may choose these fixtures; the production scene has no fixture picker or state controls.

- [ ] **Step 7: Add visual-density and deterministic-output evidence.**

  Assert at `160x90` that at least 85% of logical pixels differ from the clear colour, at least 24 distinct RGB colours are present, every landmark has non-background pixels, and rendering the same snapshot/time twice yields identical BLAKE3 bytes. These are regression floors, not substitutes for visual review.

- [ ] **Step 8: Review against the north star.**

  Run:

  ```bash
  cargo test --features storybook --test scene_guild_hall --test scene_storybook --test storybook_catalogue
  just storybook
  ```

  Inspect the six stories against `reference-art/questmancer-option-a-north-star.png`. Require environment dominance, tiny embedded adventurers, continuous materials, readable stations without explanatory labels, warm depth and no dashboard chrome.

- [ ] **Step 9: Self-review and commit.**

  Confirm visual variants are workspace-seeded, all operational changes derive from snapshot facts, and no generated/reference pixels entered `src`. Commit:

  ```bash
  git add src/scene src/storybook tests/scene_guild_hall.rs tests/scene_storybook.rs tests/storybook_catalogue.rs
  git commit -m "feat: author the scene-first Guild Hall"
  ```

---

## Task 5: Author the complete connected Delve vertical slice

**Files:**

- Create: `src/scene/assets/delve.rs`
- Create: `src/scene/render/delve.rs`
- Modify: `src/scene/assets/mod.rs`
- Modify: `src/scene/render/mod.rs`
- Modify: `src/scene/render/lighting.rs`
- Modify: `src/storybook/assets.rs`
- Modify: `src/storybook/catalogue.rs`
- Modify: `src/storybook/fixtures.rs`
- Test: `tests/scene_delve.rs`
- Modify: `tests/scene_storybook.rs`
- Modify: `tests/storybook_catalogue.rs`

**Interfaces:**

- Consumes: the same `SceneSnapshot`, `ScenePlan`, actor sprites, camera and framebuffer as Guild Hall.
- Produces: `render::delve::paint`, all Delve environment assets, and five fixed Delve review stories.

- [ ] **Step 1: Write failing connectivity and station tests.**

  Prove the walkable visual floor is one connected authored dungeon: every room doorway connects through floor pixels to the entrance/exit; shared walls are painted once; actors sit within their station regions; and no chamber is surrounded by a card border or independent background.

- [ ] **Step 2: Lock the Delve authored asset set.**

  Author original indexed sprites/tiles for:

  ```text
  dressed stone wall, cracked/mossy floor, arches, doors, descending stair,
  active passage, sealed gate, exit landing, camp, torch, brazier,
  rune stones, roots, columns, rubble, puddles, bones, chests and dungeon clutter
  ```

  Use cool teal/blue-green ambient light, restrained amber torches, moss green, mineral violet and deep blue-black shadow. Workspace identity may choose masonry, moss and prop variants but may not change agent state representation.

- [ ] **Step 3: Paint one connected dungeon composition.**

  Use a canonical `160x90` logical scene with entrance, central junction, two side rooms, descending corridor and camp/exit. Every space shares the same floor/wall coordinate system. Wider and smaller viewports use the same extension/camera rules as Guild Hall; never generate a grid of per-agent rooms.

- [ ] **Step 4: Paint truthful Delve stations.**

  Working actors occupy active passages, blocked actors wait at a sealed gate, done actors stand on the exit landing, idle actors rest at camp, unknown actors remain in an unlit passage, and exited actors are absent. Painter order uses each actor's foot row so sprites pass behind foreground arches/props where appropriate.

- [ ] **Step 5: Add fixed Delve stories and ownership.**

  Add stories for:

  ```text
  Delve Active Party
  Delve Mixed States
  Delve Sealed Gate
  Delve Reconnecting
  Delve Minimum Viewport
  ```

  The sealed-gate story uses `PixelSceneFixture::in_world(snapshot, WorldScene::Delve)`. The `#[cfg(feature = "storybook")]` override facade is unavailable to the default production build; keep automatic world choice unchanged.

- [ ] **Step 6: Add deterministic, density and palette evidence.**

  At `160x90`, require at least 85% non-clear pixels, at least 24 distinct colours, connected floor evidence, all named architecture regions non-empty and deterministic BLAKE3 output. Define mean scene coolness as `mean(g + b - 2*r)` over opaque buffer pixels using signed arithmetic; the Delve fixture must exceed the Guild Hall fixture by at least 20.

- [ ] **Step 7: Review against the north star.**

  ```bash
  cargo test --features storybook --test scene_delve --test scene_storybook --test storybook_catalogue
  PROPTEST_CASES=2048 cargo test --test scene_stage_properties
  just storybook
  ```

  Require a single dungeon world, small embedded actors, coherent depth, connected passages, cool lighting and no arbitrary divider lines or chamber cards.

- [ ] **Step 8: Self-review and commit.**

  Confirm Guild Hall and Delve share the engine and actor vocabulary but not a continuous map, and Storybook-only world forcing cannot be constructed by production code. Commit:

  ```bash
  git add src/scene src/storybook tests/scene_delve.rs tests/scene_storybook.rs tests/storybook_catalogue.rs
  git commit -m "feat: author the scene-first Delve"
  ```

---

## Task 6: Prove responsive cameras, motion and idle cost

**Files:**

- Modify: `src/scene/stage.rs`
- Modify: `src/scene/render/mod.rs`
- Modify: `src/scene/render/guild_hall.rs`
- Modify: `src/scene/render/delve.rs`
- Modify: `src/storybook/catalogue.rs`
- Modify: `src/storybook/fixtures.rs`
- Test: `tests/scene_stage.rs`
- Test: `tests/scene_stage_properties.rs`
- Test: `tests/scene_guild_hall.rs`
- Test: `tests/scene_delve.rs`
- Modify: `tests/storybook_properties.rs`

**Interfaces:**

- Consumes: fixed scene painters and Task 2 stage contracts.
- Produces: final automatic camera transforms, deterministic animation phases, `SceneFrame::next_frame_in`, and reduced/no-motion stories.

- [ ] **Step 1: Write failing viewport-matrix tests.**

  Render both worlds at logical sizes `0x0`, `1x1`, `80x48`, `120x72`, `160x90`, `240x120` and arbitrary `0..400 x 0..240`. Prove no panic, target size remains exact, focused crops stay inside authored-world bounds, and non-empty supported viewports contain environment pixels.

- [ ] **Step 2: Implement camera transforms without scaling.**

  Whole-room cameras centre the canonical authored scene and extend material into surplus space. Focused cameras clamp a viewport-sized crop around the stage anchor. Never resample pixels, stretch sprites or create a separate narrow layout.

- [ ] **Step 3: Write failing animation-phase tests.**

  For fixed snapshot timestamps, assert exact frame changes at 6 FPS working, 2 FPS blocked, 8 FPS fresh completion and 1 FPS idle. Assert settled scenes return no deadline and `Motion::None` produces identical frames for all timestamps.

- [ ] **Step 4: Complete exact frame invalidation through the existing renderer result.**

  Retain the Task 3 interface exactly:

  ```rust
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub struct SceneFrame {
      pub world: WorldScene,
      pub next_frame_in: Option<std::time::Duration>,
  }

  pub fn render_scene(
      snapshot: &SceneSnapshot,
      viewport: PixelSize,
      target: &mut RgbBuffer,
  ) -> SceneFrame;
  ```

  Compute phase and next deadline from the supplied `Timestamp`; never sample a clock in `scene`. This task completes its phase-boundary behavior rather than adding a second frame-result type.

- [ ] **Step 5: Prove buffer reuse and static cadence.**

  Render 1,000 fixed frames through one buffer and assert capacity remains unchanged. Prove a static snapshot returns `None`, and active snapshots never return a period shorter than 125 ms. Do not add wall-clock performance assertions to CI.

- [ ] **Step 6: Add fixed compatibility-within-scope stories.**

  Add scene-first stories for full, reduced and no motion plus minimum viewport. Do not add ANSI-256 or ASCII scene stories; those remain cutover questions served by the legacy renderer.

- [ ] **Step 7: Run the renderer stress gate.**

  ```bash
  cargo test --test scene_stage --test scene_stage_properties --test scene_guild_hall --test scene_delve
  PROPTEST_CASES=4096 cargo test --test scene_pixel_properties --test scene_stage_properties
  cargo test --features storybook --test storybook_properties --test scene_storybook
  ```

- [ ] **Step 8: Self-review and commit.**

  Confirm every timer is justified by visible animation, time is injected, no scaling entered the camera, and the renderer reuses the caller's buffer. Commit:

  ```bash
  git add src/scene src/storybook tests/scene_stage.rs tests/scene_stage_properties.rs tests/scene_guild_hall.rs tests/scene_delve.rs tests/storybook_properties.rs
  git commit -m "feat: complete scene cameras and cadence"
  ```

---

## Task 7: Add a feature-gated live Herdr preview without changing production

**Files:**

- Modify: `Cargo.toml`
- Create: `src/bin/questmancer_scene_preview.rs`
- Modify: `src/terminal.rs`
- Modify: `justfile`
- Test: `tests/scene_live_preview.rs`
- Modify: `tests/runtime_loop.rs`
- Modify: `tests/scripts.sh`

**Interfaces:**

- Consumes: the current Herdr/persistence/runtime model and `scene::render_scene`.
- Produces: `terminal::run_scene_preview`, `RenderExperience`, generic animation scheduling and `just scene-preview`.

- [ ] **Step 1: Write failing default-path and preview-input tests.**

  Prove the release binary still calls `terminal::run`, the plugin manifest still invokes only `questmancer`, and no `questmancer-scene-preview` string appears in `herdr-plugin.toml` or `herdr/`. For preview input, assert only plain `q`, `Ctrl+C`, stream close and process signals exit; `1`, `2`, arrows, Enter, `r`, `/`, Space, mouse and paste events produce no action or command.

- [ ] **Step 2: Add the feature-gated binary contract.**

  Extend `Cargo.toml`:

  ```toml
  [features]
  default = []
  storybook = []
  scene-preview = []

  [[bin]]
  name = "questmancer-scene-preview"
  path = "src/bin/questmancer_scene_preview.rs"
  required-features = ["scene-preview"]
  ```

  The binary installs the existing panic hook and calls `terminal::run_scene_preview().await`. It has no Herdr plugin manifest entry and is not packaged by release automation.

- [ ] **Step 3: Make render experience explicit in the existing runtime.**

  Add an internal enum:

  ```rust
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  enum RenderExperience { Legacy, SceneFirstPreview }
  ```

  Keep `terminal::run(initial_view)` as a wrapper over `run_with_experience(initial_view, Legacy)`. Gate `run_scene_preview()` behind `scene-preview` and call `run_with_experience(None, SceneFirstPreview)`. Both paths reuse startup, Herdr supervision, persistence diagnostics, terminal restoration and model updates.

- [ ] **Step 4: Generalise animation scheduling without teaching it scenes.**

  Replace `AnimationScheduler::reset_for(model, area, projection, clock)` with:

  ```rust
  pub fn reset_after(
      &mut self,
      sampled_at: Timestamp,
      delay: Option<Duration>,
      clock: &RuntimeClock,
  );
  ```

  The legacy draw computes `next_projected_frame_in`; the scene preview draw computes `SceneFrame::next_frame_in`. Both pass the result to the same scheduler. Preserve all current scheduler tests and add parity tests for `None` and exact deadlines.

- [ ] **Step 5: Render live snapshots and suppress interaction effects.**

  On each preview draw, build `SceneSnapshot::from_model(model)`, ensure one reusable buffer matches `frame.area().width x frame.area().height * 2`, render the scene and flush it. Preview input is handled by a small `preview_exit_for_event` predicate; it never calls `reduce_action`, `dispatch_action_effects`, `CommandExecutor`, persistence mutation or `mark_selected_attention_read`.

- [ ] **Step 6: Add developer commands and release-surface guards.**

  Add:

  ```make
  scene-preview:
      cargo run --features scene-preview --bin questmancer-scene-preview

  scene-preview-test:
      cargo test --all-targets --features scene-preview
  ```

  Extend `tests/scripts.sh` to fail if the preview binary leaks into the manifest, install script, run script, control script or release archive list.

- [ ] **Step 7: Run default, feature and release-surface tests.**

  ```bash
  cargo test --test scene_live_preview --features scene-preview
  cargo test --test runtime_loop
  cargo test --all-targets
  cargo test --all-targets --all-features
  bash tests/scripts.sh
  cargo build --release
  ```

  Expected: the default/release binary is unchanged; the preview binary builds only with its feature.

- [ ] **Step 8: Self-review and commit.**

  Confirm the preview shares runtime truth but no action path, the default binary calls the legacy renderer, and release packaging still contains exactly one `questmancer` binary. Commit:

  ```bash
  git add Cargo.toml Cargo.lock src/bin/questmancer_scene_preview.rs src/terminal.rs justfile tests/scene_live_preview.rs tests/runtime_loop.rs tests/scripts.sh
  git commit -m "feat: add live scene-first preview"
  ```

---

## Task 8: Capture evidence and hold the cutover gate

**Files:**

- Create: `docs/manual-test/questmancer-scene-preview.md`
- Create: `docs/superpowers/reviews/2026-07-17-scene-first-cutover.md`
- Modify: `README.md`
- Modify: `reference-art/README.md`

**Interfaces:**

- Consumes: complete Storybook and live-preview paths.
- Produces: a reproducible visual/manual test, measured evidence ledger and an explicit stop before production cutover.

- [ ] **Step 1: Document the exact offline Storybook review.**

  Include:

  ```bash
  cd /Users/alancurrie/Projects/herdr-web-master
  just storybook-test
  just storybook
  ```

  List every scene-first story by exact title, the reference viewport, minimum viewport, motion variants, expected truthful station and the corresponding north-star criterion. State clearly that Storybook controls are development-only.

- [ ] **Step 2: Document the guarded live Herdr preview.**

  Require a clean baseline, Herdr 0.7.4, a pre-existing server, and an existing linked plugin. Never stop the server or unlink the plugin. From an existing Herdr shell/pane where `HERDR_SOCKET_PATH` is already exported, verify the environment, then build and launch only the feature-gated preview:

  ```bash
  test -n "$HERDR_SOCKET_PATH"
  cargo build --features scene-preview --bin questmancer-scene-preview
  cargo run --features scene-preview --bin questmancer-scene-preview
  ```

  Verify working -> Delve, blocked -> Guild Hall Counsel Bell, done -> fresh Spoils theatre then settled return, idle -> Hearth, exited -> absent, reconnect truth, automatic camera, `q` exit and ignored legacy action keys. If Herdr 0.7.4 cannot synthesize a state, mark it blocked rather than passed.

- [ ] **Step 3: Record objective evidence without approving cutover.**

  In the review ledger, add fields for:

  ```text
  Guild Hall visual approval
  Delve visual approval
  state-truth pass/fail/blocked table
  minimum viewport
  full/reduced/no-motion review
  idle CPU sample after 30 seconds static
  active CPU sample with working animation
  reconnect behavior
  terminal restore
  known visual defects
  ANSI-256 decision
  legacy renderer delete-or-dev-only decision
  cutover decision: NOT YET DECIDED
  ```

  Do not pre-fill measurements or mark visual approval in code review.

- [ ] **Step 4: Update user-facing architecture and developer commands.**

  README must state that the current plugin UI remains production, the new scene-first renderer is an experimental developer preview, Questmancer does not prompt agents in that preview, and Codex CLI plus Herdr remain authoritative. Link the design, north star, Storybook command and manual preview guide.

- [ ] **Step 5: Run the complete automated gate.**

  ```bash
  cargo fmt --all --check
  cargo clippy --all-targets --all-features -- -D warnings
  cargo test --all-targets --all-features
  PROPTEST_CASES=4096 cargo test --test scene_pixel_properties --test scene_stage_properties
  bash tests/scripts.sh
  bash -n herdr/install.sh herdr/run.sh herdr/control.sh
  cargo build --release
  git diff --check
  ```

  Expected: every command exits zero. Record exact failures as code, environment or blocked-manual evidence; do not translate an environmental limitation into a pass.

- [ ] **Step 6: Run Storybook visual review and the guarded live pass.**

  Capture terminal screenshots for Guild Hall mixed party, Counsel Bell, returned spoils, Delve active party, sealed gate and minimum viewport. Store only approved, original evidence under `docs/assets/scene-first/`; do not copy Pixtuoid or generated north-star pixels into production assets.

- [ ] **Step 7: Stop at the cutover review.**

  Present the completed ledger to the user. Do not switch `questmancer`, remove legacy controls, alter the plugin manifest or decide ANSI compatibility inside this plan. A production cutover requires an approved follow-up plan based on the evidence.

- [ ] **Step 8: Commit the evidence package.**

  Stage only the documentation and approved screenshots produced by this task:

  ```bash
  git add README.md reference-art/README.md docs/manual-test/questmancer-scene-preview.md docs/superpowers/reviews/2026-07-17-scene-first-cutover.md docs/assets/scene-first
  git commit -m "docs: prepare the scene-first cutover review"
  ```

---

## Completion Boundary

This plan is complete when both authored worlds run through the new RGB pipeline in Storybook and the feature-gated live Herdr preview, every automated gate passes, and the cutover ledger is ready for user review. Completion does **not** mean the new renderer is the shipped default.

After the user approves the evidence, write a small follow-up cutover plan that makes exactly three decisions explicit:

1. flip the production renderer from legacy to scene-first;
2. delete the legacy UI or retain it as a development-only binary;
3. add ANSI-256 quantisation or document true colour as the release requirement.
