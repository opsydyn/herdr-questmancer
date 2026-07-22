#!/usr/bin/env ruby
# frozen_string_literal: true

require "set"
require "yaml"

def abort_contract(message)
  warn "FAIL: workflow contract: #{message}"
  exit 1
end

def load_workflow(path)
  document = YAML.safe_load(
    File.read(path),
    permitted_classes: [],
    permitted_symbols: [],
    aliases: true,
    filename: path
  )
  abort_contract("#{path} root is not a mapping") unless document.is_a?(Hash)

  jobs = document["jobs"]
  abort_contract("#{path} jobs is not a mapping") unless jobs.is_a?(Hash)

  jobs
rescue Errno::ENOENT, Psych::SyntaxError => error
  abort_contract(error.message)
end

def job(jobs, name)
  value = jobs[name]
  abort_contract("missing active job #{name}") unless value.is_a?(Hash)

  value
end

def steps(job_value, job_name)
  value = job_value["steps"]
  unless value.is_a?(Array) && value.all? { |step| step.is_a?(Hash) }
    abort_contract("#{job_name}.steps is not a sequence of mappings")
  end

  value
end

def active_runs(job_value, job_name)
  steps(job_value, job_name).each_with_object([]) do |step, runs|
    runs << step["run"] if step["run"].is_a?(String)
  end
end

def active_uses(job_value, job_name)
  steps(job_value, job_name).each_with_object([]) do |step, uses|
    uses << step["uses"] if step["uses"].is_a?(String)
  end
end

def require_run_line(job_value, job_name, expected)
  present = active_runs(job_value, job_name).any? do |run|
    run.lines.any? { |line| line.strip == expected }
  end
  abort_contract("#{job_name} has no active run line: #{expected}") unless present
end

def require_exact_keys(mapping, label, expected)
  actual = mapping.keys
  return if actual.length == expected.length && actual.to_set == expected.to_set

  abort_contract("#{label} keys were #{actual.sort.inspect}, expected #{expected.sort.inspect}")
end

def require_exact_uses(job_value, job_name, expected)
  actual = active_uses(job_value, job_name)
  return if actual.length == expected.length && actual.to_set == expected.to_set

  abort_contract("#{job_name} active uses were #{actual.sort.inspect}, expected #{expected.sort.inspect}")
end

def require_checkout_depth(job_value, job_name)
  checkout = steps(job_value, job_name).find { |step| step["uses"] == "actions/checkout@v5" }
  abort_contract("#{job_name} has no active actions/checkout@v5 step") unless checkout

  with = checkout["with"]
  abort_contract("#{job_name} checkout does not set fetch-depth: 0") unless with.is_a?(Hash) && with["fetch-depth"] == 0
end

def require_text(document, label, expected)
  abort_contract("#{label} is missing: #{expected}") unless document.include?(expected)
end

def forbid_text(document, label, forbidden)
  abort_contract("#{label} contains invalid workflow text: #{forbidden}") if document.include?(forbidden)
end

def require_text_order(document, label, before, after)
  before_index = document.index(before)
  after_index = document.index(after)
  unless before_index && after_index && before_index < after_index
    abort_contract("#{label} must place #{before.inspect} before #{after.inspect}")
  end
end

workflow_paths = if ARGV.empty?
                   [".github/workflows/release.yml", ".github/workflows/ci.yml"]
                 elsif ARGV.length == 2
                   ARGV
                 else
                   abort_contract("usage: workflow_contract.rb [WORKFLOW WORKFLOW]")
                 end

workflows = workflow_paths.map { |path| [path, load_workflow(path)] }
release = workflows.find { |_path, jobs| %w[verify build publish].all? { |name| jobs.key?(name) } }
ci = workflows.find { |_path, jobs| jobs.key?("check") }
abort_contract("could not identify one release workflow and one CI workflow") unless release && ci

_release_path, release_jobs = release
require_exact_keys(release_jobs, "release jobs", %w[verify build publish])
verify = job(release_jobs, "verify")
build = job(release_jobs, "build")
publish = job(release_jobs, "publish")

require_checkout_depth(verify, "verify")
[
  "cargo fmt --all --check",
  "cargo clippy --all-targets --all-features -- -D warnings",
  "cargo test --all-targets --all-features",
  "bash tests/scripts.sh",
  "bash -n herdr/install.sh herdr/run.sh herdr/control.sh",
  "cargo build --release",
  'git diff --check "${base_sha}"...HEAD'
].each { |command| require_run_line(verify, "verify", command) }

abort_contract("build must need exactly verify") unless build["needs"] == "verify"
abort_contract("publish must need exactly build") unless publish["needs"] == "build"

require_exact_uses(
  verify,
  "verify",
  ["actions/checkout@v5", "dtolnay/rust-toolchain@1.90.0", "Swatinem/rust-cache@v2"]
)
require_exact_uses(
  build,
  "build",
  [
    "actions/checkout@v5",
    "dtolnay/rust-toolchain@1.90.0",
    "Swatinem/rust-cache@v2",
    "taiki-e/install-action@v2",
    "actions/upload-artifact@v7"
  ]
)
require_exact_uses(
  publish,
  "publish",
  ["actions/checkout@v5", "actions/download-artifact@v8", "softprops/action-gh-release@v3"]
)

matrix = build.dig("strategy", "matrix", "include")
abort_contract("build matrix include is not a sequence") unless matrix.is_a?(Array)
tuples = matrix.each_with_object([]) do |entry, values|
  values << [entry["target"], entry["os"], entry["builder"]] if entry.is_a?(Hash)
end.to_set
expected_tuples = Set[
  ["x86_64-unknown-linux-gnu", "ubuntu-latest", "cargo"],
  ["aarch64-unknown-linux-gnu", "ubuntu-latest", "cross"],
  ["x86_64-apple-darwin", "macos-latest", "cargo"],
  ["aarch64-apple-darwin", "macos-latest", "cargo"]
]
unless tuples == expected_tuples && matrix.length == 4
  abort_contract("build matrix tuples differ from the four exact target, runner, and builder tuples")
end

require_run_line(build, "build", 'archive="questmancer-v${version}-${target}.tar.gz"')
require_run_line(build, "build", 'tar -C "$staging" -czf "$archive" questmancer')
require_run_line(publish, "publish", 'sha256sum "${expected[@]}" >SHA256SUMS')

_ci_path, ci_jobs = ci
require_exact_keys(ci_jobs, "CI jobs", ["check"])
check = job(ci_jobs, "check")
require_checkout_depth(check, "check")
require_exact_uses(
  check,
  "check",
  ["actions/checkout@v5", "dtolnay/rust-toolchain@1.90.0", "Swatinem/rust-cache@v2"]
)
[
  "cargo fmt --all --check",
  "cargo clippy --all-targets --all-features -- -D warnings",
  "cargo test --all-targets --all-features",
  "bash tests/scripts.sh",
  "bash -n herdr/install.sh herdr/run.sh herdr/control.sh",
  "cargo build --release",
  'git diff --check "${base_sha}"...HEAD'
].each { |command| require_run_line(check, "check", command) }

readme = File.read("README.md")
manual = File.read("docs/manual-test/questmancer-0.1.0.md")
plan = File.read("docs/superpowers/plans/2026-07-17-questmancer-great-room.md")

[
  "cargo build --release\nherdr plugin link .\nherdr plugin action invoke opsydyn.questmancer.open",
  "opsydyn.questmancer.guild",
  "opsydyn.questmancer.delve",
  "just storybook",
  "feature-gated Storybook",
  "Guild Hall",
  "Delve",
  "selection rune",
  "contextual parchment overlays",
  "counsel",
  "search",
  "scrying",
  "twenty-five fixed production stories",
  "Librarian's Ledger",
  "persistent Librarian"
].each { |expected| require_text(readme, "README", expected) }

[
  'TEST_CHECKOUT="$(pwd -P)"',
  'REGISTRATION_SOURCE_ROOT_RAW=',
  'REGISTRATION_SOURCE_ROOT=',
  'test "$candidate_root" != "/"',
  'test -d "$candidate_root"',
  '.result.plugins[]?',
  'select(.plugin_id == "opsydyn.questmancer")',
  '.plugin_root',
  '.result.snapshot.panes[]?',
  '.result.snapshot.tabs[]?',
  '.result.snapshot.focused_pane_id',
  '.result.snapshot.focused_tab_id',
  'test "$REGISTRATION_SOURCE_ROOT" = "$TEST_CHECKOUT"',
  "BASELINE_FOCUS_PANE_ID=",
  "BASELINE_FOCUS_TAB_ID=",
  "PREEXISTING_LINK=0",
  "PREEXISTING_MANAGED_PANE_ID=",
  'test -z "$PREEXISTING_MANAGED_PANE_ID"',
  "BASELINE_LABELLED_MANAGED_PANE_IDS=",
  ".label? | strings | ascii_downcase | contains(\"questmancer\")",
  "TEST_CREATED_LINK=0",
  "TEST_CREATED_MANAGED_PANE=0",
  "MANAGED_PANE_IS_TEST_OWNED=0",
  "refresh_managed_pane_ownership() {",
  "POST_REOPEN_SNAPSHOT=",
  "Exactly one candidate-owned current runtime pane must exist after reopen.",
  "TEST_CREATED_TAB=0",
  "TEST_CREATED_PANE=0",
  "LIVE_TESTS_PERMITTED=0",
  "DISPOSABLE_IDENTITY_VERIFIED=0",
  'if test "$DISPOSABLE_IDENTITY_VERIFIED" = 1; then',
  "Skip directly to restoration after any disposable identity mismatch.",
  "All live rows are BLOCKED unless `LIVE_TESTS_PERMITTED=1`.",
  "git status --short --branch",
  "cargo build --release",
  "herdr plugin action invoke opsydyn.questmancer.open",
  "herdr plugin action invoke opsydyn.questmancer.close",
  "herdr pane report-agent",
  "herdr pane release-agent",
  'CURRENT_PANE_JSON="$(herdr pane current 2>/dev/null || true)"',
  "TAB_ID_FROM_RESPONSE=\"$(jq -er '.result.tab.tab_id",
  "CURRENT_TAB_ID=\"$(jq -er '.result.pane.tab_id",
  "PANE_ID=\"$(jq -er '.result.pane.pane_id",
  "80x24",
  "Herdr 0.7.4 cannot synthesize `done`",
  'herdr pane release-agent "$PANE_ID"',
  'herdr tab close "$TAB_ID"',
  "PRE_UNLINK_PLUGIN_LIST=",
  "PRE_UNLINK_PLUGIN_MATCH_COUNT=",
  "PRE_UNLINK_REGISTRATION_SOURCE_ROOT=",
  "Revalidate the registration immediately before unlinking.",
  'herdr plugin unlink opsydyn.questmancer',
  'herdr tab focus "$BASELINE_FOCUS_TAB_ID"',
  "FINAL_REGISTRATION_SOURCE_ROOT=",
  "Final baseline comparison",
  "PASS",
  "FAIL",
  "BLOCKED"
].each { |expected| require_text(manual, "guarded manual test", expected) }

require_text_order(
  manual,
  "guarded manual test",
  'if test "$DISPOSABLE_IDENTITY_VERIFIED" = 1; then',
  'herdr pane report-agent "$PANE_ID"'
)

[
  "## Post-merge candidate acceptance — pending",
  "The merged checkout commit and plugin registration root must match exactly.",
  "Do not repoint or unlink a protected registration.",
  "- [ ] Candidate open and singleton behaviour",
  "- [ ] Real `done` transition observed"
].each { |expected| require_text(manual, "post-merge candidate acceptance", expected) }

[
  "- [x] The Guild Hall reads as one inhabited Great Room rather than bordered panels.",
  "- [x] Standard and 4096-case property suites pass without excessive rejection.",
  "- [x] Format, Clippy, all-feature tests, script tests, property stress, release build, and workflow contract pass.",
  "- [ ] Guarded live candidate open, interactions, persistence, and screenshots pass.",
  "- [ ] Real `done` transition is observed in live post-merge acceptance.",
  "- [ ] Manual-test resources are cleaned up and the original Herdr environment is restored."
].each { |expected| require_text(plan, "Great Room final acceptance", expected) }

forbid_text(
  plan,
  "Great Room final acceptance",
  "Format, Clippy, all-feature tests, script tests, release build, and guarded Herdr checks pass."
)

[
  ".result.panes",
  ".result.tabs",
  ".source_root?",
  ".source_path?",
  "herdr tab current",
  'herdr pane focus "$BASELINE_FOCUS_PANE_ID"',
  'herdr pane close "$PANE_ID"',
  "herdr pane read --source",
  ".label // .title",
  "BASELINE_MANAGED_PANE_ID"
].each { |forbidden| forbid_text(manual, "guarded manual test", forbidden) }

puts "workflow contracts: valid"
