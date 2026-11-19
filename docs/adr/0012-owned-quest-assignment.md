---
status: accepted
date: 2026-11-19
decision-makers: Questmancer owner
---

# Record explicit assignment on owned quests

## Context

The Board and War Table can already display source-owned assignments. An
owned quest needs a durable assignee that can be set, replaced or cleared
by the Questmancer. Verified Herdr association is a separate roadmap slice.

## Decision

The Board captures a single-line assignee name explicitly entered by the
user. This supports a person, adventurer or role by name while offline. The
serial writer checks the selected quest, source, expected revision, prior
assignment and changed value. It saves the new assignment and the attributed
receipt together. The receipt stores the prior assignment so that a retained
chain of replacements and clears can be validated backwards.

Use the existing quest assignment projection. Its opaque worker value for
this source is derived from the command ID and identifies that assignment
record. Live bindings still require verified Herdr evidence through the
existing binding model. Assignment never sends counsel or changes quest
lifecycle, prerequisites, review status, or agent presence.

The first assignment edit raises the owned snapshot to v5. Keep v5 after
clearing the assignment and after receipt eviction. Prior stores retain
their existing versions until this feature is used. The cache wire format
already supports assignments. Keep a pre-v5 copy for older binaries.

## Alternatives and consequences

| Option | Consequence |
| --- | --- |
| Save a name on the selected quest | Works offline and keeps responsibility visible. Duplicate names have no shared identity. |
| Save a live adventurer or pane identifier | Couples a local work decision to a volatile execution identifier and requires verified association. |
| Create a separate assignee directory | Adds identity, rename, deletion and migration policies beyond a quest assignment. |

Check set, replacement, clearing, exact retry, stale or foreign commands,
receipt integrity, failure recovery, restart and refusal by a pre-v5 reader.
