# Tool-class card fallback alignment

The Questmancer approved the tool ritual production visuals and this card
alignment step on 2026-09-08. Bard, Artificer and Testmender now reuse their
personalised world sprites in the card fallback. **Card visuals were approved by the Questmancer on 2026-09-08.** Native-terminal and live Herdr acceptance remain separate.

Each card uses the exact static `16x24` working master, centred inside the
existing `24x32` portrait canvas with four transparent pixels on every side.
There is no scaling, animation or separate copy of the sprite. This extends
the existing Wizard/Ranger/Barbarian route to the three newly refreshed classes.

- [Before and after](01-fallback-before-after.png): retained old portrait masters,
  new fallbacks, matching world sprites and literal 1x samples.
- [Cards in context](02-cards-in-context.png): actual production Ratatui cards
  for all three classes in truecolour/Unicode and ANSI-16/ASCII, including
  current keepsakes, labels and actions.

The implementation only extends the class allowlist in
`src/scene/assets/adventurer.rs`. Card bounds, text, actions, native PNG routes,
roster art and the independent Librarian fallback are retained. The portrait
is a static identity view even when the adventurer is blocked or resting;
presence still comes from Herdr and the existing card labels.

The regression failed first on the old Bard card, then passed with exact
pixel equality and transparent margins for all six refreshed classes under
two skin/hair combinations. The former tool-batch test requiring independent
cards was superseded by this explicitly approved continuity contract. Asset,
scene, overlay, Librarian and Storybook checks passed. Final full verification
and release-build results are recorded in `verification.json` with their logs.
`just verify` passed **593 Rust tests across 54 runs, 28 shell tests, formatting,
Clippy with warnings denied and shell syntax**. The release build passed.
The focused gate passed 68 tests across five suites.

Both sheets were inspected. They are RGB exports and reconstructed Ratatui
buffers with Menlo text, not terminal screenshots or native image transport
proof. They supersede the older independent cards shown in the tool ritual
production pack. No live agent, pane, server or plugin registration was used.

To regenerate with Pillow available:

```bash
REVIEW_PYTHON=/path/to/python3 bash docs/design/reviews/2026-09-08-tool-card-fallbacks/regenerate.sh
```

The exporter resolves its libraries from Cargo build receipts and compiles
with warnings denied. Previous cards come from retained archetype masters;
current cards and world sprites use the production accessors. No image
resampling is used in the production fallback.

After card visual approval, the next proposed art batch is a **Cleric, Paladin
and Druid storyboard**: explore distinct book, shield and living-staff gestures
before production implementation. That design step has not started. This
work remains uncommitted and does not extend the earlier clean package or
published-release qualification.
