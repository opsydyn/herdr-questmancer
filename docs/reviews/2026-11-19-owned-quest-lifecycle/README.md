# Owned quest lifecycle evidence

- **result** · complete
- **claim** · L2a is locally implemented and qualified through production Board input, durable restart and downgrade protection.
- **evidence** · The verification ledger below and the [release-binary PTY receipt](pty-receipt.json).
- **remaining uncertainty** · Human Ghostty and live Herdr acceptance.
- **next** · Keep the planned human review at the end of the durability track. Dependency editing is the next distinct L2 decision.

## Change contract

The selected owned quest can be marked active, marked complete and reopened
through one confirmation form on the Quest Board. The owned writer commits
the state and attributed receipt together. The Board and War Table reflect the
acknowledged result across restart. Existing v1 and v2 snapshots retain their
read and write contracts until the first lifecycle write upgrades them to v3.
Keep failure, indeterminate and view-pending behavior across that boundary.

## Boundary review

Safe with noted risk. The Board submits a typed command captured at the
displayed owned revision. The store rechecks source, state and revision before
publication. The projection carries the existing quest fields and Herdr owns
live presence. The integrity risk is accepting a changed quest on the strength
of an earlier receipt, so the retained transitions must agree with the latest
quest state and revision. Use the current form, worker and snapshot boundary
for [ADR 0010](../../adr/0010-owned-quest-lifecycle.md).

## Verification ledger

| Claim | Evidence | Scope |
| --- | --- | --- |
| The Board offers one explicit transition at a time and requires confirmation. The War Table follows the saved state. | `board_quest_lifecycle_requires_confirmation_and_survives_restart` first failed because `t` kept the Board open. It now drives production input, a narrow form, default refusal, publication, Board and War Table projection, shutdown, and store/cache reopen. | Production input and Ratatui test backend |
| The first confirmed transition upgrades a legacy store to v3. The format persists after receipt eviction. | `legacy_stores_upgrade_on_a_lifecycle_write_and_keep_v3_after_receipt_eviction` opens snapshots from v0.2.4, checks byte preservation on open, writes 128 unrelated artifacts, then verifies v3 and the quest state on reopen. The v2 origin retains its open decision and review. | Synthetic data from a published binary |
| The writer rejects invalid changes and preserves an exact saved outcome. | `lifecycle_rejects_wrong_source_state_and_revision_and_checks_retained_receipts` covers a foreign source, stale revision, illegal state, replay and a modified retained receipt. `first_lifecycle_save_recovers_v3_after_an_indeterminate_commit` injects failures before and after replacement, then reconciles and reopens. | Backend and filesystem-fault tests |
| The writer releases its lease when it shuts down and excludes a second owner while held. | The first full run saw `Busy` on reopen. A duplicated lock descriptor reproduced that failure deterministically. The lease now explicitly unlocks on drop and `owner_shutdown_releases_lock_with_an_inherited_descriptor` passes. The existing process-level contention test also passes. | Darwin arm64. A duplicated descriptor models transient descriptor inheritance without depending on process timing. |
| Full engineering gate passes. | `just release-verify` passed 877 Rust tests across 68 runs, 28 shell checks, formatting, all-feature Clippy with warnings denied, script syntax, release build and the diff check. `just property-test 4096` passed 154 tests across eight runs. `cargo doc --locked --all-features --no-deps` passed. | Darwin arm64, Rust 1.90.0, local changes on `75e7d35` |

The legacy fixtures contain synthetic quests and decisions. The production v0.2.4
snapshots were formatted for readability. The v2 fixture's temporary Git path
was normalized in both the review and its receipt. Their SHA-256 digests are
`4e1ad3ffffa7ba67563ae4d5bc2f3119a18ca43a9747d8c90b7284b5741d1cc6` (v1) and
`26d4051855f885a2cb5468b47a1054292766f3c0ac1c20acf52633e9ace0ff3c` (v2).

## Offline terminal evidence

The rebuilt release binary opened a v2 snapshot byte-for-byte, then accepted
Planned → Active at 120×34 and Active → Completed after resizing to 60×20.
A fresh process displayed Completed before reopening the quest at 120×34.
Each run read back the acknowledged cache cursor, authoritative quest state
and attributed receipt. Trying the default confirmation preserved the
authority bytes each time. The open decision and review matched their v2
records after all transitions.

The executable installed from the published macOS ARM64 v0.2.4 archive refused
the v3 snapshot with `unsupported version`. The store bytes survived the refused
open and normal quit. All three processes exited normally and emitted terminal
teardown sequences. The isolated profile was removed and no Herdr resources
were created. See the [recorded receipt](pty-receipt.json).

Reproduce with the rebuilt release binary and an executable installed from the
published v0.2.4 archive.

```sh
python3 docs/reviews/2026-11-19-owned-quest-lifecycle/pty-lifecycle.py \
  --previous /absolute/path/to/published/v0.2.4/questmancer \
  --logs /tmp/questmancer-lifecycle-review
```

## Review outcome

Manual code and boundary self-review covered the source and revision checks,
confirmation defaults, retained receipt chain, format selection after eviction
and explicit writer-lock release. The isolated PTY exercises Ratatui input
and the terminal fallback. Live Herdr, Kitty graphics and Ghostty visual
acceptance remain open.

**Recommendation** · merge for the qualified local scope.
