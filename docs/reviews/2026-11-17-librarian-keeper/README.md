# Keeper of the Stacks Librarian — 2026-11-17

- result: complete
- claim: The user-selected B world sprite and matching independent Ledger fallback are implemented, engineering-qualified and loaded in the retained dogfood UI.
- evidence: `verification.json`, `engineering.json`, red/green/affected logs, production review sheets and `live-refresh.json`.
- remaining uncertainty: B's world direction is approved; the production fallback and native appearance await human review. Runtime refresh is not visual acceptance.
- next: Review the Librarian in the dogfood Hall and the fallback sheet before approving the production visuals. Commit/release and other artwork remain separate.

## Implemented

Production changes are confined to the palette and two matrices in
`src/scene/assets/librarian.rs`. The world retains the exact approved Keeper
pixels, a `16x24` canvas, row-21 feet and original horizontal foot anchors.
The `24x32` Ledger fallback is independently authored with a larger face,
spectacles, muzzle, fingers and three book spines. It uses the Keeper's
materials without stretching or padding the world sprite.

The native illustration is unchanged. Both static accessor implementations
remain byte-identical: no animation clock, NPC state or renderer hook was
added. Help interaction, complete click regions, party exclusion, Hall tiers,
handbook navigation, saved personas and persistence schema are unchanged.

The [approved study](../../design/reviews/2026-11-17-librarian-study/visual-approval.md)
records the user's `b` choice. Original study images are preserved; their
"current" comparison remains historical rather than being regenerated after
implementation.

## Production review

- [Previous and Keeper art, three backgrounds and colour modes](01-librarian-proportions.png)
- [Canonical/compact Hall and actual Ledger fallback](02-librarian-context.png)

The exporter links the current production library and uses actual asset
accessors, Hall rendering, half-block conversion and Ledger overlay. Text
is reconstructed with Menlo. These are production fixture exports, not native
screenshots. Both truecolour and ANSI-16 are included; reduced colour naturally
loses material distinctions and remains a separate visual review item.

The before images come from the original study's production asset export,
captured before implementation. They are the actual previous world and
independent portrait, not an old padded fallback or a guessed recreation.

## Qualification

The focused approved-world test first failed against the old sprite at the
expected mismatch `(6,1)`. It now compares every world pixel against the
independent approved-study fixture. A second test checks that the larger
fallback retains the Keeper's fur, spectacles, robe, book and page materials.
Existing tests protect dimensions, grounding, independent portrait width,
nonempty fallback and decodable native art.

- Librarian asset suite: **6 passed**.
- Affected Storybook-feature suites: **84 tests / 5 runs passed**.
- Fresh `just verify`: **793 Rust tests / 64 runs**, **28 shell tests**, formatting, Clippy with warnings denied and shell syntax passed.
- Release build: passed; exact SHA-256 is in `verification.json`.
- Review export compiles with warnings denied. Rust review formatting, script syntax and `git diff --check` pass.
- Final repeated production sheets have identical hashes.

The first layout attempt contained an apostrophe quoting error in its caption;
it was fixed before producing the final sheets. No production change was
needed for that export failure. Unlike the direction study, this exporter
uses the original production library only and creates no background clone.

The baseline covers 229 source/test/script/manifest files. Only
`src/scene/assets/librarian.rs` and `tests/librarian_assets.rs` changed; the
remaining 227 are byte-identical, including the Druid and native illustrations.
The new approved-art fixture is `tests/fixtures/art/librarian-keeper-master.json`.
Additional edits are the art guidance, handoff and review evidence. Existing
owned-work/compatibility work and approvals were preserved.

Full-gate/release logs are
`/tmp/questmancer-librarian-keeper-verify-20261117.log` and
`/tmp/questmancer-librarian-keeper-release-20261117.log`. Final export logs are
`/tmp/questmancer-librarian-keeper-export-20261117.log` and
`/tmp/questmancer-librarian-keeper-repeat-20261117.log`.

## Retained dogfood UI

The recorded test-owned UI was checked for matching terminal/runtime identity
and a visible adventurer card before reopening. No authoring form was visible.
It is running the new candidate in the **Questmancer dogfood** tab, left in the
Guild Hall. The authoritative store remains byte-identical at revision 7 and
saved persona records compare equal. All four synthetic identities remain,
and the UI remains outside the adventurer set. The shared server was not
restarted and no real-agent input, counsel, relink or cleanup occurred.

`live-refresh.json` records the new process/build identity. Native appearance
still needs human observation; the Ledger normally displays its unchanged
native illustration on the supported transport, so the new non-native
fallback should also be reviewed in the sheet above.

The live resource ledger remains `/private/tmp/qmdogfood-74b_9_02/receipt.json`.
Revalidate ownership before later control or cleanup. The party is intentionally
left open.

## Regeneration

Use a Pillow-equipped Python:

```bash
REVIEW_PYTHON=/path/to/python3 bash docs/reviews/2026-11-17-librarian-keeper/regenerate.sh
```

The current host can reuse the existing cached Pillow wheel:

```bash
PYTHONPATH=/Users/alancurrie/.cache/uv/archive-v0/18AOTubXh2D76Zrz \
REVIEW_PYTHON=/opt/homebrew/bin/python3.13 \
bash docs/reviews/2026-11-17-librarian-keeper/regenerate.sh
```

Regeneration against changed source requires fresh inspection and evidence
hashes. No commit, installation or publication occurred.

Recommendation: **needs review** for production visuals and native presentation.
