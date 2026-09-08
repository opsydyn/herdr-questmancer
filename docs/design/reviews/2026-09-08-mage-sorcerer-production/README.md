# Mage and Sorcerer production review

The Questmancer approved the [storyboard](../2026-09-08-mage-sorcerer-storyboard/README.md)
on 2026-09-08. Its sixteen frames now use the existing production asset,
persona and timing paths. **Production visuals were approved by the Questmancer on 2026-09-08.**
Native-terminal, live Herdr and release acceptance remain separate.

## Production behaviour

- Mage tends a green ember in a silver censer beside its planted skull staff.
  Sorcerer turns a silver focus around a warm spark under a separate gold halo.
  Each has eight authored `16x24` moments with stable feet.
- Working alternates every 500 ms with fixed heads and feet. A fresh blocked
  episode signals once for 600 ms, then waits still. A counsel receipt alone
  does not resume work; a newer Herdr working state must do that.
- Spoils are held initially and placed at 1000 ms. Shared completion theatre
  settles by three seconds. A sealed reagent case and wrapped focus are
  metaphors; they do not claim verified task contents or outcomes.
- Reduced/still modes and disconnected retained facts remain static. Newer
  quiet or exited states interrupt obsolete theatre. This batch changes no
  domain, persistence, counsel or persona generation logic.
- Existing independent card fallbacks, native PNGs and roster families remain.
  Storybook now has 45 stories, including **Mage Poses** and **Sorcerer Poses**.

Authored data lives in `src/scene/assets/rituals/arcane_art.rs`. The shared
registry now covers all fourteen classes exhaustively. Its sampler no longer
needs a class eligibility check; the existing timing sequences are unchanged.
The two new entries omit roster overrides, preserving narrow-view families.

## Review files

1. [Production poses](01-production-poses.png): eight moments per class,
   native 1x samples, twelve deterministic persona variants and retained rosters.
2. [Rooms and viewports](02-rooms-and-viewports.png): actual Ratatui buffers
   across canonical, compact, roster, vignette/crop and status-only sizes,
   an eleven-adventurer party and ANSI/ASCII examples.
3. [Ritual timeline](03-ritual-timeline.png): each class at its real station in
   both rooms, using individual fixtures.
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
passed 86 tests across six suites.

The regression failed first: Mage returned no room deadline where 500 ms was
required. The shared timing suite now covers all fourteen classes in both
rooms. Persona-mask, fixed work-head/foot and lifecycle interruption checks
cover the final pair too. With no unanimated class left, static-control fixtures
now select reduced motion explicitly. Full-motion completion still checks its
125 ms visible-effect deadline.

The quiet Delve reference changed because its resting Sorcerer uses the new
master. The old and new rendered fixtures were inspected before updating its
hash to `5ab466c7677e1815b5e5f07806a4bd4a56fea78a620ea9552ff9c2272db60626`.
The exporter independently compares all sixteen production poses with the
approved JSON: transparency and class-owned material pixels match exactly;
persona role colours remain dynamic. All five sheets, both working GIF frames
and six counsel/arrival/placement/completion journey frames were inspected.
Both GIFs' frame counts and durations were checked.

The full gate uses the isolated local Unix-socket fixture access required by
existing runtime tests. No live Herdr pane, server, registration or real
adventurer was operated on.

```bash
REVIEW_PYTHON=/path/to/python3 bash docs/design/reviews/2026-09-08-mage-sorcerer-production/regenerate.sh
```

Use Python with Pillow. Rust dependencies are resolved from Cargo build
receipts. Original storyboard images remain the design approval record;
they are not regenerated against these production routes.

After production visual approval, the next bounded slice is to align the
Mage and Sorcerer card fallbacks with their personalised world sprites using
native-size centring in the existing canvas. That will complete room/card
identity continuity across all fourteen classes. No commit, package
qualification, tag or publication is part of this implementation slice.

The subsequent [card alignment](../2026-09-08-mage-sorcerer-card-fallbacks/README.md)
now uses the same personalised world sprites at native size in the existing
canvas. Its card visual review is separate. Original production sheets remain
the approval record and show the previous independent card examples.
