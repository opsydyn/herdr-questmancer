# Task 5 Report: Debounced single-writer persistence worker

## Status

Implemented the single Tokio persistence actor and its typed client protocol.
State writes are deduplicated and debounced from the latest distinct stage by
exactly 250 milliseconds; guestbook append, flush, and shutdown calls
acknowledge completion; filesystem failures emit bounded diagnostics without
terminating the worker.

## RED evidence

### Missing worker boundary

Command:

```text
cargo test --test persistence_worker
```

Exit `101` with the expected missing-feature error:

```text
error[E0432]: unresolved imports `herdr_webmaster::persistence::PersistenceWorker`,
`herdr_webmaster::persistence::WorkerPaths`
```

The test target contained only debounce and unchanged-projection examples at
this boundary. A prior run also exposed an invalid test assertion comparing
`io::Result<SystemTime>` directly; that test defect was corrected and RED was
rerun until the only failure was the missing worker API above.

### Missing acknowledgement operations

After debounce and deduplication were GREEN, the shutdown, disabled-path,
append, flush, and error examples were added. The next command:

```text
cargo test --test persistence_worker
```

exited `101` with seven `E0599` errors because `PersistenceClient::flush` and
`PersistenceClient::append_guestbook` did not exist. This was the expected
second RED boundary before implementing acknowledged operations.

### Resettable deadline proof

Self-review found that staging three values at the same virtual instant did
not prove that a later distinct stage resets the deadline. The unproven reset
branch was removed, and a spaced-stage example was added. This command:

```text
cargo test --test persistence_worker \
  a_later_distinct_state_resets_the_debounce_deadline -- --exact
```

exited `101` with:

```text
state.json was published too early
```

Restoring `Sleep::reset` to 250 milliseconds from the latest distinct message
made the same exact test GREEN.

Paused-time setup is deliberate: each worker is yielded once immediately after
`start` so it reaches a pending receive, and once after staging so it drains
the ready messages and arms the timer before `advance`. The test asserts no
destination at 249 milliseconds of virtual time. After the exact one
millisecond boundary, one bounded `spawn_blocking` observer waits up to one
second of wall time only for real tempfile I/O to complete. The spaced-stage
test uses the same bounded observer while virtual time remains paused to prove
the earlier deadline did not publish. No flush or test-only production hook is
used in either debounce assertion path.

## GREEN evidence

The first production slice made the two debounce/deduplication examples pass.
The second slice made all eight acknowledgement, disabled-path, error, and
bounded-diagnostic examples pass. After the explicit reset RED/GREEN cycle,
the final focused command was:

```text
cargo test --test persistence_worker
```

Result: exit `0`; 9 passed, 0 failed.

Clippy initially identified two `manual_inspect` warnings in diagnostic side
effects. Replacing behavior-preserving `map_err` calls with `inspect_err` made
the required lint command clean.

## Implementation

- Added the exact `PersistenceMessage` variants for staging state, appending a
  guestbook entry, flushing, and shutting down with oneshot acknowledgements.
- Added public `WorkerPaths`, `PersistenceWorker`, and `PersistenceClient`
  exports from `persistence`.
- Used one unbounded command channel for infallible synchronous state staging
  and one bounded diagnostic channel with capacity 16. Diagnostic emission is
  non-blocking, so a saturated UI queue cannot stall persistence or shutdown.
- Kept deduplication at the client boundary through `last_staged`; unchanged
  projections return `false` and send no actor message.
- Kept one optional dirty projection and one optional resettable Tokio sleep.
  Every distinct staged state replaces the dirty projection and resets the
  deadline to 250 milliseconds from that message. Successful or failed
  publication clears the attempted projection and disarms the timer.
- Serialized state publication and guestbook append effects in actor message
  order. Guestbook acknowledgement occurs after the existing `sync_data` call.
- Made `flush` publish dirty state immediately, cancel its pending debounce,
  and acknowledge the actual result.
- Made `shutdown` process after earlier FIFO messages, publish the latest dirty
  state, acknowledge once, and terminate even when publication returns an
  error.
- Made `None` state and guestbook paths successful no-ops.
- Converted every persistence error into the existing diagnostic shape with
  the same operation, path, optional line, and source message. Failures are
  reported without terminating the actor.
- Cleared failed dirty state while retaining the client's last staged
  projection, so the same failed projection is deduplicated and only a later
  distinct state triggers another attempt.

## Files

- `.superpowers/sdd/task-5-report.md`
- `src/persistence/mod.rs`
- `src/persistence/worker.rs`
- `tests/persistence_worker.rs`

## Verification

Run from
`/Users/alancurrie/Projects/herdr-web-master/.worktrees/persistence`:

- `cargo fmt --all` - exit `0`.
- `cargo test --test persistence_worker` - exit `0`; 9 passed, 0 failed.
- `cargo clippy --all-targets --all-features -- -D warnings` - exit `0`.
- `cargo test --all-targets --all-features` - exit `0`; all 278 tests passed.
- `git diff --check` - exit `0`.

## Self-review

- The fixed `state.json.tmp` name now has exactly one production writer: only
  the actor calls `publish_state` for staged runtime state.
- `stage_state` updates `last_staged` only after a successful channel send.
- Actor FIFO ordering ensures a shutdown message cannot acknowledge before any
  previously sent append or flush message completes.
- Timer publication, flush, and shutdown all take the dirty projection before
  attempting I/O. A failed write therefore cannot become a repeating retry;
  the worker remains available for later distinct state and append messages.
- Flush disarms its pending sleep on both success and failure. A clean worker
  has no armed timer and no periodic wakeup.
- A disabled state path still clears attempted dirty state successfully; a
  disabled guestbook path acknowledges without touching disk.
- A full diagnostic receiver never backpressures the actor. Excess diagnostics
  are dropped by `try_send`, keeping memory and shutdown latency bounded.
- Channel closure without an explicit shutdown only terminates the actor; it
  does not invent implicit durability semantics. Runtime integration must use
  the acknowledged `shutdown` operation defined by this task.
- The pre-existing modification to `.superpowers/sdd/task-2-report.md` was not
  changed or included in this task's commit.

## Concerns

No unresolved concern within Task 5 scope. Task 7 must retain the client long
enough to request and await `shutdown`; simply dropping the client deliberately
does not flush dirty state.
