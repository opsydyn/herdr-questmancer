# Questmancer 0.1.0 guarded acceptance

This runbook is deliberately conservative. Protect every server, plugin link,
pane, tab and agent that existed before the test. Never stop a pre-existing
Herdr server, unlink a pre-existing plugin, report against another real agent,
or close a pane/tab that the test did not create. Record every assertion as
`PASS`, `FAIL`, or `BLOCKED`; an inaccessible interactive terminal is a blocked
visual check, not evidence of a product defect.

## Guarded Great Room procedure

Run from the Questmancer checkout. Keep the literal source path in
`TEST_CHECKOUT`; do not use a broad directory or an unrelated checkout.

### 0. Establish the candidate boundary and ownership ledger

Resolve the checkout before any build, link, open, report, or focus command.
The five `TEST_CREATED_*` flags are deliberately initialized to `0`: change a
flag only after its corresponding command succeeded and capture the resulting
ID immediately. A protected resource is never retroactively made test-owned.

```bash
TEST_CHECKOUT="$(pwd -P)"
test -n "$TEST_CHECKOUT" && test "$TEST_CHECKOUT" != "/" || {
  printf '%s\n' 'Refusing to test from an unresolved or root checkout.' >&2
  exit 1
}

TEST_CREATED_LINK=0
TEST_CREATED_MANAGED_PANE=0
TEST_CREATED_TAB=0
TEST_CREATED_PANE=0
TEST_CREATED_REPORT=0
PREEXISTING_LINK=0
PREEXISTING_MANAGED_PANE_ID=""
MANAGED_PANE_IS_TEST_OWNED=0
LIVE_TESTS_PERMITTED=0

resolve_existing_root() {
  local candidate_root=$1
  test -n "$candidate_root" \
    && test "$candidate_root" != "/" \
    && test -d "$candidate_root" \
    || return 1
  (cd -- "$candidate_root" && pwd -P)
}

BASELINE_SNAPSHOT="$(mktemp)"
BASELINE_PLUGIN_LIST="$(mktemp)"
herdr api snapshot >"$BASELINE_SNAPSHOT"
herdr plugin list --json >"$BASELINE_PLUGIN_LIST"

BASELINE_FOCUS_PANE_ID="$(jq -er '.result.snapshot.focused_pane_id // empty' \
  "$BASELINE_SNAPSHOT" 2>/dev/null || true)"
BASELINE_FOCUS_TAB_ID="$(jq -er '.result.snapshot.focused_tab_id // empty' \
  "$BASELINE_SNAPSHOT" 2>/dev/null || true)"
BASELINE_PANE_IDS="$(jq -r '.result.snapshot.panes[]?.pane_id' \
  "$BASELINE_SNAPSHOT" | sort)"
BASELINE_TAB_IDS="$(jq -r '.result.snapshot.tabs[]?.tab_id' \
  "$BASELINE_SNAPSHOT" | sort)"

PLUGIN_MATCH_COUNT="$(jq '[.result.plugins[]? \
  | select(.plugin_id == "opsydyn.questmancer")] | length' \
  "$BASELINE_PLUGIN_LIST")"
REGISTRATION_SOURCE_ROOT_RAW=""
REGISTRATION_SOURCE_ROOT=""
if test "$PLUGIN_MATCH_COUNT" -gt 0; then
  PREEXISTING_LINK=1
fi
if test "$PLUGIN_MATCH_COUNT" = 1; then
  REGISTRATION_SOURCE_ROOT_RAW="$(jq -er '
    .result.plugins[]?
    | select(.plugin_id == "opsydyn.questmancer")
    | .plugin_root
  ' "$BASELINE_PLUGIN_LIST" 2>/dev/null || true)"
  REGISTRATION_SOURCE_ROOT="$(resolve_existing_root \
    "$REGISTRATION_SOURCE_ROOT_RAW" 2>/dev/null || true)"
fi
BASELINE_REGISTRATION_SOURCE_ROOT="$REGISTRATION_SOURCE_ROOT"

HERDR_STATE_HOME="${XDG_STATE_HOME:-$HOME/.local/state}"
QUESTMANCER_RUNTIME="$HERDR_STATE_HOME/herdr/plugins/opsydyn.questmancer/runtime.json"
RUNTIME_PANE_ID="$(jq -er '.pane_id | select(type == "string" and length > 0)' \
  "$QUESTMANCER_RUNTIME" 2>/dev/null || true)"
BASELINE_MANAGED_PANE_ID="$(jq -r --arg pane "$RUNTIME_PANE_ID" '
  .result.snapshot.panes[]?
  | select(.pane_id == $pane)
  | .pane_id
' "$BASELINE_SNAPSHOT" | head -n 1)"
PREEXISTING_MANAGED_PANE_ID="$BASELINE_MANAGED_PANE_ID"
```

Record the baseline focus, pane/tab lists, registration root and any managed
pane. The installed 0.7.4 snapshot does not expose pane titles or labels, so
the managed-pane identity comes from Questmancer's runtime registration and is
accepted only when that exact pane still exists in the snapshot. An absent,
duplicate, empty, root (`/`), missing or otherwise unresolved plugin root keeps
the live gate closed; it is never passed to `cd`.

### 1. Baseline the environment

```bash
git status --short --branch
git rev-parse HEAD
herdr --version
herdr status
herdr plugin list --json
herdr api snapshot
```

Record the server status, plugin registration/source, focused pane, all pane and
tab IDs, and whether a Questmancer pane already exists. If the server is already
running, it is protected and must remain running. If `opsydyn.questmancer` is
already linked, it is protected and must remain linked. A pre-existing managed
Questmancer pane is also protected: use it for read-only inspection, but do not
claim singleton creation, persistence restart or cleanup against it.

### 2. Build, link only when absent, and verify registration

```bash
cargo build --release
test -x target/release/questmancer
herdr plugin list --json
```

Only when the baseline proves the plugin is absent:

```bash
herdr plugin link "$TEST_CHECKOUT"
TEST_CREATED_LINK=1
POST_LINK_PLUGIN_LIST="$(mktemp)"
herdr plugin list --json >"$POST_LINK_PLUGIN_LIST"
PLUGIN_MATCH_COUNT="$(jq '[.result.plugins[]?
  | select(.plugin_id == "opsydyn.questmancer")] | length'
  "$POST_LINK_PLUGIN_LIST")"
REGISTRATION_SOURCE_ROOT_RAW=""
REGISTRATION_SOURCE_ROOT=""
if test "$PLUGIN_MATCH_COUNT" = 1; then
  REGISTRATION_SOURCE_ROOT_RAW="$(jq -er '
    .result.plugins[]?
    | select(.plugin_id == "opsydyn.questmancer")
    | .plugin_root
  ' "$POST_LINK_PLUGIN_LIST" 2>/dev/null || true)"
  REGISTRATION_SOURCE_ROOT="$(resolve_existing_root
    "$REGISTRATION_SOURCE_ROOT_RAW" 2>/dev/null || true)"
fi
```

Verify version `0.1.0`, minimum Herdr `0.7.4`, local source, enabled status and
the five actions `open`, `close`, `toggle`, `guild`, and `delve`. Remember
whether this test created the link so only that link may be removed later.

Before continuing, prove that the registration points to this exact resolved
checkout and that there was no protected managed pane in the baseline:

```bash
if test "$PLUGIN_MATCH_COUNT" = 1 \
  && test -n "$REGISTRATION_SOURCE_ROOT" \
  && test "$REGISTRATION_SOURCE_ROOT" = "$TEST_CHECKOUT" \
  && test -z "$BASELINE_MANAGED_PANE_ID"; then
  LIVE_TESTS_PERMITTED=1
fi
printf 'candidate root: %s\n' "$TEST_CHECKOUT"
printf 'registered root: %s\n' "$REGISTRATION_SOURCE_ROOT"
printf 'baseline managed pane: %s\n' "${BASELINE_MANAGED_PANE_ID:-none}"
printf 'live tests permitted: %s\n' "$LIVE_TESTS_PERMITTED"
```

All live rows are BLOCKED unless `LIVE_TESTS_PERMITTED=1`. If it remains `0`,
mark every open/singleton/report/interaction/screenshot row `BLOCKED`, explain
the root or protected-pane mismatch, and jump directly to step 7's restoration
verification. Do not use a protected pane as a substitute candidate.

### 3. Verify singleton open

Run this step only when `LIVE_TESTS_PERMITTED=1`.

Invoke open twice and compare snapshots:

```bash
if test "$LIVE_TESTS_PERMITTED" = 1 \
  && test "$REGISTRATION_SOURCE_ROOT" = "$TEST_CHECKOUT" \
  && test -z "$PREEXISTING_MANAGED_PANE_ID"; then
  herdr plugin action invoke opsydyn.questmancer.open
  herdr plugin action invoke opsydyn.questmancer.open
  herdr api snapshot
else
  printf '%s\n' 'BLOCKED: candidate root or managed-pane ownership guard failed.'
  LIVE_TESTS_PERMITTED=0
fi
```

Exactly one managed Questmancer pane should exist. Record its pane/tab ID and
whether this run created it. Inspect the corresponding plugin action logs. Do
not close a managed pane that existed in the baseline.

Capture only the newly created managed pane and stop if discovery is ambiguous:

```bash
if test "$LIVE_TESTS_PERMITTED" = 1 \
  && test "$REGISTRATION_SOURCE_ROOT" = "$TEST_CHECKOUT"; then
POST_OPEN_SNAPSHOT="$(mktemp)"
herdr api snapshot >"$POST_OPEN_SNAPSHOT"
MANAGED_PANE_ID="$(jq -r '
  .result.snapshot.panes[]?
  | .pane_id
' "$POST_OPEN_SNAPSHOT" | sort | comm -13 \
  <(printf '%s\n' "$BASELINE_PANE_IDS") -)"
MANAGED_PANE_COUNT="$(printf '%s\n' "$MANAGED_PANE_ID" | sed '/^$/d' | wc -l | tr -d ' ')"
if test "$MANAGED_PANE_COUNT" != 1; then
  printf '%s\n' 'Managed pane discovery was ambiguous; block remaining live rows.' >&2
  LIVE_TESTS_PERMITTED=0
fi
MANAGED_TAB_ID="$(jq -r --arg pane "$MANAGED_PANE_ID" '
  .result.snapshot.panes[]? | select(.pane_id == $pane) | .tab_id
' "$POST_OPEN_SNAPSHOT" | head -n 1)"
test -n "$MANAGED_PANE_ID" && test -n "$MANAGED_TAB_ID" || {
  printf '%s\n' 'Managed pane discovery was ambiguous; block remaining live rows.' >&2
  LIVE_TESTS_PERMITTED=0
}
if test "$LIVE_TESTS_PERMITTED" = 1; then
  TEST_CREATED_MANAGED_PANE=1
  MANAGED_PANE_IS_TEST_OWNED=1
fi
fi
```

### 4. Create one disposable plain pane and synthetic adventurer

Run this step only when `LIVE_TESTS_PERMITTED=1` and
`MANAGED_PANE_IS_TEST_OWNED=1`. Otherwise leave `TEST_CREATED_TAB`,
`TEST_CREATED_PANE`, and `TEST_CREATED_REPORT` at `0` and go to step 7.

Use the currently focused workspace, but create a new test-owned tab and plain
pane. Never target Codex, Questmancer, or another agent-owned pane.

```bash
if test "$LIVE_TESTS_PERMITTED" = 1 \
  && test "$MANAGED_PANE_IS_TEST_OWNED" = 1 \
  && test "$REGISTRATION_SOURCE_ROOT" = "$TEST_CHECKOUT"; then
WORKSPACE_ID=$(herdr workspace list |
  jq -r '.result.workspaces[] | select(.focused) | .workspace_id' |
  head -n 1)
TEST_LABEL="questmancer-great-room-$(date +%s)"
TAB_CREATE_JSON="$(herdr tab create --workspace "$WORKSPACE_ID" --cwd "$PWD" \
  --label "$TEST_LABEL" --focus)"
TAB_ID="$(jq -er '.result.tab.tab_id' <<<"$TAB_CREATE_JSON")"
TEST_CREATED_TAB=1
CURRENT_PANE_JSON="$(herdr pane current)"
CURRENT_TAB_ID="$(jq -er '.result.pane.tab_id' <<<"$CURRENT_PANE_JSON")"
PANE_ID="$(jq -er '.result.pane.pane_id' <<<"$CURRENT_PANE_JSON")"
test "$CURRENT_TAB_ID" = "$TAB_ID" || {
  printf '%s\n' 'Focused pane does not belong to the created tab; block live rows.' >&2
  LIVE_TESTS_PERMITTED=0
}
TEST_CREATED_PANE=1
SOURCE_ID="questmancer-manual-$(date +%s)-$$-$RANDOM"

herdr pane report-agent "$PANE_ID" \
  --source "$SOURCE_ID" \
  --agent questmancer-smoke \
  --state working \
  --message "mapping the Great Room" \
  --seq 1
TEST_CREATED_REPORT=1

herdr pane report-agent "$PANE_ID" \
  --source "$SOURCE_ID" \
  --agent questmancer-smoke \
  --state blocked \
  --message "Counsel requested at the sealed gate" \
  --seq 2
else
  printf '%s\n' 'BLOCKED: synthetic-agent ownership guard failed.'
  LIVE_TESTS_PERMITTED=0
fi
```

Confirm the snapshot contains only this test-owned synthetic source and that
Questmancer renders its blocked Summons at the Counsel Bell.

Herdr 0.7.4 cannot synthesize `done`: `report-agent` accepts `idle`, `working`,
`blocked`, and `unknown`, but not a `done` literal. A real agent transition or
the fixed Storybook/rendering fixtures are required for returned Spoils. Do not
claim live `done`, Hearth/resting, or Spoils coverage from an ambiguous
synthetic transition.

### 5. Exercise safe commands and persistence

Run this step only when `LIVE_TESTS_PERMITTED=1`,
`MANAGED_PANE_IS_TEST_OWNED=1`, and all three synthetic ownership flags are
`1`. Otherwise mark every interaction and persistence row `BLOCKED` and go to
step 7.

In Questmancer, search `/` for `questmancer-smoke` before acting. Verify the
result owns `PANE_ID`; do not use `j`/`k` when an adjacent selection could be a
protected agent. Then check:

- `/` selects the synthetic adventurer and its campaign table;
- `r` sends counsel only to `PANE_ID`;
- Space acknowledges only the selected unread Summons;
- `o` refreshes selected recent output without changing selection;
- Enter observes only `PANE_ID`;
- `1` and `2` preserve selection across Guild Hall and Delve;
- Reviewr `v` is invoked only when the footer advertises it.

While the synthetic adventurer remains blocked, close and reopen only a
Questmancer pane created by this run:

```bash
if test "$LIVE_TESTS_PERMITTED" = 1 \
  && test "$MANAGED_PANE_IS_TEST_OWNED" = 1 \
  && test "$TEST_CREATED_REPORT" = 1 \
  && test "$REGISTRATION_SOURCE_ROOT" = "$TEST_CHECKOUT"; then
  herdr plugin action invoke opsydyn.questmancer.close
  herdr plugin action invoke opsydyn.questmancer.open
else
  printf '%s\n' 'BLOCKED: persistence restart ownership guard failed.'
fi
```

Confirm the saved Guild Hall view, selected persona and acknowledged Summons
survive. If the managed pane existed before the test, mark restart persistence
`BLOCKED` instead of closing it.

### 6. Capture Great Room evidence

Run this step only when `LIVE_TESTS_PERMITTED=1` and
`MANAGED_PANE_IS_TEST_OWNED=1`; otherwise record both visual rows as `BLOCKED`
and go to step 7.

With the synthetic adventurer selected, capture one wide screenshot at 120
columns or more and one exact 80x24 screenshot. The wide image must show one
continuous hall with Guild Door, Quest Wall, all Campaign Tables, Counsel Bell,
Hearth, Chronicle Lectern, Scrying Alcove and Spoils Desk. The 80x24 image must
retain the same campaign identity and actions through the cropped-room camera.
Also inspect landmark-camera navigation below 80 columns when safe.

If the controlling PTY cannot resize or capture the user's terminal, record
these checks as `BLOCKED` with the available `herdr pane read
"$MANAGED_PANE_ID" --source visible --format text` evidence. Never substitute a
text read for a screenshot claim.

### 7. Clean up only test-created resources and verify restoration

This is the only cleanup path. Each mutation is conditioned on the exact flag
and captured ID. If a capture is missing or no longer matches the live
snapshot, leave the resource untouched, record `BLOCKED`, and do not guess.

Return and release the synthetic source only when this run created it:

```bash
if test "$TEST_CREATED_REPORT" = 1 \
  && test "$TEST_CREATED_PANE" = 1 \
  && test -n "${PANE_ID:-}" \
  && test -n "${SOURCE_ID:-}"; then
  herdr pane report-agent "$PANE_ID" \
    --source "$SOURCE_ID" \
    --agent questmancer-smoke \
    --state working \
    --message "manual test complete" \
    --seq 3
  herdr pane release-agent "$PANE_ID" \
    --source "$SOURCE_ID" \
    --agent questmancer-smoke \
    --seq 4
fi

if test "$TEST_CREATED_PANE" = 1 && test -n "${PANE_ID:-}"; then
  herdr pane close "$PANE_ID"
fi
if test "$TEST_CREATED_TAB" = 1 && test -n "${TAB_ID:-}"; then
  herdr tab close "$TAB_ID"
fi

CURRENT_SNAPSHOT="$(mktemp)"
herdr api snapshot >"$CURRENT_SNAPSHOT"
CURRENT_MANAGED_PANE_ID="$(jq -r --arg pane "${MANAGED_PANE_ID:-}" '
  .result.snapshot.panes[]? | select(.pane_id == $pane) | .pane_id
' "$CURRENT_SNAPSHOT" | head -n 1)"
if test "$TEST_CREATED_MANAGED_PANE" = 1 \
  && test -n "${MANAGED_PANE_ID:-}" \
  && test "$CURRENT_MANAGED_PANE_ID" = "$MANAGED_PANE_ID" \
  && test "$REGISTRATION_SOURCE_ROOT" = "$TEST_CHECKOUT"; then
  herdr plugin action invoke opsydyn.questmancer.close
fi
if test "$TEST_CREATED_LINK" = 1; then
  herdr plugin unlink opsydyn.questmancer
fi

if test -n "${BASELINE_FOCUS_TAB_ID:-}" \
  && herdr api snapshot | jq -e --arg tab "$BASELINE_FOCUS_TAB_ID" \
    '.result.snapshot.tabs[]? | select(.tab_id == $tab)' >/dev/null; then
  herdr tab focus "$BASELINE_FOCUS_TAB_ID"
fi
```

Herdr 0.7.4 exposes exact tab focus but `herdr pane focus` only moves in a
direction relative to a pane. The procedure therefore restores the exact
baseline tab and records the final pane ID for comparison; it does not claim
that the previously focused pane within a multi-pane tab can be restored by
the CLI.

Never stop a pre-existing server. Finally run the comparison rather than relying
on a successful cleanup command:

```bash
herdr api snapshot
herdr plugin list --json
herdr status
git status --short --branch
```

### Final baseline comparison

Compare pane IDs, tab IDs, focus, server state and protected plugin links with
the baseline. The environment is restored only when those protected resources
match and the synthetic source is absent:

```bash
FINAL_SNAPSHOT="$(mktemp)"
FINAL_PLUGIN_LIST="$(mktemp)"
herdr api snapshot >"$FINAL_SNAPSHOT"
herdr plugin list --json >"$FINAL_PLUGIN_LIST"
FINAL_PANE_IDS="$(jq -r '.result.snapshot.panes[]?.pane_id' "$FINAL_SNAPSHOT" | sort)"
FINAL_TAB_IDS="$(jq -r '.result.snapshot.tabs[]?.tab_id' "$FINAL_SNAPSHOT" | sort)"
FINAL_FOCUS_PANE_ID="$(jq -er '.result.snapshot.focused_pane_id // empty' \
  "$FINAL_SNAPSHOT" 2>/dev/null || true)"
FINAL_FOCUS_TAB_ID="$(jq -er '.result.snapshot.focused_tab_id // empty' \
  "$FINAL_SNAPSHOT" 2>/dev/null || true)"
FINAL_REGISTRATION_SOURCE_ROOT_RAW="$(jq -er '
  .result.plugins[]?
  | select(.plugin_id == "opsydyn.questmancer")
  | .plugin_root
' "$FINAL_PLUGIN_LIST" 2>/dev/null || true)"
FINAL_REGISTRATION_SOURCE_ROOT="$(resolve_existing_root \
  "$FINAL_REGISTRATION_SOURCE_ROOT_RAW" 2>/dev/null || true)"
test "$FINAL_PANE_IDS" = "$BASELINE_PANE_IDS"
test "$FINAL_TAB_IDS" = "$BASELINE_TAB_IDS"
test "$FINAL_FOCUS_TAB_ID" = "$BASELINE_FOCUS_TAB_ID"
printf 'baseline/final pane focus (informational): %s / %s\n' \
  "${BASELINE_FOCUS_PANE_ID:-none}" "${FINAL_FOCUS_PANE_ID:-none}"
if test "$PREEXISTING_LINK" = 1; then
  test "$FINAL_REGISTRATION_SOURCE_ROOT" = "$BASELINE_REGISTRATION_SOURCE_ROOT"
else
  test -z "$FINAL_REGISTRATION_SOURCE_ROOT"
fi
```

## Great Room release-candidate record — 2026-07-17

The automated release gate ran from the Task 8 working tree based on `dd049b1`
on macOS arm64 with Rust 1.90.0. Herdr client/server were already running at
0.7.4, protocol 16, compatible.

The guarded live pass stopped after read-only baselining because the environment
was protected and did not point at this feature checkout. The existing
`opsydyn.questmancer` link resolved to
`/Users/alancurrie/Projects/herdr-web-master`, while this release candidate was
in its isolated `questmancer-great-room` worktree. Managed pane `w2:pN`,
synthetic pane `w2:pM`, the five existing panes/tabs and focused pane `w2:p8`
all predated this run. Reading `w2:pN` showed the earlier bordered Guild Hall,
so it was not used as evidence for the Great Room.

| Guarded assertion | Result | Evidence |
|---|---|---|
| Herdr compatibility | PASS | Client/server 0.7.4, protocol 16, compatible, no restart required. |
| Release binary | PASS | `cargo build --release` produced executable `target/release/questmancer` from the feature worktree. |
| Registration shape | PASS | The protected registration was enabled at 0.1.0/minimum 0.7.4 with all five required actions. Its source path was the main checkout, not this candidate. |
| Candidate link/open/singleton | BLOCKED | Repointing or unlinking the pre-existing protected registration would violate the guardrail; the existing managed pane could not prove candidate creation. |
| Synthetic working/blocked agent | BLOCKED | Existing test and real-agent panes were protected, and no candidate TUI was safely linked. No report was sent. |
| Search, selection and counsel | BLOCKED | Acting through the protected old managed pane would test the wrong binary and risk another agent. |
| Acknowledge and output refresh | BLOCKED | Same protected wrong-binary boundary; no agent action was attempted. |
| Persistence restart | BLOCKED | Managed pane `w2:pN` existed at baseline and could not be closed by this run. |
| Wide Great Room screenshot | BLOCKED | The candidate was not linked and this PTY could not capture the user's interactive terminal. |
| Exact 80x24 screenshot | BLOCKED | The candidate was not linked and this PTY could not resize/capture the user's interactive terminal. |
| Explicit synthetic `done` | BLOCKED | Herdr 0.7.4 cannot synthesize `done`; no live completion is claimed. |
| Cleanup/restoration | PASS | No pane, tab, report, link or server was created or mutated. Final panes/tabs/focus, plugin sources and running server matched the baseline exactly. |

### Automated release gate

All automated commands completed successfully:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
bash tests/scripts.sh
bash -n herdr/install.sh herdr/run.sh herdr/control.sh
PROPTEST_CASES=4096 cargo test --test property_domain --test persisted_state --test guild_room_properties
cargo build --release
test -x target/release/questmancer
ruby tests/workflow_contract.rb
git diff --check
```

The all-target/all-feature run passed every test target, including the
feature-gated Storybook. The focused 4,096-case run passed all 34 property,
persistence and Great Room geometry tests. Shell workflows passed all 20 tests.

The first shell run exposed a stale audit pattern that classified the accepted
Great Room landmark name `SPOILS DESK` as legacy vocabulary even though the
production renderer and accepted design both use it. Removing that obsolete
test pattern was a test-only correction; the rerun passed all 20 shell tests.

### Accepted-design evidence audit

| Criterion | Result | Production/test/manual evidence |
|---|---|---|
| One inhabited Great Room | PASS | `wide_guild_is_one_great_room` and furnished empty-room rendering pass. |
| All wide landmarks | PASS | `wide_guild_hall_renders_every_operational_region` covers Door, Quest Wall, campaign tables, Counsel Bell, Hearth, Chronicle, Scrying and Spoils. |
| Multiple campaign tables | PASS | Projection identity tests and six/eleven-campaign rendering regressions pass. |
| Exactly one truthful representation per non-exited agent | PASS | The 4,096-case `every_non_exited_adventurer_appears_exactly_once` property passes; exited adventurers never project. |
| Full-body art only at truthful stations | PASS | Station-mapping unit/property tests restrict physical figures to Hearth and Spoils and projections to Counsel. |
| Stable wide, medium and narrow room identity | PASS | Exact 120/80/sub-80 mode boundaries, cropped campaign identity and deterministic landmark-camera tests pass. |
| Command parity | PASS | Interaction tests cover observe, counsel, acknowledge, refresh, search, Reviewr and view switching without duplicate loads. |
| Non-contradictory connection theatre | PASS | Typed-notice tests and `connected_room_never_renders_connecting_notice` pass. |
| Useful empty/disconnected/integration-unavailable states | PASS | Furnished empty hall, reconnect/offline overlays, all connection Storybook fixtures and quiet unavailable-Reviewr tests pass. |
| Startup and `HOMET PATH` regressions | PASS | Connected-notice and complete Delve route-home row regressions pass. |
| Exhaustive production-backed Storybook | PASS | Catalogue ownership, fixed scenes, all connection states, camera modes and exhaustive asset-family tests pass. |
| Guarded live candidate checks | BLOCKED | Safety preserved the pre-existing main-checkout link and pane; wide/80x24 candidate screenshots and live interactions require a clean or explicitly disposable Herdr session. |

The blocked live rows are environmental limitations, not automated product-test
failures. A follow-up interactive run should begin with a session where the
candidate link and managed pane are test-owned, then follow the procedure above
without weakening its cleanup boundary.

## Previous guarded acceptance record

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
only the two test-owned panes with `herdr pane read "$PANE_ID" --source visible
--format text` (and the separately captured managed pane ID).

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
