# Task 4 report: connected café scene geometry

Status: complete.

Implemented `src/ui/cafe_scene.rs`, exported as `ui::cafe_scene`, with
deterministic BLAKE3 workspace variants (`WallRow`, `CornerBooth`, and
`BackRoomLab`), sorted workspace bay layout, and authored non-overlapping seat
anchors bounded by each bay rectangle. Selection is intentionally ignored by
the pure geometry model so renderer emphasis remains snapshot-stable.

Added `tests/cafe_scene.rs` covering variant determinism and reachability, bay
ordering, bounds, stable generated layouts, and property checks for seat
non-overlap. The existing property-domain suite remains green.

Verification:

```text
cargo clippy --all-targets -- -D warnings   PASS
cargo test --test cafe_scene --test property_domain -- --nocapture   PASS (10 tests)
```

Concern: on very small surfaces, later bays can be off-screen or zero-height;
the renderer should choose a compact visible-bay strategy while preserving
navigation across every workspace.

Review follow-up:

- Added `CafeBay::rect` so the renderer has explicit architecture bounds even
  for zero-agent bays.
- Seat columns and rows are now bounded by actual bay width/height and seat
  capacity; tiny surfaces cannot produce duplicate anchors.
- Coordinate arithmetic uses `u32` intermediates and checked bounded casts.
- Added a tiny-area test and expanded the property generator to exercise
  zero-to-small terminal dimensions.
