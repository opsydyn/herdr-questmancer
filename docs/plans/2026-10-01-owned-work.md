# L1: owned work, explicit decisions and restart

Date: 2026-10-01. Direction accepted by the user's “Agreed” after the proposal to
own the project/work backend. Planning is authorised. L1a/L1b implementation,
installation, live testing and release are not promoted by this planning slice.
Work inline; preserve the current checkout. No worktrees or subagents.

Update 2026-10-03: the user subsequently approved L1a implementation. Its backend
and source proof are implemented; see the
[engineering receipt](../reviews/2026-10-03-owned-work-backend/README.md) for gates
and limits. L1b authoring, installation, live testing and release remain unpromoted.

The user subsequently approved L1b form and boundary review. The concrete
[authoring design](../design/questmancer-owned-authoring.md) and
[implementation proposal](2026-10-03-owned-authoring.md) now cover explicit setup,
campaign/decision pickers, five forms and recovery. This review changed documents
only; L1b implementation and production form visuals are not approved by it.

The [owned-work contract](../design/questmancer-owned-work.md) is the proposed
implementation specification. It extends the existing presentation/source split
without making the UI an execution engine. Herdr Projects qualification is parked.

## Milestone outcome

A human can create a campaign and quest, request and answer a decision, register an
artifact, and recover the records after restart through the existing Board/Table.
The backend owns those facts; Herdr owns live execution. Commands affect only this
owned source. External source records, local seen/defer and counsel keep their
existing ownership rules.

Proposed appetite: one local store, one owner, five commands and minimal forms.
No daemon, database, external writer protocol or autonomous orchestration. The
first approved implementation step is L1a only; the full user workflow requires
L1b and its separate presentation review.

## L1a: authoritative backend proof

1. **Red test first.** Add `tests/owned_work.rs` with an explicit temporary store
   and fixed input IDs/time. Create campaign/quest/decision, answer the matching
   incarnation, register an artifact, drop/reopen, then verify identical authoritative
   records and an owned source projection with no open answered decision. Establish
   the expected missing-backend failure before implementing.
2. **Pure command transition.** Add `src/owned_work/` with explicit state, typed
   commands, result/error and receipt types. Reuse shared source-qualified entities;
   keep answer detail authoritative. Enforce identity, references, revisions, text
   bounds, terminal decision incarnations and retry-payload equality. No I/O or clock
   reads in transition functions. No assignment/readiness/agent effects.
3. **Owned storage.** Add explicit create/open, versioned atomic snapshot, lifetime
   writer lock and protected-file behaviour. Commit state/answer/receipt together;
   acknowledge file and directory durability. Test pre-commit failure and
   post-replacement indeterminate outcomes using a narrow test I/O boundary, not
   a general pluggable persistence framework. Never promote `work-state.json`
   or Chronicle into authority. Reuse existing primitives only where their
   durability/error behaviour meets the owned contract.
4. **Local source.** Add `src/sources/local_work/` producing the existing Complete
   baseline/update types after a durable commit, with no live bindings. Compose it
   with the existing reducer and acknowledged cache handler in tests. Prove cache
   publication failure leaves store commit intact and reconciliation restores the
   view. Any required typed runtime/cache additions preserve Projects checkpoints
   and external source facts. Do not wire ordinary startup or add forms in L1a.
5. **Qualify and report.** Run affected domain/persistence/property suites, full
   `just verify`, `just property-test 4096`, release build and diff check. Record
   original file preservation and exact output. No install, shared server, source
   home, network service or live agent is needed. L1a completion is backend proof,
   not the complete product milestone.

Expected affected paths: new `src/owned_work/`, new `src/sources/local_work/`,
`src/lib.rs`, `src/sources/mod.rs`, and focused additions to `src/work/runtime.rs`,
`src/persistence/work_cache.rs`/worker only if required by the composed path.
Tests: new `tests/owned_work.rs`, existing `tests/work.rs`, source/runtime and cache
regressions as warranted. `Cargo.toml`/lock remain unchanged unless a concrete
dependency need is separately reviewed. UI/Herdr protocol and agent commands are
outside L1a.

## L1a decisive cases

- The create/request/answer/artifact records survive reopening the authoritative store.
- Seen/deferred remains local intent; progress/agent removal does not resolve a decision.
- Identical command retry returns the same receipt; changed-payload retry, stale
  revision and foreign-source command fail without writes.
- Decision answer and resolution cannot be persisted independently; a new request
  requires a fresh incarnation rather than reopening a resolved one.
- A second writer, malformed/future/oversized store and replaced identity fail
  without clobbering data; a missing selected store is not recreated implicitly.
- Commit acknowledgement loss is reconciled by the stored receipt; cache failure
  is reported separately and cannot undo authoritative success.
- Complete owned baselines affect only their source; Projects cache/checkpoint
  compatibility and legacy live/counsel behaviour remain green.

Property tests should cover sequences of accepted/rejected/retried commands and
reopen: revision monotonicity, unchanged rejection, stable entity identity,
no duplicate retry effects and no open resolved incarnation. Avoid tests that
merely mirror constructors or pixel coordinates.

## L1b: human authoring through Questmancer

After L1a evidence, draft/review minimal Board parchment forms for creation,
requesting, answering and artifact registration. Then promote their implementation:
explicit owned-store selection, backend lifecycle, bounded typed command requests,
committed/pending/indeterminate/rejected feedback, draft cancellation and readable
answered detail. The command owner is independent of Herdr connectivity and sends
committed events to the UI without file polling on animation frames.

Expected areas: configuration/startup, `src/terminal.rs` lifecycle,
`src/interaction.rs`, `src/ui/input.rs`, Board/selected details and feature-gated
Storybook. Preserve native portrait fallback, small viewport scrolling, `!`,
external read-only work, counsel drafts and the single RGB production renderer.
No keys or layout are approved merely by this plan.

Run the whole five-action workflow through real input, close/reopen and distinguish
automated/fixture proof from native/live acceptance. L1 is complete only when the
user workflow and restart are qualified within their recorded scope.

## Later sequence

L2: explicit lifecycle/dependency/assignment commands and evidence-bound Herdr
association. L3: Forge and lazy attributed knowledge. L4: journal/replay design.
L5: consider a separate planning/delegation module and external source conformance
after core use justifies them. Each needs its own bounded plan and promotion.
Projects/Firstmate are references; none of these phases requires installing them.


## L1b implementation update — 2026-10-03

The user subsequently promoted the reviewed implementation. Explicit setup and
remembered startup Open, the five-action Board forms, owned campaign/decision
pickers and checked answer detail are implemented. Dedicated serial ownership,
state v2 migration and independent save/view acknowledgement are in scope.
See the [L1b receipt](../reviews/2026-10-03-owned-authoring/README.md). Automated
input/restart evidence is separate from user visual approval, native use and release.
The initial planning and form-review paragraphs above record their original authority;
they do not negate this subsequent implementation promotion. L2–L5 remain unpromoted.


## L1b presentation approval and native gate — 2026-10-03

The user approved the production form images; the
[approval record](../reviews/2026-10-03-owned-authoring/visual-approval.md) pins them.
Native acceptance is unperformed: Computer Use refuses Ghostty for safety reasons.
The [bounded manual trial](2026-10-03-owned-authoring-native.md) is prepared, with
no operational resources created by its availability check. Do not infer native
or managed-pane acceptance from the existing PTY result. L2–L5 remain unpromoted.
