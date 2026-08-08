#!/usr/bin/env bash
# Copies Cargo.toml's version into herdr-plugin.toml.
#
# release-plz owns the version and bumps Cargo.toml alone. Herdr reads its own
# manifest, and `herdr/install.sh` builds the release archive name from it, so a
# drift between the two produces a plugin that downloads an archive no release
# published. This is the one command that fixes it.
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$ROOT/Cargo.toml" | head -n 1)
[[ -n $version ]] || { echo "Cargo.toml declares no version" >&2; exit 1; }

tmp=$(mktemp)
awk -v v="$version" '
  !done && /^version = "/ { print "version = \"" v "\""; done = 1; next }
  { print }
' "$ROOT/herdr-plugin.toml" >"$tmp"
mv "$tmp" "$ROOT/herdr-plugin.toml"

echo "herdr-plugin.toml set to $version"
