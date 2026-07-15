# Defect-First Connected Café Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the managed-webmaster-pane defects and replace the current line-based café with deterministic connected workspace bays, then verify the result against Herdr 0.7.4.

**Architecture:** Keep the existing pure domain reducer and Ratatui renderer. Carry the managed pane identity from `HERDR_PANE_ID` into `Model`, exclude it in snapshot normalization and event/command boundaries, and introduce a small pure café scene model that maps workspace identity to stable bay variants and anchored seats.

**Tech Stack:** Rust stable, Tokio, Ratatui 0.30, Crossterm, Serde/JSON, BLAKE3, TestBackend, proptest.

## Global Constraints

- Fix defects before adding Herdr 0.7.4 sidebar integrations.
- The managed pane must never enter domain state or receive output, focus, reply, attention, or guestbook operations.
- Workspace maps to one café bay; agents in that workspace share the bay.
- Room variants are deterministic from workspace identity and are not configurable in this slice.
- Use original Unicode/ASCII art; do not copy the supplied IT Crowd reference.
- A workstation must be anchored to a room object; arbitrary decorative lines are not valid scene geometry.
- Preserve 80x24 actionability, ASCII mode, reduced/no-motion modes, zero-size safety, and low idle cost.
- Keep the existing protocol floor and verify Herdr 0.7.4 only after the current-version suite passes.
- Every task ends with focused tests and a small commit.

---

## File map

- Modify `src/app.rs`: store the optional managed pane identity on `Model` and expose a read-only accessor/setter.
- Modify `src/terminal.rs`: read `HERDR_PANE_ID` once and configure the model before live bootstrap.
- Modify `src/domain/state.rs`: add exclusion-aware snapshot normalization while retaining the existing test-friendly wrapper.
- Modify `src/update/reducer.rs`: pass the model’s exclusion into snapshot replacement and preserve it across reconnects.
- Modify `src/herdr/event_adapter.rs`: ignore status/exit events for the managed pane before they request snapshots or mutate state.
- Modify `src/command.rs` and `src/interaction.rs`: reject managed-pane focus, reply, output, and plugin-action commands at the effect boundary.
- Add/modify `tests/normalization.rs`, `tests/reducer.rs`, `tests/event_adapter.rs`, `tests/command.rs`, `tests/interaction.rs`, and `tests/runtime_loop.rs` for the self-pane invariants.
- Modify `docs/superpowers/specs/2026-07-15-user-quick-start-manual-test-design.md` and `README.md` with the dedicated plain-pane manual test path.
- Add `src/ui/cafe_scene.rs`: pure bay/variant/seat geometry and deterministic scene data.
- Modify `src/ui/views/cafe.rs`: render connected bays and route compact fallbacks through scene geometry.
- Modify `src/ui/widgets/agent_crt.rs` and `src/ui/widgets/profile_card.rs`: render workstation furniture and selected details as scene elements rather than independent dashboard cards.
- Modify `src/ui/pixel/palette.rs` and `src/ui/theme.rs` only where the authored room palette needs semantic wall/floor/furniture roles.
- Add `tests/cafe_scene.rs` and update `tests/cafe_rendering.rs`, `tests/cafe_widgets.rs`, `tests/persona_art.rs`, and `tests/property_domain.rs`.
- Modify `PLAN.md` and `README.md` with the connected-bay behavior and verification commands.

---

### Task 1: Make the managed pane a first-class runtime exclusion

**Files:**
- Modify: `src/app.rs`
- Modify: `src/terminal.rs`
- Test: `tests/app.rs`, `tests/runtime.rs`

**Interfaces:**
- Produce `Model::managed_pane_id(&self) -> Option<&PaneId>`.
- Produce `Model::set_managed_pane_id(&mut self, pane_id: Option<PaneId>)`.
- `terminal::run` reads `HERDR_PANE_ID` before `bootstrap_model` and sets it on the model. Missing environment remains valid for offline mode.

- [ ] **Step 1: Write the failing model tests.** Add a test that a new model has no managed pane, and a test that `set_managed_pane_id(Some(PaneId::new("w2:p3")))` round-trips through the accessor.

- [ ] **Step 2: Run the focused tests.**

  ```bash
  cargo test --test app managed_pane -- --nocapture
  ```

  Expected: FAIL because the model field and methods do not exist.

- [ ] **Step 3: Add the model field and environment wiring.** Store `managed_pane_id: Option<PaneId>` in `Model`, initialize it to `None`, and in `terminal::run` use:

  ```rust
  let managed_pane_id = std::env::var("HERDR_PANE_ID")
      .ok()
      .filter(|value| !value.is_empty())
      .map(PaneId::new);
  model.set_managed_pane_id(managed_pane_id);
  ```

  Keep the existing runtime registration call unchanged; registration and domain exclusion have separate responsibilities.

- [ ] **Step 4: Run the focused tests and the existing runtime suite.**

  ```bash
  cargo test --test app --test runtime -- --nocapture
  ```

  Expected: PASS.

- [ ] **Step 5: Commit.**

  ```bash
  git add src/app.rs src/terminal.rs tests/app.rs tests/runtime.rs
  git commit -m "fix: carry managed pane identity into model"
  ```

### Task 2: Exclude the managed pane from snapshots, events, and effects

**Files:**
- Modify: `src/domain/state.rs`
- Modify: `src/update/reducer.rs`
- Modify: `src/herdr/event_adapter.rs`
- Modify: `src/runtime_loop.rs`
- Test: `tests/normalization.rs`, `tests/reducer.rs`, `tests/event_adapter.rs`, `tests/runtime_loop.rs`

**Interfaces:**
- Add `DomainState::from_snapshot_excluding(snapshot: &SessionSnapshot, observed_at: Timestamp, excluded_pane: Option<&PaneId>) -> Self`.
- Keep `DomainState::from_snapshot(snapshot, observed_at)` as a wrapper that calls the new method with `None`, so existing fixture/unit callers remain stable.
- Change `AppEvent::SnapshotReplaced` to carry `excluded_pane: Option<PaneId>`.
- `apply_connection_update` and `apply_command_result` pass `model.managed_pane_id().cloned()` when constructing snapshot events.

- [ ] **Step 1: Write failing normalization tests.** Add a fixture mutation that appends an `AgentInfo` with pane `w2:p3`; assert `from_snapshot_excluding(..., Some(&PaneId::new("w2:p3")))` has no such agent, no site agent key for that pane, and never selects it.

- [ ] **Step 2: Run the normalization test to verify failure.**

  ```bash
  cargo test --test normalization managed_pane -- --nocapture
  ```

  Expected: FAIL because all snapshot agents are currently inserted.

- [ ] **Step 3: Implement exclusion at the domain boundary.** Change the snapshot loop to skip sources whose `pane_id` matches `excluded_pane`. Do not filter by display name, workspace, or agent status.

- [ ] **Step 4: Add reducer reconnect coverage.** A `SnapshotReplaced` containing the managed pane must replace runtime state without reintroducing it and must preserve the previous selected real agent where valid.

- [ ] **Step 5: Add adapter coverage.** Status and exit events whose pane id equals the managed pane must produce no `AppEvent`, no guestbook entry, and no refresh request. Unknown non-managed panes retain the existing refresh behavior.

- [ ] **Step 6: Thread the exclusion through runtime loop snapshot paths.** Both supervisor-connected snapshots and explicit `SnapshotLoaded` results must include the model’s managed pane id.

- [ ] **Step 7: Run focused and property tests.**

  ```bash
  cargo test --test normalization --test reducer --test event_adapter --test runtime_loop -- --nocapture
  cargo test --test property_domain -- --nocapture
  ```

  Expected: PASS.

- [ ] **Step 8: Commit.**

  ```bash
  git add src/domain/state.rs src/update/reducer.rs src/herdr/event_adapter.rs src/runtime_loop.rs tests/normalization.rs tests/reducer.rs tests/event_adapter.rs tests/runtime_loop.rs
  git commit -m "fix: exclude webmaster pane from domain state"
  ```

### Task 3: Guard command effects and repair the manual test guide

**Files:**
- Modify: `src/command.rs`
- Modify: `src/interaction.rs`
- Modify: `tests/command.rs`, `tests/interaction.rs`
- Modify: `README.md`, `docs/superpowers/specs/2026-07-15-user-quick-start-manual-test-design.md`

**Interfaces:**
- Every command executor receives the managed pane id or a `CommandPolicy` containing it.
- A managed-pane target returns a user-visible non-fatal failure and never calls the Herdr client.
- The manual guide uses a dedicated plain pane for synthetic reporting.

- [ ] **Step 1: Write failing command tests.** Add fake-client tests proving `FocusPane`, `SendReply`, `LoadOutput`, and optional Reviewr invocation are not sent when the target pane equals the managed pane.

- [ ] **Step 2: Run the command and interaction tests.**

  ```bash
  cargo test --test command --test interaction managed_pane -- --nocapture
  ```

  Expected: FAIL because command paths currently accept any selected agent pane.

- [ ] **Step 3: Implement one guard at the command boundary.** Use a shared predicate:

  ```rust
  fn is_managed_pane(&self, pane_id: &PaneId) -> bool {
      self.managed_pane_id.as_ref() == Some(pane_id)
  }
  ```

  Return a `CommandResult::Failed` with a stable message such as `refused operation on webmaster pane` and do not invoke the socket client.

- [ ] **Step 4: Document a dedicated manual pane.** Replace the previous instruction that reports over an existing Codex pane with: create/select an unowned plain pane, capture its pane id, report `working`/`blocked`/`idle`, verify snapshot/UI presence, then restore and release the synthetic source.

- [ ] **Step 5: Run tests and inspect the documentation diff.**

  ```bash
  cargo test --test command --test interaction -- --nocapture
  rg -n "dedicated|plain pane|report-agent|webmaster-smoke" README.md docs/superpowers/specs/2026-07-15-user-quick-start-manual-test-design.md
  ```

- [ ] **Step 6: Commit.**

  ```bash
  git add src/command.rs src/interaction.rs tests/command.rs tests/interaction.rs README.md docs/superpowers/specs/2026-07-15-user-quick-start-manual-test-design.md
  git commit -m "fix: guard managed pane effects and manual test path"
  ```

### Task 4: Add pure connected-bay scene geometry

**Files:**
- Create: `src/ui/cafe_scene.rs`
- Modify: `src/ui/mod.rs`
- Test: `tests/cafe_scene.rs`

**Interfaces:**
- Define:

  ```rust
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub enum BayVariant { WallRow, CornerBooth, BackRoomLab }

  #[derive(Clone, Debug, Eq, PartialEq)]
  pub struct CafeBay { pub workspace_id: WorkspaceId, pub variant: BayVariant, pub seats: Vec<SeatAnchor> }

  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub struct SeatAnchor { pub x: u16, pub y: u16, pub width: u16, pub height: u16 }

  pub fn variant_for_workspace(workspace_id: &WorkspaceId) -> BayVariant;
  pub fn layout_bays(sites: &BTreeMap<WorkspaceId, Site>, agents: &BTreeMap<AgentKey, Agent>, area: Rect, selected: Option<&WorkspaceId>) -> Vec<CafeBay>;
  ```

- [ ] **Step 1: Write failing deterministic geometry tests.** Prove the same workspace id always returns the same variant, all three variants are reachable over a table of ids, bay ordering is sorted by `WorkspaceId`, and every generated seat is inside the bay rectangle.

- [ ] **Step 2: Run the new test target.**

  ```bash
  cargo test --test cafe_scene -- --nocapture
  ```

  Expected: FAIL because the module and functions do not exist.

- [ ] **Step 3: Implement the pure scene model.** Use BLAKE3 over `b"cafe-variant\0" + workspace_id.as_str()` and map the first digest byte to the three variants. Define authored seat anchors for each variant; no random or frame-time values.

- [ ] **Step 4: Add proptest invariants.** Generate workspace ids and agent counts; assert variant stability, no overlapping seat rectangles within a bay, and no seat rectangle exceeds the supplied bay area.

- [ ] **Step 5: Run focused tests.**

  ```bash
  cargo test --test cafe_scene --test property_domain -- --nocapture
  ```

- [ ] **Step 6: Commit.**

  ```bash
  git add src/ui/cafe_scene.rs src/ui/mod.rs tests/cafe_scene.rs
  git commit -m "feat: add deterministic connected cafe scene geometry"
  ```

### Task 5: Render authored bays and anchored workstations

**Files:**
- Modify: `src/ui/views/cafe.rs`
- Modify: `src/ui/widgets/agent_crt.rs`
- Modify: `src/ui/widgets/profile_card.rs`
- Modify: `src/ui/pixel/palette.rs`, `src/ui/theme.rs` as needed
- Test: `tests/cafe_rendering.rs`, `tests/cafe_widgets.rs`, `tests/persona_art.rs`

**Interfaces:**
- `cafe::render` consumes `layout_bays` and paints room layers in order: architecture, furniture, seats/personas, state theatre, selection, then compact controls.
- `render_workstation` accepts a `SeatAnchor` and draws the desk/CRT/chair relative to it; it does not create a full dashboard border for every station.
- `render_profile_card` becomes a compact inspector for the selected agent and no longer competes with the active bay’s room geometry.

- [ ] **Step 1: Write failing rendering assertions.** Replace tests that require literal shared labels (`CABLE RUN`, `FLOOR / CABLE RUN / COUNTER`) with assertions for authored geometry: bay signage, doorway/aisle, desk/CRT/furniture marks, every agent name, and selected-bay focus. Add a test that two workspaces produce two connected bay cues.

- [ ] **Step 2: Run café tests to verify the old renderer fails the new contract.**

  ```bash
  cargo test --test cafe_rendering --test cafe_widgets -- --nocapture
  ```

  Expected: FAIL on missing bay/variant/furniture assertions.

- [ ] **Step 3: Implement architecture layers.** Replace `paint_room` and the card-first grid with deterministic room painting. Use a shared `RoomCanvas`/`Rect` per bay and draw wall, signage, doorway, floor, counter, and furniture before agents. Keep all drawing clipped to the supplied Ratatui area.

- [ ] **Step 4: Anchor sprites to furniture.** Render each seated persona at its `SeatAnchor`; state theatre changes posture, CRT content, lamp, and one-shot effects without moving the seat or changing identity.

- [ ] **Step 5: Implement active-bay emphasis and compact fallback.** At wide sizes, show the selected bay prominently with neighboring bays simplified. At 80x24, show one active bay plus a navigable bay strip. Below the compact threshold, preserve the existing actionable list but use the authored workstation iconography.

- [ ] **Step 6: Run rendering, persona, and safety tests.**

  ```bash
  cargo test --test cafe_rendering --test cafe_widgets --test persona_art --test rendering -- --nocapture
  ```

- [ ] **Step 7: Commit.**

  ```bash
  git add src/ui/views/cafe.rs src/ui/widgets/agent_crt.rs src/ui/widgets/profile_card.rs src/ui/pixel/palette.rs src/ui/theme.rs tests/cafe_rendering.rs tests/cafe_widgets.rs tests/persona_art.rs
  git commit -m "feat: render connected authored cafe bays"
  ```

### Task 6: Prove responsive accessibility and animation invariants

**Files:**
- Modify: `tests/cafe_rendering.rs`, `tests/cafe_widgets.rs`, `tests/property_domain.rs`, `tests/theatre.rs`
- Modify: `README.md`, `PLAN.md`

- [ ] **Step 1: Add golden coverage.** Render empty, one-bay, two-bay, mixed-state, selected, disconnected, ASCII, reduced-motion, 80x24, 60x18, and zero-sized surfaces with `TestBackend`. Assert no panic, all required state labels, and no nested TUI output.

- [ ] **Step 2: Add animation boundaries.** Keep the existing injected clock and assert done confetti ends exactly once, no-motion performs no periodic invalidation, and unchanged idle rooms do not request output reads or persistence.

- [ ] **Step 3: Add property invariants.** Generate arbitrary workspace/agent sets and assert every visible agent belongs to exactly one bay, every selected key remains selectable, and the managed pane is absent from every rendered surface.

- [ ] **Step 4: Run the full current-version quality gate.**

  ```bash
  cargo fmt --check
  cargo clippy --all-targets --all-features -- -D warnings
  cargo test --all-targets --all-features
  ```

  Expected: PASS with no tracked-file changes from the test run.

- [ ] **Step 5: Update user docs.** Document the connected-bay model, deterministic variants, keyboard navigation, compact fallback, and the dedicated plain-pane manual test.

- [ ] **Step 6: Commit.**

  ```bash
  git add tests/cafe_rendering.rs tests/cafe_widgets.rs tests/property_domain.rs tests/theatre.rs README.md PLAN.md
  git commit -m "test: harden connected cafe invariants and docs"
  ```

### Task 7: Verify Herdr 0.7.4 and plan sidebar integration

**Files:**
- Modify: `README.md` only if the verified command path changes
- Create: `docs/superpowers/plans/2026-07-15-herdr-074-sidebar-integration.md` after compatibility evidence is collected
- Test/evidence: `docs/manual-tests/` if that directory already exists; otherwise record results in the plan, not source code

- [ ] **Step 1: Confirm the running Herdr version and protocol.**

  ```bash
  herdr --version
  herdr api ping
  ```

  Expected: Herdr `0.7.4`, protocol `16`, and a compatible response.

- [ ] **Step 2: Build and link the current plugin without changing its minimum version.**

  ```bash
  cargo build --release
  herdr plugin link .
  herdr plugin list
  ```

  Expected: `opsydyn.webmaster` remains enabled and the existing five actions remain available.

- [ ] **Step 3: Run the live smoke path.** Open exactly one webmaster pane, verify desk/café switching, verify the managed pane is absent from both views, create a dedicated plain test pane, report a supported blocked state, confirm inbox/HELP rendering, and clean up the test source/pane.

- [ ] **Step 4: Inspect plugin logs and restore the environment.** Confirm successful action logs, close only test-created panes, leave the server and pre-existing link running, and finish with:

  ```bash
  git status --short --branch
  ```

- [ ] **Step 5: Record sidebar opportunities without implementing them.** Capture the smallest follow-up: ambient workspace/agent metadata rows and an optional popup preview. Do not add 0.7.4 sidebar behavior until the defect-first live smoke passes.

- [ ] **Step 6: Commit only documentation/evidence if needed.**

  ```bash
  git add README.md docs/superpowers/plans/2026-07-15-herdr-074-sidebar-integration.md
  git commit -m "docs: record Herdr 0.7.4 verification and sidebar follow-up"
  ```

## Self-review checklist

- [ ] Managed pane exclusion is covered at startup, reconnect, event, and command boundaries.
- [ ] Manual test guidance no longer reports over a real agent-owned pane.
- [ ] Café geometry has explicit architecture/furniture/seat layers and no arbitrary room labels.
- [ ] Workspace-to-bay mapping and variant selection are deterministic.
- [ ] Selected agent, blocked state, done transition, exited state, ASCII, reduced motion, and compact layouts remain actionable.
- [ ] Proptest and TestBackend coverage include the new invariants.
- [ ] Herdr 0.7.4 verification is last and does not silently expand the implementation scope.
