# Questmancer living control plane

Date: 2026-09-27. Architecture contract approved by the user on 2026-09-30.
The Quest Board interaction concept was approved 2026-09-30. M1 is implemented
locally and the corrected production Quest Board was approved on 2026-09-30;
see the [qualification receipt](../reviews/2026-09-30-durable-obligations/README.md)
for engineering evidence and the separate native terminal acceptance boundary. The user agreed with the 2026-09-26 assessment and requested planning.
[Roadmap](../plans/2026-09-27-living-control-plane-roadmap.md) and
[first-slice plan](../plans/2026-09-27-durable-obligations.md) own sequencing.

## Purpose

Answer: what needs the human next, why, and what work does it affect?
Retain the scene-first guild, truthful facts, local-first runtime and useful
single-agent experience. At fleet scale, show obligations and campaign context
before individual actors. Herdr owns execution topology; sources/orchestrators
own work and delegation; Questmancer owns presentation and local attention intent.

The earlier ideation is direction, not an API specification. This design narrows
it to a work projection beside the existing live domain, preserving its reducer,
capture identity, counsel delivery, scene engine and Chronicle evidence rules.

## Current source baseline

Inspected local main at `f380720`, with existing uncommitted pane-header repair
and documentation. This is a source inspection, not fresh release qualification.

- `src/domain/campaign.rs`: Campaign is a Herdr workspace and its party.
- `src/domain/state.rs`: snapshots reconstruct live workspace/agent inventories.
- `src/update/reducer.rs`: status transitions replace per-agent attention.
- `src/app.rs`: `!` cycles a deterministic urgency-ranked agent list.
- `src/persistence/state.rs`: state v1 retains presentation intent and standing.
- `src/persistence/chronicle_jsonl.rs`: replay loads history, not domain state.
- `src/persistence/startup.rs`: live facts await Herdr; Chronicle is display data.

No existing Chronicle entry establishes durable task completion, complete upstream
history or an authoritative campaign lifecycle. A workspace disappearance is not
campaign completion. A done agent is not proof of delivered work.

## Authority and identity

| Concept | Authority | Questmancer responsibility |
| --- | --- | --- |
| Workspace, pane, session, live presence | Herdr | Existing live projection |
| Durable campaign, quest, assignment, dependencies | Originating work source | Versioned cached projection |
| Obligation request and resolution | Source that owns the obligation | Retain, explain and rank |
| Artifact and validation assertions | Producing source, with evidence | Display provenance and applicability |
| Acknowledgement, defer time, selection | Human through Questmancer | Durable local intent |
| Help handbook | Questmancer | Keep accessible independently of project knowledge |
| Project knowledge | External source | Read-only, attributed presentation |

Use newtypes wrapping source-qualified opaque IDs, not UUID-only replacements
for upstream identity. A SourceId distinguishes installations/homes, not merely
adapter kind. CampaignId, QuestId, ObligationId and ArtifactId bind SourceId to
the stable upstream identifier. Revisions and source generations remain separate.
Preserve current WorkspaceId, AgentKey and capture/session identity.

During the first slice, use `WorkCampaign` for the new type and retain the existing
`Campaign` unchanged. A work campaign has zero or more explicit workspace bindings;
one workspace may host work for several campaigns. Never infer identity from a
label, path, branch name or approximate prompt match. Unbound work remains useful.
Fixture bindings are explicit. Live binding policy is part of adapter qualification.

Assignment records retain source worker identity and historical display name.
An optional verified session binding enables observation of a live pane. Removing
that session disables navigation but does not erase assignment history or work.
Do not merge entities across sources without a separately specified correlation.

## Domain dimensions

WorkCampaign contains identity, name, optional objective, explicit source lifecycle
and timestamps. Offline is freshness, not a campaign lifecycle transition.

Quest contains identity, campaign, title, optional description, kind, source
lifecycle, assignments and dependency references. Scout/Ship/Review/Validate are
neutral domain kinds; unknown upstream kinds must remain visibly unsupported or
preserved as an explicit extension, never silently coerced.

Keep three independent dimensions:

- Work lifecycle: planned, ready, active, completed, failed or cancelled, as
  explicitly asserted by its owner. Pause/block reasons may be additional facts.
- Execution/delivery evidence: worker presence, attempts, PR/review/check/merge
  observations with their own revisions and provenance.
- Human obligation: the responsible actor, request, required action and resolution.

A dependency projection may say prerequisites are satisfied. It does not set
Ready, assign a worker or execute work. Missing dependencies and cycles are
diagnostics; neither counts as satisfied. Lifecycle transition and reopen rules
must be explicit per source, not guessed from the order of status names.

Obligation is the attention primitive. Kinds include Decision, Permission,
Review and ExecutionAction. Store campaign, optional quest/worker, literal prompt,
context reference, responsible actor, origin, available action, source revision,
resolution condition, request time and source state (open, resolved, superseded).
Decision choices/answers are typed detail where the source exposes them.

Local acknowledgement means seen. Defer means omit from the active queue until
a durable deadline. Neither closes the source obligation. M1 supports reading,
acknowledging, deferring and observing a verified pane; it cannot answer an
external decision. A later reply requires an adapter capability and correlated
delivery result; source resolution is independent from delivery acknowledgement.

Artifact contains identity, campaign/quest, kind, title, optional URI and typed
evidence. Keep commit/head revision, producer, check-run identity and validation
applicability when available. Unknown evidence is unknown. Producing an artifact
does not earn XP or trigger completion theatre. Existing spoils keep their meaning.

## Ingestion and reducer boundary

```text
Herdr -> existing adapter -> live reducer -> existing live domain
Work source -> baseline / updates -> work reducer -> work projection
Human -> attention-intent events -> durable local intent
                         |
                  derived attention + scene projection
                         |
                  existing RGB scene and parchment overlays
```

Reducers perform no I/O, read no clocks and emit explicit effects. Adapters may
understand source storage; UI and core types do not. No universal shared protocol
is assumed to exist. The initial internal boundary is fixture-driven and async-
compatible; polling sources may publish normalized updates outside render wakes.

A coherent baseline names source, generation, schema version, cursor, coverage,
captured time and entity revisions. Updates carry immutable event identity,
expected/source revision, ordered cursor and explicit upserts/tombstones.
Keep source occurrence time distinct from local observation time. Use cursor and
revision for order, never wall-clock sorting. No global order across sources is
claimed; stable source/entity identity breaks display ties deterministically.

Apply complete covered baselines atomically per source. Absence in partial data
does not delete entities. Reconnection rejects the old generation and resumes
from a checkpoint or requests a new coherent baseline. Identical replay is inert;
conflicting equal revisions, cursor gaps and malformed batches retain prior facts,
mark reconciliation required and withhold further authoritative deltas. Explicit
source resolution or complete inventory coverage can retire an obligation;
ordinary progress, pane death and disconnection cannot.

Source freshness is visible separately from cached domain facts. Unavailable data
does not become idle, successful or resolved. Last-known open obligations remain
visible with freshness evidence. M1 has one source; multiple-source interleaving
must not change the final independent projections.

## Attention policy and interaction

Derive a single queue from source obligations and legacy summons. Include only
human-owned open obligations and existing eligible summons; supervisor/external
waits appear as context. Deterministic ranks:

1. Explicit blocked work requiring a human action.
2. Decision or permission requiring the human's answer.
3. Failed execution with an explicit human recovery action.
4. Work explicitly ready for human review.
5. Existing eligible summons.

Within a rank: oldest retained request time, then source-qualified identity.
Missing source times use a stable first-observed time. Acknowledgement preserves
eligibility; deferral suppresses it until expiry. Pass time into the projection.
Do not create an obligation merely because CI changes or an agent is blocked.

`!` cycles distinct attention targets and opens their context. With no work-source
items it preserves existing agent-only cycling. A source obligation and legacy
summons collapse only with an explicit matching session/reason/episode binding;
similar text is insufficient. Work-only selections need no live adventurer.
Empty queues keep selection and show the existing quiet result.

M1 adds a Quest Board parchment in the existing scene. Details show why an item
needs attention, source/freshness, affected work, artifacts and available actions.
No new station art, graph renderer or manual sprite steering is required.

## Persistence and replay

Keep `state.json` and v1/v2 `chronicle.jsonl` compatible. Do not reinterpret legacy
entries as complete work events. Plan a separate versioned `work-state.json` for
the cached work projection, accepted cursor/event identities and local intent.
This is a declared cache of work facts, not copied Herdr topology or terminal output.
The existing persistence rule needs this narrow documented extension at implementation.

For M1 use atomic snapshots, acknowledged writes and unchanged-state suppression.
Publish visible durable success only after write acknowledgement; failure retains
the last durable checkpoint, exposes an error and does not claim restart safety.
Persist defer/ack intent coherently with the snapshot. Unknown future versions are
read-only protected; malformed snapshots are diagnosed and never silently replaced.
Restore cached facts as awaiting source verification. Baseline reconciliation
must preserve local intent for matching obligation incarnations.

M1 replay means deterministic fixture reconstruction and restart restoration.
It does not mean time travel. A later work journal needs committed batch boundaries,
complete baseline coverage, checkpoint offsets, bounded deduplication, corruption
policy and migrations. Decide that format before claiming historical reconstruction.
Retained Chronicle limits must never remove current unresolved obligations.

## Product and operational limits

At 1/10/100 agents, show campaign/obligation summaries, then selected parties.
The eleven-slot Hall is not enlarged into a hundred-actor display. The War Table
later adds deterministic dependency composition, filtering and clear small-size
fallbacks through the existing RGB engine. Keep every interaction keyboard reachable.

Source knowledge is displayed on demand with attribution and freshness; the
Librarian retains fixed help. Remote/service reads, URI opening and adapter
commands require explicit capabilities, configured boundaries and qualified
recipes. The fixture slice introduces no network or external command execution.

Non-goals: spawning, planning, delegation, model/prompt routing, worktree or branch
management, merge automation, autonomous assignment, workflow execution and
project-memory ownership. Domain observations do not authorize those actions.

## Approval and later design gates

The user's 2026-09-30 approval accepts the WorkCampaign naming/mapping, independent
obligation lifecycle, atomic work-cache persistence and legacy-summons fallback
specified here. The [Quest Board concept](reviews/2026-09-30-quest-board/README.md)
was approved as a concept, then implemented and corrected following user review.
The user approved the corrected production Quest Board on 2026-09-30. M1 is
complete for its fixture-backed scope. M2 was subsequently implemented and its
production visuals approved on 2026-09-30. M3 source inspection and planning were
then approved; see the [pinned assessment](../reviews/2026-09-30-herdr-projects-source/README.md)
and [adapter plan](../plans/2026-09-30-herdr-projects-adapter.md). M3a.1 was later
promoted and engineering-qualified as a synthetic export/decoder proof. The user
then promoted the [registered identity/coverage design](herdr-projects-identity-coverage.md).
That documentation slice is complete. Mapper/cache implementation was subsequently
promoted and engineering-qualified; see the [receipt](../reviews/2026-09-30-herdr-projects-mapper/README.md).
State-cue visuals were subsequently approved and the user promoted the
[opt-in runtime slice](../plans/2026-09-30-herdr-projects-runtime.md).
Its synthetic producer/consumer is implemented and its
[diagnostic images are approved](../reviews/2026-09-30-herdr-projects-runtime/visual-approval.md).
Installed-source and native/live acceptance remain unperformed; the next
[bounded operational plan](../plans/2026-09-30-herdr-projects-acceptance.md)
was promoted, but its prerequisite audit stopped before runtime resources were
created. The [audit](../reviews/2026-09-30-herdr-projects-acceptance/README.md)
records the concrete missing prerequisites. Later milestones
remain unpromoted. Native source identity and
reply authority are not established by the retained-record policy.

Reference projects were reviewed on 2026-09-26; re-inspect pinned source before an
adapter implementation. Their current internals are not a stable protocol promise:
[Herdr Projects operations](https://github.com/eliasstravik/herdr-projects/blob/main/docs/operations.md),
[Firstmate architecture](https://github.com/kunchenguid/firstmate/blob/main/docs/architecture.md).
