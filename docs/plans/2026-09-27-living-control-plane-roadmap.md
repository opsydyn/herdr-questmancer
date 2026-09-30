# Living control plane roadmap

Date: 2026-09-27. Roadmap and ownership contract approved 2026-09-30;
P0 interaction concept approved 2026-09-30. M1 is complete for its approved
fixture-backed scope; engineering qualification and production visual approval
are tracked in the
[M1 receipt](../reviews/2026-09-30-durable-obligations/README.md).
Direction: durable human obligations and campaign work context, with execution
and orchestration retained by their current owners.
[Design contract](../design/questmancer-living-control-plane.md).

## Success criteria

A human can identify the next obligation, why it matters and its affected work
in roughly five seconds. A decision survives unrelated progress, agent exit and
restart. Healthy activity stays quiet. Work remains readable without a live pane.
At fleet scale, grouping reduces navigation rather than requiring every actor
to fit in the Hall. Five-second comprehension is a human review target, not a
claim established by deterministic tests.

## Sequence

| Phase | Scope | Exit evidence |
| --- | --- | --- |
| P0: ownership and interaction | Review the design contract, identity rules, persistence extension and Quest Board concept | Recorded design decisions and reviewed interaction concept |
| M1: durable obligations | Fixture source, work projection, atomic restart cache, derived queue, `!`, selected-work parchment | Decisive restart scenario, reducer/property/runtime tests, full gate and separate visual approval |
| M2: War Table | Selected campaign dependencies, ownership/context, obligations and artifacts through existing RGB renderer | Deterministic composition, keyboard navigation, small-viewport fallbacks and visual approval |
| M3: first live source | M3a: pinned read-only export contract, faithful campaign/thread inventory and reconciliation; M3b: explicit request/session binding contracts | Separate contract, inventory and attention/binding receipts; isolated owned-resource evidence and stale/offline/restart checks |
| M4: Forge evidence | PR, revision-bound checks, review and merge observations, optional explicit navigation | No false delivery/completion under stale revision, re-run or missing evidence; visual review |
| M5: source knowledge | Attributed project knowledge through Librarian, fixed help retained | Lazy loading, source freshness and unavailable-source tests; help remains accessible |
| M6: historical reconstruction | Work journal/checkpoints, migrations, read-only historical model and time controls | Deterministic reconstruction, corruption recovery and zero command effects during replay |
| M7: second source and scale | Firstmate or another source, multiple homes, aggregate fleet navigation | Adapter conformance, source isolation, duplicate binding checks and 1/10/100 review |

M1 is detailed in [the first-slice plan](2026-09-27-durable-obligations.md).
Phases are individually bounded. Completion evidence and approval to begin the
next phase remain separate; this roadmap does not authorize all phases at once.
M3 may precede M2 after M1 if live usefulness outweighs graph work; record that
choice rather than silently expanding the current slice. Forge and knowledge
depend on an accepted source contract. M6 follows a separate journal design.

## M1 product proof

Fixture campaign: Replace Authentication Layer. Investigation is completed; API
migration is active; web migration has a human compatibility decision; integration
validation depends on both migrations. An illustrative PR is an artifact, with no
claim that it was opened remotely or that checks passed.

The decision stays visible after a progress update, worker exit and restart.
Only explicit matching source resolution closes it. Local seen/deferred intent
does not impersonate an answer. Dependency readiness remains a derived fact.

## Adapter strategy

Herdr remains the live agent/pane source throughout M1; no replacement of the
existing projection is scheduled. Herdr alone cannot supply durable quests or
decision semantics that it does not expose.

Herdr Projects is the proposed first adapter because it shares Herdr and has
durable projects, thread records, reports and operational groupings. Reinspect
its supported exports at a pinned revision before committing to implementation.
The 2026-09-30 [source assessment](../reviews/2026-09-30-herdr-projects-source/README.md)
found no coherent versioned project export at `0.2.34`. The
[M3 plan](2026-09-30-herdr-projects-adapter.md) proposes an exporter contract first,
then a reduced-capability inventory adapter; structured request and live-binding
acceptance are separate M3b work. Do not infer a structured question from a group
label or parse prose into answers.
Polling is allowed inside an adapter; rendering never polls upstream services.

Firstmate is the proposed second-source conformance case because keyed decisions,
worker state and hierarchy test the abstraction. Consume its authoritative folds
or exports rather than cloning their semantics. Do not read remote homes merely
because a path appears in source metadata. Sources without required semantics
advertise reduced capability and retain unknowns.

## Dependencies and existing work

The existing pane-header repair remains independently uncommitted, with native
visual and Windows checks incomplete in its dated receipt. This roadmap does not
qualify, publish or roll it out. Before M1, inventory the actual checkout and
record the implementation baseline; preserve unrelated changes and separate
new-feature checks from existing repair evidence. No new worktree, shared-server
restart or subagent workflow is implied.

## Deferred product ideas

No autonomous assignment or dependency activation, orchestrator controls,
memory authoring, merge buttons, generic workflow engine, giant actor grid,
global event ordering or exactly-once distributed execution. Artifact production
does not mint XP. Speculative completion and success inferred from disappearance
remain prohibited.

## Status ledger

- P0 documents drafted on 2026-09-27 and approved by the user on 2026-09-30.
- [Quest Board interaction concept](../design/reviews/2026-09-30-quest-board/README.md)
  approved on 2026-09-30; M1 implementation authorised.
- M1 implemented inline and engineering-qualified on 2026-09-30: 696 Rust tests
  across 59 runs, 28 shell tests, the 4,096-case property gate and release build.
  See the receipt for the dirty-checkout baseline and exact evidence.
- The user approved the corrected production Quest Board visuals on 2026-09-30.
  M1 is complete and unreleased. Native terminal interaction and live-source
  acceptance remain separate, unperformed checks.
- M2 was promoted by the user on 2026-09-30. The bounded
  [War Table plan](2026-09-30-war-table.md) and
  [receipt](../reviews/2026-09-30-war-table/README.md) track implementation and
  the separate production visual gate. Engineering qualification passed 707 Rust
  tests across 60 runs, 28 shell tests, the 4,096-case property gate and release
  build. The user approved production visuals on 2026-09-30. M2 is complete and
  unreleased within its fixture-backed scope; native terminal and live-source
  acceptance remain unperformed. Mapper/cache/runtime integration, M3b and M4–M7 remain unpromoted.
  No live adapter or replay journal is implemented.
- The user approved M3 source inspection and adapter planning on 2026-09-30.
  The pinned source assessment and M3 plan are written; no adapter/exporter code
  or live operational trial was authorised or implemented by that planning approval.
- The user subsequently authorised M3a.1 with “Let’s continue”. The experimental
  snapshot/exporter and strict decoder are implemented against synthetic source
  trees; [receipt](../reviews/2026-09-30-herdr-projects-exporter/README.md). Coverage
  and native identity remain explicitly unsupported for authoritative retirement/
  durable mapping. M3a.1 is engineering-qualified and complete for this synthetic
  proof, unreleased: 722 Rust tests/61 runs, 28 shell tests, property gate and release
  build passed.
- The user subsequently promoted the M3a.2 identity/coverage design slice.
  The [selected contract](../design/herdr-projects-identity-coverage.md) and
  [design receipt](../reviews/2026-09-30-herdr-projects-identity/README.md) settle
  registered/retained identity, locked partial coverage, omission guards and an
  atomic typed checkpoint. Native identity remains unsupported. This is complete
  as documentation; no mapping, cache migration or runtime code was added.
- The user subsequently promoted the M3a.2 mapper/v2 cache proof. It is implemented
  and engineering-qualified with 739 Rust tests/61 runs, 28 shell tests and the
  4,096-case property gate. The [receipt and state-cue review](../reviews/2026-09-30-herdr-projects-mapper/README.md)
  separate synthetic proof from native/live acceptance. The user subsequently
  [approved the state-cue images](../reviews/2026-09-30-herdr-projects-mapper/visual-approval.md).
- The user then promoted M3a.3 explicit, disabled-by-default selection, owned
  producer/acknowledgement and runtime fixtures. Root leases, identity latching,
  stale completion fencing, transient recovery and atomic restart are implemented
  under the [runtime plan](2026-09-30-herdr-projects-runtime.md). The
  [runtime receipt](../reviews/2026-09-30-herdr-projects-runtime/README.md) records
  qualification (754 Rust tests/62 runs, 28 shell tests, property gate and release
  build) and changed diagnostic images. The user subsequently
  [approved those images](../reviews/2026-09-30-herdr-projects-runtime/visual-approval.md),
  completing M3a.3 within synthetic scope. Installed-source compatibility,
  native input and live trials remain unperformed. The next
  [bounded acceptance plan](2026-09-30-herdr-projects-acceptance.md) is proposed,
  not promoted or executed.
  Full M3 attention/binding acceptance and M4–M7 remain unpromoted.
