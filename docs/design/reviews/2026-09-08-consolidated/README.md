# Consolidated visual review — Questmancer 0.1.9

These files were freshly exported from the 0.1.9 candidate on 2026-09-08.
They are production RGB/Ratatui fixtures, not native terminal screenshots.
Unchanged files are reused only when their new exports match byte-for-byte;
changed outputs are saved here without overwriting earlier review evidence.

## Review order

1. Librarian proportions and Hall/Ledger context.
2. Wizard/Ranger/Barbarian poses, working playback and party journey.
3. Current cards, counsel feedback and both rooms at small sizes.
4. Campaign crests and the applied four-row sidebar.

## Current production files

- [librarian / 01-librarian-proportions.png](../2026-09-05-librarian/01-librarian-proportions.png)
- [librarian / librarian-02-librarian-context.png](librarian-02-librarian-context.png)
- [party-pilot / 01-production-poses.png](../2026-09-05-party-pilot/01-production-poses.png)
- [party-pilot / party-pilot-02-rooms-and-viewports.png](party-pilot-02-rooms-and-viewports.png)
- [party-pilot / party-pilot-03-ritual-timeline.png](party-pilot-03-ritual-timeline.png)
- [party-pilot / 04-counsel-outcomes.png](../2026-09-05-party-pilot/04-counsel-outcomes.png)
- [party-pilot / party-pilot-05-card-and-delve-check.png](party-pilot-05-card-and-delve-check.png)
- [party-pilot / party-pilot-party-journey.gif](party-pilot-party-journey.gif)
- [party-pilot / party-pilot-working-loop.gif](party-pilot-working-loop.gif)
- [card-fallbacks / 01-fallback-before-after.png](../2026-09-05-card-fallbacks/01-fallback-before-after.png)
- [card-fallbacks / card-fallbacks-02-cards-in-context.png](card-fallbacks-02-cards-in-context.png)

- [Campaign heraldry](../2026-09-08-campaign-heraldry/campaign-heraldry.png)
- [Applied four-row sidebar](../2026-09-08-sidebar-comparison/four.toml)
- [Sidebar comparison and caveats](../2026-09-08-sidebar-comparison/README.md)

## Native pass in Ghostty

Computer Use rejected Ghostty access for safety reasons. The following needs
direct user observation. Nothing in the export or isolated integration receipt
closes these checks.

From this checkout run:

```bash
just storybook
```

Review World / Guild Hall and World / Delve; resize through canonical, compact,
roster and the Hall vignette/status-only sizes. Review the Librarian, pilot pose
galleries and class cards. Enter inspects; Escape returns; q exits. Record whether
the header reports native portraits or authored fallbacks. Storybook uses fixed
fixture time; use the playback files above for the authored animation sequence.

Then, in your normal Herdr session with Questmancer linked to this checkout,
reopen Questmancer to load the rebuilt 0.1.9 binary. Inspect the actual four-row
sidebar at narrow/normal widths, both rooms, the Librarian Ledger and native
portraits. Confirm the full pane transport, not only standalone Storybook.
Do not send test counsel to real adventurers or claim synthetic done acceptance;
the [guarded guide](../../../manual-test/questmancer-scene-preview.md) owns live tests.

Record separately: visual approval or requested changes; native terminal/size;
native portrait or fallback result; live transitions actually observed.

## Engineering status

The actual 0.1.9 plugin passed an isolated Herdr 0.9 server check: singleton
open, metadata for working/blocked/idle/unknown, view-action success and status
transition delivery. All owned panes, identities, link and server were cleaned
up. Native visuals and user sign-off remain pending. Clean release qualification
is recorded in [the candidate receipt](../../../reviews/2026-09-08-release-candidate/README.md).
