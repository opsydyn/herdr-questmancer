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

### Fixed

- Topology-event replay no longer causes a tight resubscription loop; the
  supervisor refreshes pane membership and rebuilds subscriptions only when
  the subscribed pane set actually changes.
