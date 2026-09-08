# Three-class production pilot

Date: 2026-09-05. Status: implemented and automatically verified in the current
checkout; production visual approval and manual terminal/live acceptance are
pending. The Questmancer authorised the Librarian refresh followed by this
pilot with “Yes librarian then move on”. The earlier storyboard and roster
state-cue approvals remain recorded in their own review packs.

The later [card fallback refresh](../2026-09-05-card-fallbacks/README.md)
supersedes the old portrait fallbacks visible in sheet 05. This pack retains
the original pilot review state.

## Review files

- [Production poses](01-production-poses.png): all three classes, eight
  authored moments, literal 1x views, persona variation and new roster bodies.
- [Rooms and viewports](02-rooms-and-viewports.png): mixed-party Guild Hall
  and Delve, selected/unselected labels, six sizes, ANSI-16/ASCII roster cues,
  and full-capacity compositions.
- [Ritual timeline](03-ritual-timeline.png): production room crops showing
  work, quiet counsel, fresh spoils and settled completion for every class.
- [Counsel outcomes](04-counsel-outcomes.png): draft, sending, confirmed,
  rejected, uncertain, submission recovery and a late ASCII confirmation.
- [Cards and exact Delve golden](05-card-and-delve-check.png): the existing
  authored portrait fallbacks with readable card text, plus the exact
  still-motion dungeon fixture whose golden hash changed in this slice.
- [Working playback](working-loop.gif): two frames at 500 ms each, repeating.
- [Party journey](party-journey.gif): one playback of work → blocked gesture
  → still wait → confirmed-counsel wait → later working report → returned
  spoils → calm completion. The confirmation's actual notice is in sheet 04.

The timeline and playback use separate one-adventurer room fixtures so all
three classes remain visible. The Delve's existing limit is two actors per
station plus an overflow marker; mixed-party composition is shown separately.
The Hall's `64x40` three-person fixture correctly chooses the roster because
its compact layout must also reserve the Librarian. Size alone never promises
a particular capacity tier.

These are current production sprite, RGB painter and Ratatui buffer exports.
Pillow only arranges those pixels and reconstructs terminal text in Menlo.
They are not terminal screenshots or live Herdr evidence. The Storybook
itself has fixed fixture time; its new pose galleries do not turn it into a
live animation player. GIFs sample production frames at explicit times.

## Implemented behaviour

| Moment | Current behaviour |
| --- | --- |
| Working | Wizard turns a page, Ranger checks a map, Barbarian adjusts the tool; two 500 ms frames, stable feet |
| Counsel | One 600 ms raised-hand gesture from the presence episode, then a still wait; the shared lantern marker remains |
| Confirmed send | Only a correlated text-and-submit success creates the typed red seal in the existing six-second notice; it does not change presence |
| Return | Class-specific spoils held for one second, then placed; shared effects end at the exact three-second deadline |
| Completed/resting/unknown | Distinct static poses; completion does not imply testing, review or acceptance |
| Reduced/still | First working frame and settled one-shot poses; no decorative or cleanup wake |
| Repeated/reconnected facts | Old counsel/return gestures do not replay; newer presence controls the next pose |
| Departure | Exited adventurers leave the live party |

The pilot registers 24 authored `16x24` frames and three independent `8x12`
roster bodies from the approved studies. Persona skin, hair, garb and accent
roles remain runtime substitutions; no saved persona changes. The settled
counsel hand was authored from the approved gesture and working silhouette;
one Ranger belt pixel preserves the existing garb role. Portraits and the
other eleven classes retain their existing art routes.

Frame definitions and timing live in `src/scene/assets/rituals.rs`. Both rooms
share the same sampling and next-visible-pixel deadline. The unused
`ScenePlan.cadence` was removed; `SceneFrame.next_frame_in` remains the runtime
rendering contract. No timer fetches output or writes domain/persistent state.

## Verification record

Baseline: `main` at `efcd87d`, seven commits ahead of the local `origin/main`
reference, plus preserved earlier edits and this slice. This is a dirty-tree
engineering receipt, not release-commit acceptance.

Focused tests were first proven failing for working/counsel timing,
reduced/still completion stability, correlated counsel sealing and Storybook
inventory. The runtime test exercises repeated blocked reports, disconnect,
reconnect and a fresh working/blocked episode for all three classes.
Additional rendered tests check native dimensions, stable foot anchors,
placement of spoils, the final 1 ms deadline and quiet completion.

The final `just verify` passed: **550 Rust tests across 50 test runs, 26 shell
tests, formatting, Clippy with warnings denied and script syntax**. The
exporters compiled against the current Cargo library with warnings denied;
the five pilot sheets and two Librarian sheets were visually inspected by the
implementer. All 80 local documentation links resolve, review scripts pass
syntax checks, GIF frame counts/timing match their metadata, and
`git diff --check` passes. Product visual approval remains pending.

Automated coverage includes both rooms, roster state cues, overflow/hit-region
contracts, selected overlays, Unicode/ASCII confirmation, rejected and
uncertain counsel, submission recovery, late results, reconnect and quiet
motion. Review native transport and actual terminal interaction separately.
No live server, pane, plugin registration, commit or release was changed.

## Regeneration and next gate

From the repository root, with Pillow in the chosen Python environment:

```bash
REVIEW_PYTHON=python3 bash docs/design/reviews/2026-09-05-party-pilot/regenerate.sh
```

This builds the Storybook-enabled production library, links the exporter to
those exact artifacts and typesets the review files. Temporary files are
removed on exit. Preserve approved images before regenerating after later art
changes. The exporter also samples all viewport ANSI/ASCII variants and the
12-adventurer case; selected examples are typeset in the room sheet.

Next, review the Librarian and pilot sheets/playback for recognition and
comfort, then run the [manual terminal matrix](../../../manual-test/questmancer-scene-preview.md)
and guarded Herdr checks from the eventual intended build. The real-agent
done/resting transition and native portrait transport remain unverified.
Keep class expansion and campaign heraldry in the backlog until this pilot's
visual direction is accepted and the next bounded slice is authorised.
