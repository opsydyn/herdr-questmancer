---
status: accepted
date: 2026-11-19
decision-makers: Questmancer owner
---

# Edit prerequisites between owned quests

## Context

The War Table already displays prerequisites from source work. In the owned
store, quests can change lifecycle state but cannot author those links. The
approved slice is to add or remove a prerequisite within one owned campaign.

## Decision

The selected quest depends on the chosen prerequisite. The Board offers one
explicit Add or Remove choice and a save action. The serial writer validates
both quest IDs, their shared campaign, the expected store revision, whether the
link already exists, and acyclicity. A successful command updates the dependent
quest's revision and commits its attributed receipt in the same snapshot. The
existing projection supplies the Board and War Table after acknowledgement.

Use the existing ordered quest dependency field. Store v4 is selected on the
first successful prerequisite edit and retained by the snapshot envelope
after link removal and receipt eviction. Reads and ordinary writes retain
the v1–v3 compatibility rules until the first prerequisite edit. Older
readers refuse v4. The work cache already supports dependencies and keeps
its existing format. A pre-v4 copy is the rollback path for older binaries.

## Alternatives and consequences

| Option | Consequence |
| --- | --- |
| Separate graph records or a second store | Adds identity and transaction work for an existing quest field. |
| Replace the entire prerequisite list in one command | Makes one edit easier to overwrite and obscures which link the receipt represents. |
| Save a single add or remove in the owned store | Gives each link change an explicit receipt and reuses the current publication and graph paths. |
| Derive the format from current links alone | Loses the compatibility fence when the final link and its receipts are gone. |

Reject invalid current graphs and check retained quest edits against the
saved effects. The bounded receipt window does not reconstruct the whole
history. Quest lifecycle, summons, assignment and Herdr execution remain
separately controlled.
