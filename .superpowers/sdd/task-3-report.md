# Task 3 report: atomic state loading and publication

## Outcome

Implemented the filesystem safety boundary for versioned `state.json`.
Malformed, unsupported, and relationship-invalid documents now fail closed
through structured path-bearing diagnostics. Successful publication serializes
the complete document before touching the filesystem and uses a synced
same-directory `state.json.tmp` followed by atomic rename on macOS/Linux, so a
reader sees either the previous complete state or the next complete state.

## RED evidence

### Load and validation boundary

The first `cargo test --test atomic_state` run exited 101 before production code
was added. The compiler reported exactly:

```text
error[E0432]: unresolved imports `herdr_webmaster::persistence::load_state`, `herdr_webmaster::persistence::parse_state`
 --> tests/atomic_state.rs:8:58
  |
8 |     persistence::{AttentionEpisodeKey, PersistedStateV1, load_state, parse_state},
  |                                                          ^^^^^^^^^^  ^^^^^^^^^^^ no `parse_state` in `persistence`
  |                                                          |
  |                                                          no `load_state` in `persistence`
```

After the minimal parser and loader were implemented, the focused suite was
green with `9 passed; 0 failed`.

### Publication boundary

Publication tests were then added before publication code. The next
`cargo test --test atomic_state` run exited 101 with:

```text
error[E0432]: unresolved import `herdr_webmaster::persistence::publish_state`
  --> tests/atomic_state.rs:10:73
   |
10 |         AttentionEpisodeKey, PersistedStateV1, load_state, parse_state, publish_state,
   |                                                                         ^^^^^^^^^^^^^ no `publish_state` in `persistence`
```

After the minimal atomic publisher was implemented, the deterministic focused
suite was green with `15 passed; 0 failed`.

The required bounded concurrent filesystem property was then added with
`ProptestConfig::with_cases(64)`. Its first focused run was green with
`16 passed; 0 failed` in 2.85 seconds. A final direct non-`NotFound` read-error
coverage case brought the completed focused suite to 17 tests.

## Changes

- Added public `parse_state`, `load_state`, and `publish_state` interfaces.
- Added structured `PersistenceDiagnostic` and `PersistenceError` values with
  operation, path, optional one-based line, and source message fields.
- Deserialization requires a `PersistedStateV1` and delegates all schema and
  relationship checking to the existing `PersistedStateV1::validate()`.
- Loading maps only `ErrorKind::NotFound` to `Ok(None)`. Parse, validation, and
  all other read failures preserve the destination and any leftover temporary
  file.
- Publication serializes pretty JSON plus one trailing newline before opening
  files, creates missing parent directories, truncates/reuses `state.json.tmp`,
  writes and `sync_all`s it, then renames it over `state.json`.
- After rename, parent-directory sync runs in `tokio::task::spawn_blocking` and
  is intentionally best effort. Failed publication best-effort removes only
  `state.json.tmp` and never removes or rewrites the prior destination.
- Added real-tempfile tests for malformed and arbitrary bytes, schema version
  2, every invalid Task 2 relationship, exact invalid-file preservation,
  formatting, replacement, missing directories, stale-temp reuse, temp-create
  failure, and rename failure in a read-only Unix directory.
- Added a 64-case property that performs four alternating synced publications
  and 32 concurrent destination reads for each generated state pair. Every read
  is parsed and must equal one of the two complete generated states.

## Files

- `src/persistence/atomic_json.rs` (new)
- `src/persistence/mod.rs`
- `tests/atomic_state.rs` (new)
- `tests/support/mod.rs`
- `.superpowers/sdd/task-3-report.md` (new)

The pre-existing uncommitted change to `.superpowers/sdd/task-2-report.md` was
not modified or staged.

## Final verification

- `cargo fmt --all` — exit 0.
- `cargo fmt --all --check` — exit 0.
- `cargo test --test atomic_state` — exit 0; 17 passed, 0 failed, 0 ignored.
- `cargo clippy --all-targets --all-features -- -D warnings` — exit 0 with no
  warnings.
- `cargo test --all-targets` — exit 0; every unit and integration test binary
  passed with 0 failures.
- `git diff --check` — exit 0.

## Self-review and concerns

- Serialization precedes parent creation and temporary-file opening, so a
  serialization failure cannot truncate either the destination or temp file.
- The temp file is in the destination directory, and rename is the only
  destination-changing operation. Failure paths remove only the temp file on a
  best-effort basis and retain the prior destination byte-for-byte.
- JSON syntax diagnostics retain `serde_json`'s one-based line; validation and
  I/O errors correctly use `None` because they have no source line.
- The read-only-parent regression is guarded by `#[cfg(unix)]` and verifies a
  true rename failure after the existing temp file has been opened and synced.
- Atomic replacement is intentionally scoped to macOS/Linux for v0.1, as
  confirmed during execution. Windows rename-over-existing support is out of
  scope and is not implied by this implementation.
- The fixed `state.json.tmp` protocol assumes publication is serialized by the
  Task 5 persistence worker. Multiple concurrent writers to the same path are
  outside this task's contract; readers are safe during the required single
  writer's alternating publications.
