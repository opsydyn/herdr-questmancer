# Hall responsiveness assessment — 2026-11-17

- result: complete
- claim: Current production exports reproduce the oversized compact Hall, identify the layout/scale coupling and compare a whole-scene fit proposal without changing production.
- evidence: User screenshot, current source, 18 deterministic production renders, `measurements.json`, the two review sheets and `verification.json`.
- remaining uncertainty: The screenshot's exact cell geometry is unmeasured. The proposed native-image transport, labels, interaction mapping, animation delivery and native resize behaviour have not been implemented or qualified.
- next: Review the fit proposal, then promote a bounded Storybook/native-output spike before any production renderer or contract change.

## Recommendation

**Reshape responsiveness around stable room and actor proportions.** The user's
goal is a smaller terminal that still feels like a complete guild, with smaller
characters retaining their authored detail. The current medium layout maximizes
native-size sprite capacity, which solves a different problem.

The strongest candidate for the requested result is a capability-gated **fit
scene** presentation: render the same authored `160x90` RGB Hall, then display it
through native terminal graphics at a smaller physical scale with preserved
aspect ratio. The existing world renderer and domain projections can remain;
this would add an output adapter and a coordinate transform, not another world
implementation. Current production confines native graphics to cards/Ledger,
so expanding that contract needs an explicit decision before integration.

Keep an authored half-block layout for unsupported transports. Smaller authored
sprites can improve those layouts, but have a smaller information budget. The
existing `8x12` roster is partly family-based and cannot promise all the detail
of the newly approved `16x24` class masters. Do not mechanically downsample those
masters and call that equivalent art.

## Visual evidence

- [Current compact output versus fitted canonical scene](01-current-versus-fit.png).
  Left is current production RGB output. Right is a **hypothetical presentation**
  of the actual canonical RGB buffer in the same assumed physical pane. It uses
  integer nearest-neighbour display scaling, with no source pixel discarded.
- [Current resize breakpoints](02-current-breakpoints.png). All six panels are
  actual current production renders at one fixed display scale.

Both use five fixed fixture adventurers: Druid Working, Bard Blocked, Cleric
Working and Sorcerers Idle/Unknown, plus the existing Librarian. They are not
screenshots or captures of the live party. The comparison assumes `8x16` physical
pixels per terminal cell; it does not claim that is the user's measured font
geometry. Native-fit panels exclude text overlays, cards and actual mouse input.
They illustrate the visual direction, not functioning Kitty output.

## Findings

### 1. The medium Hall changes the composition but keeps character scale

**Observed in source and fixture renders; visible in the supplied screenshot.**
`src/terminal.rs:534` creates a scene buffer of terminal columns by twice the
rows. `src/ui/scene_adapter.rs:21` maps each logical RGB pixel column directly
to a terminal column. A `16x24` actor therefore always occupies sixteen columns
and twelve rows in this path.

`src/scene/render/guild_hall.rs:205` selects the canonical Hall only at `160x90`
RGB pixels or above: **160 columns × 45 rows**. Below either axis, a
capacity-checked compact layout keeps the same `16x24` masters. Its architecture
is recomposed, and `compact_actor_origin` packs the party and Librarian into a
bottom-aligned grid. This explains the lineup and lost room depth.

For five adventurers, measured results are:

| Terminal cells | Current composition | Actor height / scene height |
| --- | --- | --- |
| 160×45 | Canonical | 27% |
| 160×44 | Compact | 27% |
| 120×34 | Compact | 35% |
| 96×28 | Compact | 43% |
| 80×24 | Compact, two rows | 50% |
| 60×20 | Roster, `8x12` | 30% |

The **one-row cliff at 160×45 → 160×44** replaces the complete room even though
almost the same area remains available. At smaller sizes, capacity permits
sprites that dominate the pane. The scene is not being geometrically squashed;
the alternate composition is losing the intended room/actor relationship.

### 2. The small roster starts too late to preserve the desired proportions

**Observed in source and fixture renders.** Compact is chosen before roster
whenever minimum dimensions and capacity permit. Thus five actors remain large
at 80×24 while eleven actors switch to the smaller roster at that same size.
The same terminal changes character scale when membership changes.

Moving the existing roster threshold earlier is a cheap fallback improvement,
but by itself still gives an inventory-like arrangement, not the fully furnished
guild in the user's request. A future authored small Hall could distribute
smaller actors around stations; its class readability needs its own art review.
No additional fourteen-class art project is proposed for the initial spike.

### 3. Canonical capacity is station-dependent, not just an eleven-person limit

**Observed in the eleven-adventurer fixture.** At 160×45 and 180×50, the frame
returns **7/11** visible adventurers for this single-campaign state distribution.
At 160×44, compact returns **11/11**. `actor_anchors` exhausts the campaign and
other station slots plus overflow alcoves; the canonical composition gate checks
only total actor count. See `src/scene/render/guild_hall.rs:1082` onward.

This is a concrete visibility failure in this fixture, not a claim about every
eleven-person party. A native-fit implementation that always requests the
canonical canvas would inherit it. Before production integration, eligibility
must consider actual placements and provide truthful overflow/fallback behaviour.
Showing a pleasing full room is insufficient if some adventurers disappear.

### 4. Full-detail reduction needs a denser output path

**Source constraint; native-fit feasibility remains a hypothesis.** Half blocks
provide two independently coloured vertical RGB samples per cell. Reducing a
`16x24` master to eight columns in that output necessarily removes samples or
requires a separately authored representation. A global terminal-font reduction
would shrink every UI element and may reach the canonical breakpoint, but does
not provide app-owned scene scaling.

The [Kitty graphics specification](https://sw.kovidgoyal.net/kitty/graphics-protocol/#controlling-displayed-image-layout)
supports fitting an image into a requested column/row rectangle with aspect
preservation and supports images under text. That makes the proposed adapter
plausible. It does not prove resize, redraw, stacking or throughput through the
installed Herdr pane. Existing native portraits establish only the already
observed card path. The earlier
[capability probe](../../../reviews/2026-11-16-native-capabilities/README.md)
also explicitly separates acknowledgements from actual native playback.

## Proposed bounded next slice — not implemented

Promote a **Storybook-only native-fit spike** for one Guild Hall fixture,
starting with negotiated Kitty transport:

1. Use the existing RGB renderer, approved assets and fixed snapshot. Derive
   displayed scale from cell pixel geometry and pane bounds. Prefer integer
   image scaling where possible, preserve aspect ratio and letterbox excess.
2. Fit at 120×34, 96×28 and 80×24 cells. Compare five and eleven adventurers.
   Detect incomplete station placement; do not silently present seven as eleven.
3. Keep labels at readable terminal text size. Show selected/urgent identity
   first when label space is limited. Prototype their allocation rather than
   shrinking all labels with the picture.
4. Map actor/Librarian hit regions through the exact fit transform. Check every
   selectable actor and boundaries after resize, with keyboard navigation intact.
5. Exercise card/Ledger stacking, closing overlays, repeated resize, hiding/showing
   the pane, reconnect and cleanup using only test-owned image IDs/resources.
   Keep existing render deadlines; require no wakes in static scenes and measure
   full-motion redraw/upload cost rather than adding an independent animation loop.
6. Qualify an unsupported/graphics-off fallback. A native query acknowledgement
   must not leave a blank world when transport or geometry is unavailable.

The proposed appetite is one bounded qualification slice, ending in review
images, runtime evidence and a pursue/reshape decision. Stop or reshape if cell
geometry, redraw, stacking, hit mapping or complete-party visibility cannot be
made reliable. If native fit fails, review an authored smaller Hall using an
earlier roster tier before changing production.

This assessment does not authorize a Herdr App port, a new renderer, shared
server configuration changes or production-native world output. The existing
AGENTS/native-world contract would need updating with an approved integration.

## Evidence and preservation

`export.rs` compiles against the production Storybook library with warnings
denied. Eighteen exports cover nine sizes at party sizes five and eleven.
`measurements.json` records actual actor bounds/counts and Librarian visibility.
The next-frame field is retained as raw data; static snapshots do not constitute
animation qualification. The review uses deterministic fixed time and poses.

Two generations produced identical PNG hashes. The source/configuration baseline
is unchanged; see `verification.json`. No production file, Herdr state, party,
configuration, runtime process or native asset was changed. The full application
gate was not rerun for review-only artifacts; the preceding Cleric implementation
passed 798 Rust tests / 64 runs and 28 shell tests, which is separate evidence.

Regenerate with Pillow available:

```sh
REVIEW_PYTHON=python3.13 bash docs/design/reviews/2026-11-17-hall-responsiveness/regenerate.sh
```

Local logs: `/tmp/questmancer-hall-responsiveness-20261117.log` and
`/tmp/questmancer-hall-responsiveness-repeat-20261117.log`. Existing cached Pillow
was used without an install. Regeneration writes only this review directory.

Recommendation: **needs review**. Prefer the native-fit qualification spike for
the user's full-detail goal, with the authored half-block path retained.
