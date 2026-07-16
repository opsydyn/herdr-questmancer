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
  make_binary "$plugin_root/bin/questmancer"

  local output
  output=$(HERDR_PLUGIN_ROOT="$plugin_root" "$ROOT/herdr/run.sh" ui --view delve)
  [[ $output == "ui --view delve" ]] || fail "installed runner received: $output"
}

test_run_falls_back_to_release_binary() {
  local plugin_root="$TMP/run-release"
  make_binary "$plugin_root/target/release/questmancer"

  local output
  output=$(HERDR_PLUGIN_ROOT="$plugin_root" "$ROOT/herdr/run.sh" ui --view guild)
  [[ $output == "ui --view guild" ]] || fail "release runner received: $output"
}

test_run_falls_back_to_debug_binary() {
  local plugin_root="$TMP/run-debug"
  make_binary "$plugin_root/target/debug/questmancer"

  local output
  output=$(HERDR_PLUGIN_ROOT="$plugin_root" "$ROOT/herdr/run.sh" ui --view guild)
  [[ $output == "ui --view guild" ]] || fail "debug runner received: $output"
}

test_run_maps_only_exact_initial_views() {
  local plugin_root="$TMP/run-env"
  make_binary "$plugin_root/bin/questmancer"

  local output
  output=$(QUESTMANCER_INITIAL_VIEW=delve HERDR_PLUGIN_ROOT="$plugin_root" "$ROOT/herdr/run.sh" ui)
  [[ $output == "ui --view delve" ]] || fail "delve runner received: $output"

  output=$(QUESTMANCER_INITIAL_VIEW=default HERDR_PLUGIN_ROOT="$plugin_root" "$ROOT/herdr/run.sh" ui)
  [[ $output == "ui" ]] || fail "default runner received: $output"
}

make_herdr() {
  local path=$1
  cat >"$path" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >>"$FAKE_HERDR_LOG"
case "$*" in
  "pane get live-pane") exit 0 ;;
  "pane get fail-pane") exit 0 ;;
  "pane get"*) exit 1 ;;
  "pane close fail-pane") exit 1 ;;
  "plugin pane open"*)
    if [[ -n ${FAKE_REGISTER_VIEW:-} ]]; then
      mkdir -p "$HERDR_PLUGIN_STATE_DIR"
      printf '{"pane_id":"new-pane","initial_view":"%s"}\n' "$FAKE_REGISTER_VIEW" \
        >"$HERDR_PLUGIN_STATE_DIR/runtime.json"
    fi
    printf '{"result":{"plugin_pane":{"pane":{"pane_id":"new-pane"}}}}\n'
    ;;
esac
SH
  chmod +x "$path"
}

make_date() {
  local path=$1
  cat >"$path" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
if [[ -n ${FAKE_REGISTER_DURING_DATE:-} ]]; then
  mkdir -p "$HERDR_PLUGIN_STATE_DIR"
  printf '{"pane_id":"new-pane","initial_view":"%s"}\n' "$FAKE_REGISTER_DURING_DATE" \
    >"$HERDR_PLUGIN_STATE_DIR/runtime.json"
fi
exec /bin/date "$@"
SH
  chmod +x "$path"
}

make_ln() {
  local path=$1
  cat >"$path" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
if [[ -n ${FAKE_LN_FAILURE:-} ]]; then
  exit 23
fi
exec /bin/ln "$@"
SH
  chmod +x "$path"
}

run_control() {
  HERDR_BIN_PATH="$TMP/herdr" \
  HERDR_PLUGIN_ID="opsydyn.questmancer" \
  HERDR_PLUGIN_STATE_DIR="$TMP/state" \
  HERDR_PLUGIN_ROOT="$ROOT" \
  FAKE_REGISTER_VIEW="${FAKE_REGISTER_VIEW:-}" \
  FAKE_REGISTER_DURING_DATE="${FAKE_REGISTER_DURING_DATE:-}" \
  FAKE_LN_FAILURE="${FAKE_LN_FAILURE:-}" \
  FAKE_HERDR_LOG="$TMP/herdr.log" \
  PATH="$TMP/bin:$PATH" \
    "$ROOT/herdr/control.sh" "$@"
}

test_open_creates_one_delve_pane() {
  rm -rf "$TMP/state"
  : >"$TMP/herdr.log"

  run_control delve

  assert_contains "$TMP/herdr.log" "plugin pane open --plugin opsydyn.questmancer --entrypoint guild-hall --placement tab --env QUESTMANCER_INITIAL_VIEW=delve --focus"
  assert_contains "$TMP/state/runtime.json" '"pane_id":"new-pane"'
}

test_open_and_closed_toggle_omit_initial_view() {
  for action in open toggle; do
    rm -rf "$TMP/state"
    : >"$TMP/herdr.log"

    run_control "$action"

    assert_contains "$TMP/herdr.log" "plugin pane open --plugin opsydyn.questmancer --entrypoint guild-hall --placement tab --focus"
    if grep -F -- "--env QUESTMANCER_INITIAL_VIEW" "$TMP/herdr.log" >/dev/null; then
      fail "$action passed an explicit initial view"
    fi
  done
}

test_control_does_not_overwrite_runtime_registration() {
  rm -rf "$TMP/state"
  : >"$TMP/herdr.log"

  FAKE_REGISTER_VIEW=delve run_control open

  assert_contains "$TMP/state/runtime.json" '"initial_view":"delve"'
}

test_control_fallback_publication_is_atomic_no_clobber() {
  rm -rf "$TMP/state"
  : >"$TMP/herdr.log"

  FAKE_REGISTER_DURING_DATE=delve run_control open

  assert_contains "$TMP/state/runtime.json" '"initial_view":"delve"'
}

test_control_reports_fallback_publication_failure() {
  rm -rf "$TMP/state"
  : >"$TMP/herdr.log"

  if FAKE_LN_FAILURE=1 run_control open >"$TMP/control.err" 2>&1; then
    fail "control succeeded after fallback publication failed"
  fi

  assert_contains "$TMP/control.err" "could not publish runtime registration"
  [[ ! -e "$TMP/state/runtime.json" ]] || fail "failed publication left runtime state"
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

test_view_actions_switch_a_live_existing_pane() {
  mkdir -p "$TMP/state"
  printf '{"pane_id":"live-pane"}\n' >"$TMP/state/runtime.json"
  : >"$TMP/herdr.log"

  run_control delve

  assert_contains "$TMP/herdr.log" "pane send-keys live-pane 2"
  assert_contains "$TMP/herdr.log" "plugin pane focus live-pane"
}

test_close_uses_plain_pane_close_and_clears_state() {
  mkdir -p "$TMP/state"
  printf '{"pane_id":"live-pane"}\n' >"$TMP/state/runtime.json"
  : >"$TMP/herdr.log"

  run_control close

  assert_contains "$TMP/herdr.log" "pane close live-pane"
  [[ ! -e "$TMP/state/runtime.json" ]] || fail "close left runtime state behind"
}

test_failed_close_preserves_singleton_state() {
  mkdir -p "$TMP/state"
  printf '{"pane_id":"fail-pane"}\n' >"$TMP/state/runtime.json"
  : >"$TMP/herdr.log"

  if run_control close 2>"$TMP/close.err"; then
    fail "failed pane close was reported as successful"
  fi

  [[ -e "$TMP/state/runtime.json" ]] || fail "failed close discarded runtime state"
}

test_stale_state_is_replaced() {
  mkdir -p "$TMP/state"
  printf '{"pane_id":"stale-pane"}\n' >"$TMP/state/runtime.json"
  : >"$TMP/herdr.log"

  run_control guild

  assert_contains "$TMP/herdr.log" "pane get stale-pane"
  assert_contains "$TMP/herdr.log" "plugin pane open --plugin opsydyn.questmancer --entrypoint guild-hall --placement tab --env QUESTMANCER_INITIAL_VIEW=guild --focus"
  assert_contains "$TMP/state/runtime.json" '"pane_id":"new-pane"'
}

test_busy_control_lock_refuses_a_second_action() {
  mkdir -p "$TMP/state/control.lock"
  : >"$TMP/herdr.log"

  if QUESTMANCER_LOCK_ATTEMPTS=1 run_control open 2>"$TMP/busy.err"; then
    fail "a second control action acquired an existing lock"
  fi

  assert_contains "$TMP/busy.err" "questmancer control is busy"
  [[ ! -s "$TMP/herdr.log" ]] || fail "busy control action called Herdr"
  rmdir "$TMP/state/control.lock"
}

test_release_packaging_contract() {
  local workflow="$ROOT/.github/workflows/release.yml"
  local version
  version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$ROOT/herdr-plugin.toml" | head -n 1)

  [[ -f $workflow ]] || fail "$workflow does not exist"
  assert_contains "$workflow" 'archive="questmancer-v${version}-${target}.tar.gz"'
  assert_contains "$workflow" "SHA256SUMS"
  assert_contains "$ROOT/herdr/install.sh" "QUESTMANCER_REPOSITORY"
  assert_contains "$ROOT/herdr/install.sh" "bin/questmancer"

  local target archive expected
  while IFS='|' read -r target expected; do
    archive="questmancer-v$version-$target.tar.gz"
    [[ $archive == "$expected" ]] || fail "release asset was $archive, expected $expected"
    assert_contains "$workflow" "target: $target"
  done <<'TARGETS'
x86_64-unknown-linux-gnu|questmancer-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
aarch64-unknown-linux-gnu|questmancer-v0.1.0-aarch64-unknown-linux-gnu.tar.gz
x86_64-apple-darwin|questmancer-v0.1.0-x86_64-apple-darwin.tar.gz
aarch64-apple-darwin|questmancer-v0.1.0-aarch64-apple-darwin.tar.gz
TARGETS

  assert_contains "$ROOT/README.md" "opsydyn.questmancer.open"
  assert_contains "$ROOT/README.md" "opsydyn.questmancer.guild"
  assert_contains "$ROOT/README.md" "opsydyn.questmancer.delve"
  assert_contains "$ROOT/README.md" "herdr plugin list --json"
  assert_contains "$ROOT/README.md" '.name == "webmaster"'
  assert_contains "$ROOT/README.md" '.source.kind == "local"'
  assert_contains "$ROOT/README.md" '[[ -n $previous_plugin ]]'

  assert_contains "$ROOT/justfile" "guild-test:"
  assert_contains "$ROOT/justfile" "delve-test:"
  assert_contains "$ROOT/justfile" "--test theatre"
  assert_contains "$ROOT/justfile" 'run view="guild":'
  assert_contains "$ROOT/justfile" "target/release/questmancer"
  if rg -n 'guestbook|desk-test|cafe-test|view="desk"' "$ROOT/justfile" >"$TMP/stale-recipes"; then
    cat "$TMP/stale-recipes" >&2
    fail "contributor recipes retain superseded test or view names"
  fi
}

test_current_release_surfaces_have_no_webmaster_identity() {
  local -a surfaces=(
    "$ROOT/Cargo.toml"
    "$ROOT/herdr"
    "$ROOT/herdr-plugin.toml"
    "$ROOT/.github"
    "$ROOT/README.md"
    "$ROOT/justfile"
  )

  if rg -n 'opsydyn\.webmaster|herdr-webmaster|WEBMASTER_INITIAL_VIEW' "${surfaces[@]}" >"$TMP/legacy-release-surfaces"; then
    cat "$TMP/legacy-release-surfaces" >&2
    fail "current release surfaces retain Webmaster identity"
  fi
}

make_herdr "$TMP/herdr"
mkdir -p "$TMP/bin"
make_date "$TMP/bin/date"
make_ln "$TMP/bin/ln"
test_run_prefers_installed_binary
test_run_falls_back_to_release_binary
test_run_falls_back_to_debug_binary
test_run_maps_only_exact_initial_views
test_open_creates_one_delve_pane
test_open_and_closed_toggle_omit_initial_view
test_control_does_not_overwrite_runtime_registration
test_control_fallback_publication_is_atomic_no_clobber
test_control_reports_fallback_publication_failure
test_open_focuses_a_live_existing_pane
test_view_actions_switch_a_live_existing_pane
test_close_uses_plain_pane_close_and_clears_state
test_failed_close_preserves_singleton_state
test_stale_state_is_replaced
test_busy_control_lock_refuses_a_second_action
test_release_packaging_contract
test_current_release_surfaces_have_no_webmaster_identity

echo "scripts: 17 passed"
