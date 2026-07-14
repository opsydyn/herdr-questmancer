# Milestone 5 Cybercafe Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> `superpowers:subagent-driven-development` (recommended) or
> `superpowers:executing-plans` to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the empty cafe projection with an actionable animated room of
original seated agent sprites and a separately composed full-body profile.

**Architecture:** The domain remains the sole owner of identity, presence, and
attention. Pure theatre functions derive pose, transition frame, and render
cadence from an `Agent`, display preferences, and the injected model clock.
Pixel modules compose semantic colour roles on clipped logical canvases and
pack two vertical pixels into one Ratatui cell; widgets only project those
packed results.

**Tech Stack:** Rust 2024, Ratatui 0.30, xterm-256/ANSI-16 colours, Unicode
half-blocks, `TestBackend`, Tokio.

## Global Constraints

- Keep one shared `Model` for desk and cafe.
- Use original block art only; do not reproduce supplied characters, outfits,
  logos, poses, or composition.
- Cafe sprites are dedicated 10x12 logical-pixel seated compositions.
- Profiles are dedicated 16x32 logical-pixel full-body compositions.
- Unicode half-block rendering is canonical; ASCII is a semantic fallback.
- State must never depend on colour alone.
- Renderers own no timers and mutate no domain state.
- Animation must never trigger pane reads or persistence writes.
- `Motion::None` is event-driven; idle unchanged cafes do no periodic work.
- Zero-sized and tiny areas must never panic.
- No terminal image protocols, raster assets, sound, database, or theme
  framework in this milestone.

---

### Task 1: Pixel canvas, semantic palette, and half-block packing

**Files:**
- Create: `src/ui/pixel/mod.rs`
- Create: `src/ui/pixel/canvas.rs`
- Create: `src/ui/pixel/palette.rs`
- Create: `src/ui/pixel/pack.rs`
- Modify: `src/ui/mod.rs`
- Test: `tests/pixel.rs`

**Interfaces:**
- Produces: `Canvas::new(width, height)`, clipped `set`, `fill_rect`, and
  `pixels`; `ColorRole`; `Palette::{Xterm256,Ansi16}`; and
  `pack(&Canvas, &Palette, ColorRole) -> Text<'static>`.
- Consumers: Tasks 3 and 4 compose canvases and render packed text.

- [ ] **Step 1: Write failing canvas tests** for transparent initialization,
  in-bounds writes, clipped out-of-bounds writes, and clipped rectangle fills.
- [ ] **Step 2: Run `cargo test --test pixel`** and confirm failure because the
  pixel module/API does not exist.
- [ ] **Step 3: Implement the minimal canvas** using one row-major
  `Vec<Option<ColorRole>>`; dimensions are `u16`, indexing is checked, and
  drawing outside the canvas is ignored.
- [ ] **Step 4: Add failing packing tests** for empty/empty, colour/empty,
  empty/colour, same/same, and distinct top/bottom pairs. Assert glyph plus
  foreground/background colour, not terminal escape bytes.
- [ ] **Step 5: Implement palette resolution and packing** with ` `, `▀`, `▄`,
  and `█`. Xterm roles resolve to `Color::Indexed`; ANSI roles resolve to the
  closest named `Color` while preserving contrast.
- [ ] **Step 6: Run focused tests, formatting, and Clippy**, then commit
  `feat: add semantic terminal pixel canvas`.

### Task 2: Pure theatre state and display preferences

**Files:**
- Modify: `src/app.rs`
- Create: `src/ui/theatre.rs`
- Modify: `src/ui/mod.rs`
- Test: `tests/theatre.rs`

**Interfaces:**
- Produces: `Motion::{Full,Reduced,None}`,
  `CharacterSet::{Unicode,Ascii}`, `DisplayPreferences`,
  `TheatrePose::{Working,Blocked,DoneUnseen,DoneSeen,Idle,Exited,Unknown}`,
  `TheatreFrame { pose, animation_frame, focused, label }`, and
  `frame_for(agent, now, preferences)`.
- Produces: `RenderCadence::{EventDriven,Fps(u8)}` and
  `cadence_for(model) -> RenderCadence`.
- Consumers: Tasks 3-6.

- [ ] **Step 1: Write failing pose tests** mapping every presence/attention
  combination to its explicit label (`BUILDING`, `HELP!`, `UPDATE READY`,
  `DONE`, `IDLE`, `BROKEN LINK`, `UNKNOWN`).
- [ ] **Step 2: Verify RED** with `cargo test --test theatre`.
- [ ] **Step 3: Implement preferences on `Model` and pure pose derivation**.
  Done-unseen animation is derived from attention time and ends after exactly
  1,000 ms; it does not mutate attention.
- [ ] **Step 4: Add failing clock/cadence tests** for 6 fps working, 2 fps
  blocked, 8 fps one-shot done, 1 fps idle, static done-seen/exited/unknown,
  reduced-motion static effects, and no-motion event-driven rendering.
- [ ] **Step 5: Implement deterministic frame/cadence calculation** with no
  wall-clock reads inside the module.
- [ ] **Step 6: Run focused/full tests, fmt, and Clippy**, then commit
  `feat: derive deterministic cafe theatre state`.

### Task 3: Original seated and full-body persona composition

**Files:**
- Create: `src/ui/persona/mod.rs`
- Create: `src/ui/persona/appearance.rs`
- Create: `src/ui/persona/cafe_sprite.rs`
- Create: `src/ui/persona/profile.rs`
- Create: `src/ui/persona/state_pose.rs`
- Modify: `src/ui/mod.rs`
- Test: `tests/persona_art.rs`

**Interfaces:**
- Consumes: `PersonaAppearance`, `TheatreFrame`, `Canvas`, and `ColorRole`.
- Produces: `appearance_roles(&PersonaAppearance) -> AppearanceRoles`,
  `compose_seated(&PersonaAppearance, TheatreFrame) -> Canvas`, and
  `compose_profile(&PersonaAppearance) -> Canvas`.

- [ ] **Step 1: Write failing logical-sprite tests** with fixed persona traits
  for compact, tall, and broad silhouettes. Assert 10x12 seated and 16x32
  profile dimensions plus distinct silhouette masks.
- [ ] **Step 2: Verify RED** with `cargo test --test persona_art`.
- [ ] **Step 3: Implement semantic appearance-role mapping** for skin, hair,
  top, bottom, shoes, accessory, accent, highlight, and shadow. Adjacent roles
  that collapse to one colour fall back to a known contrasting set.
- [ ] **Step 4: Implement neutral full-body composition** from reusable clipped
  primitives for head/hair/face, torso/top, arms/accessory, legs/bottom, and
  shoes. Profiles remain neutral regardless of presence.
- [ ] **Step 5: Implement dedicated seated composition** with CRT-facing,
  raised-hand blocked, relaxed done/idle, and absent-person exited poses. Do
  not scale or crop the profile canvas.
- [ ] **Step 6: Add golden logical-role maps** proving the same recognition
  anchors survive both representations and blocked/exited silhouettes remain
  distinct without colour.
- [ ] **Step 7: Run focused/full tests, fmt, and Clippy**, then commit
  `feat: compose original webmaster personas`.

### Task 4: Workstation and profile widgets

**Files:**
- Create: `src/ui/widgets/mod.rs`
- Create: `src/ui/widgets/agent_crt.rs`
- Create: `src/ui/widgets/profile_card.rs`
- Modify: `src/ui/mod.rs`
- Test: `tests/cafe_widgets.rs`

**Interfaces:**
- Produces: `render_workstation(frame, area, agent, theatre, selected,
  preferences)` and `render_profile_card(frame, area, agent, theatre,
  preferences)`.
- Consumers: Task 5 cafe layout.

- [ ] **Step 1: Write failing `TestBackend` workstation tests** for working,
  blocked, done-unseen, done-seen, idle, exited, focused, and unknown. Assert
  explicit labels and non-colour markers such as raised hand/help card, update
  badge, empty chair/broken CRT, and live lamp.
- [ ] **Step 2: Verify RED** with `cargo test --test cafe_widgets`.
- [ ] **Step 3: Implement the workstation** with a name row, six scene rows,
  state row, border, CRT, desk, chair, sprite, deterministic modem lights, and
  optional custom status. Minimum usable area is 28x10 cells.
- [ ] **Step 4: Add failing profile-card tests** proving the full 16x32 figure,
  handle, site/state details, and accessory are visible independently of the
  seated pose.
- [ ] **Step 5: Implement Unicode and ASCII widget branches**. ASCII uses
  labelled compact silhouettes and `[~]`, `[!]`, `[+]`, and `[x]`; it never
  emits half-block glyphs.
- [ ] **Step 6: Run focused/full tests, fmt, and Clippy**, then commit
  `feat: render cafe workstations and profiles`.

### Task 5: Responsive actionable cybercafe

**Files:**
- Modify: `src/ui/views/cafe.rs`
- Modify: `src/ui/mod.rs`
- Modify: `src/interaction.rs`
- Test: `tests/cafe_rendering.rs`
- Test: `tests/interaction.rs`

**Interfaces:**
- Consumes: shared `Model`, theatre derivation, workstation/profile widgets,
  and existing `Action` reduction.
- Produces: responsive cafe projection and the same visit/reply/search/seen
  commands already used by the desk.

- [ ] **Step 1: Write failing responsive rendering tests** at 160x50, 120x30,
  80x24, 60x18, and zero/tiny dimensions using a three-agent fixed model.
- [ ] **Step 2: Verify RED** with `cargo test --test cafe_rendering`.
- [ ] **Step 3: Implement the room layout**: shared wall/floor/cables, largest
  useful workstation grid, and selected profile beside the grid at >=120
  columns. At 80-119 columns render the grid without the side profile; below
  80 render a vertical compact workstation list.
- [ ] **Step 4: Add failing action tests** proving cafe selection, search,
  visit, reply, seen, and output refresh reuse the existing typed command
  boundary without cafe-specific Herdr effects.
- [ ] **Step 5: Implement contextual footer and disconnected overlay** while
  preserving the last poses. Selection adds a cursor, double corner, lamp, and
  `LIVE` label without replacing the state label.
- [ ] **Step 6: Add ASCII, ANSI-16, reduced-motion, and tiny-layout tests** and
  ensure blocked/exited remain explicit.
- [ ] **Step 7: Run focused/full tests, fmt, and Clippy**, then commit
  `feat: build responsive actionable cybercafe`.

### Task 6: Adaptive animation scheduling and milestone verification

**Files:**
- Modify: `src/terminal.rs`
- Modify: `src/ui/theatre.rs`
- Modify: `README.md`
- Modify: `PLAN.md`
- Modify: `CHANGELOG.md`
- Modify: `justfile`
- Test: `tests/theatre.rs`
- Test: `tests/cafe_rendering.rs`
- Test: `tests/runtime_loop.rs`

**Interfaces:**
- Consumes: `cadence_for(&Model)`.
- Produces: an event-driven render invalidator that waits indefinitely for
  `EventDriven` and schedules only the next required visible frame otherwise.

- [ ] **Step 1: Write failing scheduler tests** proving full-motion working and
  transition frames invalidate at their required cadence, idle at 1 fps,
  reduced motion only when a visible slow frame changes, and no motion never
  wakes on time alone.
- [ ] **Step 2: Verify RED** with focused theatre/runtime tests.
- [ ] **Step 3: Replace the fixed one-second terminal interval** with a resettable
  sleep derived after every model/input/runtime event. Keep shutdown and input
  cancellation safe and never spawn one task per frame.
- [ ] **Step 4: Add a done-transition test** proving confetti renders for eight
  frames and the stable update badge remains after one second.
- [ ] **Step 5: Update docs and focused recipes** with cafe controls, display
  preferences, state mapping, and exact verification commands. Do not claim
  config-file persistence until Milestone 6.
- [ ] **Step 6: Run `cargo fmt --all --check`, warnings-denied Clippy, all Rust
  tests, shell tests, shell syntax, release build, diff check, and a live Herdr
  `0.7.3` cafe smoke test. Confirm an unchanged no-motion cafe causes no
  timer-driven redraws.
- [ ] **Step 7: Commit** `feat: complete animated cybercafe milestone`.

## Acceptance

- Three fixed personas are distinguishable by silhouette in seated and profile
  forms.
- Working, blocked, done-unseen, done-seen, idle, exited, focused, and unknown
  have explicit non-colour state communication.
- Done confetti stops after exactly one second.
- 80x24 remains actionable; sub-80, zero, and tiny areas are safe.
- Unicode, ASCII, ANSI-16, reduced-motion, and no-motion projections pass.
- Selection, focus, reply, seen, search, and refresh use the same typed effects
  as the desk.
- Idle/no-motion rendering performs no unnecessary periodic work.
- The final visuals remain original and use the supplied IT Crowd image only as
  a fidelity reference.
