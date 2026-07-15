# Task 8 report: core properties and persistence milestone gate

## Status

Task 8 product, test, and documentation changes are implemented in
`a05e067 docs: complete persistence milestone`.

The complete automated Milestone 6.1 gate passes on that commit. Live Herdr
acceptance is not passed: Herdr 0.7.3 is installed, but its server was not
running, so linking and action invocation were blocked before any interactive
state could be exercised.

## RED/GREEN evidence

### RED 1: missing core-domain property strategies

After adding the seven named properties to `tests/property_domain.rs`, the
first focused command was:

```text
cargo test --test property_domain
```

It exited `101` with thirteen expected unresolved support functions, including
`agent_identity`, `attention`, `domain_with_one_agent`, `agent_status`,
`status_event`, `topology_events`, and `guestbook_event`. This proved the new
property file was compiled before its explicit generators existed.

### GREEN 1: all seven named properties

After implementing the smallest explicit strategies and event constructors:

```text
cargo test --test property_domain
```

exited `0`; 7 passed, 0 failed. The properties cover:

- deterministic persona keys and appearances for arbitrary identity branches;
- idempotent `Attention::mark_seen`;
- duplicate semantic status events leaving state stable and emitting no second
  command batch;
- stale revisions preserving both state and an empty command list;
- valid selection after arbitrary snapshot additions, real pane exits, and
  workspace closures, while independently comparing state and commands;
- the documented site-status priority over the same agent set in both orders;
- stable guestbook IDs for equal event identity fields even when non-identity
  metadata differs.

### Shrinking and persisted regression evidence

The first 1,024-case run was:

```text
PROPTEST_CASES=1024 cargo test --test property_domain --test persisted_state
```

The persisted-state binary passed all 13 tests. The site-order property then
found a test-generator flaw after 903 successful cases: duplicate generated
`AgentKey` values meant forward and reverse map insertion retained different
agents. Proptest shrank the case to two agents with key `a-` and wrote seed
`85b96fea...6889` to
`tests/property_domain.proptest-regressions`.

The property was corrected to reverse the already deduplicated agent set. No
production code was changed for this test defect. The same 1,024-case pair then
passed with 13 persisted-state tests and 7 core-domain properties.

For the directed shrink check, the local `marking_attention_seen_is_idempotent`
assertion was temporarily inverted and the focused test exited `101`. Proptest
reported the minimal input `attention = Clear`. The original equality
assertion was restored; the focused test and the 1,024-case pair both passed.
The generated regression file was retained and committed; no empty or manually
fabricated regression file was created. `.gitignore` explicitly preserves both
`*.proptest-regressions` files and `proptest-regressions` directories.

### Retained Task 4 test hardening

The ambiguous malformed-record test now names both actual cases:
schema-invalid JSON and malformed UTF-8. The truncated-tail example now writes
one complete newline-terminated record followed by an unterminated valid JSON
record and proves the complete record replays while only line 2 is rejected as
truncated.

Focused verification:

```text
cargo test --test property_domain --test guestbook_persistence
```

exited `0`; property-domain 7 passed and guestbook-persistence 10 passed.

## Acceptance defect discovered and fixed

While grounding the recovery documentation, review identified a cross-task
contract defect: `load_startup` rejected malformed state but still passed the
same writable `state.json` destination to the worker. The first connected
snapshot emitted `PersistState`, so shutdown could replace the recovery
evidence with a new valid snapshot.

### RED 2: malformed state overwritten by initial snapshot

The integration-level regression drives `load_startup`, a real fixture-backed
`ConnectionUpdate::Connected`, the resulting `PersistState`, a guestbook append,
and bounded worker shutdown:

```text
cargo test --test persistence_worker malformed_state_survives_initial_snapshot_flush_while_guestbook_stays_writable
```

It exited `101`. The final byte assertion showed the original
`{not valid state json}` had been replaced by pretty-printed schema-v1 state.

### GREEN 2: protect invalid state without disabling guestbook

Startup now retains an explicit private `StateLoad` outcome. A missing or valid
state keeps the worker destination; an unreadable, unparseable, unsupported, or
relationship-invalid state clears only the worker's state destination for that
process lifetime. The original file is never renamed or removed, and the
guestbook destination remains active.

The focused regression then exited `0`: malformed state bytes survived exactly
and the independently appended guestbook entry replayed without diagnostics.
The startup suite also proves a future schema and an unreadable `state.json`
directory protect only state publication, while valid and missing state paths
remain writable:

```text
cargo test --test startup
```

Result: 12 passed, 0 failed.

README recovery guidance now instructs users to correct or remove `state.json`
and restart webmaster before durable snapshot writes are re-enabled.

## Documentation and contributor workflow

- `README.md` documents the complete config, bounds, all four files and their
  owners, view precedence, saved preference precedence, durable-state scope,
  corruption behavior, recovery, local-only/no-telemetry behavior, fake-agent
  restart checks, `PROPTEST_CASES`, regression files, focused commands, and
  tempfile-only test posture.
- `PLAN.md` marks only Milestone 6.1 local persistence/property testing
  complete. Release automation, recordings, final live acceptance, and idle-CPU
  work remain open under Milestone 6.2.
- `CHANGELOG.md` records config, state, guestbook, worker, property, and
  invalid-state protection behavior.
- `justfile` adds `persistence-test`, parameterized `property-test`, and
  `persistence-verify` recipes. `just` is not installed in this environment, so
  the direct commands were used; the README continues to describe `just` as
  optional.

## Exact automated gate

Final-state results:

- `cargo fmt --all --check` — exit `0`, no output.
- `cargo clippy --all-targets --all-features -- -D warnings` — exit `0`, no
  warnings. The first attempt exposed a duplicated test-module allow attribute;
  scoping the allow at each integration-test module fixed it before this fresh
  gate.
- `cargo test --all-targets --all-features` — exit `0`; every unit and
  integration binary passed, including property-domain 7, persisted-state 13,
  persistence-worker 17, startup 12, and guestbook-persistence 10.
- `PROPTEST_CASES=1024 cargo test --test property_domain --test persisted_state`
  — exit `0`; 7 and 13 passed respectively.
- `bash tests/scripts.sh` — exit `0`; `scripts: 14 passed`.
- `bash -n herdr/install.sh herdr/run.sh herdr/control.sh` — exit `0`, no output.
- `cargo build --release` — exit `0`; optimized release build completed.
- `git diff --check` — exit `0`, no output.

No automated test created a checkout-local `config.toml`, `state.json`,
`guestbook.jsonl`, or `runtime.json`.

## Live Herdr 0.7.3 acceptance

Commands and exact outcomes:

- `herdr --version` — exit `0`: `herdr 0.7.3`.
- `herdr status` — exit `0`: client 0.7.3, protocol 16; server status
  `not running`; socket `/Users/alancurrie/.config/herdr/herdr.sock`.
- `cargo build` — exit `0`.
- `herdr plugin link .` — exit `1`: `Error: Os { code: 2, kind: NotFound,
  message: "No such file or directory" }`.
- `herdr plugin action invoke opsydyn.webmaster.open` — exit `1` with the same
  missing-socket error.

The live gate is therefore environmentally blocked, not passed. Blocked, seen,
done, restarted-pane, view/preference/persona restoration, guestbook replay,
reconnect deduplication, focus/reply, and selected-output reads could not be
observed. Separately, Herdr 0.7.3's `pane report-agent` rejects synthetic
`done`; the documented procedure requires a real completion event for that
state.

## Files

- `.gitignore`
- `CHANGELOG.md`
- `PLAN.md`
- `README.md`
- `justfile`
- `src/persistence/startup.rs`
- `tests/guestbook_persistence.rs`
- `tests/persisted_state.rs`
- `tests/persistence_worker.rs`
- `tests/property_domain.rs`
- `tests/property_domain.proptest-regressions`
- `tests/startup.rs`
- `tests/support/mod.rs`
- `tests/support/strategies.rs`

## Commit and residual risks

Product/docs/test commit: `a05e067 docs: complete persistence milestone`.

The only remaining acceptance risk in Task 8 scope is the unexecuted live Herdr
loop described above. Release automation, recordings, and idle-CPU acceptance
remain intentionally open. The pre-existing unrelated modification to
`.superpowers/sdd/task-2-report.md` was preserved untouched and excluded from
the product commit.
