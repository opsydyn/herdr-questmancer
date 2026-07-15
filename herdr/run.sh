#!/usr/bin/env bash
set -euo pipefail

ROOT=${HERDR_PLUGIN_ROOT:-$(cd "$(dirname "$0")/.." && pwd)}
INITIAL_VIEW_ENV=QUESTMANCER_INITIAL_VIEW

for binary in \
  "$ROOT/bin/questmancer" \
  "$ROOT/target/release/questmancer" \
  "$ROOT/target/debug/questmancer"
do
  if [[ -x $binary ]]; then
    if [[ ${1:-} == "ui" && $# -eq 1 ]]; then
      case ${!INITIAL_VIEW_ENV:-} in
        guild|delve) exec "$binary" ui --view "${!INITIAL_VIEW_ENV}" ;;
      esac
    fi
    exec "$binary" "$@"
  fi
done

echo "questmancer binary not found; run 'cargo build' for a linked plugin" >&2
exit 1
