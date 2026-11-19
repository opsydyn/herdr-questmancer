# Stock Herdr dev boot in Ghostty

- **Result** · Launcher qualified. Delve and portrait screenshots pending.
- **Claim** · `just dev-ghostty` starts the source-linked Questmancer plugin on stock Herdr without a manual resize.
- **Evidence** · A failing ordinary launch, two fresh boots, a reattach, live Kitty output through Ghostty, and the [stock Hall capture](ghostty-hall.png). See the [receipt](receipt.json).
- **Remaining uncertainty** · Visual Delve and portrait review of this specific stock trial.
- **Next** · Review the Delve and adventurer card in the retained stock session.

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
| Release gate | `just release-verify` passed formatting, Clippy with warnings denied, 887 Rust tests in 68 runs, 28 shell checks, shell syntax and the release build. |

The term snapshots corroborate Kitty placement and party size. The screenshot
records visible fit in Ghostty. The test environment uses stock Herdr and the
release binary from the development checkout.

## Retained stock review

Open a plain Ghostty shell in a new window and reattach with

```sh
python3 /tmp/qm-stock-kitty/review.py attach
```

The correct workspace is **Questmancer stock dev boot**, with three
`stock-boot-` tabs. The managed pane is set to Delve with an adventurer card open.
