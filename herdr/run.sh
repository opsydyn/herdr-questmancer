#!/usr/bin/env bash
set -euo pipefail

ROOT=${HERDR_PLUGIN_ROOT:-$(cd "$(dirname "$0")/.." && pwd)}

for binary in \
  "$ROOT/bin/herdr-webmaster" \
  "$ROOT/target/release/herdr-webmaster" \
  "$ROOT/target/debug/herdr-webmaster"
do
  if [[ -x $binary ]]; then
    if [[ ${1:-} == "ui" && $# -eq 1 ]]; then
      case ${WEBMASTER_INITIAL_VIEW:-} in
        desk|cafe) exec "$binary" ui --view "$WEBMASTER_INITIAL_VIEW" ;;
      esac
    fi
    exec "$binary" "$@"
  fi
done

echo "webmaster binary not found; run 'cargo build' for a linked plugin" >&2
exit 1
