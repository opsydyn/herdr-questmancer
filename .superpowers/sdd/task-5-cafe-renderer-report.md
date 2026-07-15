# Task 5 report: authored connected café renderer

Implemented the renderer slice from `task-5-brief.md`.

## Changes

- Replaced the shared line/card-first café body with connected workspace bays.
- Bays are laid out through `ui::cafe_scene::layout_bays` and use deterministic workspace variants.
- Added authored architecture layers: bay signage, room variant label, doorway/aisle cue, and floor treatment.
- Workstations are rendered from `SeatAnchor` geometry (with a `Rect` compatibility path for existing widget callers).
- Removed the full profile panel from the café scene so the room remains the primary spatial surface.
- Kept compact list rendering and keyboard/footer behavior unchanged.
- Removed the obsolete shared `CABLE RUN`/counter room painting path.

## Verification

Passing: `cargo test --test cafe_scene --test cafe_widgets --test persona_art --test rendering -- --nocapture`

The legacy `cafe_rendering` assertions still expect the removed shared labels and full profile/card grid; those failures are expected until the corresponding contract tests are replaced by the task-level rendering test update.

## Concerns

- Very small bay rectangles intentionally fall back to compact workstation text; the 80x24 path remains actionable but needs the new golden assertions to lock its final composition.
- `render_profile_card` remains available for desk/other callers but is no longer painted over the active café bay.

## Contract test update

The legacy café rendering tests were replaced with authored-scene assertions covering bay cues, aisle/floor/furniture marks, connected workspaces, selected workstations, compact actionability, and zero-size safety.

Exact verification after the test update:

```text
cargo test --test cafe_rendering --test cafe_scene --test cafe_widgets --test persona_art --test rendering -- --nocapture
17 + 4 + 15 + 11 + 3 tests passed; 0 failed.
cargo clippy --all-targets --all-features -- -D warnings
passed.
```

## Review follow-up

- `WallRow`, `CornerBooth`, and `BackRoomLab` now alter seat offsets and authored wall, furniture, doorway, aisle, and floor cues. Rendering tests assert distinct output for all three variants.
- The selected workspace is passed into bay layout and rendered as the active, fully authored bay; neighboring bays use simplified architecture while retaining their seats and labels.
- Transition columns draw explicit doorway/connection geometry between adjacent bays.
- The shared arbitrary `CONNECTED BAYS / 56K FLOOR` frame label was removed.
- The 80x24 contract now tests a compact neighbor-bay strip plus navigation/actions.

Follow-up verification: 19 café rendering + 4 scene + 15 widget + 11 persona-art + 3 general rendering tests passed; Clippy with `-D warnings` passed.

Final visual/compatibility follow-up:

- Wide rooms now render a selected workspace as the full active scene; neighboring workspaces are represented by a compact strip at 80x24 and simplified geometry at larger sizes.
- Bay signage is workspace-attached; arbitrary `CAFE WALL`, `BAY`, `neighbor bay`, and shared floor labels were removed.
- ASCII transition tests verify doorway joins use ASCII glyphs, while Unicode retains box-drawing transitions.
- Compact and multi-workspace ASCII tests cover navigation and reachability.

Final verification: 20 café rendering + 4 scene + 15 widget + 11 persona-art + 3 rendering tests passed; `cargo clippy --all-targets --all-features -- -D warnings` passed.
