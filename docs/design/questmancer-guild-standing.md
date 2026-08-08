# Questmancer guild standing

Status: implemented. One experience score for this Questmancer install.

## Decision

Questmancer keeps a single lifetime experience score, shown as a badge in the
top-right corner of the room and in full on the Librarian's Ledger. It is
earned by work the Chronicle recorded, it only ever climbs, and it unlocks
nothing.

## Why the guild rather than the adventurer

The obvious design is a level per adventurer — it is what a tabletop party
would do. It does not survive contact with how agents actually exist.

`PersonaKey` is derived in three tiers:

```
session\0{source}\0{agent}\0{kind}\0{value}   stable across restarts
workspace-agent\0{root}\0{name}               stable while the name holds
pane\0{workspace}\0{pane}                     lost when the pane is recreated
```

An adventurer without a session identity falls through to the pane tier, so a
level attached to it would reset silently — for exactly the agents least able
to explain why. Parties change every session; the guild is the thing with
continuity, so the guild is what keeps the score.

This also matches what was actually asked for. The score belongs to the person
running the guild, not to a sprite that will be gone next week.

## What earns it

| Chronicle event | Worth | Why |
| --- | --- | --- |
| Spoils returned | 10 | work finished |
| Campaign closed | 25 | a milestone finished |
| Counsel requested | 0 | an adventurer got *stuck* |
| Adventurer joined, delve began, rested, departed | 0 | not work |

`CounselRequested` earning nothing is the deliberate one, because it is the
event a guild master acts on most. It records somebody getting stuck rather
than anybody getting unstuck; paying for it would reward agents for blocking.
There is no "counsel given" event in the Chronicle to reward instead.

## Two rules the implementation follows

- **Stored, never derived.** The Chronicle is a bounded ring, so a score
  computed from its contents would *fall* as history rolled off — the same trap
  `$quest_hoard` documents, except standing is supposed to be permanent.
  `the_chronicle_forgets_but_standing_must_not` demonstrates the decay a
  derived score would suffer.
- **Earned where the Chronicle dedupes.** The award hooks the one place the
  reducer emits `AppendChronicle`, which fires only for events the Chronicle
  judged new, so the same returned spoils cannot be paid for twice. Startup
  replay assigns the Chronicle wholesale rather than appending through the
  reducer, so a restart re-reads history without re-earning it.

## Compatibility

The field carries `#[serde(default)]` and `STATE_SCHEMA_VERSION` stays at `1`.
A state file written before standing existed loads with its personas intact and
starts at zero. Bumping the schema would have discarded them, because
validation is an exact-match check that fails closed.

## Non-goals

- **No gating.** Standing unlocks nothing and hides nothing. A tool that
  withheld features until you had used it enough would be a worse tool.
- **No per-adventurer levels**, for the identity reasons above.
- **No score for presence.** Nothing is earned for having the plugin open, or
  for an agent merely existing. This is the one number Questmancer keeps that
  is not a fact about your agents, and it stays honest only while every point
  traces to work that actually happened.
