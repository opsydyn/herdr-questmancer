# Task 1 report: terminal-independent RGB core and half-block boundary

## Implementation summary

- Added the terminal-independent `scene` module with opaque `Rgb` pixels,
  signed points/rectangles, checked buffer indexing, clipped drawing, reusable
  `RgbBuffer` storage, and checked allocation-size multiplication.
- Added transparent `SpriteFrame` values plus clipped normal and mirrored blits.
- Added the Ratatui-only `ui::scene_adapter::flush_rgb` boundary. It writes a
  static upper-half block (`▀`) with the top logical pixel as foreground and the
  bottom logical pixel as background, using the supplied fallback for missing
  pixel rows.
- Added direct example tests, 1,024-case geometry property tests, and adapter
  boundary tests. The pre-existing `ui::pixel` implementation and tests were
  not changed.

## RED evidence

### Core examples

Command:

```bash
cargo test --test scene_pixel
```

Output:

```text
error[E0433]: failed to resolve: could not find `scene` in `questmancer`
 --> tests/scene_pixel.rs:1:18
  |
1 | use questmancer::scene::pixel::{PixelRect, Rgb, RgbBuffer};
  |                  ^^^^^ could not find `scene` in `questmancer`
```

This was expected because the public `scene` module did not yet exist.

### Sprite examples

Command:

```bash
cargo test --test scene_pixel
```

Output:

```text
error[E0432]: unresolved import `questmancer::scene::sprite`
 --> tests/scene_pixel.rs:3:5
  |
3 |     sprite::{SpriteFrame, blit, blit_mirrored},
  |     ^^^^^^ could not find `sprite` in `scene`
```

This was expected because the transparent sprite API had not been introduced.

### Half-block adapter examples

Command:

```bash
cargo test --test scene_adapter
```

Output:

```text
error[E0432]: unresolved import `questmancer::ui::scene_adapter`
 --> tests/scene_adapter.rs:3:9
  |
3 |     ui::scene_adapter::flush_rgb,
  |         ^^^^^^^^^^^^^ could not find `scene_adapter` in `ui`
```

This was expected because the Ratatui adapter boundary had not been introduced.

The property suite follows the brief's ordering (implementation in step 5,
properties in step 6), so its first execution is a GREEN regression check
rather than a separate RED cycle.

## GREEN evidence

```bash
cargo test --test scene_pixel
```

Result: 4 passed; 0 failed.

```bash
PROPTEST_CASES=1024 cargo test --test scene_pixel_properties
```

Result: 2 passed; 0 failed, including 1,024 cases per property.

```bash
cargo test --test scene_adapter
```

Result: 4 passed; 0 failed.

```bash
cargo test --test scene_pixel --test scene_pixel_properties --test scene_adapter
cargo test --test pixel
cargo clippy --all-targets --all-features -- -D warnings
```

Result: foundation tests 10 passed; legacy `pixel` tests 11 passed; Clippy
completed with exit status 0 and no warnings.

```bash
cargo test
```

Result: the full suite completed with exit status 0.

## Files changed

- `src/lib.rs`
- `src/scene/mod.rs`
- `src/scene/pixel.rs`
- `src/scene/sprite.rs`
- `src/ui/mod.rs`
- `src/ui/scene_adapter.rs`
- `tests/scene_pixel.rs`
- `tests/scene_pixel_properties.rs`
- `tests/scene_adapter.rs`

## Self-review

- `rg -n "ratatui|crossterm|HERDR|std::fs|std::time::SystemTime" src/scene`
  returned no matches.
- `flush_rgb` iterates clipped cells directly and performs no `String`, `Vec`,
  `format!`, or `Text` construction.
- `git diff --check` returned no whitespace errors.
- The adapter tests cover red/blue pairing, non-zero destination coordinates,
  odd-height fallback, zero area, target clipping, and a source wider than the
  requested destination.

## Concerns

None.

## Review fix

Addressed the signed-coordinate overflow identified in review. `blit` and
`blit_mirrored` now use checked destination-coordinate addition and skip an
opaque source pixel whenever either destination coordinate is unrepresentable.

### RED evidence

Command:

```bash
cargo test --test scene_pixel extreme_destination_coordinates
```

Result before the fix:

```text
running 2 tests
test blit_skips_unrepresentable_extreme_destination_coordinates ... FAILED
test mirrored_blit_skips_unrepresentable_extreme_destination_coordinates ... FAILED

thread 'blit_skips_unrepresentable_extreme_destination_coordinates' panicked at src/scene/sprite.rs:55:28:
attempt to add with overflow

thread 'mirrored_blit_skips_unrepresentable_extreme_destination_coordinates' panicked at src/scene/sprite.rs:55:28:
attempt to add with overflow
```

The two new focused tests cover normal and mirrored blits at both `i32::MAX`
and `i32::MIN` origins.

### GREEN evidence

```bash
cargo test --test scene_pixel
```

Result: 6 passed; 0 failed.

```bash
PROPTEST_CASES=1024 cargo test --test scene_pixel_properties
```

Result: 2 passed; 0 failed, including 1,024 cases per property.
