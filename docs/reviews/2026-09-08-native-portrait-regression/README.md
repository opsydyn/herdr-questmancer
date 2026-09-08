# Native portrait regression — Herdr 0.9 managed panes

On 2026-09-08 the Questmancer supplied Ghostty screenshots and accepted the
sidebar, Guild Hall and Delve appearance, while reporting that previously
working illustrated cards had regressed to sprite fallbacks. Both the
adventurer card and Librarian were affected. Reopening Questmancer without a
code change did not restore them; the user confirmed the same symptom.

## Reproduced cause

The running executable was the current release build from this checkout, with
Herdr client/server 0.9.0 and protocol 22. The legacy graphics setting was true
and `herdr config check` passed. `src/portrait.rs` and the native rendering
routes were unchanged by the all-class fallback alignment.

Test-owned panes captured the transport difference:

| Launch | Kitty query | CSI 16t cell query | PTY pixel dimensions | Production gallery |
| --- | --- | --- | --- | --- |
| Plain terminal pane | OK | 8x18 | Present | All fourteen native cards and Librarian prepared |
| Managed plugin pane | OK | No reply | 0x0 with a valid character grid | Unsupported; sprite fallback |
| Managed plugin with host-reported missing pixels supplied | OK | Still relies on PTY geometry | Derived from `pane.graphics.info` | All fourteen native cards and Librarian prepared |

The raw managed response contained Kitty OK, device attributes and device
status, but no cell-size response. `ratatui-image` 11.0.6 requires both a
protocol and a font size; when CSI 16t and PTY pixel geometry are unavailable,
it returns its Halfblocks picker. Questmancer then retains that result for the
process lifetime. A plain-pane probe alone did not reproduce this regression.

## Repair

Before the existing Picker query, managed startup checks its own PTY geometry.
Only when pixel dimensions are missing does it request `pane.graphics.info`
for its own `HERDR_PANE_ID`, through the existing correlated Herdr client.
The response supplies authoritative cell width/height. The repair fills missing
pixel axes, rereads the grid after the request to respect intervening resizes,
and preserves existing pixel dimensions. Zero/overflowing values are rejected;
the local request has a 500 ms deadline.

The implementation uses safe `rustix` terminal calls. It does not force Kitty,
read another pane, change character rows/columns, alter Herdr configuration,
create a graphics layer or add recurring wakes. Picker's subsequent terminal
query still decides whether native rendering is supported. Errors retain the
authored fallback. The art, card footprint and scene renderer are unchanged.

## Verification and visual boundary

A focused regression used a real disposable PTY and a fake Herdr socket.
Before implementation it failed with `(0, 0)` instead of `(1640, 972)` pixels.
After implementation it passed while preserving the `205x54` character grid.
Additional tests cover existing geometry without an API call, partial geometry,
empty grids, zero/overflow, malformed/denied/stale responses and bounded timeout.
The existing unsupported-protocol and malformed-PNG fallback tests still pass.

See `verification.json` and the logs for the completed full-gate, release-build
and managed-plugin results. Native visual restoration is recorded separately
there; successful protocol preparation alone is not visual acceptance. The user
subsequently confirmed restoration and supplied Ghostty screenshots showing the
native Artificer, Bard and Librarian illustrations on 2026-09-08.

All diagnostic plugins and panes were created for this investigation and
removed afterward. Baseline panes and focus were preserved. The Questmancer
plugin itself was reopened using its normal ordered close/open actions; no
shared Herdr server was restarted. Run-specific IDs in the receipts are evidence
only and must never be reused.

This repair follows clean commit `9ea8501`; that commit's qualification does not
cover the new source. Visual restoration is confirmed; commit and requalify the fixed
candidate before publication. The earlier accepted Hall, Delve and sidebar
screens do not establish real-agent completion or published-installer acceptance.
