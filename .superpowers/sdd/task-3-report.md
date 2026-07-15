# Task 3 report: guard command effects and repair manual test path

## Status

Complete.

## Changes

- `CommandExecutor` now receives the managed pane identity and refuses focus,
  reply, output-read, and Reviewr-open effects targeting webmaster's pane.
  Refusals return the stable non-fatal message `refused operation on webmaster
  pane` before touching the Herdr client.
- `RuntimeConnection` passes `HERDR_PANE_ID` into the command executor.
- Interaction reduction treats the managed pane as non-actionable, preventing
  visit, refresh, and reply effects from being emitted for it.
- README and the user-quick-start design now require a dedicated unowned plain
  pane for synthetic reports, use a unique source ID, and release the source
  during cleanup. They explicitly explain why an existing agent or webmaster
  pane is invalid for this test.

## Verification

- `cargo test --test command --test interaction -- --nocapture`: 31 passed.
- `cargo test --all-targets --all-features`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- Documentation search for `dedicated`, `plain pane`, `report-agent`, and
  `webmaster-smoke`: passed; manual path references a dedicated pane.

## Concerns

- The command executor constructor now requires an explicit
  `Option<PaneId>`; all in-repository call sites were updated. External users
  should provide `None` for offline or non-plugin use.

## Follow-up review fix

The interaction review found that navigation compared raw selected panes before
and after `First`, `Last`, `Next`, and `Previous`, bypassing the managed-pane
guard. `select_agent` now uses the same `selected_pane` predicate as all other
effects, so selecting webmaster's pane cannot schedule `LoadOutput`.

Added coverage for transitions into a managed pane and a model containing only
the managed pane. Focused verification after the fix:

- `cargo test --test interaction --test command -- --nocapture`: 33 passed,
  0 failed.
