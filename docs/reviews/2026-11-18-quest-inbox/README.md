# Quest Inbox study

The study tests an inbox-first Quest Board with a fixed local campaign sample.
Each row represents a quest. Pi and Codex appear as contributors in its evidence.
The queue reports human action due, checks still running, and snoozed work.
The fixture's PRs, revisions, checks and reports are illustrative. Triage and
settlements last for the Storybook process.

```bash
cargo run --locked --features storybook --bin questmancer-storybook -- \
  --story interaction.quest-inbox --inspect --no-graphics
```

1. Press `Space` on **Keep counsel visible in a full Hall**. It becomes seen,
   and the due count stays at two.
2. Press `j`, then `Enter` on **Walk to the Delve station**. The diff is for
   `781b96e` and the passing tests are for `55d18a7`. Press `a` and check that
   the review stays open and explains which revision needs tests.
3. Press `Esc`, then `s` to snooze the Delve quest. Press `k` and `Enter` to
   review the Hall quest. Compare the acceptance criteria, diff, checks, PR
   state and agent reports. Press `a` to settle the review for `c7409e1`.
4. Press `Esc`. The queue shows zero due, one waiting and one snoozed. Select
   the Delve quest and press `s` again to return it to the due count.

Use `j` and `k` to read the review in a compact pane. `Esc` returns from the
review to the inbox and from the inbox to the catalogue. `q` quits.

The Storybook checks exercise keyboard triage at wide and compact sizes. An
isolated terminal run repeated the stale refusal, snooze, current-revision
settlement and clean exit. The `Quest Inbox - Storybook` tab in Herdr was
opened for review and confirmed to render the fixture queue.
`just verify` passed formatting, Clippy, 865 Rust tests and 28 shell checks.

The next product check is three real Pi/Codex reviews against the current
Quest Board and pane-switching flow. Record pane opens and whether the reviewer
can identify the next action. Include stale evidence and a deferred item that
survives restart before considering an opt-in production home.
