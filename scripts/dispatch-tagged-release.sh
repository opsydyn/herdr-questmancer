#!/usr/bin/env bash
# Tag pushes made with GITHUB_TOKEN do not start release.yml. Explicit
# workflow_dispatch is a supported exception and uses the same scoped token.
set -euo pipefail

: "${GITHUB_REPOSITORY:?GitHub repository is required}"
: "${GH_TOKEN:?GitHub workflow token is required}"
: "${RELEASES:?release-plz output is required}"

tag=$(jq -er '
  select(type == "array" and length == 1)
  | .[0]
  | select(.package_name == "questmancer")
  | select((.version | type) == "string" and (.tag | type) == "string")
  | select(.version | test("\\A(0|[1-9][0-9]*)\\.(0|[1-9][0-9]*)\\.(0|[1-9][0-9]*)\\z"))
  | select(.tag == "v" + .version)
  | .tag
' <<<"$RELEASES") || {
  echo 'Expected exactly one matching stable Questmancer release.' >&2
  exit 1
}

gh workflow run release.yml --repo "$GITHUB_REPOSITORY" --ref main -f tag="$tag"
printf 'Requested archive build for %s\n' "$tag"
