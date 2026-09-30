# M1: durable obligations and campaign context

Date: 2026-09-27. Plan and ownership contract approved by the user 2026-09-30.
M1 is complete for its fixture-backed scope: engineering qualification passed
and the corrected production visuals were approved on 2026-09-30. See the
[M1 receipt](../reviews/2026-09-30-durable-obligations/README.md).
The [design contract](../design/questmancer-living-control-plane.md)
is accepted; the [Quest Board interaction concept](../design/reviews/2026-09-30-quest-board/README.md)
was approved by the user on 2026-09-30, authorising M1 implementation.
[Roadmap](2026-09-27-living-control-plane-roadmap.md).

## Scope and decisive acceptance

One deterministic FixtureSource supplies one work campaign, four quests, one
human decision and one illustrative PR artifact. Integrate a derived attention
queue with the existing `!` control and a selected-work parchment. Persist cached
work and local attention intent atomically. Keep Herdr and its current domain live.

Acceptance sequence:

1. Load a coherent fixture baseline and request the compatibility decision.
2. Apply worker progress and newer work facts. The decision remains open.
3. Remove the bound live worker. Work, artifact and decision remain readable;
   observing the absent pane is disabled.
4. Acknowledge/defer locally, flush, stop and restart using temporary storage.
5. Restore the obligation with its intent and cached/unverified source status.
6. Reconcile a matching baseline without duplicating or reopening resolved work.
7. Apply the exact source resolution. Remove the active queue target and retain
   the resulting work evidence. A stale close cannot settle a newer incarnation.

This is synthetic work-source proof, not live Firstmate/Herdr Projects acceptance.
No War Table, Forge lifecycle integration, project knowledge reader, journal time
travel, remote writes, automated answers or new completion rewards belong to M1.

## Implementation boundaries

Proposed locations, confirmed against source again before editing:

| Area | Location and change |
| --- | --- |
| Domain | New `src/work/` modules for IDs, entities, source facts and pure reducer |
| Ingestion | New `src/work/source.rs` and fixture module; internal versioned baseline/update contract |
| Runtime | `src/runtime_loop.rs`: independent work-source task/effects, no animation I/O |
| Model | `src/app.rs`: separate work projection and attention target selection |
| Attention | Pure combined queue module; reuse existing legacy urgency rules |
| Persistence | New work-cache module under `src/persistence/`, separate schema/file and acknowledged atomic writes |
| UI | Existing overlay/input/scene projection boundaries for Quest Board and details |
| Storybook | Feature-gated work fixture/interaction stories using production paths |
| Tests/docs | New focused work tests, affected runtime/persistence/input/scene suites and operational recipe |

Do not rename the current Campaign, replace the live reducer, extend the legacy
Chronicle enum into an authoritative work log, or add a second production renderer.
FixtureSource is test/Storybook-only; ordinary startup has no synthetic campaign.
Production starts with an empty work projection and unchanged legacy navigation
unless explicitly configured with a qualified future work source.

## Task 0: baseline and interaction checkpoint

- [x] Read current AGENTS.md, status/log, model, input, reducer, persistence and
  render boundaries; record existing dirty work and actual base revision.
- [x] Confirm SourceId qualification, WorkCampaign naming, source resolution,
  separate work cache and durable local acknowledgement/defer semantics.
- [x] Present a concrete Quest Board concept in existing parchment style: queue,
  literal reason, campaign/quest detail, source/freshness, artifact and optional
  observe action. Include empty, narrow and worker-exited cases.
- [x] Record visual/interaction review separately from domain-design review.
  User approved the concrete concept on 2026-09-30; production images have a separate gate.

## Task 1: identities, facts and pure projection

- [x] Start with a focused failing test: source-qualified duplicate local IDs
  remain distinct and a worker exit cannot remove its obligation.
- [x] Add minimal WorkCampaign, Quest, Obligation and Artifact types. Separate
  source lifecycle, execution/delivery facts and attention intent.
- [x] Add coherent baseline and ordered update reducer with explicit tombstones,
  identity/revision validation, source generation and deterministic diagnostics.
- [x] Test identical duplicates, conflicting equal revisions, older generations,
  partial baseline coverage, dependency cycles/missing references and exact resolution.
- [x] Property-test inert duplicate application and preservation of open obligations
  through unrelated work updates; exercise actual Herdr exit in the composed test.
  Avoid tests that merely restate enum values.

## Task 2: complete fixture and source lifecycle

- [x] Red test: disconnect and reconnect cannot turn last-known open work into success.
- [x] Build deterministic FixtureSource with injected times and ordered scenario steps.
  It supplies events/baselines; reducer and rendering contain no fixture branches.
- [x] Author the four-quest auth scenario and explicit worker/session bindings.
  Preserve an unbound-work case; a title/path match never creates a binding.
- [x] Wire an independent work-source runtime channel. Pause or reconcile on
  cursor gaps; reject old-generation results; shutdown owns only its own task.
- [x] Verify no work-source traffic from animation wakes, no output reads to
  infer work facts and no synthetic state in ordinary startup.

## Task 3: restart-safe cache and local intent

- [x] Red test the complete restart scenario with temporary `work-state.json`.
- [x] Implement a versioned atomic snapshot containing work facts, accepted source
  checkpoint, entity/incarnation identities and local seen/defer intent.
- [x] Correlate reducer/runtime state with acknowledged persistence. Define the
  pending-write state and block false durable-success claims after write failure.
  On failure, preserve the last durable checkpoint and reconcile before accepting
  further durable updates. Inject publication and acknowledgement failures; verify
  old bytes survive. Abrupt process-death simulation remains unperformed.
- [x] Restore source facts as cached/unverified, preserve matching local intent
  and re-enable authoritative updates only through valid baseline/cursor recovery.
- [x] Test future-version protection, malformed snapshots, failure before atomic replacement,
  unchanged-write suppression, write failure and acknowledged shutdown/restart.
  This is fault injection, not power-loss or abrupt-process-death acceptance.
- [x] Prove legacy state and mixed v1/v2 Chronicle files are not rewritten, and
  retained-history truncation does not lose current work obligations or award XP.

## Task 4: combined attention queue and selection

- [x] Red test: `!` reaches the open decision without a live agent; legacy-only
  cycling still works and an explicitly deferred target is skipped until expiry.
- [x] Add typed attention targets for obligation or legacy agent episode. Keep
  source owner/state separate from local seen/deferred intent.
- [x] Apply the deterministic rank/age/identity policy from the design. Do not
  escalate source waits owned by a supervisor or external dependency.
- [x] Collapse legacy/source duplicates only with explicit episode correlation.
  Test that similar prompts or reused panes never collapse unrelated obligations.
- [x] Exercise zero/one/many items, equal times, removed selections, worker exit
  and deferred expiry with an injected clock. The M1 protocol requires timestamps;
  future adapters must assign and retain a first-observed fallback when absent.
- [x] Integrate selection without breaking counsel drafts, uncertain delivery,
  scrying or managed-pane exclusion. Observe only a currently verified binding.

## Task 5: Quest Board parchment and truthful details

- [x] Red interaction/render test: selected work-only obligation displays its
  literal reason and source freshness, with no unavailable pane action.
- [x] Implement reviewed keyboard navigation and contextual overlay through the
  existing rendering path. Local seen/defer labels must not imply answered/resolved.
- [x] Display campaign objective, quest lifecycle, dependency facts, assignment
  history and artifact reference. Unknown validation is visibly unknown.
- [x] Register bindings in the fixed handbook's binding table and update help.
  Preserve fixed Ledger access and all existing overlay dismissal/draft behaviour.
- [x] Add Storybook stories for populated, empty, offline, deferred, resolved,
  narrow and worker-exited states. Do not require live services for visual review.
- [x] Obtain production visual approval independently from automated rendering checks.
  Nine production-path exports are retained. The user approved the corrected
  production Quest Board on 2026-09-30; native terminal review remains separate.

## Task 6: qualify the composed slice

- [x] Compose actual fixture source, runtime, reducer, persistence, restart and
  queue selection in one integration suite, including injected write failures.
- [x] Replay the fixture into an empty projection twice; assert equivalent state
  with a fixed time/environment. Re-deliver accepted updates into a populated
  projection; assert no duplicate entities, obligations or effects.
- [x] Run affected reducer/property, runtime, persistence, input, help, scene and
  Storybook suites, then `just property-test 4096`, `just verify`,
  `cargo build --release` and `git diff --check`.
- [x] Use 1/10/100 synthetic work/attention inventories to inspect bounded queue
  and overlay work; record latency measurements and actual rendered actor limits.
  Do not claim five-second human comprehension from performance measurements.
- [x] Update AGENTS.md/PLAN.md and operating docs to describe the accepted cache
  exception, new controls and remaining gates. Retain a dated receipt with exact
  revision/dirty status, commands, test outcomes, failures and visual review.
- [x] Report M1 complete only when required engineering and visual evidence is
  satisfied; otherwise distinguish implementation from remaining qualification.

## Qualification scope

The dated receipt records passing engineering checks against the dirty checkout.
The work-only composed test creates no legacy files; existing mixed v1/v2 Chronicle
composition and atomic-state suites pass in the same full gate. No legacy schema
migration or Chronicle-dependent work reconstruction is implemented. Injected
persistence failure proves old checkpoint preservation, not every physical crash
boundary. The user approved the corrected production visuals on 2026-09-30,
completing the required M1 product gate. Native terminal acceptance remains separate.

## Regression matrix

| Scenario | Required behaviour |
| --- | --- |
| Decision then progress/idle/exit | Decision remains open; optional binding becomes unavailable |
| Same upstream ID in two sources | Independent entities and obligations |
| Duplicate, stale, conflicting revision | Inert duplicate/stale; conflict requires reconciliation |
| Complete versus partial baseline | Covered inventory may retire absent items; partial data cannot |
| Source disconnect, gap, generation change | Cached facts remain attributed; no guessed success/resolution |
| Seen/deferred versus explicit resolution | Only source close settles source state |
| Matching versus newer obligation incarnation | Old resolution/intent cannot settle or hide a new request |
| Restart beyond Chronicle display limit | Work and local intent survive independently of history |
| Failed/corrupt/future-version persistence | Diagnostic, protected prior state and no false durability claim |
| Dependency finished or missing/cyclic | Derived evidence only; no task activation or assignment |
| Artifact produced; check for an old head | No completion/XP; validation does not apply to a new revision |
| Source obligation + legacy summons | Collapse only explicit matching episodes; legacy fallback retained |
| No live binding / managed pane binding | Work readable; no invalid focus/output/counsel command |
| Render tick / historical fixture replay | No upstream read, persistence write or external command from rendering |

## Completion and next boundary

Each behaviour task proves its expected failing test before implementation,
then focused green and affected regression checks. Work inline, preserve existing
changes, and do not start subagents or worktrees without user direction.
The subsequent 2026-09-30 concept approval authorised M1 implementation.
Commit, release, plugin installation and shared-server changes remain separate. After M1 qualification, recommend M2
War Table or the bounded M3 read-only adapter with its reason; wait for promotion.
