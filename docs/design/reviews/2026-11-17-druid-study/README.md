# Druid silhouette and material study — 2026-11-17

- result: complete
- claim: Two review-only Druid directions are authored beside the current world master, with silhouette, card-reference and Hall/Delve placement comparisons. Production activation awaits a visual choice.
- evidence: The three sheets, `candidates.json`, `export.rs` and `verification.json`; deterministic regeneration, indexed decode and foot/bounds checks.
- remaining uncertainty: Persona variation, all semantic poses, animation, roster continuity and native appearance of the candidates are not qualified. These are static placement studies, not activated production rituals.
- next: Choose or revise A/B, then implement only the approved Druid direction through the existing ritual and persona paths.

## Directions

- **A / Grove Keeper:** front-facing eyes, rounded leaf hood, fuller beard, warm bark staff on the left and a soft cloak edge. The staff side follows the current detailed card illustration.
- **B / Bramble Sage:** turned face, asymmetric leaf cloak, compact pouch and right-hand living staff. This is the stronger pose study; it retains the world master's existing staff side.

Both keep the `16x24` canvas, feet on row 21 and production foot columns
`4,5,6,9,10,11`. The pale skin base/shadow/highlight matches the fixed current
review persona. Other candidate materials are authored study colours, not an
implemented persona substitution contract. The existing native card is shown
only as an identity reference; it is unchanged.

## Review sheets

1. [Current and two directions](01-druid-directions.png): 10x nearest-neighbour inspection, literal 1x pixels, silhouette and existing card reference.
2. [Room crops](02-druid-room-crops.png): each `16x24` master placed at the same actor position on identical production Hall and Delve backgrounds; the whole crop is enlarged evenly by 8x.
3. [Full rooms](03-druid-full-rooms.png): canonical `160x90` rooms shown at 3x, for actor-to-world proportion.

The exporter links the current Storybook library, decodes candidates through
production `indexed_sprite`, obtains real room backgrounds and actor bounds,
then composites each master after room lighting. All three comparisons use the
same compositing method. It passes the result through the production Ratatui
half-block adapter and checks lossless RGB reconstruction. Pixels outside the
actor footprint are identical between options. This demonstrates placement,
shape and contrast, not candidate lighting integration or runtime acceptance.

The current comparison is the production **master** at the fixed fixture's
working pose. It is not a screenshot of the retained live party. The latter was
not operated during this design slice.

## Verification and preservation

`verification.json` records exact image hashes and baseline preservation.
The review exporter compiles with warnings denied and validates dimensions,
palette decoding, nonempty art, stable feet and distinct options. Two final
regenerations produced identical PNG hashes. Formatting, script syntax and
`git diff --check` pass. The build is the existing Storybook library; no
dependency or production source change is included.

The source baseline covers `src`, `tests`, `herdr`, Cargo manifests/lock,
plugin manifest and justfile. All baseline bytes remain unchanged. Earlier
uncommitted authoring/compatibility work and review packs were preserved.
The complete application gate was not rerun for review-only files; the
preceding dogfood receipt records the fresh 789-Rust/28-shell passing gate.

During setup, the default Python lacked Pillow; the cached Pillow 13 wheel
was for Python 3.13, so the exporter used the installed Python 3.13 and existing
cache without an install. An initial Pillow deprecation warning in the layout
script was removed. Final build/export/layout logs are
`/tmp/questmancer-druid-study-20261117.log` and
`/tmp/questmancer-druid-study-repeat-20261117.log`.

## Regeneration

Run from this checkout with a Python containing Pillow:

```bash
REVIEW_PYTHON=/path/to/python3 bash docs/design/reviews/2026-11-17-druid-study/regenerate.sh
```

On the current host, the existing cached dependency can be used without installation:

```bash
PYTHONPATH=/Users/alancurrie/.cache/uv/archive-v0/18AOTubXh2D76Zrz \
REVIEW_PYTHON=/opt/homebrew/bin/python3.13 \
bash docs/design/reviews/2026-11-17-druid-study/regenerate.sh
```

`rendered.json` is the review export, not a new renderer or asset registry.
Regeneration against changed production source changes its comparison baseline
and requires fresh inspection and verification hashes. No Herdr process,
plugin action, live state, native image route, pose timing or saved persona is
changed by this study. No commit, installation or release was performed.

Recommendation: **needs review**.
