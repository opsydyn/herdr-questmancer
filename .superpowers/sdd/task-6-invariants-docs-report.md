# Task 6 report: responsive, accessibility, and animation invariants

## Scope

Task 6 adds the remaining proof coverage around the connected café scene and
updates the user-facing documentation. Existing task-6-report.md and the
task-6-gap report were preserved.

## Changes

- Added a TestBackend rendering contract for selected and disconnected agents,
  including a guard against nested webmaster/café output.
- Added a Proptest topology invariant proving each generated agent is assigned
  to exactly one visible bay seat.
- Added a Proptest managed-pane invariant covering both normalized state and a
  rendered café surface.
- Added an explicit no-motion scheduler assertion proving completion cannot
  request a future frame.
- Documented connected workspace bays, deterministic room variants, compact
  fallbacks, in-world selection, and the dedicated plain-pane manual test.
- Updated PLAN.md to carry the invariant gates and defer Herdr 0.7.4 sidebar
  verification until after the current-version release checks.

## Verification

All required current-version gates passed:

```text
cargo fmt --check                         PASS
cargo clippy --all-targets --all-features -- -D warnings  PASS
cargo test --all-targets --all-features    PASS
```

The focused new suites also passed: 22 café rendering tests, 9 property tests,
and 18 theatre/animation tests.

## Review notes

- No production code was changed.
- No generated regression artifacts or tracked test-run files were created.
- Herdr 0.7.4 remains a post-fix compatibility/sidebar verification target;
  this task keeps the plugin's current protocol boundary intact.
