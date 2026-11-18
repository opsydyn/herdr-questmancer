# ADR 0008 · One writer per Questmancer profile

## Context

The persistence worker serializes writes within a process. Two Questmancer
processes can load the same profile at startup and later publish whole state
snapshots with different changes. Atomic replacement protects individual
reads from partial files. Snapshot replacement also needs a single writer.

## Decision

Acquire a non-blocking advisory lock on a stable sibling file before loading a
configured profile. The application holds the lock through persistence shutdown.
A second launch can read the profile and view the party. It starts with profile
writes disabled and reports the reason. Restarting the second pane after the
writer exits reloads the latest state before taking the lease.

Use the existing serial worker for explicit preference saves. Confirm success
after durable publication. Continue debouncing navigation, selection and
other incidental state.

## Consequences

Profile writes stay local and share the existing file format. This avoids
merging complete snapshots from stale processes and gives caches and history
the same writer boundary. Separate profiles can still have independent writers.
The lock coordinates launches that implement this decision. Older binaries
do not participate in the lock.
