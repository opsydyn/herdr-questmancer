# Questmancer 0.2.0 release qualification

- **result:** complete
- **claim:** Local engineering qualification and release documentation are complete for 0.2.0. Publication is authorized and pending the clean-source/package and remote workflow gates.
- **evidence:** 841 Rust tests / 67 runs, 28 shell tests, property/build/site gates, eight PTY scene scenarios, two exact pixel comparisons and owned-authoring restart; see [verification.json](verification.json).
- **remaining uncertainty:** Native authoring and exhaustive native visual/input acceptance remain separate; remote workflow, archives and installer checks are pending.
- **next:** Verify, commit, merge the remote 0.1.11 version history, qualify the clean source, push/tag and check publication.

## Release scope

This release includes the previously committed durable Board/Table and optional
Projects inventory, the local owned-work backend and authoring UI, Herdr 0.9.4+
compatibility, Bramble Sage Druid/Storm Channeler Sorcerer/Sunward Chaplain Cleric
and Librarian art, fitted Hall/Delve rooms, profiles over fitted worlds, and
automatic stable density through 64 adventurers. It documents the separately
installed supported Pi hook. No new execution/delegation authority is introduced.

The latest fetched upstream is `35d7aa7`, published as 0.1.11. Its three remote
commits change only version metadata and will be preserved by a merge. Version
0.2.0 is the next feature release; no historical tag will be moved.

## Existing evidence

- [Owned authoring](../2026-10-03-owned-authoring/README.md): serial owned store,
  acknowledged projections, recovery and automated PTY input/restart.
- [Automatic density](../2026-11-17-automatic-density/README.md): 841 Rust tests,
  28 shell tests, eight PTY scenarios, 42 exports and two exact native payload
  comparisons for the preceding binary. Those hashes are not the version-bumped
  release binary's identity.
- [Pi integration](../2026-11-17-pi-integration/README.md): real native identity,
  working/resting capture, reload/restart and owned-resource cleanup. Resting
  does not establish success or earn XP. Existing Pi sessions were not reloaded.

## Publication boundaries

User authorization covers committing, pushing, tagging and the existing GitHub
archive/site workflow. Registry publication remains disabled. The retained
synthetic dogfood party and shared Herdr server must remain open and unchanged.
Private credentials, raw session identity and private qualification scratch
files are excluded from this repository. Review assets remain tracked but are
excluded from the existing 10 MiB crate surface.

Native input/compositing acceptance, the known potentially stale integration
banner and other-platform binary execution remain follow-ups. This release
must not relabel automated rendering evidence as native approval.

Recommendation: **hold** until the fresh release gates pass.

## Fresh local qualification

Host-captured checks pass on the version-bumped source:

```sh
just verify
just property-test 4096
cargo build --release --locked --offline
cargo build --features storybook --bin questmancer-storybook --locked --offline
cargo doc --all-features --no-deps --locked --offline
(cd site && bun run verify)
git diff --check
```

The property gate passes 151 tests across eight runs with `PROPTEST_CASES=4096`.
Existing scene/profile/transport and owned-authoring PTY harnesses were rerun
with outputs redirected into private release scratch storage, preserving the
historical review receipts. Eight scene scenarios exit cleanly and restore the
terminal; both 32-person 800x464 RGBA uploads match the source projections exactly.
Owned authoring acknowledges revisions 0–5, retains the answer after restart,
leaves authority bytes unchanged and removes its temporary resources. No Herdr
resource is created by those harnesses.

Self-review covered store/command ownership and shutdown, capability fallback and
profile masking, version/archive agreement, registry gating and test provenance.
This is self-review supported by host checks, not independent review. Credential
signature scanning of changed/untracked files found no matches; the largest
new review file is below 9 MB. No raw Pi session identity or credentials are
added. Clean-source packaging and remote distribution remain the next gates.
