# Cleric, Paladin and Druid ritual storyboard

Prepared on 2026-09-08 after the Questmancer approved the Bard, Artificer and
Testmender card fallbacks and authorised the next design-only batch.
**Storyboard visually approved by the Questmancer on 2026-09-08.** No production routes are changed
by this pack.

The proposed masters extend the approved large-headed, short-legged direction.
Each class has eight authored `16x24` indexed frames, planted at row 21.
Class palettes provide the material colours, with parchment and seal roles
for completion props. Their retained gear keeps the three silhouettes distinct.

| Class | Working gesture | Held / placed spoils metaphor |
| --- | --- | --- |
| Cleric | Traces an open book with one finger; the gem staff rests alongside. | The book closed and sealed. |
| Paladin | Checks the shield's leather strap; the halberd rests alongside. | The shield wrapped and sealed. |
| Druid | Gently bends the living staff's leafy sprig with one hand. | A small sealed seed pouch. |

These are presence-state metaphors. They do not identify an agent's actual
task or claim verified contents, tests, inventory or successful outcomes.

## Review

- [01 — Current and proposed directions](01-directions.png): enlarged masters,
  native 1x samples, silhouettes and current card/roster baselines.
- [02 — Eight moments per class](02-rituals.png): work A/B, counsel signal/wait,
  held spoils, settled completion, rest and unknown.
- [03 — Guild Hall and Delve placements](03-worlds.png): actual production
  backgrounds and actor bounds, with native-size candidate compositing.
- [04 — Proposed working playback](04-working-study.gif): two frames at
  500 ms per frame, with enlarged and native 1x samples.

Work keeps faces and feet fixed. The proposed counsel signal lasts 600 ms,
then holds still beneath the existing lantern. Herdr must report working to
resume the work loop; a counsel delivery receipt does not resume it. Spoils
are held initially, placed at 1000 ms and calm by three seconds. Reduced/still
motion uses static poses. Unknown keeps its question mark; resting does not
claim completion. Exited adventurers leave the scene. Existing state and
connection boundaries must interrupt obsolete theatre in implementation.

## Evidence and limits

`candidates.json` is the authored source. `export.rs` decodes it through
production `indexed_sprite`, validates all 24 frames, and exports current
world sprites, card fallbacks and rosters. Its scene fixtures provide the
actual room backgrounds and actor bounds. `layout.py` typesets these RGB
exports using nearest-neighbour enlargement. `regenerate.sh` resolves Rust
dependencies from Cargo build receipts and compiles the exporter with
warnings denied.

Current samples use fixed personas: Rose skin, Vestments, silver hair for
Cleric/Druid and chestnut for Paladin. Candidates show authored material
colours before persona substitution. Existing saved identities and production
palette substitution are unchanged. The current independent card fallbacks,
native illustrations and roster art are retained for continuity review.

Room panels are placement studies: the candidates have not been sampled,
lit or overlaid as production ritual sequences. GIF timing illustrates the
proposal and does not validate runtime scheduling. Shared static state marks
show the existing visual vocabulary. This pack establishes no native-terminal,
live Herdr or release acceptance.

The three sheets and both GIF frames were inspected. Export checks passed
for palette tokens, frame dimensions, nonempty bodies, stable vertical and
horizontal foot anchors, distinct working frames, distinct counsel signal/wait,
and unchanged heads and feet across work frames. A second regeneration
produced identical image hashes. See `verification.json` and `regenerate.log`.
The application gate was not rerun for this design-only change; prior
production verification remains recorded separately.

```bash
REVIEW_PYTHON=/path/to/python3 bash docs/design/reviews/2026-09-08-cleric-paladin-druid-storyboard/regenerate.sh
```

Use a Python with Pillow; document text uses macOS fonts. Regeneration against
a changed checkout updates current-production comparisons and requires a new
inspection. After storyboard approval, the next bounded step is production
implementation through the existing persona and ritual paths, with focused
test-first checks, full verification and a separate production visual review.

The approved frames are now implemented. See the [production review](../2026-09-08-cleric-paladin-druid-production/README.md)
for the current routes, verification and separate production visual gate.
The original storyboard images remain the design approval record.
