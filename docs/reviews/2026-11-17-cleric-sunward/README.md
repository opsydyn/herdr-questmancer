# Sunward Chaplain Cleric — 2026-11-17

- result: complete
- claim: Approved direction A now supplies all eight Cleric ritual frames and the personalised card fallback. The release build is running in the retained test-owned UI, with the Delve open for review.
- evidence: `verification.json`, focused red/green logs, full-gate log, production fixture sheets and `live-refresh.json`.
- remaining uncertainty: Production poses, animation and native terminal appearance await human review. Synthetic runtime text and fixture exports do not establish native visual acceptance.
- next: Review the Cleric in the open Delve and the production pose/playback sheets. Commit and publication remain separate.

## Implemented scope

Only the Cleric art section of `src/scene/assets/rituals/support_art.rs` changes
production pixels. Sunward Chaplain retains the approved A open face, beard,
silver shoulders, cream tabard, sun mace and gold-bound prayerbook. All eight
frames use the existing `16x24` canvas, row-21 feet and foot columns
`5,6,7,10,11,12`.

Working has two small hand/book changes while the head, mace and feet stay
still. Counsel raises a hand above the book then settles. Spoils are held in
front and placed beside the boots. Resting closes the eyes and lowers the hand;
unknown has a separate expression. The mace and prayerbook remain visible in
every pose, including quiet and settled states. The existing 500 ms / 600 ms /
1000 ms sampling and three-second completion window are unchanged.

Skin, hair and accent use existing saved-persona roles. Two small trim stitches
retain the saved garb role; they do not repaint the tabard or armour. The
existing fallback route centres the exact personalised Working A sprite without
scaling in its `24x32` canvas. All native PNGs, roster art, persona assignments,
persistence, renderer and timing code remain unchanged. The Paladin/Druid
sections and class routing in the shared file remain byte-identical. There is
no architecture or public API change.

The user selected “a” from the [original study](../../design/reviews/2026-11-17-cleric-study/README.md).
Its [approval record](../../design/reviews/2026-11-17-cleric-study/visual-approval.md)
distinguishes that direction choice from later production acceptance. Original
study PNGs remain byte-identical.

## Production review

- [Eight poses and six saved skin tones](01-production-poses.png)
- [Actual lit Hall and Delve placements](02-production-rooms.png)
- [Ratatui fallback card and retained roster](03-card-fallback.png)
- [Exact updated canonical Delve fixture](04-canonical-delve.png)
- [Existing native card and new personalised fallback](05-native-and-fallback.png)
- [Two-frame working playback](working-loop.gif)

These are production fixture exports, not terminal screenshots. The room
exports use actual sampling, lighting, grounding and identity labels. The
fallback card reconstructs actual Ratatui cells with Menlo. The native
comparison uses the unchanged embedded PNG as the class identity reference.
No Herdr resource or persistent application state is created by the exporters.

## Qualification

The first focused tests failed against the previous hooded master at `(1,1)`
and the missing prayerbook in Settled. They pass with the replacement. The
tests protect the approved Working A mask across all six skin tones, exact
fallback continuity, fixed head/mace/feet during working, settled counsel,
all-pose anchors, retained equipment and distinct completion gestures.
Existing all-class tests cover palette substitution, garb and card contracts.

- Focused Sunward tests: **2 passed**.
- Affected Storybook-feature suites: **90 tests / 6 runs passed**.
- Fresh `just verify`: **798 Rust tests / 64 runs**, **28 shell tests**,
  formatting, Clippy with denied warnings and shell syntax passed.
- `cargo build --release`: passed; candidate SHA-256 in `verification.json`.
- Review exporters compile with warnings denied. Repeat generation produced
  identical PNG hashes. Review Rust/Python syntax, script syntax and
  `git diff --check` passed.

The first affected run correctly rejected the old canonical Delve golden.
The exact new scene was exported and inspected before updating the retained
assertion to `6baaf0f61bc4ddeb24db765776ff3b2aa30c6b67409101daf0c286169750667d`.
The full gate ran with temporary Unix-socket/PTY permissions, as required by
the existing runtime tests; no tests were skipped or relaxed.

Full-gate and release logs are `/tmp/questmancer-cleric-verify-20261117.log`
and `/tmp/questmancer-cleric-release-20261117.log`. Export logs are
`/tmp/questmancer-cleric-production-20261117.log` and
`/tmp/questmancer-cleric-production-repeat-20261117.log`.

The source baseline changed only the shared support art file, archetype test
file and inspected Delve golden test. Exact counts and image hashes are in
`verification.json`. The new approved-master fixture was captured from the
study before implementation. All unrelated preexisting owned-work,
compatibility, Druid, Librarian and Sorcerer work was preserved.

## Retained dogfood UI

`refresh_dogfood.py` revalidated the existing resource ledger, UI pane terminal
and runtime PID, plus all four synthetic receiver identities/processes. It
captured owned-work and saved-persona digests before a clean UI exit, waited
for the owned shell, then started the release binary through the existing
trial launcher. Both comparisons passed after restart.

The Delve is left open with the card closed. The harness did not search for,
select, observe or counsel any agent. All four synthetic identities and their
statuses remain intact; the UI is not an adventurer. Plugin logs are empty.
No shared-server restart, plugin relink or cleanup occurred. The selected
Cleric in the user's screenshot was a real Codex agent; its identity and state
were not edited for the art review.

The run-specific ledger remains `/private/tmp/qmdogfood-74b_9_02/receipt.json`.
Revalidate it before later control or cleanup. The party remains open in the
**Questmancer dogfood** tab. Native appearance awaits direct human review.

## Regeneration

With Pillow available to the selected Python:

```sh
REVIEW_PYTHON=python3.13 bash docs/reviews/2026-11-17-cleric-sunward/regenerate.sh
```

This rebuilds the exporters against current production and writes only this
review's fixture data and sheets. It does not modify production art, operate
Herdr or regenerate historical approved study images. The local run used the
existing cached Pillow dependency without installation.

Recommendation: **needs review** for production animation and native appearance.
