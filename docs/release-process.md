# Questmancer release process

Three things cut a release, and each owns one job.

| Stage | Owner | Produces |
| --- | --- | --- |
| Version | `release-plz` on `main` | a release pull request, then the `vX.Y.Z` tag |
| Binaries | `release.yml`, explicitly dispatched after tagging | four archives and `SHA256SUMS` on a GitHub release |
| Registry | `release.yml`, gated | a crates.io publish when `PUBLISH_TO_CRATES=true`; live setting unverified |

## Distribution status — 2026-09-06

Read-only GitHub API checks still find only published `v0.1.0` and
[`v0.1.3`](https://github.com/opsydyn/herdr-questmancer/releases/tag/v0.1.3).
The latest published release is `v0.1.3`. All four archives now have verified
SHA-256 matches, one root-level executable, and the expected platform/CPU.
The current installer also installed the published `0.1.3` manifest into a
temporary directory and its macOS ARM64 binary reported `questmancer 0.1.3`.
The other platform binaries were inspected, not executed on this host.

At that inspection, the checkout and public `main` declared Questmancer `0.1.8`, and a `v0.1.8`
tag exists at `a594845f040d54dcc2f175524d4720e8008f8773`, but no matching release
is published. The tagged commit is historical; it does not contain the current
candidate work. The verified `0.1.3` install does not establish a working
`0.1.8` install. Use the documented source-link workflow for this checkout.

The [2026-08-11 tagging run](https://github.com/opsydyn/herdr-questmancer/actions/runs/31478542616)
reported `releases_created: true` for `v0.1.8`, with an empty
`RELEASE_PLZ_TOKEN` condition. No later `Release` run exists. This confirms the
old tag-to-build handoff failure. Reading current repository secret names and
variables returned HTTP 403, so their present settings and the registry gate
remain unverified. No credentials or repository settings were changed.

The workflow repair below remains local. No new tag, workflow run,
GitHub release or registry publication was created during this pass.
The [local preflight receipt](reviews/2026-09-06-release-readiness/README.md)
records verification, packaging, archive checks and remaining acceptance gates.

## Local candidate — 2026-09-08

The current candidate is `0.1.9`, requiring Herdr `0.9.0` / protocol `22`.
Its [qualification receipt](reviews/2026-09-08-release-candidate/README.md)
and [consolidated visual review](design/reviews/2026-09-08-consolidated/README.md)
track local checks separately from native approval and published distribution.
The historical `v0.1.8` tag remains untouched.

## Cutting one

1. Write the change into `CHANGELOG.md` under `## [Unreleased]` as part of the
   work itself. Notes are hand-written here: release-plz's generated section is
   a list of commit subjects, and with no tags in the repository the first run
   produced one covering all three hundred commits, reintroducing the
   project's pre-rename identity. `release-plz` opens or updates a release
   pull request with the version bump alone.
2. Nothing. The workflow syncs `herdr-plugin.toml` onto the release branch
   itself. release-plz bumps `Cargo.toml` and knows nothing about the Herdr
   manifest, which `herdr/install.sh` uses to build the archive name, so drift
   there 404s every `herdr plugin install`. This was a manual step and was
   missed twice — fixed by hand once, broken again by the next bump. A manual
   step inside an automated pipeline is a step that eventually does not happen.
   `tests/scripts.sh` still fails when the two disagree, as a backstop.
3. Merge the release pull request after the release candidate is approved.
   The `release` job tags the commit with `GITHUB_TOKEN`, validates the emitted
   single-package stable release, and explicitly dispatches `release.yml` for
   that tag. The dispatch step has `actions: write`; a failed dispatch fails
   the job and remains recoverable with the manual command below.
4. `release.yml` builds four targets, checks the packaged crate, verifies the
   tag matches both manifests, takes the release body from the first section of
   `CHANGELOG.md` that has content, and publishes the GitHub release with
   checksums. The "with content" part matters: a changelog conventionally keeps
   an empty `## [Unreleased]` at the top between releases, and releasing that
   would ship a blank body.

## A tag without a release breaks installation

`herdr plugin install opsydyn/herdr-questmancer` is how a Herdr user installs
this plugin. Herdr fetches the repository and runs `herdr/install.sh`, which
builds an archive name from `herdr-plugin.toml` and downloads it from the
matching GitHub release. So a version in that manifest with no published
release is not an untidy loose end — it is a broken install for every user,
returning 404 from the download.

This was observed for v0.1.1 and v0.1.2: release-plz tagged them and no build
followed. The current distribution check above records which releases are
published now; a historical tag is not a current installer target.

## Explicit dispatch closes the tag-to-build gap

GitHub suppresses ordinary workflow-triggering events created with
`GITHUB_TOKEN`, including tag pushes. It explicitly permits
`workflow_dispatch` and `repository_dispatch` to start another workflow.
See [GitHub's trigger documentation](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/trigger-a-workflow).

`release-plz.yml` now tags using `GITHUB_TOKEN` and runs
`scripts/dispatch-tagged-release.sh` only when `releases_created` is true.
The helper accepts exactly one `questmancer` release with a stable `X.Y.Z`
version and a matching `vX.Y.Z` tag. Malformed, ambiguous, prerelease and
mismatched outputs fail before calling GitHub. The release job alone gains
`actions: write`; tag creation continues to use `contents: write`.

The helper requests `release.yml` from `main`, passing the emitted tag. Every
release job checks out that tag and the existing full gate validates both
manifests, the packaged crate, four archives and checksums before publishing.
Dispatch acknowledgement is not evidence that those later jobs succeeded.

A personal access token is no longer required or read by this workflow.
Existing repository secrets were not removed. Using `GITHUB_TOKEN` for the tag
also avoids a second build arriving from a personal-token push alongside the
explicit dispatch.

For an approved recovery of an existing tag whose build never ran or failed:

```bash
gh workflow run release.yml --repo opsydyn/herdr-questmancer --ref main -f tag=v0.1.8
```

That command publishes the historical `v0.1.8` source after its gates pass; it
cannot publish the current uncommitted work. Do not move an existing tag to
include newer changes. The eventual clean candidate needs its own reviewed
version and tag. This recovery command has not been run during preparation.

## Why the changelog is not generated

release-plz prepends a generated section rather than respecting a curated one.
The commit messages in this repository carry the reasoning behind each change;
reducing them to subject lines loses exactly the part worth keeping. The
release body is therefore taken from `CHANGELOG.md` itself, which is also what
stops a release shipping with an empty body — the failure that generation
normally exists to prevent.

## Why the split

release-plz can create the GitHub release itself, and does not here. Only the
build job holds the four platform archives, so a release created earlier would
be an empty one that the binaries had to catch up with — briefly advertising a
version nobody could install.

## crates.io

The publish job is written and contracted, and runs only when the repository
variable `PUBLISH_TO_CRATES` is `true`. Its live value was not inspected in
this documentation pass. Registry setup still needs verification of:

- a `CARGO_REGISTRY_TOKEN` secret in a `crates-io` GitHub environment;
- confirmation that the crate name is free.

It runs last on purpose. A publish cannot be undone — a version may be yanked
but never replaced — so nothing reaches the registry until the artefacts people
actually install exist. Left ungated with no token, a release would go red
*after* the GitHub release had already succeeded, which reads as a failed
release that in fact shipped.

The crate is kept publishable by the release gate, which runs `cargo package`
and fails over 10 MiB. That check exists because the crate once measured 32 MiB:
`src/assets` held sixteen 1536x1024 source illustrations totalling 26 MiB that
nothing referenced, and root screenshots rode along besides. Source art now
lives in `reference-art/`, and `Cargo.toml` excludes repository material.

## What the guards cover

- `tests/scripts.sh` — both documents lead with `herdr plugin install` and
  neither presents `cargo install` as a way to install the plugin; the two
  manifest versions agree, and release archive
  names are derived from the manifest rather than pinned. That test used to
  hardcode `0.1.0`, so the first automated bump would have failed it with a
  message pointing at the release rather than at the test.
- `tests/workflow_contract.rb` — job graph, action pins, the crates.io gate,
  tag checkout in every release job, and the explicit release-plz dispatch,
  conditional and token permissions.
- Dispatch behaviour tests use a fake `gh` executable: the exact tag is sent
  once; invalid outputs never reach GitHub; a rejected dispatch stays failed.
