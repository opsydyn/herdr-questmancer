---
status: accepted
date: 2026-11-19
decision-makers: Questmancer owner
---

# Record explicit quest lifecycle in owned work

## Context

Questmancer v0.2.4 can author quests and retain decisions, artifacts and
revision reviews. Owned quests currently stay Planned. A settled review records
the outcome for a specific Git commit. A human needs a separate way to record
whether the quest itself is active or complete and recover that fact after a
restart or adventurer exit.

## Decision

Start L2 with a Board action for the selected owned quest. Show the current
state and proposed transition before the user confirms.

| Action | Current state | Saved state |
| --- | --- | --- |
| Mark active | Planned | Active |
| Mark complete | Active | Completed |
| Reopen quest | Completed | Active |

The serial owned-work backend validates the source, quest, expected revision
and transition. Commit the new state, entity revision and attributed command
receipt together. The Quest Board and War Table show the resulting state through
the existing work projection. Open decisions and review summons remain visible
and actionable after a lifecycle change.

Use the existing shared quest states. The new lifecycle receipts change the
owned snapshot contract, so use format v3 on the first successful lifecycle
write. Opening an existing v1 or v2 store does not rewrite it. Writes before the
first lifecycle change follow the current version rules. Keep v3 after receipt
eviction and preserve the reader's protection for unknown formats. Revisit
quest receipt validation for mutable quest revisions.
An earlier CreateQuest or lifecycle receipt can remain valid after a later
transition, and the latest retained transition for a quest must agree with its
committed state.

## Options considered and trade-offs

| Option | Consequence |
| --- | --- |
| Keep status in quest text | A description can communicate progress, but the Board and War Table cannot use it as an authoritative state. |
| Derive lifecycle from a settled review or a Herdr status | Low interaction cost, with quest completion inferred from evidence for a different fact. |
| Confirm an owned lifecycle change in the Board | Adds a deliberate action and keeps quest state authoritative across disconnect and restart. |

An explicit completion records a human decision about the quest. Review
settlement, PR state, adventurer presence and Chronicle experience keep their
own authority. Dependency editing, assignment and agent execution are later
decisions.

## First implementation and verification

Proposed appetite is two engineering days for one Board-to-store workflow.
Start with a temporary owned store and prove Planned → Active → Completed →
Active across reopen and restart. Check existing v1 and v2 stores, later receipt
eviction, stale revisions, foreign sources, repeat commands and failed or
indeterminate saves. Verify that Board and War Table recover the committed
state, and that lifecycle changes leave decision and review records intact.

If the workflow requires a new authority, an unbounded history or changes to
external sources, revisit the scope before adding those dependencies. Revisit
after the restart proof. The prepared Ghostty acceptance for the shipped
authoring and Inbox remains a separate human check.
