# Campaign heraldry production receipt

2026-09-08: implemented and automatically verified; user visual sign-off is
explicitly deferred to the final review. The inline checkout's previous work
was preserved. No commit or release was published.

[Production review sheet](campaign-heraldry.png) contains all sixteen field/charge
combinations, a current Hall fixture, eight campaigns sharing two tables, and
the selected adventurer card in Unicode/RGB and ASCII/ANSI-16.

The world crests are authored 7x8 RGB pennants at native scale. Their four charge
shapes remain distinct in one colour. A versioned BLAKE3 mapping uses only the
workspace ID: names, party state, snapshot order and time do not change the
crest. Combinations can collide; full campaign labels remain the identity, and
nothing promises persistence beyond Herdr's workspace-key lifetime.

Each table has four separate slots below its adventurer row. An over-capacity
table omits its entire crest set. Smaller Hall compositions retain their existing
rendering. Crests add no actor targets, commands, stored fields or render wakes.
The card uses the same named field and charge in both rooms, with ASCII symbols
and indexed ANSI colours available.

## Verification

The first parchment regression failed because the crest was absent. The initial
world-position test also exposed an overlap with full-size adventurers; moving
the pennants to the clear lower strip fixed it. Focused projection, table capacity,
RGB and overlay tests now pass. Full `just verify` passes 570 Rust tests across
51 runs and 28 shell tests, formatting, denied-warning Clippy and syntax checks.
The release build and packaged-source verification pass separately.
[Machine-readable receipt](verification.json), [full gate](verify.log),
[release build](release-build.log) and [packaging](package.log) retain the evidence.

The sheet is generated from production renderers and Ratatui cells by
`export.rs` and `layout.py`; text is reconstructed with Menlo. It was inspected
for completeness and layout, not accepted on the user's behalf. It is not a
native screenshot, live Herdr completion receipt or proof of native graphics.
The small glyph before the card's colour/charge name is a text counterpart to
the world shape, not a sidebar image or embedded PNG.

The [slice plan](../../../plans/2026-09-08-campaign-heraldry.md) records scope;
[final review queue](../../../reviews/2026-09-08-final-review.md) records the
remaining visual and release gates.
