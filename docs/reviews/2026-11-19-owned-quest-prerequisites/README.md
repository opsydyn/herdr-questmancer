# Owned quest prerequisite evidence

- **result** · complete
- **claim** · L2b supports same-campaign prerequisite editing with checked publication and restart recovery.
- **evidence** · The verification ledger below and the [release binary PTY receipt](pty-receipt.json).
- **remaining uncertainty** · Human Ghostty and live Herdr acceptance.
- **next** · Review this local slice before promoting explicit assignment. Keep the planned human review at the end of the durability track.

## Change contract

A Questmancer can add and remove one prerequisite between two owned quests in
the same campaign. Use the existing board confirmation and serial writer.
Prove default refusal, add/remove direction, cycle and reference rejection,
atomic publication, cache and War Table projection, and restart. Prove the
v4 format boundary with an earlier executable and after removal and receipt
eviction. Native Ghostty and live Herdr acceptance remain separate.

## Boundary and decision review

Verdict · safe with noted risk. The UI captures the selected quest and shown
revision. The owned writer revalidates both endpoints and the graph. The
Board, War Table and cache consume the existing acknowledged projection.
The key risks are stale edits, reversed arrows and accidental downgrade
after the last link disappears. Use a one-link command and preserve the
version at the existing atomic boundary. A separate graph or data service
would add an unnecessary writer and transaction boundary.

Decision · proceed. The War Table already represents the relationships.
Durable editing is the missing local capability. Stop and revisit the design
if this requires a new cache format, external-source writes or execution
authority. See [ADR 0011](../../adr/0011-owned-quest-prerequisites.md).

## Verification ledger

| Claim | Evidence |
| --- | --- |
| The Board offers an explicit add or remove and the War Table keeps the correct arrow direction. | `board_edits_and_reopens_owned_prerequisites` first failed because `p` stayed on the Board. It now drives the production input handler, verifies default refusal, checks the visible dependency and exact War Table edge after each save, then reopens authority and cache. |
| Invalid edits leave the authoritative store byte-for-byte identical. | `prerequisite_commands_reject_wrong_endpoints_and_transitive_cycles_without_writing` covers foreign endpoints, missing quest, cross-campaign and self-links, duplicate add, absent remove, stale revision, and a transitive cycle. It also opens a snapshot with a cyclic graph and checks protected-file behavior. |
| The v4 envelope stays beyond the final link and receipt. | `prerequisite_format_survives_link_removal_and_receipt_eviction` interleaves lifecycle and link edits, checks exact replay, rejects a changed receipt and a downgraded header, and verifies another write after reopen with both link receipts evicted. |
| Publication keeps the format and receipt together. | `first_prerequisite_save_reconciles_the_v4_envelope` injects failure before rename and acknowledgement loss after rename. Reconciliation restores the acknowledged v4 snapshot and retry returns the original receipt. |
| The new shortcut fits the Librarian's Keyring. | The full suite first failed `the_ledger_shows_the_whole_keyring` at 130×30 because the longer label wrapped and hid the bottom keys. The shortened entry passed the targeted rerun. |

The initial full gate flagged function length in the command converter and
the endpoint test. The converter now has a focused prerequisite action
function, and the test reuses the existing campaign fixture.
The shell vocabulary guard treated the Rust field spelling `prerequisite:`
as the old `Site:` label. Naming the field `prerequisite_id` makes the receipt
endpoint explicit. The existing guard then passed all 28 shell checks.

## Full gate and terminal evidence

On Darwin arm64 with Rust 1.90.0 and local changes on `0a9752f`,
`just release-verify` passed 881 Rust tests across 68 runs, 28 shell checks,
formatting, all-feature Clippy with warnings denied, script syntax, the release
build and the diff check. `just property-test 4096` passed 156 tests across
eight runs. `cargo doc --locked --all-features --no-deps` passed.

The retained lifecycle binary created a v3 store. The rebuilt release binary
opened it byte-for-byte, added the link at 120×34 and showed it after a fresh
launch. It removed the link after resizing to 60×20. Both edits reached the
acknowledged cache and saved an attributed receipt. The default save and
an attempted cycle preserved the store bytes. The earlier binary refused the
v4 store after the last link was removed and also preserved the bytes. All
offline terminal sessions exited normally and the test profile was removed.
The [receipt](pty-receipt.json) records the binary digests and revisions.

```sh
python3 docs/reviews/2026-11-19-owned-quest-prerequisites/pty-prerequisites.py \
  --previous /absolute/path/to/lifecycle-build/questmancer \
  --logs /tmp/questmancer-prerequisite-review
```

## Review outcome

Code and test self-review checked quest identity, edge direction, cycle detection,
retained receipt validation and the v4 format across indeterminate publication.
The PTY probe establishes keyboard, resize and restart behavior in the fallback
terminal path. Human Ghostty and live Herdr acceptance remain open.

**Recommendation** · merge for the qualified local scope.
