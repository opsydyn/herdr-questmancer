# Questmancer release process

Three things cut a release, and each owns one job.

| Stage | Owner | Produces |
| --- | --- | --- |
| Version | `release-plz` on `main` | a release pull request, then the `vX.Y.Z` tag |
| Binaries | `release.yml` on the tag | four archives and `SHA256SUMS` on a GitHub release |
| Registry | `release.yml`, gated | a crates.io publish — **currently off** |

## Cutting one

1. Write the change into `CHANGELOG.md` under `## [Unreleased]` as part of the
   work itself. Notes are hand-written here: release-plz's generated section is
   a list of commit subjects, and with no tags in the repository the first run
   produced one covering all three hundred commits, reintroducing the
   project's pre-rename identity. `release-plz` opens or updates a release
   pull request with the version bump alone.
2. **Run `scripts/sync-plugin-version.sh` on that branch and push.** release-plz
   bumps `Cargo.toml` and knows nothing about `herdr-plugin.toml`, which Herdr
   reads and `herdr/install.sh` uses to build the archive name. `tests/scripts.sh`
   fails while they disagree, so the pull request shows red until it is done.
3. Merge the release pull request. The `release` job tags the merged commit.
   That job was missing at first: `release-pr` opens the version pull request
   and nothing tags it, so the first merged one bumped the version and stopped,
   and `release.yml` — which triggers on the tag — never ran.
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

That is the state v0.1.1 and v0.1.2 are in: release-plz tagged them, nothing
built them, and the manifest points at a release that does not exist. It is
also why the release job below matters more than it looks.

## The tag has to come from a personal access token

GitHub does not start workflows from events created with `GITHUB_TOKEN`; it
blocks that to stop a workflow triggering itself. release-plz tags with exactly
that token, so a merged release pull request creates the tag and `release.yml`
— which triggers on tags — never runs. The first v0.1.0 tag was created and
nothing built from it.

Give the `release` job a personal access token with `contents: write` as
`GITHUB_TOKEN` and tags start triggering the release properly.

Until then, and for re-running a release whose build failed, `release.yml`
accepts a manual dispatch with the tag to release:

```bash
gh workflow run release.yml -f tag=v0.1.0
```

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

The publish job is written, contracted and gated off behind the repository
variable `PUBLISH_TO_CRATES`. Two things are outstanding:

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
  and the release-plz settings this document depends on.
