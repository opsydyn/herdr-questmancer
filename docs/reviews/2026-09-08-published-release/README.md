# Questmancer 0.1.9 published — 2026-09-08

[Release v0.1.9](https://github.com/opsydyn/herdr-questmancer/releases/tag/v0.1.9)
was published at 22:18:41 UTC from clean commit
`98f557d66b2ff939dbc24b1277477d62264456cc`. The Questmancer explicitly authorised
commit, push and release after approving the native portrait restoration and
resolving the counsel report as fixed synthetic-demo state.

## Verified release

- Clean local qualification: 600 Rust tests across 54 runs, 28 shell tests,
  formatting, warning-denied Clippy, release build and verified source package.
- Source crate: 4,970,598 bytes, all 102 production Rust/PNG files match the
  clean commit, and review packs remain excluded.
- Fresh isolated Herdr runtime and owned-resource cleanup passed.
- [Release workflow](https://github.com/opsydyn/herdr-questmancer/actions/runs/34284545689)
  passed the Linux gate, packaged build, version checks and all four builds.
  The normal authenticated tag push triggered it; recovery dispatch was unnecessary.
- Four downloaded archives match `SHA256SUMS`; each contains exactly one
  root-level executable with the expected CPU/platform format.
- The real `herdr plugin install opsydyn/herdr-questmancer --ref v0.1.9 --yes`
  command passed with isolated temporary XDG storage. Its installed macOS ARM64
  executable matches the release archive and reports `questmancer 0.1.9`.
- The GitHub release is public and stable. The crates.io job was skipped;
  registry publication remains separate.

`verification.json` records hashes and receipts. Full logs are retained under
`/tmp/questmancer-019-release-qualification/`. The existing shared plugin link,
server and user panes were not changed by qualification or installer testing.

The user approved current Hall/Delve/sidebar appearance and confirmed native
Artificer, Bard and Librarian restoration. Execution of the other three binaries
on their native hosts, exhaustive terminal resize/native-card coverage,
real-agent resting/completion and optional Reviewr remain separate evidence.

## Phase closure

The approved party-delight and native-regression phase is complete and published.
The recommended next bounded work is a design for Chronicle capture semantics:
reconnect baselines, observed transitions, identity and campaign closure, with
deduplication. It remains unapproved; durable mementos should follow that work.
