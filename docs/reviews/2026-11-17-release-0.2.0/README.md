# Questmancer 0.2.0 release qualification

- **result:** complete
- **claim:** Published 0.2.0 from clean `11ddd02`, with four verified archives, a checked native installer and deployed feature/Pi documentation.
- **evidence:** 841 Rust tests / 67 runs, 28 shell tests, property/build/site/package gates, successful GitHub workflows, archive hashes and local/installed PTY checks; see [verification.json](verification.json).
- **remaining uncertainty:** Native authoring, exhaustive native visual/input acceptance, other-platform execution and registry publication remain separate. The stale-banner follow-up remains open.
- **next:** Manually `/reload` existing Pi sessions when ready; retain the synthetic party for native review and promote the next bounded acceptance slice.

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

Recommendation: **needs follow-up** for the separate native acceptance and integration-banner work.

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
added. Clean-source packaging and remote distribution subsequently passed below.

Staged diff checking exposed preserved blank lines/right-padding in old and new
raw logs and PTY grids. The initial local commit command continued after that
check failed; no push occurred. A narrow `.gitattributes` rule now excludes only
review `.log` captures and `restart-pty-screen.txt` grids from whitespace lint.
Their bytes remain intact; source and prose keep the normal check. The complete
upstream-to-candidate diff passed before publication and in GitHub CI.

## Published distribution

[Questmancer v0.2.0](https://github.com/opsydyn/herdr-questmancer/releases/tag/v0.2.0)
is published from `11ddd0262284b11e1c5f2db9735f63b085b5fa38`, preserving both local
feature commits and the upstream 0.1.11 history. The merge tree exactly retained
the qualified candidate. Main and the new annotated tag were pushed atomically;
no existing tag was moved.

- [Release](https://github.com/opsydyn/herdr-questmancer/actions/runs/44179949669):
  tagged-source gate, four platform builds and publication passed; crates.io skipped.
- [CI](https://github.com/opsydyn/herdr-questmancer/actions/runs/44179950349) and
  [release automation](https://github.com/opsydyn/herdr-questmancer/actions/runs/44179950468): passed.
- [Site deployment](https://github.com/opsydyn/herdr-questmancer/actions/runs/44179950416):
  passed. The public URL returned HTTP 200 with the new features, Pi commands,
  Herdr prerequisite and 64-adventurer documentation present.

The clean source package is **5,182,077 bytes**, 306 files, SHA-256
`9e14ff7130d7eae0e030aa5538e027b5fbd615744ffabe18a65970b50700e481`.
Its verified build and packaged Storybook check passed. Review packs, launch site
and reference art are excluded from the crate; runtime code/assets are present.

All four downloaded release archives match `SHA256SUMS`, contain one root-level
executable and identify the expected OS/CPU. The published body exactly matches
the curated 0.2.0 changelog. The actual `herdr/install.sh` installed into private
temporary storage, and the macOS ARM64 executable reports `questmancer 0.2.0`.
That downloaded binary also passes native, unsupported-graphics, missing-geometry
and native-interrupt PTY scenarios, restoring raw mode, alternate screen and mouse
capture in every case. Other platform binaries were inspected, not executed.

All nine prior shared-session terminal identities remain present. No shared
server restart/reload, plugin relink, existing agent input or focus command was
issued. Current focus differs from the earlier Pi-trial baseline and was left
alone. The synthetic party remains open. Private release scratch and downloaded
artifacts are retained outside the checkout; no credentials are published.
