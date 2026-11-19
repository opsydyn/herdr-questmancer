# Stock Herdr dev boot in Ghostty

The [normal-action review](../2026-11-19-attached-kitty-open/README.md)
records the subsequent fix for opening after client attachment.

- **Result** · Complete. The launcher passed and the Hall, Delve and native portrait were verified in Ghostty on stock Herdr.
- **Claim** · `just dev-ghostty` starts the source-linked Questmancer plugin on stock Herdr without a manual resize.
- **Evidence** · A failing ordinary launch, two fresh boots, a reattach, and the [Hall](ghostty-hall.png) and [Delve with native portrait](ghostty-delve-portrait.png) captures. See the [receipt](receipt.json).
- **Remaining uncertainty** · Ordinary new tabs after attachment remain subject to Herdr issue #5594.
- **Next** · Use `just dev-ghostty` to repeat the isolated development boot.

## What changed

`just dev-ghostty` builds the current checkout and starts an isolated stock
Herdr server. It invokes the ordinary `opsydyn.questmancer.open` action before
attaching the terminal. Once the action has created the tab, the launcher
attaches Ghostty. The managed pane waits up to five seconds for measured PTY
pixels before launching Questmancer. If geometry is unavailable, Questmancer
starts on its supported fallback. See the [dev boot instructions](../../troubleshooting/native-portrait-rendering.md#repeatable-ghostty-dev-boot).

The existing server reported valid 8×18 client cells when the ordinary `open`
action was invoked after attachment. The new tab still started with zero
pixel dimensions and produced 5,310 half-blocks and zero Kitty placements.
This reproduced the failure before the launcher change. [Herdr issue #5594](https://github.com/herdrdev/herdr/issues/5594)

## Verification

| Gate | Evidence |
| --- | --- |
| Repeatability | Two independent `just dev-ghostty` boots on stock Herdr 0.9.5 used real Herdr clients in test PTYs at 155×45 and 124×39. Hall, Delve and the Ledger used Kitty without a resize in both runs. |
| Reopen and reattach | A second `open` action reused the same pane and process in each run. Reattaching to the first session also reused the pane. |
| Startup guards | The launcher rejected nested and noninteractive starts. A zero-pixel PTY reached the fallback after the bounded five-second wait. |
| Live Ghostty | First attachment supplied 1680×972 pixels with zero recorded resize events. Both rooms used Kitty; the Ledger produced 264 native portrait placeholders. The [Hall capture](ghostty-hall.png) shows all three adventurers within the fitted room in a split pane. |
| Delve portrait after reattachment | The [Ghostty capture](ghostty-delve-portrait.png) shows the `stock-boot-` tabs and the stock campaign on the Adventurer card. Pi’s native Gnome Druid portrait is visible, and the three synthetic adventurers remain visible beneath the card in the fitted Delve. |
| Release gate | `just release-verify` passed formatting, Clippy with warnings denied, 887 Rust tests in 68 runs, 28 shell checks, shell syntax and the release build. |

The terminal snapshots corroborate Kitty placement and party size. The captures
record visible fit and native portrait rendering in Ghostty at the observed split
pane sizes. The test environment uses stock Herdr and the release binary from
the development checkout.

## Retained stock review

Open a plain Ghostty shell in a new window and reattach with

```sh
python3 /tmp/qm-stock-kitty/review.py attach
```

The correct workspace is **Questmancer stock dev boot**, with three
`stock-boot-` tabs. The managed pane is set to Delve with an adventurer card open.
