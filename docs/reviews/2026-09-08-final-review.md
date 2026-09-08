# Final visual and release review queue

The Questmancer explicitly requested that visual sign-off happen last on
2026-09-08. Engineering has progressed without converting automated checks or
fixture exports into visual acceptance. Nothing in this queue is marked approved.

| Review | Current evidence | Remaining decision |
| --- | --- | --- |
| Four-row sidebar and class colours | Applied local config, checker pass; [comparison](../design/reviews/2026-09-08-sidebar-comparison/README.md) | Native 24/32-column readability, state/vigil emphasis, ASCII and empty bag |
| Librarian | [Production review](../design/reviews/2026-09-05-librarian/README.md) | World proportions, Ledger fallback/native card and discoverable help access |
| Wizard/Ranger/Barbarian pilot | [Pose and playback pack](../design/reviews/2026-09-05-party-pilot/README.md); [later card fallback correction](../design/reviews/2026-09-05-card-fallbacks/README.md) | Working, counsel and spoils loop in both rooms; old pilot card sheet is superseded |
| Campaign heraldry | [Current production sheet](../design/reviews/2026-09-08-campaign-heraldry/README.md) | Crest readability and table association; shared-table density; matching card identity |
| Room and native transport acceptance | [Guarded procedure](../manual-test/questmancer-scene-preview.md) | Both rooms, small sizes, native portraits, real resting/completion observations |

The source-linked release binary is rebuilt. Close/reopen Questmancer when
starting it to load that binary. This does not start, stop or reload a shared
Herdr server automatically. Re-baseline all live IDs for native tests.

## Release preparation

The current 0.1.9 [candidate qualification](2026-09-08-release-candidate/README.md)
supersedes the earlier dirty 0.1.8 package preparation. The
[consolidated review pack](../design/reviews/2026-09-08-consolidated/README.md)
contains fresh production exports, including updated room and card context.

Before publishing: complete native acceptance, then build the four archives,
verify checksums and the installer. Existing `v0.1.8` is a historical tag and
must not be moved or mistaken for this source. Current release workflow repairs
remain local; live dispatch/registry settings remain unverified.

No tag, remote workflow dispatch, GitHub release or crates.io publication has
been performed. Keepsake illustrations, cat reactions, Chronicle chapters and
further class rituals remain later slices.
