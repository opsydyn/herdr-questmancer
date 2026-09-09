# C2 — Typed Chronicle observations and compatible replay

Date: 2026-09-09. The user approved C2 after C1. Work is inline on `main`, based
on `a26386c` plus the uncommitted C1 changes. C2 is implemented locally; C3 and
publication remain separate approval gates.

## Result

A qualified working-to-blocked snapshot now appends one zero-XP record saying
“Codex was observed needing counsel”. Repeated snapshots, newer same-state
metadata and display-text changes add nothing. Snapshot-first spoils cannot earn
retroactive XP from matching metadata; a subsequent status-confirmed episode
retains the existing ten-XP reward. Unknown whereabouts has its own category.

Capture compares complete session/terminal evidence without changing persona
identity. Missing or ambiguous identity establishes a quiet subject baseline;
a temporarily unidentified occupant does not imply departure. The same terminal
retains its revision boundary across pane moves. Terminal replacement invalidates
old scrying ownership and the cat's same-party comparison, including replacements
whose key, pane and revision match. No counsel/focus task cancellation is added.

Workspace-close hints require snapshot confirmation. A surviving workspace stays
visible and consumes the hint; later unrelated absence does not inherit it.
Campaign removal absorbs membership losses caused by that same absence and never
awards successful-delivery XP. Unversioned pane-exit hints request reconciliation;
explicit identity and newer revision qualify a departure. Subscription pane-set
changes still require a quiet C1 baseline, so capture deliberately has gaps.

New production writes use a v2 envelope with immutable observation ID, local
observation time, captured subject and typed source/payload. The runtime supplies
a random capture-run ID; the pure reducer commits an ordinal only for an accepted
record. Raw terminal/session metadata and the capture clock remain ephemeral.
Records persist only an incarnation digest. V1 records retain their original IDs,
wording and categories; mixed replay does not rewrite history or re-award standing.
New categories cannot be written or read as flat legacy records. Unknown versions,
payloads and invalid source context produce bounded diagnostics. Published 0.1.9
readers skip v2 records with diagnostics on downgrade.

Records label local observation times. Chapters count presence, unknown-whereabouts,
membership, visibility-loss and campaign-removal categories separately from spoils
returns and legacy identity events. They retain UTC windows, recorded summaries,
source IDs and the retained-history notice. Equal-time observations follow accepted
order within a capture run.

## Evidence and verification

The [first red test](red.log) observed zero appends where one was required; its
[initial green result](green.log) proved the bounded working-to-blocked case.
Additional red regressions covered [missing identity](identity-red.log),
[same-terminal revision rollback after a move](move-red.log),
[cat reaction on terminal replacement](cat-red.log),
[scrying ownership after replacement](output-red.log) and
[equal-time chapter ordering](order-red.log). The full passing gate covers their
final forms.

The capture suite exercises duplicate/reward suppression in both arrival orders,
quiet reconnects and old results, membership and campaign evidence, ambiguity,
wall-clock rollback, history eviction, managed-pane exclusion, source identity,
restart/replay and arbitrary snapshot sequences. Persistence tests cover mixed
v1/v2 lines, unchanged legacy bytes, unknown/invalid envelopes and immutable IDs.
A renderer test checks visible local-observation wording.

| Gate | Result |
| --- | --- |
| `just verify` | 652 Rust tests / 55 runs; 28 shell tests; format, warning-denied Clippy, workflow contracts and shell syntax pass ([log](verify.log)) |
| `just property-test 4096` | Verified `PROPTEST_CASES=4096`; 29 capture tests, 21 persisted-state tests and 8 domain properties pass ([log](property.log)) |
| `cargo build --release` | Pass ([log](release.log)) |
| `cargo package --allow-dirty --locked --offline` | Package and extracted-crate build pass; 214 files, 5,004,554 compressed bytes, below 10 MiB ([log](package.log)) |
| Archive inspection | New capture modules, regression suite and manual recipe included; review receipts excluded |
| `git diff --check` and documentation links | Pass |

Archive SHA-256: `a398eca70b1b793dfc19c100e9242a008bc8296c3267354f6abc9d65dec8e10a`.
This package includes the corrected positional property command documentation.
The [invalid-argument run](property-invalid-argument.log) is retained separately
and is not counted as high-case evidence.

## Property-command correction

The earlier command `just property-test cases=4096` expanded to
`PROPTEST_CASES=cases=4096`. Its successful tests did not prove the requested case
count. The [C1 receipt](../2026-09-09-chronicle-c1/README.md) now records that limit;
the canonical command is `just property-test 4096`. The current receipt uses the
verified numeric override and includes C2's sequence/replay property in the recipe.
No retrospective rerun of the C1-only checkout is claimed.

## Limits and next slice

These are local tests and temporary-file/socket fixtures. No live Herdr panes or
shared server were operated on; the plugin was not relinked or restarted, and
nothing was committed or published. A local package is not a release artifact.
State/history are not an atomic transaction, and append failure can still lose
an observation through the existing diagnostic/acknowledgement path. Capture is
not complete upstream history or a globally exactly-once ledger.

C3 is the recommended next bounded slice, subject to approval: exercise the full
supervisor/runtime/reducer/persistence composition, review the production Chronicle
screens, and qualify isolated live observations with owned synthetic identities.
Production visual approval and real-agent completion acceptance remain unclaimed.
Use the [manual recipe](../../manual-test/chronicle-capture.md) and the existing
guarded ownership rules. Herdr 0.9 cannot synthesize explicit done.
