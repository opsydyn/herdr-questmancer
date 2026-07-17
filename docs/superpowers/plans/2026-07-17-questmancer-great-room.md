# Questmancer Great Room Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the Guild Hall panel dashboard with the accepted Great Room, preserving operational truth and existing commands while fixing the stale connection notice and Delve route-home regressions.

**Architecture:** Keep `Model` and `DomainState` authoritative. Add a pure Great Room projection that derives room mode, fixed landmarks, campaign tables, and exactly one truthful representation for each non-exited adventurer. Render that projection through one scene renderer and small landmark widgets; responsive layouts are cameras over the same room, not separate stateful views.

**Tech stack:** Rust 1.90, Ratatui 0.30, Crossterm, existing domain/reducer/runtime boundaries, `proptest` 1.11, Ratatui `TestBackend`, the feature-gated Storybook binary, and Herdr 0.7.4 for the guarded manual pass.

**Accepted design:** `docs/superpowers/specs/2026-07-17-questmancer-great-room-design.md`

## Global constraints

- Preserve the user's unrelated `PLAN.md` modification; never stage it with this work.
- Use red-green-refactor for every behavior change and make the named test fail for the intended reason before editing production code.
- Do not persist room locations. Geometry and stations are derived projections.
- Do not create a second Guild Hall state model for wide, medium, or narrow rendering.
- Do not alter the Herdr protocol or introduce Herdr 0.7.4 sidebar-row integration in this slice.
- Reuse the production persona composer in `src/ui/persona/chamber_sprite.rs`; Storybook stories must call production renderers.
- Keep ASCII, ANSI-16, reduced-motion, and no-motion modes semantically complete.
- Run focused tests after each green step and the complete release gate in Task 8.

## Intended file structure

### New files

- `src/ui/guild_room_projection.rs` — pure room modes, geometry, campaign identity, and Truthful Stations.
- `src/ui/views/great_room.rs` — layered scene renderer for all three camera modes.
- `src/ui/widgets/guild_landmark.rs` — reusable production furniture and landmark renderers.
- `tests/guild_room_projection.rs` — example-driven projection tests.
- `tests/guild_room_properties.rs` — arbitrary geometry and representation invariants.

### Principal modified files

- `src/app.rs`, `src/runtime_loop.rs`, `src/interaction.rs`, `src/terminal.rs` — typed notices and landmark focus.
- `src/ui/mod.rs`, `src/ui/views/guild_hall.rs`, `src/ui/views/delve.rs`, `src/ui/views/mod.rs`, `src/ui/widgets/mod.rs` — projection and rendering integration.
- `src/storybook/{assets,atlas,catalogue,fixtures}.rs` — exhaustive Great Room asset ownership and fixed scenes.
- `tests/{runtime_loop,interaction,delve_rendering,guild_hall_rendering,render_projection}.rs` — regressions and parity.
- `README.md`, `docs/manual-test/questmancer-0.1.0.md` — user-facing room model and guarded Herdr test.

---

## Task 1: Type notices and remove contradictory connection theatre

**Files:**

- Modify: `src/app.rs`
- Modify: `src/runtime_loop.rs`
- Modify: `src/interaction.rs`
- Modify: `src/terminal.rs`
- Modify: `src/ui/views/guild_hall.rs`
- Modify: `src/ui/views/reply.rs`
- Test: `tests/app.rs`
- Test: `tests/goblins.rs`
- Test: `tests/runtime_loop.rs`
- Test: `tests/interaction.rs`
- Test: `tests/guild_hall_rendering.rs`

- [ ] **Write the failing connection regression tests.**

  Add tests proving that bootstrap produces a connection diagnostic, a successful `ConnectionUpdate::Connected` removes only that diagnostic, and action/persistence/integration notices survive connection success. Add a rendering assertion that `CONNECTED` and `connecting to Herdr` cannot coexist.

- [ ] **Run the focused tests and confirm the intended failures.**

  Run:

  ```bash
  cargo test --test runtime_loop connected_clears_only_connection_notice -- --exact
  cargo test --test guild_hall_rendering connected_room_never_renders_connecting_notice -- --exact
  ```

  Expected: tests fail because `Model` still owns one untyped `status_message`.

- [ ] **Introduce an origin-aware notice type in `src/app.rs`.**

  Implement this semantic shape, adapting visibility to existing crate conventions:

  ```rust
  #[derive(Clone, Debug, Eq, PartialEq)]
  pub enum Notice {
      ConnectionDiagnostic(String),
      ActionFeedback(String),
      PersistenceDiagnostic(String),
      IntegrationDiagnostic(String),
  }

  impl Notice {
      pub fn message(&self) -> &str { /* exhaustive match */ }
      pub const fn is_connection_diagnostic(&self) -> bool { /* variant match */ }
  }
  ```

  Replace `Model::status_message: Option<String>` with `notice: Option<Notice>`. Add explicit setters for each origin plus `clear_connection_notice()`. Keep a read-only `status_message() -> Option<&str>` compatibility accessor during this slice so renderers can read common copy without losing the notice origin. Remove the generic `set_status_message` mutator and migrate every producer, including tests, so new untyped writes cannot re-enter the model.

- [ ] **Migrate notice producers without classifying strings.**

  Map startup/reconnect messages in `runtime_loop.rs` to `ConnectionDiagnostic`; command success/failure in `runtime_loop.rs` and user operations in `interaction.rs` to `ActionFeedback` or `IntegrationDiagnostic`; startup/state warnings in `terminal.rs` to `PersistenceDiagnostic`. Migrate fixture setup in `tests/app.rs`, `tests/goblins.rs`, and the named test suites through the corresponding typed setters. Do not inspect message prefixes to choose a variant.

- [ ] **Clear only the connection diagnostic on a confirmed connection.**

  In the existing connection-update path, set `ConnectionState::Connected` and call `clear_connection_notice()`. Preserve all other notice variants and the last complete domain snapshot.

- [ ] **Render notices at their truthful landmark.**

  Until the Great Room renderer lands, keep connection diagnostics with connection theatre and ordinary feedback in the existing status location. Ensure incompatible protocol detail remains exact.

- [ ] **Run the complete focused suite.**

  ```bash
  cargo test --test app
  cargo test --test goblins
  cargo test --test runtime_loop
  cargo test --test interaction
  cargo test --test guild_hall_rendering
  cargo clippy --all-targets --all-features -- -D warnings
  ```

- [ ] **Self-review and commit.**

  Check that every producer chooses a notice origin deliberately, no renderer uses string-prefix matching, and no persistence shape changed. Commit:

  ```bash
  git add src/app.rs src/runtime_loop.rs src/interaction.rs src/terminal.rs src/ui/views/guild_hall.rs src/ui/views/reply.rs tests/app.rs tests/goblins.rs tests/runtime_loop.rs tests/interaction.rs tests/guild_hall_rendering.rs
  git commit -m "fix: separate Questmancer notice origins"
  ```

---

## Task 2: Replace the complete Delve route-home row

**Files:**

- Modify: `src/ui/views/delve.rs`
- Test: `tests/delve_rendering.rs`

- [ ] **Add a failing regression that reproduces the residual text.**

  Render the connected multi-delve fixture at the screenshot width and assert the buffer contains `HOME PATH` exactly where the route changes and never contains `HOMET PATH` or remnants of the previous architecture row.

- [ ] **Run the test and observe the corruption.**

  ```bash
  cargo test --test delve_rendering route_home_replaces_the_complete_architecture_row -- --exact
  ```

- [ ] **Render the route overlay across its full owned row.**

  Change `render_route_home` to produce the same full-width `architecture_row(area, path)` shape used by the underlying scene, then overwrite the complete row. Do not fix this by adding a trailing space to the nine-column label.

- [ ] **Verify all Delve render modes.**

  ```bash
  cargo test --test delve_rendering
  cargo test --test delve_scene
  cargo test --test delve_widgets
  ```

- [ ] **Self-review and commit.**

  Confirm Unicode and ASCII fixtures both replace the full row and no connected Delve architecture changed otherwise. Commit:

  ```bash
  git add src/ui/views/delve.rs tests/delve_rendering.rs
  git commit -m "fix: replace the complete Delve home row"
  ```

---

## Task 3: Project deterministic Great Room geometry and campaign identity

**Files:**

- Create: `src/ui/guild_room_projection.rs`
- Modify: `src/ui/mod.rs`
- Test: `tests/guild_room_projection.rs`
- Test: `tests/render_projection.rs`

- [ ] **Write table-driven failing tests for modes, labels, and geometry.**

  Cover widths `119`, `120`, `79`, and `80`; zero-sized areas; multiple workspaces; and campaign label fallback in this order: meaningful workspace label, checkout basename, workspace ID. Include generic labels `""`, whitespace, and `"~"` as rejected labels.

- [ ] **Run the new tests and confirm the module/API is absent.**

  ```bash
  cargo test --test guild_room_projection
  ```

- [ ] **Define the pure projection vocabulary.**

  In `src/ui/guild_room_projection.rs`, add:

  ```rust
  pub enum GuildRoomMode { WholeRoom, CroppedRoom, LandmarkCamera }
  pub enum GuildLandmark {
      Door,
      QuestWall,
      CampaignTable(WorkspaceId),
      CounselBell,
      Hearth,
      Chronicle,
      Scrying,
      Spoils,
  }
  pub struct ProjectedLandmark { pub landmark: GuildLandmark, pub area: Rect }
  pub struct ProjectedCampaignTable {
      pub workspace_id: WorkspaceId,
      pub label: String,
      pub seal: u64,
      pub area: Rect,
      pub selected: bool,
  }
  pub struct GuildRoomProjection {
      pub mode: GuildRoomMode,
      pub landmarks: Vec<ProjectedLandmark>,
      pub campaigns: Vec<ProjectedCampaignTable>,
      pub adventurers: Vec<AdventurerRepresentation>,
  }
  ```

  Use deterministic hashes already accepted by the project for seals/table identity. Geometry must use saturating Ratatui layout operations and tolerate empty areas.

- [ ] **Implement one mode-independent projection entry point.**

  Add `project(model: &Model, area: Rect) -> GuildRoomProjection`. Derive the mode solely from the supplied area, then place the same stable landmark identities for the selected camera. Do not perform I/O, mutate `Model`, read clocks, or load output.

- [ ] **Integrate the projection with `RenderProjection`.**

  Export the module from `src/ui/mod.rs`; add a `guild_room: Option<GuildRoomProjection>` field and populate it for `View::Guild`. Retain existing render-authority evidence until the new renderer replaces the old one in Task 5.

- [ ] **Verify deterministic example behavior.**

  ```bash
  cargo test --test guild_room_projection
  cargo test --test render_projection
  ```

- [ ] **Self-review and commit.**

  Confirm mode thresholds are exact, selection changes only presentation, campaign identity is stable, and the projection performs no effects. Commit:

  ```bash
  git add src/ui/guild_room_projection.rs src/ui/mod.rs tests/guild_room_projection.rs tests/render_projection.rs
  git commit -m "feat: project the Questmancer Great Room"
  ```

---

## Task 4: Derive Truthful Stations and prove core invariants

**Files:**

- Modify: `src/ui/guild_room_projection.rs`
- Create: `tests/guild_room_properties.rs`
- Modify: `tests/guild_room_projection.rs`
- Modify: `tests/support/strategies.rs`

- [ ] **Write failing state-mapping examples.**

  Prove these exact mappings: exited -> absent; blocked -> projection at Counsel Bell; done/unseen -> physical at Spoils; idle -> physical at Hearth; working, done/seen, and unknown -> token at owning campaign table. Add focused-agent tests proving lighting changes without relocation.

- [ ] **Write failing property tests.**

  Generate arbitrary safe `Rect`s, workspace/agent mixtures, presence, attention, and selection. Assert:

  - each non-exited adventurer appears exactly once;
  - exited adventurers never appear;
  - every representation uses an allowed station;
  - all projected rectangles are contained by the supplied area;
  - projected landmark/table rectangles do not overlap;
  - identical input produces identical output;
  - selection does not change stable table identity;
  - no arbitrary input panics.

- [ ] **Run 512 cases and confirm meaningful failures.**

  ```bash
  PROPTEST_CASES=512 cargo test --test guild_room_properties
  ```

- [ ] **Implement explicit representation types and mapping.**

  Add:

  ```rust
  pub enum AdventurerRepresentation {
      Physical { agent: AgentKey, station: GuildLandmark },
      Token { agent: AgentKey, table: WorkspaceId },
      Projection { agent: AgentKey, station: GuildLandmark },
  }
  ```

  Derive representations directly from `Presence` plus local `Attention`. Use stable row-major slots for multiple occupants. Do not store a station on `Agent`, in persisted state, or in the reducer.

- [ ] **Make geometry invariants structural.**

  Partition the scene into non-overlapping authored zones before placing tables or station occupants. `ProjectedLandmark` owns shared architectural zones; `ProjectedCampaignTable` owns campaign-table zones, so the same table rectangle is not duplicated in both collections even though `GuildLandmark::CampaignTable` remains a valid semantic station. Occupants are contained by their owning zone and are excluded from the zone-overlap assertion. Clip/suppress detail in undersized zones while preserving landmark identity; never repair overlaps after rendering.

- [ ] **Run the standard and stress property suites.**

  ```bash
  PROPTEST_CASES=512 cargo test --test guild_room_properties
  PROPTEST_CASES=4096 cargo test --test guild_room_properties
  cargo test --test guild_room_projection
  cargo test --test property_domain
  ```

- [ ] **Self-review and commit.**

  Inspect minimized proptest failures, check that arbitrary strategies do not filter excessively, and verify the mapping is exhaustive. Commit:

  ```bash
  git add src/ui/guild_room_projection.rs tests/guild_room_projection.rs tests/guild_room_properties.rs tests/support/strategies.rs
  git commit -m "feat: derive truthful Great Room stations"
  ```

---

## Task 5: Render the wide Great Room as one authored pixel world

**Files:**

- Create: `src/ui/views/great_room.rs`
- Create: `src/ui/widgets/guild_landmark.rs`
- Modify: `src/ui/views/guild_hall.rs`
- Modify: `src/ui/views/mod.rs`
- Modify: `src/ui/widgets/mod.rs`
- Modify: `src/ui/mod.rs`
- Test: `tests/guild_hall_rendering.rs`
- Test: `tests/render_projection.rs`

- [ ] **Add failing wide-scene rendering tests.**

  At widths `120` and `160`, assert the frame exposes all stable landmark labels, campaign banners/tables, selected-table lighting, and one visible representation per projected adventurer. Add an empty-guild fixture that still renders a furnished Hearth and Quest Wall, plus unavailable-Reviewr and failed-output fixtures that do not reserve blank panels.

- [ ] **Add render-authority tests before replacing the renderer.**

  Assert every projected landmark and representation has one production renderer path. Keep projection evidence independent from incidental glyph counts so tests survive art refinement.

- [ ] **Run the new tests and confirm the panel renderer fails them.**

  ```bash
  cargo test --test guild_hall_rendering wide_guild_is_one_great_room -- --exact
  cargo test --test render_projection
  ```

- [ ] **Build small production landmark widgets.**

  Implement Door, Quest Wall, campaign table, Counsel Bell, Hearth, Chronicle Lectern, Scrying Alcove, and Spoils Desk in `guild_landmark.rs`. Widgets render only their projected area/data. Borders represent masonry, beams, furniture, rugs, and shelves—not a grid of application cards.

- [ ] **Build the layered scene renderer.**

  In `great_room.rs`, consume `GuildRoomProjection` and render in this order: room architecture; fixed furniture; campaign banners/tables; tokens/projections/full-body adventurers; semantic effects/selection lighting; readable labels/notices/footer.

  Physical and projected adventurers must reuse `src/ui/persona/chamber_sprite.rs` and `src/ui/pixel::pack`; tokens may be purpose-built small marks. The scene renderer must not infer state a second time.

- [ ] **Reduce `guild_hall.rs` to orchestration.**

  Retain terminal-size guard, footer/modal orchestration, and goblin Easter-egg integration. Delegate the room body to `great_room::render`. Remove the old `render_wide`, `render_medium`, and panel ownership only when their replacement tests are green.

- [ ] **Render typed notices at truthful locations.**

  Connection state/diagnostics go to the Door; output loading/failure to Scrying; integration feedback to the quiet Spoils Desk or footer; persistence diagnostics remain readable without contradicting Door state.

- [ ] **Verify wide, empty, disconnected, and compatibility cases.**

  ```bash
  cargo test --test guild_hall_rendering
  cargo test --test rendering
  cargo test --test render_projection
  cargo test --test persona_art
  ```

- [ ] **Self-review and commit.**

  Inspect TestBackend output at 120x40 and 160x50. Confirm the room reads as one continuous architecture, full-body sprites occur only at physical/projection stations, and Reviewr unavailability produces no empty operational panel. Commit:

  ```bash
  git add src/ui/views/great_room.rs src/ui/widgets/guild_landmark.rs src/ui/views/guild_hall.rs src/ui/views/mod.rs src/ui/widgets/mod.rs src/ui/mod.rs tests/guild_hall_rendering.rs tests/render_projection.rs
  git commit -m "feat: render the Questmancer Great Room"
  ```

---

## Task 6: Add cropped-room and landmark-camera interaction parity

**Files:**

- Modify: `src/app.rs`
- Modify: `src/interaction.rs`
- Modify: `src/runtime_loop.rs`
- Modify: `src/ui/mod.rs`
- Modify: `src/ui/theatre.rs`
- Modify: `src/ui/guild_room_projection.rs`
- Modify: `src/ui/views/great_room.rs`
- Modify: `src/ui/views/guild_hall.rs`
- Modify: `src/storybook/catalogue.rs`
- Test: `tests/app.rs`
- Test: `tests/guild_room_projection.rs`
- Test: `tests/guild_hall_rendering.rs`
- Test: `tests/interaction.rs`
- Test: `tests/input.rs`
- Test: `tests/render_projection.rs`
- Test: `tests/runtime_loop.rs`
- Test: `tests/theatre.rs`

- [ ] **Write failing responsive-camera tests.**

  Medium fixtures must show the selected campaign table, Door, compact Quest Wall, Hearth, integrated Scrying information, and compact markers for other campaigns. Sub-80 fixtures must show one landmark plus a room breadcrumb. Include 80x24 and zero/tiny areas.

- [ ] **Write failing interaction-parity tests.**

  Prove `j/k`, `/`, `enter`, `r`, `space`, `o`, and `v` retain their existing commands; switching Guild/Delve preserves selection; and selection changes request at most one output load. Prove `tab` cycles a deterministic landmark order in narrow mode and preserves existing region focus semantics outside landmark-camera mode.

- [ ] **Replace panel regions with room focus vocabulary.**

  Introduce a `GuildFocus` (or equivalently named) enum in `src/app.rs`:

  ```rust
  pub enum GuildFocus {
      QuestWall,
      CampaignTables,
      CounselBell,
      Hearth,
      Chronicle,
      Scrying,
      Spoils,
      Door,
  }
  ```

  Migrate `Model::region` and `cycle_region` deliberately, including projection evidence, runtime fixtures, Storybook catalogue mappings, and theatre tests. Do not persist terminal geometry; persistence may retain only the semantic focus if current compatibility requires it.

- [ ] **Implement camera derivation in the pure projection.**

  For 80–119 columns, crop around the selected campaign table while preserving required shared landmarks. For less than 80, project the focused landmark and a breadcrumb identifying the one-room route. All modes use the same campaign identities and Truthful Stations.

- [ ] **Render medium and narrow modes from the shared scene vocabulary.**

  Detail may reduce, but labels, connection truth, attention, selected identity, and valid actions remain available. ASCII/ANSI-16 must retain the same landmark names and status words.

- [ ] **Run responsive and command-parity suites.**

  ```bash
  cargo test --test guild_room_projection
  cargo test --test guild_hall_rendering
  cargo test --test interaction
  cargo test --test input
  cargo test --test command
  cargo test --test runtime_loop --test render_projection --test theatre
  ```

- [ ] **Self-review and commit.**

  Verify `tab` never mutates domain state, view switching retains selected agent/campaign, and no interaction duplicates Chronicle entries or output loads. Commit:

  ```bash
  git add src/app.rs src/interaction.rs src/runtime_loop.rs src/ui/mod.rs src/ui/theatre.rs src/ui/guild_room_projection.rs src/ui/views/great_room.rs src/ui/views/guild_hall.rs src/storybook/catalogue.rs tests/app.rs tests/guild_room_projection.rs tests/guild_hall_rendering.rs tests/interaction.rs tests/input.rs tests/render_projection.rs tests/runtime_loop.rs tests/theatre.rs
  git commit -m "feat: add responsive Great Room cameras"
  ```

---

## Task 7: Catalogue every authored room asset and accessibility mode in Storybook

**Files:**

- Modify: `src/storybook/assets.rs`
- Modify: `src/storybook/atlas.rs`
- Modify: `src/storybook/catalogue.rs`
- Modify: `src/storybook/fixtures.rs`
- Modify: `src/storybook/ui.rs`
- Test: `tests/storybook_catalogue.rs`
- Test: `tests/storybook_fixtures.rs`
- Test: `tests/storybook_properties.rs`
- Test: `tests/storybook_rendering.rs`

- [ ] **Write failing exhaustive-ownership tests.**

  Extend the asset catalogue so each stable landmark, token, projection, physical-station pose, room camera mode, and compatibility mode has a fixed story. Assert every authored production asset is owned exactly once by an atlas story; scene stories may reuse assets without claiming ownership.

- [ ] **Create fixed semantic fixtures.**

  Add stable campaigns such as Ironmere, Saltwatch, and Moonfen. Include empty furnished hall, one campaign, several campaigns, mixed Truthful Stations, each connection state, unavailable Reviewr, failed Scrying output, and all responsive modes. Freeze clocks and deterministic persona identities.

- [ ] **Route stories through production renderers.**

  Atlas stories call `guild_landmark`/persona renderers; scene stories call `great_room::render` with production projections. Do not maintain Storybook-only copies of sprites, furniture, or room geometry.

- [ ] **Add compatibility stories.**

  Cover Unicode/truecolor, Unicode/ANSI-16, ASCII/ANSI-16, full motion, reduced motion, and no motion. Assertions must check semantic labels and valid actions, not color alone.

- [ ] **Run all Storybook tests and inspect the dev binary.**

  ```bash
  cargo test --test storybook_catalogue --test storybook_fixtures --test storybook_properties --test storybook_rendering
  just storybook
  ```

  Manually visit every fixed Great Room story and confirm navigation can reach the stories after the atlas; do not expand the previously backlogged scroll redesign in this slice.

- [ ] **Self-review and commit.**

  Confirm every authored asset is shown exactly once in ownership coverage, every scene uses production code, and motion-disabled stories retain semantic state. Commit:

  ```bash
  git add src/storybook/assets.rs src/storybook/atlas.rs src/storybook/catalogue.rs src/storybook/fixtures.rs src/storybook/ui.rs tests/storybook_catalogue.rs tests/storybook_fixtures.rs tests/storybook_properties.rs tests/storybook_rendering.rs
  git commit -m "test: catalogue the Questmancer Great Room"
  ```

---

## Task 8: Ship user documentation, guarded manual verification, and release gate

**Files:**

- Modify: `README.md`
- Modify: `docs/manual-test/questmancer-0.1.0.md`
- Test: `tests/workflow_contract.rb`

- [ ] **Update user-facing product language and operating instructions.**

  Explain the Great Room landmarks, Truthful Stations, one-hall/many-tables model, responsive cameras, and key bindings. Keep literal build/link/open commands first. Describe Storybook as a dev-only asset review tool and do not mention implementation-only types in the primary user path.

- [ ] **Update the guarded Herdr 0.7.4 manual test.**

  Document baseline `git status`, release build, existing-link detection, singleton open, synthetic blocked agent, search/selection/counsel/acknowledge/output refresh, persistence restart, wide and 80x24 screenshots, cleanup of only resources created by the test, and final environment restoration. Explicitly record that Herdr 0.7.4 cannot synthesize `done`; use a real agent or fixture and never claim it was covered otherwise.

- [ ] **Update workflow-contract assertions.**

  Assert the README/manual retain the current install/link/action commands and the new Great Room/Storybook instructions. Avoid brittle assertions over prose unrelated to executable workflow.

- [ ] **Run the complete automated release gate.**

  ```bash
  cargo fmt --all --check
  cargo clippy --all-targets --all-features -- -D warnings
  cargo test --all-targets --all-features
  bash tests/scripts.sh
  bash -n herdr/install.sh herdr/run.sh herdr/control.sh
  PROPTEST_CASES=4096 cargo test --test property_domain --test persisted_state --test guild_room_properties
  cargo build --release
  test -x target/release/questmancer
  ruby tests/workflow_contract.rb
  git diff --check
  ```

- [ ] **Perform the guarded Herdr manual pass.**

  Follow `docs/manual-test/questmancer-0.1.0.md` exactly. Do not stop a pre-existing Herdr server, unlink a pre-existing plugin, operate on another real agent, or leave test-created panes/reports behind. Capture wide and 80x24 Great Room screenshots and record each item as pass, fail, or blocked.

- [ ] **Audit the accepted design against production evidence.**

  Check every acceptance criterion in the design spec. In particular: one continuous room; all wide landmarks; multiple campaign tables; exactly one truthful representation per non-exited agent; responsive room identity; command parity; non-contradictory connection theatre; cozy empty/disconnected states; both regressions; exhaustive Storybook coverage.

- [ ] **Self-review documentation and working tree.**

  Confirm docs describe behavior that actually passed, environment failures are reported separately from product defects, and only intended files are staged. Preserve the unrelated `PLAN.md` modification.

- [ ] **Commit the documentation and workflow contract.**

  ```bash
  git add README.md docs/manual-test/questmancer-0.1.0.md tests/workflow_contract.rb
  git commit -m "docs: document the Questmancer Great Room"
  ```

## Final acceptance checklist

- [x] The Guild Hall reads as one inhabited Great Room rather than bordered panels.
- [x] Wide mode renders Door, Quest Wall, all campaign tables, Counsel Bell, Hearth, Chronicle, Scrying, and Spoils.
- [x] Every non-exited adventurer has exactly one approved representation; exited adventurers have none.
- [x] Full-body sprites occur only at Counsel, Hearth, and Spoils truth stations.
- [x] Wide, medium, and narrow cameras retain one stable room and campaign identity.
- [x] Existing observe, counsel, acknowledge, refresh, search, and Reviewr actions retain command parity.
- [x] Connected theatre cannot retain a connecting diagnostic.
- [x] The Delve route-home row cannot render residual `HOMET PATH` text.
- [x] Empty, disconnected, incompatible, and integration-unavailable states remain readable and furnished.
- [x] Storybook inventories every authored production room asset once and exposes all fixed scenes.
- [x] Standard and 4096-case property suites pass without excessive rejection.
- [x] Format, Clippy, all-feature tests, script tests, property stress, release build, and workflow contract pass.
- [ ] Guarded live candidate open, interactions, persistence, and screenshots pass.
- [ ] Real `done` transition is observed in live post-merge acceptance.
- [ ] Manual-test resources are cleaned up and the original Herdr environment is restored.
