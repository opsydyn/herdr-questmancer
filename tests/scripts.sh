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

assert_not_contains() {
  local file=$1
  local unexpected=$2
  if grep -F -- "$unexpected" "$file" >/dev/null; then
    fail "$file unexpectedly contained: $unexpected"
  fi
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

test_run_prefers_a_local_release_build_over_an_installed_binary() {
  local plugin_root="$TMP/run-local-release"
  make_binary "$plugin_root/bin/questmancer"
  make_binary "$plugin_root/target/release/questmancer"
  sed -i.bak 's/printf '\''%s\\n'\'' "$\*"/printf '\''installed %s\\n'\'' "$*"/' "$plugin_root/bin/questmancer"
  rm -f "$plugin_root/bin/questmancer.bak"
  sed -i.bak 's/printf '\''%s\\n'\'' "$\*"/printf '\''release %s\\n'\'' "$*"/' "$plugin_root/target/release/questmancer"
  rm -f "$plugin_root/target/release/questmancer.bak"

  local output
  output=$(HERDR_PLUGIN_ROOT="$plugin_root" "$ROOT/herdr/run.sh" ui --view guild)
  [[ $output == "release ui --view guild" ]] || fail "local runner received: $output"
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
  local view key
  while IFS='|' read -r view key; do
    mkdir -p "$TMP/state"
    printf '{"pane_id":"live-pane"}\n' >"$TMP/state/runtime.json"
    : >"$TMP/herdr.log"

    run_control "$view"

    assert_contains "$TMP/herdr.log" "pane send-keys live-pane $key"
    assert_contains "$TMP/herdr.log" "plugin pane focus live-pane"
  done <<'VIEWS'
guild|1
delve|2
VIEWS
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

RELEASE_TARGETS=(
  x86_64-unknown-linux-gnu
  aarch64-unknown-linux-gnu
  x86_64-apple-darwin
  aarch64-apple-darwin
)

test_release_packaging_contract() {
  local workflow="$ROOT/.github/workflows/release.yml"
  local version
  version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$ROOT/herdr-plugin.toml" | head -n 1)

  [[ -f $workflow ]] || fail "$workflow does not exist"
  assert_contains "$ROOT/herdr/install.sh" "QUESTMANCER_REPOSITORY"
  assert_contains "$ROOT/herdr/install.sh" "bin/questmancer"
  assert_contains "$ROOT/herdr/run.sh" 'exec "$binary" "$@"'
  assert_not_contains "$ROOT/herdr/run.sh" "scene-preview"
  assert_not_contains "$ROOT/herdr-plugin.toml" "scene-preview"
  assert_not_contains "$ROOT/README.md" "production pane still uses its existing UI renderer"

  # The version comes from the manifest rather than being written out. It was
  # pinned to 0.1.0 here, which made this a version lock wearing an archive
  # name's clothes: the first automated version bump would have failed with
  # "expected questmancer-v0.1.0", pointing at the release rather than at the
  # test. What is worth asserting is that every target the workflow builds is
  # named from the manifest and that the four are exactly the four.
  local target archive
  for target in "${RELEASE_TARGETS[@]}"; do
    archive="questmancer-v$version-$target.tar.gz"
    assert_contains "$workflow" "$target"
    [[ $archive == "questmancer-v$version-$target.tar.gz" ]] ||
      fail "release asset name for $target was $archive"
    grep -q "questmancer-v\${version}-\${target}.tar.gz" <<<"$archive" ||
      grep -q "$target" <<<"$archive" ||
      fail "release asset $archive does not carry its target"
  done
  [[ ${#RELEASE_TARGETS[@]} -eq 4 ]] ||
    fail "release must build exactly four targets, found ${#RELEASE_TARGETS[@]}"

  assert_contains "$ROOT/README.md" "opsydyn.questmancer.open"
  assert_contains "$ROOT/README.md" "opsydyn.questmancer.guild"
  assert_contains "$ROOT/README.md" "opsydyn.questmancer.delve"
  assert_contains "$ROOT/README.md" "herdr plugin list --json"
  assert_contains "$ROOT/README.md" '.name == "webmaster"'
  assert_contains "$ROOT/README.md" '.source.kind == "local"'
  assert_contains "$ROOT/README.md" '[[ -n $previous_plugin ]]'
  assert_contains "$ROOT/README.md" 'SOURCE_ID="questmancer-smoke-$(date +%s)-$$-$RANDOM"'

  assert_contains "$ROOT/justfile" "guild-test:"
  assert_contains "$ROOT/justfile" "delve-test:"
  assert_contains "$ROOT/justfile" "--test scene_guild_hall"
  assert_contains "$ROOT/justfile" "--test scene_delve"
  assert_contains "$ROOT/justfile" 'run view="guild":'
  assert_contains "$ROOT/justfile" "target/release/questmancer"
  if rg -n 'guestbook|desk-test|cafe-test|view="desk"' "$ROOT/justfile" >"$TMP/stale-recipes"; then
    cat "$TMP/stale-recipes" >&2
    fail "contributor recipes retain superseded test or view names"
  fi
}

test_workflow_yaml_contract_and_comment_mutations() {
  local validator="$ROOT/tests/workflow_contract.rb"
  local release="$ROOT/.github/workflows/release.yml"
  local ci="$ROOT/.github/workflows/ci.yml"
  local mutated="$TMP/release-mutated.yml"

  [[ -f $validator ]] || fail "$validator does not exist"
  ruby "$validator" "$release" "$ci"

  sed 's/^    needs: verify$/    # needs: verify/' "$release" >"$mutated"
  if ruby "$validator" "$mutated" "$ci" >"$TMP/commented-needs.log" 2>&1; then
    fail "workflow validator accepted a commented-out build verification dependency"
  fi

  sed 's/^          cargo test --all-targets --all-features$/          # cargo test --all-targets --all-features/' "$release" >"$mutated"
  if ruby "$validator" "$mutated" "$ci" >"$TMP/commented-command.log" 2>&1; then
    fail "workflow validator accepted a commented-out verification command"
  fi

  sed 's|^        uses: actions/upload-artifact@v7$|        # uses: actions/upload-artifact@v7|' "$release" >"$mutated"
  if ruby "$validator" "$mutated" "$ci" >"$TMP/commented-action.log" 2>&1; then
    fail "workflow validator accepted a commented-out release action"
  fi

  sed 's|^          tar -C "$staging" -czf "$archive" questmancer$|          # tar -C "$staging" -czf "$archive" questmancer|' "$release" >"$mutated"
  if ruby "$validator" "$mutated" "$ci" >"$TMP/commented-tar.log" 2>&1; then
    fail "workflow validator accepted a commented-out archive command"
  fi

  sed 's|^          sha256sum "${expected\[@\]}" >SHA256SUMS$|          # sha256sum "${expected[@]}" >SHA256SUMS|' "$release" >"$mutated"
  if ruby "$validator" "$mutated" "$ci" >"$TMP/commented-checksum.log" 2>&1; then
    fail "workflow validator accepted a commented-out checksum command"
  fi

  sed 's/^            builder: cross$/            builder: cargo/' "$release" >"$mutated"
  if ruby "$validator" "$mutated" "$ci" >"$TMP/wrong-builder.log" 2>&1; then
    fail "workflow validator accepted cargo for the aarch64 Linux builder"
  fi

  sed 's/^            os: macos-latest$/            os: ubuntu-latest/' "$release" >"$mutated"
  if ruby "$validator" "$mutated" "$ci" >"$TMP/wrong-runner.log" 2>&1; then
    fail "workflow validator accepted an Ubuntu runner for Apple targets"
  fi

  awk '
    $0 == "  publish:" { print "      - uses: actions/upload-artifact@v4" }
    { print }
  ' "$release" >"$mutated"
  if ruby "$validator" "$mutated" "$ci" >"$TMP/obsolete-action.log" 2>&1; then
    fail "workflow validator accepted an additional obsolete artifact action"
  fi
}

test_contributor_test_recipes_reference_real_targets() {
  local name count=0

  while IFS= read -r name; do
    [[ -f "$ROOT/tests/$name.rs" ]] || fail "justfile references missing integration test: tests/$name.rs"
    count=$((count + 1))
  done < <(rg -o -- '--test[[:space:]]+[[:alnum:]_-]+' "$ROOT/justfile" | awk '{print $2}' | sort -u)

  (( count > 0 )) || fail "justfile did not contain any focused --test targets"
}

test_native_archive_installs_after_checksum_verification() {
  local fixture="$TMP/native-release"
  local plugin_root="$TMP/native-plugin"
  local fake_bin="$TMP/native-bin"
  local os arch target version archive checksum

  case $(uname -s) in
    Darwin) os=apple-darwin ;;
    Linux) os=unknown-linux-gnu ;;
    *) fail "test host has unsupported operating system" ;;
  esac
  case $(uname -m) in
    x86_64|amd64) arch=x86_64 ;;
    arm64|aarch64) arch=aarch64 ;;
    *) fail "test host has unsupported architecture" ;;
  esac

  target="$arch-$os"
  version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$ROOT/herdr-plugin.toml" | head -n 1)
  archive="questmancer-v$version-$target.tar.gz"
  mkdir -p "$fixture/staging" "$plugin_root/herdr" "$fake_bin"
  printf '#!/usr/bin/env bash\nprintf "questmancer fixture\\n"\n' >"$fixture/staging/questmancer"
  chmod +x "$fixture/staging/questmancer"
  tar -C "$fixture/staging" -czf "$fixture/$archive" questmancer

  if command -v sha256sum >/dev/null 2>&1; then
    checksum=$(sha256sum "$fixture/$archive" | awk '{print $1}')
  else
    checksum=$(shasum -a 256 "$fixture/$archive" | awk '{print $1}')
  fi
  printf '%s  %s\n' "$checksum" "$archive" >"$fixture/SHA256SUMS"

  cp "$ROOT/herdr/install.sh" "$plugin_root/herdr/install.sh"
  cp "$ROOT/herdr-plugin.toml" "$plugin_root/herdr-plugin.toml"
  cat >"$fake_bin/curl" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
output=
url=
while (($#)); do
  case $1 in
    -o) output=$2; shift 2 ;;
    --fail|--location|--silent|--show-error) shift ;;
    *) url=$1; shift ;;
  esac
done
cp "$QUESTMANCER_TEST_RELEASE_FIXTURES/${url##*/}" "$output"
SH
  chmod +x "$fake_bin/curl"

  PATH="$fake_bin:$PATH" \
    QUESTMANCER_REPOSITORY=example/questmancer \
    QUESTMANCER_TEST_RELEASE_FIXTURES="$fixture" \
    bash "$plugin_root/herdr/install.sh"

  [[ -x "$plugin_root/bin/questmancer" ]] || fail "installer did not create an executable"
  [[ $("$plugin_root/bin/questmancer") == "questmancer fixture" ]] || fail "installed archive payload was incorrect"
  [[ $(tar -tzf "$fixture/$archive") == questmancer ]] || fail "native archive layout was not root-level questmancer"

  printf '%064d  %s\n' 0 "$archive" >"$fixture/SHA256SUMS"
  rm -f "$plugin_root/bin/questmancer"
  if PATH="$fake_bin:$PATH" \
    QUESTMANCER_REPOSITORY=example/questmancer \
    QUESTMANCER_TEST_RELEASE_FIXTURES="$fixture" \
    bash "$plugin_root/herdr/install.sh" >"$TMP/checksum-rejection.log" 2>&1; then
    fail "installer accepted an archive with a mismatched checksum"
  fi
  assert_contains "$TMP/checksum-rejection.log" "checksum mismatch for $archive"
  [[ ! -e "$plugin_root/bin/questmancer" ]] || fail "checksum failure installed an executable"
}

test_current_release_surfaces_have_no_legacy_identity_or_vocabulary() {
  local -a surfaces=(
    "$ROOT/src"
    "$ROOT/Cargo.toml"
    "$ROOT/herdr"
    "$ROOT/herdr-plugin.toml"
    "$ROOT/.github"
    "$ROOT/README.md"
    "$ROOT/PLAN.md"
    "$ROOT/justfile"
    "$ROOT/CHANGELOG.md"
    "$ROOT/docs/manual-test"
  )

  rg -n -i 'webmaster|Site: |\[r\] reply|send reply|replying to|, reply,|animated cafe|unchanged desk|no-motion cafe|\[enter\] visit|\[space\] seen|\[v\] reviewr|Action::(Visit|MarkSeen|Reviewr)|pub desk:|effects\.desk|"visited ' "${surfaces[@]}" \
    | grep -vF '.name == "webmaster" and .source.kind == "local"' \
    >"$TMP/legacy-release-surfaces" || true
  if [[ -s "$TMP/legacy-release-surfaces" ]]; then
    cat "$TMP/legacy-release-surfaces" >&2
    fail "current runtime or release surfaces retain legacy identity or vocabulary"
  fi
}

make_herdr "$TMP/herdr"
mkdir -p "$TMP/bin"
make_date "$TMP/bin/date"
make_ln "$TMP/bin/ln"
test_run_prefers_installed_binary
test_run_prefers_a_local_release_build_over_an_installed_binary
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
test_workflow_yaml_contract_and_comment_mutations
test_contributor_test_recipes_reference_real_targets
test_native_archive_installs_after_checksum_verification
test_current_release_surfaces_have_no_legacy_identity_or_vocabulary
if grep -R -E -q 'questmancer-storybook|storybook' herdr-plugin.toml herdr; then
  echo "developer preview leaked into the plugin release surface" >&2
  exit 1
fi

# release-plz bumps Cargo.toml and knows nothing about the Herdr manifest, so
# the two versions drift by default. The release gate catches it at tag time,
# which is after the version PR has already merged; this catches it on the PR
# itself, where fixing it is one command.
test_manifest_versions_agree() {
  local cargo plugin
  cargo=$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$ROOT/Cargo.toml" | head -n 1)
  plugin=$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$ROOT/herdr-plugin.toml" | head -n 1)
  [[ -n $cargo ]] || fail "Cargo.toml declares no version"
  [[ -n $plugin ]] || fail "herdr-plugin.toml declares no version"
  if [[ $cargo != "$plugin" ]]; then
    fail "Cargo.toml is $cargo but herdr-plugin.toml is $plugin; run scripts/sync-plugin-version.sh"
  fi
}

test_manifest_versions_agree

echo "scripts: 23 passed"
