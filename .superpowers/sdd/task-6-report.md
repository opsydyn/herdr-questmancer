# Task 6 report: adaptive animation scheduling and Milestone 5 verification

## Outcome

- Replaced both fixed one-second render intervals with one
  `AnimationScheduler` backed by a resettable Tokio sleep.
- The scheduler stores no sleep when the complete visible model has no future
  frame boundary and therefore cannot wake on elapsed time. Animated cafes arm
  only the earliest phase-aware boundary across all agents.
- Cadence is re-derived after every draw following input, runtime, or clock
  events. Runtime and input handling synchronize `Model::now` before reduction,
  so a newly observed done transition schedules immediately.
- Replaced the separate 50 ms shutdown flag poll with event-driven Tokio Unix
  signal streams for `SIGINT`, `SIGTERM`, and `SIGHUP`. Static desks and
  no-motion cafes now retain no periodic application timer.
- Kept animation purely visual: timer wakes update only the injected model clock
  and do not issue Herdr commands, output reads, or persistence writes.
- Documented the responsive cafe, controls, state language, display preference
  model, original-art boundary, focused recipes, and exact verification gate.
  Documentation explicitly says preference persistence is Milestone 6 work.

## TDD evidence

### Reviewer regression: phase-aware deadlines

An independent review found that the initial nominal-period reset could drift
from presence/attention phases. For example, resetting at completion +999 ms
would schedule another 125 ms instead of the exact +1 ms transition end, and a
mixed 6/8 fps room could miss interleaved boundaries.

The regression tests were written first and produced the expected RED errors:

```text
error[E0425]: cannot find function `next_visible_frame_in`
error[E0599]: no method named `reset_for` found for struct `AnimationScheduler`
```

The GREEN implementation now derives each agent's next semantic frame boundary
from its presence or attention timestamp and selects the minimum across the
complete cafe model. Paused-time tests drive scheduler wait/reset and model time
through all of these cases:

- reset at done +999 ms wakes after 1 ms and is stable at +1,000 ms;
- prolonged 6 fps work follows 167, 334, 500, 667, 834, and 1,000 ms boundaries
  without drift or skipped semantic frames;
- mixed 6/8 fps work follows every interleaved earliest boundary, including the
  shared 500 and 1,000 ms boundaries;
- no-motion remains pending after 24 hours.

### RED 1: scheduler API

Command:

```text
cargo test --test runtime_loop --test cafe_rendering --test theatre
```

Observed expected failure:

```text
error[E0432]: unresolved import `herdr_webmaster::terminal::AnimationScheduler`
```

This established the missing scheduler boundary before implementation.

### GREEN 1: scheduler and exact transition

The focused suite passed with:

- 13 runtime-loop tests;
- 16 cafe-rendering tests;
- 16 theatre tests at that point.

It initially proved nominal 8, 6, 2, and 1 fps periods plus event-driven waiting.
The reviewer regression above supersedes nominal reset behavior with exact
phase-aware scheduling. The cafe projection still renders exactly one confetti
marker for each of frames 1 through 8, then no confetti and a stable
`UPDATE READY` badge at exactly 1,000 ms.

### RED 2: cadence-owned frame period

Command:

```text
cargo test --test theatre render_cadence_exposes_only_the_next_visible_frame_delay
```

Observed expected failure:

```text
error[E0599]: no method named `frame_period` found for enum `RenderCadence`
```

### GREEN 2

The original completed focused gate passed 13 runtime-loop, 16 cafe-rendering,
and 17 theatre tests. After review, `next_visible_frame_in(&Model)` owns the
phase calculation and the terminal remains only the effectful timer owner.

## Final verification

Passed:

```text
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
bash tests/scripts.sh                         # scripts: 9 passed
bash -n herdr/install.sh herdr/run.sh herdr/control.sh
cargo build --release
git diff --check
```

The post-refactor full Rust test run passed all targets, including the real
event-driven `SIGHUP` shutdown test. The release binary built successfully.

After the phase-aware scheduling fix, the same complete gate was rerun and
passed again. The focused runtime-loop suite now contains 14 tests, including
the done +999 ms, prolonged 6 fps, mixed 6/8 fps, and 24-hour event-driven
regressions.

`just milestone5-verify` could not run because `just` is not installed in this
environment (`zsh: command not found: just`); every command in that recipe was
run directly as listed above.

## Live Herdr handoff

Per the task boundary, this worker did not start or mutate a live Herdr server.
The installed client is ready and confirmed as:

```text
herdr 0.7.3
protocol: 16
server status: not running
```

Exact smoke sequence for the parent task:

```bash
herdr server
cargo build
herdr plugin link .
herdr plugin action invoke opsydyn.webmaster.cafe
```

Then publish a blocked agent with the README's `pane report-agent` command and
confirm `HELP!`, raised-hand pose, selection, visit, reply, seen, search, and
refresh at 80x24. The unchanged no-motion no-wake property is deterministic and
covered with paused Tokio time; the runtime currently exposes the documented
default display preferences and does not yet persist or offer preference
configuration.
