# Keepsake card details — 2026-09-08

The Questmancer approved the keepsake artwork on 2026-09-08 after reviewing
the completed implementation. Native transport observations remain separate.

[Production review sheet](keepsakes.png) shows all six saved keepsakes at
native scale, enlarged for inspection, and in the actual detailed card; two
minimum-size compact cards show the text fallback in ANSI-16. This is RGB and
Ratatui cell reconstruction with Menlo, not native terminal screenshots.

Each existing assignment selects fixed copy and a distinct authored static
8x8 sprite. The detailed card remains 78x18 cells with its 24x16-cell class
portrait. The compact card retains its existing width and height, omits the
illustration, and shows the keepsake name and description. Live fact lines in both cards
truncate horizontally instead of wrapping away the controls at 60x14.
No persona generation, saved assignment, persistence schema, animation deadline,
world sprite, input or command behaviour changes.

## Verification

The first focused regression failed because the keepsake name/description was
absent. It now checks all six assignments in both rooms, both colour modes and
four sizes, including the compact and detailed thresholds. A separate buffer
comparison checks that changing the assignment affects only the reserved
keepsake section, preserving the portrait, identity, controls and surrounding
world. A long-label regression protects status and controls in both card sizes.
Asset checks require complete distinct silhouettes; existing persona
and persisted-state tests cover saved identity continuity.

`just verify` passes 574 Rust tests across 52 runs and 28 shell tests,
formatting, Clippy with warnings denied and shell syntax. The release build
and review regeneration also pass. [Verification receipt](verification.json). Visual approval and native transport acceptance remain separate.
The prior clean 0.1.9 candidate receipt applies to `c3720a9`; this slice is
subsequent local work and has not been tagged or published.

## Regenerate and inspect

Run `python3 docs/design/reviews/2026-09-08-keepsakes/regenerate.py` with Pillow
installed. The script builds the current production library with Storybook,
exports production cards and arranges the review sheet without Herdr or agents.
`just storybook` also exposes the current card render path. Reopen a source-linked
Questmancer pane to load a newly built release binary when reviewing natively.
