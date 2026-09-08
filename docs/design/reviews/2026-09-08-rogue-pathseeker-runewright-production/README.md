# Rogue, Pathseeker and Runewright production review

The Questmancer approved the [storyboard](../2026-09-08-rogue-pathseeker-runewright-storyboard/README.md)
on 2026-09-08. Its twenty-four frames now use the existing production asset,
persona and scheduling paths. **Production visuals were approved by the Questmancer on 2026-09-08.**
Native-terminal, live Herdr and release acceptance remain separate.

## Production behaviour

- Rogue turns a pick in a practice lock, Pathseeker adjusts a compass, and
  Runewright lowers a mallet to a rune stone. Every class has eight authored
  `16x24` moments with stable feet.
- Working alternates every 500 ms with fixed heads and feet. A fresh blocked
  episode signals once for 600 ms, then waits still. A counsel receipt alone
  does not resume work; a newer Herdr working state must do that.
- Spoils are held initially and placed at 1000 ms. Shared completion theatre
  settles by three seconds. A sealed lock case, route folio and bound stone
  are metaphors; they do not claim verified task contents or outcomes.
- Reduced/still modes and disconnected retained facts remain static. Newer
  quiet or exited states interrupt obsolete theatre. This batch changes no
  domain, persistence, counsel, persona generation or scheduler logic.
- Existing independent card fallbacks, native PNGs and roster families remain.
  Storybook now has 43 stories, including **Rogue Poses**, **Pathseeker Poses**
  and **Runewright Poses**.

Authored data lives in `src/scene/assets/rituals/trail_art.rs`. The shared
registry now contains twelve ritual classes. Each new entry explicitly omits
a roster override, preserving the current narrow-view family route.

## Review files

1. [Production poses](01-production-poses.png): eight moments per class,
   native 1x samples, twelve deterministic persona variants and retained rosters.
2. [Rooms and viewports](02-rooms-and-viewports.png): actual Ratatui buffers
   across canonical, compact, roster, vignette/crop and status-only sizes,
   an eleven-adventurer party and ANSI/ASCII examples.
3. [Ritual timeline](03-ritual-timeline.png): each class at its real station in
   both rooms. Solo fixtures prevent the Delve's station capacity hiding one.
4. [Counsel outcomes](04-counsel-outcomes.png): reducer/result-handler fixtures
   retain blocked presence through confirmed, rejected, uncertain and
   submit-recovery outcomes. No text is sent to a real agent.
5. [Cards and Delve](05-card-and-delve-check.png): retained independent card
   fallbacks beside the new world sprites, plus the quiet Delve reference.

[Working playback](working-loop.gif) repeats two 500 ms frames.
[Party journey](party-journey.gif) plays once through counsel, a separately
observed working state, fresh spoils, placement and settled completion.
Alternating 120/130 ms GIF durations represent paired 125 ms completion
samples; the Rust scheduler owns actual deadlines.

These are production RGB and reconstructed Ratatui buffers using fixed
Storybook fixtures. Document text uses Menlo. They are not terminal screenshots;
fixture completion does not establish a real-agent done transition. Herdr's
synthetic interface cannot provide an explicit done report.

## Verification

See `verification.json`, focused red/green logs, the integration log,
`just-verify.log` and `release-build.log` for this uncommitted worktree's
results. These do not extend the older clean `c3720a9` package qualification.

`just verify` passed: **593 Rust tests across 54 runs and 28 shell tests**,
with formatting, Clippy and workflow contracts green. `cargo build --release`
also passed. The focused suite passed 7 tests; the affected integration run
passed 87 tests across six suites.

The regression failed first: Rogue returned no room deadline where 500 ms
was required. The shared timing suite now covers all twelve ritual classes
in both rooms. Persona-mask, fixed work-head/foot and lifecycle interruption
checks cover the new trio too. Static-control fixtures now use Mage, which
still has no work sequence. The quiet Delve reference hash remains unchanged.

The exporter independently compares all 24 production poses with the approved
storyboard JSON: every transparency pixel and class-owned material pixel
matches. Persona role colours remain dynamic. All five sheets, both working
GIF frames and counsel/arrival/placement/completion journey frames were
inspected. Both GIFs' frame counts and durations were checked.

The full gate uses the isolated local Unix-socket fixture access required by
existing runtime tests. No live Herdr pane, server, registration or real
adventurer was operated on.

```bash
REVIEW_PYTHON=/path/to/python3 bash docs/design/reviews/2026-09-08-rogue-pathseeker-runewright-production/regenerate.sh
```

Use Python with Pillow. Rust dependencies are resolved from Cargo build
receipts. Original storyboard images remain the design approval record;
they are not regenerated against these new production routes.

After production visual approval, the next bounded slice is to align these
three card fallbacks with their personalised world sprites using native-size
centring in the existing canvas. This preserves identity between rooms and
cards. No commit, package qualification, tag or publication is part of this
implementation slice.

The subsequently authorised [card alignment](../2026-09-08-rogue-pathseeker-runewright-card-fallbacks/README.md)
now centres the same personalised world sprites in the existing fallback canvas.
Its visual review is separate; the original production sheets remain the
approval record and retain the previous card examples.
