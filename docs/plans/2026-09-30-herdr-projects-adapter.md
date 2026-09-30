# M3: Herdr Projects source contract and read-only adapter

Date: 2026-09-30. **Source inspection/planning, M3a.1 snapshot/exporter proof and
the subsequent identity/coverage design slice authorised. The user later approved
mapper/cache implementation and its production visuals. M3a.3 runtime integration
was then promoted, engineering-qualified and visually accepted within synthetic
scope; see its [bounded plan](2026-09-30-herdr-projects-runtime.md).** M1/M2 are accepted within fixture-backed scope and
unreleased. Work inline on the current dirty `main`; preserve pane-header work.
The [source assessment](../reviews/2026-09-30-herdr-projects-source/README.md)
pins Herdr Projects `0.2.34` at `4e4548c3c43888e6be1c96906215998f07dc63f0`.
This plan makes no installation, shared-server, upstream contribution or live
agent authorisation implicit.

## Problem, outcome and appetite

The Board/Table currently prove their behaviour with fixture/cache data. The
human cannot yet open a real Herdr Projects campaign there. The current
alternative is Herdr Projects' own overview/JSON listing and manual inspection
of its reports. Whether a second presentation reduces effort remains a hypothesis.

Proposed appetite: one source-contract slice before any runtime integration,
then one opt-in single-project read-only adapter. End the first slice in learning:
prove what can be read coherently and reject unsupported semantic claims. End
the adapter slice in a working campaign/thread/report path through existing UI.
Do not spend this appetite adding a general orchestration protocol, graph
heuristics, remote transport or source-owned question storage.

## Scope and capability boundary

The inspected source has no coherent versioned campaign export. Introduce a
small snapshot format between a pinned source-facing exporter and the consumer:

```text
Explicitly selected Herdr Projects home/project
  -> strict, bounded, read-only snapshot exporter
  -> versioned source DTO with coverage/provenance
  -> pure mapping and adapter observation ordering
  -> existing SourceInbox / work reducer / work cache
  -> existing Board and War Table
```

The exporter may understand upstream storage; the work reducer and UI may not.
Implement it as an isolated source module/companion, not a copy of the upstream
orchestrator. Prefer a supported upstream export if one becomes available; its
addition/merge is a separate task. The first contract is internal and versioned,
not a claim that a shared `campaign.*` protocol exists.

First supported capabilities: campaign metadata/lifecycle, retained thread
inventory, source assignment labels, literal last-recorded group/error/reason,
home report and PR references. No dependency declarations exist in the inspected
thread/task types; declare that capability unavailable and supply no invented
edges. Unsupported work stays readable without a live pane.

Initially omit structured decisions, permission episodes, human review requests,
remote reads and live bindings. Do not map WaitingOnYou into a Decision, or
interpret `## Next`, an inbox item, 100%, Done or a PR merge as an answered
request/completed quest. Legacy Herdr summons remain available through `!`.
This is a deliberate narrower M3a; full M3 binding/attention acceptance is not
satisfied by inventory success. M3b requires the later contracts below.

## Proposed snapshot contract

Freeze an example/schema and conformance fixtures before runtime wiring.

- Header: schema version, supported upstream format/revision, configured source
  token, configured campaign token, adapter capture identity/time, coverage,
  capability flags and typed diagnostics. Unsupported version stops ingestion.
- One explicitly configured project per source namespace for this slice. Source
  identity distinguishes installation/project scopes, not only adapter kind.
  Tokens are user/configuration supplied opaque IDs; path, slug, display name,
  branch and current agent name never mint durable identity.
- The subsequent [identity/coverage decision](../design/herdr-projects-identity-coverage.md)
  selects explicit registration plus retained records. A validated `t-NNNN` is
  scoped to that registration; creation is checkpoint evidence, not a native
  incarnation. Changed creation evidence rejects the candidate and requires
  reconciliation; do not silently mint a new quest from its timestamp.
  Rename requires an explicit path rebind retaining the configured token;
  disappearance does not automatically bind a different same-named directory.
- Typed project record: name, goal and literal active/paused/archived status.
  Typed thread record: ID/incarnation, title, status, closure reason/error,
  recorded grouping and update time, profile/agent label, report hash/ack hash,
  optional PR URI and bounded home-report reference. Keep placement separate
  from work kind. Do not export whole memory, command flags or terminal output.
- Reads are limited to the selected project and allowlisted regular files.
  Reject symlinks/path escapes, malformed or oversized files, duplicate IDs,
  unknown enum variants and a filename/record-ID mismatch. Do not follow
  `thread_dir`, repository paths, source shell commands, URLs or remote machines.
- Bound reading and concurrency. Use the existing project lock only if it can
  be opened read-only and acquired without creating/truncating a file; prove
  the locking contract against upstream writers. Never hold it across external
  I/O. A missing lock remains valid inspection-only output and cannot refresh
  durable work; never make the lock or claim a complete snapshot. Atomic per-file rename and
  equal directory/mtime reads alone do not prove transactional coverage.
- Source CLI list helpers silently omit bad records; the exporter must not.
  Publish no changed batch when a required record fails. Preserve the accepted
  cache and mark unavailable/reconciliation required. Retain exact diagnostics.
- Initial coverage is partial and omissions have no retirement authority.
  Explicit known-entity disappearance invalidates the observation rather than
  presenting retained entities as current. No `complete: true` baseline or
  omission-based request closure until coherent inventory coverage is proven.
- Export capture time is local observation time. Upstream update strings,
  hashes and inbox counters are not a global revision/event cursor. The adapter
  maintains its own ordered observation generation/cursor and entity revisions,
  anchored to the accepted work-cache checkpoint. Label them as observer order;
  do not claim source replay or unseen intermediate transitions. Unchanged facts
  reuse revisions; changed normalized facts increment them. Reconnect/restart
  discards old in-flight reads and starts a fresh generation beyond the cache.
  Counter exhaustion/conflict requires reconciliation, never wrap/reset.

## Minimal model compatibility proposal

Current enums cannot represent the inspected source faithfully. The implementation
proposal must add neutral `QuestKind::Unspecified`, `QuestState::Unknown` and
`QuestState::Closed`, plus `CampaignState::Archived`, before this mapping:

| Source fact | Work projection |
| --- | --- |
| active / paused / archived project | Active / Paused / Archived campaign |
| starting / open thread | Unknown work lifecycle, with the exact source thread status in details |
| failed thread | Failed, explicitly labelled as a source thread/launch failure with its error |
| resolved thread | Closed, with the literal closure reason; no successful outcome asserted |
| placement worktree/tab/checkout/adopted | Source context only; quest kind remains Unspecified |
| percent, activity, recorded group | Attributed source context only; no readiness/completion/decision inference |
| home report | Latest Report reference slot when allowlisted bytes match; relative reference/hash in context, no filesystem URI |
| recorded PR text | Latest PullRequest reference slot; bounded literal context, URI only under the selected allowlist, no head/check/review validation |

These generic unknown/closure values preserve the orchestration-neutral core.
Do not add Herdr Projects placement/group enums to it. Details must identify
last-recorded context and unsupported dependency/attention/binding capabilities.
Render Closed without a success check/celebration, and Unknown without a working
cue. No new station/world renderer or art is needed.

Review cache compatibility in the same slice: new enum values make v1 snapshots
unreadable to old binaries. Propose work-cache v2, migrate valid v1 on read without
loss, preserve future-version/malformed-file protection and acknowledge atomic
publication. The selected contract requires a bounded typed registration/creation
checkpoint in the same atomic envelope as the projection. Existing projection
generation/cursor/entity revisions supply ordering; do not duplicate them. Do not
change `state.json`, Chronicle v1/v2, standing or XP. No work journal or arbitrary
adapter map is required.

## Consumer and runtime proposal

Add an explicit disabled-by-default source configuration: configured IDs,
absolute selected project path and pinned format. No home discovery or automatic
plugin installation/version update. Runtime owns a cancellable producer feeding
`SourceInbox`; the pure mapper takes a validated DTO and prior checkpoint.

Proposed polling: one source capture at a time, five seconds while healthy,
bounded backoff up to thirty seconds on failure, explicit timeouts/cancellation.
Confirm actual costs with fixture captures at 1/10/100 threads. No per-frame,
per-card or selected-output polling; no shell process per thread. Emit changed
observations only; do not persist an unchanged inventory every polling interval.
Keep last successful capture/probe diagnostics distinct from domain revision.

Reconnect/offline/error clears bindings and retains cached work as unverified or
unavailable. Parsing/version/coverage failure cannot close a quest or obligation.
Source producer failure is independent from Herdr supervisor recovery. Shutdown
cancels and joins owned work tasks, drops the sender and flushes acknowledged
cache/intent work. No external source files are written and no ticker is managed.

## Later M3b contracts, explicitly not supplied by M3a

A report-review episode is a plausible first human obligation because the source
has report/ack hashes. Specify when a report becomes ready, its stable request
identity, matching acknowledgement resolution, changed-report supersession and
reopen before implementation. Ordinary progress/working/idle cannot close an
already-open episode; source `thread ack` must not be conflated with Questmancer
Space/seen. Never send `thread ack`, resolve, keys or prompts from the adapter.
Questions need a literal prompt, responsible actor, stable incarnation and
explicit resolution assertion from their owner; grouping is insufficient.

Live observation additionally requires explicit same-server and thread-incarnation
binding, current terminal plus the complete Herdr session tuple, and uniqueness.
Reject managed Questmancer/Codex/unknown panes, remote panes, reused IDs, ambiguous
adopted matches and a changed terminal/session. Preserve `bound_agent`'s existing
fingerprint check; do not replace it with upstream name/path matching. Restart
never restores bindings. Fresh inspection of actual Herdr 0.9.1 compatibility is
required; do not change Questmancer's protocol or upgrade a shared server silently.

## Implementation sequence and gates

1. **M3a.1 contract/exporter proof (complete, unreleased).** Freeze DTO/schema,
   capability matrix and synthetic source-tree fixtures. Test-first strict reads,
   identity/coverage, malformed file retention, unknown versions, path bounds and
   source-byte invariance. Prove or explicitly fail concurrent-write consistency
   with test-owned writer processes; include missing-lock and record-replacement
   cases. Deliver exporter/decoder evidence only; no live integration or full M3 claim.
2. **M3a.2 identity/coverage and mapper/cache proof (engineering-qualified).** The
   selected contract uses explicit registration/retained records and partial
   coverage; native incarnation guarantees remain unsupported. Require a locked
   observation and guard all known-thread omissions before reduction. Add neutral states and v2 migration with
   focused red/green tests. Prove closure is not completion, unavailable requests
   are not invented, report hash mismatch is not passed validation, unchanged
   observations are inert, and duplicate/stale reads cannot replace newer facts.
3. **M3a.3 opt-in runtime/UI (promoted and implemented).** Own producer/configuration/shutdown, attach through
   SourceInbox, expose capabilities/freshness and use existing Board/Table details.
   Composed test: baseline -> title/report change -> disconnect -> restart cache
   -> valid new baseline. No source writes/commands, replay, XP or execution effects.
4. Run affected domain/property/persistence/runtime/scene suites, `just verify`,
   `just property-test 4096`, release build and diff checks. Export changed
   production surfaces for user visual approval. Automated gates prove contracts,
   not live usefulness, native input or five-second comprehension.
5. After separate live-test authorisation, qualify one disposable local project
   and owned Herdr server/panes, with the guarded recipe and fresh IDs. Baseline
   binaries/protocol, inventories and file hashes; exercise disconnect/restart
   and cleanup only owned resources. Keep any shared-server change separately
   approved. Real agent launches/messages require explicit authorisation.
6. Promote M3b separately if source request/binding contracts become available.
   M3a inventory acceptance must not be labelled full M3 attention/binding acceptance.

## Stop or reshape conditions

Stop authoritative delivery if the selected registration policy or known-entity
coverage checks cannot be established without source mutation. If safe coherent reads require a native
upstream exporter, recommend that as a separate prerequisite rather than publish
false complete baselines. An exporter emitting only partial facts can be accepted
only with visibly reduced capability and the omission policy above.

Do not expand M3a to infer structured decisions, dependencies or success merely
to populate the accepted UI. Keep native upstream export availability, actual
installed-source compatibility, concurrent-write behaviour and human navigation
benefit as unresolved evidence. The user owns promotion of implementation and
any later operational trial. M4–M7 remain unpromoted.

## M3a.1 implementation status

The user subsequently said “Let’s continue”, authorising the bounded first proof.
`src/sources/herdr_projects/` now implements the experimental read-only DTO/exporter
and strict decoder. Coverage remains Partial and native durable identity is
explicitly unsupported; no work mapping, source attachment or persistence change
is implemented. See the [receipt](../reviews/2026-09-30-herdr-projects-exporter/README.md)
for engineering evidence: 722 Rust tests across 61 runs, 28 shell tests, the
4,096-case property gate and release build passed. M3a.1 is complete for its
synthetic contract-proof scope and unreleased. Full M3 acceptance remains outstanding.

## M3a.2 mapper/cache implementation proposal

The user subsequently approved settling identity/coverage rules. The
[selected design contract](../design/herdr-projects-identity-coverage.md) and
[design receipt](../reviews/2026-09-30-herdr-projects-identity/README.md) record that
documentation-only slice. The user subsequently approved the following code work.
It is implemented; the [mapper receipt](../reviews/2026-09-30-herdr-projects-mapper/README.md)
records final engineering evidence; its production visual gate was subsequently
[approved](../reviews/2026-09-30-herdr-projects-mapper/visual-approval.md).
It proves mapping and atomic restart continuity with synthetic input; it does
not attach a source, poll a real home or add a new production surface.

1. **Pure registered mapping.** Extend `src/sources/herdr_projects/` with typed
   registration/checkpoint/proposal types and a pure mapper. Validate selected
   context, source/campaign IDs, format, locked coherence, canonical creation
   evidence and the full known-thread set before producing any candidate. Add
   focused failing tests to `tests/herdr_projects_source.rs` for changed creation,
   omitted known records and unlocked exports. Prove rejection preserves the
   accepted projection/checkpoint. Keep the v1 DTO and unsupported capability
   flags intact; no source writes or runtime configuration.
2. **Faithful domain values.** Add Unspecified/Unknown/Closed/Archived to
   `src/work/types.rs` and cover them in pure Board/Table/scene projections and
   relevant matches. Map literal source context into bounded attributed details,
   not new source enums in the work domain. Produce no obligations, dependencies
   or live bindings. Map latest report/PR reference slots, updating previously
   known slots to unavailable when evidence disappears. Tests in `tests/work.rs`,
   `tests/herdr_projects_source.rs` and affected War Table suites prove Closed has
   no success cue, Unknown has no working cue, references revoke stale links and
   hashes/groups/acknowledgements do not imply review or delivery success.
3. **Atomic v2 work cache.** Extend `src/persistence/work_cache.rs`, the existing
   worker request/client and `src/work/runtime.rs` boundary so projection plus
   bounded typed adapter checkpoint is validated, published and acknowledged
   together. Preserve ordinary fixture/source publication and local intent without
   losing existing checkpoints. Work reducers do not inspect adapter storage.
   Validate checkpoint/projection correspondence and reject duplicate JSON keys.
   Valid v1 loads without checkpoints and migrates only on acknowledged publication;
   future/malformed files remain protected. Persistence/runtime tests exercise
   failure before publication, publication followed by restart, unchanged-byte
   suppression and source-token collisions. Do not introduce another durable file.
4. **Composed synthetic proof and qualification.** Test locked baseline -> thread
   change -> report loss -> rejected thread omission -> full retained recovery ->
   atomic restart; add property checks for identity isolation, inert repeats and
   checked counters. Run the affected source/domain/persistence/runtime/scene
   suites, `just verify`, `just property-test 4096`, release build and diff checks.
   Record the exact scope and any compromises. A changed production presentation
   needs its separate visual gate; synthetic mapping proof is not live acceptance.

The lease, source producer acknowledgement channel, disabled-by-default selection,
poll/cancellation policy and installed-source compatibility belong to M3a.3.
Its producer must detect observed root replacement across captures and discard
obsolete capture results; v1 DTO tokens alone do not supply that guarantee.
The user subsequently approved the M3a.2 images and promoted M3a.3 under the
[runtime plan](2026-09-30-herdr-projects-runtime.md). The owned producer, directory
lease, acknowledged SourceInbox and live/offline lifecycle are implemented;
the [runtime receipt](../reviews/2026-09-30-herdr-projects-runtime/README.md)
records synthetic qualification. The user subsequently
[approved its diagnostic images](../reviews/2026-09-30-herdr-projects-runtime/visual-approval.md).
The next [installed-source/isolated-runtime plan](2026-09-30-herdr-projects-acceptance.md)
was promoted, but its [prerequisite audit](../reviews/2026-09-30-herdr-projects-acceptance/README.md)
stopped before runtime resource creation; native/live gates remain unperformed.
Report contents, structured summons, execution controls, upstream contribution,
shared-server changes and live acceptance remain outside this proposed slice.
