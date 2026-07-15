# Task 4 report: byte-safe guestbook replay and append

## Outcome

Implemented tolerant byte-oriented replay and synced append-only publication for
`guestbook.jsonl`. Replay preserves valid history around damaged records and
routes every accepted entry through the existing `Guestbook::append` boundary,
so deterministic-ID deduplication, chronological ordering, and configured
retention remain the domain object's responsibility.

## TDD evidence

### Replay RED

Command:

```text
cargo test --test guestbook_persistence
```

Result: exit `101`. The compiler reported the intended missing API boundary:

```text
error[E0432]: unresolved imports `herdr_webmaster::persistence::load_guestbook`, `herdr_webmaster::persistence::replay_guestbook`
 --> tests/guestbook_persistence.rs:5:19
  |
5 |     persistence::{load_guestbook, replay_guestbook},
  |                   ^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^ no `replay_guestbook` in `persistence`
  |                   |
  |                   no `load_guestbook` in `persistence`
```

After the minimal byte replay and missing-file loader were implemented, the
same command exited `0`: 5 passed, 0 failed.

### Append RED

The real-tempfile append tests were added only after replay was green.

Command:

```text
cargo test --test guestbook_persistence
```

Result: exit `101`. The compiler reported:

```text
error[E0432]: unresolved import `herdr_webmaster::persistence::append_guestbook`
 --> tests/guestbook_persistence.rs:5:19
  |
5 |     persistence::{append_guestbook, load_guestbook, replay_guestbook},
  |                   ^^^^^^^^^^^^^^^^ no `append_guestbook` in `persistence`
```

After implementing compact serialization, parent creation, one append handle,
`write_all`, and `sync_data`, the same command exited `0`: 9 passed, 0 failed.

### Pure interleaving property

The final property mixes valid entries, explicit duplicate records, and hostile
byte records under random bounds `1..100`, then compares replay to an
independent fold through `Guestbook::append`. It also asserts unique IDs,
chronological ordering, and the bound directly. No reduced case count is set,
so Proptest uses its default 256 cases.

The first focused invocation exposed a test-only `prop_assert!` formatting
limitation around a closure block and exited `101`; moving the expression into a
named boolean corrected the test harness without changing production code.

Command:

```text
cargo test --test guestbook_persistence arbitrary_record_interleavings_match_a_guestbook_fold
```

Final result: exit `0`; 1 passed, 0 failed, 9 filtered out, in 0.27 seconds.

## Changes

- Added `ReplayResult`, `replay_guestbook`, `load_guestbook`, and
  `append_guestbook` to the public persistence surface.
- Replay splits the original byte slice on `b'\n'`; it never decodes the full
  file or calls `str::lines`.
- Every complete line is decoded as UTF-8 independently and then deserialized
  as one `GuestbookEntry`. A malformed line contributes a one-based diagnostic
  and cannot hide later valid records.
- A non-empty unterminated final slice is rejected as truncated, even when its
  bytes would otherwise form valid JSON.
- At most five individual record diagnostics are retained. Further rejected
  records are represented by one line-less summary containing the omitted
  count.
- Missing files load as an empty bounded guestbook without diagnostics; other
  read failures return an empty bounded guestbook plus a structured path-bearing
  diagnostic.
- Append serializes one compact entry plus exactly one newline before opening
  the file, creates missing parent directories, opens one create/append handle,
  writes all bytes, and calls `sync_data` before acknowledging success.
- Atomic state publication was not changed.

## Files

- `src/persistence/guestbook_jsonl.rs` (new)
- `src/persistence/mod.rs`
- `tests/guestbook_persistence.rs` (new)
- `.superpowers/sdd/task-4-report.md`

`src/domain/guestbook.rs` required no change: replay deliberately reuses the
existing `Guestbook::new` and `Guestbook::append` semantics. The unrelated
pre-existing modification to `.superpowers/sdd/task-2-report.md` was not edited
or staged.

## Final verification

- `cargo fmt --all` — exit 0.
- `git diff --check` — exit 0.
- `cargo test --test guestbook --test guestbook_persistence` — exit 0;
  guestbook 3 passed and guestbook persistence 10 passed, with 0 failures.
- The first `cargo clippy --all-targets --all-features -- -D warnings` found one
  test-only `clippy::naive_bytecount` warning in a redundant newline-count
  assertion and exited 101. The exact-byte assertion already proves the
  serialized record plus one newline, so the redundant count was removed.
- Fresh `cargo clippy --all-targets --all-features -- -D warnings` — exit 0,
  no warnings.
- `cargo test --all-targets` — exit 0; every unit and integration test binary
  passed, including all 10 guestbook persistence tests and the default-256 pure
  replay property.

## Self-review and concerns

- Verified replay operates on real byte slices and malformed UTF-8 cannot make
  the whole history undecodable.
- Verified valid records on both sides of malformed JSON and malformed UTF-8
  survive replay with their original one-based line diagnostics.
- Verified an otherwise valid final JSON record is rejected when its newline is
  absent.
- Verified duplicate IDs, out-of-order timestamps, and retention eviction are
  not reimplemented in persistence and match a direct `Guestbook::append` fold.
- Verified append tests inspect real tempfile bytes, missing parent creation,
  sequential order, and a real path-bearing open failure.
- One-writer serialization remains Task 5's responsibility, as required by the
  milestone design; this append primitive intentionally adds no locking or
  retry after ambiguous I/O failure.
- No unresolved concern remains within Task 4 scope.
