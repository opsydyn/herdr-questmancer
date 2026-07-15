# Final Review Fixes Report

## Status

Implemented all final whole-branch review findings and committed the product,
test, and README changes as:

```text
e8537ed10b3caa1c0a4607fbcdd9445de0c5a6fd fix: preserve guestbook append durability
```

The accidentally overwritten `.superpowers/sdd/task-2-report.md` was restored
exactly to its committed `HEAD` content. It has no diff and is not repurposed
as a persistence report.

## Changes

- Guestbook append now inspects the existing file's final byte through an
  explicit read handle before opening the append handle. A non-empty file that
  does not end in `\n` receives exactly one separator newline before the new
  compact record and its terminating newline are written and synced.
- Worker diagnostics now use a bounded newest-retaining queue. Publishing is
  non-blocking; saturation evicts the oldest diagnostic and retains the newest
  for asynchronous UI consumption.
- The topology property now checks independent reducer command semantics for
  snapshots, real and repeated pane exits, real and repeated workspace
  closures, and unknown-identity no-ops. It continues to prove selection is
  absent or references a live agent after every event.
- README footer availability now refers to the fully qualified action supplied
  by `reviewr_action`, rather than hardcoding the default action.

## TDD evidence

### Truncated-tail acknowledged append

RED command:

```text
cargo test --test persistence_worker acknowledged_append_survives_restart_after_a_truncated_guestbook_tail -- --exact
```

RED result: exit `101`. Replay contained only `prior-complete`; the expected
acknowledged `acknowledged-after-truncation` entry was missing because it had
been concatenated onto the damaged final line.

GREEN commands:

```text
cargo test --test persistence_worker acknowledged_append_survives_restart_after_a_truncated_guestbook_tail -- --exact
cargo test --test guestbook_persistence
```

GREEN result: exit `0`; the focused restart regression passed and all 10
guestbook persistence tests passed.

### Newest-retaining bounded diagnostics

RED command:

```text
cargo test --test persistence_worker saturated_diagnostic_queue_retains_the_latest_failure -- --exact
```

RED result: exit `101`. After saturation, the distinct latest `open guestbook`
failure was absent from the retained diagnostics.

GREEN commands:

```text
cargo test --test persistence_worker saturated_diagnostic_queue_retains_the_latest_failure -- --exact
cargo test --test persistence_worker diagnostic_queue_remains_bounded_under_repeated_failures -- --exact
cargo check --all-targets --all-features
```

GREEN result: exit `0`; the newest failure was the last retained diagnostic,
the queue remained at its configured capacity, and every target compiled.

### Topology command semantics property

RED command:

```text
cargo test --test property_domain arbitrary_topology_changes_keep_selection_valid -- --exact
```

RED result: exit `101` because the newly required independent
`assert_topology_commands` oracle did not exist. This removed the prior
identical-invocation command comparison before the oracle and expanded event
strategy were implemented.

GREEN command:

```text
cargo test --test property_domain arbitrary_topology_changes_keep_selection_valid -- --exact
```

GREEN result: exit `0`; 1 property passed with generated snapshot transitions,
real and no-op pane exits, real and no-op workspace closures, command-class
checks, and selection validity.

## Final verification

The final gate was rerun after the last test refinement as one ordered command:

```text
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
PROPTEST_CASES=1024 cargo test --test property_domain --test persisted_state
bash tests/scripts.sh
bash -n herdr/install.sh herdr/run.sh herdr/control.sh
cargo build --release
git diff --check
```

Every command exited `0`:

- formatting check: clean;
- Clippy: all targets and features clean with warnings denied;
- all-target/all-feature tests: every test binary passed, including 19
  persistence-worker tests and 7 domain properties;
- 1,024-case property stress: 13 persisted-state tests and 7 domain properties
  passed;
- script suite: `scripts: 14 passed`;
- shell syntax checks: clean;
- release build: optimized build completed;
- diff whitespace check: clean.

## Files

- `README.md`
- `src/persistence/guestbook_jsonl.rs`
- `src/persistence/mod.rs`
- `src/persistence/worker.rs`
- `src/terminal.rs`
- `tests/persistence_worker.rs`
- `tests/property_domain.rs`
- `tests/support/strategies.rs`
- `.superpowers/sdd/final-review-fixes-report.md` (this separate report)

## Residual risks and boundaries

- Tail inspection and append are intentionally two handle operations under the
  milestone's single-writer contract. Concurrent out-of-band writers remain
  outside the supported ownership model.
- The diagnostic queue uses a short standard-library mutex only around bounded
  in-memory queue operations; no filesystem acknowledgement or worker task
  waits for UI consumption.
- No release automation was changed.
- No live Herdr acceptance was run or claimed; verification is limited to the
  repository's automated tests, shell checks, and release build.
