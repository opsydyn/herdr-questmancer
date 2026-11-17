# Living control plane roadmap

Date: 2026-09-27. Roadmap and ownership contract approved 2026-09-30;
P0 interaction concept approved 2026-09-30. M1 is complete for its approved
fixture-backed scope; engineering qualification and production visual approval
are tracked in the
[M1 receipt](../reviews/2026-09-30-durable-obligations/README.md).
Direction revised on 2026-10-01: own the durable project/work backend and its
human commands, with Questmancer as the interface and Herdr providing execution.
The user agreed to this direction, then approved L1a implementation on 2026-10-03.
The backend proof and subsequently promoted L1b implementation are present.
See the [L1b receipt](../reviews/2026-10-03-owned-authoring/README.md); production
form images are subsequently approved; native/live acceptance remains unperformed. The [owned-work contract](../design/questmancer-owned-work.md)
and [L1 plan](2026-10-01-owned-work.md) specify the next proposed slice.
See the concrete [L1b design](../design/questmancer-owned-authoring.md) and
[implementation proposal](2026-10-03-owned-authoring.md) for the current review target.
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
| L1a: owned backend proof | Authoritative local store, five typed human commands, durable decision answer, LocalWorkSource and composed cache/restart proof | Rejection/retry/indeterminate-commit tests, store reopening, source isolation and full engineering gate; no authoring UI yet |
| L1b: owned authoring | Explicit selection and minimal Board forms: create campaign/quest, request/answer decision, register artifact | Reviewed forms, complete human workflow and restart; separate native/live acceptance |
| L2: controlled work lifecycle | Explicit lifecycle, dependency and assignment commands; qualified Herdr association | No inferred execution/completion; stale/replaced session checks and human control |
| L3: Forge and knowledge | Revision-bound PR/check/review evidence and lazy attributed knowledge, fixed help retained | No false success under stale evidence; source availability and visual review |
| L4: historical reconstruction | Separately designed owned journal/checkpoints and read-only time controls | Deterministic reconstruction, recovery and zero command effects during replay |
| L5: justified expansion | Consider a separate planning/delegation module, multiple sources and fleet scale | Real-use evidence, explicit scope/authority, conformance and 1/10/100 review |

M1 is detailed in [the first-slice plan](2026-09-27-durable-obligations.md).
Phases are individually bounded. Completion evidence and approval to begin the
next phase remain separate; this roadmap does not authorize all phases at once.
P0/M1/M2 and the implemented M3a adapter remain recorded evidence, not work to
repeat. L1 replaces external-adapter qualification as the next product dependency.
L1a and L1b now implement the five-action workflow and restart. Production form
images are approved; native use remains the next acceptance gate for the user milestone.
Later stages do not follow automatically from agreeing to own the backend.

## M1 product proof

Fixture campaign: Replace Authentication Layer. Investigation is completed; API
migration is active; web migration has a human compatibility decision; integration
validation depends on both migrations. An illustrative PR is an artifact, with no
claim that it was opened remotely or that checks passed.

The decision stays visible after a progress update, worker exit and restart.
Only explicit matching source resolution closes it. Local seen/deferred intent
does not impersonate an answer. Dependency readiness remains a derived fact.

## Owned source and optional adapters

Herdr remains the live agent/pane source throughout M1; no replacement of the
existing projection is scheduled. Herdr alone cannot supply durable quests or
decision semantics that it does not expose.

The owned backend will be authoritative for our campaigns, quests, decisions and
artifact registrations. Its adapter feeds the existing reducer and UI. The current
work cache remains a cache, and external sources still own their own records.
No Projects or Firstmate installation is required to complete the owned milestone.

The completed Projects synthetic proof is retained, disabled by default. Its
installed-source trial, writer-fixture proposal and M3b contracts are parked.
The [source assessment](../reviews/2026-09-30-herdr-projects-source/README.md)
and [acceptance audit](../reviews/2026-09-30-herdr-projects-acceptance/README.md)
remain historical evidence with their original limits. Firstmate/Projects are
design references and optional future conformance cases. Their features do not
establish demand for an owned orchestration engine. Rendering never polls source
storage or services, and group labels never become invented human questions.

## Dependencies and existing work

Accumulated work, including the earlier pane-header repair, is committed at
`4923a05`; the prerequisite audit is committed at `9c52761`. Pane-header native
visual and Windows checks retain the limits in their dated receipt. Current
source wins over historical implementation baselines. The owned planning slice
changes no runtime behaviour and authorises no installation, shared-server
restart, new worktree, subagent workflow, release or imported user data.

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
  [bounded acceptance plan](2026-09-30-herdr-projects-acceptance.md) was promoted
  by “Commit and move to next”. Its prerequisite audit is complete, but the
  operational trial stopped before resource creation; see the
  [receipt](../reviews/2026-09-30-herdr-projects-acceptance/README.md).
  The accumulated work is committed at `4923a05`. The next
  [upstream-writer fixture proposal](2026-09-30-herdr-projects-writer-fixtures.md)
  is unpromoted and does not establish installed-source/live acceptance.
  Full M3 attention/binding acceptance and M4–M7 remain unpromoted.
- On 2026-10-01 the user said they wanted our own version for full control and
  agreed to owning the project/work backend. The active sequence is now L1–L5
  above. Projects installed-source acceptance, writer fixtures and M3b are parked;
  its implemented opt-in adapter and original qualification remain intact.
  The [owned-work contract](../design/questmancer-owned-work.md) and
  [first milestone plan](2026-10-01-owned-work.md) are written. L1a/L1b implementation
  remains unpromoted. The [planning receipt](../reviews/2026-10-01-owned-work-plan/README.md)
  records source preservation and static checks separately from earlier engineering gates.

- 2026-10-03: the user approved L1a. The authoritative command/store/source proof is
  implemented and its full gate passes 775 Rust tests/63 runs and 28 shell tests.
  See the [backend receipt](../reviews/2026-10-03-owned-work-backend/README.md).
  Ordinary startup and human authoring remain unimplemented; L1b form review is
  the next proposal. No commit, installation, live operation or release occurred.
- 2026-10-03: the user subsequently approved L1b form and boundary review. The
  [form design](../design/questmancer-owned-authoring.md) and
  [implementation proposal](2026-10-03-owned-authoring.md) are written; the
  [review receipt](../reviews/2026-10-03-owned-authoring-plan/README.md) records
  documentation-only validation. L1b implementation, production form visuals and
  native interaction remain unpromoted. Earlier backend code/evidence is preserved.

- 2026-10-03: the user subsequently promoted L1b implementation. Explicit store
  selection/startup, owned pickers, five forms and checked answered detail are
  implemented. The [receipt](../reviews/2026-10-03-owned-authoring/README.md) separates
  engineering and actual-input PTY evidence from unapproved form visuals, native
  terminal use, installation and release. L2–L5 remain unpromoted.

- 2026-10-03: the user approved the L1b production form images. A fresh Ghostty
  availability check was refused by Computer Use for safety reasons. The
  [native receipt](../reviews/2026-10-03-owned-authoring-native/README.md) records no
  operational resources or native acceptance; the [isolated manual plan](2026-10-03-owned-authoring-native.md)
  is prepared. Existing engineering evidence is preserved, and L2–L5 remain unpromoted.
