# Herdr Projects snapshot proof v1

This internal export contract is experimental. The promoted M3a.3 opt-in producer
uses it without source commands or stronger identity/coverage claims. It began
as the M3a.1 proof for the source layout inspected at Herdr Projects 0.2.34,
`4e4548c3c43888e6be1c96906215998f07dc63f0`. See the
[source assessment](../reviews/2026-09-30-herdr-projects-source/README.md),
[M3 plan](../plans/2026-09-30-herdr-projects-adapter.md) and
[exporter receipt](../reviews/2026-09-30-herdr-projects-exporter/README.md).

## Authority and supported data

`src/sources/herdr_projects/` owns the upstream-specific file knowledge.
`capture(Selection, Timestamp)` requires explicit source/campaign tokens and an
absolute project path; it never discovers homes or reads environment variables.
It returns a DTO, not a work Message, and cannot change work, attention, source
files or a running agent. `decode_snapshot` validates the same contract.

The snapshot contains version 1, the pinned format/revision, configured identities,
caller-supplied capture time, project name/goal/literal status, a sorted thread map,
capability flags, coverage/coherence and typed diagnostics. Threads retain ID,
creation/update evidence, title, literal status, placement, closure reason/error,
recorded group, profile/agent labels, reported/acknowledged report hashes, PR text
reference and optional home-report hash/reference. No report bodies, project
instructions/memory, paths from thread metadata, pane/session IDs or launch flags
are exported. Source enums are adapter DTOs, not additions to the work domain.

`format` names the layout this reader supports. It does not certify an installed
binary version; no installed binary was queried. Extra upstream fields outside
this selected subset are ignored and never exported. Unknown selected enum values,
missing required fields and bad record identities fail the whole capture.

The following flags are fixed for v1:

| Capability | Value |
| --- | --- |
| Project metadata, thread inventory, report references | Supported |
| Dependencies, structured obligations, live bindings | Unsupported |
| Native durable identity, revision-bound validation | Unsupported |

No UUID/native project identity exists in the inspected records. A thread's numeric
ID and creation timestamp are evidence, not a uniqueness guarantee after deletion,
reuse or replacement between polls. `NoNativeDurableIdentity` must remain visible
in every DTO. Do not promote this DTO into authoritative durable work without a
separately selected identity contract. The subsequent
[registration/coverage decision](herdr-projects-identity-coverage.md) supplies
that bounded policy. The subsequent
[mapper/cache proof](../reviews/2026-09-30-herdr-projects-mapper/README.md) implements
it with synthetic input, without runtime attachment.

## Coverage and coherence

Coverage is always Partial. There is no Complete variant and omissions have no
retirement/resolution authority. A missing required file, parse error or path/type
violation returns an error with no accepted snapshot. A missing home report or
hash disagreement is explicit evidence in diagnostics, not a passed artifact.

An existing `.state/lock` is opened read-only and acquired shared/non-blocking.
A writer holding the cooperative exclusive lock yields Busy immediately. Missing
locks are never created: they yield UnlockedObservation and MissingRecordLock.
The subsequent mapping policy accepts this as inspection-only output; durable
work requires CooperativeRecordLock and known-thread coverage checks.
A successful shared lock is only CooperativeRecordLock. It does not cover arbitrary
writers, agent report writes or a transactional source-wide export.

The reader captures the allowlisted bytes twice and rejects disagreement. It checks
that the project/state/thread-directory and lock identities are still the same.
Descriptor-relative NOFOLLOW reads pin each directory component and reject
symlinks, parent traversal, non-regular files and directory replacement. All opens
use read-only flags; special files are opened non-blocking and then rejected.
Equal reads are an observation, not proof of an event history or a complete
transaction. This is why coverage remains Partial even with cooperative locking.

Bounds: 1,000 thread records, 4,096 directory entries, 256 KiB per read file and
8 MiB per captured batch/encoded snapshot. Two passes can read up to 16 MiB. Text
fields are at most 8,192 bytes and reject terminal controls except literal newline;
IDs/hashes/timestamps have stricter validation. Filesystem latency is not bounded
by these byte limits; a future runtime producer needs its own cancellation/I/O policy.

## Decode and reference rules

JSON structs reject unknown/duplicate fields. Thread-map duplicate keys are
explicitly rejected rather than choosing a later record/closure. Keys must match
validated record IDs. Unsupported version/format/revision, falsely enabled
capabilities, invalid report path/hash/equality claims or inconsistent diagnostics
are rejected. There are no executable commands in the schema.

A home report reference is exactly `threads/<id>.md` under the selected project.
The reader hashes its bytes with SHA-256 and compares the source's recorded hash.
The body is discarded. Matching means those bytes match that source report hash;
it does not mean review/check/merge passed or a quest succeeded. PR strings remain
bounded source references; nothing opens or fetches them. The DTO itself contains
no observer cursors/entity revisions. The subsequent mapper derives observation
ordering from the accepted cache; it never upgrades these records into a global
revision stream.

The example receipt JSON is a synthetic fixture. It is not a live source export,
persistence schema, Chronicle event, public protocol promise or runtime acceptance.
