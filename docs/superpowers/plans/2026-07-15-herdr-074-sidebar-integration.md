# Herdr 0.7.4 sidebar integration follow-up

Status: deferred until the defect-first live smoke path remains green.

## Compatibility evidence

- `herdr --version` reports `herdr 0.7.4`.
- `herdr api snapshot` reports version `0.7.4` and protocol `16`.
- `cargo build --release` passes.
- `herdr plugin link .` succeeds without changing the plugin floor (`min_herdr_version = "0.7.3"`).
- `opsydyn.webmaster` remains enabled as a local source with all five actions: `open`, `close`, `toggle`, `desk`, and `cafe`.
- Live `open`, `desk`, `cafe`, and `close` actions succeeded with empty stderr.

## Smallest useful integration

After the defect-first implementation is released, add one narrow 0.7.4 slice:

1. Add an ambient webmaster status row to workspace and agent sidebar entries using Herdr's customizable row-layout metadata path.
2. Keep the row informational: workspace bay/variant, unread attention count, and selected agent handle are sufficient for the first pass.
3. Add an optional `webmaster.peek` popup pane only if the 0.7.4 popup-pane contract is already available in the installed server; it should preview the selected agent and link back to the full desk.
4. Do not move focus, reply, or reviewr actions into sidebar rendering. The desk remains the control surface and the café remains the in-world view.
5. Preserve the current protocol floor and make the feature additive; older compatible Herdr versions should continue to link unless the manifest needs an explicit capability gate.

## Implementation order

- Confirm exact 0.7.4 sidebar token and metadata names against the installed schema/docs.
- Add a pure formatter for workspace and agent row values.
- Add reducer/formatting tests for empty, blocked, and multiple-workspace states.
- Add one live smoke assertion that a configured row is visible and does not reintroduce the managed pane.
- Document the opt-in configuration and rollback path.

No sidebar behavior is implemented in this verification task.
