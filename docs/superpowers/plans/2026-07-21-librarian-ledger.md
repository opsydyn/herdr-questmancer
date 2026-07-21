# Questmancer Librarian and Ledger Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a persistent, clickable orangutan Librarian to the canonical and compact Guild Hall and replace generic help with one transient, four-page Librarian's Ledger.

**Architecture:** Keep the Librarian entirely in the presentation layer. The Guild Hall renderer paints an authored RGB world sprite and publishes a typed non-agent hit region; `?` and pointer activation reduce into the same transient Ledger modal. The Ledger renders fixed typed pages with an optional Ratatui-image illustration and a guaranteed authored RGB fallback. No Herdr topology, agent, command, Chronicle or persistence type gains a Librarian identity.

**Tech Stack:** Rust 1.90, Ratatui 0.30, ratatui-image, the existing deterministic RGB scene engine, Crossterm input, TestBackend rendering tests, and the feature-gated Storybook.

## Global Constraints

- Implement the approved contract in `docs/superpowers/specs/2026-07-21-librarian-ledger-design.md`; current source and tests win over historical plans.
- Work inline on `main`. Do not create a worktree and do not delegate unless the user explicitly changes the collaboration preference.
- Before Task 1, resolve the pre-existing changes in `src/scene/render/guild_hall.rs`, `tests/scene_guild_hall.rs`, `tests/scene_stage.rs`, `tests/scene_stage_properties.rs`, `tests/workflow_contract.rb`, and `docs/manual-test/questmancer-scene-preview.md` into their own coherent commit. Do not mix those prior responsive-layout edits into a Librarian commit. Preserve untracked `AGENTS.md` and `src/assets/librarian.png`; the latter is an input to this feature.
- Use test-first red-green-refactor for every behaviour task. Show the focused failing test before production edits.
- Do not create an `Agent`, `AgentKey`, `AdventurerPersona`, campaign member, Chronicle entry, persisted record, or Herdr command for the Librarian.
- Do not add a second production renderer. World and fallback art use the RGB scene engine; the PNG is card-only.
- A complete Librarian sprite must be visible before its hit region is returned. The Delve, Guild vignette and status-only layouts return no Librarian hit region.
- Opening, paging and closing the Ledger must emit no `AgentCommand` and no `Command::PersistState`.
- Keep handbook copy fixed and typed. Do not add scanners, dynamic providers, background tasks, AI calls or a generic help/plugin framework in this slice.
- Each task ends with its focused verification and one coherent commit. Do not stage unrelated dirty files.

---

## File Responsibility Map

| Path | Responsibility in this slice |
|---|---|
| `src/scene/assets/librarian.rs` | Authored 16x24 world master and 24x32 RGB Ledger fallback. |
| `src/scene/assets/mod.rs` | Export the Librarian asset module. |
| `src/assets/librarian.png` | Embedded native Ledger illustration; never used as a world sprite. |
| `src/portrait.rs` | Prepare and expose the native Librarian illustration independently of adventurer personas. |
| `src/scene/mod.rs` | Typed non-agent interactable regions and pointer target resolution. |
| `src/scene/render/guild_hall.rs` | Deterministic canonical/compact placement, painting and hit-region publication. |
| `src/scene/render/delve.rs` | Explicitly return no non-agent interactables. |
| `src/app.rs` | Transient `LibrarianLedger` modal and fixed-page navigation methods. |
| `src/ledger.rs` | Stable page IDs and the four fixed handbook pages. |
| `src/scene/presentation.rs` | Project the Ledger modal into the sole scene overlay. |
| `src/ui/input.rs` | Map `?`, paging keys and pointer targets without leaking actions behind the modal. |
| `src/interaction.rs` | One reducer path for keyboard and clicked Librarian activation. |
| `src/ui/scene_overlays.rs` | Responsive Ledger parchment and native/fallback illustration rendering. |
| `src/storybook/{assets,catalogue,fixtures,ui}.rs` | Own and render the new production assets and Ledger states. |
| `tests/librarian_assets.rs` | Asset dimensions, non-empty silhouettes and embedded PNG contract. |
| `tests/{scene_guild_hall,scene_interaction,input,interaction,scene_overlays,scene_stage_properties}.rs` | Presence, hit testing, modal behaviour, command/persistence isolation and responsive rendering. |
| `tests/storybook.rs` | Exact Storybook ownership and production story coverage. |
| `README.md`, `AGENTS.md`, `docs/manual-test/questmancer-scene-preview.md`, `tests/workflow_contract.rb` | User controls, architecture truth, manual acceptance and documentation contract. |

---

## Task 0: Make the Starting State Safe

**Files:** No edits.

- [ ] Run the baseline exactly:

  ```bash
  git status --short --branch
  git log -5 --oneline
  git diff -- src/scene/render/guild_hall.rs tests/scene_guild_hall.rs \
    tests/scene_stage.rs tests/scene_stage_properties.rs \
    tests/workflow_contract.rb docs/manual-test/questmancer-scene-preview.md
  ```

- [ ] Stop if the pre-existing responsive-layout slice is still uncommitted. Complete and commit that slice separately, or obtain explicit user approval for another disposition. Never stash, reset, restore, clean or silently absorb it.

- [ ] Confirm `src/assets/librarian.png` is a readable PNG and record its dimensions without rewriting it:

  ```bash
  file src/assets/librarian.png
  git status --short --branch
  ```

- [ ] Run the pre-feature focused baseline:

  ```bash
  just guild-test
  just storybook-test
  cargo test --test interaction --test input --test scene_overlays
  ```

  Expected: all pass. A baseline failure is investigated before Librarian work begins.

---

## Task 1: Author the Librarian Art Contract

**Files:**
- Create: `src/scene/assets/librarian.rs`
- Modify: `src/scene/assets/mod.rs`
- Create: `tests/librarian_assets.rs`

- [ ] Add failing asset tests first. They should express only durable production contracts:

  ```rust
  use questmancer::scene::assets::librarian::{ledger_portrait, world};

  #[test]
  fn librarian_world_master_is_native_scale_and_non_empty() {
      let sprite = world();
      assert_eq!((sprite.size().width, sprite.size().height), (16, 24));
      assert!(sprite.pixels().iter().any(Option::is_some));
  }

  #[test]
  fn librarian_ledger_fallback_fills_a_readable_portrait_canvas() {
      let sprite = ledger_portrait();
      assert_eq!((sprite.size().width, sprite.size().height), (24, 32));
      let occupied = sprite.pixels().iter().filter(|pixel| pixel.is_some()).count();
      assert!(occupied >= 120, "fallback silhouette is too sparse: {occupied}");
  }
  ```

- [ ] Run `cargo test --test librarian_assets` and confirm it fails because the module does not exist.

- [ ] Implement `src/scene/assets/librarian.rs` with the same indexed-sprite boundary as the existing archetypes:

  ```rust
  use std::sync::OnceLock;

  use super::{IndexedPaletteEntry, indexed_sprite};
  use crate::scene::{assets::palette, pixel::Rgb, sprite::SpriteFrame};

  pub const WORLD_WIDTH: u16 = 16;
  pub const WORLD_HEIGHT: u16 = 24;
  pub const PORTRAIT_WIDTH: u16 = 24;
  pub const PORTRAIT_HEIGHT: u16 = 32;

  #[must_use]
  pub fn world() -> &'static SpriteFrame {
      static FRAME: OnceLock<SpriteFrame> = OnceLock::new();
      FRAME.get_or_init(|| indexed_sprite(WORLD_ROWS, PALETTE).expect("built-in Librarian world art"))
  }

  #[must_use]
  pub fn ledger_portrait() -> &'static SpriteFrame {
      static FRAME: OnceLock<SpriteFrame> = OnceLock::new();
      FRAME.get_or_init(|| indexed_sprite(PORTRAIT_ROWS, PALETTE).expect("built-in Librarian portrait"))
  }
  ```

  Author complete fixed-width `WORLD_ROWS` and `PORTRAIT_ROWS` in this file. Use a limited warm-brown, parchment, plum and brass palette; make the orangutan face, long arms, purple/gold keeper's robe, spectacles and book/key silhouette readable. Do not derive the rows from the PNG at runtime and do not use random generation.

- [ ] Export it with `pub mod librarian;` in `src/scene/assets/mod.rs`.

- [ ] Add test assertions that every row is accepted by `indexed_sprite` and that the top-left transparent pixel remains transparent. These protect against ragged editing and an accidental opaque rectangle.

- [ ] Run:

  ```bash
  cargo test --test librarian_assets
  cargo fmt --check
  git diff --check
  ```

- [ ] Commit only the RGB asset module and its tests:

  ```bash
  git add src/scene/assets/librarian.rs src/scene/assets/mod.rs tests/librarian_assets.rs
  git commit -m "feat: author Librarian scene assets"
  ```

---

## Task 2: Prepare a Native Ledger Illustration Without a Fake Persona

**Files:**
- Modify: `src/portrait.rs`
- Add: `src/assets/librarian.png`
- Modify: `tests/librarian_assets.rs`

- [ ] Extend the asset test first:

  ```rust
  use questmancer::portrait::librarian_asset;

  #[test]
  fn embedded_librarian_art_is_a_decodable_png() {
      let image = image::load_from_memory_with_format(
          librarian_asset(),
          image::ImageFormat::Png,
      ).expect("embedded Librarian PNG decodes");
      assert!(image.width() >= 256);
      assert!(image.height() >= 256);
  }
  ```

- [ ] Run `cargo test --test librarian_assets embedded_librarian_art_is_a_decodable_png` and confirm the missing API failure.

- [ ] Add an explicit prepared illustration field to `PortraitGallery`; do not add `PortraitKey::Librarian` because that key space is intentionally ancestry/class based:

  ```rust
  pub struct PortraitGallery {
      capability: PortraitCapability,
      portraits: BTreeMap<PortraitKey, Protocol>,
      librarian: Option<Protocol>,
      diagnostic: Option<String>,
  }

  impl PortraitGallery {
      #[must_use]
      pub fn librarian(&self) -> Option<&Protocol> {
          self.librarian.as_ref()
      }
  }

  #[must_use]
  pub const fn librarian_asset() -> &'static [u8] {
      include_bytes!("assets/librarian.png")
  }
  ```

- [ ] In `from_picker`, prepare `librarian_asset()` with the existing `prepare_portrait` function when the protocol is native. Preserve partial success: a failed Librarian decode records `format!("Librarian: {error}")` in the diagnostic but does not discard valid adventurer portraits; a failed adventurer portrait does not discard the Librarian. `fallback` and Halfblocks set `librarian: None`.

- [ ] Extend the private tests in `src/portrait.rs`:
  - Halfblocks returns `None` for `gallery.librarian()`.
  - Kitty prepares `gallery.librarian().is_some()` alongside all approved adventurer portraits.
  - `Debug` reports whether the Librarian illustration exists without trying to format `Protocol`.
  - Invalid PNG preparation still cannot displace the RGB fallback.

- [ ] Run:

  ```bash
  cargo test portrait::tests
  cargo test --test librarian_assets
  cargo clippy --all-targets --all-features -- -D warnings
  ```

- [ ] Commit the supplied PNG and gallery support:

  ```bash
  git add src/assets/librarian.png src/portrait.rs tests/librarian_assets.rs
  git commit -m "feat: prepare native Librarian illustration"
  ```

---

## Task 3: Add Typed Non-Agent Scene Hit Regions

**Files:**
- Modify: `src/scene/mod.rs`
- Modify: `src/scene/render/delve.rs`
- Modify: `tests/input.rs`
- Modify: `tests/interaction.rs`
- Modify: `tests/scene_overlays.rs`

- [ ] Add failing unit tests in `tests/input.rs` for a frame containing one actor and one Librarian region. Test both terminal half-block rows of each RGB rectangle and verify empty room remains empty.

- [ ] Add the typed scene contract:

  ```rust
  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub enum SceneInteractable {
      Librarian,
  }

  #[derive(Clone, Debug, Eq, PartialEq)]
  pub struct SceneInteractableRegion {
      pub kind: SceneInteractable,
      pub bounds: PixelRect,
  }

  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub enum SceneTarget<'a> {
      Agent(&'a AgentKey),
      Interactable(SceneInteractable),
  }

  pub struct SceneFrame {
      pub world: WorldScene,
      pub next_frame_in: Option<Duration>,
      pub actors: Vec<SceneActorRegion>,
      pub interactables: Vec<SceneInteractableRegion>,
  }
  ```

- [ ] Implement `SceneFrame::target_at(column, row)` with the existing two-RGB-rows-per-terminal-cell rule. Resolve actors first, then interactables. Keep `agent_at` as a compatibility wrapper over `target_at`; add `interactable_at` as a typed wrapper. The Guild Hall renderer will later prove that regions never overlap.

- [ ] Update every current `SceneFrame` literal returned by the Delve and constructed in tests to include `interactables: Vec::new()`. Use this inventory to prevent a missed constructor:

  ```bash
  rg -n 'SceneFrame \{' src tests
  ```

  Expected production locations include `src/scene/render/delve.rs`, `src/scene/render/guild_hall.rs`, and the literals in `tests/input.rs`, `tests/interaction.rs`, and `tests/scene_overlays.rs`.

- [ ] Run:

  ```bash
  cargo test --test input --test interaction --test scene_overlays
  just delve-test
  ```

- [ ] Commit:

  ```bash
  git add src/scene/mod.rs src/scene/render/delve.rs tests/input.rs tests/interaction.rs tests/scene_overlays.rs
  git commit -m "feat: type scene interactable regions"
  ```

---

## Task 4: Place the Persistent Librarian in Responsive Guild Halls

**Files:**
- Modify: `src/scene/render/guild_hall.rs`
- Modify: `tests/scene_guild_hall.rs`
- Modify: `tests/scene_interaction.rs`
- Modify: `tests/scene_stage_properties.rs`

- [ ] Add failing renderer tests first:
  - canonical `160x90` contains exactly one `SceneInteractable::Librarian`;
  - compact `80x48` with a party that fits after reservation contains exactly one;
  - Delve, Guild vignette and status-only contain none;
  - the published rectangle is entirely inside the viewport;
  - it does not intersect any `SceneActorRegion`;
  - identical snapshot, viewport and time produce identical pixels and region bounds;
  - clicking the centre of the bounds resolves `SceneTarget::Interactable(Librarian)`.

- [ ] Run the focused tests and confirm they fail because Guild Hall returns no interactables:

  ```bash
  cargo test --test scene_guild_hall librarian
  cargo test --test scene_interaction librarian
  ```

- [ ] Add these renderer helpers in `src/scene/render/guild_hall.rs`:

  ```rust
  const LIBRARIAN_CANONICAL_ORIGIN: PixelPoint = PixelPoint::new(7, 64);

  fn paint_librarian(
      target: &mut RgbBuffer,
      origin: PixelPoint,
  ) -> Option<SceneInteractableRegion> {
      let sprite = librarian::world();
      let bounds = PixelRect::new(origin.x, origin.y, sprite.size().width, sprite.size().height);
      if !is_visible(PixelPoint::new(0, 0), bounds, target.size())
          || bounds.x < 0
          || bounds.y < 0
          || bounds.x + i32::from(bounds.width) > i32::from(target.size().width)
          || bounds.y + i32::from(bounds.height) > i32::from(target.size().height)
      {
          return None;
      }
      blit(sprite, origin, target);
      Some(SceneInteractableRegion { kind: SceneInteractable::Librarian, bounds })
  }
  ```

  The canonical station is the lower-left reading corner beside the low shelf and banner. Paint it after static furnishings and connection lighting but before transient interaction overlays; publish the translated final bounds.

- [ ] Reserve one complete 16x24 compact grid cell for the Librarian:
  - define `compact_party_capacity(viewport) = compact_actor_capacity(viewport).saturating_sub(1)`;
  - use that party capacity in `composition_for`;
  - use grid index `0` for the Librarian and shift adventurer indices by one while keeping the same centring calculation for `count + 1` occupants;
  - publish the Librarian region only when the complete sprite fits;
  - do not insert the Librarian into `plan.actors` or change `SceneSnapshot`.

- [ ] Return `interactables: vec![region]` for canonical/compact and `Vec::new()` for vignette/status-only. If painting unexpectedly cannot produce a complete region, omit both pixels and the region rather than publishing a dead click area.

- [ ] Add or extend a proptest in `tests/scene_stage_properties.rs` over viewport dimensions and party sizes. The invariant is:

  ```rust
  for librarian in &frame.interactables {
      prop_assert!(fully_inside(librarian.bounds, viewport));
      prop_assert!(frame.actors.iter().all(|actor| !overlap(actor.bounds, librarian.bounds)));
  }
  ```

- [ ] Run:

  ```bash
  just guild-test
  cargo test --test scene_interaction
  cargo test --test scene_stage_properties
  ```

- [ ] Commit:

  ```bash
  git add src/scene/render/guild_hall.rs tests/scene_guild_hall.rs \
    tests/scene_interaction.rs tests/scene_stage_properties.rs
  git commit -m "feat: station the Librarian in the Guild Hall"
  ```

---

## Task 5: Replace Generic Help with the Typed Ledger Modal

**Files:**
- Create: `src/ledger.rs`
- Modify: `src/lib.rs`
- Modify: `src/app.rs`
- Modify: `src/scene/presentation.rs`
- Modify: `tests/app.rs`
- Modify: `tests/scene_snapshot.rs`

- [ ] Add failing model tests for fresh-open, clamped navigation and transient state:

  ```rust
  #[test]
  fn ledger_opens_at_welcome_and_clamps_at_both_ends() {
      let mut model = Model::new(View::Guild);
      model.toggle_ledger();
      assert_eq!(model.ledger_page(), Some(LedgerPageId::Welcome));
      model.previous_ledger_page();
      assert_eq!(model.ledger_page(), Some(LedgerPageId::Welcome));
      model.last_ledger_page();
      assert_eq!(model.ledger_page(), Some(LedgerPageId::SafeChronicle));
      model.next_ledger_page();
      assert_eq!(model.ledger_page(), Some(LedgerPageId::SafeChronicle));
  }
  ```

- [ ] Run `cargo test --test app ledger` and confirm missing-type/API failures.

- [ ] Create `src/ledger.rs` and export it from `src/lib.rs`:

  ```rust
  #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
  pub enum LedgerPageId {
      Welcome,
      ReadingTheParty,
      QuestmancersTools,
      SafeChronicle,
  }

  impl LedgerPageId {
      pub const ALL: [Self; 4] = [
          Self::Welcome,
          Self::ReadingTheParty,
          Self::QuestmancersTools,
          Self::SafeChronicle,
      ];
  }

  #[derive(Clone, Copy, Debug, Eq, PartialEq)]
  pub struct LedgerPage {
      pub id: LedgerPageId,
      pub title: &'static str,
      pub body: &'static [&'static str],
  }

  #[must_use]
  pub const fn page(id: LedgerPageId) -> LedgerPage {
      match id {
          LedgerPageId::Welcome => LedgerPage {
              id,
              title: "Welcome to the Guild",
              body: &[
                  "You are the Questmancer. Herdr workspaces become campaigns and coding agents become adventurers.",
                  "The Guild Hall keeps the whole party visible. The Delve shows active work in the dungeon.",
                  "Questmancer projects Herdr facts into this scene; it does not manually control an adventurer's state.",
              ],
          },
          LedgerPageId::ReadingTheParty => LedgerPage {
              id,
              title: "Reading the Party",
              body: &[
                  "Working adventurers are carrying out a commission. Needs counsel means the adventurer is waiting for you.",
                  "Completed marks observed spoils. Resting is idle, and Unknown remains unknown.",
                  "The guild never invents a successful ending that Herdr did not report.",
              ],
          },
          LedgerPageId::QuestmancersTools => LedgerPage {
              id,
              title: "Questmancer's Tools",
              body: &[
                  "Use j/k, arrows or g/G to select; Enter observes the selected adventurer.",
                  "Use r for counsel, o for scrying, / to search, Space to acknowledge summons and v for optional Reviewr spoils.",
                  "Keys 1 and 2 move between the Guild Hall and Delve. Esc closes the current parchment.",
              ],
          },
          LedgerPageId::SafeChronicle => LedgerPage {
              id,
              title: "Keeping a Safe Chronicle",
              body: &[
                  "Questmancer stays local. Herdr owns topology and live agent facts; Questmancer stores only small durable intent and its Chronicle.",
                  "The managed Questmancer pane is never an adventurer and cannot receive focus, counsel, output or Reviewr commands.",
                  "Guarded tests use disposable panes and fresh IDs. Herdr 0.7.4 cannot synthesize an explicit done transition.",
              ],
          },
      }
  }
  ```

  Keep this exhaustive fixed match as written. Keep production key names literal: `j/k`, arrows, `g/G`, `Enter`, `r`, `o`, `/`, `Space`, `v`, `1/2`, `Esc` and `?`. State that Reviewr is optional and Herdr 0.7.4 does not synthesize `done` in the safe-testing page.

- [ ] Replace `Modal::Help` with `Modal::LibrarianLedger { page: LedgerPageId }`. Add:

  ```rust
  pub fn toggle_ledger(&mut self);
  pub fn open_ledger(&mut self);          // always Welcome
  pub fn next_ledger_page(&mut self);     // clamp
  pub fn previous_ledger_page(&mut self); // clamp
  pub fn first_ledger_page(&mut self);
  pub fn last_ledger_page(&mut self);
  pub const fn ledger_page(&self) -> Option<LedgerPageId>;
  ```

  Update every exhaustive `Modal` match in `src/app.rs` so Ledger input is ignored and cannot be submitted as counsel/search text.

- [ ] Replace `SceneOverlay::Help` with `SceneOverlay::LibrarianLedger` and map the modal in `ScenePresentation::from_model`.

- [ ] In `tests/scene_snapshot.rs`, prove opening and paging the Ledger does not change `SceneSnapshot`. In `tests/app.rs`, compare `PersistedStateV1::capture` before and after the same operations.

- [ ] Run:

  ```bash
  cargo test --test app --test scene_snapshot
  cargo test --test persisted_state
  ```

- [ ] Commit:

  ```bash
  git add src/ledger.rs src/lib.rs src/app.rs src/scene/presentation.rs \
    tests/app.rs tests/scene_snapshot.rs
  git commit -m "feat: model the Librarian ledger"
  ```

---

## Task 6: Route Keyboard and Pointer Input Through One Ledger Action

**Files:**
- Modify: `src/ui/input.rs`
- Modify: `src/interaction.rs`
- Modify: `tests/input.rs`
- Modify: `tests/interaction.rs`

- [ ] Add failing input tests:
  - normal `?` produces `Action::ToggleLedger`;
  - a click on the Librarian region produces `Action::SelectAt`;
  - a click on empty room produces `Action::Dismiss`;
  - while Ledger is open, `j`/Right is `Next`, `k`/Left is `Previous`, `g/G` is first/last, `Esc` is dismiss and `?` is toggle;
  - mouse clicks and adventurer actions behind the open Ledger produce `Action::None`.

- [ ] Rename `Action::ShowHelp` to `Action::ToggleLedger` everywhere. Keep `Action::SelectAt` as the coordinate-bearing pointer action so hit resolution remains against the frame used for rendering.

- [ ] Update `action_for_scene_event_in` to use `scene.target_at(mouse.column, mouse.row)`:

  ```rust
  return scene.target_at(mouse.column, mouse.row).map_or(
      Action::Dismiss,
      |_| Action::SelectAt { column: mouse.column, row: mouse.row },
  );
  ```

- [ ] Replace `intercept_help_modal` with `intercept_ledger_modal`. It must handle only paging, first/last, dismiss and toggle; every other action is consumed without commands or persistence.

- [ ] Update `reduce_scene_action` to use one typed match:

  ```rust
  match scene.target_at(column, row) {
      Some(SceneTarget::Agent(agent)) => {
          let agent = agent.clone();
          if model.selected_agent_key() == Some(&agent) && model.adventurer_card_visible() {
              model.dismiss_adventurer_card();
          } else {
              select_agent_key(model, &agent, &mut commands);
              model.show_adventurer_card();
          }
      }
      Some(SceneTarget::Interactable(SceneInteractable::Librarian)) => model.open_ledger(),
      None => model.dismiss_adventurer_card(),
  }
  ```

  Opening the Librarian must not change `selected_agent`, `adventurer_card_visible`, or emit an agent command. Closing the Ledger may reveal the pre-existing card state unchanged.

- [ ] Add reducer tests that capture `PersistedStateV1`, selected adventurer and command vectors across:
  - `ToggleLedger` open/page/close;
  - clicked Librarian open;
  - clicks and `Enter` while modal is open;
  - clicked adventurer and empty-room behaviour remain unchanged.

- [ ] Run:

  ```bash
  cargo test --test input --test interaction
  cargo test --test scene_interaction
  ```

- [ ] Commit:

  ```bash
  git add src/ui/input.rs src/interaction.rs tests/input.rs tests/interaction.rs
  git commit -m "feat: open the Ledger from keys and scene hits"
  ```

---

## Task 7: Render the Responsive Librarian's Ledger

**Files:**
- Modify: `src/ui/scene_overlays.rs`
- Modify: `tests/scene_overlays.rs`

- [ ] Add failing TestBackend cases for:
  - wide Ledger title, page title, body, `1 / 4`, and valid controls;
  - compact text-first Ledger with readable body;
  - tiny Ledger with title/navigation and no empty image rectangle;
  - fallback portrait contains non-parchment pixels;
  - native gallery path requests the prepared Librarian illustration;
  - no viewport, including zero-sized nested areas, panics.

- [ ] Run `cargo test --test scene_overlays librarian` and confirm the missing renderer failure.

- [ ] Route `SceneOverlay::LibrarianLedger` to `render_librarian_ledger(frame, model, portraits)`. Use `ledger::page(model.ledger_page().expect("Ledger overlay requires a Ledger modal"))`; the projection must not inspect domain agents.

- [ ] Implement three layouts from the terminal area:
  - wide when at least `96x22`: centred parchment up to `88x20`, `24x16` native image or `24x32` RGB fallback on the left, wrapped page text on the right;
  - compact when at least `56x16`: centred text-first parchment, optional `12x12` fallback only when body width remains at least 36 columns;
  - tiny: use the available area, omit imagery, show page title, concise wrapped body, page number and `Esc/? close`.

  Reuse `render_parchment` and `flush_rgb`. Add a dedicated `render_librarian_illustration` helper:

  ```rust
  if let Some(image) = portraits.and_then(PortraitGallery::librarian) {
      frame.render_widget(Image::new(image), area);
  } else {
      let mut pixels = RgbBuffer::filled(24, 32, PARCHMENT_RGB);
      blit(librarian::ledger_portrait(), PixelPoint::new(0, 0), &mut pixels);
      flush_rgb(frame.buffer_mut(), area, &pixels, PARCHMENT_RGB);
  }
  ```

  Do not call `portrait_for` and do not construct an `AdventurerPersona`.

- [ ] Footer text must be derived from the page index and modal contract, for example:

  ```text
  Page 2 / 4 · j/k or arrows turn pages · g/G ends · Esc/? close
  ```

  At the first/last page, the disabled direction may remain documented but navigation must clamp.

- [ ] Run:

  ```bash
  cargo test --test scene_overlays
  cargo test portrait::tests
  ```

- [ ] Commit:

  ```bash
  git add src/ui/scene_overlays.rs tests/scene_overlays.rs
  git commit -m "feat: render the Librarian ledger"
  ```

---

## Task 8: Make Storybook Own Every New Production Asset Once

**Files:**
- Modify: `src/storybook/assets.rs`
- Modify: `src/storybook/catalogue.rs`
- Modify: `src/storybook/fixtures.rs`
- Modify: `src/storybook/ui.rs`
- Modify: `tests/storybook.rs`

- [ ] Add failing Storybook expectations for these exact stories:

  ```text
  Assets / Librarian
  Interaction / Librarian's Ledger
  ```

  The asset story owns `LibrarianAssets` and shows the world master beside the RGB Ledger fallback. The interaction story owns `LibrarianLedger` and opens the production Ledger at `Welcome`. The existing `World / Guild Hall` story shows `LibrarianAssets` but does not own it.

- [ ] Replace `SceneFirstAsset::HelpParchment` with `SceneFirstAsset::LibrarianLedger` and add `SceneFirstAsset::LibrarianAssets`. Update `ALL`, exhaustive labels and coverage assertions.

- [ ] Change the catalogue from 16 to 17 stories, not 18: the old Help story becomes the Ledger interaction story, and one new Librarian asset story is added. Change `asset_stories()` from `[Story; 8]` to `[Story; 9]`.

- [ ] Add `ArchetypeGallery::Librarian` and a matching `StoryFixture::ArchetypeGallery` render arm in `src/storybook/ui.rs`. Render both authored RGB sprites at native scale with explicit labels `world 16x24` and `Ledger fallback 24x32`. Do not render `src/assets/librarian.png` in this gallery; the production Ledger story exercises native capability when available.

- [ ] Make `librarian_ledger_fixture` call `reduce_action(&mut model, Action::ToggleLedger)` rather than directly mutating the modal. This keeps Storybook on the production interaction path.

- [ ] Add compact and tiny Ledger rendering checks to `tests/storybook.rs` by rendering the same fixed interaction fixture into its minimum viewport and a smaller supported viewport. Retain exact one-owner coverage.

- [ ] Run:

  ```bash
  just storybook-test
  cargo test --features storybook --test storybook
  ```

- [ ] Launch `just storybook` for manual review. Inspect:
  - `World / Guild Hall` at canonical and compact sizes;
  - `Assets / Librarian` silhouettes and palette coherence;
  - wide, compact and tiny `Interaction / Librarian's Ledger`;
  - Storybook capability label and native/fallback behaviour.

  Record visual quality as reviewed only after the user sees and accepts these stories.

- [ ] Commit:

  ```bash
  git add src/storybook/assets.rs src/storybook/catalogue.rs src/storybook/fixtures.rs \
    src/storybook/ui.rs tests/storybook.rs
  git commit -m "feat: catalogue the Librarian in Storybook"
  ```

---

## Task 9: Update the User and Agent Operating Contract

**Files:**
- Modify: `README.md`
- Modify: `AGENTS.md`
- Modify: `docs/manual-test/questmancer-scene-preview.md`
- Modify: `tests/workflow_contract.rb`

- [ ] Add failing workflow-contract assertions first for:
  - `seventeen fixed production stories`;
  - `Librarian's Ledger`;
  - `?` as the single help entry point;
  - Librarian is a non-agent NPC and cannot receive counsel/focus/output;
  - native Librarian illustration retains an authored RGB fallback.

- [ ] Run `ruby tests/workflow_contract.rb` (or the repository's existing workflow-contract command) and confirm the new phrases are absent.

- [ ] Update `README.md`:
  - describe the persistent Librarian and four-page Ledger in user language;
  - replace help parchment references with `Librarian's Ledger`;
  - document `?`, click, page keys and close keys;
  - change Storybook count to seventeen and list the Librarian asset and Ledger interaction;
  - explain that the native PNG shares the existing graphics-bridge requirement and always has an RGB fallback.

- [ ] Update `AGENTS.md` architecture and invariants:
  - `src/ledger.rs` owns fixed typed handbook pages;
  - `SceneFrame` can publish typed non-agent interactables;
  - the Librarian is presentation-owned, persistent only in canonical/compact Guild Hall, and never an agent/domain/persistence object.

- [ ] Update the manual guide:
  - Storybook count seventeen;
  - visual checks for world/fallback/native Librarian art;
  - click opens the same Ledger as `?` without changing selected adventurer;
  - paging clamps and modal input does not leak;
  - Guild vignette omits the sprite but `?` remains available;
  - Delve has no Librarian;
  - no commands or persisted changes are expected.

- [ ] Run:

  ```bash
  ruby tests/workflow_contract.rb
  git diff --check
  ```

- [ ] Commit only these contract files:

  ```bash
  git add README.md AGENTS.md docs/manual-test/questmancer-scene-preview.md tests/workflow_contract.rb
  git commit -m "docs: teach the Librarian ledger workflow"
  ```

---

## Task 10: Complete Automated and Manual Acceptance

**Files:** Verification only unless a discovered defect requires its own test-first fix.

- [ ] Run focused suites:

  ```bash
  cargo test --test librarian_assets
  just guild-test
  just delve-test
  cargo test --test scene_interaction --test input --test interaction --test scene_overlays
  just property-test cases=4096
  just storybook-test
  ```

- [ ] Run the full engineering gate:

  ```bash
  just verify
  cargo build --release
  git diff --check
  git status --short --branch
  ```

  Expected: all automated commands pass. The status may show only explicitly preserved unrelated user changes; Librarian implementation files must be committed.

- [ ] Perform the terminal-free visual gate with `just storybook`. Capture canonical/compact Guild Hall, fallback Ledger and native Ledger evidence. Do not call art quality accepted until the user approves it.

- [ ] Only after automated and Storybook review, use the guarded live procedure in `docs/manual-test/questmancer-scene-preview.md` if live acceptance is requested. Re-baseline Herdr state and use a disposable plain pane. Confirm:
  - Librarian visible/clickable in Guild canonical and compact;
  - absent in Delve and vignette;
  - clicked Librarian and `?` open the same first page;
  - paging and close controls work;
  - selected adventurer remains unchanged;
  - no focus, counsel, output or Reviewr action is sent by Ledger use;
  - native/fallback illustration path is truthfully reported.

- [ ] Inspect plugin logs and restore only resources created by the live test. Do not stop a pre-existing server or unlink a pre-existing plugin.

- [ ] If verification caused no code changes, do not create an empty commit. If a defect was found, add a focused regression test, fix it, rerun the affected and full gates, and commit that correction separately.

---

## Plan Self-Review

- Every approved behaviour has a named source owner and an automated or manual acceptance step.
- The plan has no dynamic-insight provider, project scanner, fake agent, new persistence field or second renderer.
- The PNG is card-only; both world and Ledger fallback art are explicit RGB assets.
- Canonical/compact presence, vignette/status omission, Delve absence, complete hit bounds and non-overlap are tested.
- `?` and click converge on one modal; page state is transient and command/persistence isolation is tested.
- Storybook owns every new asset once and changes the fixed-story count from sixteen to seventeen.
- Existing dirty responsive-layout work is an explicit execution gate rather than hidden in Librarian commits.
- Manual visual and live Herdr acceptance remain separate from automated claims.
