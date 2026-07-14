# Task 3 report: original persona composition

## RED evidence

The first production edit followed a failing integration test. Command:

```text
cargo test --test persona_art
```

Observed exit code `101` with the expected missing-feature diagnostics:

```text
error[E0432]: unresolved import `herdr_webmaster::ui::persona`
  --> tests/persona_art.rs:9:9
   |
9  |         persona::{AppearanceRoles, appearance_roles, compose_profile, compose_seated},
   |         ^^^^^^^ could not find `persona` in `ui`

error[E0624]: method `width` is private
  --> tests/persona_art.rs:78:36
```

The compositional refactor also used a focused RED before removing the invalid
compact/blocked whole-sprite branch. The semantic golden expected the generic
head/torso/pose/legs/accessory pipeline and failed against the special-case map;
after removing the branch and strengthening the reusable leg/shoe primitive,
the focused test returned GREEN.

## Changes

- Added public `AppearanceRoles` and deterministic `appearance_roles` mapping
  for skin, hair, top, bottom, shoes, accessory, accent, highlight, and shadow.
- Extended `ColorRole` with typed skin, hair, fabric, footwear, and accent shade
  roles. Xterm-256 preserves trait variation; ANSI-16 uses deterministic
  adjacent-role and canvas-background contrast fallback when preferred mappings
  collapse.
- Added a dedicated 10x12 seated compositor with typed CRT-facing, raised-hand,
  relaxed, and absent poses. Working frames alternate a hand position from the
  injected `animation_frame`; blocked keeps a stable raised-hand silhouette;
  exited produces no person pixels.
- Added a separately authored 16x32 neutral full-body compositor. It has its own
  proportion layouts and clipped head, hair/face, torso/arms, legs/shoes, and
  accessory primitives; it does not scale or crop seated art.
- Exposed read-only canvas dimensions for logical-composition verification.
- Added compact, tall, and broad fixtures plus semantic logical-role goldens.
  Tests cover dimensions, distinct silhouette masks in both representations,
  shared recognition anchors, ANSI-16 adjacency contrast, deterministic working
  frames, stable blocked shape, exited absence, and neutral profile identity.

## Verification

- `cargo test --test persona_art` — 6 passed.
- `cargo test --test pixel` — 11 passed.
- `cargo test` — full Rust suite passed.
- `cargo fmt --all --check` — passed.
- `cargo clippy --all-targets --all-features -- -D warnings` — passed.
- `git diff --check` — passed.

## Self-review and originality

- Removed the first fixture-shaped compact/blocked implementation during review;
  every seated persona now uses the same compositional primitive pipeline.
- The art is original code-native block geometry derived only from the written
  fidelity grammar. It does not reproduce supplied characters, costumes, logos,
  poses, or scene composition, and it uses no raster/image protocol or theme
  framework.
- Composition functions are pure: no clock reads, randomness, timers, persistence,
  or domain mutation. Theatre state enters only through the copied `TheatreFrame`.
- No unresolved implementation concerns remain for Task 3. Workstation props and
  textual state markers remain correctly owned by the later widget task.
