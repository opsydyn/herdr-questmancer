# Bramble Sage Druid — 2026-11-17

- result: complete
- claim: The user-selected B direction now supplies all eight Druid ritual frames and the existing personalised card fallback. Engineering qualification passed and the test-owned dogfood UI was refreshed without changing its authoritative work.
- evidence: `verification.json`, `engineering.json`, red/green logs, production sheets/playback and `live-refresh.json`.
- remaining uncertainty: The static direction is approved; the production poses, animation and native appearance await human review. Synthetic refresh is not native visual acceptance or real-agent completion proof.
- next: Review the retained dogfood Druid and production playback. Commit/release and other-class art remain separate work.

## Implemented scope

Only the Druid section of `src/scene/assets/rituals/support_art.rs` changes
production art. It retains the approved B silhouette, turned face, leaf cloak,
full beard, pouch and right-hand living staff on the `16x24` canvas. Feet remain
on row 21 at columns `4,5,6,9,10,11`.

The two working frames move the hand and sprig without moving the head or feet.
The raised counsel gesture settles into a wait. Spoils use a held then placed
seed pouch; resting, unknown and settled completion remain distinct. The
existing 500 ms / 600 ms / 1000 ms timing and three-second completion window
are unchanged. No runtime scheduler or renderer changes are included.

Skin uses existing persona roles. The beard remains class-owned as it was in
the earlier Druid; two small cloth trim pixels reuse the persisted garb role.
This preserves the approved pale beard rather than turning it into a different
hair-colour treatment. The existing card route automatically centres the exact
personalised Working A master in its `24x32` canvas. Native card PNG, roster,
persona generation/assignments and persistence schema are unchanged.

The [original study](../../design/reviews/2026-11-17-druid-study/README.md)
and its images are preserved. Its separate approval record quotes the user's
choice. It remains a historical before/after study; do not regenerate it to
replace the images the user selected.

## Production review

- [Eight poses and saved skin tones](01-production-poses.png)
- [Actual lit Hall and Delve placements](02-production-rooms.png)
- [Actual fallback card and retained roster](03-card-fallback.png)
- [Exact updated canonical Delve fixture](04-canonical-delve.png)
- [Two-frame working playback](working-loop.gif)

These exports use production model, persona, ritual, lighting, half-block and
card overlay paths. Text cards are reconstructed with Menlo. They are fixture
exports, not terminal screenshots. `export.rs` creates no Herdr resource and
writes no application state. `golden.rs` contains the exact canonical snapshot
helpers from `tests/scene_delve.rs` used when inspecting the golden update.

## Qualification

The first focused test failed against the old production master at the expected
approved-silhouette mismatch `(9,0)`. It now passes for light and dark skin, and
checks that the card reuses the same master. The second focused test protects
working head/feet, semantic pose distinction and all-pose foot anchors.

- Focused Druid tests: **2 passed**.
- Affected Storybook-feature suites: **121 tests / 7 runs passed**, including all-class persona, card, animation and room checks.
- Fresh `just verify`: **791 Rust tests / 64 runs**, **28 shell tests**, formatting, Clippy with warnings denied and shell syntax passed.
- `cargo build --release`: passed; candidate SHA-256 is in `verification.json`.
- Review exporters compile with warnings denied. Formatting, script syntax and `git diff --check` pass.

The first affected run correctly rejected the old Delve image hash because the
mixed fixture includes a Druid. The exact new scene was exported and inspected
before updating the expected hash to
`204e1dc77dd98d1df21bb36a5d7c8a609d194b42ac12c484fe289b42343540f8`.
The golden assertion remains in place. Initial and corrected results are
separate; the final full gate is green.

Raw full-gate and release logs are
`/tmp/questmancer-druid-verify-20261117.log` and
`/tmp/questmancer-druid-release-20261117.log`. Export logs are
`/tmp/questmancer-druid-production-export-20261117.log` and
`/tmp/questmancer-druid-production-repeat-20261117.log`.

The source baseline covers 228 existing source/test/script/manifest files.
Only the Druid art file, `tests/archetype_assets.rs` and the inspected golden
assertion in `tests/scene_delve.rs` changed. The other 225 files are byte-identical;
the Cleric/Paladin prefix of the shared art file is also byte-identical. The
new test fixture was extracted from the approved study before implementation.
Other changes in this slice are the art guidance/handoff and review evidence.
All preexisting dirty owned-work/compatibility work was preserved.

## Test-owned live refresh

`live-refresh.json` records reopening only the previously created dogfood pane
after checking its terminal, runtime PID and visible adventurer-card state.
The current release binary is running in that pane and its synthetic Druid is
selected. All four synthetic identities remain. The authoritative owned store
is byte-identical at revision 7. No shared server restart, real-agent input,
plugin relink or cleanup occurred. Native appearance awaits direct observation.

The party remains open in the **Questmancer dogfood** tab. Its current live
resource ledger remains `/private/tmp/qmdogfood-74b_9_02/receipt.json`; historical
IDs must be revalidated before later control or cleanup.

## Regeneration

Use a Pillow-equipped Python and the existing Rust toolchain:

```bash
REVIEW_PYTHON=/path/to/python3 bash docs/reviews/2026-11-17-druid-bramble/regenerate.sh
```

On this host, the existing cached Pillow wheel can be used with Python 3.13:

```bash
PYTHONPATH=/Users/alancurrie/.cache/uv/archive-v0/18AOTubXh2D76Zrz \
REVIEW_PYTHON=/opt/homebrew/bin/python3.13 \
bash docs/reviews/2026-11-17-druid-bramble/regenerate.sh
```

Regeneration creates fixture exports and sheets. A changed source or canonical
test fixture needs renewed inspection and verification hashes. No commit,
installation, publication or other-class artwork is part of this slice.

Recommendation: **needs review** for production visuals and native presentation.
