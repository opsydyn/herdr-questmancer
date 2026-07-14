# Task 2 report: pure theatre state and display preferences

## RED evidence

The first `cargo test --test theatre` run exited 101 after the pose tests were
added. Rust reported unresolved imports for `CharacterSet`,
`DisplayPreferences`, `Motion`, and `ui::theatre`, confirming that the tests
exercised API that did not yet exist.

After the minimal preferences and pose projection made those four tests green,
the clock and cadence tests were added. The next `cargo test --test theatre`
run exited 101 because `RenderCadence` and `cadence_for` were unresolved. This
was the expected second RED boundary before scheduling logic existed.

During self-review, a visibility-boundary test was added. A focused run exited
101 because a working agent produced `Fps(6)` while the model was showing the
Desk, rather than the expected `EventDriven`. Guarding cadence derivation to the
visible Cafe made that regression test green.

The read-only review then identified that frame `0` represented both the first
confetti frame and the stable update badge. The transition-boundary test was
changed first and failed with actual `0`, expected `1`. Active completion
frames now use `1..=8`, reserving `0` for the stable badge.

A second independent review found two semantic gaps. New tests first produced
two genuine failures: at one millisecond before completion the frame was `1`
instead of stable `0`, and done with mismatched unseen attention was projected
as `DoneUnseen` instead of `DoneSeen`. The active interval now requires
`now >= since && elapsed < 1,000 ms`, and completion theatre requires unseen
`WorkCompleted` attention specifically.

## Changes

- Added `Motion`, `CharacterSet`, and `DisplayPreferences` as explicit app
  state, with full-motion Unicode defaults and `Model` accessors.
- Added pure theatre pose and label derivation for every presence state, with
  done-unseen distinguished from stable done only by unseen `WorkCompleted`
  attention. Clear, seen, snoozed, and mismatched unseen reasons are stable
  done; all non-done poses remain presence-driven regardless of attention.
- Added deterministic animation frames derived only from injected timestamps.
  Working uses four frames at 6 fps, blocked two at 2 fps, done-unseen active
  frames `1..=8` at 8 fps for strictly less than 1,000 ms, and idle four at 1
  fps. Frame `0` consistently means a static effect.
- Added adaptive `RenderCadence` derivation across visible agents. Full motion
  selects the fastest required rate, reduced motion retains only the slow idle
  animation, and no motion is entirely event-driven. An empty Cafe and all
  non-Cafe views are event-driven because no theatre animation is visible.
- Kept character-set preference in the model for later widget consumers; this
  task deliberately adds no persistence or renderer effects.

## Verification

- Focused test: `cargo test --test theatre` — 16 passed, 0 failed.
- Full test suite: `cargo test --all-targets` — all test binaries passed.
- Formatting: `cargo fmt --all` followed by `cargo fmt --all --check`.
- Lints: `cargo clippy --all-targets -- -D warnings` — clean.
- Patch hygiene: `git diff --check` — clean.

## Self-review and concerns

- `frame_for` borrows the agent and preferences and cannot mutate domain
  attention. It performs no wall-clock reads; all timing comes from `now`.
- The one-shot boundary is explicit: 999 ms still produces transition frame 8,
  while 1,000 ms returns the stable frame and event-driven cadence. Before the
  attention timestamp, the frame is also stable and cadence is event-driven.
- Pose and cadence share the same unseen-completion predicate, preventing a
  mismatched attention reason from scheduling invisible completion animation.
- Reduced motion intentionally keeps only the one-frame-per-second idle
  screensaver. Working, blocked, and completion transition effects are static.
- Preferences live only on `Model`, as requested. Config-file persistence and
  timer integration remain later milestone tasks.
- No concerns remain within Task 2 scope.
