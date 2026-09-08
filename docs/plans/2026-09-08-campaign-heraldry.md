# Campaign heraldry and deferred visual review

The Questmancer asked to progress the agreed sequence and handle visual sign-off
last on 2026-09-08. The four-row sidebar is applied with a backup; the Librarian,
pilot and sidebar retain pending visual acceptance. Release preparation proceeds
locally, without tagging or publishing the historical 0.1.8 source.

## Bounded heraldry slice

Derive a small crest from the existing workspace key through a versioned,
deterministic hash. Use a named field colour and charge shape; rename, party
status, order and time cannot change it. This is presentation identity, not a
claim of globally unique or persistent identity beyond the workspace key.

Show native-scale pennants below the canonical Hall's two campaign-table adventurer rows,
using the existing table assignment. Each table has four separate pennant slots.
If a table's entire campaign set cannot fit, omit that table's pennants rather
than overpaint them or imply one campaign owns a shared table. Smaller Hall
tiers omit pennants and keep their existing actor/capacity contracts.

Repeat the shape and colour name on the selected adventurer's parchment in
both rooms. Keep the full campaign label and authored portrait. ASCII mode uses
a readable charge abbreviation; ANSI mode uses indexed colours. No new state,
network commands, metadata token, actor target, animation wake or persistence.

Test first: visible parchment regression, then stable projection, shape/colour
variation, no changes to domain state, shared-table capacity, actor separation,
small-viewport omission and actual RGB rendering. Run affected suites, full
verification, release build and packaged-source verification. Capture current
production review assets for the final visual sign-off without calling them
accepted. Release publishing and archive/installer acceptance remain separate.

## Execution receipt

Completed locally on 2026-09-08. The initial missing-crest test and placement
overlap test both failed before their fixes. The final full gate passed 570 Rust
tests across 51 runs and 28 shell tests; release build, packaged-source
verification and diff hygiene pass. The package is below the 10 MiB gate.

The [production sheet and receipt](../design/reviews/2026-09-08-campaign-heraldry/README.md)
record the implemented appearance and checks. All user visual decisions remain
in the [final review queue](../reviews/2026-09-08-final-review.md). There were no
new live-server resources, agent messages, commits, tags or publications.
