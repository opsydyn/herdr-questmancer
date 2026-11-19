# Verified owned quest association evidence

- **result** · complete
- **claim** · L2d adds scoped, verified Observe from an explicitly linked owned quest.
- **evidence** · The verification ledger below and the [release binary PTY receipt](pty-receipt.json).
- **remaining uncertainty** · Native Ghostty and live Herdr acceptance. Herdr lookup and focus use separate requests.
- **next** · Use the guarded native walkthrough with a test-owned adventurer that has a qualified session.

## Change contract

Link the selected owned quest to the selected live adventurer with explicit
keyboard controls. Use that association for verified Observe in the Board,
War Table and Inbox. Show the active link separately from the durable
assignee. Prove that stale, ambiguous, disconnected, exited and managed pane
evidence cannot authorize observation. Verify that the association stays
out of persistent state and survives ordinary writes only in memory.

## Boundary and decision review

Verdict · safe with noted risk. Herdr owns pane, terminal and session
identity. The model owns only the connection-scoped user selection.
The writer, cache, Chronicle and summons reducers keep their existing
authority. The same focus preflight used by Inbox reviews is the I/O
boundary. A pane change between Herdr's lookup and focus calls can still
race the request. The result must be treated as unconfirmed when Herdr
returns a different identity.

Decision · proceed. The useful capability is observing the right session
from an owned quest. The riskiest shortcut is using an assignee name or
cached pane ID as identity. Reuse the complete Herdr identity already
present in the live model and verify it again at focus. See [ADR 0013](../../adr/0013-verified-owned-quest-association.md).

## Verification ledger

| Claim | Evidence |
| --- | --- |
| The Board, War Table and Inbox can observe the explicitly linked adventurer. | `owned_quest_association_enables_verified_observe_without_writing_a_binding` first failed on the absent link. The production input path now emits the verified focus command from all three views, preserves the link across an acknowledged assignment edit, and clears it explicitly with `L`. |
| Unqualified or stale evidence cannot revive a link. | `owned_quest_association_requires_current_unambiguous_evidence_and_never_revives` exercises terminal replacement, pane movement, missing or exited agents, ambiguous panes, a managed pane, unverified work and reconnect. Reinstalling prior facts does not restore revoked links. |
| Herdr preflight and focus response both corroborate the occupant. | `verified_observe_checks_the_current_occupant_before_and_after_focus` runs the production executor against a test socket. It sees only lookup on a preflight mismatch, a successful focus on matching evidence, and an identity mismatch when the occupant changes in the focus response. |
| The raw live identity stays out of durable work. | The Board integration checks that linking leaves the owned file and cache byte-for-byte identical. A later real owned write still contains no raw session token. The reopened cache contains no bindings. |

An early terminal run showed that a rejected focus left the link displayed
until the next snapshot. The fixture then reproduced this as a failing
assertion. The runtime now clears matching links as soon as the focus
identity result arrives and queues a snapshot refresh. The initial Clippy
runs also flagged the Board renderer length and three test-only clones.
The notice now renders through its own function and the tests compare borrowed
slices.

## Full gate and terminal evidence

On Darwin arm64 with Rust 1.90.0 and local changes on `e01485d`,
`just release-verify` passed 887 Rust tests across 68 runs, 28 shell checks,
formatting, all-feature Clippy with warnings denied, script syntax, the release
build and the diff check. `just property-test 4096` passed 158 tests across
eight runs. `cargo doc --locked --all-features --no-deps` passed.

The production binary connected to a disposable protocol-22 fixture through
its real subscription path. The linked Observe action issued `pane.get`
then `pane.focus`. A simulated replacement issued only `pane.get`, showed
why focus failed, and removed the stale link. Both the initial and restarted
unlinked actions issued zero focus calls. The owned file stayed byte-for-byte
identical and the synthetic session token was absent from the saved profile.
The terminals and fixture process exited and the test profile was removed.
See [recorded calls and binary digest](pty-receipt.json).

```sh
python3 docs/reviews/2026-11-19-owned-quest-association/pty-association.py \
  --logs /tmp/questmancer-association-review
```

## Review outcome

Code and test self-review checked source ownership, exact pane and session
identity, revocation, and persistence isolation. The PTY used the production
release binary with a local fixture socket. Live Herdr and Ghostty acceptance
remain open.

**Recommendation** · merge for the qualified local scope.
