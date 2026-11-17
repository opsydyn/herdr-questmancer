# Storybook proportional Hall fit — 2026-11-17

- result: complete
- claim: The approved right-hand fit direction is implemented as a feature-gated Storybook prototype with checked full-room fitting, local selection/clicks, explicit fallbacks and bounded native-image lifecycle. Production remains unchanged.
- evidence: Focused regressions, full-gate log, actual fit exports, synthetic PTY receipts and native-payload pixel comparison; see `verification.json`.
- remaining uncertainty: Ghostty/Herdr visual acceptance, font-zoom geometry, visibility/reconnect behaviour and animation delivery remain unqualified. This is a static Storybook prototype, not production integration.
- next: Run the manual Storybook review at the three target sizes, then decide whether to qualify remaining native behaviours before a separate production change.

## Approved scope and decision

The owner approved the complete-room fit mockup and said “yes stroybook first”.
[ADR-0001](../../adr/0001-storybook-hall-fit.md) records the isolated experiment,
boundaries and alternatives. The existing renderer/assets remain authoritative.
Production native-world output, a Herdr App port and renderer replacement remain
outside this approval.

## Implemented behaviour

Three explicit World-category stories are added; the original Hall and Delve
stories retain their production paths. Storybook now has **69 stories**: 66
existing stories plus three experimental fit fixtures.

- **Five:** a mixed five-person party with the current Druid, Cleric, Bard and
  Sorcerer assets, all placed in the complete Hall.
- **Eleven:** a distributed two-campaign state mix that fills the canonical
  stations and overflow without losing any actor.
- **Crowded:** eleven workers at one campaign station. The prototype rejects
  its incomplete canonical placement and displays a diagnostic responsive fallback.

`src/storybook/fit.rs` projects the existing RGB engine at 160×90. The same pure
transform chooses an integer physical-pixel scale, cell-aligned image bounds,
padding and cell-centre hit mapping. No source pixel is downsampled. Every actor
and the Librarian must remain addressable through the actual target resolver.
Invalid geometry, too-small panes, image-budget overflow or incomplete station
placement refuse native fit. Image size is capped below four million pixels.

`fit_preview.rs` uses the installed ratatui-image Kitty implementation with two
locally allocated image identities. Unchanged images are cached; resize and
selection re-encode as needed. Replacement, overlay/help/story transitions and
shutdown delete only the preview's own identities. Cleanup I/O stays in the
Storybook runtime. Card and Ledger image identities are never targeted.

The runtime shares one startup capability/geometry query with native portraits.
It does not infer pane support from an outer terminal name or force graphics.
Unsupported/error results show an explicit half-block fallback. Tmux is excluded
from this experiment. The fallback uses existing responsive layouts, including
their roster/vignette limits; it does not claim every actor is visible at every
size. The underlying production station-capacity issue is not silently changed.

Selection/attention text stays at normal terminal font size below the image.
Tab/Shift-Tab cycle fixed fixture actors; clicks use the fit transform. `c` opens
the selected card, `L` or a Librarian click opens the Ledger, and its existing
page keys work. All interaction is local Model presentation state; no
`AgentCommand`, Herdr client, persistence or backend execution is involved.
Parchment overlays temporarily use the existing backdrop; closing them restores
the fit. Native world-behind-parchment stacking is not claimed by this experiment.

## Review artifacts

- [Actual prototype fit projections](01-storybook-fit-review.png): five and
  eleven actors at all three target sizes. The world image is actual output;
  caption/footer text is reconstructed for the sheet, not a terminal screenshot.
- [First native image upload decoded from the executable](02-pty-world-upload.png).
  Every RGBA pixel matches the corresponding fit projection.
- `renders.json` and six `fit-*.png` files: exact image dimensions, scales and
  actor/Librarian counts from the prototype.
- `transport-pixels.json`: native payload round-trip result.

The rendered room reserves one header and two text rows. With fixture geometry
8×16 physical pixels per cell, its display scales are 5×, 4× and 3× at 120×34,
96×28 and 80×24 respectively. The original mockup had no text-row budget; this
implementation makes that reservation explicit while preserving the whole room.

## Engineering evidence

The first new story regression failed because `world.guild-fit` did not exist.
The later Ledger regression reproduced `j` escaping the overlay instead of
turning a page. Both pass with the final implementation. Other tests cover exact
pixel survival, literal fit geometry, all-actor/Librarian clickability, resize
offsets, local mouse/card selection, immutable base fixtures, crowded refusal,
graphics-off fallback, static cache reuse and owned-image cleanup.

Fresh `just verify` passes **807 Rust tests / 65 runs**, **28 shell tests**,
formatting, denied-warning Clippy and shell syntax. The Storybook binary builds.
The normal release build passes, and the production binary retains SHA-256
`a08b70bb57dd3daeb7cd7d34044030656caaa13510cdefaa7b3cc02995dff886`, identical
to the Cleric candidate. The final full-gate log is
`/tmp/questmancer-storybook-fit-verify-complete-20261117.log`.

Initial compile/lint attempts caught a nonexistent attention helper, oversized
functions and small style issues. They were corrected without relaxing the gate.
Earlier partial gate outputs are retained separately from the final log.

## Synthetic PTY qualification

`pty_probe.py` runs the actual binary with explicit synthetic Kitty, geometry,
status and device-attribute replies, with inherited Herdr and terminal hints
removed. It reconstructs cursor-addressed Unicode cells rather than stripping
escape sequences. No real terminal, Herdr server or live pane is controlled.

The native scenario checks startup upload, static suppression, selection,
96×28 resize, an SGR mouse click opening the Librarian's Ledger, Ledger page
navigation, cards, help, eleven/crowded stories and normal exit. Cleanup targets
at most two identities and excludes both native overlay identities. The
graphics-off scenario emits zero images. Both restore raw-mode flags, alternate
screen and mouse capture and exit zero. `pty-receipt.json` records exact packets,
counts, transcript hashes and scope.

Early harness attempts replied to the terminal's final status query before its
capability replies, causing a correct fallback. Inspection of the installed
Picker parser identified its status delimiter; the final harness preserves
that ordering. These attempts did not justify forcing a protocol or weakening
the application's detection. Raw transcripts remain under
`/private/tmp/questmancer-storybook-fit-20261117/`.

`verify_transport.py` decodes the first emitted image, checks its RGBA length,
and compares every byte with the exact FitScene export. This proves producer
encoding, not actual terminal placement or appearance.

## Manual handoff and limits

```sh
cargo run --features storybook --bin questmancer-storybook -- --story world.guild-fit --inspect
```

Follow [the manual review](../../manual-test/storybook-hall-fit.md). Normal window
resize uses startup cell geometry; restart after font-size changes. Font zoom,
visibility/reconnect, full-motion upload performance and native visual/click
acceptance remain unperformed. No animation timer is introduced: these stories
use fixed time and Motion::None and wake only on input/resize.

The live synthetic party and its UI were not operated. No commit, installation,
shared-server change or publication occurred. Production integration requires
a separate decision after this prototype's native review.

## Regeneration

With Pillow available to the selected Python:

```sh
REVIEW_PYTHON=python3.13 bash docs/reviews/2026-11-17-storybook-hall-fit/regenerate.sh
```

This compiles the export against the current Storybook library and writes this
review's PNGs/receipt only. `verify_transport.py` additionally needs the saved
native PTY transcript. The local run used existing cached Pillow without an
installation. Historical approved mockup/art files remain unchanged.

Recommendation: **needs review** in the actual terminal before production work.

## Native handoff prepared

A subsequent “lets go next up” promoted the manual native review setup.
`native-handoff.json` records a fresh Herdr 0.9.5/protocol-22 baseline, the
current full gate (807 Rust tests / 65 runs and 28 shell tests), and an isolated
PTY rerun against the exact pinned review executable. A separate **Hall fit
Storybook** tab was created without changing focus. Only its new shell received
the launcher command. The launcher is waiting for the human to make the tab
visible and press Return, so capability detection runs after attachment.

All prior panes, the four synthetic identities/statuses and the party's owned
work were preserved; plugin logs are empty. The new pane is not an adventurer.
The existing dogfood party remains open. The run-specific resource ledger and
pinned executable are under `/private/tmp/qm-storybook-native-eap_t_3e/`.
Native visual acceptance is still unreviewed; the prepared tab is intentionally
retained. Revalidate its recorded terminal/process ownership before later
control or cleanup.

The user subsequently replied “approved”. See [the approval record](user-approval.md)
for its exact wording and evidence limits. This approves the fitted prototype
direction; individual native checks were not enumerated. Clarification of
production integration scope was requested before source changes.
