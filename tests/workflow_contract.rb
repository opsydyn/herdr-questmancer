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

def require_run_text(job_value, job_name, expected)
  present = active_runs(job_value, job_name).any? { |run| run.include?(expected) }
  abort_contract("#{job_name} has no active run text: #{expected}") unless present
end

def require_uses(job_value, job_name, expected)
  abort_contract("#{job_name} has no active uses: #{expected}") unless active_uses(job_value, job_name).include?(expected)
end

def require_checkout_depth(job_value, job_name)
  checkout = steps(job_value, job_name).find { |step| step["uses"] == "actions/checkout@v5" }
  abort_contract("#{job_name} has no active actions/checkout@v5 step") unless checkout

  with = checkout["with"]
  abort_contract("#{job_name} checkout does not set fetch-depth: 0") unless with.is_a?(Hash) && with["fetch-depth"] == 0
end

release_path, ci_path = ARGV
abort_contract("usage: workflow_contract.rb RELEASE_WORKFLOW CI_WORKFLOW") unless release_path && ci_path && ARGV.length == 2

release_jobs = load_workflow(release_path)
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

require_uses(build, "build", "actions/checkout@v5")
require_uses(build, "build", "actions/upload-artifact@v7")
require_uses(publish, "publish", "actions/checkout@v5")
require_uses(publish, "publish", "actions/download-artifact@v8")
require_uses(publish, "publish", "softprops/action-gh-release@v3")

matrix = build.dig("strategy", "matrix", "include")
abort_contract("build matrix include is not a sequence") unless matrix.is_a?(Array)
targets = matrix.each_with_object([]) do |entry, values|
  values << entry["target"] if entry.is_a?(Hash) && entry["target"]
end.to_set
expected_targets = Set[
  "x86_64-unknown-linux-gnu",
  "aarch64-unknown-linux-gnu",
  "x86_64-apple-darwin",
  "aarch64-apple-darwin"
]
abort_contract("build matrix targets differ from the four release targets") unless targets == expected_targets && matrix.length == 4

require_run_text(build, "build", 'archive="questmancer-v${version}-${target}.tar.gz"')
require_run_text(build, "build", 'tar -C "$staging" -czf "$archive" questmancer')
require_run_text(publish, "publish", "sha256sum \"${expected[@]}\" >SHA256SUMS")

ci_jobs = load_workflow(ci_path)
check = job(ci_jobs, "check")
require_checkout_depth(check, "check")
[
  "cargo fmt --all --check",
  "cargo clippy --all-targets --all-features -- -D warnings",
  "cargo test --all-targets --all-features",
  "bash tests/scripts.sh",
  "bash -n herdr/install.sh herdr/run.sh herdr/control.sh",
  "cargo build --release",
  'git diff --check "${base_sha}"...HEAD'
].each { |command| require_run_line(check, "check", command) }

puts "workflow contracts: valid"
