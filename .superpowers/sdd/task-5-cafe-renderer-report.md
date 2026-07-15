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
