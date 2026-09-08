# Card fallbacks using the current sprites

Date: 2026-09-05. Implemented at the Questmancer's request to replace the old
long-torso card fallbacks with the new sprites. The Questmancer visually
approved the resulting card fallbacks on 2026-09-05 with “Approved”.

Wizard, Ranger and Barbarian now reuse the exact personalised `16x24` world
sprite as a static card portrait. It is centred in the existing `24x32` canvas
with four transparent pixels of margin on each side. The art is neither
stretched nor copied into a separate master, so subsequent changes to those
world sprites also reach their card fallbacks.

- [Before and after](01-fallback-before-after.png): previous portrait masters,
  current fallbacks and their matching world sprites, including literal 1x.
- [Production cards](02-cards-in-context.png): all three cards through the
  actual Ratatui overlay in truecolour/Unicode and ANSI-16/ASCII.

The cards retain their text, dimensions and actions. Native class illustrations,
other classes' fallbacks and the revised independent Librarian artwork keep
their existing routes. Storybook uses the same updated fallback accessor;
there is no additional production renderer or animation timer.

The regression first failed on the old Wizard portrait. It now verifies exact
world-sprite pixels, transparent margins and matching persona palettes for all
three classes with two skin/hair combinations each. All 53 affected asset,
card-overlay, Librarian and Storybook tests pass. The full `just verify` gate
also passes: **551 Rust tests across 50 test runs, 26 shell tests, formatting,
Clippy with warnings denied and script syntax**. `git diff --check` passes.

The two sheets were inspected by the implementer. They are production RGB and
Ratatui buffer exports with Menlo text reconstruction, not terminal screenshots
or live Herdr acceptance. They supersede the old portraits shown in the
original pilot's card sheet. The images retain their pre-review footer; the
approval above records the later decision without regenerating approved art.
No live pane, server or native transport was used.

To regenerate with Pillow available in the selected Python:

```bash
REVIEW_PYTHON=python3 bash docs/design/reviews/2026-09-05-card-fallbacks/regenerate.sh
```

The script compiles the exporter against the current Cargo library with
warnings denied. The before images use the retained older archetype masters;
current images use the production card and world accessors. Temporary files
are removed on exit. The next acceptance step is a real terminal resize pass
with the party and cards before expanding the remaining classes.
