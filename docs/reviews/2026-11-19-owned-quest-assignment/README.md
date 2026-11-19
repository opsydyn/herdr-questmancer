# Owned quest assignment evidence

- **result** · complete
- **claim** · L2c supports explicit owned quest assignment with checked publication and restart recovery.
- **evidence** · The verification ledger below and the [release-binary PTY receipt](pty-receipt.json).
- **remaining uncertainty** · Human Ghostty and live Herdr acceptance.
- **next** · Review the qualified local slice before promoting verified Herdr association.

## Change contract

Assign, reassign and clear a named assignee on the selected owned quest.
Show the current assignment and require an explicit choice before saving.
The same atomic snapshot carries the updated quest and attributed receipt.
The Board and War Table recover the saved value across restart.

## Boundary and decision review

Verdict · safe with noted risk. The form captures the shown revision and
prior assignee. The owned writer rejects foreign, stale, malformed and
unchanged commands. The Board and War Table read the acknowledged work
projection. The existing live binding gate still controls observation.

Decision · proceed. Reusing the quest assignment record avoids an assignee
directory and keeps Herdr session IDs out of the store. The main risk is
confusing a saved name with a verified live binding. Qualify assignment
while offline, including an unavailable Observe action, and prove the
owned-format change preserves earlier stores on open. Revisit scope if
the workflow requires identity resolution, live commands or a cache schema
change. See [ADR 0012](../../adr/0012-owned-quest-assignment.md).

## Verification ledger

| Claim | Evidence |
| --- | --- |
| The Board offers an explicit assignment and the War Table reflects it. | `board_records_replaces_and_clears_a_local_assignment` first failed on the absent Board shortcut. It now exercises the real form and confirmation handler, checks the Board and War Table after each edit, and reopens authority and cache. |
| A name does not create a live association. | The Board test has a connected synthetic Herdr snapshot with an adventurer whose name and persona match the assignee. It checks that no verified binding appears and pressing Enter produces no focus command. |
| Rejections and retries leave prior bytes intact. | `assignment_refuses_invalid_stale_and_unchanged_commands_without_writing` covers missing or foreign quests, blank and multiline names, stale revision and prior assignment, unchanged values, exact retry, and a forged pane ID in the owned assignment. |
| Clearing and receipt eviction keep v5. | `assignment_format_and_history_survive_clear_and_receipt_eviction` interleaves prerequisites and assignment edits. It tests a downgraded header, falsified prior-assignment evidence, and another save after restart with all assignment receipts evicted. |
| A lost acknowledgement is reconciled before the assignee is shown as saved. | `first_assignment_save_reconciles_before_exposing_the_assignee` injects failure before replacement and acknowledgement loss after replacement. It checks original bytes, unavailable acknowledged state, the recovered receipt and reopened v5 assignment. |

The first integration run after wiring the form found that the Observe
explanation wrapped across rows at the chosen width. The test now checks the
visible unavailable state and directly asserts that the binding and focus
command are absent. The initial Clippy pass flagged form-builder and
form-renderer length. Assignment choices and contextual form text now live
in focused functions.

## Full gate and terminal evidence

On Darwin arm64 with Rust 1.90.0 and local changes on `230067c`,
`just release-verify` passed 885 Rust tests across 68 runs, 28 shell checks,
formatting, all-feature Clippy with warnings denied, script syntax, the
release build and the diff check. `just property-test 4096` passed 158 tests
across eight runs. `cargo doc --locked --all-features --no-deps` passed. The
Librarian's Keyring test also checks the new shortcut and retained exit keys.

The retained prerequisite binary created a v4 store. The new release
binary opened it byte-for-byte, assigned a name at 120×34, and showed that
name after a fresh process start. It replaced the name with a Unicode
assignee and then cleared it at 60×20. All three edits checked the prior
value, receipt actor, resulting cache and unavailable Observe action.
The earlier binary refused to open v5 after clearing and preserved the
bytes. The PTY runs used a disposable profile that was removed after
normal terminal teardown. See [recorded digests and revisions](pty-receipt.json).

```sh
python3 docs/reviews/2026-11-19-owned-quest-assignment/pty-assignment.py \
  --previous /absolute/path/to/prerequisite-build/questmancer \
  --logs /tmp/questmancer-assignment-review
```

## Review outcome

Code and test self-review covered source and revision checks, prior-assignment
validation, the separation from live binding, and atomic v5 recovery. The full
gate and isolated terminals cover the local implementation. Native Ghostty
and live Herdr acceptance remain open.

**Recommendation** · merge for the qualified local scope.
