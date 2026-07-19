# Questmancer plan

Questmancer turns a Herdr session into a living adventurers' guild. The user is
the Questmancer; workspaces are campaigns; agents are adventurers; blocked work
raises Summons; completed work returns as spoils.

The v0.1 product is one Rust package, one domain model, and two Ratatui
projections: the operational **Guild Hall** and the spatial **Delve**. Herdr
owns live session facts. Questmancer owns only presentation and durable local
intent.

## Compatibility boundary

- Herdr `0.7.4`, protocol `16`
- `session.snapshot` plus scoped lifecycle and agent-status subscriptions
- Separate socket connections for ordinary requests and the subscription loop
- `HERDR_BIN_PATH` and Herdr-provided plugin config/state directories
- Synthetic agent states limited to `idle`, `working`, `blocked`, and `unknown`;
  `done` acceptance requires a real agent or fixture coverage

## Architecture

```text
Herdr snapshot + events
          |
          v
 pure reducer / shared model ----> debounced state + Chronicle JSONL
          |
          +----> Guild Hall
          +----> Delve
          |
          +----> commands: observe, counsel, output, optional Spoils inspection
```

Presence and attention are distinct. Rendering never owns domain truth. The
selected adventurer's output is loaded lazily, never on animation frames.
Animation derives deterministic frames and the next semantic deadline from one
injected monotonic clock. Persistence stores versioned user intent without
copying Herdr topology, agent output, or live state.

## v0.1 milestones

### 1. Executable and lifecycle — complete

- Rust 2024 binary, verified Herdr manifest, and singleton pane controller
- Safe terminal setup, structured shutdown, and offline layout mode
- Responsive empty Guild Hall and Delve projections

### 2. Herdr protocol — complete

- Protocol-16 environment validation and schema-derived fixtures
- Newline-delimited request and subscription clients
- Capped reconnect, resubscription, and fresh snapshots without discarding the
  last useful visible state

### 3. Domain core — complete

- Typed identities, presence, attention, campaigns, adventurers, personas,
  Chronicle entries, and timestamps
- Pure reducer with explicit command effects
- Stable persona generation, deterministic campaign rollups, and bounded event
  deduplication

### 4. Guild Hall — complete

- Quest board, party, Summons, Chronicle, selected adventurer, and scrying table
- Selection, search, observation, counsel, acknowledgement, output refresh, and
  optional Reviewr integration
- Wide, narrow, ASCII, ANSI-16, reduced-motion, and reconnect-safe projections

### 5. Delve — complete

- Connected deterministic dungeon geometry per campaign
- State-specific adventurer silhouettes, chambers, props, and selected profile
- Action parity with the Guild Hall at wide, compact, and tiny sizes
- Bounded semantic effects and event-driven static/no-motion rendering

### 6. Persistence and hardening — complete

- User-owned typed configuration with explicit precedence
- Atomic versioned `state.json` and tolerant append-only `chronicle.jsonl`
- Debounced writes, unchanged-state suppression, bounded diagnostics, and
  shutdown flush
- Property tests for domain and persistence invariants
- Managed-pane exclusion and goblin overlays that cannot corrupt UI truth

### 7. Product art and voice — complete

- Original fantasy silhouettes with independent ancestry, class, and keepsake
  recognition anchors
- Guild architecture, dungeon scenery, rare deterministic goblin sightings,
  and a bounded hidden outbreak interaction
- Warm but precise copy that keeps operational states truthful

### 8. Documentation and release — complete

- Source-first Herdr `0.7.4` setup, migration, operation, configuration,
  fake-agent, privacy, recovery, and cleanup guidance
- Four-target GitHub Actions release matrix with root-level `questmancer`
  archives and a release-wide `SHA256SUMS`
- Installer asset selection and checksum flow aligned with published names
- CI gates for formatting, Clippy warnings, all-target/all-feature tests, shell
  behavior and syntax, release build, and diff hygiene

### 9. Scene-first production cutover — complete

- One RGB scene renderer now owns the Guild Hall and Delve in production
- Contextual parchment overlays preserve selection, observation, counsel,
  search, scrying, acknowledgement and Reviewr actions
- The legacy text renderer and standalone scene-preview binary were removed
- Storybook now reviews eight fixed stories through the production scene path

## Release acceptance

The v0.1 release candidate is ready only when all of the following are true:

1. `cargo fmt --all --check` is clean.
2. Clippy passes for all targets and features with warnings denied.
3. All Rust and shell tests pass.
4. Lifecycle scripts pass Bash syntax checks.
5. `cargo build --release` produces executable `target/release/questmancer`.
6. Current user and release surfaces contain no superseded product identity.
7. The release workflow names exactly four supported target archives and builds
   `SHA256SUMS` after downloading them.
8. The README does not claim Herdr `0.7.4` can synthesize `done`.

## Backlog and release closure

The v0.1 product scope above is feature-complete. The following work is either
release closure or an explicitly post-v0.1 enhancement; it does not reopen the
core Guild Hall, Delve, persistence, or Storybook milestones.

### Release closure

- Repeat the guarded Herdr `0.7.4` smoke from current `main` after the RGB
  production cutover; visual approval is recorded, while live interaction and
  transition acceptance remain separate evidence.
- Publish the intended `opsydyn/herdr-questmancer` repository, tag `v0.1.0`, and
  verify all four archives, `SHA256SUMS`, and `herdr/install.sh` against the
  published release. Until then, source linking remains the supported path.
- Capture a current Guild Hall and Delve screenshot or terminal recording for
  the first release page.

### Post-v0.1 product backlog

- Make Storybook `j` / `k` and Up / Down traverse its visibly flat story list
  across category boundaries. Keep `h` / `l` and Left / Right as category jumps,
  and keep navigation clamped at the first and last story.
- Add the opt-in Herdr `0.7.4` workspace and agent sidebar rows described in
  `docs/superpowers/plans/2026-07-15-herdr-074-sidebar-integration.md`.
- Repeat the optional Reviewr integration smoke when
  `persiyanov.reviewr.open` is installed, without making Reviewr a dependency.
- Capture a real-agent resting and returned-spoils transition when available;
  Herdr `0.7.4` cannot synthesize an explicit `done` report, so fixture coverage
  remains the honest v0.1 proof for those projections.

## Engineering rules

- No unsafe Rust, telemetry, cloud service, database, copied product assets, or
  terminal image protocol in v0.1.
- Start behavior changes with a failing test.
- Keep documentation, operational recipes, and release checks in the same slice
  as the behavior they describe.
- Never poll selected output or write persistence on an animation wake.
- Restore the terminal after normal exit, error, signal, and panic.
