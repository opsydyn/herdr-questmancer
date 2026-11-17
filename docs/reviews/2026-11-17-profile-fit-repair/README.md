# Profiles preserve fitted rooms — 2026-11-17

| Work receipt | Result |
| --- | --- |
| result | Engineering-qualified; native visual review pending. |
| claim | Opening an adventurer profile retains the room's proportional fit in both the Hall and Delve. The owned dogfood UI is refreshed and open. |
| evidence | 830 Rust tests / 66 runs, 28 shell tests, release/Storybook builds, Rustdoc, eight isolated executable PTY scenarios and two byte-exact image comparisons. |
| remaining uncertainty | Actual Ghostty compositing, appearance, mouse and resize acceptance require human review. |
| next | Open/close a profile in both rooms using the [checklist](../../manual-test/profile-fit.md); recommendation: needs review. |

The screenshot exposed an explicit adapter guard: an adventurer profile disabled
native fitting and restored the enlarged half-block scene. The owner requested
the correction in both rooms. [ADR-0005](../../adr/0005-profile-keeps-world-fit.md)
supersedes the earlier profile exception; other modal parchments retain their
existing behavior.

The world retains its original fit, source pixels and image lease beneath the
existing profile. Shared row slicing paints only exposed top, left and bottom
bands, preserving the source coordinates of the same image. This also prevents
the profile from suppressing an initial image upload or leaving stale text when
closed. The existing right gutter joins the mask. Profile dimensions, native
portrait/fallback, copy and keyboard actions remain unchanged.

Drawing and input share the card rectangle. Covered or partly covered actor
targets are excluded; clicks inside the card cannot select an actor behind it or
request its output. Keyboard selection retains access to the whole party. A
fully occluded image retains its prepared lease without an animation deadline;
its first exposure still uploads correctly. No domain facts, persistence schema,
Herdr commands, portrait routing or dependencies changed.

The focused [red log](red.log) reproduced `Guild: profile replaced the fitted
world` before implementation. Regression tests now cover both rooms, unchanged
fit/cache through toggles, covered-click command safety, seven review sizes,
initial full occlusion, first exposure, resize and existing fallback paths.
Storybook adds fixed Hall/Delve profile studies, bringing the catalogue to 74.

`just verify` passes 830 Rust tests across 66 runs, 28 shell tests, formatting,
Clippy with warnings denied and script syntax. Release, Storybook and Rustdoc
builds pass. The locked release build restored missing local Cargo cache archives;
no manifest or lockfile change was needed. [verification.json](verification.json)
records hashes and commands. This local candidate is based on dirty `main` at
`9c52761`, with prior work preserved; no commit or publication occurred.

The [PTY receipt](pty-receipt.json) uses the pinned actual Storybook executable
with isolated configuration and synthetic terminal replies. Eight scenarios
cover each room initially closed at 120x34, initially open at 120x34, 80x24 and
fully occluded 60x14. Each repeats toggles, clicks the profile, resizes to 96x28,
then exits. The decoder checks exact visible placeholder coordinates and source
rows/columns, unchanged world uploads during static toggles, no stale card text,
native portrait footprint, two owned world IDs, exact-ID cleanup and terminal
restoration. It does not inspect Ghostty or operate any live adventurer.

Both initial-profile world uploads match all RGBA bytes of the shared fit
projection at 800x464. See [transport-pixels.json](transport-pixels.json).
The PNGs are source/transport evidence, not terminal screenshots. Transcripts
and the pinned probe executable remain in
`/private/tmp/questmancer-profile-fit-20261117/`.

```sh
python3 docs/reviews/2026-11-17-profile-fit-repair/pty_probe.py
bash docs/reviews/2026-11-17-profile-fit-repair/regenerate.sh
python3 docs/reviews/2026-11-17-profile-fit-repair/verify_transport.py
```

Fresh pane/process checks verified ownership before refreshing only `w8:pY`,
terminal `term_660769b53dfe0f`. Runtime PID changed from 68375 to 98140, starting
in the Delve with release SHA256
`f9e2f32004e9dcda299ba549a3fc334fc21512ff4cca29080f8a4f3bd85ae5b9`.
The remaining tracked synthetic receiver was preserved; three already-absent
historical synthetic panes were not recreated. All 13 persona records and
authoritative owned-work bytes are unchanged. Focus stayed on its existing pane;
no focus command was issued. Plugin logs report no entries. The refreshed own-UI
text has the native footer and zero half-blocks, establishing the adapter branch,
not visual acceptance. See [live-refresh.json](live-refresh.json).

The UI and remaining synthetic resource are left open. Live profile interaction
was left for human review rather than navigating real adventurers for tests.
No real-agent input, counsel, focus or selected-output test was performed, and no
shared server/configuration change occurred. Computer Use cannot access Ghostty;
no alternate GUI automation was used.
