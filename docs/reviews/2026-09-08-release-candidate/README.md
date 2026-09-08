# Questmancer 0.1.9 candidate qualification

2026-09-08. The user requested consolidated visual/native review followed by
clean release qualification. Candidate version 0.1.9 is unused in the checked
remote tags; existing v0.1.8 remains untouched. No release has been published.

## Completed preparation

- Both manifests and Cargo.lock name 0.1.9; Herdr requirement is 0.9.0/protocol 22.
- Release binary built successfully from the candidate source.
- [Fresh visual export index](../../design/reviews/2026-09-08-consolidated/README.md)
  includes Librarian, pilot, current cards, rooms and playback.
- [Actual plugin runtime receipt](isolated-runtime.json): a test-owned Herdr
  0.9 server ran the rebuilt plugin, retained one pane after repeated open,
  received truthful state metadata and processed working/counsel transitions.
  Guild/Delve action logs exited zero. This verifies invocation, not appearance.
  Managed-pane metadata exclusion passed. All created identities, panes, link
  and server were cleaned up, and the user's server remains stopped.

## Local verification

The 0.1.9 preparation passed `just verify` (570 Rust tests across 51 runs,
28 shell tests, formatting, Clippy with warnings denied and shell syntax),
`cargo build --release` and verified `cargo package --allow-dirty`.
The preparation package is 9,616,184 bytes, below the 10 MiB registry limit.

Clean qualification repeats `just verify`, `cargo build --release`, and
`cargo package` without the dirty flag after the local candidate commit.
Its commit-specific logs and receipt are written outside the checkout at
`/tmp/questmancer-019-clean-qualification/` so qualification itself leaves the
candidate clean. Consult that receipt for completion and the exact commit;
preparation results alone do not establish clean qualification.

## Outstanding gates

Visual user approval and native Ghostty/Herdr graphics acceptance remain pending;
Computer Use's prior safety rejection prevents automated native inspection.

A clean local candidate is not a published or fully accepted release. Four-platform
archives, checksums, actual published installer, live workflow dispatch and the
optional registry gate retain their own qualification. No historical tag will
be moved to represent this source.
