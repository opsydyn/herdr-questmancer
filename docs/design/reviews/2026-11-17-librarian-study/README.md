# Librarian world-art study — 2026-11-17

- result: complete
- claim: Two static `16x24` Librarian directions are ready beside the current sprite, with canonical and compact Hall placement studies. Production assets are unchanged.
- evidence: `candidates.json`, two inspected review sheets, the production-linked exporter and `verification.json`.
- remaining uncertainty: No new direction is selected or activated. The chosen direction still needs an independently authored `24x32` Ledger fallback and production/native review. Existing native art and help behavior are retained.
- next: Choose or revise A/B, then implement the selected Librarian world and fallback art through the existing static accessors.

## Directions

- **A / Hearth Archivist:** welcoming front view, broad orangutan muzzle, low gold spectacles, rust fur and short folded purple robe.
- **B / Keeper of the Stacks:** gently turned head and asymmetric shoulders, a more prominent three-book stack and a tucked volume.

Both retain the orangutan identity, green waistcoat, gold trim, book colours
and native `16x24` footprint. Feet remain on row 21, in the original columns
`3,4,5,6,8,9,10,11`. No animation, actor role, help action, station, hit region,
persona or persistence change is proposed. The Librarian remains a fixed help
NPC and is not an adventurer.

The user's supplied screenshot shows the existing detailed Ledger illustration
visibly rendering. That illustration remains the identity reference and is
unchanged. This study concerns the world sprite. The larger fallback remains
independently authored; it will follow the chosen direction rather than being
a stretched or padded world sprite.

## Review

- [Current and two directions](01-librarian-directions.png): 10x nearest-neighbour inspection, literal 1x samples, silhouettes and native illustration reference.
- [Canonical and compact Hall](02-librarian-hall-context.png): both candidates at the exact existing NPC position and canvas size, with evenly enlarged context crops.

There is no Librarian placement in the Delve. Smaller Hall tiers continue to
omit the NPC while retaining `?` access; this study changes none of those rules.

## Method and evidence limits

`export.rs` decodes both authored matrices with production `indexed_sprite`,
validates dimensions and grounding, obtains actual Hall interactable bounds,
and checks lossless RGB half-block conversion.

The original production library exports the current sprite and rooms. For
uncontaminated candidate backgrounds, `regenerate.py` makes its own disposable
source copy and replaces only the copied world matrix with transparent pixels.
The copied package has a distinct crate identity. Its unchanged renderer gives
the same bounds and background without the old NPC, so transparent candidate
edges cannot retain old sprite pixels. Both candidates are blitted at those
bounds and exported through the production adapter. Pixels outside the NPC
footprint are checked unchanged. The temporary source copy is removed by its
owner on exit; no live Herdr resource is used.

An initial harness reused the original package name and shared Cargo target,
which allowed its empty-background library to be mistaken for the current
build on repeat export. That invalid comparison was rejected. The harness now
uses a distinct package name and asserts that the current sprite is populated
while the background-only sprite is entirely transparent. The original dev
library was recompiled after invalidating the Librarian source timestamp;
source bytes and the release binary remain unchanged. Final repeat exports
and the existing Librarian asset tests pass. Failed interim previews are not
approval evidence.

These are review-only placement studies, not activated production assets or
native screenshots. Existing `24x32` fallback and native illustration are
exported/read only; no replacement is installed.

## Qualification and preservation

`verification.json` pins output hashes, deterministic regeneration, production
source preservation, bounds checks and current/background separation. Review
exporters compile with warnings denied. Existing Librarian asset tests pass;
Rust review-source formatting and `git diff --check` pass. No full application
gate was rerun for this study; the preceding Druid receipt records the passing
791-Rust/28-shell full gate.

All baseline source/test/script/manifest bytes are unchanged, including the
approved Druid implementation, current Librarian assets and native PNG.
The live party was not operated. No production mutation, commit, relink,
installation or release occurred.

Logs are `/tmp/questmancer-librarian-study-20261117.log`,
`/tmp/questmancer-librarian-study-repeat-20261117.log` and
`/tmp/questmancer-librarian-study-assets-20261117.log`.

## Regeneration

Run from the current checkout with a Pillow-equipped Python:

```bash
python3 docs/design/reviews/2026-11-17-librarian-study/regenerate.py
```

The current host can reuse its existing cached Pillow wheel:

```bash
PYTHONPATH=/Users/alancurrie/.cache/uv/archive-v0/18AOTubXh2D76Zrz \
python3.13 docs/design/reviews/2026-11-17-librarian-study/regenerate.py
```

The production build uses its locked dependencies. The disposable renamed
package resolves its copied lock offline. Regeneration against changed source
updates the current comparison and requires fresh inspection and hashes.

Recommendation: **needs review**.
