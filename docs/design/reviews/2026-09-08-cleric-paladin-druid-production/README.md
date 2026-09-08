# Cleric, Paladin and Druid production review

The Questmancer approved the [storyboard](../2026-09-08-cleric-paladin-druid-storyboard/README.md)
on 2026-09-08. Its twenty-four frames are now implemented through the existing
production asset, persona and scheduling paths. **Production visuals were approved by the Questmancer on 2026-09-08.** Native-terminal, live Herdr and release acceptance remain separate.

## Production behaviour

- Cleric traces a book, Paladin checks a shield strap, and Druid bends a living
  staff's leafy sprig. Every class has eight authored `16x24` moments.
- Working alternates every 500 ms with fixed heads and feet. A new blocked
  episode signals once for 600 ms, then waits still. A counsel receipt alone
  does not resume work; a newer Herdr working state must do that.
- Spoils are held initially and placed at 1000 ms. Shared completion theatre
  settles by three seconds. The sealed book, wrapped shield and seed pouch
  are metaphors; they do not claim verified task contents or outcomes.
- Reduced/still modes and disconnected retained facts remain static. Newer
  quiet or exited states interrupt obsolete theatre. No domain, persistence,
  counsel, persona generation or scheduler logic changed in this batch.
- Existing card fallbacks, native PNGs and roster families are retained.
  Storybook now has 40 fixed stories, including **Cleric Poses**, **Paladin
  Poses** and **Druid Poses**.

Authored data lives in `src/scene/assets/rituals/support_art.rs`. The existing
registry now contains nine ritual classes. Each new entry explicitly has no
roster override, so narrow views retain their current roster-family route.

## Review files

1. [Production poses](01-production-poses.png): eight moments per class,
   native 1x samples, twelve deterministic persona variants and retained rosters.
2. [Rooms and viewports](02-rooms-and-viewports.png): actual Ratatui buffers
   across canonical, compact, roster, vignette/crop and status-only sizes,
   an eleven-adventurer party and ANSI/ASCII examples.
3. [Ritual timeline](03-ritual-timeline.png): each class at its real station in
   both rooms. Solo fixtures avoid the Delve's per-station capacity hiding one.
4. [Counsel outcomes](04-counsel-outcomes.png): existing reducer/result-handler
   fixtures retain blocked presence through confirmed, rejected, uncertain and
   submit-recovery outcomes. No text is sent to a real agent.
5. [Cards and Delve](05-card-and-delve-check.png): retained independent card
   fallbacks beside the new world sprites, plus the current quiet Delve reference.

[Working playback](working-loop.gif) repeats two 500 ms frames.
[Party journey](party-journey.gif) plays once through counsel, a separately
observed working state, fresh spoils, placement and settled completion.
Alternating 120/130 ms GIF durations represent paired 125 ms completion
samples; the Rust scheduler owns the actual deadlines.

These are production RGB and reconstructed Ratatui buffers using fixed
Storybook fixtures. Document text uses Menlo. They are not terminal screenshots;
fixture completion does not establish a real-agent done transition. Herdr's
synthetic interface cannot provide an explicit done report.

## Verification

`just verify` passed **593 Rust tests across 54 runs, 28 shell tests,
formatting, Clippy with warnings denied and shell syntax**. The release build
also passed. See `verification.json`, the focused red/green logs, the integration log,
`just-verify.log` and `release-build.log` for results from this uncommitted
worktree. These do not extend the older clean `c3720a9` package qualification.

The working regression failed first: Cleric returned no room deadline where
500 ms was required. The shared timing suite now covers all nine ritual
classes in both rooms. Persona-mask, fixed work-head/foot and lifecycle
interruption checks cover the new trio too. The static-control fixtures use
Rogue, which still has no work sequence. The quiet Delve RGB hash was changed
only after inspecting the current production reference shown in sheet 05.

The exporter independently compares all 24 production poses with the approved
storyboard JSON: every transparency pixel and class-owned material pixel
matches. Persona role colours remain dynamic. All five sheets, both working
GIF frames and counsel/arrival/placement/completion journey frames were
inspected. Both GIFs' frame counts and durations were checked.

The first full-gate attempt encountered sandbox restrictions on existing
isolated Unix-socket fixtures. The final gate was rerun with the required
local fixture access. No live Herdr pane, server, registration or real
adventurer was operated on.

```bash
REVIEW_PYTHON=/path/to/python3 bash docs/design/reviews/2026-09-08-cleric-paladin-druid-production/regenerate.sh
```

Use a Python containing Pillow. Rust dependencies are resolved from Cargo
build receipts. Original storyboard images remain the design approval record;
they are not regenerated against these new production routes.

After production visual approval, the next bounded slice is to align these
three card fallbacks with their new personalised world sprites using native
size centring in the existing canvas. This preserves identity between rooms
and cards. No commit, package qualification, tag or publication is part of
this implementation slice.

The subsequent card alignment was approved and implemented. See the
[card fallback review](../2026-09-08-cleric-paladin-druid-card-fallbacks/README.md)
for the current card rendering and its separate visual gate. Original images
here retain the prior independent cards as the production approval record.
