# Chronicle capture implementation plan

Status: C1/C2 are implemented locally; C3 was approved and its automated and
isolated synthetic qualification passed on 2026-09-09. The user approved the
Chronicle screens on 2026-09-09; C1–C3 are complete locally. No release is authorised by this plan.
Source baseline: `a26386c` / published 0.1.9.
The [design](../design/questmancer-chronicle-capture.md) owns capture semantics.

## C1 — Qualify snapshot observations

Introduce epoch, snapshot purpose, request correlation and live-fact generation
through supervisor, command/result and runtime boundaries. Keep capture disabled
for baselines, stale results and ambiguous incarnations. Coalesce one active plus
one pending refresh; after two superseded results, resubscribe and quietly
baseline. Pass typed context into pure reduction. Preserve status reconciliation,
managed-pane exclusion, local intent, selected-output ownership and rendering.
Snapshot cancellation must not cancel or block the independent counsel/focus
command set or selected-output tasks.

First red test: a refresh requested before disconnect resolves after a fresh
baseline and must neither replace the new party nor append history. Then cover
opposite-order refreshes, same-epoch status updates overtaking a snapshot,
subscription topology churn, cancellation and the bounded recovery path.

C1 is implemented; see the [qualification receipt](../reviews/2026-09-09-chronicle-c1/README.md).
Refresh qualification also requires the pane set established by the current
subscription baseline. A changed set immediately resubscribes; obsolete results
cannot trigger that recovery. Lower revisions, conflicting equal revisions and
duplicate agent identities cannot replace current facts. At the C1 checkpoint, snapshot capture remained disabled. C2 below activates
qualified refresh observations while preserving quiet baselines.

## C2 — Capture typed observations with compatible replay

Add the dual v1/v2 reader, immutable observation identity and typed source/payload
projection. Preserve legacy records; add current-epoch observations, explicit
unknown whereabouts and evidence-qualified campaign removal. Keep all new kinds
at zero XP. Use shared acceptance guards so status and snapshot observations do
not record or reward the same state twice. Add chapter/record copy in this slice
so newly written kinds are readable immediately. Do not emit `CampaignClosed`
from workspace removal, change persona assignment or rewrite the old JSONL.

First red test: a known working subject becomes blocked in an eligible snapshot;
exactly one zero-XP observation is appended with the observed wording. Repeating
the snapshot or receiving matching status metadata adds nothing.

C2 is implemented; see the [local receipt](../reviews/2026-09-09-chronicle-c2/README.md).
All new production writes use typed v2 source evidence. Complete session identity
and terminal incarnation qualify subject history; missing/ambiguous identities
establish quiet subject baselines. Raw identity stays ephemeral. Workspace hints
are consumed by the next accepted snapshot and expire at connection boundaries.
Scrying and the cat's party comparison also respect terminal replacement. The
[manual recipe](../manual-test/chronicle-capture.md) records what C3 must qualify.

C3 is qualified below and its Chronicle screens are visually approved.
Committing and clean-commit release qualification remain separate steps.

## C3 — Qualify the complete capture path

Exercise real supervisor/runtime/reducer/persistence composition against fake
Herdr sockets and temporary files. Verify the two request channels cannot bypass
C1, and every new persisted record has valid source context. Review the Chronicle
record/chapter screens with the new categories, then use an isolated Herdr server
and owned synthetic identities for working/blocked/idle/unknown observations.
The real protocol cannot synthesize explicit done; completion remains a fixture
case unless independently observed on a real agent with user authorisation.

C3 qualification passed: four real socket/runtime/worker composition tests, the
full gate, and an isolated Herdr 0.9 run with four owned dummy Pi processes. Five
metadata observations were persisted; matching reports and restart added no
history. See the [C3 receipt](../reviews/2026-09-09-chronicle-c3/README.md) and
[production screen review](../design/reviews/2026-09-09-chronicle-capture/README.md).
These are synthetic observations, not real-agent completion or live snapshot-only
acceptance. The latter source path is qualified by composed socket fixtures.

## Required regression matrix

| Scenario | Expected result |
| --- | --- |
| Startup, reconnect, resync already blocked/done | Live facts updated; no appends or XP |
| Same epoch, working to blocked in eligible refresh | One presence observation, zero XP |
| Same state at newer revision or changed custom text | No presence history |
| Event first, matching refresh second; reverse order | One accepted change; no duplicate reward |
| Conflicting equal or older revision | Reject/reconcile; no alternating history |
| Slow snapshot overtaken by status; old epoch result | No rollback, no false record; bounded retry/resync |
| Buffered status before baseline | Current metadata reconciliation; no counterfeit completion |
| New identity first observed blocked/done | Membership observation only; no guessed prior lifecycle |
| Same session moves pane; pane reused for different session | Preserve real continuity; no cross-incarnation revision comparison |
| Ambiguous fallback identity / duplicate AgentKeys | No invented subject history; bounded diagnostic |
| Unknown status versus legacy joined record | New explicit unknown category; old ID/summary retained |
| Missing pane versus confirmed exit | Visibility observation versus departure; neither means success |
| Campaign close hint with workspace still present | No removal record; fresh reconciliation |
| Two campaigns removed at one timestamp | Distinct IDs; one record each, zero XP |
| Removed campaign with several agents | No manufactured per-agent departures from that same absence |
| Wall-clock rollback or equal timestamps | Stable immutable IDs and deterministic source order |
| More than 500 retained events then a duplicate status | Current-fact guard still suppresses duplicate/reward |
| Mixed v1/v2, unknown version, malformed/truncated line | Valid records retained; bounded diagnostics; no rewrite |
| Restart and history replay | No re-awarded standing; live baseline creates no backfill |
| Chronicle append failure | Existing diagnostic/acknowledgement behaviour; no false durability claim |
| Managed pane, metadata refresh and animation frames | No capture, output reads or extra wakes |

## Verification and completion gates

Each behaviour slice starts with its focused failing test, proves that failure,
implements the smallest coherent path and runs affected integration/property
tests. Relevant suites are `reducer`, `chronicle`, `chronicle_chapter`,
`chronicle_persistence`, `event_adapter`, `supervisor`, `runtime_loop`,
`persistence_worker`, `property_domain`, `scene_overlays` and `scene_runtime`.
Run `just verify`, `cargo build --release`, package checks and `git diff --check`
before claiming the implemented phase complete. Keep records, manual recipes,
release notes and operating guidance in the slices they describe.

Visual approval covers changed Chronicle copy/layout only. Isolated fixtures do
not establish real-agent completion, complete upstream history, cross-file
atomicity or release publication. A later release requires its own clean commit,
qualification, explicit publication authority and published-installer check.

## Explicitly deferred

Persistent topology, upstream event replay, global exactly-once guarantees,
transactional lifetime awards, successful-campaign detection, trophies/mementos,
new room art and any remote service are outside this plan.
