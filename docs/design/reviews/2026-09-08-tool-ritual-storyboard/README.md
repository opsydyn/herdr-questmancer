# Bard, Artificer and Testmender ritual storyboard

Prepared on 2026-09-08 after approval of the Chronicle screens and the next
storyboard step. **Storyboard visually approved on 2026-09-08.** This is a design pack; these
candidate frames were approved for the subsequent bounded production batch.
The original sheets remain the design approval record.

The proposal carries the approved large-headed, short-legged direction into
three distinct tool rituals. All candidates use authored `16x24` indexed pixel
rows with a stable foot line at row 21. Existing class palettes supply material
colours; parchment and seal roles are added for small completion props.

| Class | Working movement | Held / placed spoils metaphor |
| --- | --- | --- |
| Bard | Picking hand travels across a rounded lute; the other hand holds its neck. | A small sealed song folio. |
| Artificer | Fingers adjust a brass device and its gear cluster; the wrench stays stowed beside the body. | The device wrapped and sealed. |
| Testmender | A bright needle draws into a pale patch, then rises for the next stitch. | A folded and sealed patch. |

These are visual metaphors for the existing presence states. They describe no
actual agent task, test result, inventory or verified deliverable.

## Review

- [01 — Current and proposed directions](01-directions.png): enlarged masters,
  literal 1x samples, silhouette checks, and current card/roster baselines.
- [02 — Eight moments per class](02-rituals.png): work A/B, counsel signal/wait,
  fresh spoils, settled completion, rest and unknown.
- [03 — Both room placements](03-worlds.png): production backgrounds and
  actor bounds, with candidate pixels composited at their native size.
- [04 — Proposed working playback](04-working-study.gif): two frames at
  500 ms per frame, enlarged with a literal 1x sample.

The candidates retain fixed heads and feet during work. A counsel gesture
lasts 600 ms and then waits still beneath the existing lantern. A later Herdr
working state resumes the work loop. A counsel delivery receipt alone does
not. Spoils are held initially, placed at 1000 ms and fully calm by three
seconds. Reduced/still motion uses static poses with existing shape cues.
Unknown retains its question mark; idle has no completion check. Exited
adventurers leave the scene. Newer state or connection boundaries interrupt
obsolete theatre under the existing runtime contract.

## What the evidence covers

`candidates.json` is the authored review source. `export.rs` decodes it through
production `indexed_sprite`, validates 24 frames, and uses the existing scene
renderer for room backgrounds and actor bounds. It also exports current
production sprites, card fallbacks and roster art. `layout.py` typesets those
RGB exports with nearest-neighbour enlargement; `regenerate.sh` resolves its
Rust dependencies from Cargo's build receipts rather than guessing artifacts.

Current samples use a fixed persona; proposed samples show authored material
colours before persona substitution. Saved identities and palette behaviour
are not changed by this design. The later production slice must exercise
persona recolouring, motion policies, interruptions, clipping and colour modes
through the actual asset routes and both scene painters.

Room compositions are placement studies. Their candidate sprites have not
passed through runtime ritual sampling, lighting or actor overlays as a new
production class sequence. The GIF demonstrates proposed work cadence only;
it does not validate runtime scheduling. Static counsel/completion/question
marks show the shared visual vocabulary, not new state or effects.

The three sheets and both GIF frames were inspected. These outputs establish
neither user visual approval nor native-terminal, live Herdr or release
acceptance. Current independent card fallbacks, native PNGs and roster masters
are retained and displayed for continuity review; this pack does not redesign
or approve them. No production source was edited for the storyboard.

## Validation and regeneration

The exporter compiled with warnings denied. It checked all frame sizes,
palette tokens, nonempty bodies, stable vertical and horizontal foot anchors,
distinct work frames, a distinct counsel gesture/wait, and unchanged heads
and feet across work frames. All checks passed. Regeneration produced identical
PNG/GIF hashes on a second run; see `verification.json` and `regenerate.log`.
The full application gate was not rerun for this design-only change.

Run with a Python containing Pillow (macOS fonts are used for document text):

```bash
REVIEW_PYTHON=/path/to/python3 bash docs/design/reviews/2026-09-08-tool-ritual-storyboard/regenerate.sh
```

The storyboard was subsequently approved and the bounded production batch is
implemented. See the [production review](../2026-09-08-tool-ritual-production/README.md)
for its current rendering and validation. That visual approval remains
separate. The original sheets here retain the approved design; regenerating
against a changed checkout replaces the current-sprite comparisons and requires
new review. Card and roster alignment remain explicit follow-on choices.
