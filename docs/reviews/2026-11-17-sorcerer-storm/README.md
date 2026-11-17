# Storm Channeler Sorcerer — 2026-11-17

- result: complete
- claim: The approved Storm Channeler direction now supplies the Sorcerer's eight production ritual frames, personalised card fallback and matching native card. The release build is running in the retained test-owned UI.
- evidence: `verification.json`, red/green regression evidence, full-gate logs, production fixture sheets and `live-refresh.json`.
- remaining uncertainty: Production animation and native terminal visual acceptance await human review. Fixture exports and runtime text checks do not establish native appearance.
- next: Review the selected resting Sorcerer in the open dogfood party, then its working/counsel/completion playback. Commit and publication remain separate.

## Implemented scope

The Sorcerer section of `src/scene/assets/rituals/arcane_art.rs` now uses the
approved swept hair, pale streak, raised collar, asymmetric plum mantle and
hand-held orb. Working, resting and unknown retain the approved study silhouettes.
The two working frames pulse the focus and move a hand while head and feet stay
planted. Counsel raises an open palm then settles into a wait. A sealed pouch
is held then placed for completion. All eight frames retain the orb and the
same row-21 foot anchors on the existing `16x24` canvas.

Skin, hair, accent and small cloth trim use the existing saved-persona roles.
The pale streak, mantle and mint orb core remain class-owned. The fallback
automatically centres the personalised Working A master without scaling in
the existing `24x32` canvas. Roster art, class assignments, persistence, pose
timing, animation scheduling, renderers and other classes are unchanged.
There is no architecture or public API change.

`src/assets/portraits/sorcerer-card.png` embeds the selected built-in `image_gen`
card. `prepare_card.rs` performs only nearest-neighbour normalization from
1086×1448 to the existing 384×512 contract, preserving the 3:4 composition and
generated alpha. There is no cropping, painting or palette substitution. The
unmodified generated master, prompts and provenance remain in the
[original study](../../design/reviews/2026-11-17-sorcerer-storm-study/README.md).
The prepared card is RGBA, alpha 0–255; pixels with alpha greater than 8 occupy
`(28, 17, 370, 504)`. Faint generated alpha residue has not been retouched.

The user's “direciton looks good” approved the paired direction; see the
[approval record](../../design/reviews/2026-11-17-sorcerer-storm-study/visual-approval.md).
The original study images remain byte-identical.

## Production review

- [Eight poses and six saved skin tones](01-production-poses.png)
- [Actual lit Hall and Delve placements](02-production-rooms.png)
- [Ratatui fallback card and retained roster](03-card-fallback.png)
- [Exact canonical Delve fixture](04-canonical-delve.png)
- [Embedded native card and actual personalised fallback](05-native-and-fallback.png)
- [Two-frame working playback](working-loop.gif)

These are deterministic production fixture exports, not terminal screenshots.
The room exports include actual pose sampling, lighting, grounding and labels.
The fallback card is reconstructed from actual Ratatui cells using Menlo.
The native comparison composites the exact embedded PNG for inspection; native
terminal transport and display have not been visually accepted.

## Qualification

The initial focused tests failed on the old halo at `(3,0)` and missing orb.
The native-route regression independently rejected the old PNG digest before
replacement. All three new tests pass in the final affected run. They protect
the approved quiet silhouettes, mint orb core across skin tones, fallback
continuity, anchored animation, distinct semantic gestures and selected native
asset. Existing all-class tests retain persona, garb and fallback contracts.

- Affected Storybook-feature suites: **88 tests / 6 runs passed**.
- Fresh `just verify`: **796 Rust tests / 64 runs**, **28 shell tests**,
  formatting, Clippy with warnings denied and shell syntax passed.
- `cargo build --release`: passed; SHA-256 in `verification.json`.
- Review exporters compile with warnings denied. Two render runs produced
  identical PNG hashes. Exporter formatting, script syntax and `git diff --check`
  passed.

The first full gate failed because sandboxed temporary Unix-socket creation was
denied. The same gate passed when run with the required permissions. An initial
affected invocation named a nonexistent test target; the corrected invocation
used `scene_party`. A first green attempt also caught an incorrectly selected
orb-core coordinate in the test; it was corrected to the approved matrix's
actual mint core. None of these failures was hidden by changing production
contracts or dropping assertions.

The affected Delve golden correctly rejected the previous scene hash. The exact
new canonical scene was exported and inspected before updating the retained
assertion to `c59fdc01df164dbbb9d02dcc6ccedddf8d45246266f837b6a7680ae58833dbd9`.

Raw full-gate and release logs are:

```text
/tmp/questmancer-sorcerer-verify-20261117.log
/tmp/questmancer-sorcerer-verify-unrestricted-20261117.log
/tmp/questmancer-sorcerer-release-20261117.log
```

The 230-file baseline changed only the Sorcerer native PNG, shared arcane art
file, archetype test file and inspected Delve golden test. The remaining 226
files and the Mage section of the shared arcane file are byte-identical.
The new approval fixture was extracted from the study before implementation.
All unrelated preexisting owned-work, compatibility, Druid and Librarian work
was preserved.

## Retained dogfood party

`live-refresh.json` records the revalidated test-owned pane, old/new PID, release
digest, all four retained synthetic identities and selected resting Sorcerer.
The first refresh guard stopped before any input because the UI showed the
world rather than a card. After inspecting that world, the refresh proceeded.
Its shell guard then caught a brief foreground-process handoff after clean UI
exit. Resumption revalidated the shell before launching the candidate.

The byte/persona comparison spans clean shutdown through relaunch; the
authoritative work additionally matches the earlier retained revision-7
snapshot. No shared server restart, plugin relink, agent counsel or cleanup
occurred. The UI is not registered as an adventurer. The party remains open in
the **Questmancer dogfood** tab, with `dogfood-rest` selected on its Sorcerer card.
Its run-specific resource ledger remains under `/private/tmp/qmdogfood-74b_9_02/`;
revalidate it before later operation.

## Regeneration

With Pillow available to the selected Python:

```sh
REVIEW_PYTHON=python3.13 bash docs/reviews/2026-11-17-sorcerer-storm/regenerate.sh
```

This rebuilds the review exporters against current production and writes only
this review's fixture data and sheets. It does not generate art, replace native
assets or operate Herdr. `prepare_card.rs` separately records the one-time
normalization used for the native asset. Do not regenerate historical approved
study images.

Recommendation: **needs review** for production animation and native appearance.
