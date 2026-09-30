# Herdr Projects: registered identity and partial coverage

Date: 2026-09-30. The user promoted this identity/coverage design slice after
M3a.1. The user subsequently approved M3a.2 mapper/cache implementation.
**The synthetic mapper/cache proof is implemented and visually approved.
The user promoted opt-in runtime integration; live acceptance remains separate.** It supplements the
[snapshot v1 contract](herdr-projects-snapshot-contract.md) and
[M3 plan](../plans/2026-09-30-herdr-projects-adapter.md).

## Decision and trade-off

Use explicit Questmancer registration for one selected local project, under a
retained-record policy. Registration supplies opaque source/campaign identities;
it does not certify a native identity that Herdr Projects lacks. Keep the v1
`durable_identity` capability Unsupported and `NoNativeDurableIdentity` diagnostic.

This enables a read-only campaign/thread inventory without changing upstream.
The cost is a limited continuity guarantee: records must retain their numeric
IDs and creation evidence throughout that registration. Copies, restores,
deletion/reuse and project replacement require a new registration, not automatic
attachment to old work. Sources needing identity guarantees across those
operations must first provide native incarnation metadata.

Do not infer identity from title, slug, path, report hash, agent name or filesystem
inode. No source files, lock files or sidecars are written. No structured summons,
dependencies, live bindings, delivery success or historical replay are added.

Alternatives considered:

| Route | Decision |
| --- | --- |
| Explicit registration plus retained records | Selected for the bounded inventory adapter; limits stay visible |
| Require native project/thread incarnation IDs first | Stronger across copying/replacement, but requires separate upstream work; required before claiming that stronger capability |
| Derive identity from path/title/hash/creation time | Rejected: mutable locators and equal timestamps can alias unrelated work |
| Persist no work, show only the current export | Useful inspection fallback, but does not deliver durable campaign continuity |

## Evidence for the bounded policy

All source claims refer to Herdr Projects `0.2.34`, commit
`4e4548c3c43888e6be1c96906215998f07dc63f0`, not an installed binary.

- [Allocation](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/thread.rs#L197-L232)
  chooses the next number from retained records; updates rewrite the same record.
- [Resolve/reopen](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/threads.rs#L769-L821)
  changes status on that record. Reopen does not allocate a new thread.
- [Sweep](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/sweep.rs#L124-L195)
  can remove a resolved thread's working folder; that is distinct from its
  `threads/t-NNNN.toml` record and home report.
- [Rename](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/rename.rs#L326-L358)
  moves the project and rewrites paths in retained records.
- [Archive/delete](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/lifecycle.rs#L66-L134)
  distinguishes a retained archived project from moving its folder into `.trash`.
  Deletion is observed as lost attachment, never inferred successful delivery.
- [Creation time](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/project.rs#L114-L119)
  is rounded to seconds. It is not a UUID or uniqueness guarantee.

These paths support the policy as an inference about the inspected workflow,
not a published upstream immutability promise. Equal selected records after an
unobserved replacement are indistinguishable to this reader. Locking, two equal
captures and a digest cannot remove that limitation.

## Registration and entity identity

1. A registration contains the configured `SourceId`, campaign token, pinned
   format and explicit `RegisteredRetainedRecords` policy. Its absolute local
   project path is a configuration locator, not durable work identity. The source
   token is unique per registered project incarnation, not shared by every Herdr
   Projects installation. Tokens must not be reused for a replacement/copy.
2. Campaign ID is the existing source-qualified campaign token. A quest's local
   key is `<campaign>.thread.<native-id>`, under the same source. Both remain
   inside existing typed-ID length/character rules. Creation time is checkpoint
   evidence attached to that key; it is not encoded into a newly guessed quest ID.
3. Compare creation evidence as a parsed timestamp instant. Equivalent RFC3339
   spellings represent the same evidence. Changed evidence for a known number
   rejects the whole candidate and latches reconciliation; it does not silently
   create, replace or retire a quest. Missing/invalid creation already fails v1.
4. Resolve then reopen is the same quest with a newer observation revision.
   Title/profile/agent/report changes never allocate a quest. Different source
   registrations may contain identical native numbers without collision.
5. Configured registration and cache checkpoint must match exactly. A source
   token already present without a matching adapter checkpoint cannot be adopted
   by matching its titles or paths. Use a fresh token or explicit reconciliation.
6. Rename requires an explicit locator rebind retaining the registration only
   when the owner confirms the same retained project. `former_slugs` alone is
   not rebind authority. Copy/restore/reinitialisation uses fresh identities;
   old cached work remains last known, without an inferred outcome.

The mapper takes a trusted registration/capture context as well as a decoded DTO.
A DTO repeating the configured tokens is insufficient proof of attachment.
The later runtime producer owns the selected directory lease and detects observed
root replacement between captures, not just replacement within one capture.
Observed disappearance/replacement invalidates that lease and requires explicit
reconciliation; it must not auto-bind a newly created same-named directory.
The v1 DTO alone cannot implement this producer guarantee. Across restart the
registration policy remains the continuity assumption; inode reuse or identical
restores are not claimed to be detectable.

## Coverage, freshness and acceptance

Coverage stays Partial. A current source means the accepted selected facts were
observed under this policy, not that execution/review state is current or the
inventory is complete. Recorded groups remain explicitly last-recorded context.

For durable mapping, require `CooperativeRecordLock`. An unlocked v1 export is
still valid inspection output, but cannot refresh durable work. A new project
without a lock remains unavailable for mapping until its ordinary source workflow
creates the lock; Questmancer never creates it. The shared lock does not protect
agent report writes or establish a source-wide transaction.

Compare the candidate thread set with **every retained thread in the accepted
checkpoint** before calling the work reducer. A missing known record rejects
the entire candidate, retains facts and marks Reconcile. No partial baseline may
make omitted old quests current. A later locked observation containing all known
threads with unchanged creation evidence may recover from a transient omission.
Identity/lease mismatches require explicit reconciliation instead.

| Observation | Required result |
| --- | --- |
| Locked, valid first inventory, matching registration | Accept a partial baseline; no omissions or bindings authorised |
| Known threads retained plus a new valid thread | Accept new work and changed facts atomically |
| Known thread missing, including an empty replacement inventory | Reject candidate; retain all facts as reconciliation required |
| Known number has changed creation evidence | Reject and latch identity reconciliation; no new quest inferred |
| Missing/busy lock, unstable reads or required-file/parse/limit error | No candidate accepted; retain last facts as unavailable and retry outside rendering |
| Root missing/replaced, or configured tokens/format disagree | Invalidate attachment; explicit reconciliation required |
| Project archived or thread resolved | Literal Archived / Closed; no completion, abandonment, XP or retirement inferred |
| WaitingOnYou / ReadyForReview / report acknowledgement changes | Literal context only; no request opened, answered or superseded |
| Source restarts/disconnects | Cached/unverified or unavailable; no binding restored |

The existing reducer merges partial inventories and marks the resulting source
Current. The omission guard therefore belongs before reduction. Do not feed an
unguarded v1 export straight into `Message::Baseline { complete: false, .. }`.
Never emit `complete: true`, omission removals or obligation resolution for M3a.

Artifact references require explicit handling too. Use at most one latest report
reference slot and one latest PR reference slot per quest, under deterministic
source-qualified IDs. These slots describe the latest observed reference; they
are not identities of PRs or a historical artifact journal. Once emitted, include
each known slot in subsequent candidates. A now-missing/mismatched report or
removed PR reference updates its slot to explicitly unavailable, with `uri: None`
and no validation; it does not retain a usable old link as a current reference.
Initial missing references create no slot. Changing a reference updates its slot
revision. A matching report hash proves bytes only; `head_revision` and validation
remain None. Report artifact URI stays None in this slice; show its relative
reference/hash as bounded attributed quest context, without manufacturing a
filesystem link. A PR slot may expose a URI only for an HTTPS `github.com` URL
with exactly `/<owner>/<repo>/pull/<positive-number>`, no credentials, port, query
or fragment. Other source PR text stays literal context with URI None. Nothing
opens/fetches these references. Core types must not acquire source placement/group enums.

## Observation order and atomic checkpoints

Generation, cursor and entity revisions are **Questmancer observation order**.
Source timestamps, status, report hashes and acknowledgements are not global
revisions. Intermediate transitions missed between polls cannot be replayed.

- Derive the next generation from the acknowledged cache on producer start;
  use checked increments. Continue its cursor monotonically for accepted
  baselines. Within one producer generation, changed candidates advance the
  cursor; unchanged healthy observations emit nothing and do not rewrite cache.
- Compare deterministic mapped values with accepted projection values, ignoring
  revision fields. Reuse revisions for identical values; checked-increment only
  changed entities. Registration/creation evidence is checked before comparison.
- Keep capture time/health probes outside that value comparison. For first
  observations, revisions start at one. Source update strings remain export/probe
  evidence, excluded from mapped-value comparison; they cannot order two read
  results or authorise a lifecycle transition. Timestamp-only source rewrites are
  inert for durable mapping.
- Reconnection/restart requires a fresh baseline even if values are unchanged;
  its generation/cursor advances, unchanged entity revisions do not. Source
  freshness recovers only after acknowledged acceptance. Reject stale generation,
  duplicate conflicts and counter exhaustion; never reset/wrap counters.
- Read completion from an old producer/lease cannot become a candidate for a
  newer one. Allow only one outstanding proposal per source, acknowledged by the
  consumer; the current `SourceInbox` send success is not storage acknowledgement.

The v1 work cache held the projection without registration/creation
checkpoints. The implemented **v2 atomic envelope** contains the projection and a
bounded typed source-checkpoint collection. The Herdr Projects variant contains
policy/format/campaign identity and native-ID -> canonical-creation evidence.
Generation/cursor/entity revisions already live in the projection; do not
duplicate them or copy raw source records. No arbitrary JSON map, topology,
bindings, terminal output, memory body or source paths enter this checkpoint.
The work reducer stays source-neutral; adapter checkpoints belong at the
source/persistence boundary, not in quest types.

Build a candidate projection/checkpoint pair without advancing accepted state.
Validate both and their ID/creation/inventory correspondence; publish both in one
atomic work-cache replacement; await the persistence worker acknowledgement;
then expose the projection and advance the producer checkpoint. Publication
failure leaves the previous pair intact and marks the in-memory source Reconcile.
Do not publish these halves to separate files. Failure after publication but
before display is recovered from the complete pair on restart.

Load valid v1 losslessly as cached work with no adapter checkpoints; do not attach
those sources heuristically. Load v2 only after strict validation, restoring all
facts Cached and bindings empty. Unknown version, duplicate keys, invalid
checkpoint or malformed data protects the file from overwrite. Migrate to v2
only through an acknowledged publication. `state.json`, Chronicle envelopes,
standing and XP remain unchanged.

## Required proof before runtime integration

The [M3a.2 implementation plan](../plans/2026-09-30-herdr-projects-adapter.md#m3a2-mappercache-implementation-proposal)
defined the implemented mapper/cache slice. Its test vectors cover:

1. Two registered projects with the same native thread number stay isolated;
   rename/rebind preserves identity, replacement requires new registration.
2. Resolve -> reopen keeps one quest; identical/differently spelled creation
   instants match; changed creation evidence never replaces it.
3. A known thread omitted from a partial/empty candidate cannot become current,
   disappear or imply success. Complete retained inventory recovers a transient
   omission. Missing lock cannot make an inspection DTO a durable baseline.
4. Report match -> mismatch/missing and PR removal revoke usable references;
   neither creates passed validation or closes a request.
5. Unchanged polls keep revisions/cache bytes stable; changed values increment
   once; stale/duplicate/conflicting results and exhausted counters fail safely.
6. Publication failure/crash/restart never separates checkpoint from facts;
   restored sources are Cached; v1 migration preserves intent; future/malformed
   and duplicate-key files remain protected.

The [implementation receipt](../reviews/2026-09-30-herdr-projects-mapper/README.md)
records executed synthetic proof and the remaining gates. The pure mapper and
consumer classify identity failure as requiring explicit rebind; the later
producer latches that failure and enforces the directory lease in the subsequently
promoted [runtime slice](../plans/2026-09-30-herdr-projects-runtime.md).
The [changed diagnostic images are approved](../reviews/2026-09-30-herdr-projects-runtime/visual-approval.md).
Installed-version, native input and isolated live acceptance remain later gates;
the [acceptance plan](../plans/2026-09-30-herdr-projects-acceptance.md) was promoted,
but its [prerequisite audit](../reviews/2026-09-30-herdr-projects-acceptance/README.md)
stopped before runtime resource creation. Revisit this
decision if upstream supplies native incarnation/export guarantees, retained
records become unbounded, or real work needs copying/restoration continuity.
