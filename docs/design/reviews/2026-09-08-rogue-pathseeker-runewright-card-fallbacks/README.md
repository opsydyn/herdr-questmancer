# Rogue, Pathseeker and Runewright card fallback alignment

The Questmancer approved the production rituals and this card alignment step
on 2026-09-08. Rogue, Pathseeker and Runewright now reuse their personalised
world sprites in the card fallback. **Card visuals were approved by the Questmancer on 2026-09-08.**
Native transport and live Herdr acceptance remain separate.

Each card uses the exact static `16x24` working master, centred inside the
existing `24x32` portrait canvas with four transparent pixels on every side.
There is no scaling, animation or separately copied sprite. This extends the
existing route to all twelve refreshed classes. Mage and Sorcerer retain their
independent portraits.

- [Before and after](01-fallback-before-after.png): previous portrait masters,
  current personalised card sprites, matching world sprites and native 1x samples.
- [Cards in context](02-cards-in-context.png): production Ratatui cards for all
  three classes in RGB/Unicode and ANSI-16/ASCII, including current keepsakes,
  labels and actions.

The implementation extends the class allowlist in
`src/scene/assets/adventurer.rs`. Existing card bounds, text, actions, native
PNG routes, roster art and the Librarian fallback are retained. Presence still
comes from Herdr and the existing card labels; the portrait is a static identity
view. Storybook remains at 43 stories.

The regression failed first on the old Rogue card, then passed with exact
pixel equality and transparent margins for all twelve refreshed classes under
two skin/hair combinations. Asset, scene, overlay, Librarian and Storybook
checks passed. Full-gate and release results are recorded in
`verification.json` and the accompanying logs. `just verify` passed **593 Rust
tests across 54 runs, 28 shell tests, formatting, Clippy with warnings denied
and shell syntax**. The release build passed. The focused gate passed 68
tests across five suites.

Both sheets were inspected. They are RGB exports and reconstructed Ratatui
buffers with Menlo text, not terminal screenshots or native transport proof.
They supersede the independent cards shown in the earlier production ritual
pack. No live agent, pane, server or plugin registration was used.

```bash
REVIEW_PYTHON=/path/to/python3 bash docs/design/reviews/2026-09-08-rogue-pathseeker-runewright-card-fallbacks/regenerate.sh
```

Use Python with Pillow. The exporter resolves libraries from Cargo build
receipts and compiles with warnings denied. Previous cards come from retained
archetype masters; current cards and world sprites use production accessors.
Production fallback rendering does not resample the image.

After card visual approval, the next proposed design-only batch is **Mage and
Sorcerer**, the final two classes without authored rituals. Establish distinct
class gestures before production implementation. That [storyboard](../2026-09-08-mage-sorcerer-storyboard/README.md) was
subsequently authorised and prepared; its visual approval is pending. This work remains uncommitted and does not extend the earlier clean
package or published-release qualification.
