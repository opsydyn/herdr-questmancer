# Chronicle capture: observed facts and quiet baselines

Status: C1 snapshot qualification and C2 capture/replay are approved and
implemented locally, 2026-09-09. C3 automated and isolated synthetic qualification
has passed; the user visually approved the changed Chronicle screens on 2026-09-09. Source baseline:
`a26386c`, following released `v0.1.9` (`98f557d`). This design concerns history
capture; approved room art is unchanged. See the [C1 receipt](../reviews/2026-09-09-chronicle-c1/README.md) and
[C2 receipt](../reviews/2026-09-09-chronicle-c2/README.md).

## Accepted capture policy

Record what Questmancer observes while a connection remains trustworthy.
A same-connection snapshot may add an explicitly labelled observation when a
known subject changes. Startup, reconnect and subscription-resync snapshots
establish a baseline and add no history. A later observation never reconstructs
an unseen sequence or claims the exact time that work happened.

Keep ordinary confirmed status history, but distinguish it from snapshot
observations and from legacy records. New observation and removal records earn
zero XP. In particular, closing a Herdr workspace is not successful delivery.

Example: a known working adventurer is observed blocked in an eligible refresh.
The Chronicle records “Aria was observed needing counsel” at the local observation
time. If Questmancer reconnects and finds Aria blocked, it updates the live party
without adding a request, a join, a completion or a retrospective timestamp.

## Findings in the released source

| Boundary | Current behaviour | Consequence |
| --- | --- | --- |
| `src/update/reducer.rs::replace_snapshot` | Rebuilds live state, preserves Chronicle and emits only `PersistState` | Snapshot-only changes leave no history |
| `src/herdr/supervisor.rs::connection_cycle` | Subscribes before publishing the authoritative baseline; topology changes can rebuild subscriptions | A new `Connected` update is not proof of continuous observation |
| `src/runtime_loop.rs::apply_command_result` | Applies an uncorrelated `SnapshotLoaded` result | Concurrent or previous-connection refresh results lack a freshness contract |
| `src/herdr/supervisor.rs::reconcile_status_event` | Replaces unversioned event contents with current `pane.get` metadata | The result is a current observation; intermediate transitions may already be lost |
| `src/update/reducer.rs::change_status` | Unknown maps to `AdventurerJoined`; equal revision with a different status can apply | Legacy identity labels are ambiguous; conflicting observations need reconciliation |
| `src/update/reducer.rs::close_workspace` | Removes the campaign and agents without an append or timestamp | Closure taxonomy exists but is not captured here |
| `src/domain/chronicle.rs` | Event ID hashes kind, pane, revision and local timestamp; campaign/adventurer are absent | Different observations of one event can differ; pane-less campaigns can collide |
| `src/domain/chronicle.rs::Chronicle::append` | Bounded entries and bounded seen-ID set | Retention is not a durable, lifetime deduplication ledger |
| `src/runtime_loop.rs::apply_domain_event` | Every newly appended event awards `event.experience()` | Reusing `CampaignClosed` would award 25 XP on workspace closure |
| `src/persistence/chronicle_jsonl.rs` | Replays flat JSONL, diagnoses invalid lines and appends with `sync_data` | New records require an explicit compatibility boundary |

These are findings from the released baseline above. C1 adds correlation and
rejects conflicting equal-revision status changes. C2 adds qualified capture,
source identity and compatible replay. Neither implies that the connection sees
every Herdr event.

## Capture eligibility and ordering

Introduce typed, ephemeral observation context. It belongs in runtime/supervisor
coordination and is passed into pure reduction. It is not saved in `state.json`.
Suggested concepts are `ConnectionEpoch`, `SnapshotRequestId`, `ObservationId`
and `SnapshotPurpose::{Baseline, Refresh}`; names may follow local conventions.

1. Create a fresh epoch for startup, reconnect, resync or loss of ordering.
   Immediately invalidate outstanding snapshot work from the previous epoch.
2. Install the post-subscription authoritative snapshot as `Baseline`. Preserve
   saved intent/personas and retained history, but emit no Chronicle appends or XP.
   Existing live attention projection remains independent of history capture.
3. Coalesce refresh requests: one in flight and at most one pending refresh.
   Each result carries its epoch, request ID and the live-fact generation at
   request start. This generation advances on accepted topology, identity or
   presence/revision changes and reconciliation hints, not animation, selection,
   text editing or sidebar metadata.
4. Discard results from an obsolete epoch/request. If relevant live facts changed
   while a snapshot was in flight, do not apply that snapshot or derive history
   from it. Request a fresh snapshot through the same bounded path.
5. Avoid an unbounded catch-up loop: after two consecutive superseded refreshes,
   use the existing subscription-resync procedure and install a quiet baseline.
   Capturing less history is preferable to starving the live party or applying
   stale state. Disconnect/shutdown cancels this recovery work. The candidate's
   pane subscription set must match the post-subscription baseline; a mismatch
   immediately requests a new subscription and quiet baseline. This set is an
   ephemeral subscription capability, never persisted topology.
6. Compare an eligible refresh with the current authoritative domain before
   replacement. Emit only typed differences. Then install the new live facts.
   The reducer must receive enough identity evidence to detect ambiguous subjects
   before the existing map projection can collapse them.
7. Route direct status/lifecycle observations through the same acceptance boundary.
   Preserve current metadata reconciliation. A lower revision is stale; an equal
   revision with conflicting presence or identity requests reconciliation rather
   than creating alternating history. A same-state repeat adds no history even
   if its revision, summary or observation time changes.

No observation is produced by an animation frame. There is no polling interval,
terminal-output read, counsel command or extra durable copy of Herdr topology.
The supervisor and runtime must share this context: merely adding a flag to
`replace_snapshot` cannot make asynchronous refresh results trustworthy.

## What to record

The names below are proposed domain variants, not raw user-facing copy.

| Evidence during an eligible epoch | Record and copy | XP |
| --- | --- | --- |
| Existing unambiguous identity changes presence in a fresh snapshot | `PresenceObserved`: “Aria was observed delving / needing counsel / resting / with spoils reported” | 0 |
| A known identity's status becomes unknown | `WhereaboutsUnknown`: “Aria's whereabouts became unknown” for accepted status evidence; “were observed as unknown” for a snapshot | 0 |
| New identity appears after the baseline | `AdventurerObserved`: “Aria was first observed in the guild” | 0 |
| Identity disappears from a fresh snapshot | `AdventurerNoLongerVisible`: “Aria was no longer visible in the guild” | 0 |
| Corroborated pane exit for the same current incarnation | Existing departure semantics, with lifecycle evidence; never completion | 0 |
| Campaign disappears from a fresh snapshot | `CampaignRemoved`: “Campaign X was no longer visible” | 0 |
| Received workspace-close hint, then a fresh snapshot confirms absence | `CampaignRemoved` with corroborated-close evidence: “Campaign X closed” | 0 |
| Existing accepted status route explicitly reports done | Existing `SpoilsReturned` semantics and reward, with source evidence | Existing 10 |
| Startup/reconnect/resync, unknown source, stale/conflicting response | Baseline/reconcile/diagnose; no new record | 0 |

Only one history record represents a presence change. A snapshot observation
followed by matching status metadata is a duplicate, not an extra spoils return.
If the snapshot was the first accepted evidence of done, later identical metadata
must not retroactively award XP. A later working-to-done episode can still earn
through the existing status route. This conservative difference must be explicit
in tests and release notes; do not use XP as an event-completeness measure.

Identity observation establishes current membership, not an actual join time.
When a new identity is first observed blocked or done, append its identity
observation only, with that state as context; do not add a guessed prior request
or completion. A true disappearance never becomes an exit or successful task.

A campaign removal absorbs membership disappearance records caused solely by
that same missing campaign; record one campaign fact rather than manufacturing
one departure per adventurer. An independent, previously confirmed pane exit
remains valid. A moved campaign or adventurer is not removal plus arrival. Unversioned pane-exit
hints must be checked against the current incarnation; if the evidence proves
only absence, record visibility loss rather than claiming a process exit.

Do not emit the existing reward-bearing `CampaignClosed` from this new path.
Retain its legacy records and historical standing. Defining a real successful
campaign-completion contract is separate product work.

## Identity and duplicate boundaries

`AgentKey` remains presentation/persona identity; do not change its derivation or
saved assignments. Current generation prefers agent-session identity, then a
workspace/name fallback, then workspace/pane. That fallback alone does not prove
that two observations concern the same running agent.

Carry the existing protocol `terminal_id` into ephemeral capture identity,
together with pane ID and the complete available agent-session tuple. Do not
persist session text, commands or terminal contents. A same-session pane move
preserves adventurer continuity, but revision comparison restarts for a different
terminal incarnation. A same-pane identity replacement is not a status transition
of the old adventurer. Without sufficient evidence, baseline that subject and
suppress inferred lifecycle events. Multiple source agents mapping to one
`AgentKey` are ambiguous: diagnose and withhold capture for that identity; persona
collision redesign is outside this slice.

New records have immutable observation identity minted once when reduction
accepts a difference. Use a process-unique capture-run ID supplied at startup,
epoch, monotonic accepted-observation ordinal, typed subject identity and event
kind, hashed with a versioned domain separator. Campaign identity must participate
even when there is no pane. Wall-clock time and display summary are not ID inputs.
The pure reducer gets these values as input; it generates no randomness or I/O.

Semantic duplicate prevention happens before ID allocation, using current facts,
source freshness and incarnation. Do not allocate a new ordinal for a duplicate.
Multiple records in one accepted snapshot use stable subject ordering. Retries
carry the original record and ID; they never reconstruct a new observation.

History eviction cannot reset those current-fact guards. On process restart,
replay preserves recorded IDs and installs the live world as a fresh baseline;
it does not attempt to rediscover old events from a saved topology. This gives a
bounded capture contract, not globally exactly-once delivery or complete replay
of upstream history. Reappearance after an observed absence is a new membership
observation; it is not evidence of what happened while absent.

## Records, replay and standing

Introduce a versioned Chronicle record reader before new writers. Existing flat
records are legacy v1: preserve their IDs, summaries, timestamps, event kinds and
order. Do not parse prose to reinterpret old `AdventurerJoined` records. Their
chapter label remains “identity event”; never retroactively relabel them as joins.

New writes use a v2 envelope containing a record version, immutable event ID,
observation time, typed event payload, captured subject references/summary and
source evidence (`Snapshot`, `PaneMetadata`, or corroborated `Lifecycle`). An
upstream occurrence time is optional and permitted only when independently
qualified; none is inferred from receipt time. Existing fields named
`occurred_at` are legacy display times, not proof of exact upstream timing.

The v2 envelope is deliberately distinguishable from a flat v1 record so an
older reader cannot silently reinterpret it as a reward-bearing legacy event.
The new reader accepts mixed v1/v2 lines; unknown versions/events retain bounded
line diagnostics and leave the file untouched. Old 0.1.9 readers may skip v2
records with diagnostics on downgrade. Document that limitation; preserving old
records does not make new semantics readable by old binaries.

Keep append-only JSONL, malformed/truncated-tail handling and the existing
acknowledgement/diagnostic path. A crash or append failure can still lose a live
observation; do not claim that the history and `state.json` form an atomic
transaction. No automatic reconstruct-and-retry loop is introduced. Preserve
existing standing without replaying rewards, backfilling awards or deriving a
lifetime total from bounded records. New observation kinds award zero; existing
accepted spoils retain their rule. Durable awards and mementos remain deferred.

## Chapter and record presentation

Both views expose observation wording and local observation time for v2 sources;
reserve “occurred” language for independently qualified occurrence timestamps.
Keep chapter windows fixed, UTC rendering, retained-record counts, immutable
source summaries and source IDs. Count new observation categories separately
from spoils returns and legacy identity events. Two observations by one
adventurer remain two records, not two people or two completed projects.

The existing notice that only retained records are covered remains mandatory.
An empty chapter never means no work happened. Baseline gaps need no fabricated
Chronicle events or guessed missing-event count. This proposal does not add a
coverage timeline, persistent capture checkpoint or another rendering surface.

## Alternatives considered

- **Append ordinary lifecycle events for every snapshot difference:** rejected;
  it invents join/exit timing, amplifies reconnects and can award unearned XP.
- **Keep event-only history:** smallest change, but leaves observed refresh-only
  facts absent and preserves the current taxonomy ambiguity.
- **Persist every snapshot and reconstruct a complete timeline:** rejected;
  duplicates Herdr truth, grows sensitive storage and still cannot prove unseen
  transitions. It conflicts with the plugin's persistence ownership.
- **Add an upstream event log or durable exactly-once ledger now:** deferred;
  useful only after a source contract and persistence design justify that scope.

## Implementation sequence and acceptance

Follow the [bounded implementation plan](../plans/2026-09-09-chronicle-capture.md).
Approval covers the proposed observation wording, quiet reconnect/resync policy,
zero-XP observation/removal events and mixed-version Chronicle records, including
the stated downgrade limitation. C1–C3 are complete locally, including user
approval of the Chronicle screen pack on 2026-09-09. Commit and release
qualification remain separate gates. Baseline snapshots continue to append no history and
award no XP. Qualified refreshes now write the zero-XP observations above.

## Implemented C2 representation

`CaptureClock` is ephemeral reducer state: a runtime-supplied random capture-run
ID, current connection epoch, accepted-record ordinal and bounded known-workspace
close hints. The ordinal commits only when a new record is accepted. Neither it
nor `Agent::capture_identity` is part of persisted local intent. A complete
session tuple plus terminal ID qualifies subject observations. Terminal evidence
without a complete session still invalidates old per-pane I/O, but earns no
inferred subject history. Ambiguous identity produces a bounded diagnostic.

`ChronicleEntry` projects either unchanged legacy fields or a typed
`CapturedObservation`. New envelopes contain `record_version: 2`, `id`,
`observed_at`, `summary` and `observation`. The latter holds:

- `stamp`: capture run, connection epoch and accepted ordinal;
- `subject`: a typed adventurer or campaign with captured references/name;
- `evidence`: snapshot request plus typed observation, current pane metadata
  presence, or a corroborated lifecycle departure.

Adventurer evidence stores an incarnation digest and revision, never raw session
metadata or terminal contents. The reader rejects unknown versions/payloads,
invalid source context and inconsistent observation IDs with bounded diagnostics.
No upstream occurrence timestamp is emitted by the current protocol path.

The same terminal retains its revision boundary across a pane move. A changed
terminal establishes a quiet subject baseline and invalidates old scrying and
same-party cat comparisons. A missing session tuple is not proof that the
previous occupant disappeared. Subscription pane-set changes still require a
quiet C1 baseline; capture coverage is deliberately incomplete across them.

## Design verification — 2026-09-09

The existing `reducer`, `chronicle`, `chronicle_chapter`, `chronicle_persistence`,
`event_adapter`, `runtime_loop` and `supervisor` suites passed: **109 tests in
seven suites**, using the released behaviour. The log is retained at
`/tmp/questmancer-chronicle-design-baseline.log`. Document links and
`git diff --check` passed. At that design-only checkpoint, no application code, tests, live Herdr resources,
release tags or published artifacts changed. These checks establish the current
baseline; they do not qualify the proposed implementation.
