# Librarian proportion refresh

Date: 2026-09-05. Status: implemented in the current checkout; visual approval
pending. The Questmancer authorised this slice and continuation into the
three-class pilot with “Yes librarian then move on”.

The Librarian keeps orange orangutan fur, low gold spectacles, a purple robe
and books. The world silhouette has a broader face and shorter body, with
feet on row 21 of the unchanged `16x24` canvas. The Ledger's `24x32` fallback
is now independently authored, with more face and book detail. The native
illustration remains unchanged.

- [Proportions and colour modes](01-librarian-proportions.png): before/after
  masters, native reference, literal 1x and enlarged views on three backgrounds,
  including production ANSI-16 conversion.
- [Hall and Ledger context](02-librarian-context.png): canonical and compact
  Hall pixels, Librarian crops, and the production Ledger overlay in truecolour
  and ANSI-16.

These are production RGB and Ratatui buffer exports. Menlo text is reconstructed
for the document; the sheets are not terminal screenshots or live Herdr
acceptance. Visual inspection of the sheets does not establish native graphics
transport or manual click acceptance.

## Scope and evidence

Production changes are confined to `src/scene/assets/librarian.rs`. Both
existing accessors remain static and return authored sprites. No NPC animation,
new persisted state, altered station or help interaction was added.

The focused grounding/portrait test first failed because the original world
sprite ended on row 22. It now checks row 21 and that the Ledger uses more than
the world sprite's sixteen columns. All 64 focused Librarian, Hall, overlay and
Storybook tests passed. Existing tests cover complete Librarian click bounds,
separation from agents, omission in smaller tiers and responsive help access.
The full `just verify` gate passed with 543 Rust tests across 49 suites,
26 shell tests, formatting, strict Clippy and script syntax before the pilot
changes. The pilot has a separate later full gate in the active party plan.

The before reference in `baseline.json` records the previous world glyphs and
palette, with its original padded Ledger derived by `export.rs`. Current art
always comes from the production asset accessors. The exporter uses the same
Hall painters, half-block adapter and Ledger overlay as the application.

## Regeneration

From the repository root, with Pillow installed in the selected Python:

```bash
REVIEW_PYTHON=python3 bash docs/design/reviews/2026-09-05-librarian/regenerate.sh
```

The script builds the current library with Storybook enabled, compiles the
review exporter against those exact Cargo artifacts, and typesets two sheets.
Its temporary files are removed on exit. Regeneration reflects the current
checkout; preserve approved sheets before changing the review baseline.

Review the face, spectacles, book shapes, robe length and grounding at 1x and
in the room. Confirm the Ledger's portrait and text remain legible together.
The subsequent implementation is the authorised
[three-class pilot](../../../plans/2026-09-05-party-delight.md#3-build-and-review-the-three-class-pilot).
