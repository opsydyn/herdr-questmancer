# Guild Hall art direction

Status: approved direction; renderer implementation awaits visual review.
Landed so far: persona palette substitution in world masters, the compact and
vignette quiet stage, state-first nameplate truncation, and the garb-versus-
material contrast floor with its automated proof.

Scope: Guild Hall first. The Delve has since adopted the same actor-legibility
rules while keeping its darker dungeon direction: its party grounds against the
deepest stone value, blocked delvers carry the same authored counsel marker,
and a pane too small to hold the party inside a camera crop recomposes at
roster scale rather than leaving delvers off-camera.

The visual north star is
[`reference-art/questmancer-option-a-north-star.png`](../../reference-art/questmancer-option-a-north-star.png):
a dense, lived-in pixel room with clear figures, purposeful stations and
layered light. It is a composition and readability reference only. Questmancer
does not copy its art, code or camera.

## Product job

The Guild Hall is an operational world, not wallpaper. Within roughly two
seconds, the Questmancer must be able to answer:

1. Which adventurer needs counsel?
2. Who is working, resting or returning with spoils?
3. Which campaign are they part of?
4. What can I select, observe or counsel next?

Cosiness is successful only when it makes those facts quicker to see. The
current Hall contains recognisable props and expressive adventurers, but its
warm wood, rugs, tables and nameplates often occupy the same visual plane as
the actors. Adding uniform texture would make that problem worse.

## Direction

Keep the current side-on, cutaway Hall camera. Do not pivot this pass into a
top-down map, a generic tile engine or a new renderer. Instead, give the room
the depth grammar of a carefully staged pixel scene:

- **architectural shell:** rear stone, beams, door and fireplace establish the
  room without competing with actors;
- **station bays:** each activity has a recognisable prop cluster and one
  deliberate adventurer reservation;
- **grounding:** an actor has a contact shadow, local floor treatment and a
  restrained rim or light value that separates it from the bay;
- **foreground restraint:** rugs, benches and trim frame the room, but never
  obscure a complete visible actor or its hit region; and
- **quiet lanes:** visual rest and label lanes remain around the party instead
  of filling every available pixel with texture.

The desired result is a warm room with detail at the walls, fixtures and
station edges, and calm contrast around faces, gear and state signals.

## Station blueprint

The canonical Hall should read as a sequence of named, connected places rather
than a collection of furniture. The final coordinates are implementation work;
these roles and occupancy rules are the contract.

```text
rear wall:      [Librarian's shelves] [quest board / map] [counsel dais]
working floor:  [guild door] [campaign table] [campaign table] [hearth]
front floor:    [return lane] [party table]    [spoils ledger]  [overflow alcove]
```

| Bay | Purpose and visual anchor | Occupancy rule |
| --- | --- | --- |
| Guild door | departures and returned adventurers; door, travel kit and threshold light | one complete actor reservation |
| Quest board | campaign direction; map, notices and wax seals | landmark only unless deliberately promoted as a station |
| Campaign tables | working adventurers; papers, tools and chairs | one actor per table reservation |
| Counsel dais | blocked adventurers; bell, raised platform and bright signal lantern | one actor, highest attention priority |
| Hearth | resting adventurers; fire, chair and warm pool | one actor, no urgent label lane through the flame |
| Spoils ledger | completed/returned work; ledger, lockbox and small trophy light | one actor, completion theatre is one-shot |
| Librarian's shelves | persistent help NPC and the Librarian's Ledger | Librarian remains visible and clickable; never competes for party capacity |
| Overflow alcove | truthful overflow when the party exceeds Hall capacity | a separate counted/labelled arrangement; never stack actors inside another bay |

Each station reserves three rectangles before rendering: the actor footprint,
its shadow/light pool, and a label lane. Reservations may not overlap. A
station becomes unavailable rather than shrinking a `16x24` adventurer into a
token.

## Actor legibility contract

At the canonical Hall size and the compact whole-party composition, every
visible actor must retain its authored native-scale world sprite and have:

- a one-to-two RGB-pixel contact shadow or grounded base;
- a local value contrast between its silhouette and the floor/wall behind it;
- a state signal with a shape or icon as well as colour;
- enough negative space to read primary gear and face at a glance; and
- a visible selection treatment stronger than four detached corner pixels.

Selection is a modest rune ring on the floor beneath the selected adventurer,
scaled to the master it marks. It must clarify focus without turning the room
into a dashboard. Its colour is reserved: nothing else in either world may
paint it, or the room grows false positives.
Blocked adventurers gain the highest local contrast and a distinctive counsel
signal. Working, resting, returned-with-spoils, unknown and exited states stay
truthful and calmer. Completion effects remain short transition theatre, never
a permanent beacon.

## Light, palette and material rules

The Hall is candlelit, not uniformly brown. Use a constrained hierarchy:

- deep cool or neutral shadow for recesses and distant architecture;
- mid-value warm wood and stone for most construction;
- warm pools around hearth, lanterns, books and the counsel bell;
- bright values only for interaction, state attention and meaningful prop
  accents; and
- actor palettes protected from matching their immediate floor or wall.

Texture belongs in clusters: stone variation, beam grain, shelf contents, wax
seals and rug motifs. It must reduce in density behind faces, weapons, staffs,
shields and label lanes. Avoid full-scene speckle, procedural noise beneath
feet, large unbroken brown fields and decorative detail that reads as a state
indicator.

## Identity labels and overlays

World labels are status instruments, not captions for every sprite.

- The selected adventurer and any adventurer needing counsel receive a full
  name-and-status plate in the bay's reserved label lane.
- Quiet adventurers receive a compact marker, or no persistent plate when
  their station and state are already legible.
- A label may never cover an actor, another label, a station's state-critical
  prop, or the Librarian.
- The parchment Adventurer Card remains the place for full identity,
  campaign, elapsed status, output and actions. `Esc` dismisses it.
- The Librarian's Ledger remains the single help system. It is neither a party
  member nor an overflow solution.

The existing selection, search, counsel, scrying and reply flows are retained.
This is a visual hierarchy pass, not a change to Herdr truth or input controls.

## Responsive contract

The small-viewport policy is a five-rung ladder: canonical whole Hall, then a
capacity-checked compact whole-party layout, then a **roster** of authored
`8x12` masters when the party no longer fits at world scale, then a
priority-adventurer vignette, then status-only rendering. The Hall must
recompose; it must not crop a busy canonical scene and call that responsive.

The roster rung exists because a Questmancer pane is usually narrow: dropping
straight from compact to a single adventurer answers "who needs counsel" only
by accident, and answers "how large is my party" not at all. Roster masters
are authored per silhouette family, never mechanically downscaled, and carry
no pose — state is told by grounding, the counsel marker and the nameplate.
A station becomes unavailable rather than shrinking a master into a token;
the roster is a different authored size, not a squeezed one.

At every size, the selected adventurer wins priority. Otherwise an adventurer
needing counsel wins priority. The Librarian stays visible and selectable when
the vignette can fit both actors; otherwise the handbook remains reachable via
the normal help path.

## Renderer seam and invariants

The eventual implementation stays within the current scene-first pipeline:

```text
SceneSnapshot -> ScenePlan -> Guild Hall RGB renderer -> half-block adapter
                                           -> contextual overlays
```

It will touch authored Hall assets, the Guild Hall station layout, lighting,
actor grounding and identity-label placement. It must not add a parallel
terminal-image world renderer, manual sprite controls, a new theme framework,
or persistence of live topology. Hit regions continue to describe complete,
visible actors only.

## Storybook review gate

Before production rendering changes are accepted, add or revise fixed
Storybook coverage for these truthful situations:

| Review story | What must be inspected |
| --- | --- |
| Hall / one resting adventurer | silhouette, shadow, calm label behaviour and station read |
| Hall / working party | distinct stations, no actor or label collisions |
| Hall / counsel required | blocked actor is the fastest visual answer without colour alone |
| Hall / returning with spoils | one-shot completion treatment and spoils station read |
| Hall / full party and overflow | no stacking, truthful overflow treatment and Librarian visibility |
| Hall / compact and vignette | recomposition, native-scale sprites and priority selection |
| Hall / roster | whole party visible at authored 8x12, no shared silhouette edges, counsel marker per blocked adventurer |

Review the stories at the canonical `160x90` RGB world, a compact viewport and
a small Ghostty window. Automated tests should enforce station/label
non-overlap, actor-footprint visibility, deterministic output, hit-region
correctness and compact/vignette selection priority. They prove constraints,
not visual quality: final art approval remains a manual product gate.

## Non-goals

- No Delve architecture redesign: the dungeon layout, lighting and camera are
  unchanged. Only actor legibility and the responsive ladder were adopted.
- No copied Pixtuoid assets or code.
- No native-image sprites in the world scene; native portraits remain card-only.
- No return of the legacy dashboard or old renderer.
- No change to Herdr-owned state, counsel semantics, search or selection.
