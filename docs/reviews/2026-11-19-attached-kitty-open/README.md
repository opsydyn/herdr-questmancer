# Kitty startup from the normal Herdr action

- **Result** · complete
- **Claim** · The normal `open` action starts Kitty in a fresh Questmancer tab on stock Herdr 0.9.5 with an already-attached client that reports pixel geometry.
- **Evidence** · The user confirmed the [Hall](ghostty-hall.png), [Delve](ghostty-delve.png), and [native Rogue portrait](ghostty-hall-native-portrait.png) in their existing Ghostty session. The [receipt](receipt.json) records these captures and the automated measurements.
- **Remaining uncertainty** · Concurrent-client focus behavior and visual appearance across the full resize range remain unverified.
- **Next** · Use the [normal opening workflow](../../troubleshooting/native-portrait-rendering.md#opening-in-an-attached-ghostty-window) in the existing Herdr session.

On stock Herdr 0.9.5, the original action opened a 128×44 pane with zero
pixel dimensions even though the attached client reported 8×18 pixel cells.
The first Hall contained 5,310 RGB half-blocks and zero Kitty placements.
[Herdr issue #5594](https://github.com/herdrdev/herdr/issues/5594)

Questmancer now invokes `herdr tab focus` on the newly created tab so Herdr
applies the attached client's measured geometry. The application waits up to
500 ms for PTY pixels before the normal Kitty query. The extra focus request
only runs on creation. If it fails, the pane keeps its registration and renders
using the available capability.

The trials used the installed stock Herdr binary with protocol 22, a real
attached Herdr client in a test PTY and a separate disposable three-adventurer
party per trial. Every launch invoked
`herdr plugin action invoke opsydyn.questmancer.open` after attachment. The
128-column baseline and the candidate runs recorded zero client resize events.
Each candidate run reused the original managed pane and process on a second
invocation. The stock servers created for these automated tests were stopped.

## Ghostty confirmation

On November 19 the user opened Questmancer with the normal plugin action in
their existing Ghostty window. The Hall and Delve captures show each complete
room fitted within the managed pane, with all three adventurers in view. The
third capture shows the full Rogue illustration on the Adventurer card,
with the fitted Hall still visible behind it.

## Automated verification

| Check | Host-captured result |
| --- | --- |
| Command from a Herdr shell | 129×44 managed pane at 1032×792 pixels. Hall and Delve each recorded 3,600 Kitty cells and zero half-blocks. |
| Command from the same server environment | 98×38 managed pane at 784×684 pixels. Hall and Delve each recorded 1,600 Kitty cells and zero half-blocks. |
| Portraits | The fitted Delve retained Kitty room tiles and 360 Kitty cells inside the Adventurer card beside its details. The character picker and Librarian's Ledger also recorded Kitty placements. |
| Kitty disabled | With zero pixel dimensions and Kitty disabled, the same action rendered Hall, Delve and Ledger with authored half-blocks. |
| Release gate | `just release-verify` passed formatting, Clippy, 887 Rust tests across 68 runs, 29 shell checks, shell syntax and the release build. |

**Recommendation** · merge. The earlier [dev boot review](../2026-11-19-stock-ghostty-dev-boot/README.md)
preserves the separate pane-before-attachment qualification.
