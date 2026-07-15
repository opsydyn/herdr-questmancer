# Task 2 report: managed-pane exclusion

## Result

DONE

## Changes

- Added `DomainState::from_snapshot_excluding`; the existing constructor remains a compatibility wrapper.
- Added `excluded_pane` to `AppEvent::SnapshotReplaced` and threaded it through the reducer.
- Added an `adapt_update_excluding` adapter entry point while retaining the existing `adapt_update` fixture-friendly wrapper.
- Status and pane-exit events for the managed pane are now inert; unknown non-managed panes still request a snapshot.
- Runtime connection snapshots and explicit `SnapshotLoaded` results pass `Model::managed_pane_id()` through the exclusion path.

## Tests

- `cargo test --test normalization --test reducer --test event_adapter --test runtime_loop -- --nocapture` — 47 passed.
- `cargo test --test property_domain -- --nocapture` — 7 passed.
- Added normalization, reducer, adapter, and runtime coverage for exclusion and selection preservation.

Proptest emitted its existing `SourceParallel` persistence warning because integration tests do not expose a crate `lib.rs`/`main.rs`; no cases failed.

## Self-review

- Filtering is by exact pane identity at the snapshot boundary, never by display name, workspace, or status.
- Existing callers keep the old `from_snapshot` and `adapt_update` APIs.
- The reducer still preserves persona, attention, guestbook, and valid selection across resnapshot.
- No café rendering or command-guard work was included.
