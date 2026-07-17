# Questmancer scene-first redesign

**Status:** Draft for written review on 2026-07-17

**Product direction:** Approved

**Implementation:** Not started

## Purpose

Replace Questmancer's sparse widget-composed Guild Hall and Delve with two
high-fidelity, fully authored pixel worlds while keeping the current production
interface functional throughout development.

Questmancer becomes an ambient visualisation of Codex agents managed by Herdr.
It does not become another place to prompt, steer or manually manipulate those
agents. Codex CLI remains the only agent interaction surface.

This design deliberately separates the visual rebuild from the later product
cutover. Existing functionality is neither removed nor allowed to shape the new
renderer before the pixel world has proved itself.

## Evidence

The current implementation has strong domain, persistence and truth-projection
boundaries, but its visual boundary is wrong for the intended product:

- Ratatui layout rectangles allocate large semantic zones before scenery is
  drawn;
- text labels explain landmarks that have little visual presence;
- isolated character canvases sit inside otherwise sparse widgets;
- Delve chambers read as repeated cards rather than one connected dungeon;
- the dynamic footer can consume several rows and advertises a large global
  shortcut vocabulary;
- action, selection and modal state compete with the ambient world.

Questmancer already packs two logical vertical pixels into terminal block
cells, but only for isolated assets. It lacks a full-scene RGB framebuffer,
transparent sprite blitting, continuous authored environments and a single
terminal adapter at the outer edge.

The architectural reference is Pixtuoid's separation of terminal-free scene
painting from Ratatui half-block output. Questmancer borrows that rendering
boundary and visual discipline, not Pixtuoid's office, sprites, simulation,
pathfinding, pets, weather, controls or product identity.

## Approved north star

The approved reference is:

```text
reference-art/questmancer-option-a-north-star.png
```

It locks:

- two distinct authored scenes: the Guild Hall and the Delve;
- environment-dominant composition and compact adventurers;
- continuous architecture rather than panels or cards;
- dense material detail and environmental storytelling;
- warm amber Guild Hall lighting against cool teal Delve lighting;
- a cohesive 16-bit-era pixel-art vocabulary;
- restrained operational text placed above the world only when required.

It does not lock exact room geometry, furniture placement, generated pixels,
individual character designs or every depicted prop. Production art must be
original, terminal-legible and achievable through the canonical renderer.

## Product boundary

Questmancer is a read-only projection of agent reality:

```text
User works in Codex CLI
          |
          v
Codex reports semantic state
          |
          v
Herdr owns lifecycle and event delivery
          |
          v
Questmancer renders deterministic theatre
```

Authority is explicit:

| Authority | Owns |
| --- | --- |
| Codex or another coding agent | working, blocked, done and idle state; task or status text |
| Herdr | agent discovery, pane lifecycle, workspace topology, focus and semantic events |
| Questmancer domain | normalization, transition history, effect deduplication and stable persona identity |
| Questmancer scene projection | deterministic station, pose, animation, lighting and effects |
| User | agent interaction through Codex CLI; opening or closing Questmancer as an ambient view |

Questmancer never exposes controls to:

- set presence or completion state;
- select a theatre pose or station;
- move an adventurer;
- trigger completion effects;
- make an adventurer appear active;
- type or send agent prompts;
- override facts reported by Herdr.

A response entered in Codex CLI changes the scene only after Codex and Herdr
publish new state. Questmancer does not optimistically move or animate an agent
in response to local UI input.

Manual `herdr pane report-agent` calls remain test fixtures, not product
interaction.

## World structure

The Guild Hall and Delve are separate, full-screen authored scenes joined by an
in-world guild door. They share one renderer, asset vocabulary, cast and state
projection but retain independent composition and lighting.

### Guild Hall

The Guild Hall is warm, inhabited and legible at a glance. The whole canvas is
constructed from stone, timber, rugs, tables, benches, shelves, banners,
candles, maps, scrolls and a persistent hearth.

Truthful stations remain canonical:

| Herdr state | Guild Hall theatre |
| --- | --- |
| working | compact carved token or remote miniature at the campaign table |
| blocked | projected adventurer at the Counsel Bell |
| done with a fresh completion episode | returned adventurer with one-shot spoils theatre |
| done after the completion window | calm returned adventurer or campaign token |
| idle | adventurer resting near the Hearth |
| exited | no adventurer; departure remains environmental history |
| unknown | shrouded or unlit token |

These positions are derived every frame from semantic state. No separate room
location is persisted.

Freshness is derived from the recorded Herdr transition and supplied animation
time. It is not an unread flag that the user must acknowledge. Transition
history exists to deduplicate events, age one-shot effects and preserve stable
theatre across reconnects; the new scene never exposes a local seen/unseen
workflow.

### Delve

The Delve is one continuous old-school dungeon scene rather than a set of
chamber widgets. Rooms share walls, corridors, arches, light sources and props.
Its authored variants may retain deterministic workspace identity, but their
visual differences do not claim operational facts.

Every visible agent occupies one truthful dungeon station. Selection does not
move agents because the new scene has no user-managed selection.

### Camera

The production camera follows Herdr and scene truth automatically. Workspace
focus, agent lifecycle and attention may alter emphasis or choose an authored
camera crop, but Questmancer does not add manual Guild Hall or Delve navigation
during the rebuild.

The first renderer slice does not invent a scrolling map, free camera,
pathfinding or continuous guild-to-dungeon world.

## Interaction model

The new production scene contains no agent command surface, composer, search
field, selection model, acknowledgement action, refresh command, Reviewr action
or contextual command menu.

Process lifecycle controls such as `q` or `Ctrl+C` may remain so the terminal is
always recoverable. They do not affect scene truth. Configuration remains in
the plugin configuration file rather than an interactive settings screen.

The Storybook may keep development-only controls for selecting fixtures,
scenes, states, palettes and animation phases. Those controls inspect authored
assets; they do not belong to the shipped ambient experience.

## Canonical rendering architecture

The new renderer is isolated from the current mutable application model:

```text
Herdr-backed domain model
          |
          v
read-only SceneSnapshot
          |
          v
pure Guild Hall or Delve projection
          |
          v
opaque RGB framebuffer
          |
          v
background -> architecture -> props -> sprites -> light -> effects
          |
          v
Ratatui half-block terminal adapter
          |
          v
minimal process or diagnostic overlay
```

### Scene snapshot

`SceneSnapshot` is a deliberately narrow, immutable input. It contains only
facts needed to render the ambient world, such as:

- connection state;
- workspace identity and stable campaign variant;
- agents and stable personas;
- presence, transition episodes and timestamps;
- agent and workspace focus reported by Herdr;
- approved display preferences;
- a supplied animation time.

It excludes:

- Counsel drafts;
- search input;
- modal state;
- current UI region;
- locally selected agent;
- action feedback;
- effect commands;
- pane-output loading state;
- command availability.

The same snapshot and animation time must always produce the same scene.

### Pixel core

The core renderer owns terminal-independent primitives:

- `Rgb` and an opaque, reusable `RgbBuffer`;
- transparent palette-keyed sprite frames;
- deterministic animations;
- clipped sprite blitting;
- horizontal mirroring where required;
- authored tiles, props and lighting masks;
- scene painter order and z-order anchors.

The core does not depend on Ratatui, Crossterm, terminal dimensions expressed as
cells, terminal input or the runtime clock.

### Terminal adapter

Only the outer adapter knows Ratatui. Each terminal cell represents two logical
vertical pixels:

```text
top logical pixel    -> foreground RGB
bottom logical pixel -> background RGB
cell glyph           -> U+2580 upper half block
```

The adapter writes directly into the Ratatui buffer, clips to both the supplied
rectangle and the actual buffer, handles odd logical heights, survives resize
races and performs no per-cell allocation.

### Text and diagnostics

Ordinary Ratatui text is drawn after the RGB scene. It is reserved for exact
diagnostics, process lifecycle and a deliberately small amount of factual
identity. Text does not describe scenery that should have been drawn.

## Parallel migration

The current production UI remains unchanged and fully functional while the new
world is built. Its existing tests continue to run.

The new pixel-world stack is introduced beside it and exercised through the
feature-gated Storybook or an equally isolated development binary. It must not
be threaded incrementally through the existing Great Room layout, footer,
modal or interaction code.

Rules during migration:

1. Do not remove current functionality before the cutover review.
2. Do not add product features to the legacy control surface.
3. Do not make the new scene depend on legacy selection or modal state.
4. Keep legacy regression tests green.
5. Add renderer and scene tests at their new boundaries.
6. Keep the production plugin default on the existing UI until the visual and
   operational gates pass.
7. Do not ship generated north-star pixels as production art.

Git history alone is not treated as preserving functionality. The current UI
must remain buildable and runnable during the parallel phase.

## Development sequence at design level

Detailed tasks belong in the implementation plan. The design-level sequence is:

1. Establish the terminal-independent framebuffer, sprite model, clipped blit
   and half-block adapter.
2. Prove the renderer through a small Storybook scene with original assets.
3. Build one complete Guild Hall vertical slice at north-star density.
4. Extend the same engine to one complete Delve vertical slice.
5. Feed both scenes from a read-only `SceneSnapshot` built from real domain
   fixtures and then live Herdr state.
6. Validate terminal sizes, performance, motion settings and truthful state
   coverage.
7. Hold a cutover review before deleting, hiding or reintroducing any legacy
   control.

## Testing and evidence

### Pixel primitives

Tests cover:

- RGB indexing and clear behavior;
- transparent and opaque blits;
- partial and fully off-screen clipping;
- painter order;
- frame selection and wraparound;
- mirroring;
- odd-height half-block output;
- non-zero destination origins;
- resize-race bounds;
- deterministic output for fixed inputs.

### Scene projection

Property tests cover:

- every non-exited agent appears exactly once where the scene requires it;
- no exited agent appears;
- Herdr focus changes emphasis without changing stable identity;
- no projection writes domain state;
- arbitrary terminal sizes do not panic;
- scene placement and variant selection remain deterministic;
- every visible pose is justified by Herdr state and deterministic transition
  semantics.

### Visual evidence

The Storybook contains fixed, reviewable scenes for:

- empty Guild Hall;
- populated mixed-state Guild Hall;
- blocked attention;
- returned spoils;
- connected and reconnecting states;
- one authored Delve;
- mixed-state Delve;
- minimum supported viewport;
- reduced and no-motion modes.

Pure RGB outputs may be captured as PNG fixtures for visual review. Ratatui
buffer tests remain structural and avoid giant brittle text snapshots.

### Performance

The renderer reuses buffers, does not allocate inside the terminal flush, does
not reload pane output, and schedules frames only when a visible animation or
semantic event requires them. Static scenes remain event-driven.

## Compatibility during the rebuild

The first new scene targets Unicode-capable true-colour terminals. The existing
production UI continues to provide the current ANSI and ASCII compatibility
while the new renderer is under development.

The cutover review must explicitly decide whether to:

- add an ANSI-256 quantisation backend;
- retain the legacy renderer as a compatibility path; or
- document true colour as a requirement for the scene-first release.

That decision is not hidden inside the renderer foundation and does not block
the fidelity proof.

## Cutover review

The production UI changes only after evidence answers all of these questions:

1. Does the Guild Hall visibly meet the approved density, scale and warmth?
2. Does the Delve read as a connected authored dungeon?
3. Is every visible state truthful to Herdr and Codex?
4. Does the world remain useful with no agent command surface?
5. Does the automatic camera make the relevant state understandable?
6. Does the renderer work at the agreed minimum terminal size?
7. Are reduced-motion and no-motion behavior acceptable?
8. Is idle cost negligible and active animation bounded?
9. What compatibility backend is required for release?
10. Should the legacy UI be deleted or retained only as a development binary?

No control is preserved in the new production experience merely because it
already exists. No control is removed from the current production experience
before this review.

## Non-goals

This redesign does not include:

- prompting agents from Questmancer;
- direct sprite manipulation;
- a sprite editor;
- procedural dungeon generation;
- pathfinding or collision simulation;
- an ECS or game engine;
- weather, pets, sound or day/night simulation;
- Kitty, Sixel or iTerm image protocols;
- multiple visual themes;
- a continuous navigable guild-to-dungeon map;
- a plugin SDK for other renderers;
- copied Pixtuoid assets or generated north-star pixels.

## Success condition

This design succeeds when Questmancer can render an original, dense and
coherent Guild Hall and Delve from Herdr state through a reusable RGB scene
pipeline, while the existing production UI remains fully functional until a
separate evidence-based cutover decision.
