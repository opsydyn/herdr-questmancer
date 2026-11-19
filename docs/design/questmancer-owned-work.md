# Owned work backend

Date: 2026-10-01. The user agreed to owning the project/work backend, with
Questmancer as its human interface and Herdr providing execution. This direction
is accepted. The user approved L1a implementation on 2026-10-03. The authoritative
backend and LocalWorkSource now exist behind explicit library calls. Ordinary
startup and user authoring remain outside L1a. The
[L1a receipt](../reviews/2026-10-03-owned-work-backend/README.md) records qualification.
The subsequently promoted [L1b implementation](questmancer-owned-authoring.md) adds
explicit setup, remembered selection and typed command/detail interaction. See its
[receipt](../reviews/2026-10-03-owned-authoring/README.md) for engineering evidence
and outstanding visual/native gates.

L2a adds explicit quest lifecycle commands through the Board. The first
confirmed transition writes owned-store v3 with the updated quest and its
receipt. Decision answers, revision reviews and Herdr execution retain
separate authority. See the [lifecycle decision](../adr/0010-owned-quest-lifecycle.md)
and its [qualification ledger](../reviews/2026-11-19-owned-quest-lifecycle/README.md).

L2b adds one-link prerequisite edits between owned quests in the same
campaign. The writer enforces an acyclic graph and keeps the owned-store
format at v4 after its first prerequisite edit. See the [prerequisite decision](../adr/0011-owned-quest-prerequisites.md)
and [verification ledger](../reviews/2026-11-19-owned-quest-prerequisites/README.md).

This extends the [living control-plane contract](questmancer-living-control-plane.md)
for a work source we own. External sources retain authority over their own facts.
It supersedes the requirement to qualify Herdr Projects as the next product step.
It does not reinterpret existing caches, Chronicle records or upstream inventories.

## User outcome and appetite

The first complete milestone lets a human create a campaign and quest, record a
decision, answer it, attach an artifact and recover that work after restart.
The question must appear in the existing attention queue until its matching answer
is committed. No agent or third-party project installation is necessary.

Proposed appetite: one local store and this five-action workflow, delivered as
backend proof followed by minimal authoring interaction. Planning, delegation,
multi-process command transport and full historical replay are separate bets.
If the slice requires a daemon, database or an orchestrator, stop and reshape it.
Five-second comprehension remains an unmeasured product target; evaluate it after
the workflow can be used in a real session.

## Ownership

| Facts or actions | Authority |
| --- | --- |
| Owned campaign, quest, dependency and assignment records | Owned work backend |
| Owned decision request, literal answer and resolution | Owned work backend, through an explicit human command |
| Artifact registration | Owned work backend; registration is not proof of validation or delivery |
| Workspace, pane, session and live execution | Herdr |
| Seen/deferred attention, selection and presentation | Questmancer interface |
| External campaign or decision | Its originating source; local commands cannot alter it |
| Planning, autonomous delegation and workflow execution | Deferred; a future separate module if promoted |

Owning work does not make rendering a writer. The UI submits typed commands;
the backend validates and commits them, then emits facts for the existing reducer.
No command sends counsel, starts an agent, assigns execution or manages a worktree.

## Current primitives and gaps

Reuse source-qualified IDs and projection entities from `src/work/types.rs`, the
pure work reducer, `SourceInbox`, attention queue, Quest Board and War Table.
Preserve the live `Campaign`/workspace domain alongside durable `WorkCampaign`.
Keep the existing RGB scene and contextual parchment path.

The current `work-state.json` v2 is a projection cache, with observer checkpoints
and local intent. It has no authoring authority or stored decision answer. The
current CLI only opens the UI. L1b attaches an explicitly remembered owner at startup
and adds contextual Board authoring forms. No external command service exists.
`src/work/runtime.rs` and the cache worker contain explicit Projects variants;
add a small typed owned path only where required, preserving existing compatibility.
Do not undertake a generic plugin-framework rewrite to remove that coupling.

## Proposed component boundary

```text
Human action
  -> typed owned command + expected revision
  -> pure command validation/transition
  -> owned store commits state and command receipt
  -> LocalWorkSource emits normalized baseline/update
  -> existing work reducer
  -> acknowledged projection cache
  -> attention queue, Board, Table and selected details
```

Use `src/owned_work/` for authoritative state, commands and storage, and
`src/sources/local_work/` for projection into the existing work protocol. Keep both
in the current Rust crate initially. No daemon, HTTP server, new package or new
dependency is proposed. The backend can operate without Herdr; live observations
remain a separate input.

## Authoritative state and identity

Propose a versioned `owned-work.json` beside the existing plugin state files, opened
only through an explicit owned-store selection. Ordinary startup without that
selection must not create a store. Creation requires an explicit human action;
a missing previously selected store is a diagnostic, not a new empty project.

The store owns a stable random source identifier, aggregate revision, campaigns,
quests, decision detail, artifacts and bounded command receipts. Generate IDs with
the existing `getrandom` dependency. Entity IDs remain source-qualified and use
opaque local tokens; names, paths and Herdr pane IDs do not become identity.

Reuse the shared campaign/quest/artifact projection types where their meaning is
exact. A decision uses the existing `ObligationId` and incarnation, plus owned
decision detail with durable requester attribution, and answer text, human
attribution and supplied timestamp. Requester attribution survives receipt eviction. Project
it as `ObligationKind::Decision`; do not create a competing attention model.
The answer remains readable after the obligation leaves the active queue.
The shared obligation DTO has no answer field today. L1a keeps answer detail in
the backend and projects its resolution through existing messages. L1b must add
a typed selected-detail query/result so the UI can display the answer without
reading backend files or inventing answer text from status. Any projection schema
change needs an explicit cache migration, rather than adding unversioned fields.

Store no terminal output, raw session identity or verified live binding. Local
seen/deferred intent remains in the projection cache. No cached fixture or Projects
record is automatically imported into the owned store. Copy/import/restore policy
must be explicit before it is exposed as a user operation.

## Proposed first commands

| Command | Required behaviour |
| --- | --- |
| CreateCampaign | Explicit name/objective; allocate identity; initial Planned state |
| CreateQuest | Existing owned campaign; title/kind/optional description; initial Planned state |
| RequestDecision | Existing owned campaign and optional quest; literal prompt/context; human responsible; fresh incarnation; Open |
| AnswerDecision | Matching owned ID/incarnation and expected revision; non-empty literal answer; record answer and resolve atomically |
| RegisterArtifact | Existing owned campaign and optional quest; kind/title/optional URI; no inferred validation or completion |

All commands carry a stable command ID, expected aggregate revision and explicit
human origin. Supply time and generated IDs from the I/O boundary to the pure
transition. Reject invalid references, foreign-source targets, stale revisions,
reused command IDs with changed payload and oversized/invalid text before writing.
An already answered incarnation cannot be answered differently or silently reopen.

Store the last 128 command receipts with the same commit as the resulting state.
An identical retained retry returns its original result without another transition.
After receipt eviction, the original expected revision remains stale and must not
execute again. Rejected commands change neither authoritative state nor cache.
Dependency editing, lifecycle edits, assignment and decision reopening follow in
later commands; no readiness or execution is automatically activated in L1.

## Storage and publication

Propose bounded atomic JSON snapshots for L1, using existing serde/strict-decoding
and filesystem primitives. This fits a small local aggregate without introducing
a database or two-file journal/snapshot transaction. Retained receipts are not a
full work history. A separate journal design is required before time travel.

One store owner holds an OS advisory lock on a stable lock file for its lifetime.
Serialise commands behind that owner; a second writer fails clearly. Reject
symlink/invalid selections and observed store replacement rather than overwrite
another identity. Use a 16 MiB decoded-input/publication bound and explicit version
validation; corrupt, future-version and oversized stores stay protected.

Write to a temporary file, sync it, atomically rename and acknowledge the directory
sync. The current cache helper ignores directory-sync failure; that behaviour must
not be copied into an authoritative success claim. If replacement may have happened
but final acknowledgement fails, return an indeterminate result. Reload the store
and check the same command receipt before retry; never blindly apply it again or
roll back a possibly committed command.

Backend commit and projection-cache acknowledgement are distinct. A cache failure
after a store commit cannot undo the answer or report that it was never saved.
Expose committed-but-view-pending, retain previous projected facts and reconcile
from a complete owned baseline. Startup with a valid authoritative store can
rebuild its projection; cache-only work remains unverified. Do not silently replace
a corrupt cache or mark an older store revision Current over a newer cached cursor.

Use monotonically increasing store revisions as cursors; choose fresh observer
generations under the existing ordering contract with checked overflow. A healthy
owned baseline is Complete for this store only, with no bindings. Durable commits
drive source updates; rendering does not poll files. L1 has one in-process owner,
not independent CLI writers or an external command socket.

## Interaction and acceptance

L1a proves command/store/source behaviour through a composed test and fixed
production Board/Table models. It does not deliver user authoring. L1b adds minimal
contextual parchment forms for the five commands, committed/pending/rejected states,
answer detail and cancellation. Review those forms before implementing them.
Reuse selection and drafts; do not mix authoring with the `r` counsel flow or
repurpose seen/defer as Answer.

The decisive complete milestone is: create owned campaign/quest -> request decision
-> `!` reaches it -> mark seen/defer without resolving -> explicitly answer -> attach
artifact -> restart -> answer and artifact survive, queue excludes that incarnation.
Ordinary agent progress, exit and disconnect must never resolve it. Repeat the
command acknowledgement-loss case without duplicating facts.

Automated backend/restart proof, fixture rendering, native interaction and use in
a real Herdr session remain distinct evidence. Use the
[L1 plan](../plans/2026-10-01-owned-work.md) for implementation sequence and gates.

## Parked and deferred work

Herdr Projects acceptance, writer fixtures and M3b binding/request work are parked.
Keep its disabled-by-default adapter and historical receipts intact. Firstmate and
Projects remain design references, not dependencies or requirements for L1.
No external source is required to author the owned work.

Planning, spawning, delegation, prompt/model routing, worktrees, merge automation,
project-memory ownership, concurrent CLI writers and replay remain deferred. Full
control describes ownership of our contracts and data; it does not authorise all
of those capabilities in the first milestone.
