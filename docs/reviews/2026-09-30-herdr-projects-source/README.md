# M3 Herdr Projects source assessment — 2026-09-30

**Reshape M3 into a source-contract slice, then a read-only adapter.** The user
approved source inspection and planning after accepting M2. This assessment is
complete. Its scope was read-only inspection/planning. The user subsequently
authorised the bounded [M3a.1 exporter proof](../2026-09-30-herdr-projects-exporter/README.md)
with “Let’s continue”; installation, source mutation and live testing remain
unperformed. The [M3 plan](../../plans/2026-09-30-herdr-projects-adapter.md)
proposes the next bounded work.

## Baseline and evidence

Questmancer: `main` at `f380720ba63424b279980ff4f1c164d8fd4ef735`, ahead six,
with accepted M1/M2 and independent pane-header changes still uncommitted.
Current source wins over the earlier roadmap. No code changed in this assessment.

Herdr Projects was fetched into temporary storage and inspected at
[`4e4548c3c43888e6be1c96906215998f07dc63f0`](https://github.com/eliasstravik/herdr-projects/commit/4e4548c3c43888e6be1c96906215998f07dc63f0),
commit date 2026-09-28, release `0.2.34`. The fetched checkout was clean.
[Inspection receipt](inspection.json) records file hashes and scope. Upstream
source was read, not built or executed; upstream tests were inspected where
relevant, not run. No real project homes, configuration, sockets or panes were read.
No plugin, ticker, coordinator or agent was started; no shared server was changed.

All upstream links below are pinned to this inspected revision.

## Observed capabilities and constraints

| Evidence | Observation | Consequence |
| --- | --- | --- |
| [Plugin manifest](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/herdr-plugin.toml#L1-L8) | Requires Herdr 0.9.1; Questmancer currently targets 0.9.0/protocol 22 | Actual host/version/protocol compatibility is a later gate; do not upgrade a shared server as part of planning |
| [CLI](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/cli.rs#L45), [JSON rows](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/threads.rs#L1102-L1140) | `thread list/show --json` exposes thread records, group, note, Next lines and report/library paths. No complete versioned project/event export was found in the CLI | Useful reference material, insufficient as an authoritative replay/decision protocol |
| [Rows](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/threads.rs#L1062-L1097) | Local groups mix records with current Herdr reads and report hashing. Unreachable sessions retain a recorded group; remote rows use the ticker's last poll | Successful JSON output does not prove every fact is current; do not silently upgrade recorded groups to current execution truth |
| [Thread schema](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/thread.rs#L23-L130) | Status is starting/open/failed/resolved; kind is worktree/tab/checkout/adopted | Placement is not Scout/Ship/Review/Validate; thread closure is not successful quest delivery |
| [Group classifier](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/thread.rs#L565-L611) | WaitingOnYou includes launch timeout/failure, a missing pane, stuck launch, blocked harness or explicit self-wait. ReadyForReview includes unread reports or reports with an open PR, subject to live-state checks | No literal question, choices, stable request incarnation or correlated answer is exported by these group labels |
| [Project state](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/project.rs#L187-L213), [rename](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/rename.rs#L1-L15), [allocation](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/thread.rs#L208-L232) | Project status is active/paused/archived; slug/path can change. Thread IDs are allocated from the largest retained number, scoped to a project | Require explicit configured source/project identity and distinguish thread creation incarnations; labels and slugs alone cannot establish durable identity |
| [Task parser](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/tasks.rs#L84-L95) | Tasks have list/title/owner/optional thread/description; no stable task ID or dependency field in this parsed type | First adapter scope is thread inventory; do not construct durable task IDs from titles or dependency edges from prose |
| [Report acknowledgement](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/threads.rs#L748-L756), [inbox](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/inbox.rs#L13-L26) | Thread ack records the current report hash. Inbox items are coordinator events; seen/handled are separate mechanisms | A future report-review episode can use a report hash and matching source ack. Inbox handled/seen is not evidence that a human question was answered |
| [Context command](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/coordinator.rs#L516-L525) | `context` marks shown inbox items seen unless `--peek` | Do not invoke it as a read-only data feed; even peek is an unversioned prose digest |
| [Resolution](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/threads.rs#L783-L818), [ticker resolution](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/steps.rs#L563-L656) | Threads can resolve manually, after merge or after configured idle expiry; cleanup is part of source workflow | Preserve Closed and the literal reason. Never award XP, call a quest Completed, or perform cleanup from this observation |
| [Atomic writes/lock](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/project.rs#L93-L115), [enumeration](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/thread.rs#L178-L195) | Individual files are atomically replaced; writers use a project lock for records/state. Existing enumeration silently drops unreadable records | Atomic files do not establish a multi-file transaction. A strict exporter must diagnose errors and specify coverage; missing rows cannot silently close obligations |
| [Progress](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/progress.rs#L26-L61), [PR summary](https://github.com/eliasstravik/herdr-projects/blob/4e4548c3c43888e6be1c96906215998f07dc63f0/src/pr.rs#L118-L129) | Progress distinguishes terminal/session and reports Done at 100%. PR summaries can hold a head OID in ticker state | Percent/Done is execution reporting, not delivery. PR status alone is not revision-bound check success |

The current work model has only four specific quest kinds, six lifecycle states
and no Archived campaign state. It therefore cannot losslessly represent this
source with its current enums. This is an observed integration gap, not grounds
to silently select Ship, Completed or Abandoned.

`SourceInbox` and `RuntimeConnection::attach_work_source` are clean existing
extension points. They already separate work delivery from Herdr subscription
updates. `bound_agent` additionally demands a current source and an exact capture
fingerprint. Herdr Projects' thread record/JSON row lacks the complete terminal
and session evidence needed for that fingerprint. Its own looser pane/name/path
matching must not replace Questmancer's guard.

## Decision and scope

Compare three routes:

1. Existing JSON CLI: smallest connection, but lacks coherent coverage, ordering,
   stable project identity and request lifecycle. Reject as the durable protocol.
2. Direct file polling from UI/core: reject; storage knowledge and I/O belong
   behind a source adapter, and silent parse omissions would corrupt attention.
3. Versioned snapshot export, with a pinned source-facing reader and a separately
   tested consumer: recommended. A companion exporter may read the pinned layout;
   a future supported upstream export can replace it behind the same boundary.

The first useful success is seeing an explicitly configured real campaign and
its retained threads/report references in the accepted Board/Table, with literal
source states and honest missing capabilities. Durable questions, dependency
graphs, live bindings and review-resolution semantics require additional source
contracts. Planning does not establish their availability.

## Remaining evidence

Read-only snapshot consistency under concurrent source writes, the deployed
source version, lock behaviour on supported hosts, poll/shutdown cost, native
terminal usability and usefulness with real work are unknown. The five-second
attention target has no measured baseline. No full gate was rerun for these
planning-only edits; documentation links, pinned file hashes and diff checks are
recorded separately from M2's retained qualification.
