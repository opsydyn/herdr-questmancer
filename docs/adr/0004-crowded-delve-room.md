---
status: accepted
date: 2026-11-17
decision-makers: Questmancer owner
---

# Retain the dungeon when stations overflow

The user's screenshot rejects ADR-0003's broad flat-roster fallback: four resting
adventurers plus an unknown adventurer exceed one chamber's two slots and erase
the dungeon even in a roomy native-capable pane. The owner requested a fix.
This supersedes the flat-background crowd policy in ADR-0003, not its authority,
transport, geometry or input contracts.

Keep normal station placement unchanged. When stations cannot place the party,
use a centered gathering composition over the same authored dungeon. Up to sixteen
adventurers retain full 16x24 masters in two centered rows of eight, with gutters
and cue clearance. This is a presentation grouping, not relocation of domain
stations or a new live fact. Foreground furnishings paint before gathered actors
so no actor is hidden. Keep native fitting when this complete composition fits.

For larger parties or small fallback viewports use authored 8x12 roster actors,
but paint the existing dungeon backdrop rather than a blank floor. Complete,
addressable actors are still mandatory for native output. When neither gathering
nor roster can contain everyone, retain the tested station/overflow contract.
No sprite downsampling, room/asset changes, manual actor controls or source writes.

Implementation: keep composition and gathering layout inside the existing Delve
RGB painter; share its background pass with roster composition. Native projection
checks the actual canonical composition capacity. Keep one existing world-image
cache, semantic animation deadlines, scrying/other parchment paths and Hall behavior.
Completion effects must follow the gathered actor, remain bounded and quiet down.

Test first: reproduce the reported four-resting/one-unknown distribution, retain
authored landmarks and 16x24 bodies, require native output and all mouse targets at
three pane sizes. Cover concentrated 11/16-person parties, dense fallback, unique
complete bounds, ordinary station rendering and animation limits. Add a fixed
resting-crowd Storybook case; export actual production renders before refreshing
only the currently verified owned dogfood UI. Run `just verify`, release and
isolated PTY/pixel checks. Native appearance remains user acceptance.

Boundary review: safe with noted visual risk. Only presentation/layout changes;
Herdr facts, owned work, persistence and commands retain their owners. Reusing the
RGB painter avoids a second renderer or transport. Broader station/room redesign,
a new preference or a new agent orchestration feature would exceed this fix.
