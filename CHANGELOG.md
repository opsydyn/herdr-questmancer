# Changelog

All notable changes to this project will be documented here.

## [Unreleased]

### Added

- Initial Rust 2024 and Ratatui package.
- Empty webmaster desk and cybercafe projections.
- Keyboard view switching, resize redraw, signal-aware shutdown, and guarded
  terminal restoration.
- Herdr `0.7.3` plugin manifest and singleton lifecycle actions.
- Runtime registration cleanup and reliable existing-pane desk/cafe switching.
- Rust rendering, input, CLI, model, and shell lifecycle tests.
- Herdr protocol `16` environment parsing and schema-derived fixtures.
- Newline-delimited async framing and one-connection-per-request client.
- Mixed lifecycle and per-pane agent-status event subscriptions.
- Protocol compatibility reporting, cancellable capped reconnect backoff, and
  fresh snapshots after disconnects or pane-topology changes.
- Focused protocol verification and live plugin-link acceptance against Herdr
  `0.7.3` / protocol `16`.
- Typed workspace, pane, agent, persona, event, and timestamp domain values.
- Separate presence and webmaster-attention state with derived site rollups.
- Stable original persona handles and appearance traits based on strongest
  available agent identity.
- Deterministic bounded guestbook history and a pure, effect-returning reducer.
- Operational webmaster desk with responsive site, mail, guestbook, selected
  agent, connection, status, and recent-output views.
- Async live terminal runtime with reconnect/resnapshot behavior, retained
  visible state, command completion handling, and structured task shutdown.
- Lazy selected-pane output, pane focus, reply composition, local seen state,
  agent/site search, and optional `persiyanov.reviewr.open` integration.
- Rendering and interaction coverage for empty, working, blocked, done/unseen,
  exited, disconnected, narrow, modal, and contextual-footer states.
- Responsive actionable cybercafe with a workstation grid, selected-agent
  profile, compact sub-80-column list, dense-herd paging, and preserved poses
  during connection overlays.
- Original deterministic 10x12 seated pixel characters and independently
  composed 16x32 full-body profiles with stable silhouette and appearance
  anchors; the supplied visual reference is not reproduced.
- Semantic non-colour theatre signals for working, blocked, done-unseen,
  done-seen, idle, exited, focused, and unknown states.
- Unicode half-block and ASCII projections, xterm-256 and ANSI-16 palettes,
  plus full, reduced, and no-motion display models.
- A cancellation-safe adaptive animation scheduler that arms one resettable
  sleep for the earliest phase-aware boundary across every cafe agent and stays
  pending in event-driven modes.
- Event-driven Unix shutdown signals replace the previous 50 ms polling
  interval, leaving static and no-motion sessions without periodic timers.
- Exact eight-frame, one-second completion confetti followed by a stable update
  badge, with no output reads or persistence effects on animation wakes.
- Local typed `config.toml` settings with explicit CLI/saved/configured view
  precedence and persisted display preferences.
- Versioned durable intent in atomically replaced `state.json`, including stable
  personas, valid selection, and exact seen-attention episodes without copying
  Herdr-owned live topology or output.
- Append-only `guestbook.jsonl` history with ordered, deduplicated, bounded
  replay that preserves valid records around malformed or truncated input.
- One debounced persistence worker with unchanged-state suppression, acknowledged
  guestbook writes, non-fatal bounded diagnostics, and shutdown flush.
- Named persistence and core-domain Proptest invariants with tracked shrinking
  regression seeds and focused contributor recipes.

### Fixed

- Animation scheduling follows presence and attention timestamps rather than a
  nominal maximum FPS, preventing drift, skipped mixed-rate frames, and
  completion confetti extending past its exact one-second boundary.
- Runtime time samples the wall epoch once and advances with Tokio's monotonic
  clock; animation sleeps use absolute deadlines on that same origin so render
  latency and later wall-clock adjustments cannot move semantic boundaries.
- Topology-event replay no longer causes a tight resubscription loop; the
  supervisor refreshes pane membership and rebuilds subscriptions only when
  the subscribed pane set actually changes.
- Invalid local configuration and durable state fail closed with visible
  diagnostics; atomic publication retains the last complete state on write
  failure, and damaged guestbook records no longer hide valid history.
- An unreadable, malformed, unsupported, or relationship-invalid `state.json`
  now disables state publication until restart, preventing the initial live
  snapshot from overwriting the file while guestbook persistence remains live.
