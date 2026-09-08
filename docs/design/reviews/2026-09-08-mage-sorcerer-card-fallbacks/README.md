# Mage and Sorcerer card fallback alignment

The Questmancer approved the production rituals and this final card alignment
step on 2026-09-08. Mage and Sorcerer now reuse their personalised world
sprites in the card fallback. **Card visuals were approved by the Questmancer on 2026-09-08.** Native
transport and live Herdr acceptance remain separate.

Every class now uses its exact static `16x24` working master, centred inside
the existing `24x32` portrait canvas with four transparent pixels on every
side. There is no scaling, animation or separately copied sprite. This
completes room/card identity continuity for all fourteen classes.

- [Before and after](01-fallback-before-after.png): previous portrait masters,
  current personalised card sprites, matching world sprites and native 1x samples.
- [Cards in context](02-cards-in-context.png): actual production Ratatui cards
  for both classes in RGB/Unicode and ANSI-16/ASCII, including current keepsakes,
  labels and actions.

The implementation removes the remaining class split in
`src/scene/assets/adventurer.rs`: every class now uses the same centring path.
Existing card bounds, text, actions, native PNG routes, roster art and the
Librarian fallback are retained. Presence still comes from Herdr and the
existing card labels; the portrait is a static identity view. Storybook remains
at 45 stories.

The regression failed first on the old Mage card. It now checks the complete
`AdventurerClass::ALL` list for exact world-pixel equality and transparent
margins under two skin/hair combinations. Asset, scene, overlay, Librarian
and Storybook checks passed. See `verification.json` and the accompanying
logs for full verification and release-build results. `just verify` passed
**593 Rust tests across 54 runs and 28 shell tests**, with formatting, Clippy
and shell syntax green. The release build passed. The focused gate passed
68 tests across five suites.

Both sheets were inspected. They are RGB exports and reconstructed Ratatui
buffers with Menlo text, not terminal screenshots or native transport proof.
They supersede the independent cards shown in the earlier production ritual
pack. No live agent, pane, server or plugin registration was used.

```bash
REVIEW_PYTHON=/path/to/python3 bash docs/design/reviews/2026-09-08-mage-sorcerer-card-fallbacks/regenerate.sh
```

Use Python with Pillow. The exporter resolves libraries from Cargo build
receipts and compiles with warnings denied. Previous cards come from retained
archetype masters; current cards and world sprites use production accessors.
Production fallback rendering does not resample the image.

After final card approval, the next proposed slice is to prepare the complete
release candidate for review and qualification: reconcile the approval record,
review the accumulated changes, and identify the remaining clean-candidate
and live acceptance checks. That preparation was subsequently authorised and is recorded in
[the complete candidate review](../../../reviews/2026-09-08-party-candidate/README.md). This implementation
remains uncommitted and does not extend the earlier clean package or
published-release qualification. Publication is a separate gate.
