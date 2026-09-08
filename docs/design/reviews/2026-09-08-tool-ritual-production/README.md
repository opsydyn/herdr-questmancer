# Tool ritual production review

The Questmancer approved the [Bard, Artificer and Testmender storyboard](../2026-09-08-tool-ritual-storyboard/README.md)
on 2026-09-08. Its world rituals are now implemented through the existing
production asset, persona and scheduling paths. **Production visuals were approved by the Questmancer on 2026-09-08.** These fixture exports do not establish native-terminal or live
Herdr acceptance.

## What changed

- Bard plucks a lute; Artificer adjusts a brass device; Testmender draws a
  needle into a patch. Each has eight authored `16x24` moments with stable feet.
- Work alternates every 500 ms. A new blocked episode gestures once for 600 ms,
  then waits still. Spoils are placed at 1000 ms and quiet by three seconds.
- Reduced/still modes, quiet states and disconnected facts do not acquire
  decorative or cleanup wakes. Newer presence interrupts obsolete theatre.
- All poses retain standard persona recolouring. Cards retain their independent
  fallbacks and native illustrations; narrow scenes keep the current roster
  families. No domain, counsel, persistence or persona generation changes.
- Storybook now contains 37 fixed stories, including **Bard Poses**,
  **Artificer Poses** and **Testmender Poses**.

Authored data lives in `src/scene/assets/rituals/tool_art.rs`. The existing
`rituals.rs` registry and sequence sampler handle all six ritual classes.
An optional roster master makes the approved world-only scope explicit;
the three new classes fall back to their existing roster family.

## Review files

1. [Production poses](01-production-poses.png): eight moments per class,
   literal 1x samples, twelve deterministic persona variants and retained roster art.
2. [Rooms and viewports](02-rooms-and-viewports.png): actual Ratatui buffers
   across canonical, compact, roster, vignette/crop and status-only sizes,
   including an eleven-adventurer party and ANSI/ASCII examples.
3. [Ritual timeline](03-ritual-timeline.png): each class in its real station in
   both rooms. Separate solo fixtures keep the Delve station limit from hiding
   a class during review.
4. [Counsel outcomes](04-counsel-outcomes.png): real reducer/result-handler
   fixtures retain blocked presence through confirmed, rejected, uncertain and
   submit-recovery outcomes. No counsel is sent to a real agent.
5. [Cards and Delve](05-card-and-delve-check.png): existing independent card
   fallbacks beside the new world art, plus the current quiet Delve fixture.

[Working playback](working-loop.gif) repeats two 500 ms frames.
[Party journey](party-journey.gif) plays once, showing counsel, a separately
observed working state, fresh spoils, placement and settled completion. GIF
centiseconds alternate 120/130 ms to represent paired 125 ms completion
samples; the Rust scheduler owns the actual deadline.

These are production RGB and reconstructed Ratatui buffers using fixed
Storybook models. Text uses Menlo for the exported review sheets. They are
not screenshots of a terminal, and fixture completion does not establish a
real-agent done transition. The existing Herdr synthetic interface cannot
supply an explicit done report.

## Verification

`just verify` passed: **593 Rust tests across 54 test runs, 28 shell tests,
formatting, Clippy with warnings denied, and shell syntax**. The release build
also passed. These results cover the current uncommitted worktree; the older
clean `c3720a9` package qualification is not extended by them.

The new working regression failed first: the Bard returned no animation
deadline where a 500 ms deadline was required. After integration the room
timing suite covers all six ritual classes. Additional checks exercise
interruption by newer quiet/exited states, disconnected retained facts,
per-pose persona masks, fixed work heads/feet, retained cards and rosters.
The Delve and stage static-control fixtures explicitly use a non-ritual Cleric; the new
Bard motion is covered by the room timing suite. The quiet canonical RGB hash
was updated only after inspecting the exported production reference.

The export also compares all 24 production poses against the approved design
JSON. Every transparency pixel and every class-owned material pixel matches;
persona role colours remain dynamic. The five sheets were inspected, with
working and journey GIF timing checked and their key frames inspected.

The initial full gate hit sandbox denial when existing tests bound isolated
Unix sockets. Verification was rerun with the required local fixture access;
see `verification.json` and the accompanying logs for final results. No live
Herdr resource, server, registration or real adventurer was operated on.

Regenerate using a Python containing Pillow:

```bash
REVIEW_PYTHON=/path/to/python3 bash docs/design/reviews/2026-09-08-tool-ritual-production/regenerate.sh
```

The exporter resolves Rust dependencies from Cargo build receipts. It reads
the approved storyboard JSON for the independent artwork comparison. The
original storyboard PNGs remain the approval record and are not regenerated
from these changed production routes.

After production visual approval, a useful next bounded slice is to align the
three card fallbacks with their new stocky world sprites, following the
original pilot's native-size centring contract. That follow-on was subsequently approved and implemented; see the
[tool card fallback review](../2026-09-08-tool-card-fallbacks/README.md).
No commit, package qualification, tag or publication is part of this slice.
