---
status: accepted
date: 2026-11-17
decision-makers: Questmancer owner
---

# Extend proportional fitting to the Delve

The owner asked for the Hall's proportional sprite-preserving fit and complete
crowded-party fallback in the “scrying / dungeon view”. Interpret dungeon as the
Delve; the scrying parchment retains its existing terminal-output controls.
This extends ADR-0002's Hall-only boundary after explicit user promotion.

Both rooms have authored 160x90 RGB worlds. Reuse the existing pure fit transform,
Kitty cache, startup capability query, footer, semantic deadlines and projected
cell hit regions for both. Rename the production output adapter to `world_fit`;
there is one cache with two owned image identities across room switches, rather
than one adapter or identity pool per room. A world switch replaces the image
and retires only the previous owned lease. No new dependency or stored state.

Make canonical capacity checking world-specific. Hall station allocation must
not constrain the Delve. A fitted Delve is allowed only if its own canonical
stations place every non-exited adventurer with an addressable complete sprite.
Its two-per-station capacity remains authored and truthful. When saturated,
prefer the existing complete roster at any pane size where it fits. If that
roster cannot fit, retain the independently tested camera/overflow contract.
Do not compress source art, expand stations or invent successful completion.

All modal parchments, cards, unsupported Kitty/geometry/tmux/ANSI16 and tiny
panes retain the existing half-block path. The dungeon's fallback camera and
roster thresholds otherwise remain. Keep Herdr topology, counsel, output
requests, owned-work commands and persistence unchanged. Rendering remains pure.

Implementation: `scene/fit` dispatches canonical capacity by world;
`scene/render/delve` selects roster on saturation; `ui/world_fit` replaces the
Hall-only adapter. Add normal/crowded Delve Storybook fixtures and reuse preview
controls. Test-first evidence must cover native Delve output and saturated
station fallback, every source pixel/click cell at three sizes, both directions
of room switching, scrying/card suspension, fallback, cache cleanup and deadlines.
Run feature-enabled suites, full `just verify`, release build and isolated PTY
producer/pixel checks. Refresh only the verified retained dogfood UI and leave
the party open. Native visibility/animation/mouse/resize acceptance remains human.

Simplicity check: extending the existing 160x90 adapter avoids duplicate transport,
geometry and cleanup state. A second renderer or Delve-only cache would add
unnecessary coordination; changing sprite masters would discard approved detail.
Rollback remains selecting the existing half-block path. No commit, publish,
server restart, external source attachment or AT Protocol integration is included.
