# Questmancer scene-first production cutover

**Status:** Approved design

## Objective

Make the approved RGB Guild Hall and Delve the only production renderer. Remove
the legacy dashboard renderer rather than retaining a fallback that would need
parallel maintenance.

The cutover preserves Questmancer's operational utility. The pixel world is the
primary surface, while selection, search, reply, pane focus, output inspection
and explicit world switching remain available through contextual interaction.

## Product contract

- `questmancer` launches the RGB scene renderer in normal plugin use.
- Guild Hall and Delve remain full-screen authored pixel worlds.
- Herdr agent state determines each adventurer's station, pose and effects.
- `1` and `2` explicitly switch between Guild Hall and Delve.
- `j`/`k` and arrow keys select adventurers.
- `Enter` focuses the selected adventurer's Herdr pane.
- `r` opens a compact reply parchment. Enter sends and Esc cancels.
- `/` opens search over known adventurers and campaigns.
- Output inspection appears as a dismissible scrying overlay.
- Selection is represented in-world by a lamp, rune ring, outline or equivalent
  authored treatment rather than a dashboard panel.
- A minimal command ribbon appears contextually after input or through help; it
  does not permanently consume scene space.
- `q` and Ctrl-C exit safely and restore the terminal.

## Architecture

The existing `SceneSnapshot -> ScenePlan -> RgbBuffer -> half-block Ratatui`
pipeline becomes the production terminal path. The normal runtime continues to
own Herdr supervision, persistence, commands, input and animation scheduling.
Scene rendering consumes an immutable snapshot plus a small presentation state
containing the selected adventurer, chosen world and active overlay.

Interaction is separated into three layers:

1. Domain reduction preserves the existing typed commands and persistence
   boundaries.
2. Scene interaction projects selection and modal state into in-world markers
   and contextual overlays.
3. RGB composition paints the world first and overlays second, then performs a
   single half-block flush into Ratatui.

The Storybook remains the development-only review surface for authored assets,
worlds, states and overlays. It shares production scene components rather than
maintaining copies.

## Removal boundary

Remove the legacy Ratatui dashboard views, layout projections, widgets and
renderer-specific tests once every retained interaction has scene-first test
coverage. Remove the `scene-preview` feature and separate preview binary after
the normal binary uses the same pipeline. Do not retain a runtime renderer
switch, configuration fallback or compatibility branch.

Keep shared domain, protocol, persistence, command, persona and scene asset code.
Compatibility settings that only exist to select legacy glyph or colour paths
are removed when they no longer have a scene-first meaning.

## Failure behaviour

- Socket loss leaves the current world visible and adds reconnecting truth to
  the scene.
- Commands surface non-blocking errors in the active parchment or command
  ribbon without replacing the world.
- Tiny or zero-sized terminal areas remain panic-free.
- A terminal too small for an overlay shows a compact modal rather than falling
  back to the legacy renderer.
- Terminal restoration remains guaranteed for normal exit, signals and panic.

## Verification

Automated tests must prove:

- the normal `questmancer` binary enters the scene-first runtime;
- no production or release path references the preview binary or legacy
  renderer;
- Guild Hall and Delve render through the RGB pipeline;
- selection, focus, search, reply, output inspection and world switching retain
  their existing typed effects;
- overlays do not mutate agent truth or duplicate commands;
- reconnect, animation cadence, narrow viewports and terminal restoration work;
- Storybook uses the same scene components as production;
- scripts, release packaging, formatting, Clippy, unit, integration and property
  tests pass.

Manual acceptance requires the linked Herdr plugin to show both approved worlds,
exercise each retained interaction, reconnect cleanly and reopen as a singleton
without exposing any legacy dashboard surface.

## Non-goals

- Maintaining the legacy renderer as a fallback.
- Adding mouse-first interaction, audio or terminal image protocols.
- Reintroducing permanent side panels or dense dashboard chrome.
- Allowing visual controls to invent or override Herdr agent state.
