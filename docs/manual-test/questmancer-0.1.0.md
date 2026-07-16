# Questmancer 0.1.0 guarded acceptance

Tested on 2026-07-16 from commit `841ff5f` on macOS arm64 with Rust 1.90.0.
Herdr client and server were both 0.7.4, protocol 16, compatible, with no
restart required.

This was a guarded source test against a pre-existing Herdr session. The server,
Codex pane, prior plugin registration, and prior plugin pane were protected.
Questmancer remained linked from the feature worktree after cleanup.

## Result

| Assertion | Result | Observed evidence |
|---|---|---|
| Release binary | PASS | `cargo build --release` produced executable `target/release/questmancer` at `841ff5f`. |
| Registration | PASS | `opsydyn.questmancer` was enabled from the local feature worktree at version 0.1.0 with minimum Herdr 0.7.4. |
| Actions | PASS | Exactly `open`, `close`, `toggle`, `guild`, and `delve` were registered. |
| Singleton | PASS | Two consecutive `open` actions created one managed pane, `w2:pG`; both action logs exited 0 with empty stderr. |
| Guild Hall and Delve | PASS | Key `2` rendered `QUESTMANCER DELVES - paths joined`; the saved Guild Hall returned after close/reopen. |
| Dedicated agent | PASS | Only disposable pane `w2:pH` received source `questmancer-manual-1784196969945-56545` for agent `questmancer-smoke`. |
| Working projection | PASS | Snapshot reported `working`; Delve rendered `[>] DELVING \| LIVE`. |
| Blocked projection | PASS | Snapshot reported `blocked`; Delve rendered `SEALED`, `SIGNAL LANTERN`, and `[!] COUNSEL REQUESTED \| LIVE`. |
| Chronicle voice | PASS | `chronicle.jsonl` contained exactly one counsel event: `questmancer-smoke requested counsel`; no superseded product wording appeared. |
| Search and selection | PASS | `/ questmancer-smoke` selected only the disposable agent, rendered as `Sabine Copperkettle`, Gnome Testmender. |
| Acknowledge | PASS | Space removed the Summons acknowledgement footer and rendered `Summons acknowledged.` for the selected agent. |
| Persisted key | PASS | Before restart, `seen_attention` contained only `persona-6be30b4fc2d45dbf918410eb` plus `counsel_requested`; it had no `pane_revision`. |
| Restart persistence | PASS | While the agent stayed blocked, close/reopen changed the managed pane from `w2:pG` to `w2:pJ`; Guild Hall, Sabine, the acknowledgement, and the two-field key survived even though the snapshot revision was `0`. |
| Goblin containment | PASS | The exact incantation produced `The goblins deny any involvement.`; after reopen, neither the outbreak nor its status returned, and no goblin state was persisted. |
| Counsel transport | PASS | An earlier guarded run sent exact counsel only to its disposable pane; it did not target a protected pane. |
| Output refresh | PASS | An earlier guarded run refreshed selected output without changing selection or surfacing an error. |
| Explicit synthetic `done` | BLOCKED | Herdr 0.7.4 does not accept `done` as a `report-agent` state. |
| Synthetic resting/spoils | BLOCKED | A final `--state idle` probe was accepted but normalized to snapshot state `done` and rendered returned spoils, so it was not counted as proof of either documented synthetic path. |
| Reviewr | BLOCKED | The Guild Hall reported `Reviewr is unavailable.`; no action was invoked. |
| `j` / `k` | BLOCKED | Adjacent selections included protected panes, and selection can trigger lazy output reads. Search provided the safe selection path. |
| Ghostty screenshots | BLOCKED | Computer Use could not safely access the live Ghostty session, so no screenshot is claimed. CLI `pane read --source visible` supplied text evidence only. |
| Prior-link cutover | BLOCKED | The prior plugin's managed pane was protected; the old registration and pane were intentionally untouched. |

## Automated gate

The guarded live-acceptance checkout at `841ff5f` passed 393
all-target/all-feature Rust tests and all 20 shell workflow tests. The focused
4,096-case property and persistence run passed all 26 tests. These counts record
that acceptance checkout, not later branch HEADs; subsequent correction reports
carry their own fresh automated counts. Formatting, clippy with warnings denied,
shell syntax, the release build, executable check, and whitespace check also
passed at `841ff5f`:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
bash tests/scripts.sh
bash -n herdr/install.sh herdr/run.sh herdr/control.sh
PROPTEST_CASES=4096 cargo test --test property_domain --test persisted_state
cargo build --release
test -x target/release/questmancer
git diff --check
```

## Commands that work with Herdr 0.7.4

The installed CLI uses `herdr --version`, `herdr status`, and
`herdr api snapshot`; `herdr version` and `herdr api ping` are not commands.
The core test used:

```bash
cargo build --release
herdr plugin action invoke opsydyn.questmancer.open
herdr plugin action invoke opsydyn.questmancer.open

WORKSPACE_ID=$(herdr workspace list |
  jq -r '.result.workspaces[] | select(.focused) | .workspace_id' |
  head -n 1)
herdr tab create --workspace "$WORKSPACE_ID" --cwd "$PWD" \
  --label "questmancer-smoke-$(date +%s)" --focus
PANE_ID=$(herdr pane current | jq -r '.result.pane.pane_id')
SOURCE_ID="questmancer-manual-$(date +%s)-$$-$RANDOM"

herdr pane report-agent "$PANE_ID" --source "$SOURCE_ID" \
  --agent questmancer-smoke --state working \
  --message "mapping the sealed gate" --seq 1
herdr pane report-agent "$PANE_ID" --source "$SOURCE_ID" \
  --agent questmancer-smoke --state blocked \
  --message "Counsel requested at the sealed gate" --seq 2
```

The test selected the synthetic agent through `/`, acknowledged it with Space,
closed and reopened Questmancer while the agent stayed blocked, and inspected
only the two test-owned panes with `herdr pane read --source visible --format
text`.

## Persistence evidence

The acknowledged Summons was serialized as:

```json
{
  "persona": "persona-6be30b4fc2d45dbf918410eb",
  "summons": "counsel_requested"
}
```

On reopen, Herdr still reported the agent blocked with pane revision `0`. The
acknowledgement and selected persona remained intact. This proves that durable
user intent is keyed by stable persona and Summons rather than by an unrelated
pane/output revision.

Herdr 0.7.4 has no durable status-event identity. If an adventurer leaves and
returns to the same Summons entirely while Questmancer is closed, the restart
snapshot cannot distinguish that new episode; the previous acknowledgement is
preserved. An observed state or Summons change clears it normally.

## Cleanup audit

Cleanup returned and released the synthetic source at monotonically increasing
sequence numbers, closed only disposable pane/tab `w2:pH` / `w2:tG`, and closed
only the test-created Questmancer panes/tabs `w2:pG` / `w2:tF` and `w2:pJ` /
`w2:tH`.

The final snapshot exactly restored the baseline panes `w2:p1`, `w2:p7`, and
`w2:p8`; tabs `w2:t1`, `w2:t6`, and `w2:t7`; and focus `w2:p8`. The synthetic
agent was absent. Herdr remained running at 0.7.4 / protocol 16, Questmancer's
new local link was retained, and the prior plugin was untouched. Questmancer
action logs 18 through 22 all exited 0 with empty stderr.

Before the final run, Questmancer was closed and the sole prior disposable
`questmancer-smoke` Chronicle row was backed up to
`/tmp/questmancer-chronicle-pre-final-1784196892489.jsonl`, then removed with an
exact patch. `state.json` was not manually edited. The final run left its two
current-voice Chronicle records as ordinary acceptance history.
