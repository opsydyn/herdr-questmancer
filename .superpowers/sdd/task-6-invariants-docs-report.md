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

## Review follow-up evidence

- `tests/desk_rendering.rs::live_page_hides_nested_output_when_the_selected_pane_is_webmaster`
  injects a selected managed-pane preview containing `THE HERDR CYBERCAFE`,
  `CAFE WALL / 56K CABLE RUN`, and a nested webmaster header. The desk keeps
  its own outer title but renders none of the injected nested headers/content.
- The bay property now exercises 240x120, 80x24, 60x18, 1x1, and 0x0 layouts.
  It counts only seat-backed visible agents, proves each visible key has one
  owner, proves zero-sized surfaces expose no bays or seats, and requires all
  generated agents to be visible in the large reference surface.
- `tests/interaction.rs::unchanged_idle_room_emits_no_output_load_or_persistence_effects`
  proves an unchanged idle redraw emits neither `DeskCommand::LoadOutput` nor
  persistence commands.
- Reduced and no-motion café renders are asserted stable across clock changes
  while retaining BUILDING, HELP!, and BROKEN state labels.
- README now labels the 2026-07-14 live walkthrough historical and records the
  2026-07-15 setup-limited synthetic report that targeted an already-owned
  Codex pane and therefore never entered the snapshot.

Follow-up gates rerun after these changes:

```text
cargo fmt --check                         PASS
cargo clippy --all-targets --all-features -- -D warnings  PASS
cargo test --all-targets --all-features    PASS
```
