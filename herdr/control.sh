#!/usr/bin/env bash
set -euo pipefail

HERDR=${HERDR_BIN_PATH:-herdr}
PLUGIN_ID=${HERDR_PLUGIN_ID:-opsydyn.webmaster}
STATE_DIR=${HERDR_PLUGIN_STATE_DIR:?HERDR_PLUGIN_STATE_DIR is required}
RUNTIME="$STATE_DIR/runtime.json"
LOCK="$STATE_DIR/control.lock"

mkdir -p "$STATE_DIR"

locked=false
for _ in $(seq 1 "${WEBMASTER_LOCK_ATTEMPTS:-50}"); do
  if mkdir "$LOCK" 2>/dev/null; then
    locked=true
    trap 'rmdir "$LOCK" 2>/dev/null || true' EXIT
    break
  fi
  sleep 0.05
done

if [[ $locked != true ]]; then
  echo "webmaster control is busy" >&2
  exit 1
fi

runtime_pane_id() {
  [[ -f $RUNTIME ]] || return 1
  sed -n 's/.*"pane_id"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$RUNTIME" | head -n 1
}

live_pane_id() {
  local pane_id
  pane_id=$(runtime_pane_id) || return 1
  [[ -n $pane_id ]] || return 1
  if "$HERDR" pane get "$pane_id" >/dev/null 2>&1; then
    printf '%s\n' "$pane_id"
    return 0
  fi
  rm -f "$RUNTIME"
  return 1
}

write_runtime() {
  local pane_id=$1
  local initial_view=$2
  local temporary="$RUNTIME.tmp.$$"
  printf '{"pane_id":"%s","pid":0,"started_at":%s,"initial_view":"%s"}\n' \
    "$pane_id" "$(date +%s)" "$initial_view" >"$temporary"
  mv "$temporary" "$RUNTIME"
}

open_pane() {
  local initial_view=$1
  local pane_id response
  if pane_id=$(live_pane_id); then
    "$HERDR" plugin pane focus "$pane_id" >/dev/null
    return
  fi

  response=$("$HERDR" plugin pane open \
    --plugin "$PLUGIN_ID" \
    --entrypoint webmaster \
    --placement tab \
    --env "WEBMASTER_INITIAL_VIEW=$initial_view" \
    --focus)
  pane_id=$(printf '%s\n' "$response" | sed -n 's/.*"pane_id"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1)
  if [[ -z $pane_id ]]; then
    echo "webmaster could not read the new pane id" >&2
    exit 1
  fi
  write_runtime "$pane_id" "$initial_view"
}

close_pane() {
  local pane_id
  if pane_id=$(runtime_pane_id); then
    "$HERDR" pane close "$pane_id" >/dev/null 2>&1 || true
  fi
  rm -f "$RUNTIME"
}

case ${1:-} in
  open) open_pane desk ;;
  desk) open_pane desk ;;
  cafe) open_pane cafe ;;
  close) close_pane ;;
  toggle)
    if pane_id=$(live_pane_id); then
      if [[ ${HERDR_PANE_ID:-} == "$pane_id" ]]; then
        close_pane
      else
        "$HERDR" plugin pane focus "$pane_id" >/dev/null
      fi
    else
      open_pane desk
    fi
    ;;
  *)
    echo "usage: control.sh open|close|toggle|desk|cafe" >&2
    exit 2
    ;;
esac
