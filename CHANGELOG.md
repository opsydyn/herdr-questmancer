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
