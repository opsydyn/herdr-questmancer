# Cleric identity study — 2026-11-17

- result: complete
- claim: Two review-only Cleric directions are authored beside the current resting master, with silhouette, native-card reference and Hall/Delve placement comparisons.
- evidence: Three review sheets, `candidates.json`, `export.rs` and `verification.json`; production indexed decoding, fixed foot anchors, lossless half-block conversion and deterministic regeneration.
- remaining uncertainty: The intended direction, persona variation, all poses, animation and native appearance remain unreviewed. These are static placements, not activated production rituals.
- next: Select or revise A/B, then integrate only the approved Cleric direction through the existing persona, ritual and fallback paths.

The user supplied a Delve screenshot and said the adventurer could use a
redesign. The selected card identifies **Codex / Halfling Cleric**. This study
therefore addresses that Cleric's world sprite, rather than the nearby Druid
or Sorcerer. The selected Cleric is an actual agent; this study does not send
it input or alter its persona, status or identity.

## Directions

- **A / Sunward Chaplain:** open face, warm beard, silver shoulders, cream
  tabard over blue cloth, a sun mace and a gold-bound prayerbook. Recommended
  for closer continuity with the existing native card and a clearer face.
- **B / Hearth Vicar:** rounded pale cowl, soft beard, longer cream stole,
  shorter sun mace and a retained prayerbook. A gentler robed direction.

Both retain the `16x24` world canvas, row-21 feet and the current Cleric's foot
columns `5,6,7,10,11,12`. Equipment remains in the static quiet study rather
than disappearing with work activity. The mace, book, face, fabric and armour
are exact authored pixels. Candidate colours are study colours; saved-persona
substitution is an implementation obligation if a direction is approved.

## Review sheets

1. [Current and two directions](01-cleric-directions.png): 10× inspection,
   silhouettes, literal 1× pixels and the existing native-card reference.
2. [Hall and Delve crops](02-cleric-room-crops.png): matching actor positions
   and identical production room backgrounds.
3. [Whole-room scale](03-cleric-full-rooms.png): canonical rooms at 3× scale.

The current comparison is the production **resting** master at fixed Porcelain
skin and Chestnut hair, not a screenshot of the user's saved persona. All
options are placed at the same fixture Working station for a controlled
composition comparison. The exporter obtains production room backgrounds and
actor bounds, composites each master after room lighting, then passes the RGB
through the actual Ratatui half-block adapter and checks lossless reconstruction.
Pixels outside the actor bounds are identical between directions. These sheets
show placement and contrast, not candidate lighting or live integration.

The native card is an existing reference asset and is unchanged. No generated
illustration or new native portrait is included. No live Herdr operation,
agent command, party refresh or cleanup was performed.

## Verification and preservation

The exporter compiled with warnings denied. Palette decoding, non-empty frames,
dimensions, distinct candidates and stable feet passed. Two final regenerations
produced identical PNG hashes. Source/configuration baseline bytes remain
unchanged; see `verification.json` for count and hashes. Exporter formatting,
shell syntax and `git diff --check` passed. All existing dirty work and earlier
review images were preserved.

The full application gate was not rerun for these review-only files. The
preceding Sorcerer implementation passed 796 Rust tests / 64 runs and 28 shell
tests; that is separate historical evidence.

## Regeneration

With Pillow available to the chosen Python:

```sh
REVIEW_PYTHON=python3.13 bash docs/design/reviews/2026-11-17-cleric-study/regenerate.sh
```

The local run used the existing cached Pillow wheel through `PYTHONPATH`, with
no installation. Logs are `/tmp/questmancer-cleric-study-20261117.log` and
`/tmp/questmancer-cleric-study-repeat-20261117.log`. Regeneration rebuilds this
review against the current checkout; it changes no production asset or live
state. Do not regenerate earlier approved studies in place.

Recommendation: **needs review**. A is the recommended direction.
