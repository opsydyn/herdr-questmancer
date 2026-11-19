# Review an owned quest in the Quest Inbox

Open the Quest Board with `b`. Use `a` to create or open a local owned store if
needed, then create a campaign and quest. Select that quest and press `v`.

1. Enter the acceptance criterion and the physical root of a local Git
   checkout, as shown by `pwd -P` from its root. Enter two distinct full commit
   IDs for the base and candidate. Enter the check recipe and the label of the
   local operator who will run it. Optionally choose Pi or Codex and paste an
   attributed summary for this candidate. Use Tab to reach **Request revision
   review** and Enter to save.
2. Return to the Board and press `i`. Enter opens the selected summons. Tab
   moves through other summons and past reviews for the selected quest. `j`/`k`
   and the arrow keys scroll the review. The change summary compares the pinned
   commits. The review panel includes any PR link registered for this quest.
3. Run the declared check from a clean checkout of the pinned candidate. Record
   the actual full commit ID and result in a local receipt. Check the checkout
   before and after the run. Use `inconclusive` if the check did not complete
   or the checkout no longer represents the candidate. The receipt is the
   operator's attestation. Paste it with `c`, then submit the import form.
4. Compare the evidence to the acceptance criterion. Press `x`, choose **Settle
   this exact revision**, tab to the settlement button and press Enter. A failed,
   missing, unavailable or mismatched check leaves the review open. Request a
   new review to examine another candidate. Previous decisions stay accessible
   through the quest's history.

From the review, `[l]` links the adventurer currently selected in the Hall or
Delve. `[o]` observes a verified live pane. A changed connection or adventurer
session invalidates the link. `g` refreshes the evidence for the pinned commits.

## Check receipt

Paste one JSON object in the import form. The path, full `head` ID, `recipe`
and `runner` must match the review. `captured_at` is Unix time in milliseconds.
Allowed results are `passed`, `failed` and `inconclusive`. For example, after
running the agreed check and recording its outcome, the operator can generate
a receipt from the checkout root using `jq`.

```bash
jq -n \
  --arg repository "$(pwd -P)" \
  --arg head "$(git rev-parse HEAD)" \
  --arg recipe 'just verify' \
  --arg runner 'Local operator' \
  --arg result 'passed' \
  '{repository: $repository, head: $head, recipe: $recipe, runner: $runner,
    captured_at: (now * 1000 | floor), result: $result}'
```

Use the actual check outcome in `result`. The Inbox never runs the recipe.
An adventurer's report is context and cannot substitute for a check receipt.

## Qualification

- In a disposable configuration, set `inbox_on_start = true`. Launch normally
  and expect the Inbox over the saved or configured room. Esc returns to that
  room. `b` opens the Board, and `a` reaches store setup when the Inbox is empty.
  An explicit Guild Hall or Delve launch opens the requested room directly.
  Repeat with the setting omitted to confirm ordinary room startup.
- Import a passing result for the base instead of the candidate. The import
  should explain the mismatch, and settlement should remain unavailable.
- Defer an open review with `s`, quit and reopen the same profile. The Inbox
  should show the review and its wake time. Press `s` to restore it or wait for
  it to return to the due queue. Marking seen should leave it due.
- Import the matching result and settle. Reopen the profile and confirm the
  exact candidate, test result and reviewer are retained. An explicit review
  settlement does not change quest lifecycle or merge a PR.
- Repeat at a compact terminal size and with the live adventurer unavailable.
  Review evidence and keyboard navigation remain available with the local store
  and repository. Esc returns from review to Inbox, Board and room.

Existing owned-work files can be opened directly. The first committed review
upgrades the authoritative snapshot to v2. An older Questmancer binary protects
that file as an unknown version.
