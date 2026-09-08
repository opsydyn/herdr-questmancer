# Herdr upgrade and local release preflight

Date: 2026-09-06. Checkout: `efcd87d` plus the preserved earlier edits and this
authorised slice. This is local preflight; no clean release commit, tag,
workflow dispatch or publication was created.

The [assessment](../../plans/2026-09-06-herdr-082-release-readiness.md) was
written before upgrading. The main consequences are to retain Herdr's live
state authority and explicit counsel, keep experimental graphics work limited
to optional portraits, and treat Windows support as a separate port.

## Installed Herdr and protocol evidence

The official stable Herdr 0.8.2 macOS ARM64 binary is installed at
`~/.local/bin/herdr`. Its SHA-256 matches the release asset:
`a5d4f4d504d8b309c91f811050559300faba31258425f53c50852fc96f6ae574`.
The previous 0.8.0 executable is retained at
`/tmp/questmancer-herdr-082.2GmTg0/herdr-0.8.0-backup` for this session;
temporary storage is not a durable backup. Final `herdr status` reports stable
0.8.2 / protocol 20, no running default server and no restart needed.
`herdr session list` still lists only the stopped default session.

Questmancer's manifest now requires 0.8.2 and its supervisor accepts exactly
protocol 20. A focused test first failed with expected 19 / actual 20, then
passed after the change. Unsupported 19 and 21 are rejected at ping, initial
snapshot and topology refresh. The [real captures](../../../tests/fixtures/herdr/0.8.2/README.md)
retain upstream field shapes and document the two normalised runtime values.

An isolated headless Herdr server exercised the current production client and
supervisor against an owned plain shell pane. The checks covered ping,
snapshot, managed-pane exclusion, bounded output, owned-pane focus, five
registered actions, pane/campaign metadata, urgency set/clear, subscription,
and synthetic blocked/idle/unknown/working events through the production
adapter and reducer. All four published sidebar configuration examples also
passed Herdr 0.8.2's actual `config check`.

The test used separate configuration, state and cache directories. It released
each synthetic source, removed its own plugin link, and stopped only its own
server; the final server exited zero and its socket disappeared. Server logs
contain no errors. Two earlier harness attempts needed a corrected executable
path and test-local registration; both also cleaned up their resources. No
shared registration, real adventurer, user counsel or application UI was used.

## Verification and packaging

The final `just verify` passed **555 Rust tests across 50 runs and 28 shell
tests**, formatting, Clippy with warnings denied, and shell syntax. The new
dispatcher also passes explicit Bash syntax checking. `cargo build --release`
passed; its executable reports `questmancer 0.1.8`.

Packaging uses `cargo package --locked --allow-dirty` because earlier
authorised work remains uncommitted. Cargo builds the extracted crate, and the
archive is checked against the 10 MiB limit and the current source/asset file
set. The [machine-readable receipt](receipt.json) records the exact artifact
size and digest. That archive predates only the generated receipt itself;
the current source, authored assets, changelog and this review are included.
This does not replace `cargo package --locked` from the eventual clean commit.

Raw command logs and the isolated probe harness remain under
`/tmp/questmancer-herdr-082.2GmTg0` and `/tmp/questmancer-herdr-082-*.log`.
The receipt records log digests; these temporary paths may disappear.

## Distribution audit and local correction

The [published 0.1.3 release](https://github.com/opsydyn/herdr-questmancer/releases/tag/v0.1.3)
has all four expected archives: x86_64 and aarch64 for macOS and Linux GNU.
Every checksum matches `SHA256SUMS`; each archive contains only one regular,
executable, root-level `questmancer` file with the expected architecture.
The current installer, paired with the actual published 0.1.3 manifest, also
downloaded, verified and installed its macOS ARM64 binary in a temporary
directory. Only that native binary was executed.

Public `main` still declares 0.1.8, whose historical tag is
`a594845f040d54dcc2f175524d4720e8008f8773`, but no matching release exists.
The [historical tagging run](https://github.com/opsydyn/herdr-questmancer/actions/runs/31478542616)
created that tag using the workflow token without starting an archive build.
The local fix explicitly dispatches the archive workflow after a validated
single-package stable release. It uses the scoped workflow token and fails
if dispatch fails. Every job, including optional registry publication, checks
out the requested tag. Regression checks first failed on both missing
dispatch and the registry's default-branch checkout, then passed after repair.
Mocked tests verify exact dispatch arguments and reject malformed or ambiguous
outputs before calling GitHub. No live dispatch was performed.

See the [release process](../../release-process.md) for the eventual approved
publication and historical-tag recovery routes. Current repository secret
names and variable settings could not be read with the available GitHub
permissions, so `PUBLISH_TO_CRATES` and registry setup remain unverified.

## Remaining gates

- Manual native Librarian, pilot and room/resize review. Computer Use's terminal
  app restriction still prevents Ghostty automation; the prior PTY receipt is
  executable evidence, not native visual approval.
- Guarded acceptance from the eventual clean release commit, including pane
  lifecycle, persistence and current Guild Hall/Delve captures.
- A reviewed new version/tag containing the current work, followed by the
  actual four-platform build, publication and installation of that version.
- Optional Reviewr and real-agent resting/completion observations when available.

The verified historical installer and synthetic server checks close only their
stated rows. They do not certify native graphics, real-agent completion, the
unpublished 0.1.8 distribution or a future release candidate.
