# Task 4 report: workstation and profile widgets

## Outcome

Implemented pure Ratatui workstation and profile-card renderers in
`src/ui/widgets`. The public API is:

- `render_workstation(frame, area, agent, theatre, selected, preferences)`
- `render_profile_card(frame, area, agent, theatre, preferences)`

The widgets consume injected `TheatreFrame` values, render no timers, read no
output, and mutate neither app nor domain state.

## TDD evidence

The work was built in independently verified RED/GREEN increments.

1. Initial workstation RED: `cargo test --test cafe_widgets` failed with
   `E0432` because `herdr_webmaster::ui::widgets` did not exist.
2. Profile RED: the focused test failed with `E0432` because
   `render_profile_card` did not exist.
3. ASCII profile RED: the renderer lacked the required `AGENT PROFILE`
   silhouette and the test failed at that assertion.
4. Palette-boundary RED: compilation failed because `ColorMode` and
   `DisplayPreferences::color_mode` did not exist.
5. Tiny-profile RED: the 18x4 profile showed only the decorative frame and
   lost `Codex` plus `[x] BROKEN LINK`.
6. Confetti-range RED: the deterministic comparison found frames 3, 5, and 8
   produced no decoration because their selected string columns were outside
   the row bounds.
7. Explicit confetti-marker RED: frame 1 contained zero `^` markers before the
   one-marker contract was implemented.
8. Minimum-profile RED: a 34x18 card entered full mode but had no remaining row
   for the handle; compact mode now retains the handle and full mode reserves
   the required space.
9. Chair-review RED: done and exited workstations contained no resolved
   `ColorRole::Chair` cells; the semantic chair style assertion failed.
10. Selection-review RED: `selected = true` with `focused = false` still
    rendered the unlit `(.)` lamp.
11. ASCII-presentation RED: non-ASCII names and status text passed unchanged
    into full and compact widget buffers.
12. Unicode-control RED: a printable `Café` name containing an embedded newline
    and escape byte was not presented as the safe single-line `Café ...?` form.

Each failure was observed before its production change, then rerun green.

## Changes

- Added `ColorMode::{Xterm256, Ansi16}` to app display preferences, defaulting
  to xterm-256. The UI layer alone converts it to `Palette`, preserving the
  app/domain boundary.
- Added a full 28x10 workstation with border, name row, six scene rows, state
  row, CRT, desk, chair/persona, deterministic modem/CRT activity, optional
  custom status, and focus/selection treatment.
- Reused `compose_seated_for_palette` and `pack` for the palette-safe 10x12
  seated persona. A semantic chair canvas is composed first and the existing
  persona canvas is overlaid without duplicating persona geometry. Exited
  agents leave the chair visible; done/relaxed poses use shifted, kicked-back
  chair geometry.
- Added explicit non-colour pose communication for working, blocked,
  done-unseen, done-seen, idle, exited, unknown, and focused states.
- Added deterministic done-unseen confetti only for injected animation frames
  1 through 8. Stable frame 0 and done-seen contain no confetti marker.
- Added the full profile card using `compose_profile_for_palette` and `pack`,
  preserving all 16x32 logical pixels as 16 terminal rows alongside handle,
  site/pane, state, accessory, desk prop, custom status, and focus details.
- Added semantic ASCII workstation and profile silhouettes with ASCII borders,
  explicit action markers, and no block or half-block glyphs.
- Added compact zero/tiny-safe projections that retain identity and actionable
  state whenever the area can display them.
- Selection now lights the desk lamp independently of focus. Only actual focus
  adds `LIVE`, so selection never claims a state the domain does not own.
- Added one shared widget presentation boundary. ASCII mode maps non-ASCII and
  unsafe controls to printable placeholders in every full/compact domain-text
  path. Unicode preserves printable Unicode while normalizing newline/tab and
  other controls so domain data cannot alter layout.

## Verification

Final gate run from `/Users/alancurrie/Projects/herdr-web-master`:

- `cargo fmt --all --check` — passed.
- `cargo test --all-targets` — passed, including all 15 `cafe_widgets` tests
  and the existing suite.
- `cargo clippy --all-targets --all-features -- -D warnings` — passed.
- `git diff --check` — passed.

Focused coverage includes every theatre pose, focus without state replacement,
packed seated and full-body figures, xterm-256 and ANSI-16 selection, Unicode
and ASCII projections, all ASCII action markers, deterministic injected
frames, exact confetti lifetime, custom status, and zero/tiny safety.

## Self-review

- Rendering remains pure and deterministic; all animation inputs come from
  `TheatreFrame`.
- State meaning is duplicated in visible text/markers and scene posture, not
  conveyed by colour alone.
- Palette-aware persona composition is reused rather than duplicating geometry
  in widgets.
- ASCII output is checked as entirely ASCII, not merely free of half blocks.
- Selection changes the border, cursor, and lamp. `TheatreFrame::focused`
  independently controls `LIVE`, leaving Task 5 free to define cafe
  selection/focus coordination.

## Originality and concerns

All workstation composition, CRT text, ASCII silhouettes, profile ASCII art,
and animation markers were authored for this implementation. Existing original
persona composers are reused as designed; no supplied character, costume,
logo, pose, or scene composition was copied.

No blocking concerns remain. Task 5 still owns responsive placement and the
room-level meaning of selected versus focused agents.
