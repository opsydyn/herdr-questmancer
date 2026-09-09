# C1 — Chronicle snapshot qualification

Date: 2026-09-09. The user approved C1 after the capture design and bounded plan.
Work is inline on `main`, based on `a26386c` / published 0.1.9. This receipt covers
local implementation; it does not publish a new release or approve C2/C3.

## Result

Snapshots requested before a disconnect, overtaken by accepted live facts or
superseded by another request cannot roll back the current party. An ephemeral
coordinator owns connection epochs, request IDs, fact generations and the pane
set established by the authoritative post-subscription baseline. Reconciliation
hints and accepted presence/revision/topology changes advance freshness; local
input, clock changes and sidebar metadata do not.

Only one refresh is active with one pending request. An invalid or overtaken
response requests a fresh read; two consecutive such responses resubscribe and
establish a quiet baseline. A changed subscription pane set requires that baseline
immediately. Old responses cannot trigger recovery or settle newer requests.
Equal-revision conflicting status evidence requests reconciliation without
changing facts or adding history. Unsupported snapshot protocols and duplicate
agent identities cannot qualify a refresh.

Recovery replaces Questmancer's subscription task and channel, dropping queued
updates from the previous cycle. Snapshot reads have their own cancellation set;
counsel/focus and selected output retain separate ownership. The existing output
invalidation on connection boundaries remains in force. No shared Herdr server
is restarted, reconfigured or stopped.

Both typed snapshot purposes remain quiet in the pure reducer. No new Chronicle
kind, record format, XP award, persistence schema, polling interval or renderer is
introduced. Saved intent and persona overlay behaviour remain in place.

## Evidence

- First red regression: `a_refresh_from_before_disconnect_cannot_replace_the_new_connection_baseline` failed because an old response replaced the new party. It passes with correlation.
- Further red regressions reproduced equal-revision snapshot/status conflicts and acceptance of an unsubscribed pane. Each passes with qualification/recovery.
- Runtime tests cover coalescing, duplicate/late successes and failures, topology hints, two-response recovery, current failure ownership, managed-pane exclusion, quiet baselines and output ownership.
- Temporary Unix-socket fixtures verify one active read plus the latest pending read, snapshot-only cancellation while counsel/output finish, discarded old subscription messages and a baseline obtained after subscription acknowledgement.
- The full gate exposed the old property assumption that every duplicate produces no commands. Repeated conflicting equal revisions now preserve the entire state and request reconciliation; the updated property tests this explicitly and retains the discovered seed.

Final local verification:

| Gate | Result |
| --- | --- |
| `just verify` | 616 Rust tests / 54 runs; 28 shell tests; format, warning-denied Clippy, workflow contracts and shell syntax pass ([log](verify.log)) |
| `just property-test cases=4096` | 8 domain properties and 21 persisted-state tests pass ([log](property.log)); later audit found the count override invalid, so this does **not** prove 4,096 cases |
| `cargo build --release` | Pass ([log](release.log)) |
| `cargo package --allow-dirty --locked --offline` | Package creation and extracted-crate build pass; 209 files, 4,986,395 compressed bytes, below 10 MiB ([log](package.log)) |
| Archive contents | New `src/snapshot_refresh.rs` included; repository review receipts excluded |
| `git diff --check` | Pass |

Archive SHA-256: `687f8d3d72a2e5de67020b097ecda9c38a15489f6b018ae4d693f157a613d2f7`.
This archive is a local working-tree package, not a published release artifact.

The initial [rollback failure](red.log), its [focused green result](green.log),
[equal-revision snapshot failure](conflict-red.log),
[equal-revision status failure](status-conflict-red.log) and
[subscription-membership failure](subscription-red.log) are retained. The complete
passing gate above covers their final forms, including all boxed-event and
subscription-qualification changes.

## Property-command correction — C2 audit

The original invocation expanded to `PROPTEST_CASES=cases=4096`; `just` recipe
arguments are positional. Its successful tests used the default case count,
not the requested 4,096. The earlier C1 summary overstated that evidence.
The corrected command is `just property-test 4096`; see the
[C2 receipt](../2026-09-09-chronicle-c2/README.md) for verification of the current
combined C1/C2 source. This correction does not claim a retroactive rerun of the
C1-only checkout.

## Limits and next slice

All sockets and counsel acknowledgements above are isolated fixtures. This slice
did not operate on live Herdr panes, reinstall or relink the plugin, publish an
archive, or claim native/manual visual acceptance. The public release remains
0.1.9 at `98f557d`; this implementation remains local and uncommitted.

C2 is the recommended next bounded slice: typed zero-XP observations with
compatible Chronicle replay and readable record/chapter copy. It requires user
approval. C3 separately qualifies the complete capture/persistence path and live
observations; no such acceptance is inferred from C1.
