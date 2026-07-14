#!/usr/bin/env bash
set -euo pipefail

ROOT=${HERDR_PLUGIN_ROOT:-$(cd "$(dirname "$0")/.." && pwd)}

for binary in \
  "$ROOT/bin/herdr-webmaster" \
  "$ROOT/target/release/herdr-webmaster" \
  "$ROOT/target/debug/herdr-webmaster"
do
  if [[ -x $binary ]]; then
    if [[ ${1:-} == "ui" && -n ${WEBMASTER_INITIAL_VIEW:-} && $# -eq 1 ]]; then
      exec "$binary" ui --view "$WEBMASTER_INITIAL_VIEW"
    fi
    exec "$binary" "$@"
  fi
done

echo "webmaster binary not found; run 'cargo build' for a linked plugin" >&2
exit 1

