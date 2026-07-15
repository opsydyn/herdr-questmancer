# Task 7 report: integrated persistent runtime lifecycle

## Status

Implemented and verified runtime effects, live and offline persistence dispatch,
startup/view registration precedence, non-fatal diagnostics, and bounded
shutdown with terminal restoration ordered before diagnostic output.

## TDD evidence

### RED 1: typed runtime and interaction persistence effects

The runtime-loop and interaction assertions were added first, then:

```text
cargo test --test runtime_loop --test interaction
```

failed at the missing public contract:

```text
error[E0609]: no field `persistence` on type `Vec<DeskCommand>`
error[E0609]: no field `desk` on type `Vec<DeskCommand>`
error[E0609]: no field `persistence` on type `ActionReduction`
```

The tests cover one blocked-transition append plus state stage, explicit
duplicate and stale status no-ops, snapshot persistence after durable overlay,
view and distinct-persona selection changes, mark-seen, repeated actions, and
animation-only redraws.

### GREEN 1: reducer effects retained end to end

After adding `RuntimeEffects` and the action persistence channel, the same gate
passed:

```text
interaction: 22 passed, 0 failed
runtime_loop: 19 passed, 0 failed
```

`RequestSnapshot` is translated to `DeskCommand::RefreshSnapshot`; all other
domain commands remain typed persistence effects. Explicit wire revisions are
honoured so duplicate and stale status updates reach the existing reducer
revision guards instead of being synthesized as newer events.

### RED 2: default shell actions and optional terminal view

The shell and terminal contract tests were changed before their implementations:

```text
bash tests/scripts.sh
FAIL: default runner received: ui --view default

cargo test --test runtime_loop
error[E0308]: mismatched types
expected `View`, found `Option<_>`
```

### GREEN 2: action and registration precedence

After implementation:

```text
bash tests/scripts.sh
scripts: 12 passed

cargo test --test runtime
2 passed, 0 failed
```

Ordinary `open` and a closed `toggle` omit `WEBMASTER_INITIAL_VIEW`. Explicit
`desk` and `cafe` retain it and switch an existing pane. `run.sh` maps only exact
`desk`/`cafe` values to `--view`. `terminal::run` now receives `Option<View>`,
loads startup first, and registers the resolved model view; `main.rs` no longer
owns a temporary default-view registration bridge.

### RED 3: ordered runtime writer dispatch

A real-worker tempfile test was added first:

```text
cargo test --test persistence_worker runtime_dispatch_durably_appends_before_staging_the_following_state
```

failed with:

```text
error[E0432]: unresolved import
`herdr_webmaster::runtime_loop::dispatch_persistence_effects`
```

### GREEN 3: acknowledged append before following state stage

After the minimal dispatcher:

```text
runtime_dispatch_durably_appends_before_staging_the_following_state ... ok
1 passed, 0 failed
```

The dispatcher consumes the batch in order, awaits every JSONL append
acknowledgement, and captures the current model only when it reaches a
`PersistState` effect.

### RED 4: bounded runtime shutdown seam

The paused-time durability test was added with the shutdown implementation
removed, then:

```text
cargo test --test persistence_worker bounded_runtime_shutdown_flushes_latest_state_after_prior_append
```

failed because `shutdown_persistence` did not exist at the terminal call sites.

### GREEN 4: durable and non-stranding shutdown

After implementing the one-second bound:

```text
bounded_runtime_shutdown_flushes_latest_state_after_prior_append ... ok
bounded_runtime_shutdown_returns_filesystem_failure_after_worker_exit ... ok
terminal::tests::signal_shutdown_flushes_state_after_an_acknowledged_append ... ok
```

The real worker tests prove that a prior acknowledged append and the latest
dirty state are durable on shutdown, and that a filesystem failure is returned
after the worker exits. The signal test raises `SIGHUP`, observes it through the
runtime signal boundary, then verifies the same durability contract.

### Review RED/GREEN: offline selection, lifecycle seams, registration owner

Pre-commit review identified three gaps. Each received a failing regression
before its fix.

Offline restored selection:

```text
cargo test --test persistence_worker offline_view_change_preserves_restored_selection_for_reconnect
left: None
right: Some(PersonaKey("remembered-agent"))
```

`PersistedStateV1::capture` now falls back to remembered durable selection when
there is no live domain selection. The startup → offline view change → writer
shutdown regression now reloads the remembered persona.

Typed production lifecycle:

```text
cargo test --test persistence_worker quit_action_dispatches_prior_local_intent_before_bounded_shutdown
error[E0432]: unresolved imports `RuntimeExit`, `dispatch_action_effects`,
`complete_runtime_exit`
```

Both terminal loops now use `dispatch_action_effects`, return typed
`RuntimeExit::{Quit, Signal, InputClosed}`, and route normal exits through
`complete_runtime_exit`. The quit test drives `Switch(Cafe)` then `Quit` through
that production seam and proves the state is durable. The real `SIGHUP` test
returns `RuntimeExit::Signal` through the same bounded completion seam.

Runtime registration ownership:

```text
bash tests/scripts.sh
FAIL: runtime.json did not contain: "initial_view":"cafe"
```

The controller now preserves an app-published same-pane runtime registration,
so its fallback singleton write cannot overwrite the terminal's resolved view.
The shell fake regression passes with the app publishing Cafe during pane open.

### Blocking review follow-up: schema, publication, and lifecycle ownership

The independent blocking review identified three additional production gaps.
Each fix was again driven by a focused failing regression.

Revision-less Herdr 0.7.3 status fixture:

```text
cargo test --test event_adapter revisionless_duplicate_status_and_custom_status_are_inert
assertion failed: actions.is_empty()
```

`adapt_agent_status` now distinguishes `Explicit` and `Synthetic` revisions.
An exact revision-less status/custom-status duplicate is discarded before a
synthetic revision is assigned. A meaningfully different revision-less event
still receives the next synthetic revision; the implementation does not claim
that revision-less stale ordering can be inferred. Explicit stale revisions are
preserved for the domain reducer's monotonic guard.

```text
cargo test --test event_adapter
event_adapter: 9 passed, 0 failed
```

Atomic no-clobber fallback publication:

```text
bash tests/scripts.sh
FAIL: runtime file did not contain: "initial_view":"cafe"
```

The fallback now writes a complete temporary document and uses a hard link to
publish only when `runtime.json` is absent. If the application publishes first,
the link fails atomically and the application-owned resolved registration is
retained.

```text
bash tests/scripts.sh
scripts: 13 passed
```

Production lifecycle coordinator:

```text
cargo test --test persistence_worker quit_lifecycle_stops_real_runtime_then_flushes_writer
error[E0432]: unresolved import `herdr_webmaster::terminal::RuntimeLifecycle`
```

`RuntimeLifecycle`, which is used directly by `terminal::run`, owns the optional
real `RuntimeConnection`, persistence client, and writer task. Completion first
awaits Herdr runtime shutdown and then always performs bounded writer shutdown.
The quit and real `SIGHUP` regressions start a real connection/supervisor against
a missing temporary socket, schedule runtime work, and verify durable writer
completion through that same coordinator.

```text
quit_lifecycle_stops_real_runtime_then_flushes_writer ... ok
terminal::tests::signal_shutdown_flushes_state_after_an_acknowledged_append ... ok
```

The first strict Clippy pass over the coordinator reported two
`clippy::let_and_return` findings in the live/offline branches. Removing the
redundant bindings left behavior unchanged; the subsequent warnings-denied run
completed cleanly.

Focused re-review found that the atomic link failure path treated every failure
as a publication race. A fake `ln` that fails without creating the destination
first produced:

```text
bash tests/scripts.sh
FAIL: control succeeded after fallback publication failed
```

The controller now accepts a failed link only when `runtime.json` exists. A
genuine publication failure removes the temporary file, reports a clear error,
and fails the action.

```text
bash tests/scripts.sh
scripts: 14 passed
```

## Implementation

- Added `RuntimeEffects { desk, persistence }` for connection and command-result
  reducers, and retained persistence effects in `ActionReduction`.
- Compared captured durable state before and after local actions so only real
  view, selection, seen-attention, or future preference mutations stage state.
  Modal, region, repeated, no-op, and clock/redraw changes remain write-free.
- Added ordered persistence dispatch shared by live and offline loops. Offline
  local intent persists whenever worker state paths are enabled.
- Added the worker diagnostic receiver to both loop `tokio::select!` sets.
  Diagnostics update model status without terminating the loop, are collected
  and deduplicated, and print only after terminal restoration.
- Loaded config/state/guestbook before raw mode, resolved the effective view,
  registered that view, then started the writer.
- Removed the temporary `main.rs` bare-UI-to-desk registration bridge.
- On every loop exit or error, captured the loop result, stopped Herdr-owned
  tasks, bounded persistence shutdown and worker join to one second, drained
  diagnostics, explicitly dropped `TerminalGuard`, printed diagnostics, and
  only then returned the precedence-combined result.
- On persistence timeout the worker task is aborted; on persistence filesystem
  error, cleanup and terminal restoration still run before the error returns.

## Files

Modified:

```text
herdr/control.sh
herdr/run.sh
src/herdr/event_adapter.rs
src/interaction.rs
src/main.rs
src/persistence/state.rs
src/runtime.rs
src/runtime_loop.rs
src/terminal.rs
tests/interaction.rs
tests/event_adapter.rs
tests/persistence_worker.rs
tests/runtime.rs
tests/runtime_loop.rs
tests/scripts.sh
```

Created:

```text
.superpowers/sdd/task-7-report.md
```

The pre-existing `.superpowers/sdd/task-2-report.md` modification was left
untouched and excluded from this task's commit.

## Final verification

Required focused gate:

```text
cargo fmt --all
cargo test --test event_adapter --test interaction --test runtime --test runtime_loop --test persistence_worker
bash tests/scripts.sh
bash -n herdr/run.sh herdr/control.sh
cargo clippy --all-targets --all-features -- -D warnings
```

Results:

```text
interaction: 22 passed
event_adapter: 9 passed
runtime: 2 passed
runtime_loop: 19 passed
persistence_worker: 16 passed
scripts: 14 passed
shell syntax: exit 0
clippy: exit 0, no warnings
```

The first strict Clippy run found the action reducer at 108 lines and a
single-pattern shutdown match. A behavior-neutral reduction-finalizer extraction
and `if let` timeout branch resolved those findings. The final strict Clippy run
finished cleanly.

Full milestone regression gate:

```text
cargo test --all-targets --all-features
```

Result: 306 passed, 0 failed.

Hygiene:

```text
cargo fmt --all -- --check
git diff --check
```

Result: both exited 0.

## Self-review

- Confirmed each reducer path either retains the exact domain persistence
  command or translates only `RequestSnapshot` to a desk refresh.
- Confirmed blocked history is appended once and its following state capture is
  not staged until the append acknowledgement completes.
- Confirmed status events with explicit duplicate/stale revisions remain inert.
- Confirmed exact revision-less status/custom-status duplicates remain inert,
  while changed revision-less events make no unsupported stale-order claim.
- Confirmed `PersistedStateV1::capture` is not called by animation ticks and
  unchanged captures do not emit `PersistState`.
- Confirmed both loops dispatch local persistence effects and both surface
  asynchronous writer diagnostics without exiting.
- Confirmed offline capture retains a restored selected persona until a live
  reconnect can overlay it, including across an intervening view write.
- Confirmed quit and signal are typed production exits that enter bounded
  persistence completion only after action persistence dispatch, and that the
  production coordinator stops real Herdr-owned tasks before the writer.
- Confirmed the control fallback publishes with an atomic no-clobber operation
  and cannot replace a registration published concurrently by the terminal;
  non-race publication failures are reported instead of silently accepted.
- Confirmed no `?` or early return exists while `TerminalGuard` is owned after
  entering raw mode. All fallible loop and shutdown outcomes are values until
  after explicit guard drop and diagnostic emission.
- Confirmed terminal-entry failure also shuts down the already-started writer;
  `TerminalGuard::enter` restores via its local guard if setup fails after raw
  mode is enabled.
- Confirmed loop error precedence is retained over Herdr shutdown error, which
  is retained over persistence shutdown error, while all cleanup steps execute.

## Concerns

Direct cursor/raw-mode restoration is not asserted through a pseudo-terminal in
the automated suite. The guarantee is structurally enforced by `TerminalGuard`
RAII plus the explicit no-early-return shutdown sequence, and the persistence
failure regression exercises the fallible shutdown seam. No Task 7 blocker
remains.

## Pre-commit review

The initial review found the offline selection loss, lifecycle seam gap, and
registration race documented above. After their RED/GREEN fixes and full
reverification, focused re-review reported no remaining Critical or Important
issues and a ready-to-merge verdict.

The blocking follow-up review approved the schema-grounded status handling and
production lifecycle coordinator, then identified the genuine hard-link failure
case. After the recorded RED/GREEN fix, the final blocker-only pass confirmed
the issue resolved with no remaining blocker and a ready-to-commit verdict.
