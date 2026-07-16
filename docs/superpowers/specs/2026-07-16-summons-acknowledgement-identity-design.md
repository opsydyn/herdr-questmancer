# Summons acknowledgement identity

## Context

Live Herdr 0.7.4 acceptance proved that a blocked-event acknowledgement was
written with `pane_revision = 1`, then discarded after restart because
`session.snapshot` represented the same blocked agent with `revision = 0`.

The installed protocol schema explains the mismatch:

- `pane.agent_status_changed` carries no revision;
- Questmancer therefore synthesizes a transition counter while it is running;
- `AgentInfo.revision` in a snapshot is the pane/output revision used by pane
  reads, not a durable agent-status episode identifier.

The old persistence key compared unrelated counters.

## Decision

An acknowledged Summons is durable user intent identified by:

```rust
pub struct AttentionEpisodeKey {
    pub persona: PersonaKey,
    pub summons: GuildSummons,
}
```

`pane_revision` is removed from the persisted identity.

The overlay restores a read acknowledgement when the same stable persona still
has the same unread Summons. The intent is removed when Questmancer observes
that persona with no Summons or with a different Summons. A new observed
transition therefore becomes unread normally.

## Compatibility

Questmancer v0.1 is not released. Existing development `state.json` records may
still contain `pane_revision`; Serde's normal unknown-field tolerance accepts
that field, and the next successful write emits the corrected two-field key.
The v1 schema number remains unchanged because this is a pre-release correction
with a backwards-readable representation, not a released migration contract.

## Honest limitation

Herdr 0.7.4 exposes no durable status-event identity. If an adventurer leaves a
Summons state and returns to the same Summons entirely while Questmancer is
closed, the restart snapshot is indistinguishable from an unchanged episode.
Questmancer preserves the prior acknowledgement in that case. Any transition
observed while Questmancer is open clears or replaces the intent correctly.

## Safety and tests

- Persistence never alters Herdr-owned topology, status, pane revision or
  output.
- Exact persona identity remains the cross-restart boundary.
- Tests must prove revision drift no longer drops acknowledgement, observed
  state/summons changes do clear it, old JSON is readable, and new JSON omits
  the transport revision.
- The complete property and persistence suites remain green before live retest.
