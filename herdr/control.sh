#!/usr/bin/env bash
set -euo pipefail

HERDR=${HERDR_BIN_PATH:-herdr}
PLUGIN_ID=${HERDR_PLUGIN_ID:-opsydyn.questmancer}
ENTRYPOINT=guild-hall
INITIAL_VIEW_ENV=QUESTMANCER_INITIAL_VIEW
STATE_DIR=${HERDR_PLUGIN_STATE_DIR:?HERDR_PLUGIN_STATE_DIR is required}
RUNTIME="$STATE_DIR/runtime.json"
LOCK="$STATE_DIR/control.lock"

mkdir -p "$STATE_DIR"

locked=false
for _ in $(seq 1 "${QUESTMANCER_LOCK_ATTEMPTS:-50}"); do
  if mkdir "$LOCK" 2>/dev/null; then
    locked=true
    trap 'rmdir "$LOCK" 2>/dev/null || true' EXIT
    break
  fi
  sleep 0.05
done

if [[ $locked != true ]]; then
  echo "questmancer control is busy" >&2
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
  if ! ln "$temporary" "$RUNTIME" 2>/dev/null && [[ ! -e $RUNTIME ]]; then
    rm -f "$temporary"
    echo "questmancer could not publish runtime registration" >&2
    return 1
  fi
  rm -f "$temporary"
}

open_pane() {
  local initial_view=$1
  local switch_existing=${2:-false}
  local pane_id response
  if pane_id=$(live_pane_id); then
    if [[ $switch_existing == true ]]; then
      case $initial_view in
        guild) "$HERDR" pane send-keys "$pane_id" 1 >/dev/null ;;
        delve) "$HERDR" pane send-keys "$pane_id" 2 >/dev/null ;;
      esac
    fi
    "$HERDR" plugin pane focus "$pane_id" >/dev/null
    return
  fi

  if [[ $initial_view == default ]]; then
    response=$("$HERDR" plugin pane open \
      --plugin "$PLUGIN_ID" \
      --entrypoint "$ENTRYPOINT" \
      --placement tab \
      --focus)
  else
    response=$("$HERDR" plugin pane open \
      --plugin "$PLUGIN_ID" \
      --entrypoint "$ENTRYPOINT" \
      --placement tab \
      --env "$INITIAL_VIEW_ENV=$initial_view" \
      --focus)
  fi
  pane_id=$(printf '%s\n' "$response" | sed -n 's/.*"pane_id"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1)
  if [[ -z $pane_id ]]; then
    echo "questmancer could not read the new pane id" >&2
    exit 1
  fi
  write_runtime "$pane_id" "$initial_view"
}

close_pane() {
  local pane_id
  if pane_id=$(runtime_pane_id); then
    if ! "$HERDR" pane get "$pane_id" >/dev/null 2>&1; then
      rm -f "$RUNTIME"
      return
    fi
    "$HERDR" pane close "$pane_id" >/dev/null
  fi
  rm -f "$RUNTIME"
}

case ${1:-} in
  open) open_pane default false ;;
  guild) open_pane guild true ;;
  delve) open_pane delve true ;;
  close) close_pane ;;
  toggle)
    if pane_id=$(live_pane_id); then
      if [[ ${HERDR_PANE_ID:-} == "$pane_id" ]]; then
        close_pane
      else
        "$HERDR" plugin pane focus "$pane_id" >/dev/null
      fi
    else
      open_pane default false
    fi
    ;;
  *)
    echo "usage: control.sh open|close|toggle|guild|delve" >&2
    exit 2
    ;;
esac
