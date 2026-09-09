# Questmancer 0.1.10 published — 2026-09-09

[Release v0.1.10](https://github.com/opsydyn/herdr-questmancer/releases/tag/v0.1.10)
is published from clean commit `9f2f58fcaefcfaafed45b730950be369868fb0b2`.
The user authorised the release after approving Chronicle C1–C3 and requested
updated documentation and screenshots first.

## Documentation and screens

The README, launch-page gallery, changelog and in-app handbook describe qualified
local observations, quiet reconnect baselines, zero-XP observations and compatible
v1/v2 history. The release notes include the downgrade limitation. Both approved
Chronicle images are reused unchanged and labelled production reconstructions;
the [six-screen approval pack](../../design/reviews/2026-09-09-chronicle-capture/README.md)
retains the full review context. The launch-page checks passed, the changed gallery
was inspected in a browser, and the deployed page contains both Chronicle entries.

## Verified release

- `just verify`: 657 Rust tests across 57 runs, 28 shell tests, formatting,
  warning-denied Clippy and script syntax passed before commit; committed content
  was unchanged. GitHub independently passed the full gate on the tagged commit.
- Clean release build and verified source-package build passed. The crate is
  5,010,069 bytes; its VCS identity matches the release commit and all 106 packaged
  production Rust/PNG files match source. Review packs remain excluded.
- A fresh isolated Herdr server with four owned dummy Pi processes passed the
  Chronicle runtime checks. Restart added no history; observations earned zero XP.
  Test panes, plugin link, focus restoration and server cleanup passed.
- [Release workflow](https://github.com/opsydyn/herdr-questmancer/actions/runs/34377266548)
  passed all four Linux/macOS ARM64/x86-64 builds and published the release.
- Four downloaded archives match `SHA256SUMS`; each contains exactly one root-level
  executable with the expected platform and CPU format.
- The actual `herdr plugin install opsydyn/herdr-questmancer --ref v0.1.10 --yes`
  command passed in temporary XDG storage. Its macOS ARM64 executable matches the
  downloaded release and reports `questmancer 0.1.10`.
- The [site deployment](https://github.com/opsydyn/herdr-questmancer/actions/runs/34377266940)
  passed. The crates.io job was skipped; registry publication remains separate.

`verification.json` contains hashes and detailed receipts. Full local logs are
retained under `/tmp/questmancer-0110-*`. The existing shared Herdr registration,
server and user panes were not changed. The temporary site preview was stopped.

## Remaining evidence

C1–C3 and this release are complete. Real-agent completion, live snapshot-only
capture, exhaustive native-card/resize coverage, other binaries on their native
hosts and optional Reviewr remain unverified. The recommended next bounded work
is a guarded real-agent Chronicle acceptance session, if the user promotes it.
