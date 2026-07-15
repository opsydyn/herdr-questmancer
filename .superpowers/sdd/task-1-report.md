# Task 1 report: managed pane identity

## Status

DONE

## Implementation

- Added `Model::managed_pane_id` storage and accessor/setter, defaulting to `None`.
- Added model tests covering the empty default and `PaneId` round trip.
- Updated `terminal::run` to read a non-empty `HERDR_PANE_ID` before bootstrapping the model and carry it into model state. Missing or empty environment remains valid for offline mode.
- Kept runtime registration unchanged; pane registration and domain exclusion remain separate responsibilities.

## Files

- `src/app.rs`
- `src/terminal.rs`
- `tests/app.rs`

## Tests

- `cargo test --test app managed_pane -- --nocapture` — passed (2 tests).
- `cargo test --test app --test runtime -- --nocapture` — passed (10 tests).
- `cargo fmt --check` — passed.
- `git diff --check` — passed.

## Concerns

No concerns for this slice. Task 2 still needs to consume the model accessor at the domain normalization boundary; this change intentionally does not alter café rendering or filtering yet.
