# herdr-webmaster plan

`herdr-webmaster` turns a Herdr session into a useful 90s webmaster control
centre. The user is the webmaster; workspaces are sites; agents are
contributors; blocked work is webmaster mail; completed work is a site update.

The plugin is one Rust package with one domain model and two Ratatui projections:
the operational **webmaster desk** and the ambient, interactive **cybercafe**.

## Compatibility baseline

- Target Herdr `0.7.3`, where `session.snapshot`, `herdr api schema`, the `done`
  agent state, and the required subscription surface are available.
- The local development and live acceptance environment runs Herdr `0.7.3`.
- Treat `herdr api schema --output <path>` as protocol ground truth.
- Use `HERDR_BIN_PATH` for action scripts and separate socket connections for
  request/response traffic and subscriptions in the long-running TUI.

## Architecture

```text
Herdr snapshot + events
          |
          v
 pure reducer / shared Model ----> debounced JSON + guestbook JSONL
          |
          +----> webmaster desk
          +----> cybercafe
          |
          +----> effect commands (focus, reply, output, reviewr)
```

Presence and attention remain separate. Widgets render derived state and never
own domain truth. Pane output is loaded only for the selected agent. Animation
uses a clock-derived frame so rendering remains deterministic in tests.

## Milestones

### 1. Executable shell — complete

- Initialize the Rust package and repository policy files.
- Add the verified Herdr manifest and lifecycle scripts.
- Implement safe terminal setup and teardown.
- Render empty desk and cybercafe views with responsive fallbacks.
- Add deterministic `TestBackend` rendering tests and CI.

Exit: `cargo test`, `cargo fmt --check`, and `cargo clippy` pass; the local
binary can switch between empty desk and cafe views.

### 2. Herdr protocol — complete

- Parse plugin environment and validate Herdr compatibility.
- Implement newline-delimited request and subscription connections.
- Decode `session.snapshot` and subscribed lifecycle events.
- Reconnect with capped backoff and resnapshot without losing visible state.
- Add framing, unknown-field, interleaving, error, and reconnect fixtures.

Exit: fixture-driven tests prove bootstrap, subscription, disconnect, and
resnapshot behavior without a live Herdr server.

### 3. Domain core — complete

- Add typed IDs, presence, attention, agent, site, guestbook, and persona types.
- Implement the pure reducer and effect command boundary.
- Derive site rollups using the documented status priority.
- Deduplicate guestbook events and generate stable personas.

Exit: reducer tests cover every required transition and snapshot replacement.

### 4. Webmaster desk — complete

- Render sites, webmaster mail, guestbook, and selected-agent details.
- Load selected output lazily.
- Focus panes, compose/send replies, mark attention seen, and search.
- Discover and optionally invoke `persiyanov.reviewr.open`.

Exit: focused application, command, runtime, interaction, reply, and rendering
tests verify the scan, inspect, reply, seen, search, reviewr, reconnect, lazy
output, and visit behavior. The README carries a separate manual live Herdr
`0.7.3` acceptance procedure; completing that procedure remains a release
gate, not a claim made by this milestone.

### 5. Cybercafe

- Render a responsive workstation grid and tiny-terminal list fallback.
- Add deterministic personas and semantic working/blocked/done/idle/exited art.
- Use the approved half-block seated-sprite and full-body profile system from
  `docs/superpowers/specs/2026-07-14-pixel-art-design.md`.
- Separate persistent state, transition effects, and frame animation.
- Support selection, focus, reply, reduced motion, and ASCII mode.

Exit: every agent state is legible without colour and remains actionable.

### 6. Persistence, integration, and release

- Persist configuration, personas, selection/view, and bounded guestbook state.
- Harden singleton open/close/toggle/desk/cafe actions and stale-state recovery.
- Add install/checksum scripts and macOS/Linux release workflows.
- Complete README, manual fake-agent guide, recording, and privacy statement.
- Run live Herdr `0.7.3` acceptance and idle-CPU checks.

Exit: all twenty v0.1 acceptance criteria in the product handoff pass.

## Engineering rules

- No unsafe Rust, database, telemetry, network service, copied product assets,
  or terminal image protocol in v0.1.
- Every behavior change starts with a failing test.
- Every milestone includes its documentation and verification commands.
- No output polling per frame and no persistence writes per animation tick.
- The terminal must be restored after normal exit, error, signal, and panic.
