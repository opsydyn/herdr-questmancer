# Task 5 Report: Responsive actionable cybercafe

## Outcome

Replaced the empty Cafe projection with one responsive room driven entirely by
the shared `Model`. The Cafe now renders deterministic BTreeMap-ordered agent
workstations, a wide selected-agent profile, compact narrow-terminal rows,
contextual actions, and explicit connection overlays without adding a second
interaction or Herdr effect path.

## TDD evidence

The change was developed through observed RED/GREEN cycles.

1. Initial responsive RED: `cargo test --test cafe_rendering` ran four tests.
   The 120x30, 80x24, and 60x18 tests failed with `missing Alpha` because
   `views::cafe::render` did not receive or project the model. The existing
   1x1 safety behavior passed.
2. Initial layout GREEN: after passing `&Model` into the Cafe and adding the
   responsive room/grid/list boundaries, the same target passed 4/4.
3. Fallback/connection RED: the expanded 12-test target failed three focused
   cases. The empty-state copy advertised `[/] search` without any agents,
   ASCII mode retained the Unicode outer border, and Offline lacked the
   required `DISCONNECTED` / `LAST POSES PRESERVED` overlay.
4. Fallback/connection GREEN: the contextual empty copy, ASCII Cafe border,
   and state-specific room overlay made the target pass 12/12.
5. Whole-Cafe ANSI RED: tightening the ANSI-16 assertion to reject both
   `Color::Indexed` and `Color::Rgb` failed because the Cafe frame, room,
   footer, overlay, and tiny fallback still used RGB theme styles.
6. Whole-Cafe ANSI GREEN: local semantic `CafeStyles` derived from
   `Palette`/`ColorRole` made every Cafe cell use the selected palette; the
   exact ANSI test then passed.
7. Narrow-footer RED: the 60x18 test was tightened to require every valid
   action and failed with `missing [o] refresh` because the one-line legend
   clipped after Reply.
8. Narrow-footer GREEN: a two-row sub-80 legend keeps visit, reply, refresh,
   seen, and search visible while preserving all three compact workstations.
9. Exact-80 review RED: the 80x24 test failed with
   `missing [space] seen` because the two-row footer boundary excluded exactly
   80 columns. A new floor assertion also failed because the full grid
   overwrote the room's inner floor line.
10. Dense-selection review RED: a 60-agent model selected on its final
    BTreeMap entry failed with `late selection hidden`; the renderer consumed
    the map prefix and compressed workstations to one row.
11. Review GREEN: exact 80 now uses the complete two-row footer, the outer room
    border retains a dedicated `FLOOR / CABLE RUN / COUNTER` cue, and the grid
    deterministically pages whole 28x10 workstations to the page containing the
    current selection.
12. Compact-selection re-review RED: a 60-agent 60x18 model failed with
    `late selection hidden` because compact rendering still consumed the
    BTreeMap prefix.
13. Exact-80 wall re-review RED: the tightened 80x24 assertion failed with
    `full grid overwrote the shared wall cue`; the full 20-row workstation grid
    covered the inner wall caption.
14. Re-review GREEN: full and compact layouts now share deterministic selected
    page calculation, compact pages retain complete identity/state rows, and a
    semantic top-border `CAFE WALL` survives exact-height grids.

The Cafe interaction additions characterize a deliberately pre-existing
view-neutral boundary and passed on their first run. A fabricated reducer
failure would have required adding a Cafe-specific path contrary to the task.
The passing parity tests prove Cafe selection, search, visit, reply, seen,
refresh, and optional Reviewr produce exactly the same `ActionReduction` and
typed `DeskCommand` values as Desk.

## Changes

- Changed the Cafe render boundary to receive the shared `&Model`.
- Added a semantic Cafe palette for xterm-256 and ANSI-16 covering the outer
  room as well as workstation/profile widgets.
- Painted shared wall, floor/cable-run, and counter cues before child widgets.
- Added a maximum-useful full workstation grid with the widget's 28x10 minimum:
  160x50 uses three row-major columns, while 120x30 and 80x24 use two columns
  and two rows for the fixed three-agent model.
- Dense grids compute a deterministic fixed-capacity BTreeMap page containing
  the current selection, so navigation keeps late selected agents visible
  without shrinking workstation cells below 28x10.
- Added the separately composed selected profile beside the grid at terminal
  widths of 120 columns and above. The 80-column layout retains full
  workstations without the profile.
- Added a compact vertical actionable workstation list below 80 columns.
- Compact dense lists page deterministic complete two-line rows around the
  current selection, keeping late selected identity and state visible at
  60x18.
- Bounded dense-grid rows to the drawable rectangle and clamped every cell to
  the remaining grid height, so large agent maps never create off-grid cells.
- Derived every workstation and profile frame with
  `frame_for(agent, model.now(), model.preferences())`.
- Passed selection separately from actual focus so the selected desk gains its
  cursor, double border, and lamp while only domain focus renders `LIVE`.
- Kept explicit state labels visible alongside selection, including `HELP!`
  and `BROKEN LINK` without relying on colour.
- Added contextual Cafe actions for view switching, navigation, search, visit,
  reply, refresh, seen, and optional Reviewr. Agent-only actions are omitted
  when unavailable; layouts at 80 columns and below use two rows so the
  complete valid set remains visible.
- Added Offline, Reconnecting, and Incompatible room overlays while retaining
  the last visible agent poses beneath them.
- Kept the empty Cafe helpful and actionable without advertising invalid agent
  commands.
- Left `src/interaction.rs` unchanged: Cafe reuses the existing reducer,
  selection-driven single output load, and typed command boundary.

## Verification

Verification run from `/Users/alancurrie/Projects/herdr-web-master`:

- `cargo test --test cafe_rendering` - passed, 15 tests.
- `cargo test --test interaction` - passed, 19 tests.
- `cargo test --all-targets` - passed, full Rust suite.
- `cargo fmt --all --check` - passed.
- `cargo clippy --all-targets --all-features -- -D warnings` - passed.
- `git diff --check` - passed.

Coverage includes exact 160x50, 120x30, 80x24, 60x18, 1x1, zero, and other
tiny areas; profile presence/absence; all three stable row-major agents;
selection/focus markers; shared room cues; contextual footer; preserved-pose
connection overlays; ASCII; ANSI-16; reduced motion; and no motion.

## Self-review and concerns

- Rendering is pure: it owns no timer, performs no output/persistence read, and
  mutates no model or domain state.
- The layout uses domain BTreeMap iteration directly, preserving stable
  row-major ordering without a second cache. Tests explicitly verify the first
  Alpha, Beta, and Gamma occurrences and exercise a dense 60-agent map.
- ANSI-16 verification checks the complete Cafe buffer rather than only the
  persona widgets.
- ASCII verification checks the complete Cafe buffer is ASCII and retains
  explicit blocked/exited markers.
- The Cafe adds no duplicate action reducer or Cafe-specific Herdr command.
- Sub-80 layouts trade one room row for a second footer row so no valid Cafe
  action is silently clipped; exact 80 uses the same complete two-row legend.
- A floor/cable/counter title on the room's bottom border survives even when an
  exact-height full grid occupies every inner row.
- A palette-aware `CAFE WALL` title on the top border likewise survives an
  exact 80x24 grid without consuming a workstation row.
- No terminal timers, persistence behavior, output reads, raster assets, or
  supplied reference artwork were introduced.
