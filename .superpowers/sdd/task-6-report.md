# Task 6 report: concurrent startup load and deterministic precedence

## Status

Implemented and verified Task 6 without wiring the persistence worker or
terminal startup lifecycle reserved for Task 7.

## TDD evidence

### RED 1: startup restoration boundary

Tests were added first in `tests/startup.rs`, then the corrected test-only
compile was run with:

```text
cargo test --test startup
```

Expected failure:

```text
error[E0432]: unresolved imports
`herdr_webmaster::persistence::effective_view`,
`herdr_webmaster::persistence::load_startup`
```

No production startup code existed at this point.

### GREEN 1: startup restoration

After the minimum startup implementation:

```text
cargo test --test startup
```

Result: 10 passed, 0 failed. This included real temporary files for absent and
valid inputs, invalid config plus valid state, future state plus valid
guestbook, malformed guestbook diagnostics, independently missing config/state
directories, disabled paths, full view precedence, and the 64-case filesystem
hostile-input property.

### RED 2: runtime settings and configurable reviewr protocol

Runtime, interaction, command-protocol, and desk-rendering tests were changed
before their production consumers:

```text
cargo test --test runtime_loop --test interaction --test command --test desk_rendering
```

Expected failures were compile-time contract failures:

```text
variant `DeskCommand::DiscoverReviewr` has no field named `qualified_id`
variant `DeskCommand::OpenReviewr` has no field named `qualified_id`
expected `View`, found `Model` for `bootstrap_model`
```

### GREEN 2: settings and protocol consumption

After implementation, the same command passed:

```text
cargo test --test runtime_loop --test interaction --test command --test desk_rendering
```

Result: 56 passed, 0 failed:

- command: 7 passed;
- desk rendering: 12 passed;
- interaction: 21 passed;
- runtime loop: 16 passed.

The protocol test uses `acme.diff.inspect` and verifies the exact
`plugin_id = "acme.diff"` and `action_id = "inspect"` request parameters. A
non-splittable `inspect` action returns unavailable on discovery and a non-fatal
invocation error without focusing or panicking.

## Implementation

- Added `RuntimeSettings` to `Model`, sourced only from validated config.
- Added `StartupData`, `effective_view`, and `load_startup`.
- Joined raw optional config, state, and guestbook reads with `tokio::join!`.
- Interpreted config first, then validated state, then replayed guestbook with
  the configured bound.
- Restored valid durable persona intent and persisted preferences; persisted
  preferences override config preferences.
- Applied explicit, persisted, configured, then built-in desk view precedence.
- Preserved optional `WorkerPaths` for Task 7 without starting a worker.
- Refactored `bootstrap_model` to mutate only connection and status on a
  restored model.
- Replaced output-preview constants in runtime and interaction paths with model
  settings.
- Omitted elapsed text cleanly when configured off.
- Carried the configured complete reviewr action through typed commands;
  discovery compares the whole string and invocation splits on the final dot.
- Kept reviewr key/copy unchanged.
- Confirmed serialized `PersistedStateV1` excludes output lines, reviewr action,
  and elapsed visibility.
- Updated the temporary terminal bridge only to pass `Model::new(initial_view)`
  into the new bootstrap signature; it still does not load startup data or own
  persistence lifecycle.

## Files

Modified:

```text
src/app.rs
src/command.rs
src/config.rs
src/interaction.rs
src/persistence/mod.rs
src/runtime_loop.rs
src/terminal.rs
src/ui/views/desk.rs
tests/command.rs
tests/desk_rendering.rs
tests/interaction.rs
tests/runtime_loop.rs
.superpowers/sdd/task-6-report.md
```

Created:

```text
src/persistence/startup.rs
tests/startup.rs
```

The pre-existing modification to `.superpowers/sdd/task-2-report.md` was left
untouched and excluded from this task's commit.

## Final verification

Formatting and the specified focused gate:

```text
cargo fmt --all
cargo test --test startup --test runtime_loop --test command --test desk_rendering
```

Result: 45 passed, 0 failed:

- startup: 10 passed;
- runtime loop: 16 passed;
- command: 7 passed;
- desk rendering: 12 passed.

Strict lint gate:

```text
cargo clippy --all-targets --all-features -- -D warnings
```

The first run identified `reduce_action` at 102 lines after configured command
construction. A behavior-neutral `load_output` helper reduced the function
without suppressing the lint. Follow-up test-only style findings were corrected.
Final result: exit 0, no warnings.

Post-change full suite:

```text
cargo test
```

Result: 293 passed, 0 failed, including all 64 startup filesystem property
cases.

Diff hygiene:

```text
git diff --check
```

Result: exit 0.

## Self-review

- Confirmed all three reads are started together and diagnostics are emitted in
  deterministic config/state/guestbook interpretation order.
- Confirmed invalid config cannot partially alter settings or guestbook bounds.
- Confirmed future/invalid state cannot seed durable intent or override config
  preferences, while valid guestbook history remains available.
- Confirmed a valid state restores durable personas before future live-domain
  overlays and never places config-only settings in `state.json`.
- Confirmed startup selection is absent or references a live agent under hostile
  inputs.
- Confirmed bootstrap changes only connection/status and preserves restored
  view/preferences/settings/durable intent/domain history by construction.
- Confirmed no hard-coded output line count or reviewr action remains in runtime
  command construction; only validated defaults remain in config/model defaults.
- Confirmed final-dot splitting supports dotted plugin IDs.
- Confirmed no terminal worker lifecycle was added early.

## Concerns

No Task 6 blockers. Task 7 still must call `load_startup`, surface diagnostics,
start `PersistenceWorker` from `StartupData.paths`, and coordinate flush/shutdown.
