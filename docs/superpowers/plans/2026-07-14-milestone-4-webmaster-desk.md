# herdr-webmaster Milestone 4 Webmaster Desk Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: use
> `superpowers:subagent-driven-development` or `superpowers:executing-plans`,
> with `superpowers:test-driven-development` for every behavior slice.

**Goal:** Turn the protocol and domain layers into the operational webmaster
desk: live sites and mail, selection, recent output, pane focus, reply, seen
state, search, and optional reviewr launch.

**Architecture:** Expand the app model to own the domain state, connection
state, selection, output preview, and modal state. A Tokio runtime multiplexes
terminal input with `ConnectionSupervisor` updates. A wire-event adapter emits
semantic reducer events. Reducer commands execute through typed Herdr request
methods; selected output is fetched only after selection/revision changes or an
explicit refresh. Ratatui widgets remain read-only projections.

## Constraints

- Keep one shared model for desk and cafe.
- Never fetch output for every pane or on animation/render ticks.
- Keep pushed events off ordinary request connections.
- Validate all new wire methods against Herdr 0.7.3 protocol 16.
- Preserve the last visible domain state while reconnecting.
- Reply text stays local until explicit send; escape cancels it.
- Footer actions reflect current selection and optional reviewr availability.
- The user is the webmaster; no invented webmaster agent.
- Every narrow/tiny layout must remain safe.

---

### Task 1: Compose application and domain state

**Files:** `src/app.rs`, `tests/app.rs`

- [ ] Add connection state, domain state, selected region, output preview,
  status message, and modal state to the existing app model.
- [ ] Preserve the simple `Model::new(view)` empty/offline startup path.
- [ ] Add selection movement and selected-agent helpers with boundary tests.
- [ ] Commit the green slice.

### Task 2: Adapt wire updates into semantic events

**Files:** `src/herdr/event_adapter.rs`, `tests/event_adapter.rs`

- [ ] Decode lifecycle snake-case and dotted agent-status payloads without
  moving Serde `Value` handling into the reducer.
- [ ] Map connected snapshots, workspace close, pane exit, and status changes.
- [ ] Request a resnapshot for incomplete/unknown topology payloads.
- [ ] Preserve unknown events as no-op diagnostics.
- [ ] Commit the green slice.

### Task 3: Add typed interaction requests

**Files:** `src/herdr/protocol.rs`, `src/herdr/client.rs`, `tests/actions.rs`

- [ ] Add schema-derived `pane.focus`, `pane.send_text`, `pane.read`,
  `plugin.action.list`, and `plugin.action.invoke` parameters/results.
- [ ] Expose typed client helpers and verify request/result discriminators,
  recent-unwrapped text mode, and server errors with fake Unix listeners.
- [ ] Commit the green slice.

### Task 4: Execute reducer and UI commands

**Files:** `src/command.rs`, `tests/command.rs`

- [ ] Execute focus, reply, output load, resnapshot, reviewr discovery/invoke,
  and local persistence intents outside reducers/widgets.
- [ ] Surface failures as non-blocking status messages.
- [ ] Enforce focus-selected-pane before invoking reviewr.
- [ ] Commit the green slice.

### Task 5: Async terminal runtime

**Files:** `src/terminal.rs`, `src/main.rs`, `tests/runtime_loop.rs`

- [ ] Replace blocking polling with a Tokio-select loop over crossterm input,
  supervisor updates, command completions, shutdown, and render invalidation.
- [ ] Bootstrap from plugin environment, retain state through disconnect, and
  cancel all tasks before terminal restoration.
- [ ] Keep direct development startup useful when Herdr env is absent by
  showing an offline explanation instead of crashing.
- [ ] Commit the green slice.

### Task 6: Responsive webmaster desk

**Files:** `src/ui/layout.rs`, `src/ui/widgets/*`, `src/ui/views/desk.rs`,
`tests/desk_rendering.rs`

- [ ] Render wide three-column, medium two-column, and narrow tabbed layouts.
- [ ] Add site list, webmaster mail, guestbook, selected agent details, elapsed
  blocked time, connection banner, and bounded output preview.
- [ ] Test empty, working, blocked, done/unseen, exited, disconnected, 80x24,
  and sub-80 widths with injected time.
- [ ] Commit the green slice.

### Task 7: Desk input and reply flow

**Files:** `src/ui/input.rs`, `src/ui/views/reply.rs`, `tests/input.rs`,
`tests/reply.rs`

- [ ] Implement region cycling, j/k and arrows, first/last, visit, mark seen,
  refresh, reviewr, search, and contextual footer actions.
- [ ] Implement reply composer editing, send, cancel, and clear.
- [ ] Add mouse selection only after the keyboard loop is green.
- [ ] Commit the green slice.

### Task 8: Documentation and verification

**Files:** `README.md`, `PLAN.md`, `CHANGELOG.md`, `justfile`

- [ ] Document live desk startup, key map, lazy output behavior, and optional
  reviewr integration.
- [ ] Run the complete format, Clippy, Rust/shell test, fixture, release-build,
  diff, and live Herdr 0.7.3 acceptance gates.
- [ ] Commit the verified milestone.

## Acceptance

- A live blocked transition appears as unread webmaster mail without restart.
- Selection moves deterministically across sites/agents and survives updates.
- Enter focuses the real selected pane.
- Reply sends exactly the composed text to that pane.
- Selected recent output loads lazily and refreshes on relevant revisions.
- Space changes unseen attention to seen locally.
- Reviewr appears only when `persiyanov.reviewr.open` exists and focuses first.
- Disconnect preserves visible state and shows reconnecting status.
- Desk remains usable at 80x24 and safe below 80 columns.
