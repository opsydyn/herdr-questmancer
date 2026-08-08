# Questmancer release process

Three things cut a release, and each owns one job.

| Stage | Owner | Produces |
| --- | --- | --- |
| Version and notes | `release-plz` on `main` | a release pull request, then the `vX.Y.Z` tag |
| Binaries | `release.yml` on the tag | four archives and `SHA256SUMS` on a GitHub release |
| Registry | `release.yml`, gated | a crates.io publish — **currently off** |

## Cutting one

1. Merge work to `main`. `release-plz` opens or updates a release pull request
   with the version bump and the changelog.
2. **Run `scripts/sync-plugin-version.sh` on that branch and push.** release-plz
   bumps `Cargo.toml` and knows nothing about `herdr-plugin.toml`, which Herdr
   reads and `herdr/install.sh` uses to build the archive name. `tests/scripts.sh`
   fails while they disagree, so the pull request shows red until it is done.
3. Merge the release pull request. release-plz creates the tag.
4. `release.yml` builds four targets, checks the packaged crate, verifies the
   tag matches both manifests, and publishes the GitHub release with checksums.

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

- `tests/scripts.sh` — the two manifest versions agree, and release archive
  names are derived from the manifest rather than pinned. That test used to
  hardcode `0.1.0`, so the first automated bump would have failed it with a
  message pointing at the release rather than at the test.
- `tests/workflow_contract.rb` — job graph, action pins, the crates.io gate,
  and the release-plz settings this document depends on.
