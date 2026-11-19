# Questmancer 0.2.5 release qualification

- **Result** · complete
- **Claim** · GitHub published `v0.2.5` with the Kitty startup fix and all four owned-quest changes.
- **Evidence** · Four downloaded archives match their published checksums. The stock Herdr installer and the downloaded macOS ARM64 binary passed the Kitty and durability checks. See the [verification receipt](verification.json).
- **Remaining uncertainty** · The other platform binaries were inspected. The Ghostty screenshots show the source-linked candidate, while the published binary was verified using an attached Herdr client in a test PTY.
- **Next** · Use the [published plugin upgrade workflow](../../release-process.md#published-025-release--2026-11-19).

## Release scope

| Area | What ships |
| --- | --- |
| Kitty startup | The normal Herdr `open` action uses the attached client's measured pixels to fit the Hall and Delve and show native portraits. Ghostty confirmed all three in the existing window on the source-linked candidate. [Ghostty evidence](../2026-11-19-attached-kitty-open/README.md) |
| Quest lifecycle | Press `t` to activate, complete or reopen an owned quest. The saved state survives restart and a successful state change upgrades older stores to v3. [Lifecycle verification](../2026-11-19-owned-quest-lifecycle/README.md) |
| Prerequisites | Press `p` to add or remove a prerequisite between owned quests in the same campaign. The writer rejects cycles. The first saved link edit upgrades older stores to v4. [Prerequisite verification](../2026-11-19-owned-quest-prerequisites/README.md) |
| Assignment | Press `y` to set, replace or clear the recorded assignee. The first saved assignment edit upgrades older stores to v5. [Assignment verification](../2026-11-19-owned-quest-assignment/README.md) |
| Live association | Press `l` to link an owned quest to the selected verified Herdr adventurer and `L` to clear the link. Observe checks the pane occupant. The association lasts for the current connection and is separate from the saved assignee. [Live association verification](../2026-11-19-live-owned-quest-association/README.md) |

Existing v1 and v2 stores open directly. Keep a copy before the first save
that upgrades the format if you may need to return to an earlier Questmancer
version. See the [owned-work guide](../../manual-test/owned-work-authoring.md).

## Release source and local verification

[Release PR #17](https://github.com/opsydyn/herdr-questmancer/pull/17)
updated the Cargo manifest, Herdr manifest and lockfile together. The
`v0.2.5` tag points to `d824597d04120eb460fc12561c11cb92fc6a71de`, whose
tree matches merge commit `be93e47a613936d939d1feb18c5ce67b20a83848`.
The version files are the only changes from the reviewed feature commit
`aed7cdf`.

On macOS ARM64, `just release-verify` passed 887 Rust tests in 68 runs,
29 shell checks, formatting, Clippy, shell syntax and the release build.
`just property-test 4096` passed 158 tests across eight suites on the feature
commit before the version bump. [Main CI](https://github.com/opsydyn/herdr-questmancer/actions/runs/44628743989)

## Published artifacts

[GitHub published v0.2.5](https://github.com/opsydyn/herdr-questmancer/releases/tag/v0.2.5)
on 2026-11-19 at 22:38 UTC. The tagged-source gate, four platform builds and
publication passed. The packaged crate was 3,336,529 bytes. The published notes
match the curated changelog, and the crates.io job was skipped.
[Release workflow](https://github.com/opsydyn/herdr-questmancer/actions/runs/44629363116),
[post-merge CI](https://github.com/opsydyn/herdr-questmancer/actions/runs/44629195441).

Each downloaded archive matched `SHA256SUMS` and the GitHub asset digest.
Each contained one root-level executable for its declared OS and CPU.
Stock Herdr 0.9.5 installed `v0.2.5` from GitHub into a disposable instance.
The executable reported `questmancer 0.2.5` and matched the downloaded
macOS ARM64 archive byte for byte.

## Published macOS ARM64 runtime

The normal `open` command ran from a Herdr shell after its client attached.
The new 98×38 pane reported 784×684 pixels with zero client resize events.
The Hall and Delve used Kitty from their first renders with all three synthetic
adventurers in view. The expanded Adventurer card contained 360 Kitty cells
inside its borders. A second `open` reused the same pane and process. The test
stopped its disposable Herdr server.

The downloaded executable also passed the existing PTY probes for lifecycle
save and reopen, prerequisite cycle rejection and removal, assignee save,
replace and clear, and connection-scoped association and revocation. The
probes reopened saved state after restart and showed earlier readers refusing
upgraded v3, v4 and v5 stores while preserving the files.

**Recommendation** · merge the publication record.
