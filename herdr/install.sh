#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
VERSION=$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$ROOT/herdr-plugin.toml" | head -n 1)
REPOSITORY=${HERDR_WEBMASTER_REPOSITORY:-opsydyn/herdr-webmaster}

case $(uname -s) in
  Darwin) os=apple-darwin ;;
  Linux) os=unknown-linux-gnu ;;
  *) echo "unsupported operating system: $(uname -s)" >&2; exit 1 ;;
esac

case $(uname -m) in
  x86_64|amd64) arch=x86_64 ;;
  arm64|aarch64) arch=aarch64 ;;
  *) echo "unsupported architecture: $(uname -m)" >&2; exit 1 ;;
esac

target="$arch-$os"
archive="herdr-webmaster-v$VERSION-$target.tar.gz"
base_url="https://github.com/$REPOSITORY/releases/download/v$VERSION"
temporary=$(mktemp -d)
trap 'rm -rf "$temporary"' EXIT

curl --fail --location --silent --show-error "$base_url/$archive" -o "$temporary/$archive"
curl --fail --location --silent --show-error "$base_url/SHA256SUMS" -o "$temporary/SHA256SUMS"

expected=$(awk -v name="$archive" '$2 == name || $2 == "*" name { print $1 }' "$temporary/SHA256SUMS")
[[ -n $expected ]] || { echo "checksum missing for $archive" >&2; exit 1; }

if command -v sha256sum >/dev/null 2>&1; then
  actual=$(sha256sum "$temporary/$archive" | awk '{print $1}')
else
  actual=$(shasum -a 256 "$temporary/$archive" | awk '{print $1}')
fi
[[ $actual == "$expected" ]] || { echo "checksum mismatch for $archive" >&2; exit 1; }

mkdir -p "$ROOT/bin"
tar -xzf "$temporary/$archive" -C "$temporary"
install -m 0755 "$temporary/herdr-webmaster" "$ROOT/bin/herdr-webmaster"

