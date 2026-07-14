#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

assert_contains() {
  local file=$1
  local expected=$2
  grep -F -- "$expected" "$file" >/dev/null || fail "$file did not contain: $expected"
}

make_binary() {
  local path=$1
  mkdir -p "$(dirname "$path")"
  cat >"$path" <<'SH'
#!/usr/bin/env bash
printf '%s\n' "$*"
SH
  chmod +x "$path"
}

test_run_prefers_installed_binary() {
  local plugin_root="$TMP/run-installed"
  make_binary "$plugin_root/bin/herdr-webmaster"

  local output
  output=$(HERDR_PLUGIN_ROOT="$plugin_root" "$ROOT/herdr/run.sh" ui --view cafe)
  [[ $output == "ui --view cafe" ]] || fail "installed runner received: $output"
}

test_run_falls_back_to_debug_binary() {
  local plugin_root="$TMP/run-debug"
  make_binary "$plugin_root/target/debug/herdr-webmaster"

  local output
  output=$(HERDR_PLUGIN_ROOT="$plugin_root" "$ROOT/herdr/run.sh" ui --view desk)
  [[ $output == "ui --view desk" ]] || fail "debug runner received: $output"
}

make_herdr() {
  local path=$1
  cat >"$path" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >>"$FAKE_HERDR_LOG"
case "$*" in
  "pane get live-pane") exit 0 ;;
  "pane get"*) exit 1 ;;
  "plugin pane open"*) printf '{"result":{"plugin_pane":{"pane":{"pane_id":"new-pane"}}}}\n' ;;
esac
SH
  chmod +x "$path"
}

run_control() {
  HERDR_BIN_PATH="$TMP/herdr" \
  HERDR_PLUGIN_ID="opsydyn.webmaster" \
  HERDR_PLUGIN_STATE_DIR="$TMP/state" \
  HERDR_PLUGIN_ROOT="$ROOT" \
  FAKE_HERDR_LOG="$TMP/herdr.log" \
    "$ROOT/herdr/control.sh" "$@"
}

test_open_creates_one_cafe_pane() {
  rm -rf "$TMP/state"
  : >"$TMP/herdr.log"

  run_control cafe

  assert_contains "$TMP/herdr.log" "plugin pane open --plugin opsydyn.webmaster --entrypoint webmaster --placement tab --env WEBMASTER_INITIAL_VIEW=cafe --focus"
  assert_contains "$TMP/state/runtime.json" '"pane_id":"new-pane"'
}

test_open_focuses_a_live_existing_pane() {
  mkdir -p "$TMP/state"
  printf '{"pane_id":"live-pane"}\n' >"$TMP/state/runtime.json"
  : >"$TMP/herdr.log"

  run_control open

  assert_contains "$TMP/herdr.log" "plugin pane focus live-pane"
  if grep -F "plugin pane open" "$TMP/herdr.log" >/dev/null; then
    fail "open created a duplicate pane"
  fi
}

test_close_uses_plain_pane_close_and_clears_state() {
  mkdir -p "$TMP/state"
  printf '{"pane_id":"live-pane"}\n' >"$TMP/state/runtime.json"
  : >"$TMP/herdr.log"

  run_control close

  assert_contains "$TMP/herdr.log" "pane close live-pane"
  [[ ! -e "$TMP/state/runtime.json" ]] || fail "close left runtime state behind"
}

test_stale_state_is_replaced() {
  mkdir -p "$TMP/state"
  printf '{"pane_id":"stale-pane"}\n' >"$TMP/state/runtime.json"
  : >"$TMP/herdr.log"

  run_control desk

  assert_contains "$TMP/herdr.log" "pane get stale-pane"
  assert_contains "$TMP/herdr.log" "plugin pane open --plugin opsydyn.webmaster --entrypoint webmaster --placement tab --env WEBMASTER_INITIAL_VIEW=desk --focus"
  assert_contains "$TMP/state/runtime.json" '"pane_id":"new-pane"'
}

test_busy_control_lock_refuses_a_second_action() {
  mkdir -p "$TMP/state/control.lock"
  : >"$TMP/herdr.log"

  if WEBMASTER_LOCK_ATTEMPTS=1 run_control open 2>"$TMP/busy.err"; then
    fail "a second control action acquired an existing lock"
  fi

  assert_contains "$TMP/busy.err" "webmaster control is busy"
  [[ ! -s "$TMP/herdr.log" ]] || fail "busy control action called Herdr"
  rmdir "$TMP/state/control.lock"
}

make_herdr "$TMP/herdr"
test_run_prefers_installed_binary
test_run_falls_back_to_debug_binary
test_open_creates_one_cafe_pane
test_open_focuses_a_live_existing_pane
test_close_uses_plain_pane_close_and_clears_state
test_stale_state_is_replaced
test_busy_control_lock_refuses_a_second_action

echo "scripts: 7 passed"
