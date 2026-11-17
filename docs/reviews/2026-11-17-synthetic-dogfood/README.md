# Synthetic dogfood in the current Herdr session — 2026-11-17

- result: complete
- claim: The rebuilt candidate passed synthetic CLI integration dogfooding, including owned authoring and restart. The party remains open for human review.
- evidence: `verification.json`, `receipt.json`; logs and captured text under `/private/tmp/qmdogfood-74b_9_02`.
- remaining uncertainty: Native appearance, mouse behavior, physical terminal resizing and unassisted usability await human review. No live completion transition or real-agent acceptance is claimed.
- next: Review the retained **Questmancer dogfood** tab, then decide whether to close the acceptance gate.

## Verified

Fresh `cargo build --release` and `just verify` passed: 789 Rust tests across 64 runs, 28 shell tests, formatting, Clippy with warnings denied and shell syntax.
The current shared server is Herdr 0.9.5 / protocol 22. It was not restarted.

Four owned foreground receivers report working, blocked, idle and unknown. The real Codex pane remains separate. Guild/Delve/Guild, blocked selection, synthetic-only scrying, Ledger paging and search were exercised through Herdr input and checked using captured text. The working/blocked/working transition updated the selected card, and the Questmancer pane stayed out of the adventurer set.

The five forms created one campaign, one Planned quest, decisions/answers and one artifact. Two decision requests were used to distinguish raw-text newline behavior from real Enter editing. Final authority/cache revision is 7. Seen/defer did not modify source Open state; explicit answers resolved the requests. The War Table showed zero open human requests and one registered artifact while the quest remained Planned. Literal answer text was absent from the cache and Chronicle.

Only this test UI was quit/relaunched. The remembered store reopened; its exact multiline answer was visible and authoritative bytes remained unchanged. At 25 columns, submission was disabled, the draft survived restoration and was explicitly discarded. Resize helper panes were closed.

## Harness findings and limits

- Herdr's managed plugin launch replaced the requested temporary `HERDR_PLUGIN_STATE_DIR`. The initial test-created managed pane used ordinary plugin state and was closed before authoring. Automatic normal-state initialization may have occurred; unchanged normal plugin-state bytes are not claimed. Subsequent testing used a plain test-owned tab whose launcher explicitly selected temporary config/state. This validates live Herdr integration, not the ordinary managed-pane state-isolation override.
- The first helper assumed rename returns JSON; Herdr returned plain text. Creation responses were retained and setup resumed without duplicating panes.
- An early navigation assertion expected the owned decision immediately after `!`; inspecting the Board showed it correctly. Seen/defer was then exercised on the explicit Board selection.
- Raw LF in `pane send-text` did not insert a multiline break. A separate decision answered using logical Enter preserved the exact newline. This is a harness input distinction, not evidence of a native paste defect.
- Split ratio refers to the original pane fraction. The initial ratio assertion failed; its helper was closed. The corrected ratio produced 25 columns and the check passed.
- The managed-only `plugin pane focus` command rejected the plain test pane. The review helper uses `tab focus` for its recorded tab instead.
- Plugin action logs were inspected and empty: this run used direct pane launch/input rather than action invocation.

The existing source worktree matched its baseline before this evidence directory was added. No real-agent input or counsel was sent, no source code was changed, and neither preexisting Questmancer-labelled shell pane was controlled. Existing plugin registration was retained. Synthetic state and CLI text do not qualify native art.

## Review and retention

The new tab is **Questmancer dogfood**, pane `w8:pY`, tab `w8:tE` at this run's baseline. Revalidate ownership before later automation. The live resource ledger is `/private/tmp/qmdogfood-74b_9_02/receipt.json`.

```bash
python3 /private/tmp/qmdogfood-74b_9_02/control.py focus
```

Review `1` Hall, `2` Delve, `!` summons, `o` synthetic scrying, `?` Ledger and `b` Board / `w` War Table. Owned Actions are under `b`, then `a`; choose the dogfood campaign to inspect Decisions and the registered artifact. Native approval remains pending.

Cleanup is intentionally deferred. On an explicit later cleanup request, the trial's `control.py cleanup` validates terminal identities, releases only its synthetic sources, closes only its owned panes and attempts to restore original focus. It never stops the shared server or deletes the test files.

Recommendation: **needs review**.
