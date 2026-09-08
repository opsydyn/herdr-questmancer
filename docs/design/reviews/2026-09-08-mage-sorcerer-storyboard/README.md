# Mage and Sorcerer ritual storyboard

Prepared on 2026-09-08 after the Questmancer approved the Rogue, Pathseeker
and Runewright card fallbacks and authorised the final design-only batch.
**Storyboard visually approved by the Questmancer on 2026-09-08.** This pack changes no production
asset routes. Storybook remains at 43 stories and twelve ritual classes.

The proposed masters extend the approved large-headed, short-legged direction.
Each class has eight authored `16x24` indexed frames with planted feet at row
21. Existing class palettes supply material colours; parchment and seal roles
support the completion props.

| Class | Working gesture | Held / placed spoils metaphor |
| --- | --- | --- |
| Mage | Tends a small silver censer; its green ember settles lower on the second beat. The skull staff stays planted beside the deep violet hood. | A sealed reagent case. |
| Sorcerer | Turns a silver focus around a warm spark between cupped hands. The gold halo stays separate above the head. | The focus wrapped and sealed. |

These are metaphors for presence states. They do not identify the agent's
actual coding task or claim verified tests, inventory, deliverables or outcomes.
The skull staff and separate halo distinguish these classes from the Wizard's
hat and book and the other twelve class silhouettes.

## Review

- [01 — Current and proposed directions](01-directions.png): enlarged masters,
  native 1x samples, silhouettes and current card/roster baselines.
- [02 — Eight moments per class](02-rituals.png): work A/B, counsel signal/wait,
  held spoils, settled completion, rest and unknown.
- [03 — Guild Hall and Delve placements](03-worlds.png): actual production
  backgrounds and actor bounds with native-size candidate compositing.
- [04 — Proposed working playback](04-working-study.gif): two frames at
  500 ms per frame, with enlarged and native 1x samples.

Faces and feet stay fixed during work. The proposed counsel signal lasts
600 ms, then holds still beneath the shared lantern. A newer Herdr working
state resumes the work loop; a counsel delivery receipt alone does not.
Spoils are held initially, placed at 1000 ms and calm by three seconds.
Reduced/still motion uses static poses. Unknown keeps its question mark;
resting does not claim completion. Exited adventurers leave the scene. Existing
state and connection boundaries must interrupt obsolete theatre in implementation.

## Evidence and limits

`candidates.json` contains the authored rows. `export.rs` uses production
`indexed_sprite` to decode and validate all sixteen frames, and exports current
world sprites, card fallbacks and rosters. Existing scene fixtures supply the
room backgrounds and actor bounds. `layout.py` typesets the RGB exports with
nearest-neighbour enlargement. `regenerate.sh` resolves Rust dependencies
from Cargo build receipts and compiles the exporter with warnings denied.

Current samples use fixed personas: Rose skin, Vestments, chestnut hair for
Mage and gold for Sorcerer. Candidates show authored material colours before
persona substitution. Saved identities and production palette behaviour are
unchanged. Independent card fallbacks, native illustrations and roster art
are retained for continuity review.

Room panels are placement studies. Candidate pixels are composited over the
production backgrounds; they are not runtime ritual samples with production
lighting and overlays. The GIF illustrates proposed cadence and does not
validate scheduling. Shared static state marks show the existing visual
vocabulary. This pack establishes no native-terminal, live Herdr or release
acceptance.

The three sheets and both GIF frames were inspected. Export checks passed
for palette tokens, dimensions, nonempty bodies, stable vertical and horizontal
foot anchors, distinct working frames, distinct counsel signal/wait, and
unchanged work heads and feet. Repeat regeneration produced identical PNG/GIF
hashes; see `verification.json` and `regenerate.log`. The full application gate
was not rerun for this design-only change. The preceding card slice's
593 Rust tests / 54 runs, 28 shell tests and release build remain separate
receipts, and its production sources were checked for changes.

```bash
REVIEW_PYTHON=/path/to/python3 bash docs/design/reviews/2026-09-08-mage-sorcerer-storyboard/regenerate.sh
```

Use Python with Pillow; document text uses macOS fonts. Regeneration against
a changed checkout replaces current-production comparisons and requires
another inspection. After storyboard approval, the next bounded step is
implementation through existing persona and ritual routes, with test-first
checks, full verification and a separate production visual review. Card
alignment remains a later slice.

The approved frames are now implemented. See the [production review](../2026-09-08-mage-sorcerer-production/README.md)
for current routes, verification and the separate production visual gate.
Original storyboard images remain the design approval record.
