# Pane header repair — 2026-09-11

The repair is implemented in the current Questmancer checkout and the existing
`/tmp/herdr-pane-sprite` checkout. It is not published or installed in the shared
Herdr server. Earlier incremental commits did not establish feature acceptance.

The dedicated optional `pane.viewport_header.info` / `pane.viewport_header.sync`
API replaces the incomplete graphics-label extension. A sync atomically replaces
one producer's full set, including clearing omissions. Exact terminal and session
identity, monotonic producer sequence and a 30-second lease guard stale work.
Herdr owns the reserved row and image layer; ordinary graphics requests cannot
write that layer. Tiny panes omit headers when no reserved rectangle fits.

Questmancer publishes live names, classes and conditions with its personalised,
authored 8×12 RGBA roster sprite. It excludes its managed pane and unqualified
identities. A single worker coalesces changes and renews active leases every ten
seconds. Identical renewal does not dirty rendering. Unsupported capability is
quiet until reconnect. Shutdown attempts an acknowledged clear; crash cleanup
uses the server lease. No header is written through terminal input or persistence.

## Automated qualification

- [Questmancer full verification](questmancer-verify.log): 666 Rust tests across
  58 runs and 28 shell tests; formatting and warnings-denied Clippy pass.
- [Herdr check](herdr-check.log): native Clippy, 3,186 nextest tests (four skipped),
  101 maintenance tests, six hotpath tests and integration-asset checks pass.
  The complete command then fails at Windows cross-compilation because vendored
  libghostty cannot find the Windows C SDK's `stdlib.h`. This gate is incomplete.
- [Herdr documentation contracts](herdr-docs.log): seven tests pass.
- [Render-scale checks](herdr-render-scale.log): four profile cases pass. These
  retain the existing header-disabled baseline; they are not native visual QA.
- Both native release builds pass. Questmancer SHA-256:
  `9d00b77d99e67f706c723868af84542f2ba40928d91ce72730de6de3f80afc95`.
  Herdr SHA-256:
  `d707199b988b45fe07ea39157ebe64c7cdc9502a5ebc74d91f1069d8bf3cb1d3`.

Focused coverage includes stale identities/sequences, atomic clear/replacement,
expiry, unchanged renewal, managed-pane exclusion, missing capability, uncertain
publication teardown, runtime coalescing, actual reserved-row composition,
client-surface encoding and retained graphics placement. Rendering assertions
establish geometry and transport invariants, not art approval.

## Isolated runtime acceptance

[Graphics-enabled receipt](live-graphics.json) and [graphics-disabled receipt](live-text.json)
pass using both actual release binaries, a source-linked managed plugin, temporary
storage and owned compiled sleepers named `pi`. These are synthetic agents.
The [retained harness](isolated-acceptance.py) uses native session-start reports
and strictly increasing lifecycle sequences. Request logs are retained alongside
the receipts; socket acknowledgement does not prove native image display.

Both runs accepted the authored pixels, excluded the managed pane, reflected all
four supported states, accepted session replacement and rejected the old tuple.
The terminal retained its startup marker without header text or echoed commands.
Closing the released dummy pane cleared the producer set. Actual SIGTERM shutdown
sent an acknowledged empty sync. The graphics-enabled run additionally proved
restart publication, ten-second renewal and crash lease expiry within 31 seconds.
The graphics-disabled run proves API/lifecycle compatibility; text composition is
covered by rendered-buffer tests, not by a GUI screenshot.

Both final inventories contain zero panes and zero plugins. Servers exited zero,
sockets disappeared and all recorded owned process IDs were gone. Detailed server
logs retain expected rejected stale/conflicting requests, unavailable headless
portrait geometry and late `PaneDied for unknown pane` warnings after explicit
close. The crash case also records cleanup requests against its already-exited
managed pane. No panic appeared. A separate plugin-log API export was not retained.

[Attempt audit](attempts.json) records provisional fixture failures: custom sources
cannot establish native identity; copied macOS system binaries were killed;
session-start and state reports need distinct sequences; Pi replacement needs a
`new` event; report release alone can leave a detected native identity; and normal
plugin exit may already remove its pane. These were harness corrections. Every
finished attempt recorded owned-server/process cleanup.

## Remaining gates

Computer Use refuses Ghostty access for safety reasons. Native appearance,
real-terminal resizing and shared-server installation remain unreviewed. The
shared server was not restarted. Windows qualification requires an environment
with the Windows C SDK. Do not describe this repair as cross-platform release
ready until those gates are resolved.
